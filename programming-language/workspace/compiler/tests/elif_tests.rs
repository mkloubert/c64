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

//! Elif statement tests for the Cobra64 compiler.
//!
//! These tests verify that elif statements compile correctly.
//! They test various scenarios:
//! - Basic if-elif-else chains
//! - Multiple elif branches
//! - Elif without final else
//! - Nested structures with elif
//! - Complex conditions with and/or/not

use std::fs;

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Compile an elif test file and return success/failure.
fn compile_elif_test(filename: &str) -> Result<Vec<u8>, String> {
    let path = format!("tests/elif/{}", filename);
    let source =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    cobra64::compile(&source).map_err(|e| format!("Compile error: {:?}", e))
}

/// Check that an elif test file compiles successfully.
fn assert_compiles(filename: &str) {
    match compile_elif_test(filename) {
        Ok(code) => {
            assert!(
                !code.is_empty(),
                "{} should generate non-empty code",
                filename
            );
        }
        Err(e) => panic!("{} should compile but failed: {}", filename, e),
    }
}

/// Check that source code compiles successfully.
fn assert_source_compiles(source: &str, description: &str) {
    match cobra64::compile(source) {
        Ok(code) => {
            assert!(
                !code.is_empty(),
                "{} should generate non-empty code",
                description
            );
        }
        Err(e) => panic!("{} should compile but failed: {:?}", description, e),
    }
}

// ============================================================================
// Elif Test Cases from Test Files
// ============================================================================

/// Test basic if-elif-else chain.
#[test]
fn test_elif_basic() {
    assert_compiles("elif_basic.cb64");
}

/// Test multiple elif branches.
#[test]
fn test_elif_multiple() {
    assert_compiles("elif_multiple.cb64");
}

/// Test elif without final else block.
#[test]
fn test_elif_no_else() {
    assert_compiles("elif_no_else.cb64");
}

/// Test nested structures with elif.
#[test]
fn test_elif_nested() {
    assert_compiles("elif_nested.cb64");
}

/// Test elif with complex conditions (and/or/not).
#[test]
fn test_elif_complex_conditions() {
    assert_compiles("elif_complex_conditions.cb64");
}

// ============================================================================
// Additional Inline Tests
// ============================================================================

/// Test minimal elif case - single elif branch.
#[test]
fn test_elif_minimal() {
    let source = r#"
def main():
    x: byte = 5
    if x > 10:
        println("A")
    elif x > 3:
        println("B")
"#;
    assert_source_compiles(source, "minimal elif");
}

/// Test elif with exactly two branches.
#[test]
fn test_elif_two_branches() {
    let source = r#"
def main():
    x: byte = 50
    if x > 100:
        println("LARGE")
    elif x > 50:
        println("MEDIUM")
    elif x > 0:
        println("SMALL")
"#;
    assert_source_compiles(source, "two elif branches");
}

/// Test elif with else block.
#[test]
fn test_elif_with_else() {
    let source = r#"
def main():
    x: byte = 5
    if x > 100:
        println("A")
    elif x > 50:
        println("B")
    else:
        println("C")
"#;
    assert_source_compiles(source, "elif with else");
}

/// Test three elif branches with else.
#[test]
fn test_elif_three_branches_with_else() {
    let source = r#"
def main():
    x: byte = 25
    if x > 100:
        println("A")
    elif x > 75:
        println("B")
    elif x > 50:
        println("C")
    elif x > 25:
        println("D")
    else:
        println("E")
"#;
    assert_source_compiles(source, "three elif branches with else");
}

/// Test elif inside while loop.
#[test]
fn test_elif_in_while() {
    let source = r#"
def main():
    i: byte = 0
    while i < 10:
        if i > 7:
            println("HIGH")
        elif i > 3:
            println("MID")
        else:
            println("LOW")
        i = i + 1
"#;
    assert_source_compiles(source, "elif inside while loop");
}

/// Test elif inside for loop.
#[test]
fn test_elif_in_for() {
    let source = r#"
def main():
    for i in 0 to 5:
        if i > 3:
            println("HIGH")
        elif i > 1:
            println("MID")
        else:
            println("LOW")
"#;
    assert_source_compiles(source, "elif inside for loop");
}

/// Test elif with boolean variable.
#[test]
fn test_elif_with_bool() {
    let source = r#"
def main():
    flag: bool = true
    other: bool = false
    if flag and other:
        println("BOTH")
    elif flag:
        println("FIRST")
    elif other:
        println("SECOND")
    else:
        println("NONE")
"#;
    assert_source_compiles(source, "elif with boolean variables");
}

/// Test elif with comparison operators.
#[test]
fn test_elif_with_comparisons() {
    let source = r#"
def main():
    x: byte = 50
    y: byte = 50
    if x > y:
        println("X BIGGER")
    elif x < y:
        println("Y BIGGER")
    elif x == y:
        println("EQUAL")
"#;
    assert_source_compiles(source, "elif with comparison operators");
}

/// Test elif immediately after function start.
#[test]
fn test_elif_first_statement() {
    let source = r#"
def main():
    if false:
        println("A")
    elif true:
        println("B")
"#;
    assert_source_compiles(source, "elif as first control structure");
}

/// Test multiple independent elif chains.
#[test]
fn test_multiple_elif_chains() {
    let source = r#"
def main():
    x: byte = 10
    y: byte = 20

    if x > 50:
        println("X BIG")
    elif x > 5:
        println("X MED")
    else:
        println("X SMALL")

    if y > 50:
        println("Y BIG")
    elif y > 15:
        println("Y MED")
    else:
        println("Y SMALL")
"#;
    assert_source_compiles(source, "multiple independent elif chains");
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test elif with single-line blocks (minimal code in each branch).
#[test]
fn test_elif_single_statements() {
    let source = r#"
def main():
    x: byte = 1
    if x == 0:
        pass
    elif x == 1:
        pass
    else:
        pass
"#;
    assert_source_compiles(source, "elif with pass statements");
}

/// Test elif with variable assignments in branches.
#[test]
fn test_elif_with_assignments() {
    let source = r#"
def main():
    x: byte = 50
    result: byte = 0
    if x > 100:
        result = 3
    elif x > 50:
        result = 2
    elif x > 0:
        result = 1
    else:
        result = 0
    println(result)
"#;
    assert_source_compiles(source, "elif with assignments");
}

/// Test deeply nested elif (elif inside elif block).
#[test]
fn test_elif_deeply_nested() {
    let source = r#"
def main():
    a: byte = 5
    b: byte = 10
    if a > 10:
        println("A BIG")
    elif a > 0:
        if b > 20:
            println("B BIG")
        elif b > 5:
            println("B MED")
        else:
            println("B SMALL")
    else:
        println("A ZERO")
"#;
    assert_source_compiles(source, "deeply nested elif");
}
