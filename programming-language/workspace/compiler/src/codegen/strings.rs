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

//! String literal management for code generation.
//!
//! This module handles string storage and address resolution.
//! It provides:
//! - PendingStringRef structure for tracking unresolved string addresses
//! - StringManager trait for string allocation and reference emission

use super::emit::EmitHelpers;
use super::mos6510::{opcodes, zeropage};
use super::CodeGenerator;

/// A pending string reference that needs its address resolved.
#[derive(Debug, Clone)]
pub struct PendingStringRef {
    /// Offset in the code where the low byte of the address should be written.
    pub code_offset_lo: usize,
    /// Offset in the code where the high byte of the address should be written.
    pub code_offset_hi: usize,
    /// Index of the string in the strings vector.
    pub string_index: usize,
}

/// Extension trait for string literal management.
///
/// This trait provides methods for adding string literals, emitting
/// string references, and resolving string addresses during code generation.
pub trait StringManager {
    /// Add a string literal and return its index.
    ///
    /// The string is converted to bytes and null-terminated.
    fn add_string(&mut self, value: &str) -> usize;

    /// Emit code to load a string address.
    ///
    /// Sets A=low byte, X=high byte (consistent with other 16-bit values).
    /// Also sets TMP1/TMP1_HI for print_str compatibility.
    /// The actual address will be patched later when string positions are known.
    fn emit_string_ref(&mut self, string_index: usize);

    /// Resolve all pending string references.
    ///
    /// This patches the placeholder addresses in the code with the actual
    /// string addresses based on where strings will be placed in memory.
    fn resolve_string_refs(&mut self);
}

impl StringManager for CodeGenerator {
    fn add_string(&mut self, value: &str) -> usize {
        // Convert to PETSCII-like bytes (simplified: just use ASCII values)
        let mut bytes: Vec<u8> = value.bytes().collect();
        bytes.push(0); // Null terminator

        let index = self.strings.len();
        self.strings.push(bytes);
        index
    }

    fn emit_string_ref(&mut self, string_index: usize) {
        // LDA #<string_addr (placeholder)
        self.emit_byte(opcodes::LDA_IMM);
        let code_offset_lo = self.code.len();
        self.emit_byte(0); // Placeholder for low byte

        // STA TMP1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // LDX #>string_addr (placeholder)
        self.emit_byte(opcodes::LDX_IMM);
        let code_offset_hi = self.code.len();
        self.emit_byte(0); // Placeholder for high byte

        // STX TMP1_HI
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // A=low, X=high is now set (consistent with Word and String variables)

        // Record pending reference
        self.pending_string_refs.push(PendingStringRef {
            code_offset_lo,
            code_offset_hi,
            string_index,
        });
    }

    fn resolve_string_refs(&mut self) {
        // Calculate where each string will be placed
        let mut string_addresses = Vec::new();
        let mut addr = self.current_address;
        for s in &self.strings {
            string_addresses.push(addr);
            addr = addr.wrapping_add(s.len() as u16);
        }

        // Patch all pending references
        for pending in &self.pending_string_refs {
            if pending.string_index < string_addresses.len() {
                let addr = string_addresses[pending.string_index];
                self.code[pending.code_offset_lo] = (addr & 0xFF) as u8;
                self.code[pending.code_offset_hi] = (addr >> 8) as u8;
            }
        }
    }
}
