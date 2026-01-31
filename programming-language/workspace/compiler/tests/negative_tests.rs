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

//! Negative/Error tests for the Cobra64 compiler.
//!
//! These tests verify that the compiler correctly rejects invalid programs
//! and produces appropriate error messages.

use cobra64::{lexer, parser, ErrorCode};
use test_case::test_case;

// ============================================================================
// Lexer Error Tests
// ============================================================================

/// Test that the lexer rejects invalid characters.
#[test_case("def main():\n    @\n", ErrorCode::InvalidCharacter; "at_sign")]
#[test_case("def main():\n    `\n", ErrorCode::InvalidCharacter; "backtick")]
#[test_case("def main():\n    £\n", ErrorCode::InvalidCharacter; "pound_sign")]
fn test_lexer_invalid_characters(source: &str, expected_code: ErrorCode) {
    let result = lexer::tokenize(source);
    assert!(
        result.is_err(),
        "Expected lexer error for invalid character"
    );
    let err = result.unwrap_err();
    assert_eq!(err.code, expected_code);
}

/// Test that the lexer rejects unterminated strings.
#[test_case("def main():\n    x = \"hello\n", ErrorCode::UnterminatedString; "newline_in_string")]
#[test_case("def main():\n    x = \"hello", ErrorCode::UnterminatedString; "eof_in_string")]
fn test_lexer_unterminated_strings(source: &str, expected_code: ErrorCode) {
    let result = lexer::tokenize(source);
    assert!(
        result.is_err(),
        "Expected lexer error for unterminated string"
    );
    let err = result.unwrap_err();
    assert_eq!(err.code, expected_code);
}

/// Test that the lexer rejects invalid escape sequences.
#[test_case("def main():\n    x = \"\\x\"\n", ErrorCode::InvalidEscapeSequence; "invalid_x")]
#[test_case("def main():\n    x = \"\\q\"\n", ErrorCode::InvalidEscapeSequence; "invalid_q")]
fn test_lexer_invalid_escapes(source: &str, expected_code: ErrorCode) {
    let result = lexer::tokenize(source);
    assert!(result.is_err(), "Expected lexer error for invalid escape");
    let err = result.unwrap_err();
    assert_eq!(err.code, expected_code);
}

/// Test that the lexer rejects invalid number formats.
#[test_case("def main():\n    x = $GG\n", ErrorCode::InvalidHexDigit; "invalid_hex")]
#[test_case("def main():\n    x = %123\n", ErrorCode::InvalidBinaryDigit; "invalid_binary")]
#[test_case("def main():\n    x = 99999\n", ErrorCode::IntegerTooLargeForWord; "overflow")]
fn test_lexer_invalid_numbers(source: &str, expected_code: ErrorCode) {
    let result = lexer::tokenize(source);
    assert!(result.is_err(), "Expected lexer error for invalid number");
    let err = result.unwrap_err();
    assert_eq!(err.code, expected_code);
}

/// Test that the lexer rejects tabs.
#[test]
fn test_lexer_tab_not_allowed() {
    let source = "def main():\n\tpass\n";
    let result = lexer::tokenize(source);
    assert!(result.is_err(), "Expected lexer error for tab");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::TabNotAllowed);
}

/// Test that the lexer rejects unterminated char literals.
#[test_case("def main():\n    x = '\n", ErrorCode::UnterminatedCharLiteral; "newline")]
#[test_case("def main():\n    x = 'ab'\n", ErrorCode::CharLiteralTooLong; "too_long")]
#[test_case("def main():\n    x = ''\n", ErrorCode::EmptyCharLiteral; "empty")]
fn test_lexer_invalid_char_literals(source: &str, expected_code: ErrorCode) {
    let result = lexer::tokenize(source);
    assert!(
        result.is_err(),
        "Expected lexer error for invalid char literal"
    );
    let err = result.unwrap_err();
    assert_eq!(err.code, expected_code);
}

// ============================================================================
// Parser Error Tests
// ============================================================================

/// Helper to parse source and return the error code if parsing fails.
fn parse_and_get_error(source: &str) -> Option<ErrorCode> {
    let tokens = lexer::tokenize(source).ok()?;
    match parser::parse(&tokens) {
        Ok(_) => None,
        Err(e) => Some(e.code),
    }
}

/// Test that the parser rejects missing colons.
/// Note: Parser returns UnexpectedToken for most syntax errors.
#[test_case("def main()\n    pass\n", ErrorCode::UnexpectedToken; "def_no_colon")]
#[test_case("def main():\n    if true\n        pass\n", ErrorCode::UnexpectedToken; "if_no_colon")]
#[test_case("def main():\n    while true\n        pass\n", ErrorCode::UnexpectedToken; "while_no_colon")]
fn test_parser_missing_colons(source: &str, expected_code: ErrorCode) {
    let err = parse_and_get_error(source);
    assert!(err.is_some(), "Expected parser error for missing colon");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test that the parser rejects missing parentheses.
/// Note: Parser returns UnexpectedToken for most syntax errors.
#[test_case("def main:\n    pass\n", ErrorCode::UnexpectedToken; "def_no_parens")]
#[test_case("def main(:\n    pass\n", ErrorCode::ExpectedIdentifier; "def_no_close_paren")]
fn test_parser_missing_parens(source: &str, expected_code: ErrorCode) {
    let err = parse_and_get_error(source);
    assert!(err.is_some(), "Expected parser error for missing paren");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test that the parser rejects invalid expressions.
/// Note: Parser returns UnexpectedToken for most syntax errors.
#[test_case("def main():\n    x = +\n", ErrorCode::UnexpectedToken; "lone_plus")]
#[test_case("def main():\n    x = *5\n", ErrorCode::UnexpectedToken; "lone_star")]
#[test_case("def main():\n    x = ()\n", ErrorCode::UnexpectedToken; "empty_parens")]
fn test_parser_invalid_expressions(source: &str, expected_code: ErrorCode) {
    let err = parse_and_get_error(source);
    assert!(
        err.is_some(),
        "Expected parser error for invalid expression"
    );
    assert_eq!(err.unwrap(), expected_code);
}

/// Test that the parser rejects unexpected tokens.
#[test_case("def main():\n    ] pass\n", ErrorCode::UnexpectedToken; "unexpected_bracket")]
#[test_case("def main():\n    )\n", ErrorCode::UnexpectedToken; "unexpected_close_paren")]
fn test_parser_unexpected_tokens(source: &str, expected_code: ErrorCode) {
    let err = parse_and_get_error(source);
    assert!(err.is_some(), "Expected parser error for unexpected token");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test that the parser rejects elif/else without if.
/// Note: Parser returns UnexpectedToken for elif/else without preceding if.
#[test]
fn test_parser_elif_without_if() {
    let source = "def main():\n    elif true:\n        pass\n";
    let err = parse_and_get_error(source);
    assert!(err.is_some(), "Expected parser error for elif without if");
    // Parser sees 'elif' as unexpected when not following an if statement
    assert_eq!(err.unwrap(), ErrorCode::UnexpectedToken);
}

#[test]
fn test_parser_else_without_if() {
    let source = "def main():\n    else:\n        pass\n";
    let err = parse_and_get_error(source);
    assert!(err.is_some(), "Expected parser error for else without if");
    // Parser sees 'else' as unexpected when not following an if statement
    assert_eq!(err.unwrap(), ErrorCode::UnexpectedToken);
}

// ============================================================================
// Semantic Error Tests
// ============================================================================

/// Helper to compile source and return the error code if compilation fails.
fn compile_and_get_error(source: &str) -> Option<ErrorCode> {
    match cobra64::compile(source) {
        Ok(_) => None,
        Err(e) => Some(e.code),
    }
}

/// Test undefined variable errors.
#[test_case("def main():\n    x = y\n", ErrorCode::UndefinedVariable; "simple")]
#[test_case("def main():\n    println(undefined)\n", ErrorCode::UndefinedVariable; "in_call")]
#[test_case("def main():\n    x: byte = y + 1\n", ErrorCode::UndefinedVariable; "in_expr")]
fn test_semantic_undefined_variable(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for undefined variable");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test undefined function errors.
#[test_case("def main():\n    foo()\n", ErrorCode::UndefinedFunction; "simple_call")]
#[test_case("def main():\n    x: byte = bar(1)\n", ErrorCode::UndefinedFunction; "in_assignment")]
fn test_semantic_undefined_function(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for undefined function");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test type mismatch errors.
#[test_case("def main():\n    x: byte = \"hello\"\n", ErrorCode::TypeMismatch; "string_to_byte")]
#[test_case("def main():\n    x: bool = 42\n", ErrorCode::TypeMismatch; "int_to_bool")]
#[test_case("def main():\n    x: byte = true\n", ErrorCode::TypeMismatch; "bool_to_byte")]
fn test_semantic_type_mismatch_assignment(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for type mismatch");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test type mismatch in operations.
#[test_case("def main():\n    x: byte = 1 + \"a\"\n", ErrorCode::InvalidOperatorForType; "add_string")]
#[test_case("def main():\n    x: bool = true + false\n", ErrorCode::InvalidOperatorForType; "add_bool")]
fn test_semantic_type_mismatch_operations(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(
        err.is_some(),
        "Expected error for type mismatch in operation"
    );
    assert_eq!(err.unwrap(), expected_code);
}

/// Test duplicate variable declaration errors.
#[test]
fn test_semantic_duplicate_variable() {
    let source = "def main():\n    x: byte = 1\n    x: byte = 2\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for duplicate variable");
    assert_eq!(err.unwrap(), ErrorCode::VariableAlreadyDefined);
}

/// Test duplicate function definition errors.
#[test]
fn test_semantic_duplicate_function() {
    let source = "def foo():\n    pass\n\ndef foo():\n    pass\n\ndef main():\n    pass\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for duplicate function");
    assert_eq!(err.unwrap(), ErrorCode::FunctionAlreadyDefined);
}

/// Test constant reassignment errors.
#[test]
fn test_semantic_constant_reassignment() {
    let source = "const X = 10\n\ndef main():\n    X = 20\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for constant reassignment");
    assert_eq!(err.unwrap(), ErrorCode::CannotAssignToConstant);
}

/// Test missing main function.
#[test]
fn test_semantic_missing_main() {
    let source = "def foo():\n    pass\n";
    let result = cobra64::compile(source);
    assert!(result.is_err(), "Expected error for missing main");
    // The error message should mention main
    let err = result.unwrap_err();
    assert!(err.message.to_lowercase().contains("main"));
}

/// Test break outside loop.
#[test]
fn test_semantic_break_outside_loop() {
    let source = "def main():\n    break\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for break outside loop");
    assert_eq!(err.unwrap(), ErrorCode::BreakOutsideLoop);
}

/// Test continue outside loop.
#[test]
fn test_semantic_continue_outside_loop() {
    let source = "def main():\n    continue\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for continue outside loop");
    assert_eq!(err.unwrap(), ErrorCode::ContinueOutsideLoop);
}

/// Test return type mismatch.
#[test_case("def foo() -> byte:\n    return \"hello\"\n\ndef main():\n    pass\n", ErrorCode::TypeMismatch; "string_for_byte")]
#[test_case("def foo() -> byte:\n    return true\n\ndef main():\n    pass\n", ErrorCode::TypeMismatch; "bool_for_byte")]
fn test_semantic_return_type_mismatch(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for return type mismatch");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test wrong number of function arguments.
#[test_case("def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    add(1)\n", ErrorCode::WrongNumberOfArguments; "too_few")]
#[test_case("def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    add(1, 2, 3)\n", ErrorCode::WrongNumberOfArguments; "too_many")]
#[test_case("def greet():\n    pass\n\ndef main():\n    greet(1)\n", ErrorCode::WrongNumberOfArguments; "arg_to_noarg")]
fn test_semantic_wrong_argument_count(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for wrong argument count");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test wrong argument types.
#[test]
fn test_semantic_wrong_argument_type() {
    let source = "def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    add(\"x\", \"y\")\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for wrong argument type");
    assert_eq!(err.unwrap(), ErrorCode::ArgumentTypeMismatch);
}

/// Test return from void function with value.
#[test]
fn test_semantic_return_value_from_void() {
    let source = "def foo():\n    return 42\n\ndef main():\n    pass\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for return value from void");
    assert_eq!(err.unwrap(), ErrorCode::CannotReturnValueFromVoid);
}

/// Test missing return value.
#[test]
fn test_semantic_missing_return_value() {
    let source = "def foo() -> byte:\n    return\n\ndef main():\n    pass\n";
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for missing return value");
    assert_eq!(err.unwrap(), ErrorCode::MissingReturnValue);
}

/// Test logical operators on non-boolean types.
#[test_case("def main():\n    x: bool = 1 and 2\n", ErrorCode::InvalidOperatorForType; "and_on_int")]
#[test_case("def main():\n    x: bool = 1 or 2\n", ErrorCode::InvalidOperatorForType; "or_on_int")]
#[test_case("def main():\n    x: bool = not 1\n", ErrorCode::InvalidOperatorForType; "not_on_int")]
fn test_semantic_logical_on_non_bool(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for logical op on non-bool");
    assert_eq!(err.unwrap(), expected_code);
}

/// Test condition must be boolean.
#[test_case("def main():\n    if 42:\n        pass\n", ErrorCode::TypeMismatch; "if_int")]
#[test_case("def main():\n    while 42:\n        pass\n", ErrorCode::TypeMismatch; "while_int")]
fn test_semantic_condition_must_be_bool(source: &str, expected_code: ErrorCode) {
    let err = compile_and_get_error(source);
    assert!(err.is_some(), "Expected error for non-bool condition");
    assert_eq!(err.unwrap(), expected_code);
}

// ============================================================================
// Combined Tests - Multiple Errors
// ============================================================================

/// Test that the compiler catches the first error.
#[test]
fn test_first_error_reported() {
    // This has multiple errors: undefined x, undefined y
    let source = "def main():\n    a = x\n    b = y\n";
    let result = cobra64::compile(source);
    assert!(result.is_err());
    // Should report the first error
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::UndefinedVariable);
}

// ============================================================================
// Fixture-based Tests
// ============================================================================

/// Test all invalid fixture files produce errors.
#[test]
fn test_all_invalid_fixtures_fail() {
    let invalid_dir = std::path::Path::new("tests/fixtures/invalid");

    for entry in std::fs::read_dir(invalid_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "cb64") {
            let source = std::fs::read_to_string(&path).unwrap();
            let result = cobra64::compile(&source);

            assert!(
                result.is_err(),
                "Expected error for invalid fixture: {}",
                path.display()
            );
        }
    }
}

/// Test all valid fixture files compile successfully.
#[test]
fn test_all_valid_fixtures_compile() {
    let valid_dir = std::path::Path::new("tests/fixtures/valid");

    for entry in std::fs::read_dir(valid_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "cb64") {
            let source = std::fs::read_to_string(&path).unwrap();
            let result = cobra64::compile(&source);

            assert!(
                result.is_ok(),
                "Expected success for valid fixture: {}, got error: {:?}",
                path.display(),
                result.err()
            );
        }
    }
}
