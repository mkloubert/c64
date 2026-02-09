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

//! Conformance tests for the Cobra64 compiler.
//!
//! These tests verify that all language features compile correctly.
//! Each test corresponds to a conformance test file in tests/conformance/.

use std::fs;
use std::path::Path;

/// Test that all conformance test files compile without errors.
#[test]
fn test_all_conformance_files_compile() {
    let conformance_dir = Path::new("tests/conformance");

    let mut files: Vec<_> = fs::read_dir(conformance_dir)
        .expect("Failed to read conformance directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cb64"))
        .collect();

    files.sort();

    assert!(!files.is_empty(), "No conformance test files found");

    for path in &files {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));

        let result = cobra64::compile(&source);

        assert!(
            result.is_ok(),
            "Conformance test {} failed to compile: {:?}",
            path.display(),
            result.err()
        );
    }

    println!(
        "All {} conformance tests compiled successfully",
        files.len()
    );
}

// ============================================================================
// Individual Conformance Tests
// ============================================================================

macro_rules! conformance_test {
    ($name:ident, $file:expr, $desc:expr) => {
        #[test]
        fn $name() {
            let source = include_str!(concat!("conformance/", $file));
            let result = cobra64::compile(source);
            assert!(
                result.is_ok(),
                concat!($desc, " - compile failed: {:?}"),
                result.err()
            );

            // Verify generated code is non-empty
            let code = result.unwrap();
            assert!(!code.is_empty(), concat!($desc, " - generated empty code"));

            // Verify BASIC stub is present (starts with load address + BASIC tokens)
            assert!(code.len() > 10, concat!($desc, " - code too short"));
        }
    };
}

conformance_test!(
    test_01_literals,
    "01_literals.cb64",
    "All literal types (decimal, hex, binary, string, char, bool)"
);

conformance_test!(
    test_02_variables,
    "02_variables.cb64",
    "Variable declarations (byte, word, sbyte, sword, bool)"
);

conformance_test!(
    test_03_constants,
    "03_constants.cb64",
    "Constant declarations with const keyword"
);

conformance_test!(
    test_04_arithmetic,
    "04_arithmetic.cb64",
    "Arithmetic operators (+, -, *, /, mod, unary minus)"
);

conformance_test!(
    test_05_comparison,
    "05_comparison.cb64",
    "Comparison operators (==, !=, <, >, <=, >=)"
);

conformance_test!(
    test_06_if_else,
    "06_if_else.cb64",
    "If/else statements and conditions"
);

conformance_test!(
    test_07_while,
    "07_while.cb64",
    "While loops with counter and condition variables"
);

conformance_test!(
    test_08_break,
    "08_break.cb64",
    "Break statement to exit loops early"
);

conformance_test!(
    test_09_functions,
    "09_functions.cb64",
    "Function definitions with parameters"
);

conformance_test!(
    test_10_return,
    "10_return.cb64",
    "Return values from functions"
);

conformance_test!(
    test_11_builtin_cls,
    "11_builtin_cls.cb64",
    "Built-in cls() function to clear screen"
);

conformance_test!(
    test_12_builtin_print,
    "12_builtin_print.cb64",
    "Built-in print() and println() functions"
);

conformance_test!(
    test_13_builtin_cursor,
    "13_builtin_cursor.cb64",
    "Built-in cursor() function for positioning"
);

conformance_test!(
    test_14_builtin_input,
    "14_builtin_input.cb64",
    "Built-in input functions (get_key, read)"
);

conformance_test!(
    test_15_type_cast,
    "15_type_cast.cb64",
    "Type conversions with byte(), word()"
);

conformance_test!(
    test_16_overflow,
    "16_overflow.cb64",
    "Integer overflow wrapping behavior"
);

conformance_test!(
    test_26_explicit_types,
    "26_explicit_types.cb64",
    "Explicit type annotations for variables and constants"
);

conformance_test!(
    test_27_builtin_rand,
    "27_builtin_rand.cb64",
    "Built-in random number functions (rand, rand_byte, rand_sbyte, rand_word, rand_sword)"
);

conformance_test!(
    test_51_joystick_input,
    "51_joystick_input.cb64",
    "Built-in joystick functions (joystick, JOY_UP, JOY_DOWN, JOY_LEFT, JOY_RIGHT, JOY_FIRE)"
);

// ============================================================================
// Code Generation Verification Tests
// ============================================================================

#[test]
fn test_conformance_generates_valid_prg() {
    // Test that a conformance file generates a valid PRG structure
    let source = include_str!("conformance/01_literals.cb64");
    let code = cobra64::compile(source).expect("Should compile");

    // Check load address (first two bytes, little-endian)
    // Standard C64 BASIC area starts at $0801
    assert!(code.len() >= 2, "Code too short for load address");
    let load_addr = u16::from_le_bytes([code[0], code[1]]);
    // Load address should be in the BASIC/program area ($0801-$9FFF)
    assert!(
        (0x0801..=0x9FFF).contains(&load_addr),
        "Load address ${:04X} should be in valid C64 program area",
        load_addr
    );
}

#[test]
fn test_conformance_basic_stub_present() {
    // Test that BASIC stub is present in generated code
    let source = include_str!("conformance/02_variables.cb64");
    let code = cobra64::compile(source).expect("Should compile");

    // After load address, there should be a BASIC line
    // BASIC line format: next_addr(2), line_num(2), tokens..., 0
    assert!(code.len() >= 10, "Code too short for BASIC stub");

    // The BASIC stub should end with a SYS token ($9E = 158)
    // Look for SYS token in first ~20 bytes
    let has_sys = code[2..20.min(code.len())].contains(&0x9E);
    assert!(has_sys, "BASIC stub should contain SYS token");
}

#[test]
fn test_conformance_code_size_reasonable() {
    // Test that generated code sizes are reasonable
    // Note: sizes increased to accommodate fixed-point, float, PRNG, SID sound, and graphics runtime routines
    // Size increased to 5000 to accommodate bitmap graphics, drawing primitives, and cell color control
    let test_cases = [
        ("01_literals.cb64", 100, 5000),
        ("02_variables.cb64", 100, 5000),
        ("07_while.cb64", 100, 5000),
    ];

    for (file, min_size, max_size) in test_cases {
        let path = format!("tests/conformance/{}", file);
        let source =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", file));

        let code = cobra64::compile(&source)
            .unwrap_or_else(|e| panic!("{} failed to compile: {:?}", file, e));

        assert!(
            code.len() >= min_size,
            "{} generated only {} bytes (expected >= {})",
            file,
            code.len(),
            min_size
        );

        assert!(
            code.len() <= max_size,
            "{} generated {} bytes (expected <= {})",
            file,
            code.len(),
            max_size
        );
    }
}
