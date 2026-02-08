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

/// Trait for registering built-in functions and constants.
pub trait BuiltinRegistry {
    /// Register all built-in functions and constants in the symbol table.
    fn register_builtins(&mut self);

    /// Define a single built-in function.
    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>);

    /// Define a built-in constant.
    fn define_builtin_constant(&mut self, name: &str, value_type: Type);
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

        // String operations
        // str_at(s, i) -> byte - get character at position i
        self.define_builtin("str_at", vec![Type::String, Type::Byte], Some(Type::Byte));

        // Sprite control functions
        // sprite_enable(num, enable) - enable/disable a sprite
        self.define_builtin("sprite_enable", vec![Type::Byte, Type::Bool], None);

        // sprites_enable(mask) - enable sprites by bitmask
        self.define_builtin("sprites_enable", vec![Type::Byte], None);

        // sprite_x(num, x) - set sprite X position (0-511)
        self.define_builtin("sprite_x", vec![Type::Byte, Type::Word], None);

        // sprite_y(num, y) - set sprite Y position (0-255)
        self.define_builtin("sprite_y", vec![Type::Byte, Type::Byte], None);

        // sprite_pos(num, x, y) - set sprite position
        self.define_builtin("sprite_pos", vec![Type::Byte, Type::Word, Type::Byte], None);

        // sprite_get_x(num) -> word - get sprite X position
        self.define_builtin("sprite_get_x", vec![Type::Byte], Some(Type::Word));

        // sprite_get_y(num) -> byte - get sprite Y position
        self.define_builtin("sprite_get_y", vec![Type::Byte], Some(Type::Byte));

        // Sprite data and color functions (Phase 2)
        // sprite_data(num, pointer) - set sprite data pointer
        self.define_builtin("sprite_data", vec![Type::Byte, Type::Byte], None);

        // sprite_get_data(num) -> byte - get sprite data pointer
        self.define_builtin("sprite_get_data", vec![Type::Byte], Some(Type::Byte));

        // sprite_color(num, color) - set sprite color
        self.define_builtin("sprite_color", vec![Type::Byte, Type::Byte], None);

        // sprite_get_color(num) -> byte - get sprite color
        self.define_builtin("sprite_get_color", vec![Type::Byte], Some(Type::Byte));

        // sprite_multicolor1(color) - set shared multicolor 1
        self.define_builtin("sprite_multicolor1", vec![Type::Byte], None);

        // sprite_multicolor2(color) - set shared multicolor 2
        self.define_builtin("sprite_multicolor2", vec![Type::Byte], None);

        // sprite_get_multicolor1() -> byte - get shared multicolor 1
        self.define_builtin("sprite_get_multicolor1", vec![], Some(Type::Byte));

        // sprite_get_multicolor2() -> byte - get shared multicolor 2
        self.define_builtin("sprite_get_multicolor2", vec![], Some(Type::Byte));

        // Sprite multicolor and expansion functions (Phase 3)
        // sprite_multicolor(num, enable) - enable/disable multicolor mode for sprite
        self.define_builtin("sprite_multicolor", vec![Type::Byte, Type::Bool], None);

        // sprites_multicolor(mask) - set multicolor mode by bitmask
        self.define_builtin("sprites_multicolor", vec![Type::Byte], None);

        // sprite_is_multicolor(num) -> bool - check if sprite is in multicolor mode
        self.define_builtin("sprite_is_multicolor", vec![Type::Byte], Some(Type::Bool));

        // sprite_expand_x(num, expand) - enable/disable X expansion for sprite
        self.define_builtin("sprite_expand_x", vec![Type::Byte, Type::Bool], None);

        // sprite_expand_y(num, expand) - enable/disable Y expansion for sprite
        self.define_builtin("sprite_expand_y", vec![Type::Byte, Type::Bool], None);

        // sprites_expand_x(mask) - set X expansion by bitmask
        self.define_builtin("sprites_expand_x", vec![Type::Byte], None);

        // sprites_expand_y(mask) - set Y expansion by bitmask
        self.define_builtin("sprites_expand_y", vec![Type::Byte], None);

        // sprite_is_expanded_x(num) -> bool - check if sprite has X expansion
        self.define_builtin("sprite_is_expanded_x", vec![Type::Byte], Some(Type::Bool));

        // sprite_is_expanded_y(num) -> bool - check if sprite has Y expansion
        self.define_builtin("sprite_is_expanded_y", vec![Type::Byte], Some(Type::Bool));

        // Sprite priority and collision functions (Phase 4)
        // sprite_priority(num, behind_bg) - set sprite priority (behind background if true)
        self.define_builtin("sprite_priority", vec![Type::Byte, Type::Bool], None);

        // sprites_priority(mask) - set priority by bitmask
        self.define_builtin("sprites_priority", vec![Type::Byte], None);

        // sprite_get_priority(num) -> bool - check if sprite is behind background
        self.define_builtin("sprite_get_priority", vec![Type::Byte], Some(Type::Bool));

        // sprite_collision_sprite() -> byte - read sprite-sprite collision register
        self.define_builtin("sprite_collision_sprite", vec![], Some(Type::Byte));

        // sprite_collision_bg() -> byte - read sprite-background collision register
        self.define_builtin("sprite_collision_bg", vec![], Some(Type::Byte));

        // sprite_collides(mask) -> bool - check if any sprite in mask has collision
        self.define_builtin("sprite_collides", vec![Type::Byte], Some(Type::Bool));

        // C64 color constants (VIC-II palette)
        self.define_builtin_constant("COLOR_BLACK", Type::Byte);
        self.define_builtin_constant("COLOR_WHITE", Type::Byte);
        self.define_builtin_constant("COLOR_RED", Type::Byte);
        self.define_builtin_constant("COLOR_CYAN", Type::Byte);
        self.define_builtin_constant("COLOR_PURPLE", Type::Byte);
        self.define_builtin_constant("COLOR_GREEN", Type::Byte);
        self.define_builtin_constant("COLOR_BLUE", Type::Byte);
        self.define_builtin_constant("COLOR_YELLOW", Type::Byte);
        self.define_builtin_constant("COLOR_ORANGE", Type::Byte);
        self.define_builtin_constant("COLOR_BROWN", Type::Byte);
        self.define_builtin_constant("COLOR_LIGHT_RED", Type::Byte);
        self.define_builtin_constant("COLOR_DARK_GRAY", Type::Byte);
        self.define_builtin_constant("COLOR_GRAY", Type::Byte);
        self.define_builtin_constant("COLOR_LIGHT_GREEN", Type::Byte);
        self.define_builtin_constant("COLOR_LIGHT_BLUE", Type::Byte);
        self.define_builtin_constant("COLOR_LIGHT_GRAY", Type::Byte);

        // VIC-II sprite register constants
        self.define_builtin_constant("VIC_SPRITE_ENABLE", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_X_MSB", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_EXPAND_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_PRIORITY", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_MULTICOLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_EXPAND_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_COLLISION_SPRITE", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_COLLISION_BG", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_MULTICOLOR1", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_MULTICOLOR2", Type::Word);
        self.define_builtin_constant("VIC_SPRITE_POINTER_BASE", Type::Word);

        // Individual sprite position registers
        self.define_builtin_constant("VIC_SPRITE0_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE0_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE1_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE1_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE2_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE2_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE3_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE3_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE4_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE4_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE5_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE5_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE6_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE6_Y", Type::Word);
        self.define_builtin_constant("VIC_SPRITE7_X", Type::Word);
        self.define_builtin_constant("VIC_SPRITE7_Y", Type::Word);

        // Individual sprite color registers
        self.define_builtin_constant("VIC_SPRITE0_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE1_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE2_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE3_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE4_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE5_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE6_COLOR", Type::Word);
        self.define_builtin_constant("VIC_SPRITE7_COLOR", Type::Word);
    }

    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>) {
        let symbol = Symbol::function(name.to_string(), params, return_type, Span::new(0, 0));
        let _ = self.symbols.define(symbol);
    }

    fn define_builtin_constant(&mut self, name: &str, value_type: Type) {
        let symbol = Symbol::variable(name.to_string(), value_type, true, Span::new(0, 0));
        let _ = self.symbols.define(symbol);
    }
}
