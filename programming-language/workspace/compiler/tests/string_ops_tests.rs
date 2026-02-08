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

//! Tests for string operations.
//!
//! Tests cover:
//! - len() on strings (returns byte)
//! - len() on arrays still works (returns word)
//! - str_at() to get character at index
//! - String concatenation with + operator

use cobra64::error::ErrorCode;
use cobra64::{analyzer, codegen, lexer, parser};

// ============================================================================
// Helper Functions
// ============================================================================

fn analyze_source(source: &str) -> Result<(), cobra64::error::CompileError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;
    analyzer::analyze(&program).map_err(|errors| errors.into_iter().next().unwrap())?;
    Ok(())
}

fn analyze_source_errors(source: &str) -> Vec<cobra64::error::CompileError> {
    let tokens = match lexer::tokenize(source) {
        Ok(t) => t,
        Err(e) => return vec![e],
    };
    let program = match parser::parse(&tokens) {
        Ok(p) => p,
        Err(e) => return vec![e],
    };
    match analyzer::analyze(&program) {
        Ok(_) => vec![],
        Err(errors) => errors,
    }
}

fn compile_source(source: &str) -> Result<Vec<u8>, cobra64::error::CompileError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;
    analyzer::analyze(&program).map_err(|errors| errors.into_iter().next().unwrap())?;
    codegen::generate(&program)
}

// ============================================================================
// Semantic Analyzer Tests - len() on Strings
// ============================================================================

#[test]
fn test_len_string_literal() {
    let result = analyze_source(r#"
def main():
    x: byte = len("HELLO")
"#);
    assert!(result.is_ok(), "len() on string literal should work");
}

#[test]
fn test_len_string_variable() {
    let result = analyze_source(r#"
def main():
    name: string = "WORLD"
    x: byte = len(name)
"#);
    assert!(result.is_ok(), "len() on string variable should work");
}

#[test]
fn test_len_empty_string() {
    let result = analyze_source(r#"
def main():
    x: byte = len("")
"#);
    assert!(result.is_ok(), "len() on empty string should work");
}

#[test]
fn test_len_string_returns_byte() {
    // len() on string should return byte, not word
    // This should fail because we're assigning byte to word unnecessarily
    // but it should succeed because byte is assignable to word
    let result = analyze_source(r#"
def main():
    x: word = len("TEST")
"#);
    // byte is assignable to word, so this should work
    assert!(result.is_ok(), "len() result (byte) should be assignable to word");
}

#[test]
fn test_len_string_in_expression() {
    let result = analyze_source(r#"
def main():
    name: string = "TEST"
    x: byte = len(name) + 1
"#);
    assert!(result.is_ok(), "len() on string should work in expressions");
}

// ============================================================================
// Semantic Analyzer Tests - len() on Arrays (Regression)
// ============================================================================

#[test]
fn test_len_array_still_works() {
    let result = analyze_source(r#"
def main():
    arr: byte[10]
    x: word = len(arr)
"#);
    assert!(result.is_ok(), "len() on array should still work");
}

#[test]
fn test_len_array_with_values() {
    let result = analyze_source(r#"
def main():
    arr: byte[] = [1, 2, 3, 4, 5]
    x: word = len(arr)
"#);
    assert!(result.is_ok(), "len() on initialized array should work");
}

// ============================================================================
// Semantic Analyzer Tests - len() Error Cases
// ============================================================================

#[test]
fn test_len_on_integer_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = 42
    y: byte = len(x)
"#);
    assert!(!errors.is_empty(), "len() on integer should fail");
    assert_eq!(errors[0].code, ErrorCode::TypeMismatch);
}

#[test]
fn test_len_on_bool_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: bool = true
    y: byte = len(x)
"#);
    assert!(!errors.is_empty(), "len() on bool should fail");
    assert_eq!(errors[0].code, ErrorCode::TypeMismatch);
}

#[test]
fn test_len_no_args_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = len()
"#);
    assert!(!errors.is_empty(), "len() with no arguments should fail");
    assert_eq!(errors[0].code, ErrorCode::WrongNumberOfArguments);
}

#[test]
fn test_len_two_args_fails() {
    let errors = analyze_source_errors(r#"
def main():
    a: string = "A"
    b: string = "B"
    x: byte = len(a, b)
"#);
    assert!(!errors.is_empty(), "len() with two arguments should fail");
    assert_eq!(errors[0].code, ErrorCode::WrongNumberOfArguments);
}

// ============================================================================
// Code Generation Tests
// ============================================================================

#[test]
fn test_codegen_len_string_literal() {
    let result = compile_source(r#"
def main():
    x: byte = len("HELLO")
"#);
    assert!(result.is_ok(), "Should compile len() on string literal");
}

#[test]
fn test_codegen_len_string_variable() {
    let result = compile_source(r#"
def main():
    name: string = "WORLD"
    x: byte = len(name)
"#);
    assert!(result.is_ok(), "Should compile len() on string variable");
}

#[test]
fn test_codegen_len_empty_string() {
    let result = compile_source(r#"
def main():
    x: byte = len("")
"#);
    assert!(result.is_ok(), "Should compile len() on empty string");
}

#[test]
fn test_codegen_len_string_print() {
    let result = compile_source(r#"
def main():
    name: string = "TEST"
    println(len(name))
"#);
    assert!(result.is_ok(), "Should compile println(len(string))");
}

#[test]
fn test_codegen_len_array_still_works() {
    let result = compile_source(r#"
def main():
    arr: byte[10]
    x: word = len(arr)
"#);
    assert!(result.is_ok(), "Should still compile len() on array");
}

#[test]
fn test_codegen_mixed_len_usage() {
    let result = compile_source(r#"
def main():
    name: string = "HELLO"
    arr: byte[5]
    str_len: byte = len(name)
    arr_len: word = len(arr)
    println(str_len)
    println(arr_len)
"#);
    assert!(result.is_ok(), "Should compile mixed len() usage on strings and arrays");
}

// ============================================================================
// Semantic Analyzer Tests - str_at()
// ============================================================================

#[test]
fn test_str_at_string_literal() {
    let result = analyze_source(r#"
def main():
    x: byte = str_at("HELLO", 0)
"#);
    assert!(result.is_ok(), "str_at() on string literal should work");
}

#[test]
fn test_str_at_string_variable() {
    let result = analyze_source(r#"
def main():
    name: string = "WORLD"
    x: byte = str_at(name, 2)
"#);
    assert!(result.is_ok(), "str_at() on string variable should work");
}

#[test]
fn test_str_at_variable_index() {
    let result = analyze_source(r#"
def main():
    name: string = "TEST"
    i: byte = 1
    x: byte = str_at(name, i)
"#);
    assert!(result.is_ok(), "str_at() with variable index should work");
}

#[test]
fn test_str_at_in_expression() {
    let result = analyze_source(r#"
def main():
    name: string = "ABC"
    x: byte = str_at(name, 0) + 1
"#);
    assert!(result.is_ok(), "str_at() should work in expressions");
}

// ============================================================================
// Semantic Analyzer Tests - str_at() Error Cases
// ============================================================================

#[test]
fn test_str_at_wrong_first_arg_type() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = str_at(42, 0)
"#);
    assert!(!errors.is_empty(), "str_at() with non-string first arg should fail");
}

#[test]
fn test_str_at_wrong_second_arg_type() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = str_at("HELLO", "A")
"#);
    assert!(!errors.is_empty(), "str_at() with non-byte second arg should fail");
}

#[test]
fn test_str_at_no_args_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = str_at()
"#);
    assert!(!errors.is_empty(), "str_at() with no arguments should fail");
}

#[test]
fn test_str_at_one_arg_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: byte = str_at("HELLO")
"#);
    assert!(!errors.is_empty(), "str_at() with one argument should fail");
}

// ============================================================================
// Code Generation Tests - str_at()
// ============================================================================

#[test]
fn test_codegen_str_at_literal() {
    let result = compile_source(r#"
def main():
    x: byte = str_at("HELLO", 0)
"#);
    assert!(result.is_ok(), "Should compile str_at() on literal");
}

#[test]
fn test_codegen_str_at_variable() {
    let result = compile_source(r#"
def main():
    name: string = "WORLD"
    x: byte = str_at(name, 2)
"#);
    assert!(result.is_ok(), "Should compile str_at() on variable");
}

#[test]
fn test_codegen_str_at_print() {
    let result = compile_source(r#"
def main():
    name: string = "ABC"
    println(str_at(name, 1))
"#);
    assert!(result.is_ok(), "Should compile println(str_at())");
}

#[test]
fn test_codegen_str_at_loop() {
    let result = compile_source(r#"
def main():
    name: string = "TEST"
    i: byte = 0
    while i < 4:
        println(str_at(name, i))
        i += 1
"#);
    assert!(result.is_ok(), "Should compile str_at() in loop");
}

// ============================================================================
// Semantic Analyzer Tests - String Concatenation (+)
// ============================================================================

#[test]
fn test_string_concat_literals() {
    let result = analyze_source(r#"
def main():
    x: string = "HELLO" + "WORLD"
"#);
    assert!(result.is_ok(), "String concatenation of literals should work");
}

#[test]
fn test_string_concat_variables() {
    let result = analyze_source(r#"
def main():
    a: string = "HELLO"
    b: string = "WORLD"
    c: string = a + b
"#);
    assert!(result.is_ok(), "String concatenation of variables should work");
}

#[test]
fn test_string_concat_mixed() {
    let result = analyze_source(r#"
def main():
    name: string = "WORLD"
    greeting: string = "HELLO " + name
"#);
    assert!(result.is_ok(), "String concatenation of literal and variable should work");
}

#[test]
fn test_string_concat_multiple() {
    let result = analyze_source(r#"
def main():
    x: string = "A" + "B" + "C"
"#);
    assert!(result.is_ok(), "Multiple string concatenation should work");
}

// ============================================================================
// Semantic Analyzer Tests - String Concatenation Error Cases
// ============================================================================

#[test]
fn test_string_concat_with_int_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: string = "HELLO" + 42
"#);
    assert!(!errors.is_empty(), "String + int should fail");
}

#[test]
fn test_string_concat_with_byte_fails() {
    let errors = analyze_source_errors(r#"
def main():
    n: byte = 5
    x: string = "HELLO" + n
"#);
    assert!(!errors.is_empty(), "String + byte variable should fail");
}

#[test]
fn test_int_concat_with_string_fails() {
    let errors = analyze_source_errors(r#"
def main():
    x: string = 42 + "HELLO"
"#);
    assert!(!errors.is_empty(), "Int + string should fail");
}

// ============================================================================
// Code Generation Tests - String Concatenation
// ============================================================================

#[test]
fn test_codegen_string_concat_literals() {
    let result = compile_source(r#"
def main():
    x: string = "HELLO" + "WORLD"
"#);
    assert!(result.is_ok(), "Should compile string concatenation of literals");
}

#[test]
fn test_codegen_string_concat_variables() {
    let result = compile_source(r#"
def main():
    a: string = "HELLO"
    b: string = "WORLD"
    c: string = a + b
"#);
    assert!(result.is_ok(), "Should compile string concatenation of variables");
}

#[test]
fn test_codegen_string_concat_print() {
    let result = compile_source(r#"
def main():
    name: string = "WORLD"
    println("HELLO " + name)
"#);
    assert!(result.is_ok(), "Should compile println with string concatenation");
}

#[test]
fn test_codegen_string_concat_multiple() {
    let result = compile_source(r#"
def main():
    x: string = "A" + "B" + "C"
    println(x)
"#);
    assert!(result.is_ok(), "Should compile multiple string concatenation");
}

#[test]
fn test_codegen_string_concat_with_len() {
    let result = compile_source(r#"
def main():
    a: string = "HELLO"
    b: string = "WORLD"
    c: string = a + b
    println(len(c))
"#);
    assert!(result.is_ok(), "Should compile len() on concatenated string");
}

#[test]
fn test_codegen_string_concat_empty() {
    let result = compile_source(r#"
def main():
    a: string = "" + "HELLO"
    b: string = "WORLD" + ""
    println(a)
    println(b)
"#);
    assert!(result.is_ok(), "Should compile concatenation with empty strings");
}
