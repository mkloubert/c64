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

//! Function call code generation.
//!
//! This module provides code generation for function calls:
//! - Built-in functions (cls, print, println, cursor, get_key, read, readln, poke, peek, len)
//! - len() supports both arrays (compile-time) and strings (runtime)
//! - User-defined functions

use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::{cia, kernal, opcodes, petscii, sid, sprite, vic, zeropage};
use super::type_inference::TypeInference;
use super::CodeGenerator;
use crate::ast::{Expr, Type};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for function call code generation.
pub trait FunctionCallEmitter {
    /// Generate code for a function call.
    ///
    /// Handles built-in functions (cls, print, println, cursor, get_key, read,
    /// readln, poke, peek, len) and user-defined functions.
    fn generate_function_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: &Span,
    ) -> Result<(), CompileError>;
}

impl FunctionCallEmitter for CodeGenerator {
    fn generate_function_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: &Span,
    ) -> Result<(), CompileError> {
        match name {
            "cls" => {
                // Clear screen: output CHR$(147)
                self.emit_imm(opcodes::LDA_IMM, petscii::CLEAR_SCREEN);
                self.emit_abs(opcodes::JSR, kernal::CHROUT);
            }
            "print" | "println" => {
                if !args.is_empty() {
                    let arg_type = self.infer_type_from_expr(&args[0]);
                    self.generate_expression(&args[0])?;

                    // Call appropriate print routine based on type
                    match arg_type {
                        Type::String => {
                            // For string variables, copy A/X to TMP1/TMP1_HI
                            // (String literals already set TMP1/TMP1_HI, this is safe)
                            self.emit_byte(opcodes::STA_ZP);
                            self.emit_byte(zeropage::TMP1);
                            self.emit_byte(opcodes::STX_ZP);
                            self.emit_byte(zeropage::TMP1_HI);
                            self.emit_jsr_label("__print_str");
                        }
                        Type::Word => {
                            self.emit_jsr_label("__print_word");
                        }
                        Type::Sword => {
                            self.emit_jsr_label("__print_sword");
                        }
                        Type::Bool => {
                            self.emit_jsr_label("__print_bool");
                        }
                        Type::Sbyte => {
                            self.emit_jsr_label("__print_sbyte");
                        }
                        Type::Byte => {
                            self.emit_jsr_label("__print_byte");
                        }
                        Type::Fixed => {
                            self.emit_jsr_label("__print_fixed");
                        }
                        Type::Float => {
                            self.emit_jsr_label("__print_float");
                        }
                        _ => {
                            // Default to byte printing
                            self.emit_jsr_label("__print_byte");
                        }
                    }
                }
                if name == "println" {
                    self.emit_imm(opcodes::LDA_IMM, petscii::RETURN);
                    self.emit_abs(opcodes::JSR, kernal::CHROUT);
                }
            }
            "cursor" => {
                if args.len() >= 2 {
                    // KERNAL PLOT expects: X=Row, Y=Column
                    // Our API: cursor(x=column, y=row)
                    // So we need: Y=args[0] (column), X=args[1] (row)
                    self.generate_expression(&args[1])?; // row -> X
                    self.emit_byte(opcodes::TAX);
                    self.generate_expression(&args[0])?; // column -> Y
                    self.emit_byte(opcodes::TAY);
                    // Call PLOT with carry clear (set position)
                    self.emit_byte(opcodes::CLC);
                    self.emit_abs(opcodes::JSR, kernal::PLOT);
                }
            }
            "get_key" => {
                self.emit_abs(opcodes::JSR, kernal::GETIN);
            }
            "read" => {
                let wait_label = self.make_label("read_key");
                self.define_label(&wait_label);
                self.emit_abs(opcodes::JSR, kernal::GETIN);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &wait_label);
            }
            "readln" => {
                // Read a line of input into buffer at INPUT_BUFFER ($C100)
                // Returns string address in TMP1/TMP1_HI (for print_str compatibility)
                self.emit_jsr_label("__readln");
            }
            "joystick" => {
                // Read joystick state from port 1 or 2
                // Port 1 = $DC01 (CIA1_PORTB), Port 2 = $DC00 (CIA1_PORTA)
                // Bits are active-low on hardware, we invert for user-friendly API
                if !args.is_empty() {
                    // Generate port argument into A
                    self.generate_expression(&args[0])?;

                    // Check if port == 1
                    let port2_label = self.make_label("joy_port2");
                    let done_label = self.make_label("joy_done");

                    self.emit_byte(opcodes::CMP_IMM);
                    self.emit_byte(1);
                    self.emit_branch(opcodes::BNE, &port2_label);

                    // Port 1: read from $DC01
                    self.emit_abs(opcodes::LDA_ABS, cia::CIA1_PORTB);
                    self.emit_jmp(&done_label);

                    // Port 2: read from $DC00
                    self.define_label(&port2_label);
                    self.emit_abs(opcodes::LDA_ABS, cia::CIA1_PORTA);

                    // Done: invert and mask the result
                    self.define_label(&done_label);
                    // EOR #$1F inverts bits 0-4 (active-low -> active-high)
                    self.emit_byte(opcodes::EOR_IMM);
                    self.emit_byte(cia::JOY_MASK);
                    // AND #$1F keeps only joystick bits
                    self.emit_byte(opcodes::AND_IMM);
                    self.emit_byte(cia::JOY_MASK);
                }
            }
            "poke" => {
                if args.len() >= 2 {
                    // Value to poke
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::PHA);
                    // Address (A=low, X=high for word values)
                    // First set X to 0 as default for byte addresses
                    self.emit_imm(opcodes::LDX_IMM, 0);
                    self.generate_expression(&args[0])?;
                    // Store address (A=low byte, X=high byte)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Store value
                    self.emit_byte(opcodes::PLA);
                    self.emit_imm(opcodes::LDY_IMM, 0);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            "peek" => {
                if !args.is_empty() {
                    // Address (A=low, X=high for word values)
                    // First set X to 0 as default for byte addresses
                    self.emit_imm(opcodes::LDX_IMM, 0);
                    self.generate_expression(&args[0])?;
                    // Store address (A=low byte, X=high byte)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Load value
                    self.emit_imm(opcodes::LDY_IMM, 0);
                    self.emit_byte(opcodes::LDA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            "len" => {
                // len() works on both arrays and strings
                if !args.is_empty() {
                    let arg_type = self.infer_type_from_expr(&args[0]);

                    if arg_type == Type::String {
                        // String length: runtime calculation
                        // Generate expression to get string address in A/X
                        self.generate_expression(&args[0])?;

                        // Store string address in TMP1/TMP1_HI
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::STX_ZP);
                        self.emit_byte(zeropage::TMP1_HI);

                        // Call string length routine
                        // Result: A = length (byte)
                        self.emit_jsr_label("__str_len");
                    } else {
                        // Array length: compile-time constant
                        // Result: A = low byte, X = high byte
                        let array_size = self.get_array_length(&args[0], span)?;
                        self.emit_imm(opcodes::LDA_IMM, (array_size & 0xFF) as u8);
                        self.emit_imm(opcodes::LDX_IMM, ((array_size >> 8) & 0xFF) as u8);
                    }
                }
            }
            "rand" => {
                // rand() -> float (0.0 to 1.0 inclusive)
                self.emit_jsr_label("__rand");
            }
            "rand_byte" => {
                // rand_byte(from, to) -> byte
                if args.len() >= 2 {
                    self.generate_rand_byte(args, span)?;
                }
            }
            "rand_sbyte" => {
                // rand_sbyte(from, to) -> sbyte
                if args.len() >= 2 {
                    self.generate_rand_sbyte(args, span)?;
                }
            }
            "rand_word" => {
                // rand_word(from, to) -> word
                if args.len() >= 2 {
                    self.generate_rand_word(args, span)?;
                }
            }
            "rand_sword" => {
                // rand_sword(from, to) -> sword
                if args.len() >= 2 {
                    self.generate_rand_sword(args, span)?;
                }
            }
            "seed" => {
                // seed() - reseed the PRNG from hardware entropy
                self.emit_jsr_label("__prng_init");
            }
            "str_at" => {
                // str_at(s, i) -> byte - get character at position i
                if args.len() >= 2 {
                    // First, evaluate index and save it
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::PHA); // Save index on stack

                    // Evaluate string expression (address in A/X)
                    self.generate_expression(&args[0])?;

                    // Store string address in TMP1/TMP1_HI
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // Restore index into Y register
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::TAY);

                    // Load character at index: LDA (TMP1),Y
                    self.emit_byte(opcodes::LDA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            "sprite_enable" => {
                // sprite_enable(num: byte, enable: bool)
                // Sets or clears the bit for sprite 'num' in $D015
                if args.len() >= 2 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate enable flag
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2); // TMP2 = enable flag

                    // Restore sprite number and convert to bitmask
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::TAX); // X = sprite number

                    // Create bitmask: 1 << sprite_num
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    let shift_label = self.make_label("se_shift");
                    let done_shift_label = self.make_label("se_done_shift");
                    self.emit_byte(opcodes::CPX_IMM);
                    self.emit_byte(0);
                    self.emit_branch(opcodes::BEQ, &done_shift_label);
                    self.define_label(&shift_label);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::DEX);
                    self.emit_branch(opcodes::BNE, &shift_label);
                    self.define_label(&done_shift_label);
                    // A = bitmask

                    // Check if enable or disable
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = bitmask
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    let disable_label = self.make_label("se_disable");
                    let done_label = self.make_label("se_done");
                    self.emit_branch(opcodes::BEQ, &disable_label);

                    // Enable: OR the bitmask with current value
                    self.emit_abs(opcodes::LDA_ABS, sprite::ENABLE);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::ENABLE);
                    self.emit_jmp(&done_label);

                    // Disable: AND with inverted bitmask
                    self.define_label(&disable_label);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::EOR_IMM);
                    self.emit_byte(0xFF); // Invert bitmask
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sprite::ENABLE);
                    self.emit_byte(opcodes::AND_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::ENABLE);

                    self.define_label(&done_label);
                }
            }
            "sprites_enable" => {
                // sprites_enable(mask: byte) - directly write to $D015
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::ENABLE);
                }
            }
            "sprite_x" => {
                // sprite_x(num: byte, x: word)
                // Sets X position for sprite 'num' (0-511, 9-bit value)
                if args.len() >= 2 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate X position (word: A=low, X=high)
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = X low byte
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP2); // TMP2 = X high byte (MSB)

                    // Restore sprite number
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::ASL_ACC); // sprite_num * 2 for X register offset
                    self.emit_byte(opcodes::TAX);

                    // Store low byte at $D000 + sprite_num * 2
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abx(opcodes::STA_ABX, sprite::SPRITE0_X);

                    // Now handle MSB in $D010
                    // Create bitmask: 1 << sprite_num
                    self.emit_byte(opcodes::TXA);
                    self.emit_byte(opcodes::LSR_ACC); // Back to sprite_num
                    self.emit_byte(opcodes::TAX);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    let shift_label = self.make_label("sx_shift");
                    let done_shift_label = self.make_label("sx_done_shift");
                    self.emit_byte(opcodes::CPX_IMM);
                    self.emit_byte(0);
                    self.emit_branch(opcodes::BEQ, &done_shift_label);
                    self.define_label(&shift_label);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::DEX);
                    self.emit_branch(opcodes::BNE, &shift_label);
                    self.define_label(&done_shift_label);
                    // A = bitmask

                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = bitmask

                    // Check if MSB should be set (bit 0 of TMP2)
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    self.emit_imm(opcodes::AND_IMM, 0x01);
                    let clear_msb_label = self.make_label("sx_clear_msb");
                    let done_msb_label = self.make_label("sx_done_msb");
                    self.emit_branch(opcodes::BEQ, &clear_msb_label);

                    // Set MSB: OR bitmask
                    self.emit_abs(opcodes::LDA_ABS, sprite::X_MSB);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::X_MSB);
                    self.emit_jmp(&done_msb_label);

                    // Clear MSB: AND with inverted bitmask
                    self.define_label(&clear_msb_label);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::EOR_IMM);
                    self.emit_byte(0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sprite::X_MSB);
                    self.emit_byte(opcodes::AND_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::X_MSB);

                    self.define_label(&done_msb_label);
                }
            }
            "sprite_y" => {
                // sprite_y(num: byte, y: byte)
                // Sets Y position for sprite 'num' (0-255)
                if args.len() >= 2 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate Y position
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = Y value

                    // Restore sprite number
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::ASL_ACC); // sprite_num * 2 for register offset
                    self.emit_byte(opcodes::TAX);

                    // Store Y at $D001 + sprite_num * 2
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abx(opcodes::STA_ABX, sprite::SPRITE0_Y);
                }
            }
            "sprite_pos" => {
                // sprite_pos(num: byte, x: word, y: byte)
                // Convenience function to set both X and Y
                if args.len() >= 3 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate Y position and save it
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::PHA); // Save Y

                    // Evaluate X position (word: A=low, X=high)
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = X low
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP2); // TMP2 = X high (MSB flag)

                    // Restore Y position
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3); // TMP3 = Y

                    // Restore sprite number
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::ASL_ACC); // sprite_num * 2
                    self.emit_byte(opcodes::TAX);

                    // Store X low byte at $D000 + sprite_num * 2
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abx(opcodes::STA_ABX, sprite::SPRITE0_X);

                    // Store Y at $D001 + sprite_num * 2
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    self.emit_abx(opcodes::STA_ABX, sprite::SPRITE0_Y);

                    // Handle MSB in $D010
                    self.emit_byte(opcodes::TXA);
                    self.emit_byte(opcodes::LSR_ACC); // Back to sprite_num
                    self.emit_byte(opcodes::TAX);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    let shift_label = self.make_label("sp_shift");
                    let done_shift_label = self.make_label("sp_done_shift");
                    self.emit_byte(opcodes::CPX_IMM);
                    self.emit_byte(0);
                    self.emit_branch(opcodes::BEQ, &done_shift_label);
                    self.define_label(&shift_label);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::DEX);
                    self.emit_branch(opcodes::BNE, &shift_label);
                    self.define_label(&done_shift_label);

                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = bitmask

                    // Check if MSB should be set
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    self.emit_imm(opcodes::AND_IMM, 0x01);
                    let clear_msb_label = self.make_label("sp_clear_msb");
                    let done_msb_label = self.make_label("sp_done_msb");
                    self.emit_branch(opcodes::BEQ, &clear_msb_label);

                    // Set MSB
                    self.emit_abs(opcodes::LDA_ABS, sprite::X_MSB);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::X_MSB);
                    self.emit_jmp(&done_msb_label);

                    // Clear MSB
                    self.define_label(&clear_msb_label);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::EOR_IMM);
                    self.emit_byte(0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sprite::X_MSB);
                    self.emit_byte(opcodes::AND_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sprite::X_MSB);

                    self.define_label(&done_msb_label);
                }
            }
            "sprite_get_x" => {
                // sprite_get_x(num: byte) -> word
                // Returns X position (0-511) for sprite 'num'
                if !args.is_empty() {
                    // Evaluate sprite number
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3); // TMP3 = sprite_num (for MSB lookup)
                    self.emit_byte(opcodes::ASL_ACC); // sprite_num * 2
                    self.emit_byte(opcodes::TAX);

                    // Load low byte from $D000 + sprite_num * 2
                    self.emit_abx(opcodes::LDA_ABX, sprite::SPRITE0_X);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = X low byte

                    // Get MSB from $D010
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP3); // sprite_num
                    self.emit_byte(opcodes::TAX);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    let shift_label = self.make_label("sgx_shift");
                    let done_shift_label = self.make_label("sgx_done_shift");
                    self.emit_byte(opcodes::CPX_IMM);
                    self.emit_byte(0);
                    self.emit_branch(opcodes::BEQ, &done_shift_label);
                    self.define_label(&shift_label);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::DEX);
                    self.emit_branch(opcodes::BNE, &shift_label);
                    self.define_label(&done_shift_label);
                    // A = bitmask

                    // AND with MSB register
                    self.emit_abs(opcodes::AND_ABS, sprite::X_MSB);
                    let msb_clear_label = self.make_label("sgx_msb_clear");
                    self.emit_branch(opcodes::BEQ, &msb_clear_label);

                    // MSB is set: X = 1
                    self.emit_imm(opcodes::LDX_IMM, 1);
                    let done_label = self.make_label("sgx_done");
                    self.emit_jmp(&done_label);

                    // MSB is clear: X = 0
                    self.define_label(&msb_clear_label);
                    self.emit_imm(opcodes::LDX_IMM, 0);

                    self.define_label(&done_label);
                    // Load low byte into A
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Result: A = low byte, X = high byte
                }
            }
            "sprite_get_y" => {
                // sprite_get_y(num: byte) -> byte
                // Returns Y position (0-255) for sprite 'num'
                if !args.is_empty() {
                    // Evaluate sprite number
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::ASL_ACC); // sprite_num * 2
                    self.emit_byte(opcodes::TAX);

                    // Load Y from $D001 + sprite_num * 2
                    self.emit_abx(opcodes::LDA_ABX, sprite::SPRITE0_Y);
                }
            }
            "sprite_data" => {
                // sprite_data(num: byte, pointer: byte)
                // Sets sprite data pointer at $07F8 + sprite_num
                if args.len() >= 2 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate pointer value
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = pointer value

                    // Restore sprite number
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::TAX);

                    // Store pointer at $07F8 + sprite_num
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abx(opcodes::STA_ABX, sprite::POINTER_BASE);
                }
            }
            "sprite_get_data" => {
                // sprite_get_data(num: byte) -> byte
                // Returns sprite data pointer from $07F8 + sprite_num
                if !args.is_empty() {
                    // Evaluate sprite number
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::TAX);

                    // Load pointer from $07F8 + sprite_num
                    self.emit_abx(opcodes::LDA_ABX, sprite::POINTER_BASE);
                }
            }
            "sprite_color" => {
                // sprite_color(num: byte, color: byte)
                // Sets sprite color at $D027 + sprite_num
                if args.len() >= 2 {
                    // Evaluate sprite number and save it
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::PHA); // Save sprite number

                    // Evaluate color value
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1); // TMP1 = color

                    // Restore sprite number
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::TAX);

                    // Store color at $D027 + sprite_num
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abx(opcodes::STA_ABX, sprite::SPRITE0_COLOR);
                }
            }
            "sprite_get_color" => {
                // sprite_get_color(num: byte) -> byte
                // Returns sprite color from $D027 + sprite_num
                if !args.is_empty() {
                    // Evaluate sprite number
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::TAX);

                    // Load color from $D027 + sprite_num
                    self.emit_abx(opcodes::LDA_ABX, sprite::SPRITE0_COLOR);
                }
            }
            "sprite_multicolor1" => {
                // sprite_multicolor1(color: byte)
                // Sets shared multicolor 1 at $D025
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::MULTICOLOR1);
                }
            }
            "sprite_multicolor2" => {
                // sprite_multicolor2(color: byte)
                // Sets shared multicolor 2 at $D026
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::MULTICOLOR2);
                }
            }
            "sprite_get_multicolor1" => {
                // sprite_get_multicolor1() -> byte
                // Returns shared multicolor 1 from $D025
                self.emit_abs(opcodes::LDA_ABS, sprite::MULTICOLOR1);
            }
            "sprite_get_multicolor2" => {
                // sprite_get_multicolor2() -> byte
                // Returns shared multicolor 2 from $D026
                self.emit_abs(opcodes::LDA_ABS, sprite::MULTICOLOR2);
            }
            "sprite_multicolor" => {
                // sprite_multicolor(num: byte, enable: bool)
                // Sets or clears the multicolor bit for sprite 'num' in $D01C
                if args.len() >= 2 {
                    self.generate_sprite_bit_set(&args[0], &args[1], sprite::MULTICOLOR)?;
                }
            }
            "sprites_multicolor" => {
                // sprites_multicolor(mask: byte) - directly write to $D01C
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::MULTICOLOR);
                }
            }
            "sprite_is_multicolor" => {
                // sprite_is_multicolor(num: byte) -> bool
                if !args.is_empty() {
                    self.generate_sprite_bit_get(&args[0], sprite::MULTICOLOR)?;
                }
            }
            "sprite_expand_x" => {
                // sprite_expand_x(num: byte, expand: bool)
                // Sets or clears the X expansion bit for sprite 'num' in $D01D
                if args.len() >= 2 {
                    self.generate_sprite_bit_set(&args[0], &args[1], sprite::EXPAND_X)?;
                }
            }
            "sprite_expand_y" => {
                // sprite_expand_y(num: byte, expand: bool)
                // Sets or clears the Y expansion bit for sprite 'num' in $D017
                if args.len() >= 2 {
                    self.generate_sprite_bit_set(&args[0], &args[1], sprite::EXPAND_Y)?;
                }
            }
            "sprites_expand_x" => {
                // sprites_expand_x(mask: byte) - directly write to $D01D
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::EXPAND_X);
                }
            }
            "sprites_expand_y" => {
                // sprites_expand_y(mask: byte) - directly write to $D017
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::EXPAND_Y);
                }
            }
            "sprite_is_expanded_x" => {
                // sprite_is_expanded_x(num: byte) -> bool
                if !args.is_empty() {
                    self.generate_sprite_bit_get(&args[0], sprite::EXPAND_X)?;
                }
            }
            "sprite_is_expanded_y" => {
                // sprite_is_expanded_y(num: byte) -> bool
                if !args.is_empty() {
                    self.generate_sprite_bit_get(&args[0], sprite::EXPAND_Y)?;
                }
            }
            "sprite_priority" => {
                // sprite_priority(num: byte, behind_bg: bool)
                // Sets or clears the priority bit for sprite 'num' in $D01B
                // behind_bg=true means sprite appears behind background
                if args.len() >= 2 {
                    self.generate_sprite_bit_set(&args[0], &args[1], sprite::PRIORITY)?;
                }
            }
            "sprites_priority" => {
                // sprites_priority(mask: byte) - directly write to $D01B
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, sprite::PRIORITY);
                }
            }
            "sprite_get_priority" => {
                // sprite_get_priority(num: byte) -> bool
                // Returns true if sprite is behind background
                if !args.is_empty() {
                    self.generate_sprite_bit_get(&args[0], sprite::PRIORITY)?;
                }
            }
            "sprite_collision_sprite" => {
                // sprite_collision_sprite() -> byte
                // Read sprite-sprite collision register $D01E
                // Reading clears the register
                self.emit_abs(opcodes::LDA_ABS, sprite::COLLISION_SPRITE);
            }
            "sprite_collision_bg" => {
                // sprite_collision_bg() -> byte
                // Read sprite-background collision register $D01F
                // Reading clears the register
                self.emit_abs(opcodes::LDA_ABS, sprite::COLLISION_BG);
            }
            "sprite_collides" => {
                // sprite_collides(mask: byte) -> bool
                // Check if any sprite in mask has collision (sprite-sprite)
                if !args.is_empty() {
                    // Evaluate mask
                    self.generate_expression(&args[0])?;

                    // AND with collision register
                    self.emit_abs(opcodes::AND_ABS, sprite::COLLISION_SPRITE);

                    // Convert to boolean: 0 stays 0, non-zero becomes 1
                    let zero_label = self.make_label("sc_zero");
                    let done_label = self.make_label("sc_done");
                    self.emit_branch(opcodes::BEQ, &zero_label);
                    self.emit_imm(opcodes::LDA_IMM, 1); // true
                    self.emit_jmp(&done_label);
                    self.define_label(&zero_label);
                    self.emit_imm(opcodes::LDA_IMM, 0); // false
                    self.define_label(&done_label);
                }
            }

            // =================================================================
            // SID Sound Functions
            // =================================================================
            "sid_reset" => {
                // sid_reset() - clear all 25 SID registers
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_imm(opcodes::LDX_IMM, sid::REGISTER_COUNT - 1);
                let loop_label = self.make_label("sid_reset_loop");
                self.define_label(&loop_label);
                self.emit_abx(opcodes::STA_ABX, sid::BASE);
                self.emit_byte(opcodes::DEX);
                self.emit_branch(opcodes::BPL, &loop_label);
            }

            "sid_volume" => {
                // sid_volume(volume: byte) - set main volume (0-15)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Mask to low nibble and preserve high nibble (filter mode)
                    self.emit_imm(opcodes::AND_IMM, 0x0F);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sid::FILTER_MODE_VOLUME);
                    self.emit_imm(opcodes::AND_IMM, 0xF0); // Keep filter mode bits
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_MODE_VOLUME);
                }
            }

            "sid_frequency" => {
                // sid_frequency(voice: byte, frequency: word)
                if args.len() >= 2 {
                    self.generate_sid_frequency(&args[0], &args[1])?;
                }
            }

            "sid_waveform" => {
                // sid_waveform(voice: byte, waveform: byte)
                if args.len() >= 2 {
                    self.generate_sid_waveform(&args[0], &args[1])?;
                }
            }

            "sid_gate" => {
                // sid_gate(voice: byte, on: bool)
                if args.len() >= 2 {
                    self.generate_sid_gate(&args[0], &args[1])?;
                }
            }

            "sid_attack" => {
                // sid_attack(voice: byte, value: byte) - set attack (high nibble)
                if args.len() >= 2 {
                    self.generate_sid_attack(&args[0], &args[1])?;
                }
            }

            "sid_decay" => {
                // sid_decay(voice: byte, value: byte) - set decay (low nibble)
                if args.len() >= 2 {
                    self.generate_sid_decay(&args[0], &args[1])?;
                }
            }

            "sid_sustain" => {
                // sid_sustain(voice: byte, value: byte) - set sustain (high nibble)
                if args.len() >= 2 {
                    self.generate_sid_sustain(&args[0], &args[1])?;
                }
            }

            "sid_release" => {
                // sid_release(voice: byte, value: byte) - set release (low nibble)
                if args.len() >= 2 {
                    self.generate_sid_release(&args[0], &args[1])?;
                }
            }

            "sid_envelope" => {
                // sid_envelope(voice, attack, decay, sustain, release)
                if args.len() >= 5 {
                    self.generate_sid_envelope(&args[0], &args[1], &args[2], &args[3], &args[4])?;
                }
            }

            "sid_pulse_width" => {
                // sid_pulse_width(voice: byte, width: word) - 12-bit pulse width
                if args.len() >= 2 {
                    self.generate_sid_pulse_width(&args[0], &args[1])?;
                }
            }

            "sid_ring_mod" => {
                // sid_ring_mod(voice: byte, enable: bool)
                if args.len() >= 2 {
                    self.generate_sid_control_bit(&args[0], &args[1], sid::CTRL_RING_MOD)?;
                }
            }

            "sid_sync" => {
                // sid_sync(voice: byte, enable: bool)
                if args.len() >= 2 {
                    self.generate_sid_control_bit(&args[0], &args[1], sid::CTRL_SYNC)?;
                }
            }

            "sid_test" => {
                // sid_test(voice: byte, enable: bool)
                if args.len() >= 2 {
                    self.generate_sid_control_bit(&args[0], &args[1], sid::CTRL_TEST)?;
                }
            }

            "sid_filter_cutoff" => {
                // sid_filter_cutoff(frequency: word) - 11-bit cutoff
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // A = low byte, X = high byte
                    // Low 3 bits go to $D415, high 8 bits go to $D416
                    self.emit_byte(opcodes::PHA); // Save low byte
                    self.emit_imm(opcodes::AND_IMM, 0x07); // Low 3 bits
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_CUTOFF_LO);
                    self.emit_byte(opcodes::PLA); // Restore low byte
                                                  // Combine: (low >> 3) | (high << 5)
                    self.emit_byte(opcodes::LSR_ACC);
                    self.emit_byte(opcodes::LSR_ACC);
                    self.emit_byte(opcodes::LSR_ACC);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::TXA); // High byte
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_CUTOFF_HI);
                }
            }

            "sid_filter_resonance" => {
                // sid_filter_resonance(value: byte) - set resonance (0-15)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Shift to high nibble, preserve low nibble (voice routing)
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sid::FILTER_RESONANCE);
                    self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep voice routing
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_RESONANCE);
                }
            }

            "sid_filter_route" => {
                // sid_filter_route(voices: byte) - route voices through filter
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Low nibble = voice routing, preserve high nibble (resonance)
                    self.emit_imm(opcodes::AND_IMM, 0x0F);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sid::FILTER_RESONANCE);
                    self.emit_imm(opcodes::AND_IMM, 0xF0); // Keep resonance
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_RESONANCE);
                }
            }

            "sid_filter_mode" => {
                // sid_filter_mode(mode: byte) - set filter mode (LP/BP/HP)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // High nibble = filter mode, preserve low nibble (volume)
                    self.emit_imm(opcodes::AND_IMM, 0xF0);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::LDA_ABS, sid::FILTER_MODE_VOLUME);
                    self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep volume
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, sid::FILTER_MODE_VOLUME);
                }
            }

            "play_note" => {
                // play_note(voice: byte, note: byte, octave: byte)
                if args.len() >= 3 {
                    self.generate_play_note(&args[0], &args[1], &args[2])?;
                }
            }

            "play_tone" => {
                // play_tone(voice, frequency, waveform, duration)
                if args.len() >= 4 {
                    self.generate_play_tone(&args[0], &args[1], &args[2], &args[3])?;
                }
            }

            "sound_off" => {
                // sound_off() - silence all voices (clear gate bits)
                self.emit_abs(opcodes::LDA_ABS, sid::VOICE1_CTRL);
                self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
                self.emit_abs(opcodes::STA_ABS, sid::VOICE1_CTRL);
                self.emit_abs(opcodes::LDA_ABS, sid::VOICE2_CTRL);
                self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
                self.emit_abs(opcodes::STA_ABS, sid::VOICE2_CTRL);
                self.emit_abs(opcodes::LDA_ABS, sid::VOICE3_CTRL);
                self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
                self.emit_abs(opcodes::STA_ABS, sid::VOICE3_CTRL);
            }

            "sound_off_voice" => {
                // sound_off_voice(voice: byte) - silence specific voice
                if !args.is_empty() {
                    self.generate_sound_off_voice(&args[0])?;
                }
            }

            // =================================================================
            // VIC-II Graphics Functions
            // =================================================================
            "border_color" => {
                // border_color(color: byte) - set border color
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, vic::BORDER);
                }
            }

            "background_color" => {
                // background_color(color: byte) - set background color
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, vic::BACKGROUND0);
                }
            }

            "get_border_color" => {
                // get_border_color() -> byte - get current border color
                self.emit_abs(opcodes::LDA_ABS, vic::BORDER);
            }

            "get_background_color" => {
                // get_background_color() -> byte - get current background color
                self.emit_abs(opcodes::LDA_ABS, vic::BACKGROUND0);
            }

            "gfx_mode" => {
                // gfx_mode(mode: byte) - switch to graphics mode (0-4)
                // Mode 0: ECM=0, BMM=0, MCM=0 (Standard Text)
                // Mode 1: ECM=0, BMM=0, MCM=1 (Multicolor Text)
                // Mode 2: ECM=0, BMM=1, MCM=0 (Hires Bitmap)
                // Mode 3: ECM=0, BMM=1, MCM=1 (Multicolor Bitmap)
                // Mode 4: ECM=1, BMM=0, MCM=0 (ECM Text)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_jsr_label("__gfx_mode");
                }
            }

            "get_gfx_mode" => {
                // get_gfx_mode() -> byte - get current graphics mode
                self.emit_jsr_label("__get_gfx_mode");
            }

            "gfx_text" => {
                // gfx_text() - switch to standard text mode (mode 0)
                self.emit_imm(opcodes::LDA_IMM, vic::MODE_TEXT);
                self.emit_jsr_label("__gfx_mode");
            }

            "gfx_hires" => {
                // gfx_hires() - switch to hires bitmap mode (mode 2)
                self.emit_imm(opcodes::LDA_IMM, vic::MODE_BITMAP);
                self.emit_jsr_label("__gfx_mode");
            }

            "gfx_multicolor" => {
                // gfx_multicolor() - switch to multicolor bitmap mode (mode 3)
                self.emit_imm(opcodes::LDA_IMM, vic::MODE_BITMAP_MC);
                self.emit_jsr_label("__gfx_mode");
            }

            "screen_columns" => {
                // screen_columns(cols: byte) - set 38 or 40 column mode
                // Bit 3 of $D016: 0 = 38 columns, 1 = 40 columns
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Compare with 40
                    self.emit_imm(opcodes::CMP_IMM, 40);
                    let label_38col = self.make_label("col38");
                    let label_done = self.make_label("coldone");
                    self.emit_branch(opcodes::BNE, &label_38col);
                    // 40 columns: set bit 3
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
                    self.emit_imm(opcodes::ORA_IMM, vic::CSEL);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL2);
                    self.emit_jmp(&label_done);
                    // 38 columns: clear bit 3
                    self.define_label(&label_38col);
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
                    self.emit_imm(opcodes::AND_IMM, !vic::CSEL);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL2);
                    self.define_label(&label_done);
                }
            }

            "screen_rows" => {
                // screen_rows(rows: byte) - set 24 or 25 row mode
                // Bit 3 of $D011: 0 = 24 rows, 1 = 25 rows
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Compare with 25
                    self.emit_imm(opcodes::CMP_IMM, 25);
                    let label_24row = self.make_label("row24");
                    let label_done = self.make_label("rowdone");
                    self.emit_branch(opcodes::BNE, &label_24row);
                    // 25 rows: set bit 3
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
                    self.emit_imm(opcodes::ORA_IMM, vic::RSEL);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL1);
                    self.emit_jmp(&label_done);
                    // 24 rows: clear bit 3
                    self.define_label(&label_24row);
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
                    self.emit_imm(opcodes::AND_IMM, !vic::RSEL);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL1);
                    self.define_label(&label_done);
                }
            }

            "vic_bank" => {
                // vic_bank(bank: byte) - set VIC memory bank (0-3)
                // CIA2 Port A bits 0-1 control VIC bank (inverted):
                // Bank 0 ($0000-$3FFF) = %11, Bank 1 ($4000-$7FFF) = %10
                // Bank 2 ($8000-$BFFF) = %01, Bank 3 ($C000-$FFFF) = %00
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // Invert the bank number (XOR with 3)
                    self.emit_imm(opcodes::EOR_IMM, 0x03);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Read current CIA2 Port A
                    self.emit_abs(opcodes::LDA_ABS, cia::CIA2_PRA);
                    // Clear bits 0-1
                    self.emit_imm(opcodes::AND_IMM, !cia::VIC_BANK_MASK);
                    // OR in the new bank value
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Write back
                    self.emit_abs(opcodes::STA_ABS, cia::CIA2_PRA);
                }
            }

            "get_vic_bank" => {
                // get_vic_bank() -> byte - get current VIC bank (0-3)
                // Read CIA2 Port A bits 0-1 and invert
                self.emit_abs(opcodes::LDA_ABS, cia::CIA2_PRA);
                self.emit_imm(opcodes::AND_IMM, cia::VIC_BANK_MASK);
                self.emit_imm(opcodes::EOR_IMM, 0x03); // Invert
            }

            "screen_address" => {
                // screen_address(addr: word) - set screen RAM address
                // $D018 bits 4-7 = screen address / 1024
                // addr must be within current VIC bank (0-16383 offset)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // A = low byte, X = high byte
                    // We need (addr / 1024) << 4 = (addr >> 10) << 4 = addr >> 6
                    // But we need high nibble, so take high byte and mask upper 4 bits
                    self.emit_byte(opcodes::TXA); // A = high byte
                    self.emit_imm(opcodes::AND_IMM, 0x3C); // Mask to get bits 2-5 (screen addr bits)
                    self.emit_byte(opcodes::ASL_ACC); // Shift left to bits 4-7
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Read current $D018, clear bits 4-7, OR in new value
                    self.emit_abs(opcodes::LDA_ABS, vic::MEMORY);
                    self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep lower nibble
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, vic::MEMORY);
                }
            }

            "bitmap_address" => {
                // bitmap_address(addr: word) - set bitmap address
                // $D018 bit 3 = bitmap at offset 0 (0) or 8192 (1)
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // A = low byte, X = high byte
                    // Check if high byte >= $20 (8192 = $2000)
                    self.emit_byte(opcodes::TXA);
                    self.emit_imm(opcodes::CMP_IMM, 0x20);
                    let label_low = self.make_label("bmp_low");
                    let label_done = self.make_label("bmp_done");
                    self.emit_branch(opcodes::BCC, &label_low);
                    // Bitmap at 8192: set bit 3
                    self.emit_abs(opcodes::LDA_ABS, vic::MEMORY);
                    self.emit_imm(opcodes::ORA_IMM, 0x08);
                    self.emit_abs(opcodes::STA_ABS, vic::MEMORY);
                    self.emit_jmp(&label_done);
                    // Bitmap at 0: clear bit 3
                    self.define_label(&label_low);
                    self.emit_abs(opcodes::LDA_ABS, vic::MEMORY);
                    self.emit_imm(opcodes::AND_IMM, !0x08);
                    self.emit_abs(opcodes::STA_ABS, vic::MEMORY);
                    self.define_label(&label_done);
                }
            }

            "charset_address" => {
                // charset_address(addr: word) - set character set address
                // $D018 bits 1-3 = charset address / 2048
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    // A = low byte, X = high byte
                    // We need (addr / 2048) << 1 = (addr >> 11) << 1 = addr >> 10
                    // Take high byte, shift right 2, mask bits 1-3
                    self.emit_byte(opcodes::TXA); // A = high byte
                    self.emit_byte(opcodes::LSR_ACC); // >> 1
                    self.emit_byte(opcodes::LSR_ACC); // >> 2
                    self.emit_imm(opcodes::AND_IMM, 0x0E); // Mask bits 1-3
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Read current $D018, clear bits 1-3, OR in new value
                    self.emit_abs(opcodes::LDA_ABS, vic::MEMORY);
                    self.emit_imm(opcodes::AND_IMM, 0xF1); // Clear bits 1-3
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, vic::MEMORY);
                }
            }

            // =================================================================
            // Bitmap Graphics Functions
            // =================================================================
            "plot" => {
                // plot(x: word, y: byte) - set pixel in hires mode
                if args.len() >= 2 {
                    // Store Y coordinate in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store X coordinate in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call plot routine
                    self.emit_jsr_label("__plot");
                }
            }

            "unplot" => {
                // unplot(x: word, y: byte) - clear pixel in hires mode
                if args.len() >= 2 {
                    // Store Y coordinate in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store X coordinate in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call unplot routine
                    self.emit_jsr_label("__unplot");
                }
            }

            "point" => {
                // point(x: word, y: byte) -> bool - test if pixel is set
                if args.len() >= 2 {
                    // Store Y coordinate in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store X coordinate in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call point routine (returns 0 or 1 in A)
                    self.emit_jsr_label("__point");
                }
            }

            "plot_mc" => {
                // plot_mc(x: byte, y: byte, color: byte) - set multicolor pixel
                if args.len() >= 3 {
                    // Store color in TMP4
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    // Store Y coordinate in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store X coordinate in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call plot_mc routine
                    self.emit_jsr_label("__plot_mc");
                }
            }

            "point_mc" => {
                // point_mc(x: byte, y: byte) -> byte - get multicolor pixel color
                if args.len() >= 2 {
                    // Store Y coordinate in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store X coordinate in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call point_mc routine (returns color 0-3 in A)
                    self.emit_jsr_label("__point_mc");
                }
            }

            "bitmap_clear" => {
                // bitmap_clear() - clear entire bitmap
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_jsr_label("__bitmap_fill");
            }

            "bitmap_fill" => {
                // bitmap_fill(pattern: byte) - fill entire bitmap with pattern
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_jsr_label("__bitmap_fill");
                }
            }

            // =================================================================
            // Drawing Primitives
            // =================================================================
            "line" => {
                // line(x1: word, y1: byte, x2: word, y2: byte)
                // Uses Bresenham's line algorithm
                if args.len() >= 4 {
                    // Store y2 in TMP5
                    self.generate_expression(&args[3])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP5);
                    // Store x2 in stack (will use runtime vars)
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::PHA); // x2 low
                    self.emit_byte(opcodes::TXA);
                    self.emit_byte(opcodes::PHA); // x2 high
                                                  // Store y1 in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store x1 in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Pop x2 into TMP2/TMP2_HI
                    self.emit_byte(opcodes::PLA); // x2 high
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2_HI);
                    self.emit_byte(opcodes::PLA); // x2 low
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    // Call line routine
                    self.emit_jsr_label("__line");
                }
            }

            "hline" => {
                // hline(x: word, y: byte, length: word) - fast horizontal line
                if args.len() >= 3 {
                    // Store length in TMP4/TMP5
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP5);
                    // Store y in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store x in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call hline routine
                    self.emit_jsr_label("__hline");
                }
            }

            "vline" => {
                // vline(x: word, y: byte, length: byte) - fast vertical line
                if args.len() >= 3 {
                    // Store length in TMP4
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    // Store y in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store x in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call vline routine
                    self.emit_jsr_label("__vline");
                }
            }

            "rect" => {
                // rect(x: word, y: byte, width: word, height: byte)
                if args.len() >= 4 {
                    // Store height
                    self.generate_expression(&args[3])?;
                    self.emit_byte(opcodes::PHA);
                    // Store width
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP5);
                    // Store y
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store x
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Pop height into A and push to runtime location
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2); // Use TMP2 for height
                                                    // Call rect routine
                    self.emit_jsr_label("__rect");
                }
            }

            "rect_fill" => {
                // rect_fill(x: word, y: byte, width: word, height: byte)
                if args.len() >= 4 {
                    // Store height
                    self.generate_expression(&args[3])?;
                    self.emit_byte(opcodes::PHA);
                    // Store width
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP5);
                    // Store y
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store x
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Pop height
                    self.emit_byte(opcodes::PLA);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP2);
                    // Call rect_fill routine
                    self.emit_jsr_label("__rect_fill");
                }
            }

            // =================================================================
            // Cell Color Control
            // =================================================================
            "cell_color" => {
                // cell_color(cx: byte, cy: byte, fg: byte, bg: byte)
                // Set foreground (high nibble) and background (low nibble) at screen RAM position
                if args.len() >= 4 {
                    // Calculate combined color: (fg << 4) | bg
                    // Store bg in TMP4
                    self.generate_expression(&args[3])?;
                    self.emit_imm(opcodes::AND_IMM, 0x0F); // Mask to lower nibble
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    // Get fg, shift left 4, combine with bg
                    self.generate_expression(&args[2])?;
                    self.emit_imm(opcodes::AND_IMM, 0x0F); // Mask to lower nibble
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4); // TMP4 = combined color
                                                    // Store cy in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store cx in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call cell_color routine
                    self.emit_jsr_label("__cell_color");
                }
            }

            "get_cell_color" => {
                // get_cell_color(cx: byte, cy: byte) -> byte
                if args.len() >= 2 {
                    // Store cy in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store cx in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call get_cell_color routine, result in A
                    self.emit_jsr_label("__get_cell_color");
                }
            }

            "color_ram" => {
                // color_ram(cx: byte, cy: byte, color: byte)
                if args.len() >= 3 {
                    // Store color in TMP4
                    self.generate_expression(&args[2])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP4);
                    // Store cy in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store cx in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call color_ram routine
                    self.emit_jsr_label("__color_ram");
                }
            }

            "get_color_ram" => {
                // get_color_ram(cx: byte, cy: byte) -> byte
                if args.len() >= 2 {
                    // Store cy in TMP3
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP3);
                    // Store cx in TMP1
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Call get_color_ram routine, result in A
                    self.emit_jsr_label("__get_color_ram");
                }
            }

            "fill_colors" => {
                // fill_colors(fg: byte, bg: byte)
                if args.len() >= 2 {
                    // Calculate combined color: (fg << 4) | bg
                    self.generate_expression(&args[1])?;
                    self.emit_imm(opcodes::AND_IMM, 0x0F);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.generate_expression(&args[0])?;
                    self.emit_imm(opcodes::AND_IMM, 0x0F);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ASL_ACC);
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // A = combined color, call fill routine
                    self.emit_jsr_label("__fill_colors");
                }
            }

            "fill_color_ram" => {
                // fill_color_ram(color: byte)
                if args.len() >= 1 {
                    self.generate_expression(&args[0])?;
                    // A = color, call fill routine
                    self.emit_jsr_label("__fill_color_ram");
                }
            }

            // =================================================================
            // Hardware Scrolling
            // =================================================================
            "scroll_x" => {
                // scroll_x(offset: byte) - set horizontal scroll (0-7)
                // Modifies bits 0-2 of $D016, preserves bits 3-7
                if args.len() >= 1 {
                    self.generate_expression(&args[0])?;
                    self.emit_imm(opcodes::AND_IMM, 0x07); // Mask to 0-7
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Read current $D016, clear bits 0-2, OR with new value
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
                    self.emit_imm(opcodes::AND_IMM, 0xF8); // Clear bits 0-2
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL2);
                }
            }

            "scroll_y" => {
                // scroll_y(offset: byte) - set vertical scroll (0-7)
                // Modifies bits 0-2 of $D011, preserves bits 3-7
                if args.len() >= 1 {
                    self.generate_expression(&args[0])?;
                    self.emit_imm(opcodes::AND_IMM, 0x07); // Mask to 0-7
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Read current $D011, clear bits 0-2, OR with new value
                    self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
                    self.emit_imm(opcodes::AND_IMM, 0xF8); // Clear bits 0-2
                    self.emit_byte(opcodes::ORA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_abs(opcodes::STA_ABS, vic::CONTROL1);
                }
            }

            "get_scroll_x" => {
                // get_scroll_x() -> byte - get horizontal scroll (0-7)
                // Read bits 0-2 of $D016
                self.emit_abs(opcodes::LDA_ABS, vic::CONTROL2);
                self.emit_imm(opcodes::AND_IMM, 0x07);
            }

            "get_scroll_y" => {
                // get_scroll_y() -> byte - get vertical scroll (0-7)
                // Read bits 0-2 of $D011
                self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
                self.emit_imm(opcodes::AND_IMM, 0x07);
            }

            // =================================================================
            // Raster Functions
            // =================================================================
            "raster" => {
                // raster() -> word - get current raster line
                // Low byte from $D012, bit 8 from bit 7 of $D011
                // Result: A = low byte, X = high byte (0 or 1)
                self.emit_abs(opcodes::LDA_ABS, vic::CONTROL1);
                self.emit_imm(opcodes::AND_IMM, 0x80); // Get bit 7
                self.emit_byte(opcodes::ASL_ACC); // Shift into carry
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_byte(opcodes::ROL_ACC); // Rotate carry into bit 0
                self.emit_byte(opcodes::TAX); // X = high byte (0 or 1)
                self.emit_abs(opcodes::LDA_ABS, vic::RASTER); // A = low byte
            }

            "wait_raster" => {
                // wait_raster(line: word) - wait until raster reaches specific line
                if args.len() >= 1 {
                    // Store target line in TMP1/TMP1_HI
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Call wait_raster routine
                    self.emit_jsr_label("__wait_raster");
                }
            }

            // =================================================================
            // Extended Background Color Mode (ECM)
            // =================================================================
            "ecm_background" => {
                // ecm_background(index: byte, color: byte)
                // index 0-3 maps to $D021-$D024
                if args.len() >= 2 {
                    // Store color in TMP1
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Get index
                    self.generate_expression(&args[0])?;
                    self.emit_imm(opcodes::AND_IMM, 0x03); // Mask to 0-3
                    self.emit_byte(opcodes::TAX);
                    // Load color
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // Store to $D021 + index
                    self.emit_abs(opcodes::STA_ABX, vic::BACKGROUND0);
                }
            }

            "get_ecm_background" => {
                // get_ecm_background(index: byte) -> byte
                if args.len() >= 1 {
                    self.generate_expression(&args[0])?;
                    self.emit_imm(opcodes::AND_IMM, 0x03); // Mask to 0-3
                    self.emit_byte(opcodes::TAX);
                    // Load from $D021 + index
                    self.emit_abs(opcodes::LDA_ABX, vic::BACKGROUND0);
                }
            }

            _ => {
                // User-defined function
                if let Some(func_info) = self.functions.get(name).cloned() {
                    // Store arguments in parameter memory locations
                    for (i, arg) in args.iter().enumerate() {
                        if i < func_info.param_addresses.len() {
                            let param_type = &func_info.params[i];
                            let param_addr = func_info.param_addresses[i];

                            // Generate the argument value (result in A, or A/X for 16-bit)
                            self.generate_expression(arg)?;

                            // Store the value at the parameter address
                            self.emit_store_to_address(param_addr, param_type);
                        }
                    }
                    // Call function
                    self.emit_jsr_label(name);
                } else {
                    return Err(CompileError::new(
                        ErrorCode::UndefinedFunction,
                        format!("Undefined function '{}'", name),
                        span.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Helper methods for random number generation.
impl CodeGenerator {
    /// Generate code for rand_byte(from, to).
    ///
    /// Uses rejection sampling for uniform distribution.
    /// Special case: when from=0 and to=255, range overflows to 0,
    /// so we just return the raw random byte.
    fn generate_rand_byte(&mut self, args: &[Expr], _span: &Span) -> Result<(), CompileError> {
        // Evaluate 'to' first and save it
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = to

        // Evaluate 'from' and save it
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = from

        // Calculate range = to - from + 1
        // Note: if from=0 and to=255, range = 256 which overflows to 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // TMP3 = range (0 means full 256)

        // Check for full range (range == 0 means 256)
        let full_range_label = self.make_label("rb_full");
        let done_label = self.make_label("rb_done");
        self.emit_branch(opcodes::BEQ, &full_range_label);

        // Normal case: rejection sampling loop
        let retry_label = self.make_label("rb_retry");
        self.define_label(&retry_label);

        // Get random byte
        self.emit_jsr_label("__prng_next");

        // Check if value < range
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCS, &retry_label); // If >= range, retry

        // Add 'from' to get final result
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_jmp(&done_label);

        // Full range case: just return random byte (no rejection needed)
        self.define_label(&full_range_label);
        self.emit_jsr_label("__prng_next");
        // from is always 0 when range overflows, so no need to add

        self.define_label(&done_label);

        Ok(())
    }

    /// Generate code for rand_sbyte(from, to).
    ///
    /// Special case: when from=-128 and to=127, range overflows to 0,
    /// so we just return the raw random byte interpreted as signed.
    fn generate_rand_sbyte(&mut self, args: &[Expr], _span: &Span) -> Result<(), CompileError> {
        // Similar to rand_byte but with signed values
        // We convert to unsigned range, generate, then convert back

        // Evaluate 'to' and 'from'
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = to

        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = from

        // Calculate unsigned range = to - from + 1
        // Note: if from=-128 and to=127, range = 256 which overflows to 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // TMP3 = range (0 means full 256)

        // Check for full range (range == 0 means 256)
        let full_range_label = self.make_label("rsb_full");
        let done_label = self.make_label("rsb_done");
        self.emit_branch(opcodes::BEQ, &full_range_label);

        // Normal case: rejection sampling loop
        let retry_label = self.make_label("rsb_retry");
        self.define_label(&retry_label);

        // Get random byte
        self.emit_jsr_label("__prng_next");

        // Check if value < range
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCS, &retry_label);

        // Add 'from' to get final result (signed addition works the same)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_jmp(&done_label);

        // Full range case: just return random byte (interpreted as signed)
        self.define_label(&full_range_label);
        self.emit_jsr_label("__prng_next");
        // The random byte is already in the range -128 to 127 when interpreted as signed

        self.define_label(&done_label);

        Ok(())
    }

    /// Generate code for rand_word(from, to).
    ///
    /// Special case: when from=0 and to=65535, range overflows to 0,
    /// so we just return two random bytes.
    fn generate_rand_word(&mut self, args: &[Expr], _span: &Span) -> Result<(), CompileError> {
        // Evaluate 'to' (16-bit) and save it
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI); // TMP2 = to (16-bit)

        // Evaluate 'from' (16-bit) and save it
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI); // TMP1 = from (16-bit)

        // Calculate range = to - from + 1 (16-bit)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Add 1 to range (may overflow to 0x0000 if range was 0xFFFF)
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);
        let no_carry_label = self.make_label("rw_nc");
        self.emit_branch(opcodes::BNE, &no_carry_label);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.define_label(&no_carry_label);

        // Check for full range (both bytes zero after increment = 65536)
        let full_range_label = self.make_label("rw_full");
        let done_label = self.make_label("rw_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BEQ, &full_range_label);

        // Normal case: rejection sampling loop for 16-bit
        let retry_label = self.make_label("rw_retry");
        self.define_label(&retry_label);

        // Get two random bytes
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // Low byte
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5); // High byte

        // Compare with range (16-bit unsigned comparison)
        // If random >= range, retry
        let ok_label = self.make_label("rw_ok");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BCC, &ok_label); // High byte < range high, OK
        self.emit_branch(opcodes::BNE, &retry_label.clone()); // High byte > range high, retry

        // High bytes equal, compare low bytes
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCS, &retry_label); // Low >= range low, retry

        self.define_label(&ok_label);
        // Add 'from' to random value
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::TAX); // X = high byte
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4); // A = low byte
        self.emit_jmp(&done_label);

        // Full range case: just return two random bytes
        self.define_label(&full_range_label);
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // A = low byte
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::TAX); // X = high byte
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);

        self.define_label(&done_label);

        Ok(())
    }

    /// Generate code for rand_sword(from, to).
    ///
    /// Special case: when from=-32768 and to=32767, range overflows to 0,
    /// so we just return two random bytes interpreted as signed.
    fn generate_rand_sword(&mut self, args: &[Expr], _span: &Span) -> Result<(), CompileError> {
        // Similar to rand_word but with signed values
        // The range calculation works the same way for signed values

        // Evaluate 'to' (16-bit) and save it
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Evaluate 'from' (16-bit) and save it
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Calculate range = to - from + 1 (16-bit, works for signed)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Add 1 to range (may overflow to 0x0000 if range was 0xFFFF)
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);
        let no_carry_label = self.make_label("rsw_nc");
        self.emit_branch(opcodes::BNE, &no_carry_label);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.define_label(&no_carry_label);

        // Check for full range (both bytes zero after increment = 65536)
        let full_range_label = self.make_label("rsw_full");
        let done_label = self.make_label("rsw_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BEQ, &full_range_label);

        // Normal case: rejection sampling loop
        let retry_label = self.make_label("rsw_retry");
        self.define_label(&retry_label);

        // Get two random bytes
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP5);

        // Compare with range (unsigned comparison for the range value)
        let ok_label = self.make_label("rsw_ok");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BCC, &ok_label);
        self.emit_branch(opcodes::BNE, &retry_label.clone());

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BCS, &retry_label);

        self.define_label(&ok_label);
        // Add 'from' to random value (signed addition)
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP5);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_jmp(&done_label);

        // Full range case: just return two random bytes (interpreted as signed)
        self.define_label(&full_range_label);
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_jsr_label("__prng_next");
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);

        self.define_label(&done_label);

        Ok(())
    }
}

/// Helper methods for sprite bit manipulation.
impl CodeGenerator {
    /// Generate code to set/clear a bit in a sprite control register.
    ///
    /// Used for multicolor, expand_x, expand_y, priority, etc.
    /// Sets or clears the bit for sprite 'num' based on 'enable'.
    fn generate_sprite_bit_set(
        &mut self,
        num_expr: &Expr,
        enable_expr: &Expr,
        register: u16,
    ) -> Result<(), CompileError> {
        // Evaluate sprite number and save it
        self.generate_expression(num_expr)?;
        self.emit_byte(opcodes::PHA); // Save sprite number

        // Evaluate enable flag
        self.generate_expression(enable_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = enable flag

        // Restore sprite number and convert to bitmask
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Create bitmask: 1 << sprite_num
        self.emit_imm(opcodes::LDA_IMM, 1);
        let shift_label = self.make_label("sbs_shift");
        let done_shift_label = self.make_label("sbs_done_shift");
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &done_shift_label);
        self.define_label(&shift_label);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &shift_label);
        self.define_label(&done_shift_label);
        // A = bitmask

        // Check if enable or disable
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = bitmask
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        let disable_label = self.make_label("sbs_disable");
        let done_label = self.make_label("sbs_done");
        self.emit_branch(opcodes::BEQ, &disable_label);

        // Enable: OR the bitmask with current value
        self.emit_abs(opcodes::LDA_ABS, register);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABS, register);
        self.emit_jmp(&done_label);

        // Disable: AND with inverted bitmask
        self.define_label(&disable_label);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::EOR_IMM);
        self.emit_byte(0xFF); // Invert bitmask
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::LDA_ABS, register);
        self.emit_byte(opcodes::AND_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABS, register);

        self.define_label(&done_label);

        Ok(())
    }

    /// Generate code to get a bit from a sprite control register.
    ///
    /// Returns 1 (true) if the bit is set, 0 (false) otherwise.
    fn generate_sprite_bit_get(
        &mut self,
        num_expr: &Expr,
        register: u16,
    ) -> Result<(), CompileError> {
        // Evaluate sprite number
        self.generate_expression(num_expr)?;
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Create bitmask: 1 << sprite_num
        self.emit_imm(opcodes::LDA_IMM, 1);
        let shift_label = self.make_label("sbg_shift");
        let done_shift_label = self.make_label("sbg_done_shift");
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &done_shift_label);
        self.define_label(&shift_label);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &shift_label);
        self.define_label(&done_shift_label);
        // A = bitmask

        // AND with register to check if bit is set
        self.emit_abs(opcodes::AND_ABS, register);

        // Convert to boolean: 0 stays 0, non-zero becomes 1
        let zero_label = self.make_label("sbg_zero");
        let done_label = self.make_label("sbg_done");
        self.emit_branch(opcodes::BEQ, &zero_label);
        self.emit_imm(opcodes::LDA_IMM, 1); // true
        self.emit_jmp(&done_label);
        self.define_label(&zero_label);
        self.emit_imm(opcodes::LDA_IMM, 0); // false
        self.define_label(&done_label);

        Ok(())
    }
}

/// Helper methods for SID sound generation.
impl CodeGenerator {
    /// Generate code for sid_frequency(voice, frequency).
    fn generate_sid_frequency(
        &mut self,
        voice_expr: &Expr,
        freq_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate frequency first (16-bit: A=low, X=high)
        self.generate_expression(freq_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = freq low
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI); // TMP1_HI = freq high

        // Evaluate voice number
        self.generate_expression(voice_expr)?;

        // Calculate base address: voice * 7
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // Save voice number
        self.emit_byte(opcodes::ASL_ACC); // * 2
        self.emit_byte(opcodes::ASL_ACC); // * 4
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2); // * 4 + voice = * 5
        self.emit_byte(opcodes::ASL_ACC); // * 10... wait, that's wrong

        // Simpler: multiply by 7 = voice + voice*2 + voice*4
        // Actually: voice*7 = (voice << 3) - voice
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC); // * 2
        self.emit_byte(opcodes::ASL_ACC); // * 4
        self.emit_byte(opcodes::ASL_ACC); // * 8
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2); // * 8 - voice = * 7
        self.emit_byte(opcodes::TAY); // Y = offset

        // Store freq low byte
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        // Store freq high byte
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_waveform(voice, waveform).
    fn generate_sid_waveform(
        &mut self,
        voice_expr: &Expr,
        waveform_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate waveform
        self.generate_expression(waveform_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = waveform

        // Evaluate voice and calculate control register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        // voice * 7
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY); // Y = control register offset

        // Read current control register (preserve gate bit)
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep low nibble (gate, sync, ring, test)
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1); // OR with waveform
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_gate(voice, on).
    fn generate_sid_gate(&mut self, voice_expr: &Expr, on_expr: &Expr) -> Result<(), CompileError> {
        // Evaluate on/off flag
        self.generate_expression(on_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = on flag

        // Evaluate voice and calculate control register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY); // Y = control register offset

        // Check if gate on or off
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        let off_label = self.make_label("sg_off");
        let done_label = self.make_label("sg_done");
        self.emit_branch(opcodes::BEQ, &off_label);

        // Gate ON: set bit 0
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::ORA_IMM, sid::CTRL_GATE);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);
        self.emit_jmp(&done_label);

        // Gate OFF: clear bit 0
        self.define_label(&off_label);
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for sid_attack(voice, value).
    fn generate_sid_attack(
        &mut self,
        voice_expr: &Expr,
        value_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate value and shift to high nibble
        self.generate_expression(value_expr)?;
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = attack << 4

        // Calculate AD register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_ATTACK_DECAY as u8);
        self.emit_byte(opcodes::TAY);

        // Read, modify, write
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep decay
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_decay(voice, value).
    fn generate_sid_decay(
        &mut self,
        voice_expr: &Expr,
        value_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate value (low nibble)
        self.generate_expression(value_expr)?;
        self.emit_imm(opcodes::AND_IMM, 0x0F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Calculate AD register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_ATTACK_DECAY as u8);
        self.emit_byte(opcodes::TAY);

        // Read, modify, write
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, 0xF0); // Keep attack
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_sustain(voice, value).
    fn generate_sid_sustain(
        &mut self,
        voice_expr: &Expr,
        value_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate value and shift to high nibble
        self.generate_expression(value_expr)?;
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Calculate SR register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_SUSTAIN_RELEASE as u8);
        self.emit_byte(opcodes::TAY);

        // Read, modify, write
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, 0x0F); // Keep release
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_release(voice, value).
    fn generate_sid_release(
        &mut self,
        voice_expr: &Expr,
        value_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate value (low nibble)
        self.generate_expression(value_expr)?;
        self.emit_imm(opcodes::AND_IMM, 0x0F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Calculate SR register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_SUSTAIN_RELEASE as u8);
        self.emit_byte(opcodes::TAY);

        // Read, modify, write
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, 0xF0); // Keep sustain
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for sid_envelope(voice, attack, decay, sustain, release).
    fn generate_sid_envelope(
        &mut self,
        voice_expr: &Expr,
        attack_expr: &Expr,
        decay_expr: &Expr,
        sustain_expr: &Expr,
        release_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Combine attack and decay
        self.generate_expression(attack_expr)?;
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = attack << 4

        self.generate_expression(decay_expr)?;
        self.emit_imm(opcodes::AND_IMM, 0x0F);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = AD byte

        // Combine sustain and release
        self.generate_expression(sustain_expr)?;
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // TMP1_HI = sustain << 4

        self.generate_expression(release_expr)?;
        self.emit_imm(opcodes::AND_IMM, 0x0F);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // TMP1_HI = SR byte

        // Calculate register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::TAY); // Y = voice * 7

        // Write AD register
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE + sid::OFFSET_ATTACK_DECAY);

        // Write SR register
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_aby(opcodes::STA_ABY, sid::BASE + sid::OFFSET_SUSTAIN_RELEASE);

        Ok(())
    }

    /// Generate code for sid_pulse_width(voice, width).
    fn generate_sid_pulse_width(
        &mut self,
        voice_expr: &Expr,
        width_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Evaluate width (12-bit value, A=low, X=high)
        self.generate_expression(width_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // low byte
        self.emit_byte(opcodes::TXA);
        self.emit_imm(opcodes::AND_IMM, 0x0F); // only low nibble of high byte
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Calculate register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::TAY); // Y = voice * 7

        // Write pulse low
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE + sid::OFFSET_PULSE_LO);

        // Write pulse high
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_aby(opcodes::STA_ABY, sid::BASE + sid::OFFSET_PULSE_HI);

        Ok(())
    }

    /// Generate code for sid_ring_mod, sid_sync, sid_test.
    fn generate_sid_control_bit(
        &mut self,
        voice_expr: &Expr,
        enable_expr: &Expr,
        bit: u8,
    ) -> Result<(), CompileError> {
        // Evaluate enable flag
        self.generate_expression(enable_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Calculate control register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY);

        // Check enable/disable
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        let off_label = self.make_label("scb_off");
        let done_label = self.make_label("scb_done");
        self.emit_branch(opcodes::BEQ, &off_label);

        // Enable: set bit
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::ORA_IMM, bit);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);
        self.emit_jmp(&done_label);

        // Disable: clear bit
        self.define_label(&off_label);
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, !bit);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for sound_off_voice(voice).
    fn generate_sound_off_voice(&mut self, voice_expr: &Expr) -> Result<(), CompileError> {
        // Calculate control register offset
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY);

        // Clear gate bit
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for play_note(voice, note, octave).
    fn generate_play_note(
        &mut self,
        voice_expr: &Expr,
        note_expr: &Expr,
        octave_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Save voice number
        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::PHA);

        // Calculate frequency from note and octave
        // Use lookup table in runtime
        self.generate_expression(octave_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // octave

        self.generate_expression(note_expr)?;
        // A = note (0-11)

        // Call runtime routine to get frequency
        self.emit_jsr_label("__note_to_freq");
        // Returns frequency in A (low) and X (high)

        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Restore voice and set frequency
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::TAY);

        // Store frequency
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        // Set gate on
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY);
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::ORA_IMM, sid::CTRL_GATE);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }

    /// Generate code for play_tone(voice, frequency, waveform, duration).
    fn generate_play_tone(
        &mut self,
        voice_expr: &Expr,
        freq_expr: &Expr,
        waveform_expr: &Expr,
        duration_expr: &Expr,
    ) -> Result<(), CompileError> {
        // Set frequency
        self.generate_sid_frequency(voice_expr, freq_expr)?;

        // Set waveform and gate on
        self.generate_expression(waveform_expr)?;
        self.emit_imm(opcodes::ORA_IMM, sid::CTRL_GATE); // Include gate
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        self.generate_expression(voice_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, sid::OFFSET_CTRL as u8);
        self.emit_byte(opcodes::TAY);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        // Save Y (control offset) for later
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::PHA);

        // Wait for duration (simple busy loop)
        self.generate_expression(duration_expr)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        let outer_label = self.make_label("pt_outer");
        let inner_label = self.make_label("pt_inner");
        let done_label = self.make_label("pt_done");

        self.define_label(&outer_label);
        // Check if duration is zero
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BEQ, &done_label);

        // Inner delay loop
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.define_label(&inner_label);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &inner_label);

        // Decrement duration
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCS, &outer_label);
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jmp(&outer_label);

        self.define_label(&done_label);

        // Restore Y and clear gate
        self.emit_byte(opcodes::PLA);
        self.emit_byte(opcodes::TAY);
        self.emit_aby(opcodes::LDA_ABY, sid::BASE);
        self.emit_imm(opcodes::AND_IMM, !sid::CTRL_GATE);
        self.emit_aby(opcodes::STA_ABY, sid::BASE);

        Ok(())
    }
}
