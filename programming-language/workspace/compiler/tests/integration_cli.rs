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

//! End-to-end CLI integration tests.

use std::process::Command;

fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cobra64"))
}

/// Test --help flag.
#[test]
fn test_help_flag() {
    let output = cargo_bin()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cobra64") || stdout.contains("cobra64"));
    assert!(stdout.contains("-o") || stdout.contains("--output"));
    assert!(stdout.contains("-v") || stdout.contains("--verbose"));
}

/// Test --version flag.
#[test]
fn test_version_flag() {
    let output = cargo_bin()
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cobra64"));
    assert!(stdout.contains("0.1.0"));
}

/// Test compiling hello world to PRG.
#[test]
fn test_compile_hello_to_prg() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_hello.cb64");
    let output_path = temp_dir.join("test_hello.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    println("HELLO")
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output file not created");

    // Verify PRG structure (2-byte header + code)
    let data = std::fs::read(&output_path).unwrap();
    assert!(data.len() > 2, "PRG too small");
    assert_eq!(data[0], 0x01, "Load address low byte");
    assert_eq!(data[1], 0x08, "Load address high byte");

    // Clean up
    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test compiling to D64.
#[test]
fn test_compile_to_d64() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_d64.cb64");
    let output_path = temp_dir.join("test_d64.d64");

    std::fs::write(
        &source_path,
        r#"
def main():
    pass
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output file not created");

    // Verify D64 size
    let metadata = std::fs::metadata(&output_path).unwrap();
    assert_eq!(
        metadata.len(),
        174_848,
        "D64 should be exactly 174848 bytes"
    );

    // Clean up
    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test verbose flag.
#[test]
fn test_verbose_output() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_verbose.cb64");
    let output_path = temp_dir.join("test_verbose.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    pass
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg("-v")
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cobra64 Compiler"));
    assert!(stdout.contains("Output format:"));
    assert!(stdout.contains("Generated"));
    assert!(stdout.contains("bytes"));

    // Clean up
    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test error on missing output flag.
#[test]
fn test_missing_output_flag() {
    let output = cargo_bin()
        .arg("test.cb64")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Error message should mention output is required unless --run is used
    assert!(
        stderr.contains("--output")
            || stderr.contains("-o")
            || stderr.contains("Output file is required")
    );
}

/// Test that --run without -o compiles successfully (VICE will fail to launch in CI).
#[test]
fn test_run_without_output_compiles() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_run_no_output.cb64");

    std::fs::write(&source_path, "def main():\n    pass\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("--run")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Compilation should succeed (indicated by "Compiled" message)
    assert!(
        stdout.contains("Compiled"),
        "Expected compilation success, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // VICE will not be found in CI, so we expect exit code 4
    assert_eq!(
        output.status.code(),
        Some(4),
        "Expected exit code 4 (VICE not found)"
    );
}

/// Test error on unknown output format.
#[test]
fn test_unknown_output_format() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_format.cb64");

    std::fs::write(&source_path, "def main():\n    pass\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg("output.xyz")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown output format"));

    std::fs::remove_file(&source_path).ok();
}

/// Test error on missing source file.
#[test]
fn test_missing_source_file() {
    let output = cargo_bin()
        .arg("nonexistent.cb64")
        .arg("-o")
        .arg("output.prg")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot read"));
}

/// Test syntax error reporting.
#[test]
fn test_syntax_error_reporting() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_syntax_err.cb64");

    std::fs::write(
        &source_path,
        r#"
def main()
    pass
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg("output.prg")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[E"));
    assert!(stderr.contains("-->"));

    std::fs::remove_file(&source_path).ok();
}

/// Test semantic error reporting.
#[test]
fn test_semantic_error_reporting() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_semantic_err.cb64");

    std::fs::write(
        &source_path,
        r#"
def main():
    x = undefined_variable
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg("output.prg")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[E200]") || stderr.contains("Undefined"));
    assert!(stderr.contains("-->"));

    std::fs::remove_file(&source_path).ok();
}

/// Test program with variables and math.
#[test]
fn test_compile_variables_and_math() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_math.cb64");
    let output_path = temp_dir.join("test_math.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    a: byte = 10
    b: byte = 5
    c: byte = a + b
    d: byte = a * b
    e: byte = a - b
    f: byte = a / b
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test program with if/else.
#[test]
fn test_compile_if_else() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_if.cb64");
    let output_path = temp_dir.join("test_if.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    x: byte = 5
    if x > 3:
        println("BIG")
    else:
        println("SMALL")
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test program with while loop.
#[test]
fn test_compile_while_loop() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_while.cb64");
    let output_path = temp_dir.join("test_while.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    i: byte = 0
    while i < 10:
        println(i)
        i = i + 1
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test program with user-defined functions.
#[test]
fn test_compile_functions() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_func.cb64");
    let output_path = temp_dir.join("test_func.prg");

    std::fs::write(
        &source_path,
        r#"
def add(a: byte, b: byte) -> byte:
    return a + b

def main():
    result: byte = add(3, 4)
    println(result)
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test missing main function error.
#[test]
fn test_missing_main_error() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_no_main.cb64");

    std::fs::write(
        &source_path,
        r#"
def foo():
    pass
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg("output.prg")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_lowercase().contains("main"));

    std::fs::remove_file(&source_path).ok();
}

/// Test normal output message (non-verbose).
#[test]
fn test_normal_output_message() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_normal.cb64");
    let output_path = temp_dir.join("test_normal.prg");

    std::fs::write(
        &source_path,
        r#"
def main():
    pass
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Compiled"));
    assert!(stdout.contains("->"));

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test exit codes.
#[test]
fn test_exit_codes() {
    // Success case
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_exit.cb64");
    let output_path = temp_dir.join("test_exit.prg");

    std::fs::write(&source_path, "def main():\n    pass\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));

    // Compilation error case
    std::fs::write(&source_path, "def main():\n    undefined\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));

    // Unknown format error case
    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg("test.xyz")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));

    // File not found error case
    let output = cargo_bin()
        .arg("nonexistent.cb64")
        .arg("-o")
        .arg("test.prg")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test --run flag shows in help.
#[test]
fn test_run_flag_in_help() {
    let output = cargo_bin()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--run"));
    assert!(stdout.contains("-r"));
    assert!(stdout.contains("VICE"));
}

/// Test --watch flag shows in help.
#[test]
fn test_watch_flag_in_help() {
    let output = cargo_bin()
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--watch"));
    assert!(stdout.contains("-w"));
}

/// Test --run with invalid --vice-path returns exit code 4.
#[test]
fn test_run_invalid_vice_path() {
    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_run_invalid.cb64");
    let output_path = temp_dir.join("test_run_invalid.prg");

    std::fs::write(&source_path, "def main():\n    pass\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--run")
        .arg("--vice-path")
        .arg("/nonexistent/path/to/vice")
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}

/// Test --run without VICE installed shows helpful error.
#[test]
fn test_run_vice_not_found_message() {
    // Skip this test if VICE is actually installed
    if which::which("x64sc").is_ok() || which::which("x64").is_ok() {
        return;
    }

    let temp_dir = std::env::temp_dir();
    let source_path = temp_dir.join("test_run_notfound.cb64");
    let output_path = temp_dir.join("test_run_notfound.prg");

    std::fs::write(&source_path, "def main():\n    pass\n").unwrap();

    let output = cargo_bin()
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--run")
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("VICE emulator not found"));
    assert!(stderr.contains("brew install") || stderr.contains("apt install"));

    std::fs::remove_file(&source_path).ok();
    std::fs::remove_file(&output_path).ok();
}
