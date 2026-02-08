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
                self.generate_array_element_assignment(assign, name, index)?;
            }
        }

        Ok(())
    }
}

/// Helper methods for array element assignments.
impl CodeGenerator {
    /// Generate code for array element assignment (simple and compound).
    fn generate_array_element_assignment(
        &mut self,
        assign: &Assignment,
        name: &str,
        index: &crate::ast::Expr,
    ) -> Result<(), CompileError> {
        let var = self.get_variable(name).ok_or_else(|| {
            CompileError::new(
                ErrorCode::UndefinedVariable,
                format!("Undefined array '{}'", name),
                assign.span.clone(),
            )
        })?;

        let is_word_array = matches!(var.var_type, Type::WordArray(_) | Type::SwordArray(_));
        let var_address = var.address;

        match assign.op {
            AssignOp::Assign => {
                // Simple assignment - generate value first, then index
                self.generate_expression(&assign.value)?;
                if is_word_array {
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP2_HI);
                } else {
                    self.emit_byte(opcodes::PHA);
                }

                self.generate_expression(index)?;
                if is_word_array {
                    self.emit_byte(opcodes::ASL_ACC);
                }
                self.emit_byte(opcodes::TAY);

                self.emit_array_base_address(var_address);

                if is_word_array {
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
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            _ => {
                // Compound assignment - need to load current value first
                if is_word_array {
                    self.generate_word_array_compound_assignment(
                        assign,
                        index,
                        var_address,
                    )?;
                } else {
                    self.generate_byte_array_compound_assignment(
                        assign,
                        index,
                        var_address,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Emit base address loading into TMP1/TMP1_HI.
    fn emit_array_base_address(&mut self, address: u16) {
        self.emit_imm(opcodes::LDA_IMM, (address & 0xFF) as u8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::LDA_IMM, (address >> 8) as u8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
    }

    /// Generate compound assignment for byte arrays.
    fn generate_byte_array_compound_assignment(
        &mut self,
        assign: &Assignment,
        index: &crate::ast::Expr,
        var_address: u16,
    ) -> Result<(), CompileError> {
        // Step 1: Calculate index and store in Y, save to TMP4
        self.generate_expression(index)?;
        self.emit_byte(opcodes::TAY);
        self.emit_byte(opcodes::STY_ZP);
        self.emit_byte(zeropage::TMP4);

        // Step 2: Load base address into TMP1
        self.emit_array_base_address(var_address);

        // Step 3: Load current array element value into TMP3
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Step 4: Generate RHS expression and store in TMP2
        self.generate_expression(&assign.value)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        // Step 5: Restore Y register
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP4);

        // Step 6: Perform the compound operation
        match assign.op {
            AssignOp::AddAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP2);
            }
            AssignOp::SubAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP2);
            }
            AssignOp::MulAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::TAX);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_jsr_label("__mul_byte");
            }
            AssignOp::DivAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::TAX);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_jsr_label("__div_byte");
            }
            AssignOp::ModAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::TAX);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_jsr_label("__div_byte");
                self.emit_byte(opcodes::TXA); // Remainder is in X
            }
            AssignOp::BitAndAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::AND_ZP);
                self.emit_byte(zeropage::TMP2);
            }
            AssignOp::BitOrAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::ORA_ZP);
                self.emit_byte(zeropage::TMP2);
            }
            AssignOp::BitXorAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::EOR_ZP);
                self.emit_byte(zeropage::TMP2);
            }
            AssignOp::ShiftLeftAssign => {
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2);
                let loop_label = self.make_label("arr_shl_loop");
                let done_label = self.make_label("arr_shl_done");
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
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2);
                let loop_label = self.make_label("arr_shr_loop");
                let done_label = self.make_label("arr_shr_done");
                self.define_label(&loop_label);
                self.emit_byte(opcodes::CPX_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &done_label);
                self.emit_byte(opcodes::LSR_ACC);
                self.emit_byte(opcodes::DEX);
                self.emit_jmp(&loop_label);
                self.define_label(&done_label);
            }
            AssignOp::Assign => unreachable!(),
        }

        // Step 7: Restore Y and store result back to array element
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP1);

        Ok(())
    }

    /// Generate compound assignment for word arrays.
    fn generate_word_array_compound_assignment(
        &mut self,
        assign: &Assignment,
        index: &crate::ast::Expr,
        var_address: u16,
    ) -> Result<(), CompileError> {
        // Step 1: Calculate index*2 and store in Y, save to TMP4
        self.generate_expression(index)?;
        self.emit_byte(opcodes::ASL_ACC); // index * 2 for word arrays
        self.emit_byte(opcodes::TAY);
        self.emit_byte(opcodes::STY_ZP);
        self.emit_byte(zeropage::TMP4);

        // Step 2: Load base address into TMP1
        self.emit_array_base_address(var_address);

        // Step 3: Load current 16-bit array element value into TMP3/TMP3_HI
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Step 4: Generate RHS expression and store in TMP2/TMP2_HI
        self.generate_expression(&assign.value)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Step 5: Restore Y register
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP4);

        // Step 6: Perform the compound operation (16-bit)
        match assign.op {
            AssignOp::AddAssign => {
                // 16-bit addition
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::SubAssign => {
                // 16-bit subtraction
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::MulAssign => {
                // 16-bit multiplication via runtime function
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::TXA);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_jsr_label("__mul_word");
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::DivAssign => {
                // 16-bit division via runtime function
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::TXA);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_jsr_label("__div_word");
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::ModAssign => {
                // 16-bit modulo via runtime function
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::TXA);
                self.emit_byte(opcodes::PHA);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_jsr_label("__mod_word");
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::BitAndAssign => {
                // 16-bit AND
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::AND_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::AND_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::BitOrAssign => {
                // 16-bit OR
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::ORA_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::ORA_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::BitXorAssign => {
                // 16-bit XOR
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::EOR_ZP);
                self.emit_byte(zeropage::TMP2);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::EOR_ZP);
                self.emit_byte(zeropage::TMP2_HI);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
            }
            AssignOp::ShiftLeftAssign => {
                // 16-bit shift left
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2);
                let loop_label = self.make_label("warr_shl_loop");
                let done_label = self.make_label("warr_shl_done");
                self.define_label(&loop_label);
                self.emit_byte(opcodes::CPX_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &done_label);
                self.emit_byte(opcodes::ASL_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::ROL_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::DEX);
                self.emit_jmp(&loop_label);
                self.define_label(&done_label);
            }
            AssignOp::ShiftRightAssign => {
                // 16-bit shift right
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP2);
                let loop_label = self.make_label("warr_shr_loop");
                let done_label = self.make_label("warr_shr_done");
                self.define_label(&loop_label);
                self.emit_byte(opcodes::CPX_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &done_label);
                self.emit_byte(opcodes::LSR_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::ROR_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::DEX);
                self.emit_jmp(&loop_label);
                self.define_label(&done_label);
            }
            AssignOp::Assign => unreachable!(),
        }

        // Step 7: Restore Y and store 16-bit result back to array element
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP1);

        Ok(())
    }
}
