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

//! Regression tests for the Cobra64 compiler.
//!
//! These tests ensure that fixed bugs stay fixed and document known issues.
//!
//! # Naming Convention
//!
//! Test files follow the pattern: `issue_NNN_short_description.cb64`
//! - NNN: Issue number (001, 002, etc.)
//! - short_description: Brief description of the bug
//!
//! # Test Categories
//!
//! 1. **Fixed bugs**: Tests that verify a bug fix (should pass)
//! 2. **Known bugs**: Tests that document unfixed bugs (marked #[ignore] or test for expected failure)
//! 3. **Workarounds**: Tests that demonstrate working alternatives
//!
//! # Adding a New Regression Test
//!
//! When fixing a bug:
//! 1. Create `tests/regression/issue_NNN_description.cb64`
//! 2. Add a comment header explaining the bug
//! 3. Add the minimal code that triggered the bug
//! 4. Add a test function in this file
//! 5. Verify the test passes after the fix

use std::fs;
use std::path::Path;

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Compile a regression test file and return success/failure.
fn compile_regression_test(filename: &str) -> Result<Vec<u8>, String> {
    let path = format!("tests/regression/{}", filename);
    let source =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    cobra64::compile(&source).map_err(|e| format!("Compile error: {:?}", e))
}

/// Check that a regression test file compiles successfully.
fn assert_compiles(filename: &str) {
    match compile_regression_test(filename) {
        Ok(_) => {}
        Err(e) => panic!("{} should compile but failed: {}", filename, e),
    }
}

/// Check that a regression test file fails to compile (for known bugs).
#[allow(dead_code)]
fn assert_fails_to_compile(filename: &str) {
    if compile_regression_test(filename).is_ok() {
        panic!("{} should fail to compile but succeeded", filename);
    }
}

// ============================================================================
// Regression Tests - Workarounds (should compile)
// ============================================================================

/// Issue #001: elif code generation bug
/// The workaround using nested if-else should compile.
#[test]
fn test_issue_001_elif_workaround() {
    assert_compiles("issue_001_elif_codegen.cb64");
}

/// Issue #002: Comment as first line in function body
/// The workaround with inline comments should compile.
#[test]
fn test_issue_002_comment_workaround() {
    assert_compiles("issue_002_comment_first_line.cb64");
}

/// Issue #003: Signed type comparison limitations
/// Using unsigned types as workaround should compile.
#[test]
fn test_issue_003_unsigned_workaround() {
    assert_compiles("issue_003_signed_comparison.cb64");
}

/// Issue #004: Deep nesting exceeds branch limit
/// Staying within safe nesting limits should compile.
#[test]
fn test_issue_004_safe_nesting() {
    assert_compiles("issue_004_branch_limit.cb64");
}

// ============================================================================
// Known Bug Documentation Tests
// ============================================================================

/// Verify that elif works correctly (bug fixed in v0.8.0).
/// The bug was in label management in generate_if() - the next_label
/// was created but never defined for subsequent elif branches.
#[test]
fn test_elif_fixed() {
    let source = r#"
def main():
    x: byte = 5
    if x > 7:
        println("A")
    elif x > 3:
        println("B")
    else:
        println("C")
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "elif should compile correctly: {:?}",
        result.err()
    );
}

/// BUG-007: Comments as first line in block now work correctly.
/// Previously this caused "Expected indented block, found NEWLINE" error.
/// Fixed by treating comment-only lines as invisible (like empty lines).
#[test]
fn test_bug_007_comment_first_line_fixed() {
    let source = r#"def main():
    # This comment as first line used to cause the bug
    x: byte = 1
"#;
    let result = cobra64::compile(source);
    assert!(
        result.is_ok(),
        "Comment-first-line should work now: {:?}",
        result.err()
    );
}

/// Verify that deep nesting works with automatic branch trampolining.
/// The compiler now automatically converts far branches to JMP instructions.
#[test]
fn test_deep_nesting_works_with_trampolining() {
    let mut source = String::from("def main():\n    x: byte = 10\n");
    let mut indent = String::from("    ");

    // 15 levels of nesting - now works thanks to trampolining
    for i in 0..15 {
        source.push_str(&format!("{}if x > {}:\n", indent, i));
        indent.push_str("    ");
    }
    source.push_str(&format!("{}println(\"DEEP\")\n", indent));

    let result = cobra64::compile(&source);
    assert!(
        result.is_ok(),
        "Deep nesting should compile with trampolining: {:?}",
        result.err()
    );
}

// ============================================================================
// All Regression Files Compile Test
// ============================================================================

/// Test that all regression test files (workarounds) compile.
#[test]
fn test_all_regression_files_compile() {
    let regression_dir = Path::new("tests/regression");

    if !regression_dir.exists() {
        panic!("Regression test directory not found");
    }

    let files: Vec<_> = fs::read_dir(regression_dir)
        .expect("Failed to read regression directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cb64"))
        .collect();

    assert!(!files.is_empty(), "No regression test files found");

    for path in &files {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));

        let result = cobra64::compile(&source);
        assert!(
            result.is_ok(),
            "Regression test {} should compile (workaround): {:?}",
            path.display(),
            result.err()
        );
    }
}

// ============================================================================
// Template for Future Regression Tests
// ============================================================================

/// Template test showing how to add new regression tests.
/// Copy and modify this when adding tests for fixed bugs.
#[test]
fn test_template_for_new_regression() {
    // 1. Minimal code that triggered the bug
    let source = r#"
def main():
    x: byte = 42
    println(x)
"#;

    // 2. Verify it compiles after the fix
    let result = cobra64::compile(source);
    assert!(result.is_ok(), "Should compile after fix");

    // 3. Optionally verify the generated code
    let code = result.unwrap();
    assert!(!code.is_empty(), "Should generate code");
}
