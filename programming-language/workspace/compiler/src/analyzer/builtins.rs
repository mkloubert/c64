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

//! Built-in function registration for the semantic analyzer.
//!
//! This module defines the built-in functions available in Cobra64,
//! including I/O functions, memory access, and screen control.

use super::symbol::Symbol;
use super::Analyzer;
use crate::ast::Type;
use crate::error::Span;

/// Trait for registering built-in functions.
pub trait BuiltinRegistry {
    /// Register all built-in functions in the symbol table.
    fn register_builtins(&mut self);

    /// Define a single built-in function.
    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>);
}

impl BuiltinRegistry for Analyzer {
    fn register_builtins(&mut self) {
        // Screen control
        // cls() - clear screen
        self.define_builtin("cls", vec![], None);

        // Output functions
        // print(value) - print without newline
        self.define_builtin("print", vec![Type::String], None);

        // println(value) - print with newline
        self.define_builtin("println", vec![Type::String], None);

        // cursor(x, y) - set cursor position
        self.define_builtin("cursor", vec![Type::Byte, Type::Byte], None);

        // Input functions
        // get_key() -> byte - get key without waiting
        self.define_builtin("get_key", vec![], Some(Type::Byte));

        // read() -> byte - wait for key press
        self.define_builtin("read", vec![], Some(Type::Byte));

        // readln() -> string - read a line of input
        self.define_builtin("readln", vec![], Some(Type::String));

        // Memory access
        // poke(addr, value) - write to memory
        self.define_builtin("poke", vec![Type::Word, Type::Byte], None);

        // peek(addr) -> byte - read from memory
        self.define_builtin("peek", vec![Type::Word], Some(Type::Byte));

        // Random number generation
        // rand() -> fixed - random number between 0.0 and ~0.9375 (15/16)
        self.define_builtin("rand", vec![], Some(Type::Fixed));

        // rand_byte(from, to) -> byte - random integer in range [from, to]
        self.define_builtin("rand_byte", vec![Type::Byte, Type::Byte], Some(Type::Byte));

        // rand_sbyte(from, to) -> sbyte - random signed integer in range [from, to]
        self.define_builtin(
            "rand_sbyte",
            vec![Type::Sbyte, Type::Sbyte],
            Some(Type::Sbyte),
        );

        // rand_word(from, to) -> word - random 16-bit integer in range [from, to]
        self.define_builtin("rand_word", vec![Type::Word, Type::Word], Some(Type::Word));

        // rand_sword(from, to) -> sword - random signed 16-bit integer in range [from, to]
        self.define_builtin(
            "rand_sword",
            vec![Type::Sword, Type::Sword],
            Some(Type::Sword),
        );

        // seed() - reseed the random number generator from hardware entropy
        self.define_builtin("seed", vec![], None);
    }

    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>) {
        let symbol = Symbol::function(name.to_string(), params, return_type, Span::new(0, 0));
        let _ = self.symbols.define(symbol);
    }
}
