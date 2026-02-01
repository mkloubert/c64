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
//! - Built-in functions (cls, print, println, cursor, get_key, wait_for_key, readln, poke, peek, len)
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
    /// Handles built-in functions (cls, print, println, cursor, get_key, wait_for_key,
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
            "wait_for_key" => {
                let wait_label = self.make_label("wait_key");
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
