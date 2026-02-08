// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
//
// Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Control flow code generation.
//!
//! This module provides code generation for control flow statements:
//! - if/elif/else statements
//! - while loops
//! - for loops
//! - break and continue statements
//! - return statements

use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::{LabelManager, LoopContext};
use super::mos6510::{opcodes, zeropage};
use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{Expr, ForStatement, IfStatement, Type, WhileStatement};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for control flow code generation.
pub trait ControlFlowEmitter {
    /// Generate code for an if statement.
    fn generate_if(&mut self, if_stmt: &IfStatement) -> Result<(), CompileError>;

    /// Generate code for a while loop.
    fn generate_while(&mut self, while_stmt: &WhileStatement) -> Result<(), CompileError>;

    /// Generate code for a for loop.
    fn generate_for(&mut self, for_stmt: &ForStatement) -> Result<(), CompileError>;

    /// Generate code for break statement.
    fn generate_break(&mut self) -> Result<(), CompileError>;

    /// Generate code for continue statement.
    fn generate_continue(&mut self) -> Result<(), CompileError>;

    /// Generate code for return statement.
    fn generate_return(&mut self, value: Option<&Expr>) -> Result<(), CompileError>;
}

impl ControlFlowEmitter for CodeGenerator {
    fn generate_if(&mut self, if_stmt: &IfStatement) -> Result<(), CompileError> {
        let else_label = self.make_label("else");
        let end_label = self.make_label("endif");

        // Generate condition
        self.generate_expression(&if_stmt.condition)?;

        // Branch if false
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        if if_stmt.elif_branches.is_empty() && if_stmt.else_block.is_none() {
            self.emit_branch(opcodes::BEQ, &end_label);
        } else {
            self.emit_branch(opcodes::BEQ, &else_label);
        }

        // Generate then block
        self.generate_block(&if_stmt.then_block)?;
        if if_stmt.else_block.is_some() || !if_stmt.elif_branches.is_empty() {
            self.emit_jmp(&end_label);
        }

        // Generate elif branches
        // Track the current label - starts with else_label, then becomes next_label for each branch
        let mut current_label = else_label.clone();
        for (i, (cond, block)) in if_stmt.elif_branches.iter().enumerate() {
            self.define_label(&current_label);

            let next_label = if i < if_stmt.elif_branches.len() - 1 || if_stmt.else_block.is_some()
            {
                self.make_label("elif")
            } else {
                end_label.clone()
            };

            self.generate_expression(cond)?;
            self.emit_byte(opcodes::CMP_IMM);
            self.emit_byte(0);
            self.emit_branch(opcodes::BEQ, &next_label);

            self.generate_block(block)?;
            self.emit_jmp(&end_label);

            // The next elif branch will define this label
            current_label = next_label;
        }

        // Generate else block
        if let Some(else_block) = &if_stmt.else_block {
            // If no elif branches, define else_label; otherwise define the last next_label
            self.define_label(&current_label);
            self.generate_block(else_block)?;
        }

        self.define_label(&end_label);

        Ok(())
    }

    fn generate_while(&mut self, while_stmt: &WhileStatement) -> Result<(), CompileError> {
        let start_label = self.make_label("while_start");
        let end_label = self.make_label("while_end");

        // Push loop context
        self.loop_stack.push(LoopContext {
            start_label: start_label.clone(),
            end_label: end_label.clone(),
        });

        self.define_label(&start_label);

        // Generate condition
        self.generate_expression(&while_stmt.condition)?;
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &end_label);

        // Generate body
        self.generate_block(&while_stmt.body)?;

        // Jump back to start
        self.emit_jmp(&start_label);

        self.define_label(&end_label);

        // Pop loop context
        self.loop_stack.pop();

        Ok(())
    }

    fn generate_for(&mut self, for_stmt: &ForStatement) -> Result<(), CompileError> {
        let start_label = self.make_label("for_start");
        let end_label = self.make_label("for_end");

        // Allocate loop variable
        let loop_var_addr = self.allocate_variable(&for_stmt.variable, &Type::Byte, false);

        // Initialize loop variable with start value
        self.generate_expression(&for_stmt.start)?;
        self.emit_store_to_address(loop_var_addr, &Type::Byte);

        // Push loop context
        self.loop_stack.push(LoopContext {
            start_label: start_label.clone(),
            end_label: end_label.clone(),
        });

        self.define_label(&start_label);

        // Check condition (compare with end value)
        self.emit_load_from_address(loop_var_addr, &Type::Byte);
        self.emit_byte(opcodes::PHA);
        self.generate_expression(&for_stmt.end)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::PLA);

        if for_stmt.descending {
            // For downto: exit when loop_var < end
            self.emit_byte(opcodes::CMP_ZP);
            self.emit_byte(zeropage::TMP1);
            self.emit_branch(opcodes::BCC, &end_label);
        } else {
            // For to: exit when loop_var > end
            self.emit_byte(opcodes::CMP_ZP);
            self.emit_byte(zeropage::TMP1);
            self.emit_branch(opcodes::BEQ, "__for_continue");
            self.emit_branch(opcodes::BCS, &end_label);
            self.define_label("__for_continue");
        }

        // Generate body
        self.generate_block(&for_stmt.body)?;

        // Increment or decrement loop variable
        self.emit_load_from_address(loop_var_addr, &Type::Byte);
        if for_stmt.descending {
            self.emit_byte(opcodes::SEC);
            self.emit_imm(opcodes::SBC_IMM, 1);
        } else {
            self.emit_byte(opcodes::CLC);
            self.emit_imm(opcodes::ADC_IMM, 1);
        }
        self.emit_store_to_address(loop_var_addr, &Type::Byte);

        // Jump back to start
        self.emit_jmp(&start_label);

        self.define_label(&end_label);

        // Pop loop context
        self.loop_stack.pop();

        Ok(())
    }

    fn generate_break(&mut self) -> Result<(), CompileError> {
        if let Some(ctx) = self.loop_stack.last() {
            let end_label = ctx.end_label.clone();
            self.emit_jmp(&end_label);
            Ok(())
        } else {
            Err(CompileError::new(
                ErrorCode::BreakOutsideLoop,
                "break outside of loop",
                Span::new(0, 0),
            ))
        }
    }

    fn generate_continue(&mut self) -> Result<(), CompileError> {
        if let Some(ctx) = self.loop_stack.last() {
            let start_label = ctx.start_label.clone();
            self.emit_jmp(&start_label);
            Ok(())
        } else {
            Err(CompileError::new(
                ErrorCode::ContinueOutsideLoop,
                "continue outside of loop",
                Span::new(0, 0),
            ))
        }
    }

    fn generate_return(&mut self, value: Option<&Expr>) -> Result<(), CompileError> {
        if let Some(expr) = value {
            self.generate_expression(expr)?;
        }
        self.emit_byte(opcodes::RTS);
        Ok(())
    }
}
