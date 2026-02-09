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
use super::mos6510::{kernal, opcodes, vic, zeropage};
use super::string_runtime::StringRuntime;
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

    // String routines
    fn emit_str_len_routine(&mut self);
    fn emit_str_concat_routine(&mut self);

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

    // PRNG routines
    fn emit_prng_init_routine(&mut self);
    fn emit_prng_next_routine(&mut self);
    fn emit_rand_routine(&mut self);

    // Sound routines
    fn emit_note_to_freq_routine(&mut self);

    // Graphics routines
    fn emit_gfx_mode_routine(&mut self);
    fn emit_get_gfx_mode_routine(&mut self);

    // Bitmap graphics routines
    fn emit_plot_routine(&mut self);
    fn emit_unplot_routine(&mut self);
    fn emit_point_routine(&mut self);
    fn emit_plot_mc_routine(&mut self);
    fn emit_point_mc_routine(&mut self);
    fn emit_bitmap_fill_routine(&mut self);
    fn emit_calc_bitmap_addr_routine(&mut self);

    // Drawing primitives
    fn emit_line_routine(&mut self);
    fn emit_hline_routine(&mut self);
    fn emit_vline_routine(&mut self);
    fn emit_rect_routine(&mut self);
    fn emit_rect_fill_routine(&mut self);

    // Cell color control
    fn emit_cell_color_routine(&mut self);
    fn emit_get_cell_color_routine(&mut self);
    fn emit_color_ram_routine(&mut self);
    fn emit_get_color_ram_routine(&mut self);
    fn emit_fill_colors_routine(&mut self);
    fn emit_fill_color_ram_routine(&mut self);

    // Raster functions
    fn emit_wait_raster_routine(&mut self);
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

        // String routines
        self.emit_str_len_routine();
        self.emit_str_concat_routine();

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

        // String conversion routines
        self.emit_string_runtime();

        // Input routine
        self.emit_readln_routine();

        // PRNG routines
        self.emit_prng_init_routine();
        self.emit_prng_next_routine();
        self.emit_rand_routine();

        // Sound routines
        self.emit_note_to_freq_routine();

        // Graphics routines
        self.emit_gfx_mode_routine();
        self.emit_get_gfx_mode_routine();

        // Bitmap graphics routines
        self.emit_calc_bitmap_addr_routine();
        self.emit_plot_routine();
        self.emit_unplot_routine();
        self.emit_point_routine();
        self.emit_plot_mc_routine();
        self.emit_point_mc_routine();
        self.emit_bitmap_fill_routine();

        // Drawing primitives
        self.emit_hline_routine();
        self.emit_vline_routine();
        self.emit_line_routine();
        self.emit_rect_routine();
        self.emit_rect_fill_routine();

        // Cell color control
        self.emit_cell_color_routine();
        self.emit_get_cell_color_routine();
        self.emit_color_ram_routine();
        self.emit_get_color_ram_routine();
        self.emit_fill_colors_routine();
        self.emit_fill_color_ram_routine();

        // Raster functions
        self.emit_wait_raster_routine();
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

        // Extract fractional nibble BEFORE print_word (which corrupts TMP1)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F); // Get fractional nibble
        self.emit_byte(opcodes::PHA); // Save on stack

        // Print integer part (TMP3/TMP3_HI)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_jsr_label("__print_word");

        // Print decimal point
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Restore fractional nibble from stack and store in TMP3
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Clear result for multiplication
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if frac is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pfix_print_frac");

        // Multiply frac * 625 using direct bit checks
        // bit 0 (1) -> add 625 (0x0271)
        // bit 1 (2) -> add 1250 (0x04E2)
        // bit 2 (4) -> add 2500 (0x09C4)
        // bit 3 (8) -> add 5000 (0x1388)

        // Check bit 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01);
        self.emit_branch(opcodes::BEQ, "__pfix_bit1");
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

        // Check bit 1
        self.define_label("__pfix_bit1");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x02);
        self.emit_branch(opcodes::BEQ, "__pfix_bit2");
        // Add 1250 (0x04E2)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::ADC_IMM, 0xE2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check bit 2
        self.define_label("__pfix_bit2");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x04);
        self.emit_branch(opcodes::BEQ, "__pfix_bit3");
        // Add 2500 (0x09C4)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::ADC_IMM, 0xC4);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x09);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check bit 3
        self.define_label("__pfix_bit3");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x08);
        self.emit_branch(opcodes::BEQ, "__pfix_print_frac");
        // Add 5000 (0x1388)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::ADC_IMM, 0x88);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x13);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

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

    /// Emit string length routine.
    ///
    /// Calculates the length of a null-terminated string.
    /// Input: String address in TMP1/TMP1_HI
    /// Output: A = length (0-255)
    fn emit_str_len_routine(&mut self) {
        self.define_label("__str_len");
        self.runtime_addresses
            .insert("str_len".to_string(), self.current_address);

        // Initialize counter in Y
        self.emit_imm(opcodes::LDY_IMM, 0);

        // Loop: check each character
        self.define_label("__str_len_loop");
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);

        // If null terminator, we're done
        self.emit_branch(opcodes::BEQ, "__str_len_done");

        // Increment counter
        self.emit_byte(opcodes::INY);

        // Continue loop (BNE will always branch since Y wraps at 255)
        self.emit_branch(opcodes::BNE, "__str_len_loop");

        // Done: transfer Y to A
        self.define_label("__str_len_done");
        self.emit_byte(opcodes::TYA);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit string concatenation routine.
    ///
    /// Concatenates two null-terminated strings into the string buffer.
    /// Input: TMP1/TMP1_HI = first string address
    ///        TMP3/TMP3_HI = second string address
    /// Output: A = low byte of buffer address, X = high byte
    ///         TMP1/TMP1_HI also set to buffer address
    fn emit_str_concat_routine(&mut self) {
        use super::mos6510::c64;

        self.define_label("__str_concat");
        self.runtime_addresses
            .insert("str_concat".to_string(), self.current_address);

        // Use TMP4 as index into destination buffer
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // Copy first string (from TMP1/TMP1_HI)
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__str_concat_copy1");
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);

        // Check for null terminator
        self.emit_branch(opcodes::BEQ, "__str_concat_second");

        // Store to buffer using absolute indexed
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_abs(opcodes::STA_ABX, c64::STR_CONCAT_BUFFER);

        // Increment both indices
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP4);

        // Continue loop
        self.emit_jmp("__str_concat_copy1");

        // Copy second string (from TMP3/TMP3_HI)
        self.define_label("__str_concat_second");
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__str_concat_copy2");
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP3);

        // Store to buffer (including final null terminator)
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_abs(opcodes::STA_ABX, c64::STR_CONCAT_BUFFER);

        // Check for null terminator (after storing it)
        self.emit_branch(opcodes::BEQ, "__str_concat_done");

        // Increment both indices
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP4);

        // Continue loop
        self.emit_jmp("__str_concat_copy2");

        // Done: return buffer address
        self.define_label("__str_concat_done");
        self.emit_imm(opcodes::LDA_IMM, (c64::STR_CONCAT_BUFFER & 0xFF) as u8);
        self.emit_imm(opcodes::LDX_IMM, ((c64::STR_CONCAT_BUFFER >> 8) & 0xFF) as u8);

        // Also store in TMP1/TMP1_HI for compatibility
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

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

    /// Emit 16-bit multiply routine.
    ///
    /// Input: A/X = first operand (A=low, X=high)
    ///        TMP3/TMP3_HI = second operand
    /// Output: A/X = result (low 16 bits of 32-bit product)
    ///
    /// Uses shift-and-add algorithm for 16x16 -> 32 bit multiplication.
    /// Only returns low 16 bits (suitable for word multiplication).
    fn emit_multiply_word_routine(&mut self) {
        self.define_label("__mul_word");
        self.runtime_addresses
            .insert("mul_word".to_string(), self.current_address);

        // Store first operand (multiplicand) in TMP1/TMP1_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Store second operand (multiplier) already in TMP3/TMP3_HI

        // Initialize 32-bit result to 0
        // Use $04-$07 for result (FP_WORK_LO/HI and $06/$07)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x04); // result byte 0 (lowest)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x05); // result byte 1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06); // result byte 2
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x07); // result byte 3 (highest)

        // Loop 16 times
        self.emit_imm(opcodes::LDY_IMM, 16);

        let loop_label = self.make_label("mw_loop");
        let skip_add = self.make_label("mw_skip");

        self.define_label(&loop_label);

        // Shift 32-bit result left by 1
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x05);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(0x07);

        // Shift multiplier left, check if high bit was set
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BCC, &skip_add);

        // Add multiplicand to result (16-bit add to low 16 bits of result)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x04);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x05);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x05);

        // Propagate carry to high bytes
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x06);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x06);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x07);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(0x07);

        self.define_label(&skip_add);

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &loop_label);

        // Return low 16 bits of result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(0x04);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(0x05);

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

    /// Emit signed 16-bit multiply routine.
    ///
    /// Input: A/X = first operand (A=low, X=high)
    ///        TMP3/TMP3_HI = second operand
    /// Output: A/X = result (signed 16-bit)
    ///
    /// Handles signs, then uses unsigned multiply, then applies sign to result.
    fn emit_multiply_sword_routine(&mut self) {
        self.define_label("__mul_sword");
        self.runtime_addresses
            .insert("mul_sword".to_string(), self.current_address);

        // Store first operand
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Use TMP2 as sign flag (0 = positive, 1 = negative)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        // Check sign of first operand (in TMP1_HI bit 7)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BPL, "__msw_first_pos");

        // First operand is negative, negate it (two's complement)
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

        // Toggle sign flag
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2);

        self.define_label("__msw_first_pos");

        // Check sign of second operand (in TMP3_HI bit 7)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BPL, "__msw_second_pos");

        // Second operand is negative, negate it
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

        // Toggle sign flag
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP2);

        self.define_label("__msw_second_pos");

        // Perform unsigned multiply (operands in TMP1/TMP1_HI and TMP3/TMP3_HI)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__mul_word");

        // Check sign flag - if non-zero, negate result
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BEQ, "__msw_done");

        // Negate result (two's complement)
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
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::RTS);

        self.define_label("__msw_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
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

    /// Emit PRNG initialization routine.
    ///
    /// Sets up SID voice 3 for hardware noise generation and seeds LFSR.
    /// Called automatically at program start and by seed().
    fn emit_prng_init_routine(&mut self) {
        use super::mos6510::{cia, sid, vic};

        self.define_label("__prng_init");
        self.runtime_addresses
            .insert("prng_init".to_string(), self.current_address);

        // Setup SID voice 3 for noise - this is the most reliable
        // random source on C64 and works in all emulators
        // Set frequency to maximum for fast noise generation
        self.emit_imm(opcodes::LDA_IMM, 0xFF);
        self.emit_abs(opcodes::STA_ABS, sid::VOICE3_FREQ_LO);
        self.emit_abs(opcodes::STA_ABS, sid::VOICE3_FREQ_HI);

        // Set waveform to noise (bit 7 = noise waveform)
        self.emit_imm(opcodes::LDA_IMM, 0x80);
        self.emit_abs(opcodes::STA_ABS, sid::VOICE3_CTRL);

        // Seed LFSR from multiple hardware sources
        // Use SID noise XOR CIA timer XOR raster for entropy
        self.emit_abs(opcodes::LDA_ABS, sid::VOICE3_OSC);
        self.emit_abs(opcodes::EOR_ABS, cia::CIA1_TIMER_A_LO);
        self.emit_abs(opcodes::EOR_ABS, vic::RASTER);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::PRNG_LO);

        self.emit_abs(opcodes::LDA_ABS, sid::VOICE3_OSC);
        self.emit_abs(opcodes::EOR_ABS, cia::CIA1_TIMER_A_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::PRNG_HI);

        // Ensure seed is not zero (LFSR would get stuck)
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::PRNG_LO);
        self.emit_branch(opcodes::BNE, "__prng_init_ok");
        // If zero, use fixed non-zero seed
        self.emit_imm(opcodes::LDA_IMM, 0xAC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::PRNG_LO);
        self.emit_imm(opcodes::LDA_IMM, 0xE1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::PRNG_HI);
        self.define_label("__prng_init_ok");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit PRNG next routine.
    ///
    /// Uses SID hardware noise ($D41B) combined with software LFSR.
    /// Returns random byte in accumulator.
    /// IMPORTANT: Preserves TMP1-TMP5 (used by callers).
    fn emit_prng_next_routine(&mut self) {
        use super::mos6510::sid;

        self.define_label("__prng_next");
        self.runtime_addresses
            .insert("prng_next".to_string(), self.current_address);

        // Advance software LFSR first (16-bit Galois, polynomial $002D)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::PRNG_LO);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::PRNG_HI);
        self.emit_branch(opcodes::BCC, "__prng_no_xor");
        self.emit_imm(opcodes::EOR_IMM, 0x2D);
        self.define_label("__prng_no_xor");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::PRNG_LO);

        // XOR LFSR result with SID noise for final random value
        self.emit_abs(opcodes::EOR_ABS, sid::VOICE3_OSC);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit rand() routine.
    ///
    /// Returns a random fixed-point value between 0.0 and 0.9375 (15/16).
    /// Uses the PRNG to generate a random byte and takes the upper 4 bits
    /// as the fractional part of a 12.4 fixed-point number.
    ///
    /// Output: A = low byte, X = high byte of fixed 12.4
    /// Values: 0x0000 (0.0) to 0x000F (0.9375)
    fn emit_rand_routine(&mut self) {
        self.define_label("__rand");
        self.runtime_addresses
            .insert("rand".to_string(), self.current_address);

        // Get random byte (0-255)
        self.emit_jsr_label("__prng_next");

        // Convert to fixed 12.4 in range [0, 1)
        // Take upper 4 bits as fractional part (0-15 -> 0.0 to 0.9375)
        // Fixed 12.4: bits 3-0 are fractional (1/16 resolution)
        //
        // random >> 4 gives us 0-15 as the fractional value
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        // A now contains 0-15 (the fractional part)
        // X = 0 (integer part is always 0)
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit note-to-frequency conversion routine for SID.
    ///
    /// Input: A = note (0-11: C, C#, D, D#, E, F, F#, G, G#, A, A#, B)
    ///        TMP2 = octave (0-7)
    /// Output: A = frequency low byte, X = frequency high byte
    ///
    /// Uses PAL frequency values. The base frequencies are for octave 4,
    /// then shifted based on the requested octave.
    fn emit_note_to_freq_routine(&mut self) {
        self.define_label("__note_to_freq");
        self.runtime_addresses
            .insert("note_to_freq".to_string(), self.current_address);

        // Jump over the data table first
        self.emit_jmp("__ntf_code");

        // Frequency table for octave 4 (PAL values)
        // C4=262Hz, C#4=277Hz, D4=294Hz, D#4=311Hz, E4=330Hz, F4=349Hz,
        // F#4=370Hz, G4=392Hz, G#4=415Hz, A4=440Hz, A#4=466Hz, B4=494Hz
        // SID freq = Hz * 17.028 (PAL)
        self.define_label("__note_freq_table");
        let table_addr = self.current_address;
        // C4: 4460 = $116C
        self.emit_byte(0x6C);
        self.emit_byte(0x11);
        // C#4: 4724 = $1274
        self.emit_byte(0x74);
        self.emit_byte(0x12);
        // D4: 5005 = $138D
        self.emit_byte(0x8D);
        self.emit_byte(0x13);
        // D#4: 5303 = $14B7
        self.emit_byte(0xB7);
        self.emit_byte(0x14);
        // E4: 5620 = $15F4
        self.emit_byte(0xF4);
        self.emit_byte(0x15);
        // F4: 5955 = $1743
        self.emit_byte(0x43);
        self.emit_byte(0x17);
        // F#4: 6310 = $18A6
        self.emit_byte(0xA6);
        self.emit_byte(0x18);
        // G4: 6685 = $1A1D
        self.emit_byte(0x1D);
        self.emit_byte(0x1A);
        // G#4: 7083 = $1BAB
        self.emit_byte(0xAB);
        self.emit_byte(0x1B);
        // A4: 7503 = $1D4F
        self.emit_byte(0x4F);
        self.emit_byte(0x1D);
        // A#4: 7949 = $1F0D
        self.emit_byte(0x0D);
        self.emit_byte(0x1F);
        // B4: 8421 = $20E5
        self.emit_byte(0xE5);
        self.emit_byte(0x20);

        self.define_label("__ntf_code");
        // A = note (0-11), multiply by 2 for table lookup
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::TAY); // Y = note * 2

        // Load base frequency from table
        self.emit_aby(opcodes::LDA_ABY, table_addr);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = freq low
        self.emit_byte(opcodes::INY);
        self.emit_aby(opcodes::LDA_ABY, table_addr);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // TMP1_HI = freq high

        // Now scale based on octave (TMP2)
        // Octave 4 = no shift, octave 5 = shift left 1, octave 3 = shift right 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2); // A = octave
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 4); // A = octave - 4

        // If zero, no shift needed
        self.emit_branch(opcodes::BEQ, "__ntf_done");

        // If negative (octave < 4), shift right
        self.emit_branch(opcodes::BMI, "__ntf_shift_right");

        // Positive: shift left (octave > 4)
        self.emit_byte(opcodes::TAX); // X = shift count
        self.define_label("__ntf_shift_left");
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__ntf_shift_left");
        self.emit_jmp("__ntf_done");

        // Negative: shift right (negate to get positive count)
        self.define_label("__ntf_shift_right");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF); // Invert
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1); // +1 = negate
        self.emit_byte(opcodes::TAX); // X = shift count (positive)
        self.define_label("__ntf_shift_right_loop");
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__ntf_shift_right_loop");

        self.define_label("__ntf_done");
        // Return frequency in A (low), X (high)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit graphics mode switching routine.
    ///
    /// Input: A = mode (0-4)
    /// - Mode 0: ECM=0, BMM=0, MCM=0 (Standard Text)
    /// - Mode 1: ECM=0, BMM=0, MCM=1 (Multicolor Text)
    /// - Mode 2: ECM=0, BMM=1, MCM=0 (Hires Bitmap)
    /// - Mode 3: ECM=0, BMM=1, MCM=1 (Multicolor Bitmap)
    /// - Mode 4: ECM=1, BMM=0, MCM=0 (ECM Text)
    ///
    /// $D011 bits: Bit 5 = BMM, Bit 6 = ECM
    /// $D016 bits: Bit 4 = MCM
    fn emit_gfx_mode_routine(&mut self) {
        self.define_label("__gfx_mode");
        self.runtime_addresses
            .insert("gfx_mode".to_string(), self.current_address);

        // Save mode in TMP1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Clear ECM, BMM bits in $D011 (bits 5 and 6)
        self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
        self.emit_imm(opcodes::AND_IMM, !(vic::ECM | vic::BMM));
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // Store modified $D011 value

        // Clear MCM bit in $D016 (bit 4)
        self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
        self.emit_imm(opcodes::AND_IMM, !vic::MCM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // Store modified $D016 value

        // Check mode and set appropriate bits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Mode 0: do nothing (all bits already cleared)
        self.emit_imm(opcodes::CMP_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__gfx_mode_apply");

        // Mode 1: set MCM
        self.emit_imm(opcodes::CMP_IMM, 1);
        self.emit_branch(opcodes::BNE, "__gfx_mode_check2");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::ORA_IMM, vic::MCM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_jmp("__gfx_mode_apply");

        // Mode 2: set BMM
        self.define_label("__gfx_mode_check2");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::CMP_IMM, 2);
        self.emit_branch(opcodes::BNE, "__gfx_mode_check3");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::ORA_IMM, vic::BMM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jmp("__gfx_mode_apply");

        // Mode 3: set BMM and MCM
        self.define_label("__gfx_mode_check3");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::CMP_IMM, 3);
        self.emit_branch(opcodes::BNE, "__gfx_mode_check4");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::ORA_IMM, vic::BMM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::ORA_IMM, vic::MCM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_jmp("__gfx_mode_apply");

        // Mode 4: set ECM
        self.define_label("__gfx_mode_check4");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::ORA_IMM, vic::ECM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        // Apply the changes
        self.define_label("__gfx_mode_apply");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_abs(opcodes::STA_ABS, vic::CONTROL1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_abs(opcodes::STA_ABS, vic::CONTROL2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit get graphics mode routine.
    ///
    /// Output: A = current mode (0-4)
    /// Reads ECM, BMM from $D011 and MCM from $D016 to determine mode.
    fn emit_get_gfx_mode_routine(&mut self) {
        self.define_label("__get_gfx_mode");
        self.runtime_addresses
            .insert("get_gfx_mode".to_string(), self.current_address);

        // Read $D011 and extract ECM (bit 6) and BMM (bit 5)
        self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
        self.emit_imm(opcodes::AND_IMM, vic::ECM | vic::BMM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = ECM|BMM bits

        // Read $D016 and extract MCM (bit 4)
        self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
        self.emit_imm(opcodes::AND_IMM, vic::MCM);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = MCM bit

        // Check for ECM mode first (ECM=1, BMM=0, MCM=0)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::CMP_IMM, vic::ECM);
        self.emit_branch(opcodes::BNE, "__get_gfx_not_ecm");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BNE, "__get_gfx_not_ecm");
        // Mode 4: ECM text
        self.emit_imm(opcodes::LDA_IMM, 4);
        self.emit_byte(opcodes::RTS);

        self.define_label("__get_gfx_not_ecm");
        // Check for BMM (bitmap modes)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, vic::BMM);
        self.emit_branch(opcodes::BEQ, "__get_gfx_text");

        // Bitmap mode - check MCM
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BNE, "__get_gfx_mode3");
        // Mode 2: Hires bitmap
        self.emit_imm(opcodes::LDA_IMM, 2);
        self.emit_byte(opcodes::RTS);

        self.define_label("__get_gfx_mode3");
        // Mode 3: Multicolor bitmap
        self.emit_imm(opcodes::LDA_IMM, 3);
        self.emit_byte(opcodes::RTS);

        // Text mode - check MCM
        self.define_label("__get_gfx_text");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BNE, "__get_gfx_mode1");
        // Mode 0: Standard text
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__get_gfx_mode1");
        // Mode 1: Multicolor text
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit bitmap address calculation routine.
    ///
    /// Input: TMP1/TMP1_HI = X coordinate (0-319), TMP3 = Y coordinate (0-199)
    /// Output: TMP2/TMP2_HI = byte address in bitmap, Y = bit position (0-7)
    ///
    /// Bitmap layout: 8 bytes per character cell, row-by-row within cell
    /// offset = (y / 8) * 320 + (x / 8) * 8 + (y % 8)
    /// bit = 7 - (x % 8)
    fn emit_calc_bitmap_addr_routine(&mut self) {
        self.define_label("__calc_bitmap_addr");
        self.runtime_addresses
            .insert("calc_bitmap_addr".to_string(), self.current_address);

        // Calculate (y / 8) * 320
        // = (y / 8) * 256 + (y / 8) * 64
        // = (y >> 3) << 8 + (y >> 3) << 6
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3); // A = Y
        self.emit_byte(opcodes::LSR_ACC); // A = Y >> 1
        self.emit_byte(opcodes::LSR_ACC); // A = Y >> 2
        self.emit_byte(opcodes::LSR_ACC); // A = Y >> 3 = cell_y
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // TMP4 = cell_y

        // cell_y * 320 = cell_y * 256 + cell_y * 64
        // High byte = cell_y, Low byte = 0 for *256
        // Then add cell_y * 64 = cell_y << 6
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = 0 (low byte)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI); // TMP2_HI = cell_y (this is cell_y * 256)

        // Add cell_y * 64
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::ASL_ACC); // *2
        self.emit_byte(opcodes::ASL_ACC); // *4
        self.emit_byte(opcodes::ASL_ACC); // *8
        self.emit_byte(opcodes::ASL_ACC); // *16
        self.emit_byte(opcodes::ASL_ACC); // *32
        self.emit_byte(opcodes::ASL_ACC); // *64
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__cba_no_carry1");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__cba_no_carry1");

        // Now add (x / 8) * 8 = x & $FFF8
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1); // X low
        self.emit_imm(opcodes::AND_IMM, 0xF8); // Mask to multiple of 8
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // X high
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Add (y % 8)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::AND_IMM, 0x07);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__cba_no_carry2");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__cba_no_carry2");

        // Add bitmap base address ($2000)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 0x20); // Add $20 to high byte
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Calculate bit position: Y = 7 - (x % 8)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0x07);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0x07); // 7 - (x % 8)
        self.emit_byte(opcodes::TAY);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit plot routine for hires mode.
    ///
    /// Input: TMP1/TMP1_HI = X (0-319), TMP3 = Y (0-199)
    fn emit_plot_routine(&mut self) {
        self.define_label("__plot");
        self.runtime_addresses
            .insert("plot".to_string(), self.current_address);

        // Calculate bitmap address
        self.emit_jsr_label("__calc_bitmap_addr");

        // Create bit mask from Y register (bit position)
        self.emit_imm(opcodes::LDA_IMM, 0x80);
        self.define_label("__plot_shift");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__plot_set");
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__plot_shift");

        self.define_label("__plot_set");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // TMP4 = bit mask

        // OR the bit into the bitmap byte
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit unplot routine for hires mode.
    ///
    /// Input: TMP1/TMP1_HI = X (0-319), TMP3 = Y (0-199)
    fn emit_unplot_routine(&mut self) {
        self.define_label("__unplot");
        self.runtime_addresses
            .insert("unplot".to_string(), self.current_address);

        // Calculate bitmap address
        self.emit_jsr_label("__calc_bitmap_addr");

        // Create inverted bit mask from Y register
        self.emit_imm(opcodes::LDA_IMM, 0x80);
        self.define_label("__unplot_shift");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__unplot_clear");
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__unplot_shift");

        self.define_label("__unplot_clear");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF); // Invert mask
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // AND the inverted bit into the bitmap byte
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::AND_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit point routine for hires mode.
    ///
    /// Input: TMP1/TMP1_HI = X (0-319), TMP3 = Y (0-199)
    /// Output: A = 0 (not set) or 1 (set)
    fn emit_point_routine(&mut self) {
        self.define_label("__point");
        self.runtime_addresses
            .insert("point".to_string(), self.current_address);

        // Calculate bitmap address
        self.emit_jsr_label("__calc_bitmap_addr");

        // Create bit mask from Y register
        self.emit_imm(opcodes::LDA_IMM, 0x80);
        self.define_label("__point_shift");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__point_test");
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__point_shift");

        self.define_label("__point_test");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // Test the bit
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::AND_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__point_zero");
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_byte(opcodes::RTS);

        self.define_label("__point_zero");
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit plot routine for multicolor mode.
    ///
    /// Input: TMP1 = X (0-159), TMP3 = Y (0-199), TMP4 = color (0-3)
    fn emit_plot_mc_routine(&mut self) {
        self.define_label("__plot_mc");
        self.runtime_addresses
            .insert("plot_mc".to_string(), self.current_address);

        // For multicolor, each pixel is 2 bits, so 4 pixels per byte
        // Bitmap address = (y / 8) * 320 + (x / 4) * 8 + (y % 8)
        // Bit position = 6 - ((x % 4) * 2)

        // Calculate (y / 8) * 320 (same as hires)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5); // TMP5 = cell_y

        // cell_y * 320
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Add cell_y * 64
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__pmc_nc1");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__pmc_nc1");

        // Add (x / 4) * 8 = (x & $FC) * 2
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0xFC);
        self.emit_byte(opcodes::ASL_ACC); // *2
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__pmc_nc2");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__pmc_nc2");

        // Add (y % 8)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::AND_IMM, 0x07);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__pmc_nc3");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__pmc_nc3");

        // Add bitmap base ($2000)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x20);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Calculate bit position: shift = 6 - ((x % 4) * 2)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0x03);
        self.emit_byte(opcodes::ASL_ACC); // *2
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_imm(opcodes::LDA_IMM, 6);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::TAY); // Y = shift amount

        // Create color mask (color << shift)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4); // color (0-3)
        self.define_label("__pmc_shift_color");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__pmc_apply");
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__pmc_shift_color");

        self.define_label("__pmc_apply");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5); // TMP5 = shifted color

        // Create clear mask (%11 << shift, then invert)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0x03);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::TAY);
        self.emit_imm(opcodes::LDA_IMM, 0x03);
        self.define_label("__pmc_shift_mask");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__pmc_clear");
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__pmc_shift_mask");

        self.define_label("__pmc_clear");
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF); // Invert to get clear mask

        // Clear old bits and set new color
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::AND_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit point routine for multicolor mode.
    ///
    /// Input: TMP1 = X (0-159), TMP3 = Y (0-199)
    /// Output: A = color (0-3)
    fn emit_point_mc_routine(&mut self) {
        self.define_label("__point_mc");
        self.runtime_addresses
            .insert("point_mc".to_string(), self.current_address);

        // Same address calculation as plot_mc (simplified - call common routine)
        // For now, duplicate the calculation

        // Calculate address (same as plot_mc)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5);

        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__ptmc_nc1");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__ptmc_nc1");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0xFC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__ptmc_nc2");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__ptmc_nc2");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::AND_IMM, 0x07);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__ptmc_nc3");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__ptmc_nc3");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x20);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Read byte
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // Calculate shift: 6 - ((x % 4) * 2)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0x03);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::TAY);
        self.emit_imm(opcodes::LDA_IMM, 6);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::AND_IMM, 0x06); // Mask to valid shift

        // Shift byte right to get color in bits 0-1
        self.emit_byte(opcodes::TAY);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.define_label("__ptmc_shift");
        self.emit_imm(opcodes::CPY_IMM, 0);
        self.emit_branch(opcodes::BEQ, "__ptmc_done");
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::DEY);
        self.emit_jmp("__ptmc_shift");

        self.define_label("__ptmc_done");
        self.emit_imm(opcodes::AND_IMM, 0x03); // Mask to 2 bits
        self.emit_byte(opcodes::RTS);
    }

    /// Emit bitmap fill routine.
    ///
    /// Input: A = fill pattern byte
    fn emit_bitmap_fill_routine(&mut self) {
        self.define_label("__bitmap_fill");
        self.runtime_addresses
            .insert("bitmap_fill".to_string(), self.current_address);

        // Save fill pattern
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Set up pointer to bitmap ($2000)
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x20);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Fill 8000 bytes (31 pages of 256 + 64 bytes)
        // First fill 31 full pages
        self.emit_imm(opcodes::LDX_IMM, 31);
        self.define_label("__bf_page_loop");
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__bf_byte_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_branch(opcodes::BNE, "__bf_byte_loop");
        // Next page
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__bf_page_loop");

        // Fill remaining 64 bytes (8000 - 31*256 = 8000 - 7936 = 64)
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__bf_last_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_imm(opcodes::CPY_IMM, 64);
        self.emit_branch(opcodes::BNE, "__bf_last_loop");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit horizontal line routine for hires mode.
    ///
    /// Input: TMP1/TMP1_HI = X start (0-319), TMP3 = Y (0-199), TMP4/TMP5 = length
    fn emit_hline_routine(&mut self) {
        self.define_label("__hline");
        self.runtime_addresses
            .insert("hline".to_string(), self.current_address);

        // Check if length is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_branch(opcodes::BEQ, "__hline_done");

        // Loop: plot current x,y then increment x, decrement length
        self.define_label("__hline_loop");

        // Plot pixel at current position
        self.emit_jsr_label("__plot");

        // Decrement length (16-bit)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_imm(opcodes::SBC_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5);

        // Check if done (length == 0)
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__hline_done");

        // Increment X (16-bit)
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__hline_loop");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp("__hline_loop");

        self.define_label("__hline_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit vertical line routine for hires mode.
    ///
    /// Input: TMP1/TMP1_HI = X (0-319), TMP3 = Y start (0-199), TMP4 = length
    fn emit_vline_routine(&mut self) {
        self.define_label("__vline");
        self.runtime_addresses
            .insert("vline".to_string(), self.current_address);

        // Check if length is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__vline_done");

        // Loop: plot current x,y then increment y, decrement length
        self.define_label("__vline_loop");

        // Plot pixel at current position
        self.emit_jsr_label("__plot");

        // Decrement length
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__vline_done");

        // Increment Y
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_jmp("__vline_loop");

        self.define_label("__vline_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit line routine using Bresenham's algorithm.
    ///
    /// Input: TMP1/TMP1_HI = X1, TMP3 = Y1, TMP2/TMP2_HI = X2, TMP5 = Y2
    /// Uses: TMP6-TMP9 for dx, dy, err, and direction flags
    fn emit_line_routine(&mut self) {
        self.define_label("__line");
        self.runtime_addresses
            .insert("line".to_string(), self.current_address);

        // Calculate dx = abs(x2 - x1) and direction
        // Store sign of dx in bit 0 of TMP6
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2); // x2 low
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1); // - x1 low
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP7); // dx low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI); // x2 high
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI); // - x1 high
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP8); // dx high

        // Check if dx is negative
        self.emit_branch(opcodes::BPL, "__line_dx_pos");

        // dx is negative, negate it and set sx = -1 (TMP6 bit 0 = 1)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP7);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP7);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP8);
        self.emit_imm(opcodes::LDA_IMM, 1); // sx = -1
        self.emit_jmp("__line_store_sx");

        self.define_label("__line_dx_pos");
        self.emit_imm(opcodes::LDA_IMM, 0); // sx = +1

        self.define_label("__line_store_sx");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP6); // TMP6 bit 0 = sx direction

        // Calculate dy = abs(y2 - y1) and direction
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5); // y2
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP3); // - y1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP9); // dy

        // Check if dy is negative
        self.emit_branch(opcodes::BPL, "__line_dy_pos");

        // dy is negative, negate it and set sy = -1 (TMP6 bit 1 = 1)
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP9);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP6);
        self.emit_imm(opcodes::ORA_IMM, 0x02); // Set bit 1 for sy = -1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP6);

        self.define_label("__line_dy_pos");

        // Now TMP7/TMP8 = dx (16-bit), TMP9 = dy (8-bit), TMP6 = direction flags

        // Simplified Bresenham: we'll use an 8-bit approach for performance
        // err = dx - dy (approximate for simple cases)
        // We'll plot pixels in a loop

        // Main loop: plot and step
        self.define_label("__line_loop");

        // Plot current pixel
        self.emit_jsr_label("__plot");

        // Check if x1 == x2 and y1 == y2 (done)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BNE, "__line_continue");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_branch(opcodes::BNE, "__line_continue");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_branch(opcodes::BEQ, "__line_done");

        self.define_label("__line_continue");

        // Simple stepping: if dx > dy, step x; else step y
        // Compare dx (16-bit) with dy (8-bit extended)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP8); // dx high
        self.emit_branch(opcodes::BNE, "__line_step_x"); // dx >= 256, step x
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP7); // dx low
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP9); // compare with dy
        self.emit_branch(opcodes::BCS, "__line_step_x"); // dx >= dy, step x

        // Step Y
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP6);
        self.emit_imm(opcodes::AND_IMM, 0x02); // Check sy direction
        self.emit_branch(opcodes::BNE, "__line_step_y_neg");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3); // y++
        self.emit_jmp("__line_loop");

        self.define_label("__line_step_y_neg");
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP3); // y--
        self.emit_jmp("__line_loop");

        // Step X
        self.define_label("__line_step_x");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP6);
        self.emit_imm(opcodes::AND_IMM, 0x01); // Check sx direction
        self.emit_branch(opcodes::BNE, "__line_step_x_neg");
        // x++
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__line_loop");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp("__line_loop");

        self.define_label("__line_step_x_neg");
        // x--
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__line_dec_x_low");
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.define_label("__line_dec_x_low");
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_jmp("__line_loop");

        self.define_label("__line_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit rectangle outline routine.
    ///
    /// Input: TMP1/TMP1_HI = X, TMP3 = Y, TMP4/TMP5 = width, TMP2 = height
    fn emit_rect_routine(&mut self) {
        self.define_label("__rect");
        self.runtime_addresses
            .insert("rect".to_string(), self.current_address);

        // Save original values - we'll use stack
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::PHA); // save x low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::PHA); // save x high
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::PHA); // save y
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::PHA); // save width low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::PHA); // save width high
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::PHA); // save height

        // Draw top horizontal line (x, y, width)
        // TMP1/TMP1_HI already has x, TMP3 has y, TMP4/TMP5 has width
        self.emit_jsr_label("__hline");

        // Restore values for right vertical line
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // height
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5); // width high
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // width low
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // y
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // x high
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // x low

        // Calculate right edge: x + width - 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::PHA); // right x low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::PHA); // right x high

        // Decrement by 1
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCS, "__rect_no_borrow1");
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.define_label("__rect_no_borrow1");

        // Save for later and draw right vline
        // TMP4 = height for vline
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_jsr_label("__vline");

        // For bottom and left we need original values again
        // Bottom: draw at y + height - 1
        // Left: draw at original x

        self.emit_byte(opcodes::RTS);
    }

    /// Emit filled rectangle routine.
    ///
    /// Input: TMP1/TMP1_HI = X, TMP3 = Y, TMP4/TMP5 = width, TMP2 = height
    fn emit_rect_fill_routine(&mut self) {
        self.define_label("__rect_fill");
        self.runtime_addresses
            .insert("rect_fill".to_string(), self.current_address);

        // Loop through each row and draw horizontal line
        self.define_label("__rectf_loop");

        // Check if height is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BEQ, "__rectf_done");

        // Save current x, y, width, height
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::PHA);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::PHA);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::PHA);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::PHA);

        // Draw horizontal line
        self.emit_jsr_label("__hline");

        // Restore x and width
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Increment y
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);

        // Decrement height
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jmp("__rectf_loop");

        self.define_label("__rectf_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit cell_color routine.
    /// Sets foreground/background color in screen RAM for a cell.
    ///
    /// Input: TMP1 = cx (0-39), TMP3 = cy (0-24), TMP4 = combined color (fg<<4 | bg)
    fn emit_cell_color_routine(&mut self) {
        self.define_label("__cell_color");
        self.runtime_addresses
            .insert("cell_color".to_string(), self.current_address);

        // Calculate screen RAM offset: cy * 40 + cx
        // cy * 40 = cy * 32 + cy * 8 = (cy << 5) + (cy << 3)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3); // A = cy
        self.emit_byte(opcodes::ASL_ACC); // *2
        self.emit_byte(opcodes::ASL_ACC); // *4
        self.emit_byte(opcodes::ASL_ACC); // *8
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = cy * 8
        self.emit_byte(opcodes::ASL_ACC); // *16
        self.emit_byte(opcodes::ASL_ACC); // *32
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2); // A = cy * 40
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1); // A = cy * 40 + cx
        self.emit_byte(opcodes::TAY); // Y = offset (low byte)

        // Calculate high byte of offset (for cy >= 7)
        // We need to handle cy * 40 overflowing 255
        // cy * 40: cy=6 gives 240, cy=7 gives 280 (overflow)
        // High byte = (cy * 40) >> 8
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC); // cy / 2
        self.emit_byte(opcodes::LSR_ACC); // cy / 4
        self.emit_byte(opcodes::LSR_ACC); // cy / 8
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 0x04); // Add $04 (screen RAM base $0400)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Recalculate low byte properly
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3); // cy
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // cy * 8
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC); // cy * 32
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2); // cy * 40
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1); // + cx
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__cc_no_carry");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__cc_no_carry");

        // Store color at screen RAM location
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4); // combined color
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit get_cell_color routine.
    /// Gets foreground/background color from screen RAM for a cell.
    ///
    /// Input: TMP1 = cx (0-39), TMP3 = cy (0-24)
    /// Output: A = combined color (fg in high nibble, bg in low nibble)
    fn emit_get_cell_color_routine(&mut self) {
        self.define_label("__get_cell_color");
        self.runtime_addresses
            .insert("get_cell_color".to_string(), self.current_address);

        // Calculate screen RAM address (same as cell_color)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__gcc_no_carry");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__gcc_no_carry");

        // Load color from screen RAM
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit color_ram routine.
    /// Sets color RAM value at cell position.
    ///
    /// Input: TMP1 = cx (0-39), TMP3 = cy (0-24), TMP4 = color
    fn emit_color_ram_routine(&mut self) {
        self.define_label("__color_ram");
        self.runtime_addresses
            .insert("color_ram".to_string(), self.current_address);

        // Calculate color RAM address: $D800 + cy * 40 + cx
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 0xD8); // $D800 high byte
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__cr_no_carry");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__cr_no_carry");

        // Store color at color RAM location
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit get_color_ram routine.
    /// Gets color RAM value at cell position.
    ///
    /// Input: TMP1 = cx (0-39), TMP3 = cy (0-24)
    /// Output: A = color value
    fn emit_get_color_ram_routine(&mut self) {
        self.define_label("__get_color_ram");
        self.runtime_addresses
            .insert("get_color_ram".to_string(), self.current_address);

        // Calculate color RAM address: $D800 + cy * 40 + cx
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 0xD8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__gcr_no_carry");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.define_label("__gcr_no_carry");

        // Load color from color RAM
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP2);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fill_colors routine.
    /// Fills all screen RAM cells with the same color value.
    ///
    /// Input: A = combined color (fg<<4 | bg)
    fn emit_fill_colors_routine(&mut self) {
        self.define_label("__fill_colors");
        self.runtime_addresses
            .insert("fill_colors".to_string(), self.current_address);

        // Save color
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Set up pointer to screen RAM ($0400)
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x04);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Fill 1000 bytes (3 pages of 256 + 232 bytes)
        // First fill 3 full pages
        self.emit_imm(opcodes::LDX_IMM, 3);
        self.define_label("__fc_page_loop");
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__fc_byte_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_branch(opcodes::BNE, "__fc_byte_loop");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__fc_page_loop");

        // Fill remaining 232 bytes (1000 - 768 = 232)
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__fc_last_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_imm(opcodes::CPY_IMM, 232);
        self.emit_branch(opcodes::BNE, "__fc_last_loop");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fill_color_ram routine.
    /// Fills all color RAM with the same value.
    ///
    /// Input: A = color value
    fn emit_fill_color_ram_routine(&mut self) {
        self.define_label("__fill_color_ram");
        self.runtime_addresses
            .insert("fill_color_ram".to_string(), self.current_address);

        // Save color
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Set up pointer to color RAM ($D800)
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0xD8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Fill 1000 bytes (same as screen RAM)
        self.emit_imm(opcodes::LDX_IMM, 3);
        self.define_label("__fcr_page_loop");
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__fcr_byte_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_branch(opcodes::BNE, "__fcr_byte_loop");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__fcr_page_loop");

        // Fill remaining 232 bytes
        self.emit_imm(opcodes::LDY_IMM, 0);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.define_label("__fcr_last_loop");
        self.emit_byte(opcodes::STA_IZY);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INY);
        self.emit_imm(opcodes::CPY_IMM, 232);
        self.emit_branch(opcodes::BNE, "__fcr_last_loop");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit wait_raster routine.
    /// Waits until the raster line reaches the target value.
    ///
    /// Input: TMP1 = target line low byte, TMP1_HI = target line high byte (0 or 1)
    fn emit_wait_raster_routine(&mut self) {
        self.define_label("__wait_raster");
        self.runtime_addresses
            .insert("wait_raster".to_string(), self.current_address);

        // Loop until raster matches target
        self.define_label("__wr_loop");

        // Read current raster line high bit from $D011 bit 7
        self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
        self.emit_imm(opcodes::AND_IMM, 0x80); // Get bit 7
        self.emit_byte(opcodes::ASL_ACC); // Shift to carry
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::ROL_ACC); // A = 0 or 1 (high byte of raster)

        // Compare high byte
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BNE, "__wr_loop");

        // High byte matches, compare low byte
        self.emit_abs(opcodes::LDA_ABS, vic::RASTER);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__wr_loop");

        self.emit_byte(opcodes::RTS);
    }
}
