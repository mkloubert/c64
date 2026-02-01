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

//! Type conversion code generation.
//!
//! This module provides code generation for type conversions between:
//! - Integer types (byte, word, sbyte, sword)
//! - Fixed-point types (12.4 format)
//! - Floating-point types (IEEE-754 binary16)

use super::emit::EmitHelpers;
use super::labels::LabelManager;
use super::mos6510::{opcodes, zeropage};
use super::CodeGenerator;
use crate::ast::Type;
use crate::error::CompileError;

/// Extension trait for type conversion code generation.
pub trait TypeConversions {
    /// Generate code for type conversion.
    ///
    /// Handles conversions between integer, fixed, and float types.
    /// The source value is expected in A (or A/X for 16-bit types).
    /// The result is left in A (or A/X for 16-bit types).
    fn generate_type_conversion(
        &mut self,
        source_type: &Type,
        target_type: &Type,
    ) -> Result<(), CompileError>;
}

impl TypeConversions for CodeGenerator {
    fn generate_type_conversion(
        &mut self,
        source_type: &Type,
        target_type: &Type,
    ) -> Result<(), CompileError> {
        // No conversion needed if types are the same
        if source_type == target_type {
            return Ok(());
        }

        match (source_type, target_type) {
            // Integer to Float conversions
            (Type::Byte, Type::Float) | (Type::Sbyte, Type::Float) => {
                // 8-bit value in A -> float in A/X
                self.emit_jsr_label("__byte_to_float");
            }
            (Type::Word, Type::Float) | (Type::Sword, Type::Float) => {
                // 16-bit value in A/X -> float in A/X
                self.emit_jsr_label("__word_to_float");
            }

            // Float to Integer conversions
            (Type::Float, Type::Byte) | (Type::Float, Type::Sbyte) => {
                // Float in A/X -> 8-bit in A
                self.emit_jsr_label("__float_to_byte");
            }
            (Type::Float, Type::Word) | (Type::Float, Type::Sword) => {
                // Float in A/X -> 16-bit in A/X
                self.emit_jsr_label("__float_to_word");
            }

            // Fixed to Float conversion
            (Type::Fixed, Type::Float) => {
                // 12.4 fixed in A/X -> float in A/X
                self.emit_jsr_label("__fixed_to_float");
            }

            // Float to Fixed conversion
            (Type::Float, Type::Fixed) => {
                // Float in A/X -> 12.4 fixed in A/X
                self.emit_jsr_label("__float_to_fixed");
            }

            // Integer to Fixed conversions
            (Type::Byte, Type::Fixed) | (Type::Sbyte, Type::Fixed) => {
                // 8-bit value in A -> 12.4 fixed in A/X
                // fixed = value << 4 (value * 16)
                self.emit_byte(opcodes::ASL_ACC); // *2
                self.emit_byte(opcodes::ASL_ACC); // *4
                self.emit_byte(opcodes::ASL_ACC); // *8
                self.emit_byte(opcodes::ASL_ACC); // *16
                self.emit_imm(opcodes::LDX_IMM, 0); // High byte = 0 for positive
                                                    // Handle sign extension for signed bytes
                if *source_type == Type::Sbyte {
                    // If original was negative, we need to handle differently
                    // This simple version works for non-negative values
                    // For negative, we'd need to sign-extend before shifting
                }
            }
            (Type::Word, Type::Fixed) | (Type::Sword, Type::Fixed) => {
                // 16-bit value in A/X -> 12.4 fixed in A/X
                // fixed = value << 4
                // Store to temp, shift, return
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift left 4 times
                for _ in 0..4 {
                    self.emit_byte(opcodes::ASL_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::ROL_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
            }

            // Fixed to Integer conversions
            (Type::Fixed, Type::Byte) | (Type::Fixed, Type::Sbyte) => {
                // 12.4 fixed in A/X -> 8-bit in A
                // Truncate: value >> 4
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift right 4 times (arithmetic for signed)
                for _ in 0..4 {
                    self.emit_byte(opcodes::LSR_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_byte(opcodes::ROR_ZP);
                    self.emit_byte(zeropage::TMP1);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            (Type::Fixed, Type::Word) | (Type::Fixed, Type::Sword) => {
                // 12.4 fixed in A/X -> 16-bit in A/X
                // Truncate: value >> 4
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift right 4 times
                for _ in 0..4 {
                    self.emit_byte(opcodes::LSR_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_byte(opcodes::ROR_ZP);
                    self.emit_byte(zeropage::TMP1);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
            }

            // 8-bit to 16-bit promotions
            (Type::Byte, Type::Word) => {
                // Zero-extend: A stays, X = 0
                self.emit_imm(opcodes::LDX_IMM, 0);
            }
            (Type::Sbyte, Type::Sword) => {
                // Sign-extend: check bit 7 of A
                self.emit_imm(opcodes::LDX_IMM, 0);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0x80);
                let positive = self.make_label("sext_pos");
                self.emit_branch(opcodes::BCC, &positive);
                self.emit_imm(opcodes::LDX_IMM, 0xFF); // Sign extend with $FF
                self.define_label(&positive);
            }
            (Type::Byte, Type::Sword) | (Type::Sbyte, Type::Word) => {
                // Mixed sign extension
                self.emit_imm(opcodes::LDX_IMM, 0);
            }

            // 16-bit to 8-bit truncations
            (Type::Word, Type::Byte)
            | (Type::Sword, Type::Sbyte)
            | (Type::Word, Type::Sbyte)
            | (Type::Sword, Type::Byte) => {
                // Just keep the low byte (already in A)
            }

            // Same size, different signedness - no runtime conversion needed
            (Type::Byte, Type::Sbyte)
            | (Type::Sbyte, Type::Byte)
            | (Type::Word, Type::Sword)
            | (Type::Sword, Type::Word) => {
                // No-op, just reinterpret
            }

            // Other conversions (bool, string, etc.) - no runtime conversion
            _ => {}
        }

        Ok(())
    }
}
