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

//! Integration tests for data block functionality.

use cobra64::error::ErrorCode;
use std::fs;
use std::io::Write;
use std::path::Path;

fn compile(source: &str) -> Result<Vec<u8>, cobra64::error::CompileError> {
    cobra64::compile(source)
}

fn compile_with_path(source: &str, path: &Path) -> Result<Vec<u8>, cobra64::error::CompileError> {
    cobra64::compile_with_path(source, path)
}

/// Check if a byte sequence appears in the compiled code.
fn code_contains_bytes(code: &[u8], pattern: &[u8]) -> bool {
    code.windows(pattern.len()).any(|window| window == pattern)
}

// ============================================
// Parsing Tests
// ============================================

#[test]
fn test_data_block_simple() {
    let source = r#"
data SPRITE:
    $00, $3C, $00
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_multi_line() {
    let source = r#"
data SPRITE_BALL:
    $00, $3C, $00
    $00, $7E, $00
    $00, $FF, $00
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_decimal_values() {
    let source = r#"
data VALUES:
    255, 128, 64, 32, 16, 8, 4, 2, 1, 0
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_binary_values() {
    let source = r#"
data BITMAP:
    %11110000, %00001111, %10101010
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_mixed_values() {
    let source = r#"
data MIXED:
    $FF, 128, %10101010
    $00, 255, %00000000
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_with_include_file_not_found() {
    // Test that include with non-existent file produces FileNotFound error
    let source = r#"
data FONT:
    include "nonexistent.bin"
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::FileNotFound);
}

#[test]
fn test_data_block_include_with_range_file_not_found() {
    // Test that include with non-existent file produces FileNotFound error
    let source = r#"
data MUSIC:
    include "nonexistent.sid", $7E, $1000
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::FileNotFound);
}

#[test]
fn test_multiple_data_blocks() {
    let source = r#"
data SPRITE1:
    $01, $02, $03
end

data SPRITE2:
    $10, $20, $30
end

data SPRITE3:
    $AA, $BB, $CC
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

// ============================================
// Error Tests
// ============================================

#[test]
fn test_data_block_duplicate_name() {
    let source = r#"
data SPRITE:
    $00, $01, $02
end

data SPRITE:
    $10, $11, $12
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::VariableAlreadyDefined);
}

#[test]
fn test_data_block_name_conflicts_with_variable() {
    let source = r#"
SPRITE: word = 100

data SPRITE:
    $00, $01, $02
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::VariableAlreadyDefined);
}

#[test]
fn test_data_block_name_conflicts_with_constant() {
    let source = r#"
const SPRITE: word = 100

data SPRITE:
    $00, $01, $02
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::VariableAlreadyDefined);
}

#[test]
fn test_data_block_inside_function_not_allowed() {
    // Data blocks inside functions are rejected by the parser
    // (the 'data' keyword is not recognized as a statement)
    let source = r#"
def main():
    data SPRITE:
        $00, $01, $02
    end
"#;
    let result = compile(source);
    assert!(result.is_err());
    // Parser error because 'data' is not a valid statement keyword
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::UnexpectedToken);
}

#[test]
fn test_data_block_value_out_of_range() {
    let source = r#"
data BAD:
    256
end

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::ValueOutOfRange);
}

#[test]
fn test_data_block_missing_end() {
    let source = r#"
data INCOMPLETE:
    $00, $01

def main():
    pass
"#;
    let result = compile(source);
    assert!(result.is_err());
}

// ============================================
// Data Block with Other Declarations
// ============================================

#[test]
fn test_data_block_with_constants_and_variables() {
    let source = r#"
const MAX_SPRITES: byte = 8

sprite_count: byte = 0

data SPRITE_DATA:
    $00, $3C, $00
    $00, $7E, $00
end

def main():
    sprite_count = 1
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_between_functions() {
    let source = r#"
def helper():
    pass

data SHARED_DATA:
    $01, $02, $03
end

def main():
    helper()
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

// ============================================
// Code Generation Tests
// ============================================

#[test]
fn test_data_block_bytes_in_output() {
    let source = r#"
data TEST_DATA:
    $DE, $AD, $BE, $EF
end

def main():
    pass
"#;
    let code = compile(source).expect("Failed to compile");

    // The bytes $DE, $AD, $BE, $EF should appear in the output
    assert!(
        code_contains_bytes(&code, &[0xDE, 0xAD, 0xBE, 0xEF]),
        "Data block bytes not found in compiled output"
    );
}

#[test]
fn test_multiple_data_blocks_in_output() {
    let source = r#"
data BLOCK1:
    $11, $22, $33
end

data BLOCK2:
    $AA, $BB, $CC
end

def main():
    pass
"#;
    let code = compile(source).expect("Failed to compile");

    // Both byte sequences should appear in the output
    assert!(
        code_contains_bytes(&code, &[0x11, 0x22, 0x33]),
        "BLOCK1 bytes not found in compiled output"
    );
    assert!(
        code_contains_bytes(&code, &[0xAA, 0xBB, 0xCC]),
        "BLOCK2 bytes not found in compiled output"
    );
}

#[test]
fn test_data_block_multi_line_bytes_in_output() {
    let source = r#"
data SPRITE:
    $00, $3C, $00
    $00, $7E, $00
    $00, $FF, $00
end

def main():
    pass
"#;
    let code = compile(source).expect("Failed to compile");

    // All three lines should be contiguous in the output
    assert!(
        code_contains_bytes(
            &code,
            &[0x00, 0x3C, 0x00, 0x00, 0x7E, 0x00, 0x00, 0xFF, 0x00]
        ),
        "Multi-line data block bytes not found in compiled output"
    );
}

// ============================================
// Include File Tests (with real files)
// ============================================

#[test]
fn test_data_block_include_real_file() {
    // Create a temporary directory and file
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("sprite.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Write binary data to the file
    let binary_data = vec![0xCA, 0xFE, 0xBA, 0xBE];
    let mut file = fs::File::create(&bin_path).expect("Failed to create bin file");
    file.write_all(&binary_data)
        .expect("Failed to write bin file");

    // Write source file (needed for path resolution)
    let source = r#"
data SPRITE:
    include "sprite.bin"
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    // Compile with path
    let code = compile_with_path(source, &source_path).expect("Failed to compile");

    // The binary data should appear in the output
    assert!(
        code_contains_bytes(&code, &binary_data),
        "Include file bytes not found in compiled output"
    );
}

#[test]
fn test_data_block_include_with_offset() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("data.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Create a file with known data
    let binary_data: Vec<u8> = (0..16).collect();
    fs::write(&bin_path, &binary_data).expect("Failed to write bin file");

    // Include with offset 4, no length (read to end)
    let source = r#"
data PARTIAL:
    include "data.bin", $04
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    let code = compile_with_path(source, &source_path).expect("Failed to compile");

    // Should contain bytes 4-15 (offset 4 to end)
    let expected: Vec<u8> = (4..16).collect();
    assert!(
        code_contains_bytes(&code, &expected),
        "Include with offset bytes not found in compiled output"
    );
}

#[test]
fn test_data_block_include_with_offset_and_length() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("data.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Create a file with known data
    let binary_data: Vec<u8> = (0..32).collect();
    fs::write(&bin_path, &binary_data).expect("Failed to write bin file");

    // Include with offset 8 and length 4
    let source = r#"
data PARTIAL:
    include "data.bin", $08, $04
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    let code = compile_with_path(source, &source_path).expect("Failed to compile");

    // Should contain bytes 8, 9, 10, 11
    let expected: Vec<u8> = vec![8, 9, 10, 11];
    assert!(
        code_contains_bytes(&code, &expected),
        "Include with offset and length bytes not found in compiled output"
    );
}

#[test]
fn test_data_block_include_offset_out_of_bounds() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("small.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Create a small file
    fs::write(&bin_path, &[0x01, 0x02, 0x03, 0x04]).expect("Failed to write bin file");

    // Try to include with offset beyond file size
    let source = r#"
data BAD:
    include "small.bin", $100
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    let result = compile_with_path(source, &source_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::IncludeOffsetOutOfBounds);
}

#[test]
fn test_data_block_include_length_out_of_bounds() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("small.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Create a small file
    fs::write(&bin_path, &[0x01, 0x02, 0x03, 0x04]).expect("Failed to write bin file");

    // Try to include with offset + length beyond file size
    let source = r#"
data BAD:
    include "small.bin", $02, $10
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    let result = compile_with_path(source, &source_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::IncludeLengthOutOfBounds);
}

#[test]
fn test_data_block_mixed_inline_and_include() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let bin_path = temp_dir.path().join("data.bin");
    let source_path = temp_dir.path().join("test.cb64");

    // Create a file with known data
    let binary_data = vec![0xAA, 0xBB, 0xCC];
    fs::write(&bin_path, &binary_data).expect("Failed to write bin file");

    // Mix inline bytes and include
    let source = r#"
data MIXED:
    $11, $22, $33
    include "data.bin"
    $DD, $EE, $FF
end

def main():
    pass
"#;
    fs::write(&source_path, source).expect("Failed to write source file");

    let code = compile_with_path(source, &source_path).expect("Failed to compile");

    // All bytes should appear in order
    let expected = vec![0x11, 0x22, 0x33, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    assert!(
        code_contains_bytes(&code, &expected),
        "Mixed inline and include bytes not found in compiled output"
    );
}

// ============================================
// Data Block Address Resolution Tests
// ============================================

#[test]
fn test_data_block_address_in_expression() {
    // Test that data block names can be used as word values in expressions
    let source = r#"
data SPRITE_DATA:
    $CA, $FE, $BA, $BE
end

def main():
    x: word = SPRITE_DATA
    poke($D000, byte(x))
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    let code = result.unwrap();
    // The data block bytes should be in the output
    assert!(
        code_contains_bytes(&code, &[0xCA, 0xFE, 0xBA, 0xBE]),
        "Data block bytes not found in compiled output"
    );
}

#[test]
fn test_data_block_address_in_function_call() {
    // Test that data block names can be passed to functions
    let source = r#"
data TEST_DATA:
    $11, $22, $33
end

def main():
    # Use data block address in poke (writes low byte)
    poke($C000, byte(TEST_DATA))
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_data_block_address_arithmetic() {
    // Test that data block addresses can be used in arithmetic
    let source = r#"
data BLOCK1:
    $AA, $BB
end

def main():
    addr: word = BLOCK1 + 1
    x: byte = byte(addr)
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_multiple_data_block_references() {
    // Test multiple data blocks being referenced
    let source = r#"
data SPRITE1:
    $01, $02, $03
end

data SPRITE2:
    $11, $12, $13
end

def main():
    s1: word = SPRITE1
    s2: word = SPRITE2
    diff: word = s2 - s1
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());

    let code = result.unwrap();
    // Both data blocks should be in the output
    assert!(
        code_contains_bytes(&code, &[0x01, 0x02, 0x03]),
        "SPRITE1 bytes not found"
    );
    assert!(
        code_contains_bytes(&code, &[0x11, 0x12, 0x13]),
        "SPRITE2 bytes not found"
    );
}

#[test]
fn test_data_block_address_type_is_word() {
    // Data block addresses should be word type and assignable to word variables
    let source = r#"
data MY_DATA:
    $FF
end

def main():
    addr: word = MY_DATA
    hi: byte = byte(addr >> 8)
    lo: byte = byte(addr)
"#;
    let result = compile(source);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}
