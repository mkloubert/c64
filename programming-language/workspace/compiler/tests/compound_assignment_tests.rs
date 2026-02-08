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

//! Comprehensive tests for compound assignment operators.
//!
//! Tests cover:
//! - Parser: All 10 compound operators are correctly parsed
//! - Semantic Analyzer: Type checking for compound assignments
//! - Code Generator: Variable and array element compound assignments

use cobra64::ast::{AssignOp, AssignTarget, StatementKind};
use cobra64::error::ErrorCode;
use cobra64::{analyzer, codegen, lexer, parser};

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_and_get_assign_op(source: &str) -> AssignOp {
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(&tokens).unwrap();
    let main = program.main_function().unwrap();

    // Find the first assignment statement (skip var declarations)
    for stmt in &main.body.statements {
        if let StatementKind::Assignment(assign) = &stmt.kind {
            return assign.op.clone();
        }
    }
    panic!("Expected assignment statement");
}

#[allow(dead_code)]
fn parse_and_get_assign_target(source: &str) -> AssignTarget {
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(&tokens).unwrap();
    let main = program.main_function().unwrap();

    for stmt in &main.body.statements {
        if let StatementKind::Assignment(assign) = &stmt.kind {
            return assign.target.clone();
        }
    }
    panic!("Expected assignment statement");
}

fn analyze_source(source: &str) -> Result<(), cobra64::error::CompileError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;
    analyzer::analyze(&program).map_err(|errors| errors.into_iter().next().unwrap())?;
    Ok(())
}

fn compile_source(source: &str) -> Result<Vec<u8>, cobra64::error::CompileError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;
    analyzer::analyze(&program).map_err(|errors| errors.into_iter().next().unwrap())?;
    codegen::generate(&program)
}

// ============================================================================
// Parser Tests - All 10 Compound Operators
// ============================================================================

#[test]
fn test_parse_add_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 0\n    x += 5");
    assert_eq!(op, AssignOp::AddAssign);
}

#[test]
fn test_parse_sub_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 10\n    x -= 5");
    assert_eq!(op, AssignOp::SubAssign);
}

#[test]
fn test_parse_mul_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 2\n    x *= 3");
    assert_eq!(op, AssignOp::MulAssign);
}

#[test]
fn test_parse_div_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 10\n    x /= 2");
    assert_eq!(op, AssignOp::DivAssign);
}

#[test]
fn test_parse_mod_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 10\n    x %= 3");
    assert_eq!(op, AssignOp::ModAssign);
}

#[test]
fn test_parse_bitand_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 255\n    x &= 15");
    assert_eq!(op, AssignOp::BitAndAssign);
}

#[test]
fn test_parse_bitor_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 240\n    x |= 15");
    assert_eq!(op, AssignOp::BitOrAssign);
}

#[test]
fn test_parse_bitxor_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 255\n    x ^= 15");
    assert_eq!(op, AssignOp::BitXorAssign);
}

#[test]
fn test_parse_shift_left_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 1\n    x <<= 4");
    assert_eq!(op, AssignOp::ShiftLeftAssign);
}

#[test]
fn test_parse_shift_right_assign() {
    let op = parse_and_get_assign_op("def main():\n    x: byte = 128\n    x >>= 4");
    assert_eq!(op, AssignOp::ShiftRightAssign);
}

// ============================================================================
// Parser Tests - Array Element Targets
// ============================================================================

#[test]
fn test_parse_array_add_assign() {
    let source = "def main():\n    arr: byte[5]\n    arr[0] += 1";
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(&tokens).unwrap();
    let main = program.main_function().unwrap();

    // Statement 0 is var decl, statement 1 is assignment
    if let StatementKind::Assignment(assign) = &main.body.statements[1].kind {
        assert_eq!(assign.op, AssignOp::AddAssign);
        match &assign.target {
            AssignTarget::ArrayElement { name, .. } => {
                assert_eq!(name, "arr");
            }
            _ => panic!("Expected array element target"),
        }
    } else {
        panic!("Expected assignment statement");
    }
}

#[test]
fn test_parse_array_all_compound_operators() {
    let operators = [
        ("+=", AssignOp::AddAssign),
        ("-=", AssignOp::SubAssign),
        ("*=", AssignOp::MulAssign),
        ("/=", AssignOp::DivAssign),
        ("%=", AssignOp::ModAssign),
        ("&=", AssignOp::BitAndAssign),
        ("|=", AssignOp::BitOrAssign),
        ("^=", AssignOp::BitXorAssign),
        ("<<=", AssignOp::ShiftLeftAssign),
        (">>=", AssignOp::ShiftRightAssign),
    ];

    for (op_str, expected_op) in operators {
        let source = format!("def main():\n    arr: byte[5]\n    arr[2] {} 1", op_str);
        let tokens = lexer::tokenize(&source).unwrap();
        let program = parser::parse(&tokens).unwrap();
        let main = program.main_function().unwrap();

        if let StatementKind::Assignment(assign) = &main.body.statements[1].kind {
            assert_eq!(
                assign.op, expected_op,
                "Operator {} should parse to {:?}",
                op_str, expected_op
            );
        } else {
            panic!("Expected assignment for operator {}", op_str);
        }
    }
}

// ============================================================================
// Semantic Analyzer Tests - Type Checking
// ============================================================================

#[test]
fn test_analyze_byte_add_assign() {
    let source = "def main():\n    x: byte = 10\n    x += 5";
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_word_add_assign() {
    let source = "def main():\n    x: word = 1000\n    x += 500";
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_sbyte_sub_assign() {
    let source = "def main():\n    x: sbyte = 50\n    x -= 100";
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_sword_mul_assign() {
    let source = "def main():\n    x: sword = 100\n    x *= 2";
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_byte_bitwise_assign() {
    let source = r#"
def main():
    x: byte = 255
    x &= 15
    x |= 240
    x ^= 128
"#;
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_byte_shift_assign() {
    let source = r#"
def main():
    x: byte = 1
    x <<= 4
    x >>= 2
"#;
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_array_compound_assign() {
    let source = r#"
def main():
    arr: byte[10]
    arr[0] += 5
    arr[1] -= 3
    arr[2] *= 2
"#;
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_word_array_compound_assign() {
    let source = r#"
def main():
    arr: word[5]
    arr[0] += 1000
    arr[1] -= 500
    arr[2] *= 2
"#;
    assert!(analyze_source(source).is_ok());
}

#[test]
fn test_analyze_string_compound_assign_valid() {
    // String += string now works (concatenation)
    let source = r#"
def main():
    s: string = "hello"
    s += " world"
"#;
    let result = analyze_source(source);
    assert!(result.is_ok(), "String += string should work");
}

#[test]
fn test_analyze_error_string_compound_assign_with_int() {
    // String += int should still fail
    let source = r#"
def main():
    s: string = "hello"
    s += 5
"#;
    let result = analyze_source(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidOperatorForType);
}

#[test]
fn test_analyze_error_bool_compound_assign() {
    let source = r#"
def main():
    b: bool = true
    b += true
"#;
    let result = analyze_source(source);
    assert!(result.is_err());
}

#[test]
fn test_analyze_error_assign_to_const() {
    let source = r#"
const MAX: byte = 100
def main():
    MAX += 1
"#;
    let result = analyze_source(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::CannotAssignToConstant);
}

// ============================================================================
// Code Generation Tests - Variable Compound Assignments
// ============================================================================

#[test]
fn test_codegen_byte_add_assign() {
    let source = r#"
def main():
    x: byte = 10
    x += 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_sub_assign() {
    let source = r#"
def main():
    x: byte = 20
    x -= 8
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_mul_assign() {
    let source = r#"
def main():
    x: byte = 5
    x *= 3
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_div_assign() {
    let source = r#"
def main():
    x: byte = 100
    x /= 4
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_mod_assign() {
    let source = r#"
def main():
    x: byte = 17
    x %= 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_bitand_assign() {
    let source = r#"
def main():
    x: byte = 255
    x &= 15
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_bitor_assign() {
    let source = r#"
def main():
    x: byte = 240
    x |= 15
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_bitxor_assign() {
    let source = r#"
def main():
    x: byte = 170
    x ^= 85
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_shift_left_assign() {
    let source = r#"
def main():
    x: byte = 1
    x <<= 4
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_byte_shift_right_assign() {
    let source = r#"
def main():
    x: byte = 128
    x >>= 3
"#;
    assert!(compile_source(source).is_ok());
}

// ============================================================================
// Code Generation Tests - Array Element Compound Assignments
// ============================================================================

#[test]
fn test_codegen_array_add_assign() {
    let source = r#"
def main():
    arr: byte[5]
    arr[0] = 10
    arr[0] += 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_sub_assign() {
    let source = r#"
def main():
    arr: byte[5]
    arr[1] = 20
    arr[1] -= 8
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_mul_assign() {
    let source = r#"
def main():
    arr: byte[5]
    arr[2] = 5
    arr[2] *= 3
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_div_assign() {
    let source = r#"
def main():
    arr: byte[5]
    arr[3] = 100
    arr[3] /= 4
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_mod_assign() {
    let source = r#"
def main():
    arr: byte[5]
    arr[4] = 17
    arr[4] %= 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_bitand_assign() {
    let source = r#"
def main():
    arr: byte[3]
    arr[0] = 255
    arr[0] &= 15
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_bitor_assign() {
    let source = r#"
def main():
    arr: byte[3]
    arr[1] = 240
    arr[1] |= 15
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_bitxor_assign() {
    let source = r#"
def main():
    arr: byte[3]
    arr[2] = 170
    arr[2] ^= 85
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_shift_left_assign() {
    let source = r#"
def main():
    arr: byte[2]
    arr[0] = 1
    arr[0] <<= 4
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_array_shift_right_assign() {
    let source = r#"
def main():
    arr: byte[2]
    arr[1] = 128
    arr[1] >>= 3
"#;
    assert!(compile_source(source).is_ok());
}

// ============================================================================
// Code Generation Tests - Word Array Compound Assignments
// ============================================================================

#[test]
fn test_codegen_word_array_add_assign() {
    let source = r#"
def main():
    arr: word[3]
    arr[0] = 1000
    arr[0] += 500
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_word_array_sub_assign() {
    let source = r#"
def main():
    arr: word[3]
    arr[1] = 2000
    arr[1] -= 800
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_word_array_bitwise_assign() {
    let source = r#"
def main():
    arr: word[3]
    arr[0] = 65535
    arr[0] &= 255
    arr[1] = 256
    arr[1] |= 255
    arr[2] = 43690
    arr[2] ^= 21845
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_codegen_word_array_shift_assign() {
    let source = r#"
def main():
    arr: word[2]
    arr[0] = 1
    arr[0] <<= 8
    arr[1] = 32768
    arr[1] >>= 4
"#;
    assert!(compile_source(source).is_ok());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_compound_assign_with_expression_rhs() {
    let source = r#"
def main():
    x: byte = 10
    y: byte = 3
    x += y * 2
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_in_loop() {
    let source = r#"
def main():
    sum: byte = 0
    i: byte = 0
    while i < 10:
        sum += i
        i += 1
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_with_variable_index() {
    let source = r#"
def main():
    arr: byte[10]
    i: byte = 5
    arr[i] += 1
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_multiple_operations() {
    let source = r#"
def main():
    x: byte = 100
    x += 10
    x -= 5
    x *= 2
    x /= 3
    x %= 7
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_array_with_expression_index() {
    let source = r#"
def main():
    arr: byte[20]
    i: byte = 2
    arr[i + 3] += 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_sbyte_array() {
    let source = r#"
def main():
    arr: sbyte[5]
    arr[0] = 50
    arr[0] -= 100
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_sword_array() {
    let source = r#"
def main():
    arr: sword[5]
    arr[0] = 1000
    arr[0] -= 2000
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_nested_in_if() {
    let source = r#"
def main():
    x: byte = 10
    if x > 5:
        x += 5
    else:
        x -= 5
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_all_operators_in_sequence() {
    let source = r#"
def main():
    x: byte = 128
    x += 1
    x -= 1
    x *= 1
    x /= 1
    x %= 200
    x &= 255
    x |= 0
    x ^= 0
    x <<= 0
    x >>= 0
"#;
    assert!(compile_source(source).is_ok());
}

#[test]
fn test_compound_assign_array_all_operators() {
    let source = r#"
def main():
    arr: byte[10]
    arr[0] = 128
    arr[0] += 1
    arr[1] = 100
    arr[1] -= 1
    arr[2] = 10
    arr[2] *= 2
    arr[3] = 100
    arr[3] /= 5
    arr[4] = 17
    arr[4] %= 5
    arr[5] = 255
    arr[5] &= 15
    arr[6] = 240
    arr[6] |= 15
    arr[7] = 170
    arr[7] ^= 85
    arr[8] = 1
    arr[8] <<= 4
    arr[9] = 128
    arr[9] >>= 3
"#;
    assert!(compile_source(source).is_ok());
}
