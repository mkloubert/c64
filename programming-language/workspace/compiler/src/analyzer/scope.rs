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

//! Scope management for the semantic analyzer.
//!
//! A scope represents a lexical region where symbols are defined.
//! Scopes are nested to support block-level variable declarations.

use super::symbol::Symbol;
use std::collections::HashMap;

/// A scope in the symbol table.
#[derive(Debug, Default)]
pub struct Scope {
    /// Symbols defined in this scope.
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a symbol in this scope.
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Symbol> {
        if let Some(existing) = self.symbols.get(&symbol.name) {
            return Err(existing.clone());
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol in this scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Look up a symbol in this scope (mutable).
    #[allow(dead_code)]
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }
}
