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

//! Integration tests for signed integer types (sbyte and sword).
//!
//! These tests verify that the compiler correctly handles:
//! - Signed variable declarations
//! - Signed arithmetic operations
//! - Signed comparisons
//! - Mixed signed/unsigned operations
//! - Edge case values

// ============================================================================
// Signed Type Declaration Tests
// ============================================================================

#[test]
fn test_signed_types_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_types.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_types.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_declaration_negative() {
    let source = r#"
def main():
    x: sbyte = -100
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte with negative value should compile");
}

#[test]
fn test_sbyte_declaration_min_value() {
    let source = r#"
def main():
    x: sbyte = -128
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte with -128 (min value) should compile");
}

#[test]
fn test_sbyte_declaration_max_value() {
    let source = r#"
def main():
    x: sbyte = 127
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte with 127 (max value) should compile");
}

#[test]
fn test_sword_declaration_negative() {
    let source = r#"
def main():
    y: sword = -30000
    println(y)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sword with negative value should compile");
}

#[test]
fn test_sword_declaration_min_value() {
    let source = r#"
def main():
    y: sword = -32768
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sword with -32768 (min value) should compile"
    );
}

#[test]
fn test_sword_declaration_max_value() {
    let source = r#"
def main():
    y: sword = 32767
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sword with 32767 (max value) should compile"
    );
}

// ============================================================================
// Signed Arithmetic Tests
// ============================================================================

#[test]
fn test_signed_arithmetic_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_arithmetic.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_arithmetic.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_addition() {
    let source = r#"
def main():
    a: sbyte = -50
    b: sbyte = 30
    c: sbyte = a + b
    println(c)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte addition should compile");
}

#[test]
fn test_sbyte_subtraction() {
    let source = r#"
def main():
    a: sbyte = -20
    b: sbyte = 30
    c: sbyte = a - b
    println(c)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte subtraction should compile");
}

#[test]
fn test_sbyte_multiplication() {
    let source = r#"
def main():
    a: sbyte = -5
    b: sbyte = 10
    c: sbyte = a * b
    println(c)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte multiplication should compile");
}

#[test]
fn test_sbyte_division() {
    let source = r#"
def main():
    a: sbyte = -50
    b: sbyte = 10
    c: sbyte = a / b
    println(c)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte division should compile");
}

#[test]
fn test_sword_arithmetic() {
    let source = r#"
def main():
    a: sword = -1000
    b: sword = 500
    sum: sword = a + b
    diff: sword = a - b
    println(sum)
    println(diff)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sword arithmetic should compile");
}

// ============================================================================
// Signed Comparison Tests
// ============================================================================

#[test]
fn test_signed_comparisons_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_comparisons.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_comparisons.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_less_than() {
    let source = r#"
def main():
    a: sbyte = -50
    b: sbyte = 50
    if a < b:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte less-than comparison should compile");
}

#[test]
fn test_sbyte_greater_than() {
    let source = r#"
def main():
    a: sbyte = 50
    b: sbyte = -50
    if a > b:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sbyte greater-than comparison should compile"
    );
}

#[test]
fn test_sbyte_less_equal() {
    let source = r#"
def main():
    a: sbyte = -50
    if a <= 0:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte less-equal comparison should compile");
}

#[test]
fn test_sbyte_greater_equal() {
    let source = r#"
def main():
    a: sbyte = 50
    if a >= 0:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sbyte greater-equal comparison should compile"
    );
}

#[test]
fn test_sbyte_equality() {
    let source = r#"
def main():
    a: sbyte = -10
    b: sbyte = -10
    if a == b:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte equality comparison should compile");
}

#[test]
fn test_sbyte_inequality() {
    let source = r#"
def main():
    a: sbyte = -10
    b: sbyte = 10
    if a != b:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte inequality comparison should compile");
}

#[test]
fn test_sword_comparisons() {
    let source = r#"
def main():
    a: sword = -30000
    b: sword = 30000
    if a < b:
        println(1)
    if b > a:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sword comparisons should compile");
}

// ============================================================================
// Signed Loop Tests
// ============================================================================

#[test]
fn test_signed_loops_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_loops.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_loops.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_while_loop_negative_to_positive() {
    let source = r#"
def main():
    i: sbyte = -5
    while i <= 5:
        println(i)
        i = i + 1
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "while loop with sbyte counter should compile"
    );
}

#[test]
fn test_sbyte_while_loop_positive_to_negative() {
    let source = r#"
def main():
    i: sbyte = 5
    while i >= -5:
        println(i)
        i = i - 1
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "while loop counting down to negative should compile"
    );
}

// ============================================================================
// Mixed Type Tests
// ============================================================================

#[test]
fn test_signed_mixed_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_mixed.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_mixed.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_byte_to_sword_promotion() {
    let source = r#"
def main():
    b: byte = 100
    s: sword = b
    println(s)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "byte to sword promotion should compile");
}

#[test]
fn test_sbyte_to_sword_promotion() {
    let source = r#"
def main():
    sb: sbyte = -100
    sw: sword = sb
    println(sw)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte to sword promotion should compile");
}

#[test]
fn test_mixed_signed_unsigned_comparison() {
    let source = r#"
def main():
    u: byte = 100
    s: sbyte = -50
    if s < u:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "mixed signed/unsigned comparison should compile"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_signed_edge_cases_fixture_compiles() {
    let source = include_str!("fixtures/valid/signed_edge_cases.cb64");
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "signed_edge_cases.cb64 should compile successfully: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_negative_zero() {
    let source = r#"
def main():
    x: sbyte = -0
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte with -0 should compile");
}

#[test]
fn test_sbyte_boundary_comparison() {
    let source = r#"
def main():
    min: sbyte = -128
    max: sbyte = 127
    if min < max:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sbyte boundary comparison should compile");
}

#[test]
fn test_sword_boundary_comparison() {
    let source = r#"
def main():
    min: sword = -32768
    max: sword = 32767
    if min < max:
        println(1)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "sword boundary comparison should compile");
}

#[test]
fn test_negative_hex_literal() {
    let source = r#"
def main():
    x: sbyte = -$7F
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "negative hex literal should compile");
}

#[test]
fn test_negative_binary_literal() {
    let source = r#"
def main():
    x: sbyte = -%01111111
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "negative binary literal should compile");
}

// ============================================================================
// Error Case Tests (should fail to compile)
// ============================================================================

#[test]
fn test_sbyte_overflow_positive() {
    let source = r#"
def main():
    x: sbyte = 128
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "sbyte with 128 should fail (overflow)");
}

#[test]
fn test_sbyte_overflow_negative() {
    let source = r#"
def main():
    x: sbyte = -129
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "sbyte with -129 should fail (underflow)");
}

#[test]
fn test_sword_overflow_positive() {
    let source = r#"
def main():
    x: sword = 32768
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "sword with 32768 should fail (overflow)");
}

#[test]
fn test_sword_overflow_negative() {
    let source = r#"
def main():
    x: sword = -32769
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "sword with -32769 should fail (underflow)");
}

#[test]
fn test_negative_value_to_byte() {
    let source = r#"
def main():
    x: byte = -1
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "negative value to byte should fail");
}

#[test]
fn test_negative_value_to_word() {
    let source = r#"
def main():
    x: word = -1
"#;
    let result = cobra64::compile(source);
    assert!(result.is_err(), "negative value to word should fail");
}

// ============================================================================
// Code Generation Size Tests
// ============================================================================

#[test]
fn test_signed_code_reasonable_size() {
    let source = r#"
def main():
    x: sbyte = -100
    y: sbyte = 50
    if x < y:
        println(x)
"#;
    let result = cobra64::compile(source);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Code should be reasonable size (runtime + actual code)
    // Note: size increased to accommodate fixed-point and float runtime routines
    assert!(
        code.len() < 2500,
        "Generated code should be under 2500 bytes"
    );
    assert!(
        code.len() > 100,
        "Generated code should be at least 100 bytes"
    );
}

// ============================================================================
// Signed Shift Right Tests
// ============================================================================

#[test]
fn test_sbyte_shift_right() {
    // Tests arithmetic shift right for signed bytes
    // -4 >> 1 should be -2 (arithmetic shift preserves sign)
    let source = r#"
def main():
    x: sbyte = -4
    y: sbyte = x >> 1
    println(y)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sbyte shift right should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_sbyte_shift_right_multiple() {
    // Tests multiple arithmetic shifts
    // -8 >> 2 should be -2 (arithmetic shift)
    let source = r#"
def main():
    x: sbyte = -8
    y: sbyte = x >> 2
    println(y)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "sbyte multiple shift right should compile"
    );
}

#[test]
fn test_byte_shift_right_unsigned() {
    // Tests logical shift right for unsigned bytes
    // 128 >> 1 should be 64 (logical shift)
    let source = r#"
def main():
    x: byte = 128
    y: byte = x >> 1
    println(y)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "byte shift right should compile"
    );
}

#[test]
fn test_sbyte_shift_right_positive() {
    // Positive signed values should shift like unsigned
    // 8 >> 1 should be 4
    let source = r#"
def main():
    x: sbyte = 8
    y: sbyte = x >> 1
    println(y)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "positive sbyte shift right should compile"
    );
}

// ============================================================================
// Fixed to Signed Conversion Tests
// ============================================================================

#[test]
fn test_fixed_to_sbyte_positive() {
    // Tests fixed-point to signed byte conversion for positive values
    let source = r#"
def main():
    f: fixed = 5.5
    x: sbyte = sbyte(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to sbyte (positive) should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_fixed_to_sbyte_negative() {
    // Tests fixed-point to signed byte conversion for negative values
    // This requires arithmetic shift to preserve the sign
    let source = r#"
def main():
    f: fixed = -5.5
    x: sbyte = sbyte(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to sbyte (negative) should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_fixed_to_sword_positive() {
    // Tests fixed-point to signed word conversion for positive values
    let source = r#"
def main():
    f: fixed = 100.25
    x: sword = sword(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to sword (positive) should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_fixed_to_sword_negative() {
    // Tests fixed-point to signed word conversion for negative values
    // This requires arithmetic shift to preserve the sign
    let source = r#"
def main():
    f: fixed = -100.25
    x: sword = sword(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to sword (negative) should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_fixed_to_byte_unsigned() {
    // Tests fixed-point to unsigned byte conversion (logical shift)
    let source = r#"
def main():
    f: fixed = 100.75
    x: byte = byte(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to byte (unsigned) should compile: {:?}",
        result.err()
    );
}

#[test]
fn test_fixed_to_word_unsigned() {
    // Tests fixed-point to unsigned word conversion (logical shift)
    let source = r#"
def main():
    f: fixed = 100.5
    x: word = word(f)
    println(x)
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "fixed to word (unsigned) should compile: {:?}",
        result.err()
    );
}
