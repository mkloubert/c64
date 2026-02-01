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

//! Unary operation code generation.
//!
//! This module provides code generation for unary operations:
//! - Negate (two's complement for integers, sign flip for floats)
//! - Logical NOT
//! - Bitwise NOT

use super::comparisons::ComparisonHelpers;
use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::type_inference::TypeInference;
use super::CodeGenerator;
use crate::ast::{Expr, UnaryOp};
use crate::error::CompileError;

/// Extension trait for unary operation code generation.
pub trait UnaryOpsEmitter {
    /// Generate code for a unary operation.
    ///
    /// The operand is evaluated first, leaving its value in A (or A/X for 16-bit).
    /// Then the unary operation is applied.
    fn generate_unary_op(&mut self, op: UnaryOp, operand: &Expr) -> Result<(), CompileError>;
}

impl UnaryOpsEmitter for CodeGenerator {
    fn generate_unary_op(&mut self, op: UnaryOp, operand: &Expr) -> Result<(), CompileError> {
        let operand_type = self.infer_type_from_expr(operand);
        let is_fixed = self.is_fixed_type(&operand_type);
        let is_float = self.is_float_type(&operand_type);

        self.generate_expression(operand)?;

        match op {
            UnaryOp::Negate => {
                if is_float {
                    // Float negation: flip sign bit (bit 15)
                    // Result is in A (low) and X (high)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::TXA);
                    self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit
                    self.emit_byte(opcodes::TAX);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                } else if is_fixed {
                    // 16-bit two's complement negation
                    // Result is in A (low) and X (high)
                    // Negate: NOT both bytes, then add 1
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // NOT low byte
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);

                    // NOT high byte
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // Add 1 (16-bit)
                    self.emit_byte(opcodes::CLC);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::ADC_IMM, 1);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_imm(opcodes::ADC_IMM, 0);
                    self.emit_byte(opcodes::TAX);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                } else {
                    // 8-bit two's complement negation: EOR #$FF, CLC, ADC #1
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::CLC);
                    self.emit_imm(opcodes::ADC_IMM, 1);
                }
            }
            UnaryOp::Not => {
                // Logical NOT: if A == 0 then 1, else 0
                let zero_label = self.make_label("not_zero");
                let done_label = self.make_label("not_done");
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &zero_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_jmp(&done_label);
                self.define_label(&zero_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.define_label(&done_label);
            }
            UnaryOp::BitNot => {
                // Bitwise NOT
                self.emit_imm(opcodes::EOR_IMM, 0xFF);
            }
        }

        Ok(())
    }
}
