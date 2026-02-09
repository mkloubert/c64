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

//! Data block code generation for the Cobra64 compiler.
//!
//! This module handles emitting binary data blocks into the generated code.
//! Data blocks are placed after the code section and their addresses are
//! made available as word constants.

use super::emit::EmitHelpers;
use super::mos6510::opcodes;
use super::CodeGenerator;
use crate::ast::{DataBlock, DataEntry};
use crate::error::CompileError;

/// Information about a pending data block to be emitted.
#[derive(Debug, Clone)]
pub struct PendingDataBlock {
    /// The name of the data block.
    pub name: String,
    /// The data entries to emit.
    pub entries: Vec<DataEntry>,
    /// The alignment requirement (if any).
    pub alignment: Option<u16>,
}

/// A pending reference to a data block address that needs to be resolved.
#[derive(Debug, Clone)]
pub struct PendingDataBlockRef {
    /// Offset in the code where the low byte of the address should be written.
    pub code_offset_lo: usize,
    /// Offset in the code where the high byte of the address should be written.
    pub code_offset_hi: usize,
    /// Name of the data block being referenced.
    pub data_block_name: String,
}

/// Extension trait for data block code generation.
pub trait DataBlockEmitter {
    /// Register a data block for later emission.
    fn register_data_block(&mut self, data_block: &DataBlock);

    /// Emit all pending data blocks and return the number of bytes emitted.
    fn emit_data_blocks(&mut self) -> Result<usize, CompileError>;

    /// Get the address of a data block by name.
    fn get_data_block_address(&self, name: &str) -> Option<u16>;

    /// Check if a name refers to a registered data block.
    fn is_data_block(&self, name: &str) -> bool;

    /// Emit code to load a data block address.
    ///
    /// Sets A=low byte, X=high byte (consistent with other 16-bit values).
    /// The actual address will be patched later when data block positions are known.
    fn emit_data_block_ref(&mut self, name: &str);

    /// Resolve all pending data block references.
    ///
    /// This patches the placeholder addresses in the code with the actual
    /// data block addresses.
    fn resolve_data_block_refs(&mut self);
}

impl DataBlockEmitter for CodeGenerator {
    fn register_data_block(&mut self, data_block: &DataBlock) {
        let pending = PendingDataBlock {
            name: data_block.name.clone(),
            entries: data_block.entries.clone(),
            alignment: data_block.alignment,
        };
        self.pending_data_blocks.push(pending);
    }

    fn emit_data_blocks(&mut self) -> Result<usize, CompileError> {
        let mut total_bytes = 0;

        // Clone pending data blocks to avoid borrow issues
        let data_blocks: Vec<PendingDataBlock> = self.pending_data_blocks.clone();

        // Pre-load all include files to collect bytes
        // This avoids borrow issues with self
        let mut resolved_entries: Vec<Vec<Vec<u8>>> = Vec::new();

        for data_block in &data_blocks {
            let mut block_bytes: Vec<Vec<u8>> = Vec::new();
            for entry in &data_block.entries {
                match entry {
                    DataEntry::Bytes(bytes) => {
                        block_bytes.push(bytes.clone());
                    }
                    DataEntry::Include {
                        path,
                        offset,
                        length,
                        span,
                    } => {
                        // Read the file using the include resolver
                        let file_bytes = self
                            .include_resolver
                            .read_file_range(path, *offset, *length, *span)?;
                        block_bytes.push(file_bytes);
                    }
                }
            }
            resolved_entries.push(block_bytes);
        }

        // Now emit all the data
        for (i, data_block) in data_blocks.iter().enumerate() {
            // Apply alignment if specified
            if let Some(alignment) = data_block.alignment {
                let current = self.current_address;
                let remainder = current % alignment;
                if remainder != 0 {
                    let padding = alignment - remainder;
                    for _ in 0..padding {
                        self.emit_byte(0x00);
                        total_bytes += 1;
                    }
                }
            }

            // Record the address for this data block
            let block_address = self.current_address;
            self.data_block_addresses
                .insert(data_block.name.clone(), block_address);

            // Emit all resolved bytes for this block
            for bytes in &resolved_entries[i] {
                for byte in bytes {
                    self.emit_byte(*byte);
                    total_bytes += 1;
                }
            }
        }

        Ok(total_bytes)
    }

    fn get_data_block_address(&self, name: &str) -> Option<u16> {
        self.data_block_addresses.get(name).copied()
    }

    fn is_data_block(&self, name: &str) -> bool {
        self.pending_data_blocks.iter().any(|db| db.name == name)
    }

    fn emit_data_block_ref(&mut self, name: &str) {
        // Emit LDA #<placeholder_lo>
        self.emit_byte(opcodes::LDA_IMM);
        let code_offset_lo = self.code.len();
        self.emit_byte(0x00); // Placeholder for low byte

        // Emit LDX #<placeholder_hi>
        self.emit_byte(opcodes::LDX_IMM);
        let code_offset_hi = self.code.len();
        self.emit_byte(0x00); // Placeholder for high byte

        // Record pending reference
        self.pending_data_block_refs.push(PendingDataBlockRef {
            code_offset_lo,
            code_offset_hi,
            data_block_name: name.to_string(),
        });
    }

    fn resolve_data_block_refs(&mut self) {
        // Clone refs to avoid borrow issues
        let refs: Vec<PendingDataBlockRef> = self.pending_data_block_refs.clone();

        for pending in &refs {
            if let Some(addr) = self.data_block_addresses.get(&pending.data_block_name) {
                self.code[pending.code_offset_lo] = (*addr & 0xFF) as u8;
                self.code[pending.code_offset_hi] = (*addr >> 8) as u8;
            }
        }
    }
}
