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

//! Symbol and symbol type definitions for the semantic analyzer.
//!
//! This module defines the core symbol table entry structure used
//! during semantic analysis to track variables, constants, and functions.

use crate::ast::Type;
use crate::error::Span;

/// Symbol table entry.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The symbol name.
    pub name: String,
    /// The symbol type.
    pub symbol_type: SymbolType,
    /// Whether this is a constant.
    pub is_constant: bool,
    /// The memory address (assigned during code generation).
    pub address: Option<u16>,
    /// The span where this symbol was defined.
    pub span: Span,
}

impl Symbol {
    /// Create a new variable symbol.
    pub fn variable(name: String, var_type: Type, is_constant: bool, span: Span) -> Self {
        Self {
            name,
            symbol_type: SymbolType::Variable(var_type),
            is_constant,
            address: None,
            span,
        }
    }

    /// Create a new function symbol.
    pub fn function(
        name: String,
        params: Vec<Type>,
        return_type: Option<Type>,
        span: Span,
    ) -> Self {
        Self {
            name,
            symbol_type: SymbolType::Function {
                params,
                return_type,
            },
            is_constant: true, // Functions are always immutable
            address: None,
            span,
        }
    }

    /// Create a new data block symbol.
    /// Data blocks are treated as word constants (their address).
    pub fn data_block(name: String, size: usize, span: Span) -> Self {
        Self {
            name,
            symbol_type: SymbolType::DataBlock { size },
            is_constant: true, // Data blocks are always immutable
            address: None,
            span,
        }
    }

    /// Get the type of a variable symbol.
    /// For data blocks, returns Word (the address type).
    pub fn get_type(&self) -> Option<&Type> {
        match &self.symbol_type {
            SymbolType::Variable(t) => Some(t),
            SymbolType::Function { .. } => None,
            SymbolType::DataBlock { .. } => None, // Address is handled specially
        }
    }

    /// Check if this is a data block symbol.
    pub fn is_data_block(&self) -> bool {
        matches!(self.symbol_type, SymbolType::DataBlock { .. })
    }

    /// Get the size of a data block symbol.
    pub fn data_block_size(&self) -> Option<usize> {
        match &self.symbol_type {
            SymbolType::DataBlock { size } => Some(*size),
            _ => None,
        }
    }
}

/// The type of a symbol.
#[derive(Debug, Clone)]
pub enum SymbolType {
    /// A variable or constant.
    Variable(Type),
    /// A function.
    Function {
        params: Vec<Type>,
        return_type: Option<Type>,
    },
    /// A data block (address is a word constant).
    DataBlock {
        /// The size of the data block in bytes.
        size: usize,
    },
}
