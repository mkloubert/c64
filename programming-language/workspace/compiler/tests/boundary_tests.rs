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

//! Boundary and edge case tests for the Cobra64 compiler.
//!
//! These tests verify correct handling of boundary values, edge cases,
//! and extreme inputs.

use cobra64::{lexer, ErrorCode};

// ============================================================================
// Numeric Boundary Tests
// ============================================================================

/// Test byte boundary values (0-255).
#[test]
fn test_byte_min_value() {
    let source = "def main():\n    x: byte = 0\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "byte value 0 should be valid");
}

#[test]
fn test_byte_max_value() {
    let source = "def main():\n    x: byte = 255\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "byte value 255 should be valid");
}

#[test]
fn test_byte_overflow_literal() {
    // 256 is too large for a byte literal, should be lexer error
    let source = "def main():\n    x: byte = 256\n";
    let tokens = lexer::tokenize(source);
    // The lexer should accept 256 as a word-sized integer
    // but semantic analysis should catch the type mismatch
    if tokens.is_ok() {
        let result = cobra64::compile(source);
        assert!(result.is_err(), "byte value 256 should cause error");
    }
}

#[test]
fn test_byte_hex_boundaries() {
    let source = "def main():\n    a: byte = $00\n    b: byte = $FF\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "hex byte values $00 and $FF should be valid"
    );
}

#[test]
fn test_byte_binary_boundaries() {
    let source = "def main():\n    a: byte = %00000000\n    b: byte = %11111111\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "binary byte values should be valid");
}

/// Test word boundary values (0-65535).
#[test]
fn test_word_min_value() {
    let source = "def main():\n    x: word = 0\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "word value 0 should be valid");
}

#[test]
fn test_word_max_value() {
    let source = "def main():\n    x: word = 65535\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "word value 65535 should be valid");
}

#[test]
fn test_word_overflow_literal() {
    // 65536 is too large for word, lexer should catch this
    let source = "def main():\n    x: word = 65536\n";
    let tokens = lexer::tokenize(source);
    assert!(tokens.is_err(), "65536 should cause lexer overflow error");
    if let Err(e) = tokens {
        assert_eq!(e.code, ErrorCode::IntegerTooLargeForWord);
    }
}

#[test]
fn test_word_hex_boundaries() {
    let source = "def main():\n    a: word = $0000\n    b: word = $FFFF\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "hex word values $0000 and $FFFF should be valid"
    );
}

#[test]
fn test_word_common_values() {
    let source = "def main():\n    a: word = 1000\n    b: word = 49152\n    c: word = 53248\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "common C64 addresses should compile");
}

/// Test arithmetic near boundaries.
#[test]
fn test_byte_arithmetic_near_max() {
    let source = "def main():\n    x: byte = 250\n    y: byte = x + 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "arithmetic near byte max should compile");
}

#[test]
fn test_word_arithmetic_near_max() {
    let source = "def main():\n    x: word = 65530\n    y: word = x + 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "arithmetic near word max should compile");
}

// ============================================================================
// String Boundary Tests
// ============================================================================

#[test]
fn test_empty_string() {
    let source = "def main():\n    println(\"\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "empty string should be valid");
}

#[test]
fn test_single_char_string() {
    let source = "def main():\n    println(\"A\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "single char string should be valid");
}

#[test]
fn test_string_with_spaces() {
    let source = "def main():\n    println(\"HELLO WORLD\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "string with spaces should be valid");
}

#[test]
fn test_string_all_escape_sequences() {
    let source = "def main():\n    println(\"A\\nB\\tC\\rD\\0E\\\\F\\\"G\")\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "string with all escape sequences should be valid"
    );
}

#[test]
fn test_string_newline_escape() {
    let source = "def main():\n    println(\"LINE1\\nLINE2\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "string with newline escape should be valid");
}

#[test]
fn test_string_tab_escape() {
    let source = "def main():\n    println(\"COL1\\tCOL2\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "string with tab escape should be valid");
}

#[test]
fn test_string_quote_escape() {
    let source = "def main():\n    println(\"HE SAID \\\"HI\\\"\")\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "string with escaped quotes should be valid");
}

#[test]
fn test_string_backslash_escape() {
    let source = "def main():\n    println(\"PATH\\\\FILE\")\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "string with escaped backslash should be valid"
    );
}

#[test]
fn test_long_string() {
    // Create a string near the typical maximum length
    let long_str = "A".repeat(200);
    let source = format!("def main():\n    println(\"{}\")\n", long_str);
    let result = cobra64::compile(&source);
    assert!(result.is_ok(), "long string (200 chars) should be valid");
}

// ============================================================================
// Char Literal Boundary Tests
// ============================================================================

#[test]
fn test_char_literal_letter() {
    let source = "def main():\n    c: byte = 'A'\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "char literal 'A' should be valid");
}

#[test]
fn test_char_literal_digit() {
    let source = "def main():\n    c: byte = '0'\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "char literal '0' should be valid");
}

#[test]
fn test_char_literal_space() {
    let source = "def main():\n    c: byte = ' '\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "char literal space should be valid");
}

#[test]
fn test_char_literal_escaped_newline() {
    let source = "def main():\n    c: byte = '\\n'\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "char literal escaped newline should be valid"
    );
}

#[test]
fn test_char_literal_escaped_tab() {
    let source = "def main():\n    c: byte = '\\t'\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "char literal escaped tab should be valid");
}

#[test]
fn test_char_literal_escaped_backslash() {
    let source = "def main():\n    c: byte = '\\\\'\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "char literal escaped backslash should be valid"
    );
}

#[test]
fn test_char_literal_escaped_quote() {
    let source = "def main():\n    c: byte = '\\''\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "char literal escaped single quote should be valid"
    );
}

// ============================================================================
// Nesting Depth Tests
// ============================================================================

#[test]
fn test_nested_if_5_levels() {
    let source = r#"def main():
    x: byte = 5
    if x > 0:
        if x > 1:
            if x > 2:
                if x > 3:
                    if x > 4:
                        println("DEEP")
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "5 levels of nested if should compile");
}

#[test]
fn test_nested_if_4_levels() {
    // Note: Deep nesting exceeds 6510's branch range limit (~127 bytes)
    // Testing 4 levels which stays within the limit
    let mut source = String::from("def main():\n    x: byte = 10\n");
    let mut indent = String::from("    ");

    for i in 0..4 {
        source.push_str(&format!("{}if x > {}:\n", indent, i));
        indent.push_str("    ");
    }
    source.push_str(&format!("{}pass\n", indent));

    let result = cobra64::compile(&source);
    assert!(
        result.is_ok(),
        "4 levels of nested if should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_if_branch_limit_error() {
    // Verify that deeply nested code properly reports branch limit error
    let mut source = String::from("def main():\n    x: byte = 10\n");
    let mut indent = String::from("    ");

    for i in 0..15 {
        source.push_str(&format!("{}if x > {}:\n", indent, i));
        indent.push_str("    ");
    }
    source.push_str(&format!("{}println(\"TOO DEEP\")\n", indent));

    let result = cobra64::compile(&source);
    assert!(result.is_err(), "Very deep nesting should hit branch limit");
    if let Err(e) = result {
        assert!(
            e.message.contains("Branch target too far"),
            "Should report branch limit error"
        );
    }
}

#[test]
fn test_nested_while_3_levels() {
    // Note: Deep while nesting generates more code per level
    // Testing 3 levels which stays within branch limits
    let source = r#"def main():
    a: byte = 1
    b: byte = 1
    c: byte = 1
    while a > 0:
        while b > 0:
            while c > 0:
                c = 0
            b = 0
        a = 0
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "3 levels of nested while should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_expressions_deep() {
    let source = "def main():\n    x: byte = ((((((1 + 2) + 3) + 4) + 5) + 6) + 7)\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "deeply nested expressions should compile");
}

#[test]
fn test_complex_expression_many_operators() {
    let source = "def main():\n    x: byte = 1 + 2 * 3 - 4 / 2 + 5 * 6 - 7\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "complex expression with many operators should compile"
    );
}

#[test]
fn test_mixed_nesting_if_while() {
    // Simplified version to stay within branch limits
    let source = r#"def main():
    i: byte = 3
    while i > 0:
        if i == 2:
            j: byte = 2
            while j > 0:
                j = j - 1
        i = i - 1
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "mixed if/while nesting should compile: {:?}",
        result.err()
    );
}

// ============================================================================
// Identifier Boundary Tests
// ============================================================================

#[test]
fn test_single_char_identifier() {
    let source = "def main():\n    x: byte = 1\n    y: byte = x\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "single char identifiers should be valid");
}

#[test]
fn test_underscore_identifier() {
    let source = "def main():\n    _x: byte = 1\n    _: byte = 2\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "underscore identifiers should be valid");
}

#[test]
fn test_identifier_with_numbers() {
    let source = "def main():\n    x1: byte = 1\n    var2: byte = 2\n    test123: byte = 3\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "identifiers with numbers should be valid");
}

#[test]
fn test_long_identifier() {
    let source = "def main():\n    this_is_a_very_long_variable_name: byte = 42\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "long identifiers should be valid");
}

#[test]
fn test_uppercase_constant() {
    let source = "const MAX_VALUE = 255\n\ndef main():\n    x: byte = MAX_VALUE\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "uppercase constants should be valid");
}

#[test]
fn test_mixed_case_identifier() {
    let source = "def main():\n    myVariable: byte = 1\n    MyVariable: byte = 2\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "mixed case identifiers should be valid (case sensitive)"
    );
}

#[test]
fn test_identifier_similar_to_keyword() {
    let source =
        "def main():\n    definition: byte = 1\n    iffy: byte = 2\n    whiles: byte = 3\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "identifiers similar to keywords should be valid"
    );
}

// ============================================================================
// Empty/Minimal Program Tests
// ============================================================================

#[test]
fn test_minimal_program() {
    let source = "def main():\n    pass\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "minimal program should compile");
}

#[test]
fn test_program_only_comments() {
    let source = "# This is a comment\n# Another comment\ndef main():\n    pass\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "program with only comments before main should compile"
    );
}

#[test]
fn test_empty_function_body() {
    let source = "def foo():\n    pass\n\ndef main():\n    foo()\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "empty function body with pass should compile"
    );
}

#[test]
fn test_function_only_return() {
    let source = "def foo():\n    return\n\ndef main():\n    foo()\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function with only return should compile");
}

#[test]
fn test_multiple_pass_statements() {
    let source = "def main():\n    pass\n    pass\n    pass\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "multiple pass statements should compile");
}

// ============================================================================
// Boolean Boundary Tests
// ============================================================================

#[test]
fn test_boolean_true() {
    let source = "def main():\n    x: bool = true\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "boolean true should be valid");
}

#[test]
fn test_boolean_false() {
    let source = "def main():\n    x: bool = false\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "boolean false should be valid");
}

#[test]
fn test_boolean_not_true() {
    let source = "def main():\n    x: bool = not true\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "not true should be valid");
}

#[test]
fn test_boolean_not_false() {
    let source = "def main():\n    x: bool = not false\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "not false should be valid");
}

#[test]
fn test_boolean_and_chain() {
    let source = "def main():\n    x: bool = true and true and true and false\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "chained and operations should be valid");
}

#[test]
fn test_boolean_or_chain() {
    let source = "def main():\n    x: bool = false or false or false or true\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "chained or operations should be valid");
}

#[test]
fn test_boolean_complex_expression() {
    let source = "def main():\n    x: bool = (true and false) or (not true and not false)\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "complex boolean expression should be valid");
}

// ============================================================================
// Comparison Boundary Tests
// ============================================================================

#[test]
fn test_comparison_equal() {
    let source = "def main():\n    x: bool = 5 == 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "equality comparison should be valid");
}

#[test]
fn test_comparison_not_equal() {
    let source = "def main():\n    x: bool = 5 != 3\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "inequality comparison should be valid");
}

#[test]
fn test_comparison_less_than() {
    let source = "def main():\n    x: bool = 3 < 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "less than comparison should be valid");
}

#[test]
fn test_comparison_greater_than() {
    let source = "def main():\n    x: bool = 5 > 3\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "greater than comparison should be valid");
}

#[test]
fn test_comparison_less_equal() {
    let source = "def main():\n    x: bool = 5 <= 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "less equal comparison should be valid");
}

#[test]
fn test_comparison_greater_equal() {
    let source = "def main():\n    x: bool = 5 >= 5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "greater equal comparison should be valid");
}

#[test]
fn test_comparison_byte_boundaries() {
    let source = "def main():\n    a: bool = 0 < 255\n    b: bool = 255 > 0\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "comparisons at byte boundaries should be valid"
    );
}

// ============================================================================
// Function Boundary Tests
// ============================================================================

#[test]
fn test_function_no_params() {
    let source = "def greet():\n    println(\"HI\")\n\ndef main():\n    greet()\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function with no params should be valid");
}

#[test]
fn test_function_one_param() {
    let source = "def greet(x: byte):\n    println(x)\n\ndef main():\n    greet(42)\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function with one param should be valid");
}

#[test]
fn test_function_multiple_params() {
    let source = "def add(a: byte, b: byte, c: byte) -> byte:\n    return a + b + c\n\ndef main():\n    x: byte = add(1, 2, 3)\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "function with multiple params should be valid"
    );
}

#[test]
fn test_function_return_byte() {
    let source =
        "def get_value() -> byte:\n    return 42\n\ndef main():\n    x: byte = get_value()\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function returning byte should be valid");
}

#[test]
fn test_function_return_word() {
    let source =
        "def get_value() -> word:\n    return 1000\n\ndef main():\n    x: word = get_value()\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function returning word should be valid");
}

#[test]
fn test_function_return_bool() {
    let source =
        "def is_valid() -> bool:\n    return true\n\ndef main():\n    x: bool = is_valid()\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function returning bool should be valid");
}

#[test]
fn test_function_calling_function() {
    let source = r#"def inner() -> byte:
    return 1

def outer() -> byte:
    return inner() + 1

def main():
    x: byte = outer()
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "function calling function should be valid");
}

#[test]
fn test_recursive_function() {
    let source = r#"def countdown(n: byte):
    if n > 0:
        println(n)
        countdown(n - 1)

def main():
    countdown(5)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "recursive function should be valid");
}

// ============================================================================
// Whitespace and Formatting Tests
// ============================================================================

#[test]
fn test_trailing_whitespace() {
    let source = "def main():    \n    pass    \n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "trailing whitespace should be ignored");
}

#[test]
fn test_blank_lines_in_function() {
    let source = "def main():\n    x: byte = 1\n\n    y: byte = 2\n\n    z: byte = 3\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "blank lines in function should be valid");
}

#[test]
fn test_multiple_statements_proper_indent() {
    let source = "def main():\n    x: byte = 1\n    y: byte = 2\n    z: byte = 3\n    println(x)\n    println(y)\n    println(z)\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "multiple statements should compile");
}

// ============================================================================
// Operator Precedence Edge Cases
// ============================================================================

#[test]
fn test_precedence_multiply_before_add() {
    // 2 + 3 * 4 should be 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    let source = "def main():\n    x: byte = 2 + 3 * 4\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "multiply before add precedence should compile"
    );
}

#[test]
fn test_precedence_comparison_before_logical() {
    // x > 5 and y < 10 should be (x > 5) and (y < 10)
    let source = "def main():\n    x: byte = 6\n    y: byte = 8\n    z: bool = x > 5 and y < 10\n";
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "comparison before logical precedence should compile"
    );
}

#[test]
fn test_precedence_parentheses_override() {
    let source = "def main():\n    x: byte = (2 + 3) * 4\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "parentheses should override precedence");
}

#[test]
fn test_unary_minus() {
    let source = "def main():\n    x: sbyte = -5\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "unary minus should be valid");
}

#[test]
fn test_unary_not() {
    let source = "def main():\n    x: bool = not true\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "unary not should be valid");
}

#[test]
fn test_double_negation() {
    let source = "def main():\n    x: bool = not not true\n";
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "double negation should be valid");
}
