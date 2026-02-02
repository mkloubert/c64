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

        // Sprite functions
        // sprite_enable(num, enabled) - enable or disable a sprite
        self.define_builtin("sprite_enable", vec![Type::Byte, Type::Bool], None);

        // sprite_pos(num, x, y) - set sprite position (x is 9-bit, 0-320)
        self.define_builtin("sprite_pos", vec![Type::Byte, Type::Word, Type::Byte], None);

        // sprite_color(num, color) - set sprite color (0-15)
        self.define_builtin("sprite_color", vec![Type::Byte, Type::Byte], None);

        // sprite_data(num, pointer) - set sprite data pointer (block number 0-255)
        self.define_builtin("sprite_data", vec![Type::Byte, Type::Byte], None);

        // sprite_expand_x(num, enabled) - enable/disable horizontal expansion
        self.define_builtin("sprite_expand_x", vec![Type::Byte, Type::Bool], None);

        // sprite_expand_y(num, enabled) - enable/disable vertical expansion
        self.define_builtin("sprite_expand_y", vec![Type::Byte, Type::Bool], None);

        // sprite_multicolor(num, enabled) - enable/disable multicolor mode
        self.define_builtin("sprite_multicolor", vec![Type::Byte, Type::Bool], None);

        // sprite_priority(num, behind_bg) - set sprite priority (true = behind background)
        self.define_builtin("sprite_priority", vec![Type::Byte, Type::Bool], None);

        // sprite_collision() -> byte - get sprite-sprite collision flags (clears on read)
        self.define_builtin("sprite_collision", vec![], Some(Type::Byte));

        // sprite_bg_collision() -> byte - get sprite-background collision flags (clears on read)
        self.define_builtin("sprite_bg_collision", vec![], Some(Type::Byte));

        // Joystick functions
        // joystick(port) -> byte - read raw joystick state (active-low bitmask)
        self.define_builtin("joystick", vec![Type::Byte], Some(Type::Byte));

        // joy_up(port) -> bool - check if joystick is pushed up
        self.define_builtin("joy_up", vec![Type::Byte], Some(Type::Bool));

        // joy_down(port) -> bool - check if joystick is pushed down
        self.define_builtin("joy_down", vec![Type::Byte], Some(Type::Bool));

        // joy_left(port) -> bool - check if joystick is pushed left
        self.define_builtin("joy_left", vec![Type::Byte], Some(Type::Bool));

        // joy_right(port) -> bool - check if joystick is pushed right
        self.define_builtin("joy_right", vec![Type::Byte], Some(Type::Bool));

        // joy_fire(port) -> bool - check if fire button is pressed
        self.define_builtin("joy_fire", vec![Type::Byte], Some(Type::Bool));

        // SID Sound functions
        // sid_volume(vol) - set master volume (0-15)
        self.define_builtin("sid_volume", vec![Type::Byte], None);

        // sid_voice_freq(voice, freq) - set voice frequency (0-65535)
        self.define_builtin("sid_voice_freq", vec![Type::Byte, Type::Word], None);

        // sid_voice_pulse(voice, width) - set pulse width (0-4095)
        self.define_builtin("sid_voice_pulse", vec![Type::Byte, Type::Word], None);

        // sid_voice_wave(voice, waveform) - set waveform (16=tri, 32=saw, 64=pulse, 128=noise)
        self.define_builtin("sid_voice_wave", vec![Type::Byte, Type::Byte], None);

        // sid_voice_adsr(voice, attack, decay, sustain, release) - set envelope (0-15 each)
        self.define_builtin(
            "sid_voice_adsr",
            vec![Type::Byte, Type::Byte, Type::Byte, Type::Byte, Type::Byte],
            None,
        );

        // sid_voice_gate(voice, on) - set gate (start/stop sound)
        self.define_builtin("sid_voice_gate", vec![Type::Byte, Type::Bool], None);

        // sid_clear() - clear all SID registers (silence)
        self.define_builtin("sid_clear", vec![], None);

        // Screen helper functions
        // border(color) - set border color (0-15)
        self.define_builtin("border", vec![Type::Byte], None);

        // background(color) - set background color (0-15)
        self.define_builtin("background", vec![Type::Byte], None);

        // vsync() - wait for vertical blank (raster line 255)
        self.define_builtin("vsync", vec![], None);

        // raster() -> byte - get current raster line
        self.define_builtin("raster", vec![], Some(Type::Byte));
    }

    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>) {
        let symbol = Symbol::function(name.to_string(), params, return_type, Span::new(0, 0));
        let _ = self.symbols.define(symbol);
    }
}
