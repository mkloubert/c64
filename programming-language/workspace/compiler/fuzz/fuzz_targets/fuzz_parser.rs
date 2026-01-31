// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Fuzz target for the Cobra64 parser.
//!
//! This fuzzer generates random token sequences and feeds them to the parser
//! to find crashes, panics, or infinite loops.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_parser
//!
//! Run for a specific duration:
//!   cargo +nightly fuzz run fuzz_parser -- -max_total_time=60

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string
    if let Ok(source) = std::str::from_utf8(data) {
        // First tokenize (may fail, that's ok)
        if let Ok(tokens) = cobra64::lexer::tokenize(source) {
            // Then parse (should never panic)
            let _ = cobra64::parser::parse(&tokens);
        }
    }
});
