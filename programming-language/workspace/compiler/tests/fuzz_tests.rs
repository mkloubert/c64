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

//! Property-based fuzz tests for the Cobra64 compiler.
//!
//! These tests use proptest to generate random inputs and verify
//! that the compiler handles them gracefully (no panics).
//!
//! Unlike cargo-fuzz, these tests run on stable Rust.

use proptest::prelude::*;

// ============================================================================
// Lexer Fuzzing
// ============================================================================

proptest! {
    /// Fuzz the lexer with random ASCII strings.
    /// The lexer should never panic, only return Ok or Err.
    #[test]
    fn fuzz_lexer_ascii(s in "[ -~]{0,500}") {
        let _ = cobra64::lexer::tokenize(&s);
    }

    /// Fuzz the lexer with random bytes (may include invalid UTF-8).
    #[test]
    fn fuzz_lexer_bytes(bytes in prop::collection::vec(any::<u8>(), 0..500)) {
        if let Ok(s) = String::from_utf8(bytes) {
            let _ = cobra64::lexer::tokenize(&s);
        }
    }

    /// Fuzz with strings that look like Cobra64 code.
    #[test]
    fn fuzz_lexer_codelike(
        keyword in prop::sample::select(vec!["def", "if", "else", "while", "return", "break", "pass", "const", "true", "false"]),
        ident in "[a-z_][a-z0-9_]{0,10}",
        num in 0u16..65535,
        op in prop::sample::select(vec!["+", "-", "*", "/", "=", "==", "!=", "<", ">", ":", "(", ")", ","]),
    ) {
        let source = format!("{} {} {} {} {}", keyword, ident, op, num, ident);
        let _ = cobra64::lexer::tokenize(&source);
    }
}

// ============================================================================
// Parser Fuzzing
// ============================================================================

proptest! {
    /// Fuzz the parser with random function-like structures.
    #[test]
    fn fuzz_parser_function(
        name in "[a-z_][a-z0-9_]{0,10}",
        body in "[ a-z0-9_:=+\\-*/()]{0,100}",
    ) {
        let source = format!("def {}():\n    {}\n", name, body);
        if let Ok(tokens) = cobra64::lexer::tokenize(&source) {
            let _ = cobra64::parser::parse(&tokens);
        }
    }

    /// Fuzz with nested control structures.
    #[test]
    fn fuzz_parser_control_flow(
        depth in 1usize..5,
        var in "[a-z]",
    ) {
        let mut source = String::from("def main():\n");
        source.push_str(&format!("    {}: byte = 5\n", var));

        for i in 0..depth {
            let indent = "    ".repeat(i + 1);
            source.push_str(&format!("{}if {} > 0:\n", indent, var));
        }

        let final_indent = "    ".repeat(depth + 1);
        source.push_str(&format!("{}pass\n", final_indent));

        if let Ok(tokens) = cobra64::lexer::tokenize(&source) {
            let _ = cobra64::parser::parse(&tokens);
        }
    }
}

// ============================================================================
// Compiler Pipeline Fuzzing
// ============================================================================

proptest! {
    /// Fuzz the complete compiler with minimal valid-looking programs.
    #[test]
    fn fuzz_compiler_minimal(
        stmt in prop::sample::select(vec!["pass", "break", "continue", "return"]),
    ) {
        let source = format!("def main():\n    {}\n", stmt);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with variable declarations.
    #[test]
    fn fuzz_compiler_variables(
        name in "[a-z_][a-z0-9_]{0,8}",
        typ in prop::sample::select(vec!["byte", "word", "bool"]),
        value in 0u16..256,
    ) {
        let source = format!("def main():\n    {}: {} = {}\n", name, typ, value);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with arithmetic expressions.
    #[test]
    fn fuzz_compiler_arithmetic(
        a in 0u8..100,
        b in 1u8..100,  // Avoid division by zero
        op in prop::sample::select(vec!["+", "-", "*", "/"]),
    ) {
        let source = format!("def main():\n    x: byte = {} {} {}\n", a, op, b);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with print statements.
    #[test]
    fn fuzz_compiler_print(
        s in "[A-Z ]{0,20}",
    ) {
        let source = format!("def main():\n    println(\"{}\")\n", s);
        let _ = cobra64::compile(&source);
    }
}

// ============================================================================
// Edge Case Fuzzing
// ============================================================================

proptest! {
    /// Fuzz with deeply nested parentheses.
    #[test]
    fn fuzz_nested_parens(depth in 1usize..20) {
        let opens: String = "(".repeat(depth);
        let closes: String = ")".repeat(depth);
        let source = format!("def main():\n    x: byte = {}1{}\n", opens, closes);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with long identifiers.
    #[test]
    fn fuzz_long_identifiers(name in "[a-z_]{1,100}") {
        let source = format!("def main():\n    {}: byte = 1\n", name);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with boundary numbers.
    #[test]
    fn fuzz_boundary_numbers(n in prop::sample::select(vec![0u16, 1, 127, 128, 255, 256, 32767, 32768, 65535])) {
        let source = format!("def main():\n    x: word = {}\n", n);
        let _ = cobra64::compile(&source);
    }

    /// Fuzz with special string content.
    #[test]
    fn fuzz_string_escapes(
        content in prop::sample::select(vec![
            r#""#,
            r#"A"#,
            r#"\n"#,
            r#"\t"#,
            r#"\\"#,
            r#"\""#,
            r#"HELLO\nWORLD"#,
        ])
    ) {
        let source = format!("def main():\n    println(\"{}\")\n", content);
        let _ = cobra64::compile(&source);
    }
}

// ============================================================================
// Stress Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Stress test with many statements.
    #[test]
    fn fuzz_many_statements(count in 1usize..50) {
        let mut source = String::from("def main():\n");
        for i in 0..count {
            source.push_str(&format!("    x{}: byte = {}\n", i, i % 256));
        }
        let _ = cobra64::compile(&source);
    }

    /// Stress test with many functions.
    #[test]
    fn fuzz_many_functions(count in 1usize..20) {
        let mut source = String::new();
        for i in 0..count {
            source.push_str(&format!("def func{}():\n    pass\n\n", i));
        }
        source.push_str("def main():\n    pass\n");
        let _ = cobra64::compile(&source);
    }
}

// ============================================================================
// Invariant Tests
// ============================================================================

proptest! {
    /// Verify that tokenizing never produces overlapping spans.
    #[test]
    fn invariant_token_spans_non_overlapping(s in "[a-z0-9 +\\-*/=:()]{0,200}") {
        if let Ok(tokens) = cobra64::lexer::tokenize(&s) {
            let mut last_end = 0;
            for (_, span) in &tokens {
                prop_assert!(span.start >= last_end,
                    "Token spans overlap: last_end={}, span.start={}", last_end, span.start);
                prop_assert!(span.start <= span.end,
                    "Invalid span: start={} > end={}", span.start, span.end);
                last_end = span.end;
            }
        }
    }

    /// Verify that compilation either succeeds or fails gracefully.
    #[test]
    fn invariant_no_panic(s in "[ -~]{0,300}") {
        // This test passes if compile() doesn't panic
        let result = std::panic::catch_unwind(|| {
            let _ = cobra64::compile(&s);
        });
        prop_assert!(result.is_ok(), "Compiler panicked on input");
    }
}
