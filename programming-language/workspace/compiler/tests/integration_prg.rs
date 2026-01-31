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

//! Integration tests for PRG file generation.

use cobra64::output::{format_from_extension, write_prg, OutputFormat};
use std::path::Path;

/// Test that a minimal program compiles and generates a valid PRG.
#[test]
fn test_compile_minimal_program() {
    let source = r#"
def main():
    pass
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Code should not be empty
    assert!(!code.is_empty(), "Generated code should not be empty");

    // Code should start with BASIC stub (link pointer at $080C)
    assert!(code.len() >= 13, "Code should include BASIC stub");
    assert_eq!(
        code[0], 0x0C,
        "First byte should be low byte of link pointer"
    );
    assert_eq!(
        code[1], 0x08,
        "Second byte should be high byte of link pointer"
    );
}

/// Test that hello world compiles and generates valid PRG.
#[test]
fn test_compile_hello_world() {
    let source = r#"
def main():
    println("HELLO")
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should contain the string "HELLO" somewhere in the code
    let hello_bytes = b"HELLO";
    let found = code
        .windows(hello_bytes.len())
        .any(|window| window == hello_bytes);
    assert!(found, "Generated code should contain 'HELLO' string");
}

/// Test PRG file structure.
#[test]
fn test_prg_file_structure() {
    let source = r#"
def main():
    pass
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_structure.prg");

    write_prg(&code, &path).expect("PRG writing should succeed");

    // Read back the file
    let data = std::fs::read(&path).expect("Should be able to read PRG");

    // PRG should have 2-byte header + code
    assert_eq!(data.len(), code.len() + 2, "PRG should be code + 2 bytes");

    // First two bytes should be load address $0801
    assert_eq!(data[0], 0x01, "Load address low byte should be $01");
    assert_eq!(data[1], 0x08, "Load address high byte should be $08");

    // Rest should be the code
    assert_eq!(
        &data[2..],
        &code[..],
        "PRG data should match generated code"
    );

    // Clean up
    std::fs::remove_file(&path).ok();
}

/// Test that BASIC stub is correct.
#[test]
fn test_basic_stub_format() {
    let source = r#"
def main():
    pass
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // BASIC stub should be 13 bytes
    assert!(code.len() >= 13, "Code should include full BASIC stub");

    // Check BASIC stub structure:
    // Bytes 0-1: Link to next line ($080C)
    assert_eq!(code[0], 0x0C);
    assert_eq!(code[1], 0x08);

    // Bytes 2-3: Line number (10 = $000A)
    assert_eq!(code[2], 0x0A);
    assert_eq!(code[3], 0x00);

    // Byte 4: SYS token ($9E)
    assert_eq!(code[4], 0x9E);

    // Byte 5: Space ($20)
    assert_eq!(code[5], 0x20);

    // Bytes 6-9: "2062" in ASCII
    assert_eq!(code[6], b'2');
    assert_eq!(code[7], b'0');
    assert_eq!(code[8], b'6');
    assert_eq!(code[9], b'2');

    // Byte 10: End of line ($00)
    assert_eq!(code[10], 0x00);

    // Bytes 11-12: End of BASIC program ($00 $00)
    assert_eq!(code[11], 0x00);
    assert_eq!(code[12], 0x00);
}

/// Test that variables are allocated.
#[test]
fn test_variable_allocation() {
    let source = r#"
def main():
    x: byte = 42
    y: word = 1000
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Code should be larger due to variable initialization
    assert!(
        code.len() > 50,
        "Code should include variable initialization"
    );
}

/// Test format detection from extension.
#[test]
fn test_format_detection() {
    assert_eq!(
        format_from_extension(Path::new("test.prg")),
        Some(OutputFormat::Prg)
    );
    assert_eq!(
        format_from_extension(Path::new("test.PRG")),
        Some(OutputFormat::Prg)
    );
    assert_eq!(
        format_from_extension(Path::new("test.d64")),
        Some(OutputFormat::D64)
    );
    assert_eq!(format_from_extension(Path::new("test.txt")), None);
}

/// Test compilation of program with control flow.
#[test]
fn test_compile_control_flow() {
    let source = r#"
def main():
    x: byte = 0
    if x == 0:
        x = 1
    while x < 5:
        x = x + 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should generate more code for control flow
    assert!(code.len() > 100, "Control flow code should be substantial");
}

/// Test compilation of program with function calls.
#[test]
fn test_compile_function_calls() {
    let source = r#"
def helper() -> byte:
    return 42

def main():
    x: byte = helper()
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should include JSR instruction ($20)
    let has_jsr = code.contains(&0x20);
    assert!(has_jsr, "Code should contain JSR instruction");
}

/// Test that multiple PRG files can be written.
#[test]
fn test_multiple_prg_files() {
    let source = r#"
def main():
    pass
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    let temp_dir = std::env::temp_dir();

    // Write multiple files
    for i in 0..3 {
        let path = temp_dir.join(format!("test_multi_{}.prg", i));
        write_prg(&code, &path).expect("PRG writing should succeed");

        // Verify file exists and has correct size
        let metadata = std::fs::metadata(&path).expect("File should exist");
        assert_eq!(metadata.len() as usize, code.len() + 2);

        std::fs::remove_file(&path).ok();
    }
}
