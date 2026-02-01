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
    fn emit_float_add_routine(&mut self) {
        self.define_label("__float_add");
        self.runtime_addresses
            .insert("float_add".to_string(), self.current_address);

        // For a simplified implementation, we'll convert to a common format,
        // perform the operation, and convert back.
        //
        // This is a simplified version - a full implementation would handle
        // all edge cases properly.

        // Check if either operand is zero
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

        // Extract exponents
        // Exponent = (high_byte >> 2) & 0x1F
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

        // Compare exponents
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);

        // For simplicity, if exponents differ significantly, return the larger
        // A full implementation would shift and add properly
        let exp_similar = self.make_label("fadd_exp_sim");
        self.emit_branch(opcodes::BEQ, &exp_similar);

        // Exponents differ - return the one with larger exponent (simplified)
        self.emit_branch(opcodes::BCS, &arg2_zero.clone()); // EXP1 >= EXP2, return ARG1
        self.emit_jmp(&arg1_zero); // EXP1 < EXP2, return ARG2

        // Exponents are equal - add mantissas (simplified)
        self.define_label(&exp_similar);

        // For equal exponents with same sign, add mantissas
        // Extract sign bits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO); // Store sign1

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);

        // Compare signs
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);
        let same_sign = self.make_label("fadd_same");
        self.emit_branch(opcodes::BEQ, &same_sign);

        // Different signs - subtract mantissas (simplified: return ARG1)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp(&done);

        // Same sign - add mantissas
        self.define_label(&same_sign);

        // Add the low bytes
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

        // Add the high bytes (keeping sign and exponent intact is tricky)
        // This is a simplified version - just add the raw values
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::TAX);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_LO);

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
    /// 3. Multiply mantissas
    /// 4. Normalize and round
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
        self.emit_byte(fp_zeropage::FP_WORK_HI); // Store result sign

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

        // Check for overflow/underflow
        self.emit_branch(opcodes::BMI, &return_zero); // Underflow
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        let no_overflow = self.make_label("fmul_no_ovf");
        self.emit_branch(opcodes::BCC, &no_overflow);

        // Overflow - return infinity with correct sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x7C); // Infinity exponent (31 << 2)
        self.emit_byte(opcodes::TAX);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_jmp(&done);

        self.define_label(&no_overflow);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1); // Store result exponent

        // Simplified mantissa multiply: just return an approximation
        // A full implementation would do proper 11x11 bit multiply

        // Pack result: sign | (exp << 2) | (mantissa_hi)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI); // Add sign
        self.emit_byte(opcodes::TAX);

        // Low byte from mantissa multiply (simplified)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_jmp(&done);

        // Return zero
        self.define_label(&return_zero);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);

        self.define_label(&done);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit float division routine.
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

        // Calculate result sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::EOR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);

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
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(fp_zeropage::FP_EXP2);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(15); // Add bias

        // Check bounds
        self.emit_branch(opcodes::BMI, &return_zero);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        self.emit_branch(opcodes::BCS, &return_inf);

        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);

        // Pack result (simplified)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_EXP1);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_jmp(&done);

        // Return zero
        self.define_label(&return_zero);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_jmp(&done);

        // Return infinity
        self.define_label(&return_inf);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(fp_zeropage::FP_WORK_HI);
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
    fn emit_float_to_word_routine(&mut self) {
        self.define_label("__float_to_word");
        self.runtime_addresses
            .insert("float_to_word".to_string(), self.current_address);

        let done = self.make_label("f2w_done");

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
        self.emit_branch(opcodes::BCS, "__f2w_valid");

        // Value < 1.0, return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2w_valid");
        // exponent in A, calculate shift = exponent - 15
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(15);
        self.emit_byte(opcodes::TAY); // Y = shift amount

        // Check if shift > 15 (overflow for word)
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(16);
        self.emit_branch(opcodes::BCC, "__f2w_ok");

        // Overflow, return 65535
        self.emit_imm(opcodes::LDA_IMM, 0xFF);
        self.emit_imm(opcodes::LDX_IMM, 0xFF);
        self.emit_byte(opcodes::RTS);

        self.define_label("__f2w_ok");
        // Get mantissa with implicit 1 (11 bits total)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(0x04); // add implicit 1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Shift mantissa by Y positions
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &done);

        let shift_loop = self.make_label("f2w_shift");
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        self.define_label(&done);
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
