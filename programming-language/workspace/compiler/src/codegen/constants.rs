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

//! Memory layout constants for the C64.
//!
//! This module defines the memory addresses and sizes used by the code generator.

/// Standard C64 BASIC program start address.
pub const PROGRAM_START: u16 = 0x0801;

/// Size of the BASIC stub in bytes.
pub const BASIC_STUB_SIZE: u16 = 13;

/// Machine code starts at $080E (2062 decimal).
pub const CODE_START: u16 = PROGRAM_START + BASIC_STUB_SIZE;

/// Default start address for variable allocation.
pub const VARIABLE_START: u16 = 0xC000;
