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

//! Analysis context for the semantic analyzer.
//!
//! This module defines the context used during semantic analysis to track
//! the current state (inside loop, inside function, expected return type).

use crate::ast::Type;

/// Context for semantic analysis.
///
/// Tracks the current analysis state, including whether we're inside a loop
/// or function, and the expected return type of the current function.
#[derive(Debug, Clone, Default)]
pub struct AnalysisContext {
    /// Whether we're inside a loop (for break/continue validation).
    pub in_loop: bool,
    /// Whether we're inside a function.
    pub in_function: bool,
    /// The expected return type of the current function.
    pub return_type: Option<Type>,
    /// The current function name (for error messages).
    pub function_name: Option<String>,
}
