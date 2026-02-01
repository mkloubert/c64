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

//! Snapshot tests for the Cobra64 compiler.
//!
//! These tests use the `insta` crate to capture and verify output
//! from various compiler stages.

use cobra64::{error::format_error, lexer, parser, Span, Token};

// ============================================================================
// Lexer Snapshot Tests
// ============================================================================

/// Format tokens for snapshot comparison.
fn format_tokens(tokens: &[(Token, Span)]) -> String {
    let mut output = String::new();
    for (token, span) in tokens {
        output.push_str(&format!("{:?} @ {}..{}\n", token, span.start, span.end));
    }
    output
}

#[test]
fn test_lexer_snapshot_minimal() {
    let source = include_str!("fixtures/valid/minimal.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    insta::assert_snapshot!("lexer_minimal", format_tokens(&tokens));
}

#[test]
fn test_lexer_snapshot_hello() {
    let source = include_str!("fixtures/valid/hello.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    insta::assert_snapshot!("lexer_hello", format_tokens(&tokens));
}

#[test]
fn test_lexer_snapshot_variables() {
    let source = include_str!("fixtures/valid/variables.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    insta::assert_snapshot!("lexer_variables", format_tokens(&tokens));
}

#[test]
fn test_lexer_snapshot_all_literals() {
    let source = include_str!("fixtures/valid/all_literals.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    insta::assert_snapshot!("lexer_all_literals", format_tokens(&tokens));
}

#[test]
fn test_lexer_snapshot_operators() {
    let source = include_str!("fixtures/valid/operators.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    insta::assert_snapshot!("lexer_operators", format_tokens(&tokens));
}

// ============================================================================
// AST Snapshot Tests
// ============================================================================

#[test]
fn test_ast_snapshot_minimal() {
    let source = include_str!("fixtures/valid/minimal.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_minimal", format!("{:#?}", ast));
}

#[test]
fn test_ast_snapshot_hello() {
    let source = include_str!("fixtures/valid/hello.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_hello", format!("{:#?}", ast));
}

#[test]
fn test_ast_snapshot_variables() {
    let source = include_str!("fixtures/valid/variables.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_variables", format!("{:#?}", ast));
}

#[test]
fn test_ast_snapshot_control_flow() {
    let source = include_str!("fixtures/valid/control_flow.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_control_flow", format!("{:#?}", ast));
}

#[test]
fn test_ast_snapshot_functions() {
    let source = include_str!("fixtures/valid/functions.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_functions", format!("{:#?}", ast));
}

// ============================================================================
// Code Generation Snapshot Tests
// ============================================================================

/// Format code as hex dump for snapshot comparison.
fn format_hex_dump(code: &[u8]) -> String {
    let mut output = String::new();
    output.push_str(&format!("Total bytes: {}\n\n", code.len()));

    for (i, chunk) in code.chunks(16).enumerate() {
        let offset = i * 16;
        output.push_str(&format!("{:04X}: ", offset));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                output.push(' ');
            }
        }

        // Padding for incomplete lines
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                output.push_str("   ");
                if j == 7 {
                    output.push(' ');
                }
            }
        }

        // ASCII representation
        output.push_str(" |");
        for byte in chunk {
            if *byte >= 0x20 && *byte < 0x7F {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        output.push_str("|\n");
    }

    output
}

#[test]
fn test_codegen_snapshot_minimal() {
    let source = include_str!("fixtures/valid/minimal.cb64");
    let code = cobra64::compile(source).unwrap();
    insta::assert_snapshot!("codegen_minimal", format_hex_dump(&code));
}

#[test]
fn test_codegen_snapshot_hello() {
    let source = include_str!("fixtures/valid/hello.cb64");
    let code = cobra64::compile(source).unwrap();
    insta::assert_snapshot!("codegen_hello", format_hex_dump(&code));
}

#[test]
fn test_codegen_snapshot_variables() {
    let source = include_str!("fixtures/valid/variables.cb64");
    let code = cobra64::compile(source).unwrap();
    insta::assert_snapshot!("codegen_variables", format_hex_dump(&code));
}

// ============================================================================
// Error Message Snapshot Tests
// ============================================================================

/// Compile a source and return the formatted error message.
fn get_error_message(source: &str, filename: &str) -> String {
    match cobra64::compile(source) {
        Ok(_) => "No error (compilation succeeded)".to_string(),
        Err(e) => format_error(&e, source, Some(filename)),
    }
}

#[test]
fn test_error_snapshot_missing_main() {
    let source = include_str!("fixtures/invalid/missing_main.cb64");
    let error = get_error_message(source, "missing_main.cb64");
    insta::assert_snapshot!("error_missing_main", error);
}

#[test]
fn test_error_snapshot_undefined_variable() {
    let source = include_str!("fixtures/invalid/undefined_variable.cb64");
    let error = get_error_message(source, "undefined_variable.cb64");
    insta::assert_snapshot!("error_undefined_variable", error);
}

#[test]
fn test_error_snapshot_type_mismatch() {
    let source = include_str!("fixtures/invalid/type_mismatch.cb64");
    let error = get_error_message(source, "type_mismatch.cb64");
    insta::assert_snapshot!("error_type_mismatch", error);
}

#[test]
fn test_error_snapshot_syntax_missing_colon() {
    let source = include_str!("fixtures/invalid/syntax_missing_colon.cb64");
    let error = get_error_message(source, "syntax_missing_colon.cb64");
    insta::assert_snapshot!("error_syntax_missing_colon", error);
}

#[test]
fn test_error_snapshot_duplicate_function() {
    let source = include_str!("fixtures/invalid/duplicate_function.cb64");
    let error = get_error_message(source, "duplicate_function.cb64");
    insta::assert_snapshot!("error_duplicate_function", error);
}

#[test]
fn test_error_snapshot_break_outside_loop() {
    let source = include_str!("fixtures/invalid/break_outside_loop.cb64");
    let error = get_error_message(source, "break_outside_loop.cb64");
    insta::assert_snapshot!("error_break_outside_loop", error);
}

#[test]
fn test_error_snapshot_wrong_argument_count() {
    let source = include_str!("fixtures/invalid/wrong_argument_count.cb64");
    let error = get_error_message(source, "wrong_argument_count.cb64");
    insta::assert_snapshot!("error_wrong_argument_count", error);
}

// ============================================================================
// Array Snapshot Tests
// ============================================================================

#[test]
fn test_ast_snapshot_arrays_basic() {
    let source = include_str!("fixtures/valid/arrays_basic.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_arrays_basic", format!("{:#?}", ast));
}

#[test]
fn test_ast_snapshot_arrays_word() {
    let source = include_str!("fixtures/valid/arrays_word.cb64");
    let tokens = lexer::tokenize(source).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    insta::assert_snapshot!("ast_arrays_word", format!("{:#?}", ast));
}

#[test]
fn test_codegen_snapshot_arrays_basic() {
    let source = include_str!("fixtures/valid/arrays_basic.cb64");
    let code = cobra64::compile(source).unwrap();
    insta::assert_snapshot!("codegen_arrays_basic", format_hex_dump(&code));
}

#[test]
fn test_error_snapshot_array_type_mismatch() {
    let source = include_str!("fixtures/invalid/array_type_mismatch.cb64");
    let error = get_error_message(source, "array_type_mismatch.cb64");
    insta::assert_snapshot!("error_array_type_mismatch", error);
}

#[test]
fn test_error_snapshot_array_size_mismatch() {
    let source = include_str!("fixtures/invalid/array_size_mismatch.cb64");
    let error = get_error_message(source, "array_size_mismatch.cb64");
    insta::assert_snapshot!("error_array_size_mismatch", error);
}

#[test]
fn test_error_snapshot_array_index_type() {
    let source = include_str!("fixtures/invalid/array_index_type.cb64");
    let error = get_error_message(source, "array_index_type.cb64");
    insta::assert_snapshot!("error_array_index_type", error);
}

// ============================================================================
// Token Position Verification Tests
// ============================================================================

#[test]
fn test_token_positions_are_valid() {
    let source = include_str!("fixtures/valid/operators.cb64");
    let tokens = lexer::tokenize(source).unwrap();

    for (token, span) in &tokens {
        // Verify span is within source bounds
        assert!(
            span.start <= source.len(),
            "Token {:?} start {} exceeds source length {}",
            token,
            span.start,
            source.len()
        );
        assert!(
            span.end <= source.len(),
            "Token {:?} end {} exceeds source length {}",
            token,
            span.end,
            source.len()
        );
        assert!(
            span.start <= span.end,
            "Token {:?} start {} > end {}",
            token,
            span.start,
            span.end
        );
    }
}

#[test]
fn test_token_spans_are_reasonable() {
    let source = "def main():\n    pass\n";
    let tokens = lexer::tokenize(source).unwrap();

    // Filter out synthetic tokens like INDENT/DEDENT/NEWLINE which may not have real spans
    let real_tokens: Vec<_> = tokens
        .iter()
        .filter(|(t, _)| !matches!(t, Token::Indent | Token::Dedent | Token::Newline))
        .collect();

    // Verify no huge gaps between tokens (allowing for whitespace)
    for window in real_tokens.windows(2) {
        let (_, span1) = window[0];
        let (_, span2) = window[1];
        let gap = span2.start.saturating_sub(span1.end);
        // Allow reasonable gaps for whitespace
        assert!(
            gap < 50,
            "Large gap between tokens: span1.end={}, span2.start={}",
            span1.end,
            span2.start
        );
    }
}
