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
//! - User-defined functions

use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::{cia, kernal, opcodes, petscii, vic, zeropage};
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
                // Get array length as a word (16-bit) value
                // Result: A = low byte, X = high byte
                if !args.is_empty() {
                    let array_size = self.get_array_length(&args[0], span)?;
                    self.emit_imm(opcodes::LDA_IMM, (array_size & 0xFF) as u8);
                    self.emit_imm(opcodes::LDX_IMM, ((array_size >> 8) & 0xFF) as u8);
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
            "sprite_enable" => {
                // sprite_enable(num, enabled) - enable or disable a sprite
                if args.len() >= 2 {
                    self.generate_sprite_enable(args)?;
                }
            }
            "sprite_pos" => {
                // sprite_pos(num, x, y) - set sprite position
                if args.len() >= 3 {
                    self.generate_sprite_pos(args)?;
                }
            }
            "sprite_color" => {
                // sprite_color(num, color) - set sprite color
                if args.len() >= 2 {
                    self.generate_sprite_color(args)?;
                }
            }
            "sprite_data" => {
                // sprite_data(num, pointer) - set sprite data pointer
                if args.len() >= 2 {
                    self.generate_sprite_data(args)?;
                }
            }
            "sprite_expand_x" => {
                // sprite_expand_x(num, enabled) - horizontal expansion
                if args.len() >= 2 {
                    self.generate_sprite_bit_register(args, vic::SPRITE_EXPAND_X)?;
                }
            }
            "sprite_expand_y" => {
                // sprite_expand_y(num, enabled) - vertical expansion
                if args.len() >= 2 {
                    self.generate_sprite_bit_register(args, vic::SPRITE_EXPAND_Y)?;
                }
            }
            "sprite_multicolor" => {
                // sprite_multicolor(num, enabled) - multicolor mode
                if args.len() >= 2 {
                    self.generate_sprite_bit_register(args, vic::SPRITE_MULTICOLOR)?;
                }
            }
            "sprite_priority" => {
                // sprite_priority(num, behind_bg) - priority (behind background)
                if args.len() >= 2 {
                    self.generate_sprite_bit_register(args, vic::SPRITE_PRIORITY)?;
                }
            }
            "sprite_collision" => {
                // sprite_collision() -> byte - read sprite-sprite collision
                self.emit_abs(opcodes::LDA_ABS, vic::SPRITE_COLLISION);
            }
            "sprite_bg_collision" => {
                // sprite_bg_collision() -> byte - read sprite-background collision
                self.emit_abs(opcodes::LDA_ABS, vic::SPRITE_BG_COLLISION);
            }
            "joystick" => {
                // joystick(port) -> byte - read raw joystick state
                if !args.is_empty() {
                    self.generate_joystick_read(args)?;
                }
            }
            "joy_up" => {
                // joy_up(port) -> bool - check if up is pressed
                if !args.is_empty() {
                    self.generate_joystick_direction(args, cia::JOY_UP_MASK)?;
                }
            }
            "joy_down" => {
                // joy_down(port) -> bool - check if down is pressed
                if !args.is_empty() {
                    self.generate_joystick_direction(args, cia::JOY_DOWN_MASK)?;
                }
            }
            "joy_left" => {
                // joy_left(port) -> bool - check if left is pressed
                if !args.is_empty() {
                    self.generate_joystick_direction(args, cia::JOY_LEFT_MASK)?;
                }
            }
            "joy_right" => {
                // joy_right(port) -> bool - check if right is pressed
                if !args.is_empty() {
                    self.generate_joystick_direction(args, cia::JOY_RIGHT_MASK)?;
                }
            }
            "joy_fire" => {
                // joy_fire(port) -> bool - check if fire is pressed
                if !args.is_empty() {
                    self.generate_joystick_direction(args, cia::JOY_FIRE_MASK)?;
                }
            }
            "sid_volume" => {
                // sid_volume(vol) - set master volume (0-15)
                if !args.is_empty() {
                    self.generate_sid_volume(args)?;
                }
            }
            "sid_voice_freq" => {
                // sid_voice_freq(voice, freq) - set voice frequency
                if args.len() >= 2 {
                    self.generate_sid_voice_freq(args)?;
                }
            }
            "sid_voice_pulse" => {
                // sid_voice_pulse(voice, width) - set pulse width
                if args.len() >= 2 {
                    self.generate_sid_voice_pulse(args)?;
                }
            }
            "sid_voice_wave" => {
                // sid_voice_wave(voice, waveform) - set waveform
                if args.len() >= 2 {
                    self.generate_sid_voice_wave(args)?;
                }
            }
            "sid_voice_adsr" => {
                // sid_voice_adsr(voice, attack, decay, sustain, release)
                if args.len() >= 5 {
                    self.generate_sid_voice_adsr(args)?;
                }
            }
            "sid_voice_gate" => {
                // sid_voice_gate(voice, on) - set gate
                if args.len() >= 2 {
                    self.generate_sid_voice_gate(args)?;
                }
            }
            "sid_clear" => {
                // sid_clear() - clear all SID registers
                self.generate_sid_clear()?;
            }
            "border" => {
                // border(color) - set border color
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, vic::BORDER_COLOR);
                }
            }
            "background" => {
                // background(color) - set background color
                if !args.is_empty() {
                    self.generate_expression(&args[0])?;
                    self.emit_abs(opcodes::STA_ABS, vic::BACKGROUND_COLOR);
                }
            }
            "vsync" => {
                // vsync() - wait for vertical blank
                self.generate_vsync()?;
            }
            "raster" => {
                // raster() -> byte - get current raster line
                self.emit_abs(opcodes::LDA_ABS, vic::RASTER);
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
        // Ensure X is 0 for byte-sized arguments (they don't set X)
        let to_type = self.infer_type_from_expr(&args[1]);
        if to_type.is_8bit() {
            self.emit_imm(opcodes::LDX_IMM, 0);
        }
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI); // TMP2 = to (16-bit)

        // Evaluate 'from' (16-bit) and save it
        self.generate_expression(&args[0])?;
        // Ensure X is 0 for byte-sized arguments (they don't set X)
        let from_type = self.infer_type_from_expr(&args[0]);
        if from_type.is_8bit() {
            self.emit_imm(opcodes::LDX_IMM, 0);
        }
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
        // Ensure X is set correctly for byte-sized arguments
        let to_type = self.infer_type_from_expr(&args[1]);
        if to_type.is_8bit() {
            // For signed bytes, sign-extend to 16-bit
            if to_type.is_signed() {
                // Sign extend: if bit 7 is set, X = $FF, else X = $00
                self.emit_imm(opcodes::LDX_IMM, 0);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0x80);
                let skip_label = self.make_label("rsw_skip1");
                self.emit_branch(opcodes::BCC, &skip_label);
                self.emit_byte(opcodes::DEX); // X = $FF
                self.define_label(&skip_label);
            } else {
                self.emit_imm(opcodes::LDX_IMM, 0);
            }
        }
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2_HI);

        // Evaluate 'from' (16-bit) and save it
        self.generate_expression(&args[0])?;
        // Ensure X is set correctly for byte-sized arguments
        let from_type = self.infer_type_from_expr(&args[0]);
        if from_type.is_8bit() {
            // For signed bytes, sign-extend to 16-bit
            if from_type.is_signed() {
                // Sign extend: if bit 7 is set, X = $FF, else X = $00
                self.emit_imm(opcodes::LDX_IMM, 0);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0x80);
                let skip_label = self.make_label("rsw_skip2");
                self.emit_branch(opcodes::BCC, &skip_label);
                self.emit_byte(opcodes::DEX); // X = $FF
                self.define_label(&skip_label);
            } else {
                self.emit_imm(opcodes::LDX_IMM, 0);
            }
        }
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

    /// Generate code for sprite_enable(num, enabled).
    ///
    /// Sets or clears the bit for sprite `num` in the SPRITE_ENABLE register ($D015).
    fn generate_sprite_enable(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        // Evaluate enabled flag and save it
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = enabled flag

        // Evaluate sprite number and create bit mask
        self.generate_expression(&args[0])?;
        // A = sprite number (0-7)
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Create bit mask: 1 << sprite_num
        self.emit_imm(opcodes::LDA_IMM, 1);
        let shift_label = self.make_label("se_shift");
        let shift_done = self.make_label("se_done");
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &shift_done);
        self.define_label(&shift_label);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &shift_label);
        self.define_label(&shift_done);
        // A = bit mask (1 << num)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = bit mask

        // Check if enabling or disabling
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        let disable_label = self.make_label("se_disable");
        let done_label = self.make_label("se_end");
        self.emit_branch(opcodes::BEQ, &disable_label);

        // Enable: OR the bit into the register
        self.emit_abs(opcodes::LDA_ABS, vic::SPRITE_ENABLE);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABS, vic::SPRITE_ENABLE);
        self.emit_jmp(&done_label);

        // Disable: AND with inverted mask
        self.define_label(&disable_label);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF); // Invert mask
        self.emit_abs(opcodes::AND_ABS, vic::SPRITE_ENABLE);
        self.emit_abs(opcodes::STA_ABS, vic::SPRITE_ENABLE);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for sprite_pos(num, x, y).
    ///
    /// Sets sprite position. X is 9-bit (0-320), Y is 8-bit (0-255).
    /// X low 8 bits go to $D000+num*2, X bit 8 goes to $D010 bit num.
    /// Y goes to $D001+num*2.
    fn generate_sprite_pos(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        // Evaluate Y position and save
        self.generate_expression(&args[2])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // TMP3 = Y

        // Evaluate X position (16-bit) and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = X low byte
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = X high byte (bit 8)

        // Evaluate sprite number
        self.generate_expression(&args[0])?;
        // A = sprite number (0-7)
        self.emit_byte(opcodes::ASL_ACC); // A = num * 2 (offset for X/Y pair)
        self.emit_byte(opcodes::TAX); // X = offset

        // Store X low byte at $D000 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, vic::SPRITE0_X);

        // Store Y at $D001 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_abs(opcodes::STA_ABX, vic::SPRITE0_Y);

        // Now handle X MSB (bit 8) in $D010
        // First, get sprite number back (X/2)
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::LSR_ACC); // A = sprite number
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Create bit mask for this sprite
        self.emit_imm(opcodes::LDA_IMM, 1);
        let shift_label = self.make_label("sp_shift");
        let shift_done = self.make_label("sp_sdone");
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &shift_done);
        self.define_label(&shift_label);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &shift_label);
        self.define_label(&shift_done);
        // A = bit mask
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // TMP4 = bit mask

        // Check if X MSB (bit 8) is set
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2); // X high byte
        let msb_clear = self.make_label("sp_mclr");
        let msb_done = self.make_label("sp_mdone");
        self.emit_branch(opcodes::BEQ, &msb_clear);

        // MSB set: OR bit into $D010
        self.emit_abs(opcodes::LDA_ABS, vic::SPRITE_X_MSB);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_abs(opcodes::STA_ABS, vic::SPRITE_X_MSB);
        self.emit_jmp(&msb_done);

        // MSB clear: AND inverted mask into $D010
        self.define_label(&msb_clear);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_imm(opcodes::EOR_IMM, 0xFF); // Invert mask
        self.emit_abs(opcodes::AND_ABS, vic::SPRITE_X_MSB);
        self.emit_abs(opcodes::STA_ABS, vic::SPRITE_X_MSB);

        self.define_label(&msb_done);
        Ok(())
    }

    /// Generate code for sprite_color(num, color).
    ///
    /// Sets sprite color at $D027 + num.
    fn generate_sprite_color(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        // Evaluate color and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = color

        // Evaluate sprite number
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Store color at $D027 + X
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, vic::SPRITE0_COLOR);

        Ok(())
    }

    /// Generate code for sprite_data(num, pointer).
    ///
    /// Sets sprite data pointer at $07F8 + num.
    /// Pointer is a block number (0-255), actual address = pointer * 64.
    fn generate_sprite_data(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        // Evaluate pointer and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = pointer

        // Evaluate sprite number
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Store pointer at $07F8 + X
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, vic::SPRITE_POINTERS);

        Ok(())
    }

    /// Generate code for sprite bit-register functions.
    ///
    /// Used for sprite_expand_x, sprite_expand_y, sprite_multicolor, sprite_priority.
    /// Sets or clears a bit in the given register based on sprite number and enabled flag.
    fn generate_sprite_bit_register(
        &mut self,
        args: &[Expr],
        register: u16,
    ) -> Result<(), CompileError> {
        // Evaluate enabled flag and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // TMP2 = enabled flag

        // Evaluate sprite number and create bit mask
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX); // X = sprite number

        // Create bit mask: 1 << sprite_num
        self.emit_imm(opcodes::LDA_IMM, 1);
        let shift_label = self.make_label("sbr_shift");
        let shift_done = self.make_label("sbr_sdone");
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &shift_done);
        self.define_label(&shift_label);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, &shift_label);
        self.define_label(&shift_done);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // TMP1 = bit mask

        // Check if enabling or disabling
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        let disable_label = self.make_label("sbr_dis");
        let done_label = self.make_label("sbr_done");
        self.emit_branch(opcodes::BEQ, &disable_label);

        // Enable: OR the bit into the register
        self.emit_abs(opcodes::LDA_ABS, register);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABS, register);
        self.emit_jmp(&done_label);

        // Disable: AND with inverted mask
        self.define_label(&disable_label);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_abs(opcodes::AND_ABS, register);
        self.emit_abs(opcodes::STA_ABS, register);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for joystick(port) -> byte.
    ///
    /// Reads raw joystick state from CIA1.
    /// Port 1 = $DC01 (CIA1_PORT_B), Port 2 = $DC00 (CIA1_PORT_A).
    /// Returns active-low bitmask (0 = pressed).
    ///
    /// Note: We configure the DDR to set joystick bits as inputs,
    /// and disable keyboard interference by writing $FF to port A.
    fn generate_joystick_read(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        // Evaluate port number
        self.generate_expression(&args[0])?;
        // A = port number (1 or 2)

        let port1_label = self.make_label("joy_p1");
        let done_label = self.make_label("joy_done");

        // Check if port 1 or 2
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(1);
        self.emit_branch(opcodes::BEQ, &port1_label);

        // Port 2: read from $DC00
        // Set DDR A ($DC02) to $E0 - bits 7-5 output, bits 4-0 input for joystick
        self.emit_imm(opcodes::LDA_IMM, 0xE0);
        self.emit_abs(opcodes::STA_ABS, cia::CIA1_DDR_A);
        // Read joystick from port A
        self.emit_abs(opcodes::LDA_ABS, cia::CIA1_PORT_A);
        self.emit_jmp(&done_label);

        // Port 1: read from $DC01
        self.define_label(&port1_label);
        // Set port A to $FF to disable keyboard column selection
        self.emit_imm(opcodes::LDA_IMM, 0xFF);
        self.emit_abs(opcodes::STA_ABS, cia::CIA1_PORT_A);
        // Set DDR B ($DC03) to $00 - all inputs for joystick
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_abs(opcodes::STA_ABS, cia::CIA1_DDR_B);
        // Read joystick from port B
        self.emit_abs(opcodes::LDA_ABS, cia::CIA1_PORT_B);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for joy_up/down/left/right/fire(port) -> bool.
    ///
    /// Reads joystick and tests specific direction bit.
    /// Returns true if direction is pressed (bit is 0, active-low).
    fn generate_joystick_direction(&mut self, args: &[Expr], mask: u8) -> Result<(), CompileError> {
        // First read the joystick port
        self.generate_joystick_read(args)?;
        // A = raw joystick state (active-low)

        // Test the specific bit
        // If bit is 0, direction is pressed (return true = 1)
        // If bit is 1, direction is not pressed (return false = 0)
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(mask);

        // If result is 0, bit was 0 (pressed), return 1
        // If result is non-zero, bit was 1 (not pressed), return 0
        let pressed_label = self.make_label("jd_pressed");
        let done_label = self.make_label("jd_done");

        self.emit_branch(opcodes::BEQ, &pressed_label);

        // Not pressed: return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);

        // Pressed: return 1
        self.define_label(&pressed_label);
        self.emit_imm(opcodes::LDA_IMM, 1);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for sid_volume(vol).
    ///
    /// Sets the SID master volume (0-15) in bits 0-3 of $D418.
    fn generate_sid_volume(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate volume (0-15)
        self.generate_expression(&args[0])?;
        // Mask to 4 bits
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        // Store to volume register (preserving filter bits would require read-modify-write,
        // but for simplicity we just set volume and clear filter mode)
        self.emit_abs(opcodes::STA_ABS, sid::VOLUME_FILTER);
        Ok(())
    }

    /// Generate code for sid_voice_freq(voice, freq).
    ///
    /// Sets the frequency for a voice (0-2).
    fn generate_sid_voice_freq(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate frequency (16-bit) and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // freq low
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // freq high

        // Evaluate voice number
        self.generate_expression(&args[0])?;
        // A = voice (0, 1, or 2)

        // Calculate register offset: voice * 7
        // Voice 0: $D400, Voice 1: $D407, Voice 2: $D40E
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::ASL_ACC); // * 2
        self.emit_byte(opcodes::ASL_ACC); // * 4
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ASL_ACC); // * 2
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0x01); // +1 to skip the multiply issue, actually voice*7
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::TAX); // X = offset (0, 7, or 14)

        // Store frequency low byte at $D400 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_FREQ_LO);

        // Store frequency high byte at $D401 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INX);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_FREQ_LO);

        Ok(())
    }

    /// Generate code for sid_voice_pulse(voice, width).
    ///
    /// Sets the pulse width for a voice (0-4095, 12-bit).
    fn generate_sid_voice_pulse(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate pulse width (16-bit, only lower 12 bits used) and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // width low
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // width high (bits 0-3 only)

        // Mask high byte to 4 bits
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        // Evaluate voice number and calculate offset
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::TAX); // X = offset

        // Store pulse width low at $D402 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_PW_LO);

        // Store pulse width high at $D403 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INX);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_PW_LO);

        Ok(())
    }

    /// Generate code for sid_voice_wave(voice, waveform).
    ///
    /// Sets the waveform for a voice. Preserves gate bit.
    /// Waveform values: 16=triangle, 32=sawtooth, 64=pulse, 128=noise
    fn generate_sid_voice_wave(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate waveform and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // waveform

        // Evaluate voice number and calculate control register offset
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::TAX); // X = offset

        // Read current control register to preserve gate bit
        self.emit_abs(opcodes::LDA_ABX, sid::VOICE1_CTRL);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01); // Keep only gate bit
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1); // OR with new waveform
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_CTRL);

        Ok(())
    }

    /// Generate code for sid_voice_adsr(voice, attack, decay, sustain, release).
    ///
    /// Sets the ADSR envelope for a voice. Each parameter is 0-15.
    fn generate_sid_voice_adsr(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate release and save
        self.generate_expression(&args[4])?;
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // release

        // Evaluate sustain and combine with release
        self.generate_expression(&args[3])?;
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC); // << 4
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2); // SR byte

        // Evaluate decay and save
        self.generate_expression(&args[2])?;
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4); // decay

        // Evaluate attack and combine with decay
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC); // << 4
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // AD byte

        // Evaluate voice number and calculate register offset
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::TAX); // X = offset

        // Store AD at $D405 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_AD);

        // Store SR at $D406 + offset
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::INX);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_AD);

        Ok(())
    }

    /// Generate code for sid_voice_gate(voice, on).
    ///
    /// Sets or clears the gate bit to start/stop a sound.
    fn generate_sid_voice_gate(&mut self, args: &[Expr]) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Evaluate gate flag and save
        self.generate_expression(&args[1])?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // gate flag

        // Evaluate voice number and calculate control register offset
        self.generate_expression(&args[0])?;
        self.emit_byte(opcodes::TAX);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::ADC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_IMM);
        self.emit_byte(0x01);
        self.emit_byte(opcodes::TAX); // X = offset

        // Check if gate on or off
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        let gate_off = self.make_label("sg_off");
        let done_label = self.make_label("sg_done");
        self.emit_branch(opcodes::BEQ, &gate_off);

        // Gate on: set bit 0
        self.emit_abs(opcodes::LDA_ABX, sid::VOICE1_CTRL);
        self.emit_byte(opcodes::ORA_IMM);
        self.emit_byte(sid::GATE);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_CTRL);
        self.emit_jmp(&done_label);

        // Gate off: clear bit 0
        self.define_label(&gate_off);
        self.emit_abs(opcodes::LDA_ABX, sid::VOICE1_CTRL);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(!sid::GATE);
        self.emit_abs(opcodes::STA_ABX, sid::VOICE1_CTRL);

        self.define_label(&done_label);
        Ok(())
    }

    /// Generate code for sid_clear().
    ///
    /// Clears all SID registers to silence the chip.
    fn generate_sid_clear(&mut self) -> Result<(), CompileError> {
        use super::mos6510::sid;

        // Clear all 25 SID registers ($D400-$D418)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 24);

        let loop_label = self.make_label("sc_loop");
        self.define_label(&loop_label);
        self.emit_abs(opcodes::STA_ABX, sid::BASE);
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BPL, &loop_label);

        Ok(())
    }

    /// Generate code for vsync().
    ///
    /// Waits for vertical blank by monitoring the raster register.
    /// First waits for raster to reach line 250, then waits for it to go back to top.
    /// This ensures we're synchronized with the screen refresh.
    fn generate_vsync(&mut self) -> Result<(), CompileError> {
        // Wait for raster line >= 250 (bottom of visible screen)
        let wait_bottom = self.make_label("vs_bot");
        self.define_label(&wait_bottom);
        self.emit_abs(opcodes::LDA_ABS, vic::RASTER);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(250);
        self.emit_branch(opcodes::BCC, &wait_bottom);

        // Wait for raster line < 250 (top of screen, new frame)
        let wait_top = self.make_label("vs_top");
        self.define_label(&wait_top);
        self.emit_abs(opcodes::LDA_ABS, vic::RASTER);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(250);
        self.emit_branch(opcodes::BCS, &wait_top);

        Ok(())
    }
}
