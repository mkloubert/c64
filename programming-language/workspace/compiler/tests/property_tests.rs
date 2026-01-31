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

//! Property-based tests for the Cobra64 compiler.
//!
//! These tests verify important invariants and properties that should
//! hold for all inputs, using proptest for random input generation.

use cobra64::{lexer, parser};
use proptest::prelude::*;

// ============================================================================
// Lexer Property Tests
// ============================================================================

proptest! {
    /// Property: All valid tokens have non-negative length spans.
    #[test]
    fn prop_lexer_spans_valid(source in "[a-zA-Z0-9_ +\\-*/=:()\\n]{0,200}") {
        if let Ok(tokens) = lexer::tokenize(&source) {
            for (_, span) in &tokens {
                prop_assert!(
                    span.start <= span.end,
                    "Invalid span: start {} > end {}", span.start, span.end
                );
            }
        }
    }

    /// Property: Token spans are within source bounds.
    #[test]
    fn prop_lexer_spans_in_bounds(source in "[a-zA-Z0-9_ +\\-*/=:()\\n]{0,200}") {
        if let Ok(tokens) = lexer::tokenize(&source) {
            let source_len = source.len();
            for (token, span) in &tokens {
                prop_assert!(
                    span.end <= source_len,
                    "Token {:?} span end {} exceeds source length {}",
                    token, span.end, source_len
                );
            }
        }
    }

    /// Property: Token spans are non-overlapping (for real tokens).
    #[test]
    fn prop_lexer_spans_non_overlapping(source in "[a-zA-Z0-9_ ]{0,100}") {
        if let Ok(tokens) = lexer::tokenize(&source) {
            // Filter to only tokens with actual spans (not synthetic like INDENT/DEDENT)
            let real_tokens: Vec<_> = tokens.iter()
                .filter(|(t, span)| {
                    span.start != span.end || !matches!(t,
                        cobra64::Token::Indent | cobra64::Token::Dedent)
                })
                .collect();

            for window in real_tokens.windows(2) {
                let (_, span1) = window[0];
                let (_, span2) = window[1];
                prop_assert!(
                    span1.end <= span2.start,
                    "Overlapping spans: {:?} and {:?}", span1, span2
                );
            }
        }
    }

    /// Property: Lexer produces consistent results (deterministic).
    #[test]
    fn prop_lexer_deterministic(source in "[a-zA-Z0-9_ ]{0,100}") {
        let result1 = lexer::tokenize(&source);
        let result2 = lexer::tokenize(&source);

        match (result1, result2) {
            (Ok(tokens1), Ok(tokens2)) => {
                prop_assert_eq!(tokens1.len(), tokens2.len(),
                    "Different token counts on same input");
            }
            (Err(_), Err(_)) => {
                // Both failed, that's consistent
            }
            _ => {
                prop_assert!(false, "Inconsistent lexer results");
            }
        }
    }

    /// Property: Numbers in valid range tokenize correctly.
    #[test]
    fn prop_lexer_valid_numbers(n in 0u16..=65535u16) {
        let source = format!("def main():\n    x: word = {}\n", n);
        let result = lexer::tokenize(&source);
        prop_assert!(result.is_ok(), "Valid number {} should tokenize", n);
    }

    /// Property: Hex numbers tokenize correctly.
    #[test]
    fn prop_lexer_hex_numbers(n in 0u16..=65535u16) {
        let source = format!("def main():\n    x: word = ${:X}\n", n);
        let result = lexer::tokenize(&source);
        prop_assert!(result.is_ok(), "Hex ${:X} should tokenize", n);
    }

    /// Property: Binary numbers (byte range) tokenize correctly.
    #[test]
    fn prop_lexer_binary_numbers(n in 0u8..=255u8) {
        let source = format!("def main():\n    x: byte = %{:08b}\n", n);
        let result = lexer::tokenize(&source);
        prop_assert!(result.is_ok(), "Binary %{:08b} should tokenize", n);
    }
}

// ============================================================================
// Parser Property Tests
// ============================================================================

proptest! {
    /// Property: Valid minimal programs always parse.
    #[test]
    fn prop_parser_minimal_programs(
        name in "fn_[a-z][a-z0-9]{0,6}",  // Prefix to avoid keywords
        body in prop::sample::select(vec!["pass", "return"]),
    ) {
        let source = format!("def {}():\n    {}\n", name, body);
        if let Ok(tokens) = lexer::tokenize(&source) {
            let result = parser::parse(&tokens);
            prop_assert!(result.is_ok(),
                "Minimal program should parse: def {}(): {}", name, body);
        }
    }

    /// Property: Parser is deterministic.
    #[test]
    fn prop_parser_deterministic(source in "def [a-z]+\\(\\):\n    pass\n") {
        if let Ok(tokens) = lexer::tokenize(&source) {
            let result1 = parser::parse(&tokens);
            let result2 = parser::parse(&tokens);

            match (&result1, &result2) {
                (Ok(_), Ok(_)) | (Err(_), Err(_)) => {
                    // Consistent
                }
                _ => {
                    prop_assert!(false, "Parser gave inconsistent results");
                }
            }
        }
    }

    /// Property: Valid programs produce non-empty AST.
    #[test]
    fn prop_parser_produces_ast(
        var in "[a-z]",
        val in 0u8..100,
    ) {
        let source = format!("def main():\n    {}: byte = {}\n", var, val);

        if let Ok(tokens) = lexer::tokenize(&source) {
            if let Ok(ast) = parser::parse(&tokens) {
                // Check that AST was produced with items
                prop_assert!(!ast.items.is_empty(),
                    "AST should have at least one item (main function)");
            }
        }
    }

    /// Property: Nested expressions parse without stack overflow.
    #[test]
    fn prop_parser_nested_expressions(depth in 1usize..30) {
        let opens: String = "(".repeat(depth);
        let closes: String = ")".repeat(depth);
        let source = format!("def main():\n    x: byte = {}1{}\n", opens, closes);

        if let Ok(tokens) = lexer::tokenize(&source) {
            // Should not panic or stack overflow
            let _ = parser::parse(&tokens);
        }
    }
}

// ============================================================================
// Code Generator Property Tests
// ============================================================================

proptest! {
    /// Property: Valid programs generate non-empty code.
    #[test]
    fn prop_codegen_non_empty(
        body in prop::sample::select(vec![
            "pass",
            "println(\"HI\")",
            "x: byte = 1",
        ]),
    ) {
        let source = format!("def main():\n    {}\n", body);
        if let Ok(code) = cobra64::compile(&source) {
            prop_assert!(!code.is_empty(), "Generated code should not be empty");
            prop_assert!(code.len() >= 10, "Code should include at least load address + stub");
        }
    }

    /// Property: Generated code has valid load address.
    #[test]
    fn prop_codegen_valid_load_address(val in 0u8..100) {
        let source = format!("def main():\n    x: byte = {}\n", val);
        if let Ok(code) = cobra64::compile(&source) {
            prop_assert!(code.len() >= 2, "Code too short for load address");

            let load_addr = u16::from_le_bytes([code[0], code[1]]);
            prop_assert!(
                load_addr >= 0x0801 && load_addr <= 0x9FFF,
                "Load address ${:04X} out of valid range", load_addr
            );
        }
    }

    /// Property: Generated code contains BASIC stub (SYS token).
    #[test]
    fn prop_codegen_has_basic_stub(s in "[A-Z]{1,10}") {
        let source = format!("def main():\n    println(\"{}\")\n", s);
        if let Ok(code) = cobra64::compile(&source) {
            // SYS token in BASIC is $9E (158)
            let has_sys = code[2..code.len().min(30)].contains(&0x9E);
            prop_assert!(has_sys, "Code should contain SYS token for BASIC stub");
        }
    }

    /// Property: Code size is reasonable for program complexity.
    #[test]
    fn prop_codegen_reasonable_size(count in 1usize..20) {
        let mut source = String::from("def main():\n");
        for i in 0..count {
            source.push_str(&format!("    x{}: byte = {}\n", i, i % 256));
        }

        if let Ok(code) = cobra64::compile(&source) {
            // Each variable should add roughly 5-15 bytes
            // Base size increased to account for full runtime library
            // (print_word 16-bit support, print_bool TRUE/FALSE output)
            let min_expected = 20 + count * 5;
            let max_expected = 400 + count * 50;

            prop_assert!(
                code.len() >= min_expected,
                "Code too small: {} bytes for {} variables (expected >= {})",
                code.len(), count, min_expected
            );
            prop_assert!(
                code.len() <= max_expected,
                "Code too large: {} bytes for {} variables (expected <= {})",
                code.len(), count, max_expected
            );
        }
    }

    /// Property: Identical source produces identical code.
    #[test]
    fn prop_codegen_deterministic(val in 0u8..100) {
        let source = format!("def main():\n    x: byte = {}\n    println(x)\n", val);

        let result1 = cobra64::compile(&source);
        let result2 = cobra64::compile(&source);

        match (result1, result2) {
            (Ok(code1), Ok(code2)) => {
                prop_assert_eq!(code1, code2, "Same source should produce same code");
            }
            (Err(_), Err(_)) => {
                // Both failed consistently
            }
            _ => {
                prop_assert!(false, "Inconsistent compilation results");
            }
        }
    }
}

// ============================================================================
// End-to-End Property Tests
// ============================================================================

proptest! {
    /// Property: Compilation never panics on valid-looking input.
    #[test]
    fn prop_no_panic_valid_structure(
        name in "fn_[a-z][a-z0-9]{0,8}",  // Prefix to avoid keywords
        var in "[a-z]",
        val in 0u8..100,
    ) {
        let source = format!("def {}():\n    {}: byte = {}\n\ndef main():\n    {}()\n",
            name, var, val, name);

        // Should not panic
        let result = std::panic::catch_unwind(|| {
            let _ = cobra64::compile(&source);
        });
        prop_assert!(result.is_ok(), "Compilation panicked");
    }

    /// Property: Error messages contain useful information.
    #[test]
    fn prop_errors_have_messages(source in "[!@#$%^&]{1,20}") {
        // Invalid source should produce error with message
        match cobra64::compile(&source) {
            Ok(_) => {
                // Surprisingly valid, that's fine
            }
            Err(e) => {
                prop_assert!(!e.message.is_empty(), "Error should have message");
            }
        }
    }

    /// Property: Valid function calls compile.
    #[test]
    fn prop_function_calls_work(
        func_name in "fn_[a-z][a-z0-9]{0,5}",  // Prefix with fn_ to avoid keywords
        param_count in 0usize..3,
    ) {
        let params: String = (0..param_count)
            .map(|i| format!("p{}: byte", i))
            .collect::<Vec<_>>()
            .join(", ");

        let args: String = (0..param_count)
            .map(|i| format!("{}", i))
            .collect::<Vec<_>>()
            .join(", ");

        let source = format!(
            "def {}({}):\n    pass\n\ndef main():\n    {}({})\n",
            func_name, params, func_name, args
        );

        let result = cobra64::compile(&source);
        prop_assert!(result.is_ok(),
            "Function with {} params should compile: {:?}",
            param_count, result.err());
    }
}

// ============================================================================
// Regression Property Tests
// ============================================================================

proptest! {
    /// Property: Keywords are not valid identifiers.
    #[test]
    fn prop_keywords_reserved(
        keyword in prop::sample::select(vec![
            "def", "if", "else", "while", "return", "break", "continue",
            "pass", "true", "false", "and", "or", "not", "const",
            "byte", "word", "bool", "sbyte", "sword",
        ])
    ) {
        // Using keyword as variable name should fail
        let source = format!("def main():\n    {}: byte = 1\n", keyword);
        let result = cobra64::compile(&source);
        prop_assert!(result.is_err(),
            "Keyword '{}' should not be valid as variable name", keyword);
    }

    /// Property: Duplicate function names are rejected.
    #[test]
    fn prop_duplicate_functions_rejected(name in "fn_[a-z][a-z0-9]{0,5}") {
        let source = format!(
            "def {}():\n    pass\n\ndef {}():\n    pass\n\ndef main():\n    pass\n",
            name, name
        );
        let result = cobra64::compile(&source);
        prop_assert!(result.is_err(),
            "Duplicate function '{}' should be rejected", name);
    }

    /// Property: Undefined variables are caught.
    #[test]
    fn prop_undefined_variables_caught(
        defined in "[a-z]",
        undefined in "[a-z]",
    ) {
        prop_assume!(defined != undefined);

        let source = format!(
            "def main():\n    {}: byte = 1\n    println({})\n",
            defined, undefined
        );
        let result = cobra64::compile(&source);
        prop_assert!(result.is_err(),
            "Undefined variable '{}' should be caught", undefined);
    }
}
