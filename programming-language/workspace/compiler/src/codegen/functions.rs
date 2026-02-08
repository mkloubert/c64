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
use super::mos6510::{kernal, opcodes, petscii, zeropage};
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
