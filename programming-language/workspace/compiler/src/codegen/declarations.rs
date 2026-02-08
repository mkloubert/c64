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

//! Declaration code generation.
//!
//! This module provides code generation for declarations:
//! - Variable declarations (with optional initializers)
//! - Constant declarations
//! - Function definitions

use super::emit::EmitHelpers;
use super::expressions::ExpressionEmitter;
use super::labels::LabelManager;
use super::mos6510::opcodes;
use super::type_inference::TypeInference;
use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{Block, ConstDecl, Expr, ExprKind, FunctionDef, Type, VarDecl};
use crate::error::CompileError;

/// Extension trait for declaration code generation.
pub trait DeclarationEmitter {
    /// Generate code for a function.
    fn generate_function(&mut self, func: &FunctionDef) -> Result<(), CompileError>;

    /// Generate code for a variable declaration.
    fn generate_var_decl(&mut self, decl: &VarDecl) -> Result<(), CompileError>;

    /// Generate code for a constant declaration.
    fn generate_const_decl(&mut self, decl: &ConstDecl) -> Result<(), CompileError>;
}

impl DeclarationEmitter for CodeGenerator {
    fn generate_function(&mut self, func: &FunctionDef) -> Result<(), CompileError> {
        // Define label for function
        self.define_label(&func.name);

        // Update function address
        if let Some(f) = self.functions.get_mut(&func.name) {
            f.address = self.current_address;
        }

        // For main function, call global initialization (PRNG init, global vars)
        if func.name == "main" {
            self.emit_jsr_label("__init_globals");
        }

        // Note: Parameters are already allocated in the first pass.
        // The variables table already contains entries for parameter names.

        // Generate function body
        self.generate_block(&func.body)?;

        // Emit RTS at end (if not already returned)
        self.emit_byte(opcodes::RTS);

        Ok(())
    }

    fn generate_var_decl(&mut self, decl: &VarDecl) -> Result<(), CompileError> {
        // Explicit type is required (parser enforces this)
        let mut var_type = decl
            .var_type
            .clone()
            .expect("Variable declaration must have explicit type");

        // If the type is an array without a size but we have an array literal,
        // update the type to include the size from the literal
        if let Some(init) = &decl.initializer {
            if let ExprKind::ArrayLiteral { elements } = &init.kind {
                let len = elements.len() as u16;
                var_type = match var_type {
                    Type::ByteArray(None) => Type::ByteArray(Some(len)),
                    Type::WordArray(None) => Type::WordArray(Some(len)),
                    Type::BoolArray(None) => Type::BoolArray(Some(len)),
                    Type::SbyteArray(None) => Type::SbyteArray(Some(len)),
                    Type::SwordArray(None) => Type::SwordArray(Some(len)),
                    _ => var_type,
                };
            }
        }

        let address = self.allocate_variable(&decl.name, &var_type, false);

        if let Some(init) = &decl.initializer {
            // Check if initializer is an array literal
            if let ExprKind::ArrayLiteral { elements } = &init.kind {
                generate_array_literal_init(self, address, &var_type, elements)?;
            } else {
                // Use type-aware expression generation for proper literal conversion
                self.generate_expression_for_type(init, &var_type)?;
                self.emit_store_to_address(address, &var_type);
            }
        }

        Ok(())
    }

    fn generate_const_decl(&mut self, decl: &ConstDecl) -> Result<(), CompileError> {
        // Explicit type is required (parser enforces this)
        let const_type = decl
            .const_type
            .clone()
            .expect("Constant declaration must have explicit type");

        let address = self.allocate_variable(&decl.name, &const_type, true);

        // Use type-aware expression generation for proper literal conversion
        self.generate_expression_for_type(&decl.value, &const_type)?;
        self.emit_store_to_address(address, &const_type);

        Ok(())
    }
}

/// Helper trait for block generation used by declarations.
pub trait BlockGenerator {
    /// Generate code for a block.
    fn generate_block(&mut self, block: &Block) -> Result<(), CompileError>;
}

/// Generate code to initialize an array from a literal.
fn generate_array_literal_init(
    gen: &mut CodeGenerator,
    base_address: u16,
    array_type: &Type,
    elements: &[Expr],
) -> Result<(), CompileError> {
    let element_type = array_type.element_type().unwrap_or(Type::Byte);
    let element_size = match element_type {
        Type::Word | Type::Sword => 2,
        _ => 1, // Byte, Sbyte, Bool
    };

    // Optimization: check if all elements are zero
    if gen.all_elements_are_zero(elements) {
        let total_bytes = elements.len() * element_size;
        generate_zero_memory(gen, base_address, total_bytes)?;
        return Ok(());
    }

    for (i, elem) in elements.iter().enumerate() {
        let elem_address = base_address.wrapping_add((i * element_size) as u16);

        // Generate the element value
        gen.generate_expression(elem)?;

        // Store the value at the element address
        match element_type {
            Type::Word | Type::Sword => {
                // Store 16-bit value (A=low, X=high)
                gen.emit_abs(opcodes::STA_ABS, elem_address);
                gen.emit_byte(opcodes::STX_ABS);
                gen.emit_word(elem_address.wrapping_add(1));
            }
            _ => {
                // Store 8-bit value (Byte, Sbyte, Bool)
                gen.emit_abs(opcodes::STA_ABS, elem_address);
            }
        }
    }

    Ok(())
}

/// Generate efficient code to zero a block of memory.
fn generate_zero_memory(
    gen: &mut CodeGenerator,
    base_address: u16,
    byte_count: usize,
) -> Result<(), CompileError> {
    if byte_count == 0 {
        return Ok(());
    }

    // For small arrays (< 8 bytes), use individual stores
    if byte_count < 8 {
        gen.emit_imm(opcodes::LDA_IMM, 0);
        for i in 0..byte_count {
            gen.emit_abs(opcodes::STA_ABS, base_address.wrapping_add(i as u16));
        }
        return Ok(());
    }

    // For larger arrays, use a loop with X register as counter
    // Limit to 256 bytes per loop iteration
    let chunks = byte_count.div_ceil(256);
    let mut remaining = byte_count;

    for chunk in 0..chunks {
        let chunk_base = base_address.wrapping_add((chunk * 256) as u16);
        let chunk_size = remaining.min(256);

        // LDA #$00
        gen.emit_imm(opcodes::LDA_IMM, 0);

        if chunk_size == 256 {
            // LDX #$00 (will wrap from 0 to 255, then to 0)
            gen.emit_imm(opcodes::LDX_IMM, 0);
        } else {
            // LDX #(chunk_size - 1)
            gen.emit_imm(opcodes::LDX_IMM, (chunk_size - 1) as u8);
        }

        // loop: STA base,X
        let loop_addr = gen.current_address();
        gen.emit_abs(opcodes::STA_ABX, chunk_base);

        // DEX
        gen.emit_byte(opcodes::DEX);

        if chunk_size == 256 {
            // BNE loop (branch if X != 0 after wrapping)
            gen.emit_byte(opcodes::BNE);
            let offset = loop_addr.wrapping_sub(gen.current_address().wrapping_add(1)) as i8;
            gen.emit_byte(offset as u8);
        } else {
            // BPL loop (branch while X >= 0)
            gen.emit_byte(opcodes::BPL);
            let offset = loop_addr.wrapping_sub(gen.current_address().wrapping_add(1)) as i8;
            gen.emit_byte(offset as u8);
        }

        remaining -= chunk_size;
    }

    Ok(())
}
