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

//! Binary operation code generation.
//!
//! This module provides code generation for binary operations:
//! - Arithmetic: Add, Sub, Mul, Div, Mod
//! - Bitwise: And, Or, Xor, ShiftLeft, ShiftRight
//! - Comparison: Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual
//! - Logical: And, Or
//!
//! Supports:
//! - 8-bit integer operations (byte/sbyte)
//! - 16-bit integer operations (word/sword)
//! - Fixed-point operations (12.4 format)
//! - Float operations (IEEE-754 binary16)

use super::comparisons::ComparisonHelpers;
use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::type_inference::TypeInference;
use super::CodeGenerator;
use crate::ast::{BinaryOp, Expr};
use crate::error::{CompileError, ErrorCode};

/// Extension trait for binary operation code generation.
pub trait BinaryOpsEmitter {
    /// Generate code for a binary operation.
    ///
    /// Evaluates both operands, then applies the operation.
    /// Result is left in A (or A/X for 16-bit types).
    fn generate_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError>;

    /// Generate code for a fixed-point binary operation (16-bit).
    ///
    /// Fixed-point uses 12.4 format (12 bits integer, 4 bits fraction).
    /// Internal representation is value × 16 stored as signed 16-bit.
    fn generate_fixed_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError>;

    /// Generate code for a float binary operation (IEEE-754 binary16).
    ///
    /// Float uses IEEE-754 half-precision format (1 sign + 5 exp + 10 mantissa).
    fn generate_float_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError>;
}

impl BinaryOpsEmitter for CodeGenerator {
    fn generate_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Determine types
        let left_type = self.infer_type_from_expr(left);
        let right_type = self.infer_type_from_expr(right);
        let use_signed = self.is_signed_type(&left_type) || self.is_signed_type(&right_type);
        let use_fixed = self.is_fixed_type(&left_type) || self.is_fixed_type(&right_type);
        let use_float = self.is_float_type(&left_type) || self.is_float_type(&right_type);

        // Use 16-bit float operations if either operand is float
        if use_float {
            return self.generate_float_binary_op(left, op, right);
        }

        // Use 16-bit fixed-point operations if either operand is fixed
        if use_fixed {
            return self.generate_fixed_binary_op(left, op, right);
        }

        // Generate left operand
        self.generate_expression(left)?;
        self.emit_byte(opcodes::PHA); // Save left on stack

        // Generate right operand
        self.generate_expression(right)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Right in TMP1

        // Restore left to A
        self.emit_byte(opcodes::PLA);

        match op {
            BinaryOp::Add => {
                // Addition is the same for signed and unsigned (two's complement)
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Sub => {
                // Subtraction is the same for signed and unsigned (two's complement)
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Mul => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__mul_sbyte");
                } else {
                    self.emit_jsr_label("__mul_byte");
                }
            }
            BinaryOp::Div => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__div_sbyte");
                } else {
                    self.emit_jsr_label("__div_byte");
                }
            }
            BinaryOp::Mod => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__div_sbyte");
                } else {
                    self.emit_jsr_label("__div_byte");
                }
                self.emit_byte(opcodes::TXA); // Remainder is in X
            }
            BinaryOp::BitAnd => {
                self.emit_byte(opcodes::AND_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::BitOr => {
                self.emit_byte(opcodes::ORA_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::BitXor => {
                self.emit_byte(opcodes::EOR_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::ShiftLeft => {
                // Shift left by TMP1 times
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                let loop_label = self.make_label("shl");
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
            BinaryOp::ShiftRight => {
                // Shift right by TMP1 times
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                let loop_label = self.make_label("shr");
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
            BinaryOp::Equal => {
                // Equality is the same for signed and unsigned
                self.emit_byte(opcodes::CMP_ZP);
                self.emit_byte(zeropage::TMP1);
                let eq_label = self.make_label("eq");
                let done_label = self.make_label("eq_done");
                self.emit_branch(opcodes::BEQ, &eq_label);
                self.emit_imm(opcodes::LDA_IMM, 0); // Not equal
                self.emit_jmp(&done_label);
                self.define_label(&eq_label);
                self.emit_imm(opcodes::LDA_IMM, 1); // Equal
                self.define_label(&done_label);
            }
            BinaryOp::NotEqual => {
                // Inequality is the same for signed and unsigned
                self.emit_byte(opcodes::CMP_ZP);
                self.emit_byte(zeropage::TMP1);
                let ne_label = self.make_label("ne");
                let done_label = self.make_label("ne_done");
                self.emit_branch(opcodes::BNE, &ne_label);
                self.emit_imm(opcodes::LDA_IMM, 0); // Equal
                self.emit_jmp(&done_label);
                self.define_label(&ne_label);
                self.emit_imm(opcodes::LDA_IMM, 1); // Not equal
                self.define_label(&done_label);
            }
            BinaryOp::Less => {
                if use_signed {
                    // Signed A < TMP1: use SEC+SBC to set V flag, then check N XOR V
                    self.emit_signed_less_than();
                } else {
                    // Unsigned: A < TMP1 when carry clear after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let lt_label = self.make_label("lt");
                    let done_label = self.make_label("lt_done");
                    self.emit_branch(opcodes::BCC, &lt_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&lt_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::LessEqual => {
                if use_signed {
                    // Signed A <= TMP1: A < TMP1 OR A == TMP1
                    self.emit_signed_less_equal();
                } else {
                    // Unsigned: A <= TMP1 when carry clear OR zero after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let le_label = self.make_label("le");
                    let done_label = self.make_label("le_done");
                    self.emit_branch(opcodes::BCC, &le_label);
                    self.emit_branch(opcodes::BEQ, &le_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&le_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::Greater => {
                if use_signed {
                    // Signed A > TMP1: NOT (A <= TMP1)
                    self.emit_signed_greater_than();
                } else {
                    // Unsigned: A > TMP1 when carry set and not equal
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let gt_label = self.make_label("gt");
                    let done_label = self.make_label("gt_done");
                    self.emit_branch(opcodes::BEQ, &done_label.clone()); // Equal, not greater
                    self.emit_branch(opcodes::BCS, &gt_label); // Carry set = greater or equal
                                                               // Carry clear means less than
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&gt_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::GreaterEqual => {
                if use_signed {
                    // Signed A >= TMP1: NOT (A < TMP1)
                    self.emit_signed_greater_equal();
                } else {
                    // Unsigned: A >= TMP1 when carry set after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let ge_label = self.make_label("ge");
                    let done_label = self.make_label("ge_done");
                    self.emit_branch(opcodes::BCS, &ge_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&ge_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::And => {
                // Logical AND
                let false_label = self.make_label("and_false");
                let done_label = self.make_label("and_done");
                // Left is already in A
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &false_label);
                // Check right (in TMP1)
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &false_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.emit_jmp(&done_label);
                self.define_label(&false_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.define_label(&done_label);
            }
            BinaryOp::Or => {
                // Logical OR
                let true_label = self.make_label("or_true");
                let done_label = self.make_label("or_done");
                // Left is already in A
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BNE, &true_label);
                // Check right (in TMP1)
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BNE, &true_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_jmp(&done_label);
                self.define_label(&true_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.define_label(&done_label);
            }
        }

        Ok(())
    }

    fn generate_fixed_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Generate left operand (result in A=low, X=high)
        self.generate_expression(left)?;

        // Save left in FP_LEFT (TMP3/TMP3_HI)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Generate right operand
        self.generate_expression(right)?;

        // Right in A=low, X=high, save to TMP1/TMP1_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        match op {
            BinaryOp::Add => {
                // 16-bit addition: TMP3 + TMP1 -> A/X
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3); // Store result low
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                self.emit_byte(opcodes::TAX); // X = high byte
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3); // A = low byte
            }
            BinaryOp::Sub => {
                // 16-bit subtraction: TMP3 - TMP1 -> A/X
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                self.emit_byte(opcodes::TAX);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
            }
            BinaryOp::Mul => {
                // Fixed-point multiplication:
                // (a×16) × (b×16) = (a×b) × 256
                // Result needs to be shifted right by 4 to get (a×b) × 16
                self.emit_jsr_label("__mul_fixed");
            }
            BinaryOp::Div => {
                // Fixed-point division:
                // ((a×16) << 4) / (b×16) = (a/b) × 16
                self.emit_jsr_label("__div_fixed");
            }
            BinaryOp::Mod => {
                // Fixed-point modulo (same as division but return remainder)
                self.emit_jsr_label("__mod_fixed");
            }
            BinaryOp::Equal => {
                // 16-bit equality: compare both bytes
                self.emit_fixed_comparison(|s| {
                    let eq_label = s.make_label("feq");
                    let done_label = s.make_label("feq_done");
                    // Compare high bytes first
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3_HI);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1_HI);
                    s.emit_branch(opcodes::BNE, &done_label.clone());
                    // Compare low bytes
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1);
                    s.emit_branch(opcodes::BNE, &done_label);
                    // Equal
                    s.emit_imm(opcodes::LDA_IMM, 1);
                    s.emit_jmp(&eq_label);
                    s.define_label(&done_label);
                    s.emit_imm(opcodes::LDA_IMM, 0);
                    s.define_label(&eq_label);
                });
            }
            BinaryOp::NotEqual => {
                self.emit_fixed_comparison(|s| {
                    let ne_label = s.make_label("fne");
                    let done_label = s.make_label("fne_done");
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3_HI);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1_HI);
                    s.emit_branch(opcodes::BNE, &ne_label.clone());
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1);
                    s.emit_branch(opcodes::BNE, &ne_label);
                    s.emit_imm(opcodes::LDA_IMM, 0);
                    s.emit_jmp(&done_label);
                    s.define_label(&ne_label);
                    s.emit_imm(opcodes::LDA_IMM, 1);
                    s.define_label(&done_label);
                });
            }
            BinaryOp::Less => {
                // Signed 16-bit less than
                self.emit_jsr_label("__cmp_fixed_lt");
            }
            BinaryOp::LessEqual => {
                self.emit_jsr_label("__cmp_fixed_le");
            }
            BinaryOp::Greater => {
                self.emit_jsr_label("__cmp_fixed_gt");
            }
            BinaryOp::GreaterEqual => {
                self.emit_jsr_label("__cmp_fixed_ge");
            }
            _ => {
                // Bitwise and logical operations are not supported for fixed-point
                // (should be caught by analyzer)
                return Err(CompileError::new(
                    ErrorCode::InvalidOperatorForType,
                    format!("Operator {:?} is not supported for fixed-point types", op),
                    left.span.clone(),
                ));
            }
        }

        Ok(())
    }

    fn generate_float_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Generate left operand (result in A=low, X=high)
        self.generate_expression(left)?;

        // Save left in FP_ARG1 (TMP1/TMP1_HI for float runtime)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Generate right operand
        self.generate_expression(right)?;

        // Right in A=low, X=high, save to FP_ARG2 (TMP3/TMP3_HI)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        match op {
            BinaryOp::Add => {
                self.emit_jsr_label("__float_add");
            }
            BinaryOp::Sub => {
                self.emit_jsr_label("__float_sub");
            }
            BinaryOp::Mul => {
                self.emit_jsr_label("__float_mul");
            }
            BinaryOp::Div => {
                self.emit_jsr_label("__float_div");
            }
            BinaryOp::Mod => {
                // Float modulo: a - floor(a/b) * b
                // For simplicity, use fmod-like behavior
                self.emit_jsr_label("__float_mod");
            }
            BinaryOp::Equal => {
                self.emit_jsr_label("__float_cmp_eq");
            }
            BinaryOp::NotEqual => {
                self.emit_jsr_label("__float_cmp_ne");
            }
            BinaryOp::Less => {
                self.emit_jsr_label("__float_cmp_lt");
            }
            BinaryOp::LessEqual => {
                self.emit_jsr_label("__float_cmp_le");
            }
            BinaryOp::Greater => {
                self.emit_jsr_label("__float_cmp_gt");
            }
            BinaryOp::GreaterEqual => {
                self.emit_jsr_label("__float_cmp_ge");
            }
            _ => {
                // Bitwise and logical operations are not supported for float
                return Err(CompileError::new(
                    ErrorCode::InvalidOperatorForType,
                    format!("Operator {:?} is not supported for float types", op),
                    left.span.clone(),
                ));
            }
        }

        Ok(())
    }
}
