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

//! Label and branch management for code generation.
//!
//! This module handles:
//! - Pending branches (8-bit relative jumps)
//! - Pending jumps (16-bit absolute jumps)
//! - Loop context for break/continue
//! - LabelManager trait for label creation and resolution

use super::constants::PROGRAM_START;
use super::CodeGenerator;
use crate::error::{CompileError, ErrorCode, Span};

/// A pending branch that needs its target resolved.
#[derive(Debug, Clone)]
pub struct PendingBranch {
    /// Offset in the code where the branch displacement should be written.
    pub code_offset: usize,
    /// Label this branch should jump to.
    pub target_label: String,
}

/// A pending jump (16-bit) that needs its target resolved.
#[derive(Debug, Clone)]
pub struct PendingJump {
    /// Offset in the code where the jump address should be written.
    pub code_offset: usize,
    /// Label this jump should jump to.
    pub target_label: String,
}

/// Loop context for break/continue handling.
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Label at the start of the loop (for continue).
    pub start_label: String,
    /// Label at the end of the loop (for break).
    pub end_label: String,
}

/// Extension trait for label management.
///
/// This trait provides methods for creating labels, defining label addresses,
/// and resolving pending branches and jumps during code generation.
pub trait LabelManager {
    /// Generate a unique label with the given prefix.
    fn make_label(&mut self, prefix: &str) -> String;

    /// Define a label at the current code address.
    fn define_label(&mut self, name: &str);

    /// Resolve all pending branches and jumps.
    ///
    /// This patches the placeholder offsets in the code with the actual
    /// target addresses based on where labels were defined.
    fn resolve_labels(&mut self) -> Result<(), CompileError>;
}

impl LabelManager for CodeGenerator {
    fn make_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn define_label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.current_address);
    }

    fn resolve_labels(&mut self) -> Result<(), CompileError> {
        // Resolve branches (8-bit relative)
        for branch in &self.pending_branches {
            let target = self.labels.get(&branch.target_label).ok_or_else(|| {
                CompileError::new(
                    ErrorCode::UndefinedVariable, // Reusing error code
                    format!("Undefined label '{}'", branch.target_label),
                    Span::new(0, 0),
                )
            })?;

            // Calculate relative offset
            // Branch is from the byte AFTER the displacement
            let branch_addr = PROGRAM_START as i32 + branch.code_offset as i32 + 1;
            let target_addr = *target as i32;
            let displacement = target_addr - branch_addr;

            if !(-128..=127).contains(&displacement) {
                return Err(CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    format!("Branch target too far: {} bytes", displacement),
                    Span::new(0, 0),
                ));
            }

            self.code[branch.code_offset] = displacement as i8 as u8;
        }

        // Resolve jumps (16-bit absolute)
        for jump in &self.pending_jumps {
            let target = self.labels.get(&jump.target_label).ok_or_else(|| {
                CompileError::new(
                    ErrorCode::UndefinedVariable,
                    format!("Undefined label '{}'", jump.target_label),
                    Span::new(0, 0),
                )
            })?;

            self.code[jump.code_offset] = (*target & 0xFF) as u8;
            self.code[jump.code_offset + 1] = (*target >> 8) as u8;
        }

        Ok(())
    }
}
