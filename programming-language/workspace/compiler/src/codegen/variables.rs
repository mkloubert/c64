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

//! Variable management for code generation.
//!
//! This module handles variable allocation and storage information.
//! It provides:
//! - Variable and Function data structures
//! - VariableManager trait for allocation and lookup

use super::CodeGenerator;
use crate::ast::Type;

/// Variable information for code generation.
#[derive(Debug, Clone)]
pub struct Variable {
    /// Address where the variable is stored.
    pub address: u16,
    /// Type of the variable.
    pub var_type: Type,
    /// Whether this is a constant (reserved for future use).
    #[allow(dead_code)]
    pub is_const: bool,
}

/// Function information for code generation.
#[derive(Debug, Clone)]
pub struct Function {
    /// Address where the function code starts.
    pub address: u16,
    /// Parameter types.
    pub params: Vec<Type>,
    /// Parameter addresses (memory locations for each parameter).
    pub param_addresses: Vec<u16>,
    /// Return type (reserved for future use).
    #[allow(dead_code)]
    pub return_type: Option<Type>,
}

/// Extension trait for variable management.
///
/// This trait provides methods for allocating and looking up variables
/// during code generation. It is implemented for `CodeGenerator`.
pub trait VariableManager {
    /// Allocate a variable and return its address.
    ///
    /// # Arguments
    /// * `name` - The variable name
    /// * `var_type` - The type of the variable
    /// * `is_const` - Whether this is a constant
    ///
    /// # Returns
    /// The memory address where the variable is allocated.
    fn allocate_variable(&mut self, name: &str, var_type: &Type, is_const: bool) -> u16;

    /// Look up a variable by name.
    ///
    /// # Arguments
    /// * `name` - The variable name to look up
    ///
    /// # Returns
    /// A clone of the Variable if found, None otherwise.
    fn get_variable(&self, name: &str) -> Option<Variable>;

    /// Check if a variable exists.
    ///
    /// # Arguments
    /// * `name` - The variable name to check
    ///
    /// # Returns
    /// True if the variable exists, false otherwise.
    fn has_variable(&self, name: &str) -> bool;
}

impl VariableManager for CodeGenerator {
    fn allocate_variable(&mut self, name: &str, var_type: &Type, is_const: bool) -> u16 {
        let size = var_type.size() as u16;
        let address = self.next_var_address;
        self.next_var_address = self.next_var_address.wrapping_add(size);

        self.variables.insert(
            name.to_string(),
            Variable {
                address,
                var_type: var_type.clone(),
                is_const,
            },
        );

        address
    }

    fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).cloned()
    }

    fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}
