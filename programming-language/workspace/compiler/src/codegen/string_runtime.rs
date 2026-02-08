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

//! String runtime routines for numeric conversions.
//!
//! This module provides 6502 assembly routines for:
//! - Converting numeric values to string representations
//! - Parsing strings to numeric values
//!
//! All routines use a shared string buffer in zero-page adjacent memory.

use super::emit::EmitHelpers;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::CodeGenerator;

/// Buffer address for string conversion output.
/// This is a 12-byte buffer that can hold "-32768" plus null terminator
/// plus some extra space for fixed-point decimals.
const STRING_BUFFER: u16 = 0x0340; // Just after typical BASIC variables area

/// Extension trait for string runtime code generation.
pub trait StringRuntime {
    /// Emit all string runtime routines.
    fn emit_string_runtime(&mut self);

    /// Emit __byte_to_string routine.
    /// Input: A = 8-bit unsigned value
    /// Output: A/X = pointer to null-terminated string
    fn emit_byte_to_string(&mut self);

    /// Emit __word_to_string routine.
    /// Input: A/X = 16-bit unsigned value (A=low, X=high)
    /// Output: A/X = pointer to null-terminated string
    fn emit_word_to_string(&mut self);

    /// Emit __bool_to_string routine.
    /// Input: A = 0 (false) or non-zero (true)
    /// Output: A/X = pointer to "TRUE" or "FALSE" string
    fn emit_bool_to_string(&mut self);

    /// Emit __string_to_byte routine.
    /// Input: A/X = pointer to string
    /// Output: A = 8-bit unsigned value (0 on error)
    fn emit_string_to_byte(&mut self);

    /// Emit __string_to_word routine.
    /// Input: A/X = pointer to string
    /// Output: A/X = 16-bit unsigned value (0 on error)
    fn emit_string_to_word(&mut self);

    /// Emit __fixed_to_string routine.
    /// Input: A/X = 12.4 fixed-point value
    /// Output: A/X = pointer to null-terminated string
    fn emit_fixed_to_string(&mut self);

    /// Emit __float_to_string routine.
    /// Input: A/X = IEEE-754 binary16 float
    /// Output: A/X = pointer to null-terminated string
    fn emit_float_to_string(&mut self);
}

impl StringRuntime for CodeGenerator {
    fn emit_string_runtime(&mut self) {
        self.emit_byte_to_string();
        self.emit_word_to_string();
        self.emit_bool_to_string();
        self.emit_string_to_byte();
        self.emit_string_to_word();
        self.emit_fixed_to_string();
        self.emit_float_to_string();
    }

    /// Emit __fixed_to_string routine.
    /// For now, this just outputs the integer part.
    fn emit_fixed_to_string(&mut self) {
        self.define_label("__fixed_to_string");
        // For 12.4 fixed-point, shift right 4 to get integer part
        // Then convert as signed word
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Arithmetic shift right 4 times (preserve sign)
        for _ in 0..4 {
            self.emit_byte(opcodes::LDA_ZP);
            self.emit_byte(zeropage::TMP1_HI);
            self.emit_byte(opcodes::CMP_IMM);
            self.emit_byte(0x80);
            self.emit_byte(opcodes::ROR_ZP);
            self.emit_byte(zeropage::TMP1_HI);
            self.emit_byte(opcodes::ROR_ZP);
            self.emit_byte(zeropage::TMP1);
        }

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp("__word_to_string");
    }

    /// Emit __float_to_string routine.
    /// For now, converts to fixed-point first, then to string.
    fn emit_float_to_string(&mut self) {
        self.define_label("__float_to_string");
        // Convert float to fixed, then call fixed_to_string
        self.emit_jsr_label("__float_to_fixed");
        self.emit_jmp("__fixed_to_string");
    }

    fn emit_byte_to_string(&mut self) {
        // __byte_to_string: Convert 8-bit value in A to string
        // Uses division by 10 to extract digits
        self.define_label("__byte_to_string");

        // Zero-extend to 16-bit and use word routine
        self.emit_imm(opcodes::LDX_IMM, 0);
        // Fall through to word_to_string
        self.emit_jmp("__word_to_string");
    }

    fn emit_word_to_string(&mut self) {
        // __word_to_string: Convert 16-bit value in A/X to string
        // Algorithm: Divide by 10 repeatedly, store remainders as ASCII digits
        self.define_label("__word_to_string");

        // Store value in TMP1/TMP1_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // TMP2 will be our buffer pointer (working backwards)
        self.emit_imm(opcodes::LDA_IMM, (STRING_BUFFER + 11) as u8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, ((STRING_BUFFER + 11) >> 8) as u8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Store null terminator at end
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        // Check if value is zero
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        let not_zero = self.make_label("w2s_not_zero");
        self.emit_branch(opcodes::BNE, &not_zero);

        // Value is zero - just put "0"
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        let done = self.make_label("w2s_done");
        self.emit_jmp(&done);

        self.define_label(&not_zero);

        // Loop: divide by 10, store remainder as digit
        let loop_label = self.make_label("w2s_loop");
        self.define_label(&loop_label);

        // Call divide by 10 subroutine
        // Dividend in TMP1/TMP1_HI, result in same, remainder in A
        self.emit_jsr_label("__div10");

        // Convert remainder to ASCII and store
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        // Check if quotient is zero
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BNE, &loop_label);

        self.define_label(&done);

        // Return pointer to start of string
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::RTS);

        // __div10: Divide TMP1/TMP1_HI by 10
        // Returns quotient in TMP1/TMP1_HI, remainder in A
        self.define_label("__div10");

        // Simple bit-by-bit division
        self.emit_imm(opcodes::LDA_IMM, 0); // Remainder
        self.emit_imm(opcodes::LDX_IMM, 16); // 16 bits

        let div_loop = self.make_label("div10_loop");
        self.define_label(&div_loop);

        // Shift dividend left, shift bit into remainder
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ROL_ACC);

        // If remainder >= 10, subtract 10 and set bit 0 of quotient
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(10);
        let no_sub = self.make_label("div10_nosub");
        self.emit_branch(opcodes::BCC, &no_sub);

        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(10);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1);

        self.define_label(&no_sub);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &div_loop);

        self.emit_byte(opcodes::RTS);
    }

    fn emit_bool_to_string(&mut self) {
        // __bool_to_string: Convert boolean to "TRUE" or "FALSE"
        // We use the word_to_string routine since booleans are 0 or 1
        // This will print "0" for false and "1" for true
        self.define_label("__bool_to_string");

        // Zero-extend A to 16-bit and call word_to_string
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_jmp("__word_to_string");
    }

    fn emit_string_to_byte(&mut self) {
        // __string_to_byte: Parse string to 8-bit unsigned
        self.define_label("__string_to_byte");
        // Just use word routine and return low byte
        self.emit_jsr_label("__string_to_word");
        // A already contains low byte
        self.emit_byte(opcodes::RTS);
    }

    fn emit_string_to_word(&mut self) {
        // __string_to_word: Parse string to 16-bit unsigned
        // Input: A/X = pointer to string
        // Output: A/X = 16-bit value
        self.define_label("__string_to_word");
        // Also used for signed types
        self.define_label("__string_to_sbyte");
        self.define_label("__string_to_sword");

        // Store string pointer in TMP2/TMP2_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Initialize result in TMP1/TMP1_HI to 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_imm(opcodes::LDY_IMM, 0);

        // Skip leading whitespace
        let skip_ws = self.make_label("s2w_skipws");
        self.define_label(&skip_ws);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(b' ');
        let not_ws = self.make_label("s2w_notws");
        self.emit_branch(opcodes::BNE, &not_ws);
        self.emit_byte(opcodes::INY);
        self.emit_branch(opcodes::BNE, &skip_ws); // BNE acts as BRA here

        self.define_label(&not_ws);

        // Parse digits
        let parse_loop = self.make_label("s2w_loop");
        self.define_label(&parse_loop);

        // Load character
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);

        // Check if it's a digit (0-9)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(b'0');
        let done = self.make_label("s2w_done");
        self.emit_branch(opcodes::BCC, &done); // < '0'
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(b'9' + 1);
        self.emit_branch(opcodes::BCS, &done); // > '9'

        // Convert ASCII to digit
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(b'0');
        self.emit_byte(opcodes::PHA); // Save digit

        // Multiply result by 10: result = result * 8 + result * 2
        // First, save result * 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Multiply by 2
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Save result * 2
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::PHA);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::PHA);

        // Multiply by 4 (result * 8 total from original)
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Add result * 2 to get result * 10
        self.emit_byte(opcodes::PLA); // high byte of *2
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::PLA); // low byte of *2
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Add the digit
        self.emit_byte(opcodes::PLA); // Get digit
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        let no_carry = self.make_label("s2w_nc");
        self.emit_branch(opcodes::BCC, &no_carry);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.define_label(&no_carry);

        // Next character
        self.emit_byte(opcodes::INY);
        self.emit_jmp(&parse_loop);

        self.define_label(&done);

        // Return result in A/X
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::RTS);
    }
}
