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

//! Runtime library routines for the C64.
//!
//! This module contains the machine code routines that are included in every
//! compiled program to provide runtime support for:
//! - Printing values (byte, word, bool, signed types, fixed-point, float)
//! - Arithmetic operations (multiply, divide)
//! - Input handling (readln)
//!
//! These routines are emitted once and called via JSR from generated code.

use super::emit::EmitHelpers;
use super::labels::LabelManager;
use super::mos6510::{kernal, opcodes, zeropage};
use super::CodeGenerator;

/// Extension trait for runtime library emission.
///
/// This trait provides methods for emitting the C64 runtime library routines.
/// It is implemented for `CodeGenerator` and separates runtime code generation
/// from the main code generator logic.
pub trait RuntimeEmitter {
    /// Emit the complete runtime library.
    fn emit_runtime_library(&mut self);

    // Print routines
    fn emit_print_string_routine(&mut self);
    fn emit_print_byte_routine(&mut self);
    fn emit_print_word_routine(&mut self);
    fn emit_print_bool_routine(&mut self);
    fn emit_print_sbyte_routine(&mut self);
    fn emit_print_sword_routine(&mut self);
    fn emit_print_fixed_routine(&mut self);
    fn emit_print_float_routine(&mut self);

    // Arithmetic routines
    fn emit_multiply_byte_routine(&mut self);
    fn emit_multiply_word_routine(&mut self);
    fn emit_multiply_sbyte_routine(&mut self);
    fn emit_multiply_sword_routine(&mut self);
    fn emit_divide_byte_routine(&mut self);
    fn emit_divide_word_routine(&mut self);
    fn emit_divide_sbyte_routine(&mut self);
    fn emit_divide_sword_routine(&mut self);

    // Fixed-point routines
    fn emit_fixed_multiply_routine(&mut self);
    fn emit_fixed_divide_routine(&mut self);
    fn emit_fixed_modulo_routine(&mut self);
    fn emit_fixed_comparison_routines(&mut self);

    // Input routines
    fn emit_readln_routine(&mut self);
}

impl RuntimeEmitter for CodeGenerator {
    /// Emit the runtime library.
    fn emit_runtime_library(&mut self) {
        if self.runtime_included {
            return;
        }
        self.runtime_included = true;

        // Print routines
        self.emit_print_string_routine();
        self.emit_print_byte_routine();
        self.emit_print_word_routine();
        self.emit_print_bool_routine();
        self.emit_print_sbyte_routine();
        self.emit_print_sword_routine();
        self.emit_print_fixed_routine();
        self.emit_print_float_routine();

        // Multiply routines
        self.emit_multiply_byte_routine();
        self.emit_multiply_word_routine();
        self.emit_multiply_sbyte_routine();
        self.emit_multiply_sword_routine();

        // Divide routines
        self.emit_divide_byte_routine();
        self.emit_divide_word_routine();
        self.emit_divide_sbyte_routine();
        self.emit_divide_sword_routine();

        // Fixed-point routines
        self.emit_fixed_multiply_routine();
        self.emit_fixed_divide_routine();
        self.emit_fixed_modulo_routine();
        self.emit_fixed_comparison_routines();

        // Float runtime routines
        self.emit_float_runtime();

        // Input routine
        self.emit_readln_routine();
    }

    /// Emit print string routine.
    /// Input: String address in TMP1/TMP1_HI
    fn emit_print_string_routine(&mut self) {
        self.define_label("__print_str");
        self.runtime_addresses
            .insert("print_str".to_string(), self.current_address);

        // LDY #0
        self.emit_imm(opcodes::LDY_IMM, 0);

        // Loop: LDA (TMP1),Y
        self.define_label("__print_str_loop");
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);

        // BEQ done
        self.emit_branch(opcodes::BEQ, "__print_str_done");

        // JSR CHROUT
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // INY
        self.emit_byte(opcodes::INY);

        // JMP loop
        self.emit_jmp("__print_str_loop");

        // Done
        self.define_label("__print_str_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print byte routine.
    /// Input: A = byte value (0-255)
    /// Prints the decimal value without leading zeros.
    fn emit_print_byte_routine(&mut self) {
        self.define_label("__print_byte");
        self.runtime_addresses
            .insert("print_byte".to_string(), self.current_address);

        // Divide by 100 for hundreds digit
        self.emit_imm(opcodes::LDX_IMM, 100);
        self.emit_jsr_label("__div_byte_ax");
        // A = hundreds, X = remainder
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3); // Save remainder
        self.emit_byte(opcodes::TAY); // Y = hundreds

        // Divide remainder by 10
        self.emit_byte(opcodes::TXA);
        self.emit_imm(opcodes::LDX_IMM, 10);
        self.emit_jsr_label("__div_byte_ax");
        // A = tens, X = ones
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // TMP3 = tens
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP4); // TMP4 = ones
                                        // Y still has hundreds

        // Print hundreds if != 0
        self.emit_byte(opcodes::TYA);
        self.emit_branch(opcodes::BEQ, "__pb_check_tens");
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        // After printing hundreds, always print tens
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_jmp("__pb_print_tens_digit");

        // Check tens
        self.define_label("__pb_check_tens");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pb_print_ones");

        self.define_label("__pb_print_tens_digit");
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Print ones (always)
        self.define_label("__pb_print_ones");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit print word routine.
    /// Input: A = low byte, X = high byte of 16-bit value
    /// Prints the decimal value (0-65535) without leading zeros.
    fn emit_print_word_routine(&mut self) {
        self.define_label("__print_word");
        self.runtime_addresses
            .insert("print_word".to_string(), self.current_address);

        // Store the 16-bit value in TMP1 (low) and TMP1_HI (high)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // If high byte is 0, just print as byte
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__pw_full");
        self.emit_jsr_label("__print_byte");
        self.emit_byte(opcodes::RTS);

        // Full 16-bit printing needed
        self.define_label("__pw_full");

        // Use TMP3 as "started printing" flag (0 = no digits yet)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Divide by 10000 (0x2710)
        self.emit_imm(opcodes::LDA_IMM, 0x10); // low byte of 10000
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x27); // high byte of 10000
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 1000 (0x03E8)
        self.emit_imm(opcodes::LDA_IMM, 0xE8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x03);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 100 (0x0064)
        self.emit_imm(opcodes::LDA_IMM, 0x64);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 10 (0x000A)
        self.emit_imm(opcodes::LDA_IMM, 0x0A);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Print final digit (ones place, always print)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);

        // Subroutine: divide TMP1/TMP1_HI by TMP2/TMP2_HI, print digit if non-zero or started
        self.define_label("__pw_digit");
        self.emit_imm(opcodes::LDX_IMM, 0); // X = quotient digit

        self.define_label("__pw_digit_loop");
        // Compare TMP1/TMP1_HI >= TMP2/TMP2_HI
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_branch(opcodes::BCC, "__pw_digit_done"); // high < divisor high, done
        self.emit_branch(opcodes::BNE, "__pw_digit_sub"); // high > divisor high, subtract

        // High bytes equal, compare low bytes
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__pw_digit_done"); // low < divisor low, done

        // Subtract divisor from value
        self.define_label("__pw_digit_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Increment quotient
        self.emit_byte(opcodes::INX);
        self.emit_jmp("__pw_digit_loop");

        self.define_label("__pw_digit_done");
        // X = digit value. Print if X != 0 or TMP3 != 0 (already started)
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__pw_digit_print");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pw_digit_skip"); // skip leading zero

        self.define_label("__pw_digit_print");
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        // Mark that we've started printing
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__pw_digit_skip");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print bool routine.
    /// Input: A = bool value (0 = false, non-zero = true)
    /// Prints "TRUE" or "FALSE".
    fn emit_print_bool_routine(&mut self) {
        self.define_label("__print_bool");
        self.runtime_addresses
            .insert("print_bool".to_string(), self.current_address);

        // Check if A is zero (false) or non-zero (true)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, "__pb_false");

        // Print "TRUE"
        self.emit_imm(opcodes::LDA_IMM, b'T');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'R');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'U');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'E');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        // Print "FALSE"
        self.define_label("__pb_false");
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'A');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'L');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'S');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'E');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print signed byte routine.
    /// Input: A = signed byte value (-128 to 127)
    /// Prints the decimal value with minus sign if negative.
    fn emit_print_sbyte_routine(&mut self) {
        self.define_label("__print_sbyte");
        self.runtime_addresses
            .insert("print_sbyte".to_string(), self.current_address);

        // Check if negative (bit 7 set)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BCC, "__psb_positive");

        // Negative: print minus sign and negate
        self.emit_byte(opcodes::PHA); // Save value
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::PLA); // Restore value

        // Negate: two's complement
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);

        self.define_label("__psb_positive");
        // Now A contains the absolute value, print it
        self.emit_jsr_label("__print_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print signed word routine.
    /// Input: A = low byte, X = high byte of signed 16-bit value
    /// Prints the decimal value with minus sign if negative.
    fn emit_print_sword_routine(&mut self) {
        self.define_label("__print_sword");
        self.runtime_addresses
            .insert("print_sword".to_string(), self.current_address);

        // Store value in TMP1/TMP1_HI for later use
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if negative (high byte bit 7 set)
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BCC, "__psw_positive");

        // Negative: print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Negate 16-bit value: two's complement
        // NOT low byte, NOT high byte, then add 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_imm(opcodes::ADC_IMM, 0); // Add carry from low byte
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__psw_positive");
        // Load absolute value and print
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__print_word");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point (12.4) print routine.
    ///
    /// Input: A = low byte, X = high byte (12.4 fixed-point)
    /// Prints the decimal representation, e.g., 60 (internal) -> "3.75"
    fn emit_print_fixed_routine(&mut self) {
        self.define_label("__print_fixed");
        self.runtime_addresses
            .insert("print_fixed".to_string(), self.current_address);

        // Store value
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if negative (bit 15 set)
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BEQ, "__pfix_positive");

        // Print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Negate (two's complement)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_imm(opcodes::ADC_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pfix_positive");
        // Now TMP1/TMP1_HI holds absolute value
        // Integer part = value >> 4
        // Shift right 4 times
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI); // Save shifted high byte

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // 1 shift done

        // 3 more shifts
        for _ in 0..3 {
            self.emit_byte(opcodes::LSR_ZP);
            self.emit_byte(zeropage::TMP3_HI);
            self.emit_byte(opcodes::ROR_ZP);
            self.emit_byte(zeropage::TMP3);
        }

        // Print integer part (TMP3/TMP3_HI)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_jsr_label("__print_word");

        // Print decimal point
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Fractional part = (value & 0x0F) * 625
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F); // Get fractional nibble

        // Store frac in TMP3
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Clear result
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if frac is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pfix_print_frac");

        // Multiply frac * 625
        self.emit_imm(opcodes::LDY_IMM, 4); // 4 bits to check

        self.define_label("__pfix_mul_loop");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01);
        self.emit_branch(opcodes::BEQ, "__pfix_no_add");

        // Add 625 (0x0271)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::ADC_IMM, 0x71);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x02);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pfix_no_add");
        // Shift frac right
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP3);

        // Double the multiplier
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, "__pfix_mul_loop");

        self.define_label("__pfix_print_frac");
        // Print 4 digits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__print_frac4");

        self.emit_byte(opcodes::RTS);

        // Helper routine to print 4 fractional digits with leading zeros
        self.define_label("__print_frac4");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Digit 1: divide by 1000 (0x03E8)
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__pf4_d1");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x03);
        self.emit_branch(opcodes::BCC, "__pf4_d1_done");
        self.emit_branch(opcodes::BNE, "__pf4_d1_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0xE8);
        self.emit_branch(opcodes::BCC, "__pf4_d1_done");

        self.define_label("__pf4_d1_sub");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::SBC_IMM, 0xE8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_imm(opcodes::SBC_IMM, 0x03);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d1");

        self.define_label("__pf4_d1_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 2: divide by 100
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__pf4_d2");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BNE, "__pf4_d2_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(100);
        self.emit_branch(opcodes::BCC, "__pf4_d2_done");

        self.define_label("__pf4_d2_sub");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::SBC_IMM, 100);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_imm(opcodes::SBC_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d2");

        self.define_label("__pf4_d2_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 3: divide by 10
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__pf4_d3");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(10);
        self.emit_branch(opcodes::BCC, "__pf4_d3_done");
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 10);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d3");

        self.define_label("__pf4_d3_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 4: remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit float (IEEE-754 binary16) print routine.
    ///
    /// Input: A = low byte, X = high byte (binary16)
    /// Prints the decimal representation.
    fn emit_print_float_routine(&mut self) {
        self.define_label("__print_float");
        self.runtime_addresses
            .insert("print_float".to_string(), self.current_address);

        // Store value
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
        self.emit_byte(zeropage::TMP3);

        // Check for exponent = 31 (infinity or NaN)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        self.emit_branch(opcodes::BNE, "__pflt_not_special");

        // Check mantissa for NaN vs Infinity
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__pflt_nan");

        // Infinity - check sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BMI, "__pflt_neg_inf");

        // Print "INF"
        self.emit_imm(opcodes::LDA_IMM, b'I');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_neg_inf");
        // Print "-INF"
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'I');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_nan");
        // Print "NAN"
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'A');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_not_special");
        // Check for zero
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BNE, "__pflt_not_zero");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__pflt_subnormal");

        // Zero - check sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BPL, "__pflt_pos_zero");

        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.define_label("__pflt_pos_zero");
        // Print "0.0"
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_subnormal");
        // Print "0.0" for subnormal
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_not_zero");
        // Check sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BPL, "__pflt_print_val");

        // Print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Make positive
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pflt_print_val");
        // Convert to integer and print
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__float_to_word");

        // Print integer part
        self.emit_jsr_label("__print_word");

        // Print ".0"
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 8-bit multiply routine.
    /// Input: A * X
    /// Output: A = low byte, X = high byte
    fn emit_multiply_byte_routine(&mut self) {
        self.define_label("__mul_byte");
        self.runtime_addresses
            .insert("mul_byte".to_string(), self.current_address);

        // Store multiplier
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Store multiplicand
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // Clear result
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Counter
        self.emit_imm(opcodes::LDX_IMM, 8);

        self.define_label("__mul_byte_loop");
        // Shift result left
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Shift multiplier left, check carry
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, "__mul_byte_skip");

        // Add multiplicand
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__mul_byte_skip");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__mul_byte_skip");
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__mul_byte_loop");

        // Result: A = low byte
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 16-bit multiply routine (simplified).
    fn emit_multiply_word_routine(&mut self) {
        self.define_label("__mul_word");
        self.runtime_addresses
            .insert("mul_word".to_string(), self.current_address);

        // Simplified: just do byte multiply for now
        self.emit_jsr_label("__mul_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 8-bit multiply routine.
    fn emit_multiply_sbyte_routine(&mut self) {
        self.define_label("__mul_sbyte");
        self.runtime_addresses
            .insert("mul_sbyte".to_string(), self.current_address);

        // Store operands
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // TMP3 = sign flag
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Check sign of first operand
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BPL, "__msb_first_pos");
        // Negate first operand
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        // Toggle sign flag
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__msb_first_pos");
        // Check sign of second operand
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BPL, "__msb_second_pos");
        // Negate second operand
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        // Toggle sign flag
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__msb_second_pos");
        // Multiply
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jsr_label("__mul_byte");

        // Check sign flag
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__msb_done");
        // Negate result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::RTS);

        self.define_label("__msb_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 16-bit multiply routine (simplified).
    fn emit_multiply_sword_routine(&mut self) {
        self.define_label("__mul_sword");
        self.runtime_addresses
            .insert("mul_sword".to_string(), self.current_address);

        self.emit_jsr_label("__mul_sbyte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit 8-bit divide routine.
    /// Input: A / X
    /// Output: A = quotient, X = remainder
    fn emit_divide_byte_routine(&mut self) {
        self.define_label("__div_byte");
        self.define_label("__div_byte_ax");
        self.runtime_addresses
            .insert("div_byte".to_string(), self.current_address);

        // Check for divide by zero
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__div_byte_start");
        // Divide by zero: return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__div_byte_start");
        // Store dividend and divisor
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // Clear quotient
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Load dividend
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);

        self.define_label("__div_byte_loop");
        // Compare with divisor
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__div_byte_done");

        // Subtract divisor
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);

        // Increment quotient
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_jmp("__div_byte_loop");

        self.define_label("__div_byte_done");
        // A = remainder, load quotient
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 16-bit divide routine (simplified).
    fn emit_divide_word_routine(&mut self) {
        self.define_label("__div_word");
        self.runtime_addresses
            .insert("div_word".to_string(), self.current_address);

        self.emit_jsr_label("__div_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 8-bit divide routine.
    fn emit_divide_sbyte_routine(&mut self) {
        self.define_label("__div_sbyte");
        self.runtime_addresses
            .insert("div_sbyte".to_string(), self.current_address);

        // Store operands
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // Sign flags
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // Check dividend sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BPL, "__dsb_dividend_pos");
        // Negate dividend
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        // Toggle sign flags
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP4);

        self.define_label("__dsb_dividend_pos");
        // Check divisor sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BPL, "__dsb_divisor_pos");
        // Negate divisor
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        // Toggle quotient sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::EOR_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__dsb_divisor_pos");
        // Divide
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jsr_label("__div_byte");

        // Save results
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // Apply sign to quotient
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__dsb_quot_pos");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        self.define_label("__dsb_quot_pos");
        // Apply sign to remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__dsb_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        self.define_label("__dsb_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 16-bit divide routine (simplified).
    fn emit_divide_sword_routine(&mut self) {
        self.define_label("__div_sword");
        self.runtime_addresses
            .insert("div_sword".to_string(), self.current_address);

        self.emit_jsr_label("__div_sbyte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point multiply routine.
    fn emit_fixed_multiply_routine(&mut self) {
        self.define_label("__mul_fixed");
        self.runtime_addresses
            .insert("mul_fixed".to_string(), self.current_address);

        // Copy operands
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_jsr_label("__mul_sword");

        // Arithmetic right shift by 4
        let shift_loop = self.make_label("fmul_shr");

        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.emit_imm(opcodes::LDY_IMM, 4);
        self.define_label(&shift_loop);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point divide routine.
    fn emit_fixed_divide_routine(&mut self) {
        self.define_label("__div_fixed");
        self.runtime_addresses
            .insert("div_fixed".to_string(), self.current_address);

        // Shift dividend left by 4
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        let shift_loop = self.make_label("fdiv_shl");
        self.emit_imm(opcodes::LDY_IMM, 4);
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        self.emit_jsr_label("__div_sword");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point modulo routine.
    fn emit_fixed_modulo_routine(&mut self) {
        self.define_label("__mod_fixed");
        self.runtime_addresses
            .insert("mod_fixed".to_string(), self.current_address);

        self.emit_jsr_label("__div_sword");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point comparison routines.
    fn emit_fixed_comparison_routines(&mut self) {
        // Less than
        self.define_label("__cmp_fixed_lt");
        self.runtime_addresses
            .insert("cmp_fixed_lt".to_string(), self.current_address);

        let lt_true = self.make_label("flt_true");
        let lt_done = self.make_label("flt_done");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BVC, "__flt_no_ovf1");
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label("__flt_no_ovf1");
        self.emit_branch(opcodes::BMI, &lt_true.clone());
        self.emit_branch(opcodes::BNE, &lt_done.clone());

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, &lt_true);

        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&lt_done);

        self.define_label(&lt_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&lt_done);
        self.emit_byte(opcodes::RTS);

        // Less or equal
        self.define_label("__cmp_fixed_le");
        self.runtime_addresses
            .insert("cmp_fixed_le".to_string(), self.current_address);

        let le_true = self.make_label("fle_true");
        let le_done = self.make_label("fle_done");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BVC, "__fle_no_ovf1");
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label("__fle_no_ovf1");
        self.emit_branch(opcodes::BMI, &le_true.clone());
        self.emit_branch(opcodes::BNE, &le_done.clone());

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, &le_true.clone());
        self.emit_branch(opcodes::BEQ, &le_true);

        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&le_done);

        self.define_label(&le_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&le_done);
        self.emit_byte(opcodes::RTS);

        // Greater than
        self.define_label("__cmp_fixed_gt");
        self.runtime_addresses
            .insert("cmp_fixed_gt".to_string(), self.current_address);

        self.emit_jsr_label("__cmp_fixed_le");
        self.emit_imm(opcodes::EOR_IMM, 1);
        self.emit_byte(opcodes::RTS);

        // Greater or equal
        self.define_label("__cmp_fixed_ge");
        self.runtime_addresses
            .insert("cmp_fixed_ge".to_string(), self.current_address);

        self.emit_jsr_label("__cmp_fixed_lt");
        self.emit_imm(opcodes::EOR_IMM, 1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit readln routine.
    fn emit_readln_routine(&mut self) {
        self.define_label("__readln");
        self.runtime_addresses
            .insert("readln".to_string(), self.current_address);

        use super::mos6510::{c64, petscii};

        // Initialize buffer index
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Loop: get characters
        self.define_label("__readln_loop");
        self.emit_abs(opcodes::JSR, kernal::CHRIN);

        // Check for RETURN
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(petscii::RETURN);
        self.emit_branch(opcodes::BEQ, "__readln_done");

        // Store character
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_abs(opcodes::STA_ABY, c64::INPUT_BUFFER);

        // Increment index
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::STY_ZP);
        self.emit_byte(zeropage::TMP3);

        // Overflow protection
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(255);
        self.emit_branch(opcodes::BCC, "__readln_loop");

        // Done: null-terminate
        self.define_label("__readln_done");
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_abs(opcodes::STA_ABY, c64::INPUT_BUFFER);

        // Return buffer address
        self.emit_imm(opcodes::LDA_IMM, (c64::INPUT_BUFFER & 0xFF) as u8);
        self.emit_imm(opcodes::LDX_IMM, (c64::INPUT_BUFFER >> 8) as u8);

        self.emit_byte(opcodes::RTS);
    }
}
