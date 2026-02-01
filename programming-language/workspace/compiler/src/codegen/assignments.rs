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

//! Assignment code generation.
//!
//! This module provides code generation for assignments:
//! - Simple assignments (=)
//! - Compound assignments (+=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=)
//! - Array element assignments

use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{AssignOp, AssignTarget, Assignment, Type};
use crate::error::{CompileError, ErrorCode};

/// Extension trait for assignment code generation.
pub trait AssignmentEmitter {
    /// Generate code for an assignment.
    ///
    /// Handles both simple and compound assignments, as well as array element assignments.
    fn generate_assignment(&mut self, assign: &Assignment) -> Result<(), CompileError>;
}

impl AssignmentEmitter for CodeGenerator {
    fn generate_assignment(&mut self, assign: &Assignment) -> Result<(), CompileError> {
        match &assign.target {
            AssignTarget::Variable(name) => {
                let var = self.get_variable(name).ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        assign.span.clone(),
                    )
                })?;

                match assign.op {
                    AssignOp::Assign => {
                        // Use type-aware expression generation for proper literal conversion
                        self.generate_expression_for_type(&assign.value, &var.var_type)?;
                    }
                    AssignOp::AddAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::CLC);
                        self.emit_byte(opcodes::ADC_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::SubAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::SEC);
                        self.emit_byte(opcodes::SBC_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::MulAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::TAX);
                        self.generate_expression(&assign.value)?;
                        self.emit_jsr_label("__mul_byte");
                    }
                    AssignOp::DivAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        self.emit_jsr_label("__div_byte");
                    }
                    AssignOp::ModAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        self.emit_jsr_label("__div_byte");
                        self.emit_byte(opcodes::TXA); // Remainder is in X
                    }
                    AssignOp::BitAndAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::AND_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::BitOrAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::ORA_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::BitXorAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::EOR_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::ShiftLeftAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        // Shift left X times
                        let loop_label = self.make_label("shl_loop");
                        let done_label = self.make_label("shl_done");
                        self.define_label(&loop_label);
                        self.emit_byte(opcodes::CPX_IMM);
                        self.emit_byte(0);
                        self.emit_branch(opcodes::BEQ, &done_label);
                        self.emit_byte(opcodes::ASL_ACC);
                        self.emit_byte(opcodes::DEX);
                        self.emit_jmp(&loop_label);
                        self.define_label(&done_label);
                    }
                    AssignOp::ShiftRightAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        // Shift right X times
                        let loop_label = self.make_label("shr_loop");
                        let done_label = self.make_label("shr_done");
                        self.define_label(&loop_label);
                        self.emit_byte(opcodes::CPX_IMM);
                        self.emit_byte(0);
                        self.emit_branch(opcodes::BEQ, &done_label);
                        self.emit_byte(opcodes::LSR_ACC);
                        self.emit_byte(opcodes::DEX);
                        self.emit_jmp(&loop_label);
                        self.define_label(&done_label);
                    }
                }

                self.emit_store_to_address(var.address, &var.var_type);
            }
            AssignTarget::ArrayElement { name, index } => {
                // For array assignment, we need to calculate the address
                let var = self.get_variable(name).ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined array '{}'", name),
                        assign.span.clone(),
                    )
                })?;

                let is_word_array =
                    matches!(var.var_type, Type::WordArray(_) | Type::SwordArray(_));

                // Only simple assignment is supported for arrays currently
                if assign.op != AssignOp::Assign {
                    return Err(CompileError::new(
                        ErrorCode::NotImplemented,
                        "Compound assignment operators on array elements not yet supported",
                        assign.span.clone(),
                    ));
                }

                // Generate value first and save it
                self.generate_expression(&assign.value)?;
                if is_word_array {
                    // Save both bytes (A=low, X=high)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP2_HI);
                } else {
                    self.emit_byte(opcodes::PHA); // Save 8-bit value
                }

                // Generate index
                self.generate_expression(index)?;
                if is_word_array {
                    // Multiply index by 2 for word arrays
                    self.emit_byte(opcodes::ASL_ACC);
                }
                self.emit_byte(opcodes::TAY); // Y = index (or index*2)

                // Load base address into TMP1
                self.emit_imm(opcodes::LDA_IMM, (var.address & 0xFF) as u8);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_imm(opcodes::LDA_IMM, (var.address >> 8) as u8);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1_HI);

                if is_word_array {
                    // Store 16-bit value
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::INY);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP2_HI);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                } else {
                    // Store 8-bit value
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
        }

        Ok(())
    }
}
