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

//! Comparison code generation helpers for signed and fixed-point operations.
//!
//! This module provides helper methods for generating 6510 assembly code
//! for signed comparisons (which require special handling on the 6502/6510).

use super::emit::EmitHelpers;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::CodeGenerator;
use crate::ast::Type;

/// Trait for comparison code generation operations.
pub trait ComparisonHelpers {
    /// Check if a type is signed.
    fn is_signed_type(&self, var_type: &Type) -> bool;

    /// Check if a type is fixed-point.
    fn is_fixed_type(&self, var_type: &Type) -> bool;

    /// Check if a type is floating-point.
    fn is_float_type(&self, var_type: &Type) -> bool;

    /// Emit signed less-than comparison.
    /// A < TMP1 (signed): Uses SEC+SBC to set V flag, then checks N XOR V.
    /// Result: A = 1 if true, A = 0 if false.
    fn emit_signed_less_than(&mut self);

    /// Emit signed greater-equal comparison.
    /// A >= TMP1 (signed): NOT (A < TMP1), so check (N XOR V) == 0.
    fn emit_signed_greater_equal(&mut self);

    /// Emit signed less-equal comparison.
    /// A <= TMP1 (signed): A < TMP1 OR A == TMP1.
    fn emit_signed_less_equal(&mut self);

    /// Emit signed greater-than comparison.
    /// A > TMP1 (signed): NOT (A <= TMP1), which is A >= TMP1 AND A != TMP1.
    fn emit_signed_greater_than(&mut self);

    /// Helper for fixed-point comparisons.
    fn emit_fixed_comparison<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self);
}

impl ComparisonHelpers for CodeGenerator {
    fn is_signed_type(&self, var_type: &Type) -> bool {
        matches!(
            var_type,
            Type::Sbyte | Type::Sword | Type::Fixed | Type::Float
        )
    }

    fn is_fixed_type(&self, var_type: &Type) -> bool {
        matches!(var_type, Type::Fixed)
    }

    fn is_float_type(&self, var_type: &Type) -> bool {
        matches!(var_type, Type::Float)
    }

    fn emit_signed_less_than(&mut self) {
        // For signed comparison, we need to check (N XOR V) after subtraction.
        // CMP doesn't set V, so we use SEC+SBC.
        //
        // Algorithm:
        // 1. SEC; SBC TMP1 - this sets N, V, Z, C
        // 2. If V is clear, check N directly
        // 3. If V is set, the sign is inverted, so N=0 means less than
        //
        // In 6502 assembly:
        //   SEC
        //   SBC TMP1
        //   BVC no_overflow
        //   EOR #$80       ; Flip bit 7 if overflow
        // no_overflow:
        //   BMI is_less    ; If N set (after possible flip), A < TMP1
        //   LDA #0
        //   JMP done
        // is_less:
        //   LDA #1
        // done:

        let no_overflow = self.make_label("slt_no_ovf");
        let is_less = self.make_label("slt_less");
        let done_label = self.make_label("slt_done");

        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit if overflow
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BMI, &is_less);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_less);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    fn emit_signed_greater_equal(&mut self) {
        let no_overflow = self.make_label("sge_no_ovf");
        let is_ge = self.make_label("sge_true");
        let done_label = self.make_label("sge_done");

        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit if overflow
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BPL, &is_ge); // If N clear (after possible flip), A >= TMP1
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_ge);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    fn emit_signed_less_equal(&mut self) {
        // First check equality
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        let is_le = self.make_label("sle_true");
        let check_less = self.make_label("sle_check");
        let done_label = self.make_label("sle_done");

        self.emit_branch(opcodes::BEQ, &is_le); // Equal means <=

        // Not equal, check if less than
        // Reload A (CMP doesn't modify A, but we need to do SBC which does)
        // Actually, we still have A from before. Let's do the signed less check.
        self.define_label(&check_less);
        let no_overflow = self.make_label("sle_no_ovf");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BMI, &is_le); // If negative, A < TMP1, so A <= TMP1
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_le);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    fn emit_signed_greater_than(&mut self) {
        // First check equality - if equal, not greater
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        let not_gt = self.make_label("sgt_false");
        let check_ge = self.make_label("sgt_check");
        let is_gt = self.make_label("sgt_true");
        let done_label = self.make_label("sgt_done");

        self.emit_branch(opcodes::BEQ, &not_gt); // Equal means not greater

        // Not equal, check if greater or equal (which means greater since not equal)
        self.define_label(&check_ge);
        let no_overflow = self.make_label("sgt_no_ovf");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BPL, &is_gt); // If positive, A >= TMP1, and since A != TMP1, A > TMP1

        self.define_label(&not_gt);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_gt);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    fn emit_fixed_comparison<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        f(self);
    }
}
