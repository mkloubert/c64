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
use super::mos6510::opcodes;
use super::CodeGenerator;
use crate::error::{CompileError, ErrorCode, Span};
use std::collections::HashMap;

/// A pending branch that needs its target resolved.
#[derive(Debug, Clone)]
pub struct PendingBranch {
    /// Offset in the code where the branch opcode is located.
    pub opcode_offset: usize,
    /// Offset in the code where the branch displacement should be written.
    pub code_offset: usize,
    /// Label this branch should jump to.
    pub target_label: String,
    /// Whether this branch was converted to a trampoline (skip during resolution).
    pub trampolined: bool,
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
        // Iterate until all branches are within range
        // (inserting trampolines can push other branches out of range)
        const MAX_ITERATIONS: usize = 100;

        for iteration in 0..MAX_ITERATIONS {
            // Find branches that need trampolines (not already trampolined)
            let mut trampoline_indices: Vec<usize> = Vec::new();

            for (idx, branch) in self.pending_branches.iter().enumerate() {
                if branch.trampolined {
                    continue;
                }

                let target = self.labels.get(&branch.target_label).ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined label '{}'", branch.target_label),
                        Span::new(0, 0),
                    )
                })?;

                let branch_addr = PROGRAM_START as i32 + branch.code_offset as i32 + 1;
                let target_addr = *target as i32;
                let displacement = target_addr - branch_addr;

                if !(-128..=127).contains(&displacement) {
                    trampoline_indices.push(idx);
                }
            }

            // If no more trampolines needed, we're done
            if trampoline_indices.is_empty() {
                return self.resolve_labels_simple();
            }

            // Insert trampolines for branches that are too far
            self.insert_trampolines(&trampoline_indices)?;

            // Safety check to prevent infinite loops
            if iteration == MAX_ITERATIONS - 1 {
                return Err(CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    "Too many trampolining iterations - code is too complex".to_string(),
                    Span::new(0, 0),
                ));
            }
        }

        self.resolve_labels_simple()
    }
}

impl CodeGenerator {
    /// Insert trampolines for the specified branches.
    fn insert_trampolines(&mut self, trampoline_indices: &[usize]) -> Result<(), CompileError> {
        // Collect trampoline info (offset, target_label, branch_idx)
        let mut trampolines: Vec<(usize, String, usize)> = Vec::new();
        for &idx in trampoline_indices {
            let branch = &self.pending_branches[idx];
            trampolines.push((branch.code_offset + 1, branch.target_label.clone(), idx));
        }

        // Sort trampolines by offset (descending) so we can insert from end to start
        trampolines.sort_by(|a, b| b.0.cmp(&a.0));

        // Track how many bytes were inserted at each position
        let mut insertions: Vec<(usize, usize)> = Vec::new();

        for (insert_offset, ref target_label, branch_idx) in &trampolines {
            // Insert JMP instruction (3 bytes: opcode + 2-byte address)
            let jmp_bytes = vec![opcodes::JMP_ABS, 0x00, 0x00];

            // Adjust insert_offset based on previous insertions in this batch
            let mut adjusted_insert_offset = *insert_offset;
            for (prev_offset, bytes_inserted) in &insertions {
                if *insert_offset > *prev_offset {
                    adjusted_insert_offset += bytes_inserted;
                }
            }

            // Insert the bytes
            self.code
                .splice(adjusted_insert_offset..adjusted_insert_offset, jmp_bytes);
            insertions.push((*insert_offset, 3));

            // The branch opcode is one byte before the offset byte
            let opcode_code_offset = adjusted_insert_offset - 1;
            let original_opcode = self.code[opcode_code_offset];
            self.code[opcode_code_offset] = invert_branch_opcode(original_opcode);
            self.code[adjusted_insert_offset] = 3; // Skip the 3-byte JMP

            // Mark this branch as trampolined
            self.pending_branches[*branch_idx].trampolined = true;

            // Add a pending jump for the JMP we just inserted
            self.pending_jumps.push(PendingJump {
                code_offset: adjusted_insert_offset + 1,
                target_label: target_label.clone(),
            });
        }

        // Adjust all label addresses
        let mut adjusted_labels: HashMap<String, u16> = HashMap::new();
        for (label, addr) in &self.labels {
            let mut new_addr = *addr;
            for (insert_offset, bytes_inserted) in &insertions {
                let label_code_offset = (*addr as usize).saturating_sub(PROGRAM_START as usize);
                if label_code_offset > *insert_offset {
                    new_addr = new_addr.wrapping_add(*bytes_inserted as u16);
                }
            }
            adjusted_labels.insert(label.clone(), new_addr);
        }
        self.labels = adjusted_labels;

        // Adjust all pending branch code offsets (including trampolined ones)
        for branch in &mut self.pending_branches {
            for (insert_offset, bytes_inserted) in &insertions {
                if branch.code_offset > *insert_offset {
                    branch.opcode_offset += bytes_inserted;
                    branch.code_offset += bytes_inserted;
                }
            }
        }

        // Adjust pending jump code offsets (except the newly added ones)
        let original_jump_count = self.pending_jumps.len() - trampoline_indices.len();
        for jump in self.pending_jumps.iter_mut().take(original_jump_count) {
            for (insert_offset, bytes_inserted) in &insertions {
                if jump.code_offset > *insert_offset {
                    jump.code_offset += bytes_inserted;
                }
            }
        }

        // Update current_address
        let total_inserted: usize = insertions.iter().map(|(_, b)| b).sum();
        self.current_address = self.current_address.wrapping_add(total_inserted as u16);

        Ok(())
    }

    fn resolve_labels_simple(&mut self) -> Result<(), CompileError> {
        // Resolve branches (8-bit relative), skipping trampolined ones
        for branch in &self.pending_branches {
            // Skip branches that were converted to trampolines
            if branch.trampolined {
                continue;
            }

            let target = self.labels.get(&branch.target_label).ok_or_else(|| {
                CompileError::new(
                    ErrorCode::UndefinedVariable,
                    format!("Undefined label '{}'", branch.target_label),
                    Span::new(0, 0),
                )
            })?;

            let branch_addr = PROGRAM_START as i32 + branch.code_offset as i32 + 1;
            let target_addr = *target as i32;
            let displacement = target_addr - branch_addr;

            // After trampolining, all branches should fit
            if !(-128..=127).contains(&displacement) {
                return Err(CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    format!(
                        "Branch target still too far after trampolining: {} bytes",
                        displacement
                    ),
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

/// Invert a branch opcode for trampolining.
///
/// When a branch target is too far, we invert the condition and use a JMP.
/// For example: `BEQ far_target` becomes `BNE +3; JMP far_target`
fn invert_branch_opcode(opcode: u8) -> u8 {
    match opcode {
        opcodes::BEQ => opcodes::BNE,
        opcodes::BNE => opcodes::BEQ,
        opcodes::BCC => opcodes::BCS,
        opcodes::BCS => opcodes::BCC,
        opcodes::BMI => opcodes::BPL,
        opcodes::BPL => opcodes::BMI,
        opcodes::BVC => opcodes::BVS,
        opcodes::BVS => opcodes::BVC,
        _ => opcode, // Unknown opcode, return as-is
    }
}
