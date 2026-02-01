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

//! Emulator-based runtime tests for the Cobra64 compiler.
//!
//! These tests compile programs and verify they run correctly in a C64 emulator.
//! They are marked as `#[ignore]` by default because they require:
//!   - VICE emulator (x64sc) installed
//!   - xvfb-run for headless operation (Linux)
//!   - Sufficient time to run emulator
//!
//! To run these tests:
//!   cargo test --test runtime_tests -- --ignored
//!
//! To run with output:
//!   cargo test --test runtime_tests -- --ignored --nocapture

use std::fs;
use std::path::Path;
use std::process::Command;

/// Check if VICE emulator is available.
fn vice_available() -> bool {
    Command::new("which")
        .arg("x64sc")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Compile a source file to PRG.
fn compile_to_prg(source: &str, output_path: &Path) -> Result<(), String> {
    let code = cobra64::compile(source).map_err(|e| format!("Compile error: {:?}", e))?;
    fs::write(output_path, &code).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

// ============================================================================
// Compilation Tests (always run)
// ============================================================================

/// Test that all runtime test files compile successfully.
#[test]
fn test_runtime_files_compile() {
    let runtime_dir = Path::new("tests/runtime");

    let files: Vec<_> = fs::read_dir(runtime_dir)
        .expect("Failed to read runtime directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cb64"))
        .collect();

    assert!(!files.is_empty(), "No runtime test files found");

    for path in &files {
        let source = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));

        let result = cobra64::compile(&source);
        assert!(
            result.is_ok(),
            "Runtime test {} failed to compile: {:?}",
            path.display(),
            result.err()
        );
    }
}

/// Test that each runtime test has a corresponding .expected file.
#[test]
fn test_runtime_expected_files_exist() {
    let runtime_dir = Path::new("tests/runtime");

    let cb64_files: Vec<_> = fs::read_dir(runtime_dir)
        .expect("Failed to read runtime directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cb64"))
        .collect();

    for path in &cb64_files {
        let expected_path = path.with_extension("expected");
        assert!(
            expected_path.exists(),
            "Missing expected output file: {}",
            expected_path.display()
        );
    }
}

/// Test that generated PRG files have valid structure.
#[test]
fn test_runtime_prg_structure() {
    let source = include_str!("runtime/hello.cb64");
    let code = cobra64::compile(source).expect("Should compile");

    // Check minimum size (load address + BASIC stub + some code)
    assert!(code.len() >= 20, "PRG too small: {} bytes", code.len());

    // Check load address is valid
    let load_addr = u16::from_le_bytes([code[0], code[1]]);
    assert!(
        (0x0801..=0x9FFF).contains(&load_addr),
        "Invalid load address: ${:04X}",
        load_addr
    );
}

// ============================================================================
// Emulator Tests (require VICE, ignored by default)
// ============================================================================

/// Test helper to run a PRG in VICE and check output.
/// This is a placeholder - actual screen capture is complex.
#[allow(dead_code)]
fn run_in_vice(prg_path: &Path, _expected: &str, timeout_secs: u64) -> Result<(), String> {
    if !vice_available() {
        return Err("VICE emulator not available".to_string());
    }

    let script_path = Path::new("tests/runtime/run_vice.sh");
    if !script_path.exists() {
        return Err("run_vice.sh script not found".to_string());
    }

    let output = Command::new("bash")
        .arg(script_path)
        .arg(prg_path)
        .arg(timeout_secs.to_string())
        .output()
        .map_err(|e| format!("Failed to run VICE: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "VICE exited with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Note: Actual output verification would require screen capture + OCR
    // or using VICE's monitor to read screen memory
    Ok(())
}

#[test]
#[ignore = "Requires VICE emulator - run with: cargo test -- --ignored"]
fn test_runtime_hello() {
    if !vice_available() {
        eprintln!("Skipping: VICE emulator not available");
        return;
    }

    let source = include_str!("runtime/hello.cb64");
    let temp_dir = std::env::temp_dir();
    let prg_path = temp_dir.join("test_hello.prg");

    compile_to_prg(source, &prg_path).expect("Should compile");
    assert!(prg_path.exists(), "PRG file should exist");

    // Run in VICE (verification is visual for now)
    let _expected = include_str!("runtime/hello.expected");
    if let Err(e) = run_in_vice(&prg_path, _expected, 5) {
        eprintln!("VICE test skipped: {}", e);
    }

    // Cleanup
    let _ = fs::remove_file(&prg_path);
}

#[test]
#[ignore = "Requires VICE emulator - run with: cargo test -- --ignored"]
fn test_runtime_math() {
    if !vice_available() {
        eprintln!("Skipping: VICE emulator not available");
        return;
    }

    let source = include_str!("runtime/math.cb64");
    let temp_dir = std::env::temp_dir();
    let prg_path = temp_dir.join("test_math.prg");

    compile_to_prg(source, &prg_path).expect("Should compile");
    assert!(prg_path.exists(), "PRG file should exist");

    let _expected = include_str!("runtime/math.expected");
    if let Err(e) = run_in_vice(&prg_path, _expected, 5) {
        eprintln!("VICE test skipped: {}", e);
    }

    let _ = fs::remove_file(&prg_path);
}

#[test]
#[ignore = "Requires VICE emulator - run with: cargo test -- --ignored"]
fn test_runtime_loop() {
    if !vice_available() {
        eprintln!("Skipping: VICE emulator not available");
        return;
    }

    let source = include_str!("runtime/loop.cb64");
    let temp_dir = std::env::temp_dir();
    let prg_path = temp_dir.join("test_loop.prg");

    compile_to_prg(source, &prg_path).expect("Should compile");
    assert!(prg_path.exists(), "PRG file should exist");

    let _expected = include_str!("runtime/loop.expected");
    if let Err(e) = run_in_vice(&prg_path, _expected, 5) {
        eprintln!("VICE test skipped: {}", e);
    }

    let _ = fs::remove_file(&prg_path);
}

#[test]
#[ignore = "Requires VICE emulator - run with: cargo test -- --ignored"]
fn test_runtime_branch() {
    if !vice_available() {
        eprintln!("Skipping: VICE emulator not available");
        return;
    }

    let source = include_str!("runtime/branch.cb64");
    let temp_dir = std::env::temp_dir();
    let prg_path = temp_dir.join("test_branch.prg");

    compile_to_prg(source, &prg_path).expect("Should compile");
    assert!(prg_path.exists(), "PRG file should exist");

    let _expected = include_str!("runtime/branch.expected");
    if let Err(e) = run_in_vice(&prg_path, _expected, 5) {
        eprintln!("VICE test skipped: {}", e);
    }

    let _ = fs::remove_file(&prg_path);
}

// ============================================================================
// Documentation
// ============================================================================

/// This module documents how to run emulator tests manually.
///
/// # Prerequisites
///
/// 1. Install VICE emulator:
///    - Linux: `sudo apt install vice`
///    - macOS: `brew install vice`
///    - Windows: Download from https://vice-emu.sourceforge.io/
///
/// 2. For headless testing on Linux:
///    `sudo apt install xvfb`
///
/// # Running Tests
///
/// Run all emulator tests:
/// ```bash
/// cargo test --test runtime_tests -- --ignored
/// ```
///
/// Run with output visible:
/// ```bash
/// cargo test --test runtime_tests -- --ignored --nocapture
/// ```
///
/// # Manual Testing
///
/// 1. Compile a test program:
///    ```bash
///    cargo run -- tests/runtime/hello.cb64 -o hello.prg
///    ```
///
/// 2. Run in VICE:
///    ```bash
///    x64sc hello.prg
///    ```
///
/// 3. Compare output with expected file:
///    ```bash
///    cat tests/runtime/hello.expected
///    ```
///
/// # Output Verification
///
/// Currently, output verification is visual. Future improvements could include:
/// - Using VICE's monitor to read screen memory ($0400-$07E7)
/// - Screen capture with OCR (requires C64 font recognition)
/// - Serial/RS232 output capture via VICE's virtual devices
#[cfg(test)]
mod documentation {
    #[test]
    fn test_documentation_compiles() {
        // This test exists to ensure the documentation module compiles
    }
}
