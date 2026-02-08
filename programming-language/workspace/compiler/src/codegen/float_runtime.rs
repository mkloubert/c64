// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Float runtime library for IEEE-754 binary16 (half-precision) floating-point.
//!
//! This module provides 6510 assembly routines for float operations.
//!
//! ## IEEE-754 binary16 Format
//!
//! ```text
//! Bit:  15 | 14-10 | 9-0
//!       S  |  Exp  | Mantissa
//! ```
//!
//! - Sign (S): 1 bit, 0 = positive, 1 = negative
//! - Exponent: 5 bits, bias = 15 (actual = stored - 15)
//! - Mantissa: 10 bits, implicit leading 1 for normalized numbers
//!
//! ## Special Values
//!
//! - Zero: exponent = 0, mantissa = 0 (signed zero: +0 and -0)
//! - Denormal: exponent = 0, mantissa != 0 (very small numbers)
//! - Infinity: exponent = 31, mantissa = 0
//! - NaN: exponent = 31, mantissa != 0
//!
//! ## Calling Convention
//!
//! - Input operand 1: FP_ARG1 (TMP1/TMP1_HI) - zero page $22/$23
//! - Input operand 2: FP_ARG2 (TMP3/TMP3_HI) - zero page $02/$03
//! - Output: A (low byte), X (high byte)
//! - Preserved: Y register when possible
//!
//! ## Zero Page Usage
//!
//! - $02-$03: FP_ARG2 (second operand / temp)
//! - $04-$05: FP_WORK (working space)
//! - $22-$23: FP_ARG1 (first operand)
//! - $FD-$FE: Additional temp space

use super::emit::EmitHelpers;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::CodeGenerator;

/// Zero page locations for float operations.
pub mod fp_zeropage {
    /// First float argument (low byte).
    pub const FP_ARG1_LO: u8 = 0x22;
    /// First float argument (high byte).
    pub const FP_ARG1_HI: u8 = 0x23;
    /// Second float argument (low byte).
    pub const FP_ARG2_LO: u8 = 0x02;
    /// Second float argument (high byte).
    pub const FP_ARG2_HI: u8 = 0x03;
    /// Working space (low byte).
    pub const FP_WORK_LO: u8 = 0x04;
    /// Working space (high byte).
    pub const FP_WORK_HI: u8 = 0x05;
    /// Exponent 1.
    pub const FP_EXP1: u8 = 0xFD;
    /// Exponent 2.
    pub const FP_EXP2: u8 = 0xFE;
}

impl CodeGenerator {
    /// Emit all float runtime routines.
    pub fn emit_float_runtime(&mut self) {
        self.emit_float_add_routine();
        self.emit_float_sub_routine();
        self.emit_float_mul_routine();
        self.emit_float_div_routine();
        self.emit_float_mod_routine();
        self.emit_float_neg_routine();
        self.emit_float_cmp_routines();
        self.emit_float_conversion_routines();
    }

    /// Emit float addition routine.
    ///
    /// Input: FP_ARG1 (TMP1/TMP1_HI), FP_ARG2 (TMP3/TMP3_HI)
    /// Output: A/X (low/high)
    ///
    /// Algorithm:
    /// 1. Handle special cases (zero, infinity, NaN)
    /// 2. Extract sign, exponent, mantissa for both operands
    /// 3. Align mantissas by shifting the smaller exponent's mantissa right
    /// 4. Add or subtract mantissas based on signs
    /// 5. Normalize result
    /// 6. Handle overflow (→ infinity) and underflow (→ zero)
    ///
    /// IEEE-754 binary16: sign(1) + exp(5) + mantissa(10)
    /// Mantissa has implicit leading 1 for normalized numbers.
    fn emit_float_add_routine(&mut self) {
        self.define_label("__float_add");
        self.runtime_addresses
            .insert("float_add".to_string(), self.current_address);

        // Zero page usage for this routine:
        // FP_EXP1 ($FD): exponent of operand 1 / result exponent
        // FP_EXP2 ($FE): exponent of operand 2
        // FP_WORK_LO ($04): mantissa 1 low / result mantissa low
        // FP_WORK_HI ($05): mantissa 1 high / result mantissa high / result sign
        // TMP2 ($FB): mantissa 2 low
        // TMP2+1 ($FC): mantissa 2 high
        // We'll use $06/$07 for extended mantissa during alignment

        let arg1_zero = self.make_label("fadd_a1z");
        let arg2_zero = self.make_label("fadd_a2z");
        let do_add = self.make_label("fadd_do");
        let done = self.make_label("fadd_done");

        // Check if ARG1 is zero (exponent and mantissa both 0)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F); // Mask out sign bit
        self.emit_branch(opcodes::BEQ, &arg1_zero);

        // Check if ARG2 is zero
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_branch(opcodes::BEQ, &arg2_zero);

        // Both non-zero, perform addition
        self.emit_jmp(&do_add);

        // ARG1 is zero, return ARG2
        self.define_label(&arg1_zero);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_jmp(&done);

        // ARG2 is zero, return ARG1
        self.define_label(&arg2_zero);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp(&done);

        // Perform the actual addition
        self.define_label(&do_add);

        // Extract exponents: exp = (high_byte >> 2) & 0x1F
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);

        // Extract mantissas with implicit leading 1
        // Mantissa = (high & 0x03) | 0x04 for high byte (adds implicit 1)
        // Plus low byte for full 11-bit mantissa

        // Mantissa 1 -> FP_WORK_LO/HI
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03); // mantissa high 2 bits
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04); // add implicit 1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Mantissa 2 -> TMP2/TMP2+1 (using $FB/$FC)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2 + 1);

        // Extract and save signs
        // Sign 1 in bit 7 of $06, Sign 2 in bit 6 of $06
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06); // sign1 in bit 7

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::LSR_ACC); // move to bit 6
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06); // signs in bits 7 and 6

        // Compare exponents and align mantissas
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);

        let exp_equal = self.make_label("fadd_exp_eq");
        let exp1_bigger = self.make_label("fadd_e1_big");
        let do_align = self.make_label("fadd_align");

        self.emit_branch(opcodes::BEQ, &exp_equal);
        self.emit_branch(opcodes::BCS, &exp1_bigger);

        // EXP2 > EXP1: shift mantissa1 right, use EXP2 as result exponent
        // Shift amount = EXP2 - EXP1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::TAY); // Y = shift count

        // Update result exponent to the larger one
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // If shift > 11, mantissa1 becomes 0 (too small to matter)
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(12);
        let skip_shift1 = self.make_label("fadd_skip1");
        self.emit_branch(opcodes::BCS, &skip_shift1);

        // Shift mantissa1 right by Y bits
        let shift1_loop = self.make_label("fadd_sh1");
        self.define_label(&shift1_loop);
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &do_align);
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp(&shift1_loop);

        self.define_label(&skip_shift1);
        // Mantissa1 is too small, set to 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_jmp(&do_align);

        // EXP1 > EXP2: shift mantissa2 right, keep EXP1 as result exponent
        self.define_label(&exp1_bigger);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);
        self.emit_byte(opcodes::TAY); // Y = shift count

        // If shift > 11, mantissa2 becomes 0
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(12);
        let skip_shift2 = self.make_label("fadd_skip2");
        self.emit_branch(opcodes::BCS, &skip_shift2);

        // Shift mantissa2 right by Y bits
        let shift2_loop = self.make_label("fadd_sh2");
        self.define_label(&shift2_loop);
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &do_align);
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp(&shift2_loop);

        self.define_label(&skip_shift2);
        // Mantissa2 is too small, set to 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_jmp(&do_align);

        // Exponents are equal, no alignment needed
        self.define_label(&exp_equal);

        // Add or subtract mantissas based on signs
        self.define_label(&do_align);

        // Check if signs are the same (bits 7 and 6 of $06)
        // If both same (00xx xxxx or 11xx xxxx), add. If different, subtract.
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0xC0); // isolate sign bits
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x40); // only bit 6 set = signs differ (+ and -)
        let signs_differ1 = self.make_label("fadd_sdiff");
        self.emit_branch(opcodes::BEQ, &signs_differ1);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80); // only bit 7 set = signs differ (- and +)
        self.emit_branch(opcodes::BEQ, &signs_differ1);

        // Same signs: add mantissas
        let do_add_mant = self.make_label("fadd_addm");
        self.emit_jmp(&do_add_mant);

        // Different signs: subtract mantissas (smaller from larger)
        self.define_label(&signs_differ1);

        // Compare mantissas to determine which is larger
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        let m1_bigger = self.make_label("fadd_m1big");
        let m2_bigger = self.make_label("fadd_m2big");
        self.emit_branch(opcodes::BCC, &m2_bigger);
        self.emit_branch(opcodes::BNE, &m1_bigger);

        // High bytes equal, check low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, &m2_bigger);
        // Fall through to m1_bigger (or equal)

        // Mantissa1 >= Mantissa2: result = M1 - M2, sign = sign1
        self.define_label(&m1_bigger);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        // Sign = sign1 (bit 7 of $06), already there
        let normalize = self.make_label("fadd_norm");
        self.emit_jmp(&normalize);

        // Mantissa2 > Mantissa1: result = M2 - M1, sign = sign2
        self.define_label(&m2_bigger);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        // Sign = sign2: copy bit 6 to bit 7 of $06
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::ASL_ACC); // bit 6 -> bit 7
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06);
        self.emit_jmp(&normalize);

        // Same signs: add mantissas
        self.define_label(&do_add_mant);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Check for carry (overflow in mantissa addition)
        let no_carry = self.make_label("fadd_nocar");
        self.emit_branch(opcodes::BCC, &no_carry);

        // Carry occurred: shift right and increment exponent
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Check for exponent overflow
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        let no_exp_overflow = self.make_label("fadd_noexp");
        self.emit_branch(opcodes::BCC, &no_exp_overflow);

        // Exponent overflow: return infinity with correct sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80); // get sign
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x7C); // infinity exponent
        self.emit_byte(opcodes::TAX);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done);

        self.define_label(&no_exp_overflow);
        self.define_label(&no_carry);

        // Normalize result: shift left until bit 2 of high byte is set (implicit 1 position)
        // For 11-bit mantissa with implicit 1, we need bit 2 set
        self.define_label(&normalize);

        // Check if mantissa is zero (result of subtraction of equal values)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        let not_zero = self.make_label("fadd_nz");
        self.emit_branch(opcodes::BNE, &not_zero);

        // Result is zero
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_jmp(&done);

        self.define_label(&not_zero);

        // Normalize loop: shift left until bit 2 of high byte is set
        let norm_loop = self.make_label("fadd_nloop");
        let norm_done = self.make_label("fadd_ndone");

        self.define_label(&norm_loop);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x04); // check if bit 2 is set
        self.emit_branch(opcodes::BNE, &norm_done);

        // Shift mantissa left
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Decrement exponent
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Check for exponent underflow
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_branch(opcodes::BNE, &norm_loop);

        // Exponent underflow: return zero
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_jmp(&done);

        self.define_label(&norm_done);

        // Pack result: sign | (exp << 2) | (mantissa_hi & 0x03)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // temp store exp << 2

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03); // mantissa high 2 bits (remove implicit 1)
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(0x06); // add sign
        self.emit_byte(opcodes::TAX); // X = high byte

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO); // A = low byte

        self.define_label(&done);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float subtraction routine.
    ///
    /// Negates the second operand and calls float_add.
    fn emit_float_sub_routine(&mut self) {
        self.define_label("__float_sub");
        self.runtime_addresses
            .insert("float_sub".to_string(), self.current_address);

        // Negate ARG2 by flipping sign bit
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0x80); // Flip sign bit
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Call float_add
        self.emit_jsr_label("__float_add");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float multiplication routine.
    ///
    /// Algorithm:
    /// 1. XOR signs for result sign
    /// 2. Add exponents, subtract bias (15)
    /// 3. Multiply mantissas (11x11 -> 22 bit, take top 11)
    /// 4. Normalize and round
    ///
    /// For 11-bit mantissas (with implicit 1), we multiply to get 22 bits,
    /// then take the top 11 bits as the result mantissa.
    fn emit_float_mul_routine(&mut self) {
        self.define_label("__float_mul");
        self.runtime_addresses
            .insert("float_mul".to_string(), self.current_address);

        let done = self.make_label("fmul_done");
        let return_zero = self.make_label("fmul_zero");

        // Check for zero operands
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_branch(opcodes::BEQ, &return_zero);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_branch(opcodes::BEQ, &return_zero);

        // Calculate result sign (XOR of both signs)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::EOR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06); // Store result sign in $06

        // Extract and add exponents
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);

        // Add exponents and subtract bias (15)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(15); // Subtract bias

        // Check for underflow (result < 0)
        self.emit_branch(opcodes::BMI, &return_zero);

        // Check for overflow (result >= 31)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        let no_overflow = self.make_label("fmul_no_ovf");
        self.emit_branch(opcodes::BCC, &no_overflow);

        // Overflow - return infinity with correct sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x7C); // Infinity exponent (31 << 2)
        self.emit_byte(opcodes::TAX);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_jmp(&done);

        self.define_label(&no_overflow);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1); // Store result exponent

        // Extract mantissas with implicit 1
        // Mantissa 1 -> FP_WORK_LO/HI (11 bits: low 8 in LO, high 3 in HI)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04); // add implicit 1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Mantissa 2 -> TMP2/TMP2+1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2 + 1);

        // Perform 11x11 bit multiplication using shift-and-add
        // Result will be 22 bits in $07/$08/$09 (low to high)
        // Initialize result to 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x07); // result low
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x08); // result mid
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x09); // result high

        // Loop 11 times (11-bit multiplier)
        self.emit_imm(opcodes::LDY_IMM, 11);

        let mul_loop = self.make_label("fmul_loop");
        let mul_no_add = self.make_label("fmul_noadd");
        let mul_shift = self.make_label("fmul_shift");

        self.define_label(&mul_loop);
        // Check LSB of multiplier (mantissa 2)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01);
        self.emit_branch(opcodes::BEQ, &mul_no_add);

        // Add multiplicand (mantissa 1) to result
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x09);

        self.define_label(&mul_no_add);
        // Shift multiplicand left
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Shift multiplier right
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP2);

        self.define_label(&mul_shift);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &mul_loop);

        // Result is in $07/$08/$09 (22 bits)
        // For normalized result, bit 21 (the product of two implicit 1s) should be set
        // We need to extract bits 21-11 for the result mantissa

        // Check if bit 5 of $09 is set (bit 21 of 22-bit result)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x20); // bit 5
        let no_extra_shift = self.make_label("fmul_noex");
        self.emit_branch(opcodes::BNE, &no_extra_shift);

        // Not set, shift left once and decrement exponent
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Check for exponent underflow
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_branch(opcodes::BNE, &no_extra_shift);
        // Underflow to zero
        self.emit_jmp(&return_zero);

        self.define_label(&no_extra_shift);

        // Extract result mantissa from bits 21-11 of the 22-bit product
        // Bit 21 is the implicit 1, bits 20-11 are the mantissa
        // $09 has bits 21-16, $08 has bits 15-8, $07 has bits 7-0
        // Result mantissa high 2 bits = bits 20-19 = ($09 >> 3) & 0x03
        // Result mantissa low 8 bits = bits 18-11 = (($09 & 0x07) << 5) | ($08 >> 3)

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03); // mantissa high 2 bits
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Get low 8 bits of mantissa
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        // Pack result: sign | (exp << 2) | mantissa_hi
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(0x06); // Add sign
        self.emit_byte(opcodes::TAX);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_jmp(&done);

        // Return zero
        self.define_label(&return_zero);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);

        self.define_label(&done);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float division routine.
    ///
    /// Algorithm:
    /// 1. Handle special cases (zero dividend, zero divisor)
    /// 2. XOR signs for result sign
    /// 3. Subtract exponents, add bias (15)
    /// 4. Divide mantissas (11-bit / 11-bit)
    /// 5. Normalize result
    ///
    /// Division uses restoring division algorithm.
    fn emit_float_div_routine(&mut self) {
        self.define_label("__float_div");
        self.runtime_addresses
            .insert("float_div".to_string(), self.current_address);

        let done = self.make_label("fdiv_done");
        let return_zero = self.make_label("fdiv_zero");
        let return_inf = self.make_label("fdiv_inf");

        // Check for zero dividend
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_branch(opcodes::BEQ, &return_zero);

        // Check for zero divisor (return infinity)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_branch(opcodes::BEQ, &return_inf);

        // Calculate result sign (XOR of both signs)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::EOR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06); // Store result sign

        // Extract exponents
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);

        // Subtract exponents and add bias
        // result_exp = exp1 - exp2 + 15
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(15);

        // Check bounds
        self.emit_branch(opcodes::BMI, &return_zero);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        self.emit_branch(opcodes::BCS, &return_inf);

        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Extract mantissas with implicit 1
        // Dividend (mantissa 1) -> FP_WORK_LO/HI
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Divisor (mantissa 2) -> TMP2/TMP2+1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2 + 1);

        // Perform 11-bit division using restoring division
        // Quotient will be built up in $07/$08 (result mantissa)
        // Remainder in FP_WORK_LO/HI (we extend with $09 for extra bits)

        // Initialize quotient to 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x09); // extension for remainder

        // We need 12 iterations to get 11 bits of quotient (one extra for normalization)
        self.emit_imm(opcodes::LDY_IMM, 12);

        let div_loop = self.make_label("fdiv_loop");
        let div_no_sub = self.make_label("fdiv_nosub");

        self.define_label(&div_loop);

        // Shift dividend (remainder) left by 1
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x09);

        // Try to subtract divisor from remainder
        // Compare first: if remainder >= divisor, subtract and set quotient bit
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_branch(opcodes::BCC, &div_no_sub); // remainder < divisor
        self.emit_branch(opcodes::BNE, "__fdiv_do_sub"); // remainder > divisor

        // High bytes equal, compare low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, &div_no_sub);

        // Subtract divisor from remainder
        self.define_label("__fdiv_do_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x09);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2 + 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x09);

        // Set quotient bit (shift 1 into quotient)
        self.emit_byte(opcodes::SEC);
        let div_shift_q = self.make_label("fdiv_shq");
        self.emit_jmp(&div_shift_q);

        self.define_label(&div_no_sub);
        // Don't set quotient bit (shift 0 into quotient)
        self.emit_byte(opcodes::CLC);

        self.define_label(&div_shift_q);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x08);

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &div_loop);

        // Quotient is in $07/$08 (12 bits, we need top 11)
        // Check if we need to normalize (bit 11 should be set for implicit 1)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x08); // bit 3 of $08 = bit 11 of quotient
        let div_normalized = self.make_label("fdiv_norm");
        self.emit_branch(opcodes::BNE, &div_normalized);

        // Not normalized, shift left and decrement exponent
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Check for underflow
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_branch(opcodes::BNE, &div_normalized);
        self.emit_jmp(&return_zero);

        self.define_label(&div_normalized);

        // Extract result mantissa from quotient
        // Quotient has bit 11 as implicit 1, bits 10-1 as mantissa (we ignore bit 0)
        // Result mantissa high 2 bits = (quotient >> 9) & 0x03
        // Result mantissa low 8 bits = (quotient >> 1) & 0xFF

        // Get high 2 bits of mantissa
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::LSR_ACC); // bit 11 -> bit 10, bits 10-9 -> bits 9-8
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Get low 8 bits of mantissa
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x08);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01); // bit 8 of quotient
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        // Pack result: sign | (exp << 2) | mantissa_hi
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(0x06); // Add sign
        self.emit_byte(opcodes::TAX);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        self.emit_jmp(&done);

        // Return zero
        self.define_label(&return_zero);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_jmp(&done);

        // Return infinity
        self.define_label(&return_inf);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x7C);
        self.emit_byte(opcodes::TAX);
        self.emit_imm(opcodes::LDA_IMM, 0);

        self.define_label(&done);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float modulo routine.
    ///
    /// Computes fmod(a, b) = a - trunc(a/b) * b
    /// For simplicity, this uses the fact that for IEEE floats:
    /// fmod is essentially the remainder after integer division.
    fn emit_float_mod_routine(&mut self) {
        self.define_label("__float_mod");
        self.runtime_addresses
            .insert("float_mod".to_string(), self.current_address);

        // For a simplified implementation:
        // We'll compute a - (a/b)*b where we truncate the division
        // This is complex in assembly, so we use a loop-based approach:
        // While |a| >= |b|: a = a - |b| (with correct sign handling)

        let done = self.make_label("fmod_done");
        let loop_label = self.make_label("fmod_loop");
        let return_zero = self.make_label("fmod_zero");

        // Check for zero divisor -> return original value
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F); // Ignore sign
        self.emit_branch(opcodes::BEQ, &return_zero);

        // Save original sign of a
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Make both operands positive for comparison
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Loop: while a >= b, subtract b from a
        self.define_label(&loop_label);

        // Compare a with b (unsigned comparison of absolute values)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BCC, &done.clone()); // a < b, done
        self.emit_branch(opcodes::BNE, "__fmod_sub"); // a_hi > b_hi, subtract

        // High bytes equal, compare low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCC, &done); // a < b

        // a >= b, subtract
        self.define_label("__fmod_sub");
        self.emit_jsr_label("__float_sub"); // Result in A/X
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        // Clear sign for next comparison
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp(&loop_label);

        // Return zero
        self.define_label(&return_zero);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        // Done - restore sign and return
        self.define_label(&done);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float negation routine.
    ///
    /// Simply flips the sign bit.
    fn emit_float_neg_routine(&mut self) {
        self.define_label("__float_neg");
        self.runtime_addresses
            .insert("float_neg".to_string(), self.current_address);

        // Input in A/X, flip sign bit in X
        self.emit_byte(opcodes::PHA);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float comparison routines.
    fn emit_float_cmp_routines(&mut self) {
        // Less than
        self.define_label("__float_cmp_lt");
        self.runtime_addresses
            .insert("float_cmp_lt".to_string(), self.current_address);

        let lt_true = self.make_label("fclt_true");
        let lt_false = self.make_label("fclt_false");
        let lt_done = self.make_label("fclt_done");

        // Compare signs first
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::EOR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BMI, &lt_true.clone()); // Different signs

        // Same sign - compare magnitudes
        // For positive: smaller magnitude = less
        // For negative: larger magnitude = less (more negative)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BMI, "__fclt_neg");

        // Both positive: compare as unsigned
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BCC, &lt_true.clone());
        self.emit_branch(opcodes::BNE, &lt_false.clone());
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCC, &lt_true.clone());
        self.emit_jmp(&lt_false);

        // Both negative: compare reversed
        self.define_label("__fclt_neg");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BCC, &lt_true.clone());
        self.emit_branch(opcodes::BNE, &lt_false.clone());
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, &lt_true);
        self.emit_jmp(&lt_false);

        self.define_label(&lt_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_jmp(&lt_done);

        self.define_label(&lt_false);
        self.emit_imm(opcodes::LDA_IMM, 0);

        self.define_label(&lt_done);
        self.emit_byte(opcodes::RTS);

        // Less than or equal
        self.define_label("__float_cmp_le");
        self.runtime_addresses
            .insert("float_cmp_le".to_string(), self.current_address);

        // Check equality first
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        let le_check_lt = self.make_label("fcle_lt");
        self.emit_branch(opcodes::BNE, &le_check_lt);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BNE, &le_check_lt);
        self.emit_imm(opcodes::LDA_IMM, 1); // Equal
        self.emit_byte(opcodes::RTS);

        self.define_label(&le_check_lt);
        self.emit_jsr_label("__float_cmp_lt");
        self.emit_byte(opcodes::RTS);

        // Greater than
        self.define_label("__float_cmp_gt");
        self.runtime_addresses
            .insert("float_cmp_gt".to_string(), self.current_address);

        self.emit_jsr_label("__float_cmp_le");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(1);
        self.emit_byte(opcodes::RTS);

        // Greater than or equal
        self.define_label("__float_cmp_ge");
        self.runtime_addresses
            .insert("float_cmp_ge".to_string(), self.current_address);

        self.emit_jsr_label("__float_cmp_lt");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(1);
        self.emit_byte(opcodes::RTS);

        // Equality
        self.define_label("__float_cmp_eq");
        self.runtime_addresses
            .insert("float_cmp_eq".to_string(), self.current_address);

        let eq_true = self.make_label("fceq_true");
        let eq_done = self.make_label("fceq_done");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BNE, &eq_done.clone());
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BNE, &eq_done);

        self.define_label(&eq_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_byte(opcodes::RTS);

        self.define_label(&eq_done);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::RTS);

        // Not equal
        self.define_label("__float_cmp_ne");
        self.runtime_addresses
            .insert("float_cmp_ne".to_string(), self.current_address);

        self.emit_jsr_label("__float_cmp_eq");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float conversion routines.
    fn emit_float_conversion_routines(&mut self) {
        self.emit_byte_to_float_routine();
        self.emit_word_to_float_routine();
        self.emit_float_to_byte_routine();
        self.emit_float_to_word_routine();
        self.emit_fixed_to_float_routine();
        self.emit_float_to_fixed_routine();
    }

    /// Convert unsigned byte to float.
    ///
    /// Input: A = byte value
    /// Output: A/X = float (binary16)
    fn emit_byte_to_float_routine(&mut self) {
        self.define_label("__byte_to_float");
        self.runtime_addresses
            .insert("byte_to_float".to_string(), self.current_address);

        let done = self.make_label("b2f_done");
        let normalize = self.make_label("b2f_norm");

        // Check for zero
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, &normalize);

        // Return +0.0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&normalize);
        // Store value and find leading bit position
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::LDX_IMM, 0); // mantissa high bits

        // Count leading zeros and shift to normalize
        // For byte: if bit 7 set, exp = 22 (7 + 15)
        // Each shift left decreases exponent by 1
        self.emit_imm(opcodes::LDY_IMM, 22); // Start with exp for bit 7

        let shift_loop = self.make_label("b2f_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BNE, &done.clone()); // Found leading 1

        // Shift left
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::DEY);
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, &shift_loop);

        // Value was zero (shouldn't reach here)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&done);
        // Y = exponent, TMP1 = normalized mantissa (bit 7 = implicit 1)
        // Remove implicit 1, shift to get 10-bit mantissa
        // Mantissa = (TMP1 & 0x7F) << 2, taking top 10 bits

        // Pack result: sign=0, exp=Y, mantissa from TMP1
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC); // exp << 2 in bits 6-2
        self.emit_byte(opcodes::TAX); // X = exp << 2

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F); // Remove implicit 1
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC); // Top 4 bits of mantissa -> bits 3-0 of A

        // Combine high byte: exp in bits 6-2, mantissa in bits 1-0
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TAX); // X = high byte

        // Low byte: remaining mantissa bits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC); // Low 5 bits shifted to high position

        self.emit_byte(opcodes::RTS);
    }

    /// Convert unsigned word to float.
    ///
    /// Input: A/X = word value (A=low, X=high)
    /// Output: A/X = float (binary16)
    fn emit_word_to_float_routine(&mut self) {
        self.define_label("__word_to_float");
        self.runtime_addresses
            .insert("word_to_float".to_string(), self.current_address);

        let done = self.make_label("w2f_done");
        let check_low = self.make_label("w2f_low");

        // Store value
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check for zero
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BNE, &check_low);

        // Return +0.0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&check_low);
        // Find leading bit position in 16-bit value
        // Start with exponent for bit 15 = 30 (15 + 15)
        self.emit_imm(opcodes::LDY_IMM, 30);

        let shift_loop = self.make_label("w2f_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BNE, &done.clone()); // Found leading 1

        // Shift left 16-bit
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, &shift_loop);

        // Value was zero
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&done);
        // Y = exponent, TMP1/TMP1_HI = normalized value
        // Pack result: sign=0, exp=Y, mantissa from TMP1_HI:TMP1

        // High byte of float: 0 | exp[4:0] << 2 | mantissa[9:8]
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // exp << 2

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x60); // bits 6-5 (mantissa bits 9-8 after removing implicit 1)
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TAX); // X = high byte

        // Low byte of float: mantissa[7:0]
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F); // bits 4-0
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::RTS);
    }

    /// Convert float to byte (truncate).
    ///
    /// Input: A/X = float (binary16)
    /// Output: A = byte value
    fn emit_float_to_byte_routine(&mut self) {
        self.define_label("__float_to_byte");
        self.runtime_addresses
            .insert("float_to_byte".to_string(), self.current_address);

        let done = self.make_label("f2b_done");

        // Store float
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Extract exponent
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);

        // Check if exponent < 15 (value < 1.0)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(15);
        self.emit_branch(opcodes::BCS, "__f2b_valid");

        // Value < 1.0, return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2b_valid");
        // exponent in A, calculate shift = exponent - 15
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(15);
        self.emit_byte(opcodes::TAY); // Y = shift amount

        // Check if shift > 7 (overflow for byte)
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(8);
        self.emit_branch(opcodes::BCC, "__f2b_ok");

        // Overflow, return 255
        self.emit_imm(opcodes::LDA_IMM, 255);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2b_ok");
        // Get mantissa with implicit 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03); // mantissa high 2 bits
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04); // add implicit 1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Shift mantissa by Y positions
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &done);

        let shift_loop = self.make_label("f2b_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        self.define_label(&done);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::RTS);
    }

    /// Convert float to word (truncate).
    ///
    /// Input: A/X = float (binary16)
    /// Output: A/X = word value
    ///
    /// Float16: sign(1) + exp(5) + mantissa(10)
    /// Value = (1 + mantissa/1024) × 2^(exp - 15)
    /// Integer = (1024 + mantissa) × 2^(exp - 25)
    ///
    /// For exp < 25: shift RIGHT by (25 - exp)
    /// For exp >= 25: shift LEFT by (exp - 25)
    fn emit_float_to_word_routine(&mut self) {
        self.define_label("__float_to_word");
        self.runtime_addresses
            .insert("float_to_word".to_string(), self.current_address);

        // Store float
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Extract exponent: (high >> 2) & 0x1F
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = exponent

        // Check if exponent < 15 (value < 1.0)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(15);
        self.emit_branch(opcodes::BCS, "__f2w_valid");

        // Value < 1.0, return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2w_valid");
        // Get mantissa with implicit 1 (11 bits: 0x400 + mantissa)
        // mantissa low 8 bits in TMP1, high 2 bits in TMP1_HI & 0x03
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04); // add implicit 1 (0x400 >> 8 = 0x04)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI); // TMP3_HI:TMP3 = 11-bit mantissa

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Calculate shift: if exp < 25, shift right by (25 - exp)
        //                  if exp >= 25, shift left by (exp - 25)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2); // exp
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(25);
        self.emit_branch(opcodes::BCS, "__f2w_shift_left");

        // exp < 25: shift RIGHT by (25 - exp)
        self.emit_byte(opcodes::LDA_IMM);
        self.emit_byte(25);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::TAY); // Y = shift count

        // Shift right loop
        self.define_label("__f2w_shr_loop");
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, "__f2w_done");
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__f2w_shr_loop");

        // exp >= 25: shift LEFT by (exp - 25)
        self.define_label("__f2w_shift_left");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(25);
        self.emit_byte(opcodes::TAY); // Y = shift count

        // Check overflow (shift > 5 would overflow 16-bit result)
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(6);
        self.emit_branch(opcodes::BCC, "__f2w_shl_loop");

        // Overflow, return 65535
        self.emit_imm(opcodes::LDA_IMM, 0xFF);
        self.emit_imm(opcodes::LDX_IMM, 0xFF);
        self.emit_byte(opcodes::RTS);

        // Shift left loop
        self.define_label("__f2w_shl_loop");
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, "__f2w_done");
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__f2w_shl_loop");

        self.define_label("__f2w_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::RTS);
    }

    /// Convert fixed-point 12.4 to float.
    ///
    /// Input: A/X = fixed (12.4 format)
    /// Output: A/X = float (binary16)
    fn emit_fixed_to_float_routine(&mut self) {
        self.define_label("__fixed_to_float");
        self.runtime_addresses
            .insert("fixed_to_float".to_string(), self.current_address);

        let done = self.make_label("fx2f_done");
        let positive = self.make_label("fx2f_pos");

        // Store fixed value
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check for zero
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BNE, &positive);

        // Return +0.0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&positive);
        // Save sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI); // Save sign

        // Make positive if negative
        self.emit_branch(opcodes::BPL, "__fx2f_abs");

        // Negate (two's complement)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__fx2f_abs");
        // Now have absolute value in TMP1/TMP1_HI
        // Fixed point 12.4: value = raw / 16
        // For float: exponent = 15 + (bit_position - 4)

        // Find leading bit (like word_to_float but adjusted for 12.4)
        self.emit_imm(opcodes::LDY_IMM, 26); // Start exponent for bit 15: 15 + 11 = 26

        let shift_loop = self.make_label("fx2f_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BNE, &done.clone());

        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, &shift_loop);

        // Zero
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label(&done);
        // Pack float: sign from FP_WORK_HI, exp from Y, mantissa from TMP1_HI:TMP1
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Get mantissa high bits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x60);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI); // Add sign
        self.emit_byte(opcodes::TAX);

        // Low byte
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::RTS);
    }

    /// Convert float to fixed-point 12.4.
    ///
    /// Input: A/X = float (binary16)
    /// Output: A/X = fixed (12.4 format)
    fn emit_float_to_fixed_routine(&mut self) {
        self.define_label("__float_to_fixed");
        self.runtime_addresses
            .insert("float_to_fixed".to_string(), self.current_address);

        let done = self.make_label("f2fx_done");

        // Store float
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Save sign
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

        // Extract exponent
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);

        // Check if exponent < 11 (value < 1/16, too small for 12.4)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(11);
        self.emit_branch(opcodes::BCS, "__f2fx_valid");

        // Too small, return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2fx_valid");
        // Check if exponent > 26 (value >= 2048, overflow for 12.4)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(27);
        self.emit_branch(opcodes::BCC, "__f2fx_ok");

        // Overflow, return max (or min if negative)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_branch(opcodes::BMI, "__f2fx_min");

        // Return +max = 0x7FFF
        self.emit_imm(opcodes::LDA_IMM, 0xFF);
        self.emit_imm(opcodes::LDX_IMM, 0x7F);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2fx_min");
        // Return -max = 0x8000
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_imm(opcodes::LDX_IMM, 0x80);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2fx_ok");
        // Calculate shift amount = exponent - 11
        // (11 = bias for 12.4 format where 1.0 = 16)
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(11);
        self.emit_byte(opcodes::TAY); // Y = shift amount

        // Get mantissa with implicit 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Shift left by Y
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &done);

        let shift_loop = self.make_label("f2fx_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        self.define_label(&done);
        // Apply sign if negative
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_branch(opcodes::BPL, "__f2fx_ret");

        // Negate result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.define_label("__f2fx_ret");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::RTS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fp_zeropage_constants() {
        // Verify zero page locations don't overlap
        assert_ne!(fp_zeropage::FP_ARG1_LO, fp_zeropage::FP_ARG2_LO);
        assert_ne!(fp_zeropage::FP_ARG1_HI, fp_zeropage::FP_ARG2_HI);
        assert_ne!(fp_zeropage::FP_WORK_LO, fp_zeropage::FP_ARG1_LO);
    }
}
