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

//! Emit helper methods for code generation.
//!
//! This module provides low-level byte emission utilities for generating
//! 6510 machine code. It includes:
//! - Basic byte and word emission
//! - Instruction encoding (immediate, absolute modes)
//! - Branch and jump emission with label support
//! - BASIC stub generation for C64 autostart

use super::constants::CODE_START;
use super::labels::{PendingBranch, PendingJump};
use super::mos6510::opcodes;
use super::CodeGenerator;
use crate::ast::Type;

/// Extension trait for low-level code emission.
///
/// This trait provides methods for emitting bytes, words, and instructions
/// to the generated code buffer. It is implemented for `CodeGenerator` and
/// separates emission logic from the main code generator.
pub trait EmitHelpers {
    /// Emit a single byte to the code buffer.
    fn emit_byte(&mut self, byte: u8);

    /// Emit a 16-bit word in little-endian format.
    fn emit_word(&mut self, word: u16);

    /// Emit an instruction with immediate operand.
    fn emit_imm(&mut self, opcode: u8, value: u8);

    /// Emit an instruction with absolute operand.
    fn emit_abs(&mut self, opcode: u8, address: u16);

    /// Emit a branch instruction with a label target.
    fn emit_branch(&mut self, opcode: u8, label: &str);

    /// Emit a JMP instruction with a label target.
    fn emit_jmp(&mut self, label: &str);

    /// Emit a JSR instruction with a label target.
    fn emit_jsr_label(&mut self, label: &str);

    /// Emit the BASIC stub for C64 autostart.
    fn emit_basic_stub(&mut self);

    /// Emit code to load a value from an address into A (and X for 16-bit types).
    fn emit_load_from_address(&mut self, address: u16, var_type: &Type);

    /// Emit code to store A (and X for 16-bit types) to an address.
    fn emit_store_to_address(&mut self, address: u16, var_type: &Type);
}

impl EmitHelpers for CodeGenerator {
    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
        self.current_address = self.current_address.wrapping_add(1);
    }

    fn emit_word(&mut self, word: u16) {
        self.emit_byte((word & 0xFF) as u8);
        self.emit_byte((word >> 8) as u8);
    }

    fn emit_imm(&mut self, opcode: u8, value: u8) {
        self.emit_byte(opcode);
        self.emit_byte(value);
    }

    fn emit_abs(&mut self, opcode: u8, address: u16) {
        self.emit_byte(opcode);
        self.emit_word(address);
    }

    fn emit_branch(&mut self, opcode: u8, label: &str) {
        self.emit_byte(opcode);
        let offset = self.code.len();
        self.emit_byte(0x00); // Placeholder
        self.pending_branches.push(PendingBranch {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    fn emit_jmp(&mut self, label: &str) {
        self.emit_byte(opcodes::JMP_ABS);
        let offset = self.code.len();
        self.emit_word(0x0000); // Placeholder
        self.pending_jumps.push(PendingJump {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    fn emit_jsr_label(&mut self, label: &str) {
        self.emit_byte(opcodes::JSR);
        let offset = self.code.len();
        self.emit_word(0x0000); // Placeholder
        self.pending_jumps.push(PendingJump {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    fn emit_basic_stub(&mut self) {
        // BASIC stub: 10 SYS 2062
        // The machine code starts at $080E (2062 decimal)

        // Link to next BASIC line (points to end of program marker)
        self.emit_word(0x080C);

        // Line number (10)
        self.emit_word(0x000A);

        // SYS token
        self.emit_byte(0x9E);

        // Space
        self.emit_byte(0x20);

        // Address as ASCII: "2062"
        self.emit_byte(b'2');
        self.emit_byte(b'0');
        self.emit_byte(b'6');
        self.emit_byte(b'2');

        // End of BASIC line
        self.emit_byte(0x00);

        // End of BASIC program (null link pointer)
        self.emit_byte(0x00);
        self.emit_byte(0x00);

        // Machine code starts here at $080E (2062)
        assert_eq!(self.current_address, CODE_START);
    }

    fn emit_load_from_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::LDA_ABS, address);
            }
            Type::Word | Type::Sword | Type::String => {
                // String is a 16-bit pointer, same as Word
                self.emit_abs(opcodes::LDA_ABS, address);
                self.emit_abs(opcodes::LDX_ABS, address.wrapping_add(1));
            }
            _ => {
                // For other types, just load the address
                self.emit_abs(opcodes::LDA_ABS, address);
            }
        }
    }

    fn emit_store_to_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::STA_ABS, address);
            }
            Type::Word | Type::Sword | Type::String => {
                // String is a 16-bit pointer, same as Word
                self.emit_abs(opcodes::STA_ABS, address);
                self.emit_byte(opcodes::STX_ABS);
                self.emit_word(address.wrapping_add(1));
            }
            _ => {
                self.emit_abs(opcodes::STA_ABS, address);
            }
        }
    }
}
