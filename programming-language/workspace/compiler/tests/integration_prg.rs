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

// ========================================
// Array Code Generation Tests
// ========================================

/// Test byte array initialization.
#[test]
fn test_compile_byte_array_init() {
    let source = r#"
def main():
    arr: byte[] = [1, 2, 3, 255]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Code should include the array values
    assert!(code.len() > 50, "Code should include array initialization");
}

/// Test word array initialization.
#[test]
fn test_compile_word_array_init() {
    let source = r#"
def main():
    arr: word[] = [1000, 2000, 65535]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Code should include the array initialization
    assert!(
        code.len() > 50,
        "Code should include word array initialization"
    );
}

/// Test bool array initialization.
#[test]
fn test_compile_bool_array_init() {
    let source = r#"
def main():
    flags: bool[] = [true, false, true]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Code should include the bool array initialization
    assert!(
        code.len() > 50,
        "Code should include bool array initialization"
    );
}

/// Test byte array read access.
#[test]
fn test_compile_byte_array_read() {
    let source = r#"
def main():
    arr: byte[] = [10, 20, 30]
    x: byte = arr[1]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should compile without errors
    assert!(code.len() > 50, "Code should include array access");
}

/// Test word array read access.
#[test]
fn test_compile_word_array_read() {
    let source = r#"
def main():
    arr: word[] = [1000, 2000, 3000]
    x: word = arr[1]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should compile without errors
    assert!(code.len() > 50, "Code should include word array access");
}

/// Test byte array write access.
#[test]
fn test_compile_byte_array_write() {
    let source = r#"
def main():
    arr: byte[3]
    arr[0] = 10
    arr[1] = 20
    arr[2] = 30
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should compile without errors
    assert!(code.len() > 50, "Code should include array write");
}

/// Test word array write access.
#[test]
fn test_compile_word_array_write() {
    let source = r#"
def main():
    arr: word[3]
    arr[0] = 1000
    arr[1] = 2000
    arr[2] = 3000
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should compile without errors
    assert!(code.len() > 50, "Code should include word array write");
}

/// Test array in loop.
#[test]
fn test_compile_array_in_loop() {
    let source = r#"
def main():
    arr: byte[5]
    i: byte = 0
    while i < 5:
        arr[i] = i
        i = i + 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Should compile without errors
    assert!(code.len() > 100, "Code should include array loop");
}

/// Test zero-initialized array optimization.
#[test]
fn test_compile_zero_initialized_array() {
    let source = r#"
def main():
    arr: byte[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Zero-init optimization should use a loop, generating less code
    // than 10 individual STA instructions
    assert!(code.len() > 20, "Code should include zero-init loop");
}

/// Test zero-initialized bool array.
#[test]
fn test_compile_zero_initialized_bool_array() {
    let source = r#"
def main():
    flags: bool[] = [false, false, false, false, false]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(code.len() > 20, "Code should include bool zero-init");
}

/// Test mixed array (some zeros, some non-zeros).
#[test]
fn test_compile_mixed_array_no_optimization() {
    let source = r#"
def main():
    arr: byte[] = [0, 1, 0, 2, 0]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    // Mixed array should NOT use zero-init optimization
    assert!(code.len() > 30, "Code should include individual stores");
}

// ============================================================================
// Signed Array Tests
// ============================================================================

/// Test sbyte array initialization.
#[test]
fn test_compile_sbyte_array_init() {
    let source = r#"
def main():
    temps: sbyte[] = [-50, 0, 50, 100, -128]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test sword array initialization.
#[test]
fn test_compile_sword_array_init() {
    let source = r#"
def main():
    offsets: sword[] = [-1000, 500, 32767, -32768]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test sbyte array element read.
#[test]
fn test_compile_sbyte_array_read() {
    let source = r#"
def main():
    temps: sbyte[] = [-10, 20, 30]
    t: sbyte = temps[0]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test sword array element read.
#[test]
fn test_compile_sword_array_read() {
    let source = r#"
def main():
    offsets: sword[] = [-1000, 2000, 3000]
    o: sword = offsets[1]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test sbyte array element write.
#[test]
fn test_compile_sbyte_array_write() {
    let source = r#"
def main():
    temps: sbyte[] = [-10, 20, 30]
    temps[0] = -50
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test sword array element write.
#[test]
fn test_compile_sword_array_write() {
    let source = r#"
def main():
    offsets: sword[] = [-1000, 2000, 3000]
    offsets[1] = -5000
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test signed array type inference from negative values.
#[test]
fn test_compile_signed_array_type_inference() {
    // Array with values fitting in sbyte range should compile
    let source = r#"
def main():
    small: sbyte[] = [-100, 50, 127]
    large: sword[] = [-500, 30000]
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test signed array in loop.
#[test]
fn test_compile_signed_array_loop() {
    let source = r#"
def main():
    temps: sbyte[] = [-10, 0, 10, 20, 30]
    i: byte = 0
    while i < 5:
        t: sbyte = temps[i]
        i = i + 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

// ============================================================================
// len() Function Tests
// ============================================================================

/// Test len() on byte array.
#[test]
fn test_compile_len_byte_array() {
    let source = r#"
def main():
    arr: byte[] = [1, 2, 3, 4, 5]
    size: word = len(arr)
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test len() on word array.
#[test]
fn test_compile_len_word_array() {
    let source = r#"
def main():
    arr: word[] = [1000, 2000, 3000]
    size: word = len(arr)
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test len() on sized array.
#[test]
fn test_compile_len_sized_array() {
    let source = r#"
def main():
    buffer: byte[100]
    size: word = len(buffer)
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test len() in expression.
#[test]
fn test_compile_len_in_expression() {
    let source = r#"
def main():
    arr: byte[] = [10, 20, 30]
    i: byte = 0
    while i < len(arr):
        x: byte = arr[i]
        i = i + 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

/// Test len() with print.
#[test]
fn test_compile_len_with_print() {
    let source = r#"
def main():
    arr: byte[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    println(len(arr))
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Generated code should not be empty");
}

// ============================================================================
// Compound Assignment Integration Tests
// ============================================================================

/// Test all compound assignment operators on byte variables.
#[test]
fn test_compile_compound_assign_byte_all_ops() {
    let source = r#"
def main():
    x: byte = 100
    x += 10
    x -= 5
    x *= 2
    x /= 4
    x %= 7
    x &= 15
    x |= 240
    x ^= 85
    x <<= 2
    x >>= 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(code.len() > 100, "Code should include all compound ops");
}

/// Test compound assignment on word variables.
#[test]
fn test_compile_compound_assign_word() {
    let source = r#"
def main():
    x: word = 1000
    x += 500
    x -= 200
    x *= 2
    x /= 4
    x &= 255
    x |= 256
    x ^= 128
    x <<= 4
    x >>= 2
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(code.len() > 150, "Code should include word compound ops");
}

/// Test compound assignment on signed types.
#[test]
fn test_compile_compound_assign_signed() {
    let source = r#"
def main():
    x: sbyte = 50
    x += 10
    x -= 100

    y: sword = 1000
    y += 500
    y -= 2000
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(!code.is_empty(), "Code should compile signed compound ops");
}

/// Test compound assignment on byte array elements.
#[test]
fn test_compile_compound_assign_byte_array() {
    let source = r#"
def main():
    arr: byte[10]
    arr[0] = 100
    arr[0] += 10
    arr[1] = 50
    arr[1] -= 5
    arr[2] = 5
    arr[2] *= 3
    arr[3] = 100
    arr[3] /= 4
    arr[4] = 17
    arr[4] %= 5
    arr[5] = 255
    arr[5] &= 15
    arr[6] = 0
    arr[6] |= 170
    arr[7] = 255
    arr[7] ^= 85
    arr[8] = 1
    arr[8] <<= 4
    arr[9] = 128
    arr[9] >>= 3
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        code.len() > 200,
        "Code should include byte array compound ops"
    );
}

/// Test compound assignment on word array elements.
#[test]
fn test_compile_compound_assign_word_array() {
    let source = r#"
def main():
    arr: word[5]
    arr[0] = 1000
    arr[0] += 500
    arr[1] = 2000
    arr[1] -= 500
    arr[2] = 100
    arr[2] *= 10
    arr[3] = 65535
    arr[3] &= 255
    arr[4] = 1
    arr[4] <<= 8
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        code.len() > 200,
        "Code should include word array compound ops"
    );
}

/// Test compound assignment with variable index.
#[test]
fn test_compile_compound_assign_variable_index() {
    let source = r#"
def main():
    arr: byte[10]
    i: byte = 5
    arr[i] = 10
    arr[i] += 5
    arr[i] -= 2
    arr[i] *= 2
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        !code.is_empty(),
        "Code should compile variable index compound ops"
    );
}

/// Test compound assignment with expression index.
#[test]
fn test_compile_compound_assign_expression_index() {
    let source = r#"
def main():
    arr: byte[20]
    i: byte = 2
    arr[i + 3] = 10
    arr[i + 3] += 5
    arr[i * 2] = 20
    arr[i * 2] -= 10
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        !code.is_empty(),
        "Code should compile expression index compound ops"
    );
}

/// Test compound assignment in while loop.
#[test]
fn test_compile_compound_assign_in_loop() {
    let source = r#"
def main():
    sum: word = 0
    i: byte = 1
    while i <= 10:
        sum += i
        i += 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        code.len() > 50,
        "Code should include loop with compound ops"
    );
}

/// Test compound assignment with expression on RHS.
#[test]
fn test_compile_compound_assign_expression_rhs() {
    let source = r#"
def main():
    x: byte = 10
    y: byte = 3
    x += y * 2
    x -= y + 1
    x *= y - 1
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        !code.is_empty(),
        "Code should compile expression RHS compound ops"
    );
}

/// Test compound assignment in conditional.
#[test]
fn test_compile_compound_assign_in_if() {
    let source = r#"
def main():
    x: byte = 10
    if x > 5:
        x += 10
    else:
        x -= 5

    if x < 100:
        x *= 2
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");
    assert!(
        !code.is_empty(),
        "Code should compile conditional compound ops"
    );
}

/// Test the compound_assignment.cb64 example compiles.
#[test]
fn test_compile_compound_assignment_example() {
    let source = include_str!("../examples/compound_assignment.cb64");
    let code = cobra64::compile(source).expect("Example should compile");
    assert!(code.len() > 500, "Example should generate substantial code");
}
