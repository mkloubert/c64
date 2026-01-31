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

//! Fuzz target for the complete Cobra64 compiler pipeline.
//!
//! This fuzzer feeds random source code through the entire compilation
//! pipeline to find crashes at any stage.
//!
//! Run with:
//!   cargo +nightly fuzz run fuzz_compiler
//!
//! Run for a specific duration:
//!   cargo +nightly fuzz run fuzz_compiler -- -max_total_time=60

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string
    if let Ok(source) = std::str::from_utf8(data) {
        // Run the complete compilation pipeline
        // Should never panic, only return Ok or Err
        let _ = cobra64::compile(source);
    }
});
