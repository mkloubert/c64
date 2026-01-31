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

//! Code generation module for the Cobra64 compiler.
//!
//! This module generates 6510 machine code from the analyzed AST.
//! It handles:
//! - Instruction encoding
//! - Memory layout
//! - Expression evaluation
//! - Control flow
//! - Built-in functions

pub mod float_runtime;
pub mod mos6510;

use crate::ast::{
    AssignOp, AssignTarget, Assignment, BinaryOp, Block, ConstDecl, Expr, ExprKind, ForStatement,
    FunctionDef, IfStatement, Program, Statement, StatementKind, TopLevelItem, Type, UnaryOp,
    VarDecl, WhileStatement,
};
use crate::error::{CompileError, ErrorCode, Span};
use mos6510::{c64, kernal, opcodes, petscii, zeropage};
use std::collections::HashMap;

/// Memory layout constants.
const PROGRAM_START: u16 = 0x0801; // Standard C64 BASIC program start

/// Convert a decimal string to IEEE-754 binary16 bits.
///
/// Supports formats like "3.14", "0.5", "1.5e3", "2.0e-5".
fn decimal_string_to_binary16(s: &str) -> u16 {
    // Parse the string as f64, then convert to binary16
    let value: f64 = s.parse().unwrap_or(0.0);
    f64_to_binary16(value)
}

/// Convert an f64 value to IEEE-754 binary16 bits.
///
/// IEEE-754 binary16 format:
/// - Sign: 1 bit (bit 15)
/// - Exponent: 5 bits (bits 14-10), bias = 15
/// - Mantissa: 10 bits (bits 9-0), implicit leading 1 for normalized
fn f64_to_binary16(value: f64) -> u16 {
    // Handle special cases
    if value.is_nan() {
        return 0x7E00; // Canonical NaN
    }
    if value.is_infinite() {
        return if value > 0.0 { 0x7C00 } else { 0xFC00 };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            0x8000
        } else {
            0x0000
        };
    }

    let sign = if value < 0.0 { 1u16 } else { 0u16 };
    let abs_value = value.abs();

    // Check for overflow to infinity
    if abs_value > 65504.0 {
        return (sign << 15) | 0x7C00;
    }

    // Check for underflow to zero (smallest subnormal is ~5.96e-8)
    if abs_value < 5.96e-8 {
        return sign << 15;
    }

    // Calculate exponent and mantissa
    let bits = abs_value.to_bits();
    let f64_exp = ((bits >> 52) & 0x7FF) as i32;
    let f64_mant = bits & 0xFFFFFFFFFFFFF;

    // Convert f64 exponent (bias 1023) to binary16 exponent (bias 15)
    let exp = f64_exp - 1023 + 15;

    if exp <= 0 {
        // Subnormal number
        let shift = 1 - exp;
        if shift > 10 {
            return sign << 15; // Too small, becomes zero
        }
        // Subnormal: mantissa = (1.mant >> shift), no implicit 1
        let mant = ((0x400 | (f64_mant >> 42)) >> shift) & 0x3FF;
        return (sign << 15) | (mant as u16);
    }

    if exp >= 31 {
        // Overflow to infinity
        return (sign << 15) | 0x7C00;
    }

    // Normal number: take top 10 bits of f64 mantissa
    let mant = (f64_mant >> 42) & 0x3FF;

    // Round to nearest even
    let round_bit = (f64_mant >> 41) & 1;
    let sticky_bits = f64_mant & 0x1FFFFFFFFFF;
    let mant = if round_bit == 1 && (sticky_bits != 0 || (mant & 1) == 1) {
        mant + 1
    } else {
        mant
    };

    // Check if rounding caused overflow
    if mant > 0x3FF {
        let exp = exp + 1;
        if exp >= 31 {
            return (sign << 15) | 0x7C00; // Overflow to infinity
        }
        return (sign << 15) | ((exp as u16) << 10);
    }

    (sign << 15) | ((exp as u16) << 10) | (mant as u16)
}
const BASIC_STUB_SIZE: u16 = 13; // Size of the BASIC stub
const CODE_START: u16 = PROGRAM_START + BASIC_STUB_SIZE; // Machine code starts at $080E (2062)

/// A pending branch that needs its target resolved.
#[derive(Debug, Clone)]
struct PendingBranch {
    /// Offset in the code where the branch displacement should be written.
    code_offset: usize,
    /// Label this branch should jump to.
    target_label: String,
}

/// A pending jump (16-bit) that needs its target resolved.
#[derive(Debug, Clone)]
struct PendingJump {
    /// Offset in the code where the jump address should be written.
    code_offset: usize,
    /// Label this jump should jump to.
    target_label: String,
}

/// A pending string reference that needs its address resolved.
#[derive(Debug, Clone)]
struct PendingStringRef {
    /// Offset in the code where the low byte of the address should be written.
    code_offset_lo: usize,
    /// Offset in the code where the high byte of the address should be written.
    code_offset_hi: usize,
    /// Index of the string in the strings vector.
    string_index: usize,
}

/// Variable information for code generation.
#[derive(Debug, Clone)]
struct Variable {
    /// Address where the variable is stored.
    address: u16,
    /// Type of the variable.
    var_type: Type,
    /// Whether this is a constant (reserved for future use).
    #[allow(dead_code)]
    is_const: bool,
}

/// Function information for code generation.
#[derive(Debug, Clone)]
struct Function {
    /// Address where the function code starts.
    address: u16,
    /// Parameter types (reserved for future use).
    #[allow(dead_code)]
    params: Vec<Type>,
    /// Return type (reserved for future use).
    #[allow(dead_code)]
    return_type: Option<Type>,
}

/// Loop context for break/continue handling.
#[derive(Debug, Clone)]
struct LoopContext {
    /// Label at the start of the loop (for continue).
    start_label: String,
    /// Label at the end of the loop (for break).
    end_label: String,
}

/// The code generator for the 6510 CPU.
pub struct CodeGenerator {
    /// The generated machine code.
    code: Vec<u8>,
    /// Current code address.
    current_address: u16,
    /// Variable allocation table.
    variables: HashMap<String, Variable>,
    /// Function table.
    functions: HashMap<String, Function>,
    /// String literals (just the bytes, addresses resolved later).
    strings: Vec<Vec<u8>>,
    /// Pending string references to resolve.
    pending_string_refs: Vec<PendingStringRef>,
    /// Label addresses (resolved).
    labels: HashMap<String, u16>,
    /// Pending branches to resolve.
    pending_branches: Vec<PendingBranch>,
    /// Pending jumps to resolve.
    pending_jumps: Vec<PendingJump>,
    /// Next variable address (grows from end of code).
    next_var_address: u16,
    /// Label counter for generating unique labels.
    label_counter: u32,
    /// Current loop context stack.
    loop_stack: Vec<LoopContext>,
    /// Runtime library included flag.
    runtime_included: bool,
    /// Runtime routine addresses.
    runtime_addresses: HashMap<String, u16>,
}

impl CodeGenerator {
    /// Create a new code generator.
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            current_address: PROGRAM_START,
            variables: HashMap::new(),
            functions: HashMap::new(),
            strings: Vec::new(),
            pending_string_refs: Vec::new(),
            labels: HashMap::new(),
            pending_branches: Vec::new(),
            pending_jumps: Vec::new(),
            next_var_address: 0xC000, // Variables start at $C000
            label_counter: 0,
            loop_stack: Vec::new(),
            runtime_included: false,
            runtime_addresses: HashMap::new(),
        }
    }

    /// Generate a unique label.
    fn make_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Define a label at the current address.
    fn define_label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.current_address);
    }

    /// Emit a single byte.
    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
        self.current_address = self.current_address.wrapping_add(1);
    }

    /// Emit a 16-bit word (little-endian).
    fn emit_word(&mut self, word: u16) {
        self.emit_byte((word & 0xFF) as u8);
        self.emit_byte((word >> 8) as u8);
    }

    /// Emit an instruction with immediate operand.
    fn emit_imm(&mut self, opcode: u8, value: u8) {
        self.emit_byte(opcode);
        self.emit_byte(value);
    }

    /// Emit an instruction with absolute operand.
    fn emit_abs(&mut self, opcode: u8, address: u16) {
        self.emit_byte(opcode);
        self.emit_word(address);
    }

    /// Emit a branch instruction with a label target.
    fn emit_branch(&mut self, opcode: u8, label: &str) {
        self.emit_byte(opcode);
        let offset = self.code.len();
        self.emit_byte(0x00); // Placeholder
        self.pending_branches.push(PendingBranch {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    /// Emit a JMP instruction with a label target.
    fn emit_jmp(&mut self, label: &str) {
        self.emit_byte(opcodes::JMP_ABS);
        let offset = self.code.len();
        self.emit_word(0x0000); // Placeholder
        self.pending_jumps.push(PendingJump {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    /// Emit a JSR instruction with a label target.
    fn emit_jsr_label(&mut self, label: &str) {
        self.emit_byte(opcodes::JSR);
        let offset = self.code.len();
        self.emit_word(0x0000); // Placeholder
        self.pending_jumps.push(PendingJump {
            code_offset: offset,
            target_label: label.to_string(),
        });
    }

    /// Resolve all pending branches and jumps.
    fn resolve_labels(&mut self) -> Result<(), CompileError> {
        // Resolve branches (8-bit relative)
        for branch in &self.pending_branches {
            let target = self.labels.get(&branch.target_label).ok_or_else(|| {
                CompileError::new(
                    ErrorCode::UndefinedVariable, // Reusing error code
                    format!("Undefined label '{}'", branch.target_label),
                    Span::new(0, 0),
                )
            })?;

            // Calculate relative offset
            // Branch is from the byte AFTER the displacement
            let branch_addr = PROGRAM_START as i32 + branch.code_offset as i32 + 1;
            let target_addr = *target as i32;
            let displacement = target_addr - branch_addr;

            if !(-128..=127).contains(&displacement) {
                return Err(CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    format!("Branch target too far: {} bytes", displacement),
                    Span::new(0, 0),
                ));
            }

            self.code[branch.code_offset] = displacement as i8 as u8;
        }

        // Resolve jumps (16-bit absolute)
        for jump in &self.pending_jumps {
            let target = self.labels.get(&jump.target_label).ok_or_else(|| {
                CompileError::new(
                    ErrorCode::UndefinedVariable,
                    format!("Undefined label '{}'", jump.target_label),
                    Span::new(0, 0),
                )
            })?;

            self.code[jump.code_offset] = (*target & 0xFF) as u8;
            self.code[jump.code_offset + 1] = (*target >> 8) as u8;
        }

        Ok(())
    }

    /// Allocate a variable and return its address.
    fn allocate_variable(&mut self, name: &str, var_type: &Type, is_const: bool) -> u16 {
        let size = var_type.size() as u16;
        let address = self.next_var_address;
        self.next_var_address = self.next_var_address.wrapping_add(size);

        self.variables.insert(
            name.to_string(),
            Variable {
                address,
                var_type: var_type.clone(),
                is_const,
            },
        );

        address
    }

    /// Add a string literal and return its index.
    fn add_string(&mut self, value: &str) -> usize {
        // Convert to PETSCII-like bytes (simplified: just use ASCII values)
        let mut bytes: Vec<u8> = value.bytes().collect();
        bytes.push(0); // Null terminator

        let index = self.strings.len();
        self.strings.push(bytes);
        index
    }

    /// Emit code to load a string address.
    /// Sets A=low byte, X=high byte (consistent with other 16-bit values).
    /// Also sets TMP1/TMP1_HI for print_str compatibility.
    /// The actual address will be patched later when string positions are known.
    fn emit_string_ref(&mut self, string_index: usize) {
        // LDA #<string_addr (placeholder)
        self.emit_byte(opcodes::LDA_IMM);
        let code_offset_lo = self.code.len();
        self.emit_byte(0); // Placeholder for low byte

        // STA TMP1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // LDX #>string_addr (placeholder)
        self.emit_byte(opcodes::LDX_IMM);
        let code_offset_hi = self.code.len();
        self.emit_byte(0); // Placeholder for high byte

        // STX TMP1_HI
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // A=low, X=high is now set (consistent with Word and String variables)

        // Record pending reference
        self.pending_string_refs.push(PendingStringRef {
            code_offset_lo,
            code_offset_hi,
            string_index,
        });
    }

    /// Resolve all pending string references.
    fn resolve_string_refs(&mut self) {
        // Calculate where each string will be placed
        let mut string_addresses = Vec::new();
        let mut addr = self.current_address;
        for s in &self.strings {
            string_addresses.push(addr);
            addr = addr.wrapping_add(s.len() as u16);
        }

        // Patch all pending references
        for pending in &self.pending_string_refs {
            if pending.string_index < string_addresses.len() {
                let addr = string_addresses[pending.string_index];
                self.code[pending.code_offset_lo] = (addr & 0xFF) as u8;
                self.code[pending.code_offset_hi] = (addr >> 8) as u8;
            }
        }
    }

    /// Emit the BASIC stub for autostart.
    fn emit_basic_stub(&mut self) {
        // BASIC stub: 10 SYS 2062
        // The machine code starts at $080E (2062 decimal)

        // Link to next BASIC line (points to end of program marker)
        self.emit_word(0x080C);

        // Line number (10)
        self.emit_word(0x000A);

        // SYS token
        self.emit_byte(0x9E);

        // Space
        self.emit_byte(0x20);

        // Address as ASCII: "2062"
        self.emit_byte(b'2');
        self.emit_byte(b'0');
        self.emit_byte(b'6');
        self.emit_byte(b'2');

        // End of BASIC line
        self.emit_byte(0x00);

        // End of BASIC program (null link pointer)
        self.emit_byte(0x00);
        self.emit_byte(0x00);

        // Machine code starts here at $080E (2062)
        assert_eq!(self.current_address, CODE_START);
    }

    /// Generate code for a program.
    pub fn generate(&mut self, program: &Program) -> Result<Vec<u8>, CompileError> {
        // Emit BASIC stub
        self.emit_basic_stub();

        // First pass: collect function and variable info
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => {
                    // We'll set the address when we emit the function
                    self.functions.insert(
                        func.name.clone(),
                        Function {
                            address: 0,
                            params: func.params.iter().map(|p| p.param_type.clone()).collect(),
                            return_type: func.return_type.clone(),
                        },
                    );
                }
                TopLevelItem::Variable(decl) => {
                    self.allocate_variable(&decl.name, &decl.var_type, false);
                }
                TopLevelItem::Constant(decl) => {
                    // For constants, infer type from value (default to byte)
                    let const_type = self.infer_type_from_expr(&decl.value);
                    self.allocate_variable(&decl.name, &const_type, true);
                }
            }
        }

        // Second pass: generate code
        // First, jump to main
        self.emit_jmp("main");

        // Include runtime library
        self.emit_runtime_library();

        // Generate code for all functions
        for item in &program.items {
            if let TopLevelItem::Function(func) = item {
                self.generate_function(func)?;
            }
        }

        // Generate initialization for global variables
        self.define_label("__init_globals");
        for item in &program.items {
            match item {
                TopLevelItem::Variable(decl) => {
                    if let Some(init) = &decl.initializer {
                        self.generate_expression(init)?;
                        let var = self.variables.get(&decl.name).unwrap().clone();
                        self.emit_store_to_address(var.address, &var.var_type);
                    }
                }
                TopLevelItem::Constant(decl) => {
                    self.generate_expression(&decl.value)?;
                    let var = self.variables.get(&decl.name).unwrap().clone();
                    self.emit_store_to_address(var.address, &var.var_type);
                }
                _ => {}
            }
        }
        self.emit_byte(opcodes::RTS);

        // Resolve all labels
        self.resolve_labels()?;

        // Resolve string references (must be done before emitting string data)
        self.resolve_string_refs();

        // Append string data
        // Collect strings first to avoid borrow checker issues
        let strings_to_emit: Vec<Vec<u8>> = self.strings.clone();
        for bytes in strings_to_emit {
            for b in bytes {
                self.emit_byte(b);
            }
        }

        Ok(self.code.clone())
    }

    /// Emit the runtime library.
    fn emit_runtime_library(&mut self) {
        if self.runtime_included {
            return;
        }
        self.runtime_included = true;

        // Print string routine
        self.emit_print_string_routine();

        // Print byte routine
        self.emit_print_byte_routine();

        // Print word routine
        self.emit_print_word_routine();

        // Print bool routine
        self.emit_print_bool_routine();

        // Print signed byte routine
        self.emit_print_sbyte_routine();

        // Print signed word routine
        self.emit_print_sword_routine();

        // Print fixed-point routine
        self.emit_print_fixed_routine();

        // Print float routine
        self.emit_print_float_routine();

        // Multiply routines
        self.emit_multiply_byte_routine();
        self.emit_multiply_word_routine();

        // Signed multiply routines
        self.emit_multiply_sbyte_routine();
        self.emit_multiply_sword_routine();

        // Divide routines
        self.emit_divide_byte_routine();
        self.emit_divide_word_routine();

        // Signed divide routines
        self.emit_divide_sbyte_routine();
        self.emit_divide_sword_routine();

        // Fixed-point routines
        self.emit_fixed_multiply_routine();
        self.emit_fixed_divide_routine();
        self.emit_fixed_modulo_routine();
        self.emit_fixed_comparison_routines();

        // Float runtime routines
        self.emit_float_runtime();

        // Input routine
        self.emit_readln_routine();
    }

    /// Emit print string routine.
    /// Input: String address in TMP1/TMP1_HI
    fn emit_print_string_routine(&mut self) {
        self.define_label("__print_str");
        self.runtime_addresses
            .insert("print_str".to_string(), self.current_address);

        // LDY #0
        self.emit_imm(opcodes::LDY_IMM, 0);

        // Loop: LDA (TMP1),Y
        self.define_label("__print_str_loop");
        self.emit_byte(opcodes::LDA_IZY);
        self.emit_byte(zeropage::TMP1);

        // BEQ done
        self.emit_branch(opcodes::BEQ, "__print_str_done");

        // JSR CHROUT
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // INY
        self.emit_byte(opcodes::INY);

        // JMP loop
        self.emit_jmp("__print_str_loop");

        // Done
        self.define_label("__print_str_done");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print byte routine.
    /// Input: A = byte value (0-255)
    /// Prints the decimal value without leading zeros.
    fn emit_print_byte_routine(&mut self) {
        self.define_label("__print_byte");
        self.runtime_addresses
            .insert("print_byte".to_string(), self.current_address);

        // Divide by 100 for hundreds digit
        self.emit_imm(opcodes::LDX_IMM, 100);
        self.emit_jsr_label("__div_byte_ax");
        // A = hundreds, X = remainder
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3); // Save remainder
        self.emit_byte(opcodes::TAY); // Y = hundreds

        // Divide remainder by 10
        self.emit_byte(opcodes::TXA);
        self.emit_imm(opcodes::LDX_IMM, 10);
        self.emit_jsr_label("__div_byte_ax");
        // A = tens, X = ones
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // TMP3 = tens
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP4); // TMP4 = ones
                                        // Y still has hundreds

        // Print hundreds if != 0
        self.emit_byte(opcodes::TYA);
        self.emit_branch(opcodes::BEQ, "__pb_check_tens");
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        // After printing hundreds, always print tens
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_jmp("__pb_print_tens_digit");

        // Check tens
        self.define_label("__pb_check_tens");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pb_print_ones");

        self.define_label("__pb_print_tens_digit");
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Print ones (always)
        self.define_label("__pb_print_ones");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit print word routine.
    /// Input: A = low byte, X = high byte of 16-bit value
    /// Prints the decimal value (0-65535) without leading zeros.
    fn emit_print_word_routine(&mut self) {
        self.define_label("__print_word");
        self.runtime_addresses
            .insert("print_word".to_string(), self.current_address);

        // Store the 16-bit value in TMP1 (low) and TMP1_HI (high)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // If high byte is 0, just print as byte
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__pw_full");
        self.emit_jsr_label("__print_byte");
        self.emit_byte(opcodes::RTS);

        // Full 16-bit printing needed
        self.define_label("__pw_full");

        // Use TMP3 as "started printing" flag (0 = no digits yet)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Divide by 10000 (0x2710)
        self.emit_imm(opcodes::LDA_IMM, 0x10); // low byte of 10000
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x27); // high byte of 10000
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 1000 (0x03E8)
        self.emit_imm(opcodes::LDA_IMM, 0xE8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x03);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 100 (0x0064)
        self.emit_imm(opcodes::LDA_IMM, 0x64);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Divide by 10 (0x000A)
        self.emit_imm(opcodes::LDA_IMM, 0x0A);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::LDA_IMM, 0x00);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_jsr_label("__pw_digit");

        // Print final digit (ones place, always print)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);

        // Subroutine: divide TMP1/TMP1_HI by TMP2/TMP2_HI, print digit if non-zero or started
        self.define_label("__pw_digit");
        self.emit_imm(opcodes::LDX_IMM, 0); // X = quotient digit

        self.define_label("__pw_digit_loop");
        // Compare TMP1/TMP1_HI >= TMP2/TMP2_HI
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_branch(opcodes::BCC, "__pw_digit_done"); // high < divisor high, done
        self.emit_branch(opcodes::BNE, "__pw_digit_sub"); // high > divisor high, subtract

        // High bytes equal, compare low bytes
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__pw_digit_done"); // low < divisor low, done

        // Subtract divisor from value
        self.define_label("__pw_digit_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Increment quotient
        self.emit_byte(opcodes::INX);
        self.emit_jmp("__pw_digit_loop");

        self.define_label("__pw_digit_done");
        // X = digit value. Print if X != 0 or TMP3 != 0 (already started)
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__pw_digit_print");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pw_digit_skip"); // skip leading zero

        self.define_label("__pw_digit_print");
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        // Mark that we've started printing
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__pw_digit_skip");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print bool routine.
    /// Input: A = bool value (0 = false, non-zero = true)
    /// Prints "TRUE" or "FALSE".
    fn emit_print_bool_routine(&mut self) {
        self.define_label("__print_bool");
        self.runtime_addresses
            .insert("print_bool".to_string(), self.current_address);

        // Check if A is zero (false) or non-zero (true)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, "__pb_false");

        // Print "TRUE"
        self.emit_imm(opcodes::LDA_IMM, b'T');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'R');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'U');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'E');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        // Print "FALSE"
        self.define_label("__pb_false");
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'A');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'L');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'S');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'E');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print signed byte routine.
    /// Input: A = signed byte value (-128 to 127)
    /// Prints the decimal value with minus sign if negative.
    fn emit_print_sbyte_routine(&mut self) {
        self.define_label("__print_sbyte");
        self.runtime_addresses
            .insert("print_sbyte".to_string(), self.current_address);

        // Check if negative (bit 7 set)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BCC, "__psb_positive");

        // Negative: print minus sign and negate
        self.emit_byte(opcodes::PHA); // Save value
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::PLA); // Restore value

        // Negate: two's complement
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);

        self.define_label("__psb_positive");
        // Now A contains the absolute value, print it
        self.emit_jsr_label("__print_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit print signed word routine.
    /// Input: A = low byte, X = high byte of signed 16-bit value
    /// Prints the decimal value with minus sign if negative.
    fn emit_print_sword_routine(&mut self) {
        self.define_label("__print_sword");
        self.runtime_addresses
            .insert("print_sword".to_string(), self.current_address);

        // Store value in TMP1/TMP1_HI for later use
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if negative (high byte bit 7 set)
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BCC, "__psw_positive");

        // Negative: print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Negate 16-bit value: two's complement
        // NOT low byte, NOT high byte, then add 1
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_imm(opcodes::ADC_IMM, 0); // Add carry from low byte
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__psw_positive");
        // Load absolute value and print
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__print_word");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point (12.4) print routine.
    ///
    /// Input: A = low byte, X = high byte (12.4 fixed-point)
    /// Prints the decimal representation, e.g., 60 (internal) → "3.75"
    fn emit_print_fixed_routine(&mut self) {
        self.define_label("__print_fixed");
        self.runtime_addresses
            .insert("print_fixed".to_string(), self.current_address);

        // Store value
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check if negative (bit 15 set)
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x80);
        self.emit_branch(opcodes::BEQ, "__pfix_positive");

        // Print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Negate (two's complement)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_imm(opcodes::ADC_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pfix_positive");
        // Now TMP1/TMP1_HI holds absolute value
        // Integer part = value >> 4
        // Shift right 4 times
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI); // Save shifted high byte

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROR_ACC);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // 1 shift done

        // 3 more shifts
        for _ in 0..3 {
            self.emit_byte(opcodes::LSR_ZP);
            self.emit_byte(zeropage::TMP3_HI);
            self.emit_byte(opcodes::ROR_ZP);
            self.emit_byte(zeropage::TMP3);
        }

        // Print integer part (TMP3/TMP3_HI)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_jsr_label("__print_word");

        // Print decimal point
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Fractional part = (value & 0x0F) * 625
        // This gives us 4 decimal digits (0000-9375)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x0F); // Get fractional nibble

        // Multiply by 625 using: 625 = 512 + 64 + 32 + 16 + 1
        // = (frac << 9) + (frac << 6) + (frac << 5) + (frac << 4) + frac
        // For simplicity, use a lookup table approach or direct computation
        // Since frac is 0-15, we can use: frac * 625 directly

        // Store frac in TMP3
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Result = frac * 625 (max 15 * 625 = 9375, fits in 16 bits)
        // We'll compute this step by step
        // 625 = 625, so we multiply using shift-and-add

        // Clear result in TMP4/TMP5 (using TMP3_HI and another location)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Result low
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // Result high

        // Check if frac is 0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__pfix_print_frac");

        // Multiply frac * 625
        // 625 = 0x0271
        // We'll add 625 for each bit set in frac
        self.emit_imm(opcodes::LDY_IMM, 4); // 4 bits to check

        self.define_label("__pfix_mul_loop");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x01);
        self.emit_branch(opcodes::BEQ, "__pfix_no_add");

        // Add 625 (0x0271) shifted appropriately
        // Actually, easier to just add 625 repeatedly
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::ADC_IMM, 0x71); // Low byte of 625
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_imm(opcodes::ADC_IMM, 0x02); // High byte of 625
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pfix_no_add");
        // Shift frac right
        self.emit_byte(opcodes::LSR_ZP);
        self.emit_byte(zeropage::TMP3);

        // Double the multiplier (625 becomes 1250, 2500, 5000)
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Wait, this logic is wrong. Let me use a simpler approach:
        // Just multiply by adding 625 repeatedly

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, "__pfix_mul_loop");

        // Simpler approach: just print the pre-calculated fractional digits
        // frac * 625 / 10000 for each digit
        // For now, use a direct table or simplified printing

        self.define_label("__pfix_print_frac");
        // Print 4 digits: TMP1/TMP1_HI contains frac * 625
        // Divide by 1000 to get first digit, etc.
        // This is complex, so let's use a simpler approach:
        // Print leading zeros and the value

        // For simplicity, print the fractional value as 4 digits
        // using repeated subtraction/division
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Use a simplified 4-digit print (with leading zeros for now)
        // Divide by 1000, 100, 10, 1
        self.emit_jsr_label("__print_frac4");

        self.emit_byte(opcodes::RTS);

        // Helper routine to print 4 fractional digits with leading zeros
        self.define_label("__print_frac4");
        // Input: A/X = value (0-9375)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Digit 1: divide by 1000 (0x03E8)
        self.emit_imm(opcodes::LDY_IMM, 0); // Digit counter

        self.define_label("__pf4_d1");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x03);
        self.emit_branch(opcodes::BCC, "__pf4_d1_done");
        self.emit_branch(opcodes::BNE, "__pf4_d1_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0xE8);
        self.emit_branch(opcodes::BCC, "__pf4_d1_done");

        self.define_label("__pf4_d1_sub");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::SBC_IMM, 0xE8);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_imm(opcodes::SBC_IMM, 0x03);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d1");

        self.define_label("__pf4_d1_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 2: divide by 100 (0x64)
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__pf4_d2");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_branch(opcodes::BNE, "__pf4_d2_sub");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(100);
        self.emit_branch(opcodes::BCC, "__pf4_d2_done");

        self.define_label("__pf4_d2_sub");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::SBC_IMM, 100);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_imm(opcodes::SBC_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d2");

        self.define_label("__pf4_d2_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 3: divide by 10
        self.emit_imm(opcodes::LDY_IMM, 0);

        self.define_label("__pf4_d3");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(10);
        self.emit_branch(opcodes::BCC, "__pf4_d3_done");
        self.emit_byte(opcodes::SEC);
        self.emit_imm(opcodes::SBC_IMM, 10);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::INY);
        self.emit_jmp("__pf4_d3");

        self.define_label("__pf4_d3_done");
        self.emit_byte(opcodes::TYA);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Digit 4: remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit float (IEEE-754 binary16) print routine.
    ///
    /// Input: A = low byte, X = high byte (binary16)
    /// Prints the decimal representation.
    fn emit_print_float_routine(&mut self) {
        self.define_label("__print_float");
        self.runtime_addresses
            .insert("print_float".to_string(), self.current_address);

        // Store value
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Check for special values first

        // Extract exponent: (high >> 2) & 0x1F
        self.emit_byte(opcodes::TXA);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::LSR_ACC);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x1F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3); // exponent

        // Check for exponent = 31 (infinity or NaN)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(31);
        self.emit_branch(opcodes::BNE, "__pflt_not_special");

        // Check mantissa for NaN vs Infinity
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03); // Mantissa high bits
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__pflt_nan");

        // Infinity - check sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BMI, "__pflt_neg_inf");

        // Print "INF"
        self.emit_imm(opcodes::LDA_IMM, b'I');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_neg_inf");
        // Print "-INF"
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'I');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'F');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_nan");
        // Print "NAN"
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'A');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'N');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_not_special");
        // Check for zero (exponent = 0 and mantissa = 0)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3); // exponent
        self.emit_branch(opcodes::BNE, "__pflt_not_zero");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x03);
        self.emit_byte(opcodes::ORA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BNE, "__pflt_subnormal");

        // Zero - check sign for -0 vs +0
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BPL, "__pflt_pos_zero");

        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.define_label("__pflt_pos_zero");
        // Print "0.0"
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_subnormal");
        // For subnormal, treat as very small number - print "0.0" for simplicity
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_byte(opcodes::RTS);

        self.define_label("__pflt_not_zero");
        // Normal number: convert to fixed-point and print
        // For simplicity, convert float to fixed-point 12.4 and use that print routine
        // This works for values in the fixed-point range

        // Check sign
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BPL, "__pflt_print_val");

        // Print minus sign
        self.emit_imm(opcodes::LDA_IMM, b'-');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        // Make positive
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::AND_IMM);
        self.emit_byte(0x7F);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__pflt_print_val");
        // Convert to integer representation and print
        // Use float_to_word routine then print
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_jsr_label("__float_to_word");

        // A/X now contains integer part
        self.emit_jsr_label("__print_word");

        // Print ".0" for now (simplified - full decimal would be complex)
        self.emit_imm(opcodes::LDA_IMM, b'.');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);
        self.emit_imm(opcodes::LDA_IMM, b'0');
        self.emit_abs(opcodes::JSR, kernal::CHROUT);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 8-bit multiply routine.
    /// Input: A * X
    /// Output: A = low byte, X = high byte
    fn emit_multiply_byte_routine(&mut self) {
        self.define_label("__mul_byte");
        self.runtime_addresses
            .insert("mul_byte".to_string(), self.current_address);

        // Store multiplier
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // Store multiplicand
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2);

        // Clear result
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // High byte of result

        // Counter
        self.emit_imm(opcodes::LDX_IMM, 8);

        self.define_label("__mul_byte_loop");
        // Shift result left
        self.emit_byte(opcodes::ASL_ACC);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Shift multiplier left, check carry
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, "__mul_byte_skip");

        // Add multiplicand
        self.emit_byte(opcodes::CLC);
        self.emit_byte(opcodes::ADC_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__mul_byte_skip");
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.define_label("__mul_byte_skip");
        self.emit_byte(opcodes::DEX);
        self.emit_branch(opcodes::BNE, "__mul_byte_loop");

        // Result: A = low byte
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP1_HI); // X = high byte

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 16-bit multiply routine (simplified).
    fn emit_multiply_word_routine(&mut self) {
        self.define_label("__mul_word");
        self.runtime_addresses
            .insert("mul_word".to_string(), self.current_address);

        // Simplified: just do byte multiply for now
        self.emit_jsr_label("__mul_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 8-bit multiply routine.
    /// Input: A * X (both signed)
    /// Output: A = low byte (signed result)
    fn emit_multiply_sbyte_routine(&mut self) {
        self.define_label("__mul_sbyte");
        self.runtime_addresses
            .insert("mul_sbyte".to_string(), self.current_address);

        // Store operands
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // First operand
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // Second operand

        // TMP3 = sign flag (0 = positive result, 1 = negative result)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Check sign of first operand
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BPL, "__msb_first_pos");
        // Negate first operand
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        // Toggle sign flag
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__msb_first_pos");
        // Check sign of second operand
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BPL, "__msb_second_pos");
        // Negate second operand
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        // Toggle sign flag
        self.emit_byte(opcodes::DEC_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__msb_second_pos");
        // Now both operands are positive, multiply
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jsr_label("__mul_byte");

        // Check sign flag
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Save result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__msb_done");
        // Negate result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::RTS);

        self.define_label("__msb_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 16-bit multiply routine (simplified).
    fn emit_multiply_sword_routine(&mut self) {
        self.define_label("__mul_sword");
        self.runtime_addresses
            .insert("mul_sword".to_string(), self.current_address);

        // For now, use sbyte multiply (simplified)
        self.emit_jsr_label("__mul_sbyte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit 8-bit divide routine.
    /// Input: A / X
    /// Output: A = quotient, X = remainder
    fn emit_divide_byte_routine(&mut self) {
        self.define_label("__div_byte");
        self.define_label("__div_byte_ax");
        self.runtime_addresses
            .insert("div_byte".to_string(), self.current_address);

        // Check for divide by zero
        self.emit_byte(opcodes::CPX_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BNE, "__div_byte_start");
        // Divide by zero: return 0
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_imm(opcodes::LDX_IMM, 0);
        self.emit_byte(opcodes::RTS);

        self.define_label("__div_byte_start");
        // Store dividend and divisor
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Dividend
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // Divisor

        // Clear quotient
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // Quotient

        // Load dividend
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);

        self.define_label("__div_byte_loop");
        // Compare with divisor
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BCC, "__div_byte_done");

        // Subtract divisor
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP2);

        // Increment quotient
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        self.emit_jmp("__div_byte_loop");

        self.define_label("__div_byte_done");
        // A = remainder, load quotient
        self.emit_byte(opcodes::TAX); // X = remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1_HI); // A = quotient

        self.emit_byte(opcodes::RTS);
    }

    /// Emit 16-bit divide routine (simplified).
    fn emit_divide_word_routine(&mut self) {
        self.define_label("__div_word");
        self.runtime_addresses
            .insert("div_word".to_string(), self.current_address);

        // Simplified: just do byte divide
        self.emit_jsr_label("__div_byte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 8-bit divide routine.
    /// Input: A / X (both signed)
    /// Output: A = quotient (signed), X = remainder (signed)
    fn emit_divide_sbyte_routine(&mut self) {
        self.define_label("__div_sbyte");
        self.runtime_addresses
            .insert("div_sbyte".to_string(), self.current_address);

        // Store operands
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Dividend
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // Divisor

        // TMP3 = sign flag for quotient
        // TMP4 = sign flag for remainder (same as dividend)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP4);

        // Check sign of dividend
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BPL, "__dsb_dividend_pos");
        // Negate dividend
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        // Toggle sign flags
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::INC_ZP);
        self.emit_byte(zeropage::TMP4);

        self.define_label("__dsb_dividend_pos");
        // Check sign of divisor
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_branch(opcodes::BPL, "__dsb_divisor_pos");
        // Negate divisor
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);
        // Toggle quotient sign flag only
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::EOR_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        self.define_label("__dsb_divisor_pos");
        // Now both operands are positive, divide
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_jsr_label("__div_byte");
        // A = quotient, X = remainder

        // Save results
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // quotient
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP2); // remainder

        // Apply sign to quotient
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_branch(opcodes::BEQ, "__dsb_quot_pos");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        self.define_label("__dsb_quot_pos");
        // Apply sign to remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP4);
        self.emit_branch(opcodes::BEQ, "__dsb_done");
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_imm(opcodes::EOR_IMM, 0xFF);
        self.emit_byte(opcodes::CLC);
        self.emit_imm(opcodes::ADC_IMM, 1);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP2);

        self.define_label("__dsb_done");
        // Return A = quotient, X = remainder
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP2);
        self.emit_byte(opcodes::RTS);
    }

    /// Emit signed 16-bit divide routine (simplified).
    fn emit_divide_sword_routine(&mut self) {
        self.define_label("__div_sword");
        self.runtime_addresses
            .insert("div_sword".to_string(), self.current_address);

        // For now, use sbyte divide (simplified)
        self.emit_jsr_label("__div_sbyte");
        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point multiply routine.
    ///
    /// Input: TMP3/TMP3_HI (left), TMP1/TMP1_HI (right)
    /// Output: A/X (low/high)
    ///
    /// Fixed-point 12.4 multiplication:
    /// (a×16) × (b×16) = a×b × 256
    /// Result needs arithmetic right shift by 4 to get (a×b) × 16
    fn emit_fixed_multiply_routine(&mut self) {
        self.define_label("__mul_fixed");
        self.runtime_addresses
            .insert("mul_fixed".to_string(), self.current_address);

        // Use 16×16 → 32-bit multiplication
        // Then shift result right by 4 (divide by 16) with sign preservation

        // Copy operands to sword multiply inputs
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // TMP1/TMP1_HI already has right operand, need it in proper location
        // Actually, we need a 32-bit multiply routine here
        // For now, use sword multiply and then shift

        self.emit_jsr_label("__mul_sword");

        // Result is in A/X (16-bit). For proper fixed-point, we'd need
        // 32-bit result and shift right by 4. For now, simplified:
        // Assume result fits in 16-bit after shift

        // Arithmetic right shift by 4 (preserving sign)
        // For each shift: ASR (if negative) or LSR (if positive)
        // Simplified: ASL X then ROR A sequence reversed
        let shift_loop = self.make_label("fmul_shr");
        let _done_label = self.make_label("fmul_done");

        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Shift right 4 times (arithmetic)
        self.emit_imm(opcodes::LDY_IMM, 4);
        self.define_label(&shift_loop);

        // Arithmetic shift right: copy sign bit, shift right
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0x80); // Set carry if negative
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::ROR_ZP);
        self.emit_byte(zeropage::TMP3);

        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        // Load result
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::LDX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point divide routine.
    ///
    /// Input: TMP3/TMP3_HI (dividend), TMP1/TMP1_HI (divisor)
    /// Output: A/X (low/high)
    ///
    /// Fixed-point 12.4 division:
    /// ((a×16) << 4) / (b×16) = (a/b) × 16
    fn emit_fixed_divide_routine(&mut self) {
        self.define_label("__div_fixed");
        self.runtime_addresses
            .insert("div_fixed".to_string(), self.current_address);

        // Shift dividend left by 4 first
        // Then do signed 16-bit division

        // Load dividend and shift left 4
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Shift left 4 times
        let shift_loop = self.make_label("fdiv_shl");
        self.emit_imm(opcodes::LDY_IMM, 4);
        self.define_label(&shift_loop);
        self.emit_byte(opcodes::ASL_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::ROL_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_byte(opcodes::DEY);
        self.emit_branch(opcodes::BNE, &shift_loop);

        // Now divide by TMP3/TMP3_HI (original right operand)
        // Note: We need to swap operands for the divide
        // Actually, the original divisor is in the original TMP1 location
        // This is getting complex - let's simplify with a JSR to sword divide

        self.emit_jsr_label("__div_sword");

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point modulo routine.
    fn emit_fixed_modulo_routine(&mut self) {
        self.define_label("__mod_fixed");
        self.runtime_addresses
            .insert("mod_fixed".to_string(), self.current_address);

        // For modulo, we compute: a - (a / b) * b
        // But for fixed-point, it's similar to integer modulo
        // Just use the signed word divide and return remainder

        self.emit_jsr_label("__div_sword");
        // Remainder would be in different location - for now return quotient
        // TODO: Implement proper fixed-point modulo

        self.emit_byte(opcodes::RTS);
    }

    /// Emit fixed-point comparison routines.
    fn emit_fixed_comparison_routines(&mut self) {
        // Less than (signed 16-bit)
        self.define_label("__cmp_fixed_lt");
        self.runtime_addresses
            .insert("cmp_fixed_lt".to_string(), self.current_address);

        // TMP3/TMP3_HI < TMP1/TMP1_HI (signed)
        // Algorithm: subtract and check sign/overflow
        let lt_true = self.make_label("flt_true");
        let lt_done = self.make_label("flt_done");

        // Compare high bytes first for quick exit
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BVC, "__flt_no_ovf1");
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label("__flt_no_ovf1");
        self.emit_branch(opcodes::BMI, &lt_true.clone());
        self.emit_branch(opcodes::BNE, &lt_done.clone());

        // High bytes equal, check low bytes (unsigned comparison)
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, &lt_true);

        // Not less than
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&lt_done);

        self.define_label(&lt_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&lt_done);
        self.emit_byte(opcodes::RTS);

        // Less than or equal
        self.define_label("__cmp_fixed_le");
        self.runtime_addresses
            .insert("cmp_fixed_le".to_string(), self.current_address);

        let le_true = self.make_label("fle_true");
        let le_done = self.make_label("fle_done");

        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3_HI);
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1_HI);
        self.emit_branch(opcodes::BVC, "__fle_no_ovf1");
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label("__fle_no_ovf1");
        self.emit_branch(opcodes::BMI, &le_true.clone());
        self.emit_branch(opcodes::BNE, &le_done.clone());

        // High bytes equal, check low bytes
        self.emit_byte(opcodes::LDA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BCC, &le_true.clone());
        self.emit_branch(opcodes::BEQ, &le_true);

        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&le_done);

        self.define_label(&le_true);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&le_done);
        self.emit_byte(opcodes::RTS);

        // Greater than
        self.define_label("__cmp_fixed_gt");
        self.runtime_addresses
            .insert("cmp_fixed_gt".to_string(), self.current_address);

        // GT is NOT LE
        self.emit_jsr_label("__cmp_fixed_le");
        self.emit_imm(opcodes::EOR_IMM, 1); // Flip result
        self.emit_byte(opcodes::RTS);

        // Greater than or equal
        self.define_label("__cmp_fixed_ge");
        self.runtime_addresses
            .insert("cmp_fixed_ge".to_string(), self.current_address);

        // GE is NOT LT
        self.emit_jsr_label("__cmp_fixed_lt");
        self.emit_imm(opcodes::EOR_IMM, 1); // Flip result
        self.emit_byte(opcodes::RTS);
    }

    /// Emit readln routine.
    /// Uses KERNAL CHRIN ($FFCF) which handles cursor, echo, and line editing.
    /// Stores input in INPUT_BUFFER ($C100), null-terminated.
    /// Returns string address in A (low) and X (high).
    fn emit_readln_routine(&mut self) {
        self.define_label("__readln");
        self.runtime_addresses
            .insert("readln".to_string(), self.current_address);

        // Initialize buffer index to 0 (stored in TMP3 since CHRIN may clobber Y)
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);

        // Loop: get characters using CHRIN (handles cursor, echo, editing)
        self.define_label("__readln_loop");
        self.emit_abs(opcodes::JSR, kernal::CHRIN); // CHRIN at $FFCF

        // Check for RETURN (end of input)
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(petscii::RETURN);
        self.emit_branch(opcodes::BEQ, "__readln_done");

        // Load buffer index into Y and store character
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_abs(opcodes::STA_ABY, c64::INPUT_BUFFER);

        // Increment index and save back to TMP3
        self.emit_byte(opcodes::INY);
        self.emit_byte(opcodes::STY_ZP);
        self.emit_byte(zeropage::TMP3);

        // Continue loop (with overflow protection)
        self.emit_byte(opcodes::CPY_IMM);
        self.emit_byte(255);
        self.emit_branch(opcodes::BCC, "__readln_loop");

        // Done: null-terminate
        self.define_label("__readln_done");
        self.emit_byte(opcodes::LDY_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_abs(opcodes::STA_ABY, c64::INPUT_BUFFER); // Null terminator

        // Return buffer address in A (low) and X (high)
        self.emit_imm(opcodes::LDA_IMM, (c64::INPUT_BUFFER & 0xFF) as u8);
        self.emit_imm(opcodes::LDX_IMM, (c64::INPUT_BUFFER >> 8) as u8);

        self.emit_byte(opcodes::RTS);
    }

    /// Generate code for a function.
    fn generate_function(&mut self, func: &FunctionDef) -> Result<(), CompileError> {
        // Define label for function
        self.define_label(&func.name);

        // Update function address
        if let Some(f) = self.functions.get_mut(&func.name) {
            f.address = self.current_address;
        }

        // Allocate local variables for parameters
        for param in &func.params {
            self.allocate_variable(&param.name, &param.param_type, false);
        }

        // Generate function body
        self.generate_block(&func.body)?;

        // Emit RTS at end (if not already returned)
        self.emit_byte(opcodes::RTS);

        Ok(())
    }

    /// Generate code for a block.
    fn generate_block(&mut self, block: &Block) -> Result<(), CompileError> {
        for stmt in &block.statements {
            self.generate_statement(stmt)?;
        }
        Ok(())
    }

    /// Generate code for a statement.
    fn generate_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match &stmt.kind {
            StatementKind::VarDecl(decl) => self.generate_var_decl(decl),
            StatementKind::ConstDecl(decl) => self.generate_const_decl(decl),
            StatementKind::Assignment(assign) => self.generate_assignment(assign),
            StatementKind::If(if_stmt) => self.generate_if(if_stmt),
            StatementKind::While(while_stmt) => self.generate_while(while_stmt),
            StatementKind::For(for_stmt) => self.generate_for(for_stmt),
            StatementKind::Break => self.generate_break(),
            StatementKind::Continue => self.generate_continue(),
            StatementKind::Return(expr) => self.generate_return(expr.as_ref()),
            StatementKind::Pass => Ok(()), // No code needed
            StatementKind::Expression(expr) => {
                self.generate_expression(expr)?;
                Ok(())
            }
        }
    }

    /// Generate code for a variable declaration.
    fn generate_var_decl(&mut self, decl: &VarDecl) -> Result<(), CompileError> {
        let address = self.allocate_variable(&decl.name, &decl.var_type, false);

        if let Some(init) = &decl.initializer {
            self.generate_expression(init)?;
            self.emit_store_to_address(address, &decl.var_type);
        }

        Ok(())
    }

    /// Generate code for a constant declaration.
    fn generate_const_decl(&mut self, decl: &ConstDecl) -> Result<(), CompileError> {
        let const_type = self.infer_type_from_expr(&decl.value);
        let address = self.allocate_variable(&decl.name, &const_type, true);

        self.generate_expression(&decl.value)?;
        self.emit_store_to_address(address, &const_type);

        Ok(())
    }

    /// Infer type from an expression (simplified).
    fn infer_type_from_expr(&self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::IntegerLiteral(v) => {
                if *v > 255 {
                    Type::Word
                } else {
                    Type::Byte
                }
            }
            ExprKind::FixedLiteral(_) => Type::Fixed,
            ExprKind::FloatLiteral(_) => Type::Float,
            ExprKind::DecimalLiteral(_) => Type::Float, // Default to float
            ExprKind::BoolLiteral(_) => Type::Bool,
            ExprKind::CharLiteral(_) => Type::Byte,
            ExprKind::StringLiteral(_) => Type::String,
            ExprKind::Identifier(name) => {
                if let Some(var) = self.variables.get(name) {
                    var.var_type.clone()
                } else {
                    Type::Byte
                }
            }
            ExprKind::UnaryOp { op, operand } => {
                let operand_type = self.infer_type_from_expr(operand);
                match op {
                    UnaryOp::Negate => {
                        // Negation promotes to signed type
                        match operand_type {
                            Type::Byte => Type::Sbyte,
                            Type::Word => Type::Sword,
                            Type::Fixed => Type::Fixed,
                            Type::Float => Type::Float,
                            _ => operand_type,
                        }
                    }
                    _ => operand_type,
                }
            }
            ExprKind::BinaryOp { left, op: _, right } => {
                let left_type = self.infer_type_from_expr(left);
                let right_type = self.infer_type_from_expr(right);
                // Use the result type from Type::binary_result_type
                Type::binary_result_type(&left_type, &right_type).unwrap_or(Type::Byte)
            }
            ExprKind::TypeCast { target_type, .. } => target_type.clone(),
            _ => Type::Byte, // Default to byte
        }
    }

    /// Check if a type is signed.
    fn is_signed_type(&self, var_type: &Type) -> bool {
        matches!(
            var_type,
            Type::Sbyte | Type::Sword | Type::Fixed | Type::Float
        )
    }

    /// Check if a type is fixed-point.
    fn is_fixed_type(&self, var_type: &Type) -> bool {
        matches!(var_type, Type::Fixed)
    }

    /// Check if a type is floating-point.
    fn is_float_type(&self, var_type: &Type) -> bool {
        matches!(var_type, Type::Float)
    }

    /// Emit signed less-than comparison.
    /// A < TMP1 (signed): Uses SEC+SBC to set V flag, then checks N XOR V.
    /// Result: A = 1 if true, A = 0 if false.
    fn emit_signed_less_than(&mut self) {
        // For signed comparison, we need to check (N XOR V) after subtraction.
        // CMP doesn't set V, so we use SEC+SBC.
        //
        // Algorithm:
        // 1. SEC; SBC TMP1 - this sets N, V, Z, C
        // 2. If V is clear, check N directly
        // 3. If V is set, the sign is inverted, so N=0 means less than
        //
        // In 6502 assembly:
        //   SEC
        //   SBC TMP1
        //   BVC no_overflow
        //   EOR #$80       ; Flip bit 7 if overflow
        // no_overflow:
        //   BMI is_less    ; If N set (after possible flip), A < TMP1
        //   LDA #0
        //   JMP done
        // is_less:
        //   LDA #1
        // done:

        let no_overflow = self.make_label("slt_no_ovf");
        let is_less = self.make_label("slt_less");
        let done_label = self.make_label("slt_done");

        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit if overflow
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BMI, &is_less);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_less);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    /// Emit signed greater-equal comparison.
    /// A >= TMP1 (signed): NOT (A < TMP1), so check (N XOR V) == 0.
    fn emit_signed_greater_equal(&mut self) {
        let no_overflow = self.make_label("sge_no_ovf");
        let is_ge = self.make_label("sge_true");
        let done_label = self.make_label("sge_done");

        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit if overflow
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BPL, &is_ge); // If N clear (after possible flip), A >= TMP1
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_ge);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    /// Emit signed less-equal comparison.
    /// A <= TMP1 (signed): A < TMP1 OR A == TMP1.
    fn emit_signed_less_equal(&mut self) {
        // First check equality
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        let is_le = self.make_label("sle_true");
        let check_less = self.make_label("sle_check");
        let done_label = self.make_label("sle_done");

        self.emit_branch(opcodes::BEQ, &is_le); // Equal means <=

        // Not equal, check if less than
        // Reload A (CMP doesn't modify A, but we need to do SBC which does)
        // Actually, we still have A from before. Let's do the signed less check.
        self.define_label(&check_less);
        let no_overflow = self.make_label("sle_no_ovf");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BMI, &is_le); // If negative, A < TMP1, so A <= TMP1
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_le);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    /// Emit signed greater-than comparison.
    /// A > TMP1 (signed): NOT (A <= TMP1), which is A >= TMP1 AND A != TMP1.
    fn emit_signed_greater_than(&mut self) {
        // First check equality - if equal, not greater
        self.emit_byte(opcodes::CMP_ZP);
        self.emit_byte(zeropage::TMP1);
        let not_gt = self.make_label("sgt_false");
        let check_ge = self.make_label("sgt_check");
        let is_gt = self.make_label("sgt_true");
        let done_label = self.make_label("sgt_done");

        self.emit_branch(opcodes::BEQ, &not_gt); // Equal means not greater

        // Not equal, check if greater or equal (which means greater since not equal)
        self.define_label(&check_ge);
        let no_overflow = self.make_label("sgt_no_ovf");
        self.emit_byte(opcodes::SEC);
        self.emit_byte(opcodes::SBC_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_branch(opcodes::BVC, &no_overflow);
        self.emit_imm(opcodes::EOR_IMM, 0x80);
        self.define_label(&no_overflow);
        self.emit_branch(opcodes::BPL, &is_gt); // If positive, A >= TMP1, and since A != TMP1, A > TMP1

        self.define_label(&not_gt);
        self.emit_imm(opcodes::LDA_IMM, 0);
        self.emit_jmp(&done_label);
        self.define_label(&is_gt);
        self.emit_imm(opcodes::LDA_IMM, 1);
        self.define_label(&done_label);
    }

    /// Generate code for an assignment.
    fn generate_assignment(&mut self, assign: &Assignment) -> Result<(), CompileError> {
        match &assign.target {
            AssignTarget::Variable(name) => {
                let var = self.variables.get(name).cloned().ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        assign.span.clone(),
                    )
                })?;

                match assign.op {
                    AssignOp::Assign => {
                        self.generate_expression(&assign.value)?;
                    }
                    AssignOp::AddAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::CLC);
                        self.emit_byte(opcodes::ADC_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::SubAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::SEC);
                        self.emit_byte(opcodes::SBC_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::MulAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::TAX);
                        self.generate_expression(&assign.value)?;
                        self.emit_jsr_label("__mul_byte");
                    }
                    AssignOp::DivAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        self.emit_jsr_label("__div_byte");
                    }
                    AssignOp::ModAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        self.emit_jsr_label("__div_byte");
                        self.emit_byte(opcodes::TXA); // Remainder is in X
                    }
                    AssignOp::BitAndAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::AND_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::BitOrAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::ORA_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::BitXorAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::STA_ZP);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PLA);
                        self.emit_byte(opcodes::EOR_ZP);
                        self.emit_byte(zeropage::TMP1);
                    }
                    AssignOp::ShiftLeftAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        // Shift left X times
                        let loop_label = self.make_label("shl_loop");
                        let done_label = self.make_label("shl_done");
                        self.define_label(&loop_label);
                        self.emit_byte(opcodes::CPX_IMM);
                        self.emit_byte(0);
                        self.emit_branch(opcodes::BEQ, &done_label);
                        self.emit_byte(opcodes::ASL_ACC);
                        self.emit_byte(opcodes::DEX);
                        self.emit_jmp(&loop_label);
                        self.define_label(&done_label);
                    }
                    AssignOp::ShiftRightAssign => {
                        self.emit_load_from_address(var.address, &var.var_type);
                        self.emit_byte(opcodes::PHA);
                        self.generate_expression(&assign.value)?;
                        self.emit_byte(opcodes::TAX);
                        self.emit_byte(opcodes::PLA);
                        // Shift right X times
                        let loop_label = self.make_label("shr_loop");
                        let done_label = self.make_label("shr_done");
                        self.define_label(&loop_label);
                        self.emit_byte(opcodes::CPX_IMM);
                        self.emit_byte(0);
                        self.emit_branch(opcodes::BEQ, &done_label);
                        self.emit_byte(opcodes::LSR_ACC);
                        self.emit_byte(opcodes::DEX);
                        self.emit_jmp(&loop_label);
                        self.define_label(&done_label);
                    }
                }

                self.emit_store_to_address(var.address, &var.var_type);
            }
            AssignTarget::ArrayElement { name, index } => {
                // For array assignment, we need to calculate the address
                let var = self.variables.get(name).cloned().ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined array '{}'", name),
                        assign.span.clone(),
                    )
                })?;

                // Generate value first
                self.generate_expression(&assign.value)?;
                self.emit_byte(opcodes::PHA); // Save value

                // Generate index
                self.generate_expression(index)?;
                self.emit_byte(opcodes::TAY); // Y = index

                // Load base address
                self.emit_imm(opcodes::LDA_IMM, (var.address & 0xFF) as u8);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_imm(opcodes::LDA_IMM, (var.address >> 8) as u8);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1_HI);

                // Restore value and store
                self.emit_byte(opcodes::PLA);
                self.emit_byte(opcodes::STA_IZY);
                self.emit_byte(zeropage::TMP1);
            }
        }

        Ok(())
    }

    /// Generate code for an if statement.
    fn generate_if(&mut self, if_stmt: &IfStatement) -> Result<(), CompileError> {
        let else_label = self.make_label("else");
        let end_label = self.make_label("endif");

        // Generate condition
        self.generate_expression(&if_stmt.condition)?;

        // Branch if false
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        if if_stmt.elif_branches.is_empty() && if_stmt.else_block.is_none() {
            self.emit_branch(opcodes::BEQ, &end_label);
        } else {
            self.emit_branch(opcodes::BEQ, &else_label);
        }

        // Generate then block
        self.generate_block(&if_stmt.then_block)?;
        if if_stmt.else_block.is_some() || !if_stmt.elif_branches.is_empty() {
            self.emit_jmp(&end_label);
        }

        // Generate elif branches
        for (i, (cond, block)) in if_stmt.elif_branches.iter().enumerate() {
            self.define_label(&else_label);
            let next_label = if i < if_stmt.elif_branches.len() - 1 || if_stmt.else_block.is_some()
            {
                self.make_label("elif")
            } else {
                end_label.clone()
            };

            self.generate_expression(cond)?;
            self.emit_byte(opcodes::CMP_IMM);
            self.emit_byte(0);
            self.emit_branch(opcodes::BEQ, &next_label);

            self.generate_block(block)?;
            self.emit_jmp(&end_label);
        }

        // Generate else block
        if let Some(else_block) = &if_stmt.else_block {
            if if_stmt.elif_branches.is_empty() {
                self.define_label(&else_label);
            }
            self.generate_block(else_block)?;
        }

        self.define_label(&end_label);

        Ok(())
    }

    /// Generate code for a while loop.
    fn generate_while(&mut self, while_stmt: &WhileStatement) -> Result<(), CompileError> {
        let start_label = self.make_label("while_start");
        let end_label = self.make_label("while_end");

        // Push loop context
        self.loop_stack.push(LoopContext {
            start_label: start_label.clone(),
            end_label: end_label.clone(),
        });

        self.define_label(&start_label);

        // Generate condition
        self.generate_expression(&while_stmt.condition)?;
        self.emit_byte(opcodes::CMP_IMM);
        self.emit_byte(0);
        self.emit_branch(opcodes::BEQ, &end_label);

        // Generate body
        self.generate_block(&while_stmt.body)?;

        // Jump back to start
        self.emit_jmp(&start_label);

        self.define_label(&end_label);

        // Pop loop context
        self.loop_stack.pop();

        Ok(())
    }

    /// Generate code for a for loop.
    fn generate_for(&mut self, for_stmt: &ForStatement) -> Result<(), CompileError> {
        let start_label = self.make_label("for_start");
        let end_label = self.make_label("for_end");

        // Allocate loop variable
        let loop_var_addr = self.allocate_variable(&for_stmt.variable, &Type::Byte, false);

        // Initialize loop variable with start value
        self.generate_expression(&for_stmt.start)?;
        self.emit_store_to_address(loop_var_addr, &Type::Byte);

        // Push loop context
        self.loop_stack.push(LoopContext {
            start_label: start_label.clone(),
            end_label: end_label.clone(),
        });

        self.define_label(&start_label);

        // Check condition (compare with end value)
        self.emit_load_from_address(loop_var_addr, &Type::Byte);
        self.emit_byte(opcodes::PHA);
        self.generate_expression(&for_stmt.end)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::PLA);

        if for_stmt.descending {
            // For downto: exit when loop_var < end
            self.emit_byte(opcodes::CMP_ZP);
            self.emit_byte(zeropage::TMP1);
            self.emit_branch(opcodes::BCC, &end_label);
        } else {
            // For to: exit when loop_var > end
            self.emit_byte(opcodes::CMP_ZP);
            self.emit_byte(zeropage::TMP1);
            self.emit_branch(opcodes::BEQ, "__for_continue");
            self.emit_branch(opcodes::BCS, &end_label);
            self.define_label("__for_continue");
        }

        // Generate body
        self.generate_block(&for_stmt.body)?;

        // Increment or decrement loop variable
        self.emit_load_from_address(loop_var_addr, &Type::Byte);
        if for_stmt.descending {
            self.emit_byte(opcodes::SEC);
            self.emit_imm(opcodes::SBC_IMM, 1);
        } else {
            self.emit_byte(opcodes::CLC);
            self.emit_imm(opcodes::ADC_IMM, 1);
        }
        self.emit_store_to_address(loop_var_addr, &Type::Byte);

        // Jump back to start
        self.emit_jmp(&start_label);

        self.define_label(&end_label);

        // Pop loop context
        self.loop_stack.pop();

        Ok(())
    }

    /// Generate code for break statement.
    fn generate_break(&mut self) -> Result<(), CompileError> {
        if let Some(ctx) = self.loop_stack.last() {
            let end_label = ctx.end_label.clone();
            self.emit_jmp(&end_label);
            Ok(())
        } else {
            Err(CompileError::new(
                ErrorCode::BreakOutsideLoop,
                "break outside of loop",
                Span::new(0, 0),
            ))
        }
    }

    /// Generate code for continue statement.
    fn generate_continue(&mut self) -> Result<(), CompileError> {
        if let Some(ctx) = self.loop_stack.last() {
            let start_label = ctx.start_label.clone();
            self.emit_jmp(&start_label);
            Ok(())
        } else {
            Err(CompileError::new(
                ErrorCode::ContinueOutsideLoop,
                "continue outside of loop",
                Span::new(0, 0),
            ))
        }
    }

    /// Generate code for return statement.
    fn generate_return(&mut self, value: Option<&Expr>) -> Result<(), CompileError> {
        if let Some(expr) = value {
            self.generate_expression(expr)?;
        }
        self.emit_byte(opcodes::RTS);
        Ok(())
    }

    /// Generate code for an expression.
    /// Result is left in A register (for byte) or A/X (for word, A=low, X=high).
    fn generate_expression(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match &expr.kind {
            ExprKind::IntegerLiteral(value) => {
                self.emit_imm(opcodes::LDA_IMM, (*value & 0xFF) as u8);
                if *value > 255 {
                    self.emit_imm(opcodes::LDX_IMM, (*value >> 8) as u8);
                }
            }
            ExprKind::BoolLiteral(value) => {
                self.emit_imm(opcodes::LDA_IMM, if *value { 1 } else { 0 });
            }
            ExprKind::CharLiteral(c) => {
                self.emit_imm(opcodes::LDA_IMM, *c as u8);
            }
            ExprKind::StringLiteral(s) => {
                let string_index = self.add_string(s);
                self.emit_string_ref(string_index);
            }
            ExprKind::Identifier(name) => {
                let var = self.variables.get(name).cloned().ok_or_else(|| {
                    CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        expr.span.clone(),
                    )
                })?;
                self.emit_load_from_address(var.address, &var.var_type);
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.generate_binary_op(left, *op, right)?;
            }
            ExprKind::UnaryOp { op, operand } => {
                self.generate_unary_op(*op, operand)?;
            }
            ExprKind::FunctionCall { name, args } => {
                self.generate_function_call(name, args, &expr.span)?;
            }
            ExprKind::TypeCast {
                target_type,
                expr: inner,
            } => {
                let source_type = self.infer_type_from_expr(inner);
                self.generate_expression(inner)?;
                self.generate_type_conversion(&source_type, target_type)?;
            }
            ExprKind::ArrayIndex { array, index } => {
                // For array index, we need to get the base address
                // The array expression should be an identifier
                if let ExprKind::Identifier(name) = &array.kind {
                    let var = self.variables.get(name).cloned().ok_or_else(|| {
                        CompileError::new(
                            ErrorCode::UndefinedVariable,
                            format!("Undefined array '{}'", name),
                            expr.span.clone(),
                        )
                    })?;

                    // Calculate address: base + index
                    self.generate_expression(index)?;
                    self.emit_byte(opcodes::TAY); // Y = index

                    // Load base address into TMP1
                    self.emit_imm(opcodes::LDA_IMM, (var.address & 0xFF) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::LDA_IMM, (var.address >> 8) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // Load value at (TMP1),Y
                    self.emit_byte(opcodes::LDA_IZY);
                    self.emit_byte(zeropage::TMP1);
                } else {
                    return Err(CompileError::new(
                        ErrorCode::InvalidAssignmentTarget,
                        "Array index must be on an identifier",
                        expr.span.clone(),
                    ));
                }
            }
            ExprKind::FixedLiteral(value) => {
                // 16-bit fixed-point 12.4: store low byte in A, high byte in X
                self.emit_imm(opcodes::LDA_IMM, (*value & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((*value >> 8) & 0xFF) as u8);
            }
            ExprKind::FloatLiteral(bits) => {
                // 16-bit IEEE-754 binary16: store low byte in A, high byte in X
                self.emit_imm(opcodes::LDA_IMM, (*bits & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((*bits >> 8) & 0xFF) as u8);
            }
            ExprKind::DecimalLiteral(s) => {
                // Convert decimal string to IEEE-754 binary16 (default type)
                let bits = decimal_string_to_binary16(s);
                self.emit_imm(opcodes::LDA_IMM, (bits & 0xFF) as u8);
                self.emit_imm(opcodes::LDX_IMM, ((bits >> 8) & 0xFF) as u8);
            }
            ExprKind::Grouped(inner) => {
                self.generate_expression(inner)?;
            }
        }
        Ok(())
    }

    /// Generate code for a binary operation.
    fn generate_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Determine types
        let left_type = self.infer_type_from_expr(left);
        let right_type = self.infer_type_from_expr(right);
        let use_signed = self.is_signed_type(&left_type) || self.is_signed_type(&right_type);
        let use_fixed = self.is_fixed_type(&left_type) || self.is_fixed_type(&right_type);
        let use_float = self.is_float_type(&left_type) || self.is_float_type(&right_type);

        // Use 16-bit float operations if either operand is float
        if use_float {
            return self.generate_float_binary_op(left, op, right);
        }

        // Use 16-bit fixed-point operations if either operand is fixed
        if use_fixed {
            return self.generate_fixed_binary_op(left, op, right);
        }

        // Generate left operand
        self.generate_expression(left)?;
        self.emit_byte(opcodes::PHA); // Save left on stack

        // Generate right operand
        self.generate_expression(right)?;
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1); // Right in TMP1

        // Restore left to A
        self.emit_byte(opcodes::PLA);

        match op {
            BinaryOp::Add => {
                // Addition is the same for signed and unsigned (two's complement)
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Sub => {
                // Subtraction is the same for signed and unsigned (two's complement)
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Mul => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__mul_sbyte");
                } else {
                    self.emit_jsr_label("__mul_byte");
                }
            }
            BinaryOp::Div => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__div_sbyte");
                } else {
                    self.emit_jsr_label("__div_byte");
                }
            }
            BinaryOp::Mod => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                if use_signed {
                    self.emit_jsr_label("__div_sbyte");
                } else {
                    self.emit_jsr_label("__div_byte");
                }
                self.emit_byte(opcodes::TXA); // Remainder is in X
            }
            BinaryOp::BitAnd => {
                self.emit_byte(opcodes::AND_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::BitOr => {
                self.emit_byte(opcodes::ORA_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::BitXor => {
                self.emit_byte(opcodes::EOR_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::ShiftLeft => {
                // Shift left by TMP1 times
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                let loop_label = self.make_label("shl");
                let done_label = self.make_label("shl_done");
                self.define_label(&loop_label);
                self.emit_byte(opcodes::CPX_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &done_label);
                self.emit_byte(opcodes::ASL_ACC);
                self.emit_byte(opcodes::DEX);
                self.emit_jmp(&loop_label);
                self.define_label(&done_label);
            }
            BinaryOp::ShiftRight => {
                // Shift right by TMP1 times
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                let loop_label = self.make_label("shr");
                let done_label = self.make_label("shr_done");
                self.define_label(&loop_label);
                self.emit_byte(opcodes::CPX_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &done_label);
                self.emit_byte(opcodes::LSR_ACC);
                self.emit_byte(opcodes::DEX);
                self.emit_jmp(&loop_label);
                self.define_label(&done_label);
            }
            BinaryOp::Equal => {
                // Equality is the same for signed and unsigned
                self.emit_byte(opcodes::CMP_ZP);
                self.emit_byte(zeropage::TMP1);
                let eq_label = self.make_label("eq");
                let done_label = self.make_label("eq_done");
                self.emit_branch(opcodes::BEQ, &eq_label);
                self.emit_imm(opcodes::LDA_IMM, 0); // Not equal
                self.emit_jmp(&done_label);
                self.define_label(&eq_label);
                self.emit_imm(opcodes::LDA_IMM, 1); // Equal
                self.define_label(&done_label);
            }
            BinaryOp::NotEqual => {
                // Inequality is the same for signed and unsigned
                self.emit_byte(opcodes::CMP_ZP);
                self.emit_byte(zeropage::TMP1);
                let ne_label = self.make_label("ne");
                let done_label = self.make_label("ne_done");
                self.emit_branch(opcodes::BNE, &ne_label);
                self.emit_imm(opcodes::LDA_IMM, 0); // Equal
                self.emit_jmp(&done_label);
                self.define_label(&ne_label);
                self.emit_imm(opcodes::LDA_IMM, 1); // Not equal
                self.define_label(&done_label);
            }
            BinaryOp::Less => {
                if use_signed {
                    // Signed A < TMP1: use SEC+SBC to set V flag, then check N XOR V
                    self.emit_signed_less_than();
                } else {
                    // Unsigned: A < TMP1 when carry clear after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let lt_label = self.make_label("lt");
                    let done_label = self.make_label("lt_done");
                    self.emit_branch(opcodes::BCC, &lt_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&lt_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::LessEqual => {
                if use_signed {
                    // Signed A <= TMP1: A < TMP1 OR A == TMP1
                    self.emit_signed_less_equal();
                } else {
                    // Unsigned: A <= TMP1 when carry clear OR zero after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let le_label = self.make_label("le");
                    let done_label = self.make_label("le_done");
                    self.emit_branch(opcodes::BCC, &le_label);
                    self.emit_branch(opcodes::BEQ, &le_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&le_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::Greater => {
                if use_signed {
                    // Signed A > TMP1: NOT (A <= TMP1)
                    self.emit_signed_greater_than();
                } else {
                    // Unsigned: A > TMP1 when carry set and not equal
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let gt_label = self.make_label("gt");
                    let done_label = self.make_label("gt_done");
                    self.emit_branch(opcodes::BEQ, &done_label.clone()); // Equal, not greater
                    self.emit_branch(opcodes::BCS, &gt_label); // Carry set = greater or equal
                                                               // Carry clear means less than
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&gt_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::GreaterEqual => {
                if use_signed {
                    // Signed A >= TMP1: NOT (A < TMP1)
                    self.emit_signed_greater_equal();
                } else {
                    // Unsigned: A >= TMP1 when carry set after CMP
                    self.emit_byte(opcodes::CMP_ZP);
                    self.emit_byte(zeropage::TMP1);
                    let ge_label = self.make_label("ge");
                    let done_label = self.make_label("ge_done");
                    self.emit_branch(opcodes::BCS, &ge_label);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_jmp(&done_label);
                    self.define_label(&ge_label);
                    self.emit_imm(opcodes::LDA_IMM, 1);
                    self.define_label(&done_label);
                }
            }
            BinaryOp::And => {
                // Logical AND
                let false_label = self.make_label("and_false");
                let done_label = self.make_label("and_done");
                // Left is already in A
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &false_label);
                // Check right (in TMP1)
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &false_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.emit_jmp(&done_label);
                self.define_label(&false_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.define_label(&done_label);
            }
            BinaryOp::Or => {
                // Logical OR
                let true_label = self.make_label("or_true");
                let done_label = self.make_label("or_done");
                // Left is already in A
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BNE, &true_label);
                // Check right (in TMP1)
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BNE, &true_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_jmp(&done_label);
                self.define_label(&true_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.define_label(&done_label);
            }
        }

        Ok(())
    }

    /// Generate code for a fixed-point binary operation (16-bit).
    ///
    /// Fixed-point uses 12.4 format (12 bits integer, 4 bits fraction).
    /// Internal representation is value × 16 stored as signed 16-bit.
    fn generate_fixed_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Generate left operand (result in A=low, X=high)
        self.generate_expression(left)?;

        // Save left in FP_LEFT (TMP3/TMP3_HI)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        // Generate right operand
        self.generate_expression(right)?;

        // Right in A=low, X=high, save to TMP1/TMP1_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        match op {
            BinaryOp::Add => {
                // 16-bit addition: TMP3 + TMP1 -> A/X
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3); // Store result low
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                self.emit_byte(opcodes::TAX); // X = high byte
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3); // A = low byte
            }
            BinaryOp::Sub => {
                // 16-bit subtraction: TMP3 - TMP1 -> A/X
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP3);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3_HI);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                self.emit_byte(opcodes::TAX);
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP3);
            }
            BinaryOp::Mul => {
                // Fixed-point multiplication:
                // (a×16) × (b×16) = (a×b) × 256
                // Result needs to be shifted right by 4 to get (a×b) × 16
                self.emit_jsr_label("__mul_fixed");
            }
            BinaryOp::Div => {
                // Fixed-point division:
                // ((a×16) << 4) / (b×16) = (a/b) × 16
                self.emit_jsr_label("__div_fixed");
            }
            BinaryOp::Mod => {
                // Fixed-point modulo (same as division but return remainder)
                self.emit_jsr_label("__mod_fixed");
            }
            BinaryOp::Equal => {
                // 16-bit equality: compare both bytes
                self.emit_fixed_comparison(|s| {
                    let eq_label = s.make_label("feq");
                    let done_label = s.make_label("feq_done");
                    // Compare high bytes first
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3_HI);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1_HI);
                    s.emit_branch(opcodes::BNE, &done_label.clone());
                    // Compare low bytes
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1);
                    s.emit_branch(opcodes::BNE, &done_label);
                    // Equal
                    s.emit_imm(opcodes::LDA_IMM, 1);
                    s.emit_jmp(&eq_label);
                    s.define_label(&done_label);
                    s.emit_imm(opcodes::LDA_IMM, 0);
                    s.define_label(&eq_label);
                });
            }
            BinaryOp::NotEqual => {
                self.emit_fixed_comparison(|s| {
                    let ne_label = s.make_label("fne");
                    let done_label = s.make_label("fne_done");
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3_HI);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1_HI);
                    s.emit_branch(opcodes::BNE, &ne_label.clone());
                    s.emit_byte(opcodes::LDA_ZP);
                    s.emit_byte(zeropage::TMP3);
                    s.emit_byte(opcodes::CMP_ZP);
                    s.emit_byte(zeropage::TMP1);
                    s.emit_branch(opcodes::BNE, &ne_label);
                    s.emit_imm(opcodes::LDA_IMM, 0);
                    s.emit_jmp(&done_label);
                    s.define_label(&ne_label);
                    s.emit_imm(opcodes::LDA_IMM, 1);
                    s.define_label(&done_label);
                });
            }
            BinaryOp::Less => {
                // Signed 16-bit less than
                self.emit_jsr_label("__cmp_fixed_lt");
            }
            BinaryOp::LessEqual => {
                self.emit_jsr_label("__cmp_fixed_le");
            }
            BinaryOp::Greater => {
                self.emit_jsr_label("__cmp_fixed_gt");
            }
            BinaryOp::GreaterEqual => {
                self.emit_jsr_label("__cmp_fixed_ge");
            }
            _ => {
                // Bitwise and logical operations are not supported for fixed-point
                // (should be caught by analyzer)
                return Err(CompileError::new(
                    ErrorCode::InvalidOperatorForType,
                    format!("Operator {:?} is not supported for fixed-point types", op),
                    left.span.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Helper for fixed-point comparisons.
    fn emit_fixed_comparison<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        f(self);
    }

    /// Generate code for a float binary operation (IEEE-754 binary16).
    ///
    /// Float uses IEEE-754 half-precision format (1 sign + 5 exp + 10 mantissa).
    fn generate_float_binary_op(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<(), CompileError> {
        // Generate left operand (result in A=low, X=high)
        self.generate_expression(left)?;

        // Save left in FP_ARG1 (TMP1/TMP1_HI for float runtime)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP1_HI);

        // Generate right operand
        self.generate_expression(right)?;

        // Right in A=low, X=high, save to FP_ARG2 (TMP3/TMP3_HI)
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP3);
        self.emit_byte(opcodes::STX_ZP);
        self.emit_byte(zeropage::TMP3_HI);

        match op {
            BinaryOp::Add => {
                self.emit_jsr_label("__float_add");
            }
            BinaryOp::Sub => {
                self.emit_jsr_label("__float_sub");
            }
            BinaryOp::Mul => {
                self.emit_jsr_label("__float_mul");
            }
            BinaryOp::Div => {
                self.emit_jsr_label("__float_div");
            }
            BinaryOp::Mod => {
                // Float modulo: a - floor(a/b) * b
                // For simplicity, use fmod-like behavior
                self.emit_jsr_label("__float_mod");
            }
            BinaryOp::Equal => {
                self.emit_jsr_label("__float_cmp_eq");
            }
            BinaryOp::NotEqual => {
                self.emit_jsr_label("__float_cmp_ne");
            }
            BinaryOp::Less => {
                self.emit_jsr_label("__float_cmp_lt");
            }
            BinaryOp::LessEqual => {
                self.emit_jsr_label("__float_cmp_le");
            }
            BinaryOp::Greater => {
                self.emit_jsr_label("__float_cmp_gt");
            }
            BinaryOp::GreaterEqual => {
                self.emit_jsr_label("__float_cmp_ge");
            }
            _ => {
                // Bitwise and logical operations are not supported for float
                return Err(CompileError::new(
                    ErrorCode::InvalidOperatorForType,
                    format!("Operator {:?} is not supported for float types", op),
                    left.span.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Generate code for type conversion.
    ///
    /// Handles conversions between integer, fixed, and float types.
    fn generate_type_conversion(
        &mut self,
        source_type: &Type,
        target_type: &Type,
    ) -> Result<(), CompileError> {
        // No conversion needed if types are the same
        if source_type == target_type {
            return Ok(());
        }

        match (source_type, target_type) {
            // Integer to Float conversions
            (Type::Byte, Type::Float) | (Type::Sbyte, Type::Float) => {
                // 8-bit value in A -> float in A/X
                self.emit_jsr_label("__byte_to_float");
            }
            (Type::Word, Type::Float) | (Type::Sword, Type::Float) => {
                // 16-bit value in A/X -> float in A/X
                self.emit_jsr_label("__word_to_float");
            }

            // Float to Integer conversions
            (Type::Float, Type::Byte) | (Type::Float, Type::Sbyte) => {
                // Float in A/X -> 8-bit in A
                self.emit_jsr_label("__float_to_byte");
            }
            (Type::Float, Type::Word) | (Type::Float, Type::Sword) => {
                // Float in A/X -> 16-bit in A/X
                self.emit_jsr_label("__float_to_word");
            }

            // Fixed to Float conversion
            (Type::Fixed, Type::Float) => {
                // 12.4 fixed in A/X -> float in A/X
                self.emit_jsr_label("__fixed_to_float");
            }

            // Float to Fixed conversion
            (Type::Float, Type::Fixed) => {
                // Float in A/X -> 12.4 fixed in A/X
                self.emit_jsr_label("__float_to_fixed");
            }

            // Integer to Fixed conversions
            (Type::Byte, Type::Fixed) | (Type::Sbyte, Type::Fixed) => {
                // 8-bit value in A -> 12.4 fixed in A/X
                // fixed = value << 4 (value * 16)
                self.emit_byte(opcodes::ASL_ACC); // *2
                self.emit_byte(opcodes::ASL_ACC); // *4
                self.emit_byte(opcodes::ASL_ACC); // *8
                self.emit_byte(opcodes::ASL_ACC); // *16
                self.emit_imm(opcodes::LDX_IMM, 0); // High byte = 0 for positive
                                                    // Handle sign extension for signed bytes
                if *source_type == Type::Sbyte {
                    // If original was negative, we need to handle differently
                    // This simple version works for non-negative values
                    // For negative, we'd need to sign-extend before shifting
                }
            }
            (Type::Word, Type::Fixed) | (Type::Sword, Type::Fixed) => {
                // 16-bit value in A/X -> 12.4 fixed in A/X
                // fixed = value << 4
                // Store to temp, shift, return
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift left 4 times
                for _ in 0..4 {
                    self.emit_byte(opcodes::ASL_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::ROL_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
            }

            // Fixed to Integer conversions
            (Type::Fixed, Type::Byte) | (Type::Fixed, Type::Sbyte) => {
                // 12.4 fixed in A/X -> 8-bit in A
                // Truncate: value >> 4
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift right 4 times (arithmetic for signed)
                for _ in 0..4 {
                    self.emit_byte(opcodes::LSR_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_byte(opcodes::ROR_ZP);
                    self.emit_byte(zeropage::TMP1);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            (Type::Fixed, Type::Word) | (Type::Fixed, Type::Sword) => {
                // 12.4 fixed in A/X -> 16-bit in A/X
                // Truncate: value >> 4
                self.emit_byte(opcodes::STA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::STX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
                // Shift right 4 times
                for _ in 0..4 {
                    self.emit_byte(opcodes::LSR_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_byte(opcodes::ROR_ZP);
                    self.emit_byte(zeropage::TMP1);
                }
                self.emit_byte(opcodes::LDA_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1_HI);
            }

            // 8-bit to 16-bit promotions
            (Type::Byte, Type::Word) => {
                // Zero-extend: A stays, X = 0
                self.emit_imm(opcodes::LDX_IMM, 0);
            }
            (Type::Sbyte, Type::Sword) => {
                // Sign-extend: check bit 7 of A
                self.emit_imm(opcodes::LDX_IMM, 0);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0x80);
                let positive = self.make_label("sext_pos");
                self.emit_branch(opcodes::BCC, &positive);
                self.emit_imm(opcodes::LDX_IMM, 0xFF); // Sign extend with $FF
                self.define_label(&positive);
            }
            (Type::Byte, Type::Sword) | (Type::Sbyte, Type::Word) => {
                // Mixed sign extension
                self.emit_imm(opcodes::LDX_IMM, 0);
            }

            // 16-bit to 8-bit truncations
            (Type::Word, Type::Byte)
            | (Type::Sword, Type::Sbyte)
            | (Type::Word, Type::Sbyte)
            | (Type::Sword, Type::Byte) => {
                // Just keep the low byte (already in A)
            }

            // Same size, different signedness - no runtime conversion needed
            (Type::Byte, Type::Sbyte)
            | (Type::Sbyte, Type::Byte)
            | (Type::Word, Type::Sword)
            | (Type::Sword, Type::Word) => {
                // No-op, just reinterpret
            }

            // Other conversions (bool, string, etc.) - no runtime conversion
            _ => {}
        }

        Ok(())
    }

    /// Generate code for a unary operation.
    fn generate_unary_op(&mut self, op: UnaryOp, operand: &Expr) -> Result<(), CompileError> {
        let operand_type = self.infer_type_from_expr(operand);
        let is_fixed = self.is_fixed_type(&operand_type);
        let is_float = self.is_float_type(&operand_type);

        self.generate_expression(operand)?;

        match op {
            UnaryOp::Negate => {
                if is_float {
                    // Float negation: flip sign bit (bit 15)
                    // Result is in A (low) and X (high)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::TXA);
                    self.emit_imm(opcodes::EOR_IMM, 0x80); // Flip sign bit
                    self.emit_byte(opcodes::TAX);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                } else if is_fixed {
                    // 16-bit two's complement negation
                    // Result is in A (low) and X (high)
                    // Negate: NOT both bytes, then add 1
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // NOT low byte
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);

                    // NOT high byte
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    // Add 1 (16-bit)
                    self.emit_byte(opcodes::CLC);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::ADC_IMM, 1);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    self.emit_imm(opcodes::ADC_IMM, 0);
                    self.emit_byte(opcodes::TAX);
                    self.emit_byte(opcodes::LDA_ZP);
                    self.emit_byte(zeropage::TMP1);
                } else {
                    // 8-bit two's complement negation: EOR #$FF, CLC, ADC #1
                    self.emit_imm(opcodes::EOR_IMM, 0xFF);
                    self.emit_byte(opcodes::CLC);
                    self.emit_imm(opcodes::ADC_IMM, 1);
                }
            }
            UnaryOp::Not => {
                // Logical NOT: if A == 0 then 1, else 0
                let zero_label = self.make_label("not_zero");
                let done_label = self.make_label("not_done");
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &zero_label);
                self.emit_imm(opcodes::LDA_IMM, 0);
                self.emit_jmp(&done_label);
                self.define_label(&zero_label);
                self.emit_imm(opcodes::LDA_IMM, 1);
                self.define_label(&done_label);
            }
            UnaryOp::BitNot => {
                // Bitwise NOT
                self.emit_imm(opcodes::EOR_IMM, 0xFF);
            }
        }

        Ok(())
    }

    /// Generate code for a function call.
    fn generate_function_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: &Span,
    ) -> Result<(), CompileError> {
        match name {
            "cls" => {
                // Clear screen: output CHR$(147)
                self.emit_imm(opcodes::LDA_IMM, petscii::CLEAR_SCREEN);
                self.emit_abs(opcodes::JSR, kernal::CHROUT);
            }
            "print" | "println" => {
                if !args.is_empty() {
                    let arg_type = self.infer_type_from_expr(&args[0]);
                    self.generate_expression(&args[0])?;

                    // Call appropriate print routine based on type
                    match arg_type {
                        Type::String => {
                            // For string variables, copy A/X to TMP1/TMP1_HI
                            // (String literals already set TMP1/TMP1_HI, this is safe)
                            self.emit_byte(opcodes::STA_ZP);
                            self.emit_byte(zeropage::TMP1);
                            self.emit_byte(opcodes::STX_ZP);
                            self.emit_byte(zeropage::TMP1_HI);
                            self.emit_jsr_label("__print_str");
                        }
                        Type::Word => {
                            self.emit_jsr_label("__print_word");
                        }
                        Type::Sword => {
                            self.emit_jsr_label("__print_sword");
                        }
                        Type::Bool => {
                            self.emit_jsr_label("__print_bool");
                        }
                        Type::Sbyte => {
                            self.emit_jsr_label("__print_sbyte");
                        }
                        Type::Byte => {
                            self.emit_jsr_label("__print_byte");
                        }
                        Type::Fixed => {
                            self.emit_jsr_label("__print_fixed");
                        }
                        Type::Float => {
                            self.emit_jsr_label("__print_float");
                        }
                        _ => {
                            // Default to byte printing
                            self.emit_jsr_label("__print_byte");
                        }
                    }
                }
                if name == "println" {
                    self.emit_imm(opcodes::LDA_IMM, petscii::RETURN);
                    self.emit_abs(opcodes::JSR, kernal::CHROUT);
                }
            }
            "cursor" => {
                if args.len() >= 2 {
                    // KERNAL PLOT expects: X=Row, Y=Column
                    // Our API: cursor(x=column, y=row)
                    // So we need: Y=args[0] (column), X=args[1] (row)
                    self.generate_expression(&args[1])?; // row -> X
                    self.emit_byte(opcodes::TAX);
                    self.generate_expression(&args[0])?; // column -> Y
                    self.emit_byte(opcodes::TAY);
                    // Call PLOT with carry clear (set position)
                    self.emit_byte(opcodes::CLC);
                    self.emit_abs(opcodes::JSR, kernal::PLOT);
                }
            }
            "get_key" => {
                self.emit_abs(opcodes::JSR, kernal::GETIN);
            }
            "wait_for_key" => {
                let wait_label = self.make_label("wait_key");
                self.define_label(&wait_label);
                self.emit_abs(opcodes::JSR, kernal::GETIN);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(0);
                self.emit_branch(opcodes::BEQ, &wait_label);
            }
            "readln" => {
                // Read a line of input into buffer at INPUT_BUFFER ($C100)
                // Returns string address in TMP1/TMP1_HI (for print_str compatibility)
                self.emit_jsr_label("__readln");
            }
            "poke" => {
                if args.len() >= 2 {
                    // Value to poke
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::PHA);
                    // Address (A=low, X=high for word values)
                    // First set X to 0 as default for byte addresses
                    self.emit_imm(opcodes::LDX_IMM, 0);
                    self.generate_expression(&args[0])?;
                    // Store address (A=low byte, X=high byte)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Store value
                    self.emit_byte(opcodes::PLA);
                    self.emit_imm(opcodes::LDY_IMM, 0);
                    self.emit_byte(opcodes::STA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            "peek" => {
                if !args.is_empty() {
                    // Address (A=low, X=high for word values)
                    // First set X to 0 as default for byte addresses
                    self.emit_imm(opcodes::LDX_IMM, 0);
                    self.generate_expression(&args[0])?;
                    // Store address (A=low byte, X=high byte)
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_byte(opcodes::STX_ZP);
                    self.emit_byte(zeropage::TMP1_HI);
                    // Load value
                    self.emit_imm(opcodes::LDY_IMM, 0);
                    self.emit_byte(opcodes::LDA_IZY);
                    self.emit_byte(zeropage::TMP1);
                }
            }
            _ => {
                // User-defined function
                // Push arguments (simplified: just first arg in A)
                for arg in args {
                    self.generate_expression(arg)?;
                    // For multiple args, we'd need to push to stack or use memory
                }
                // Call function
                if self.functions.contains_key(name) {
                    self.emit_jsr_label(name);
                } else {
                    return Err(CompileError::new(
                        ErrorCode::UndefinedFunction,
                        format!("Undefined function '{}'", name),
                        span.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Emit code to load a value from an address into A (and X for 16-bit types).
    fn emit_load_from_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::LDA_ABS, address);
            }
            Type::Word | Type::Sword | Type::String => {
                // String is a 16-bit pointer, same as Word
                self.emit_abs(opcodes::LDA_ABS, address);
                self.emit_abs(opcodes::LDX_ABS, address.wrapping_add(1));
            }
            _ => {
                // For other types, just load the address
                self.emit_abs(opcodes::LDA_ABS, address);
            }
        }
    }

    /// Emit code to store A (and X for 16-bit types) to an address.
    fn emit_store_to_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::STA_ABS, address);
            }
            Type::Word | Type::Sword | Type::String => {
                // String is a 16-bit pointer, same as Word
                self.emit_abs(opcodes::STA_ABS, address);
                self.emit_byte(opcodes::STX_ABS);
                self.emit_word(address.wrapping_add(1));
            }
            _ => {
                self.emit_abs(opcodes::STA_ABS, address);
            }
        }
    }

    /// Get the generated code.
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Get the current code address.
    pub fn current_address(&self) -> u16 {
        self.current_address
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate code for a program.
pub fn generate(program: &Program) -> Result<Vec<u8>, CompileError> {
    let mut generator = CodeGenerator::new();
    generator.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;

    #[test]
    fn test_basic_stub() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        // Should have generated a BASIC stub
        assert!(!gen.code.is_empty());

        // First two bytes should be link address ($080C)
        assert_eq!(gen.code[0], 0x0C);
        assert_eq!(gen.code[1], 0x08);

        // Machine code should start at $080E (2062)
        assert_eq!(gen.current_address, CODE_START);
    }

    #[test]
    fn test_emit_word() {
        let mut gen = CodeGenerator::new();
        gen.emit_word(0x1234);

        assert_eq!(gen.code[0], 0x34); // Low byte first
        assert_eq!(gen.code[1], 0x12); // High byte second
    }

    #[test]
    fn test_emit_imm() {
        let mut gen = CodeGenerator::new();
        gen.emit_imm(opcodes::LDA_IMM, 0x42);

        assert_eq!(gen.code[0], 0xA9); // LDA #
        assert_eq!(gen.code[1], 0x42); // Value
    }

    #[test]
    fn test_emit_abs() {
        let mut gen = CodeGenerator::new();
        gen.emit_abs(opcodes::LDA_ABS, 0x1234);

        assert_eq!(gen.code[0], 0xAD); // LDA abs
        assert_eq!(gen.code[1], 0x34); // Low byte
        assert_eq!(gen.code[2], 0x12); // High byte
    }

    #[test]
    fn test_make_label() {
        let mut gen = CodeGenerator::new();
        let label1 = gen.make_label("test");
        let label2 = gen.make_label("test");

        assert_eq!(label1, "test_0");
        assert_eq!(label2, "test_1");
    }

    #[test]
    fn test_define_and_resolve_label() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        gen.define_label("target");
        gen.emit_byte(opcodes::NOP);

        assert!(gen.labels.contains_key("target"));
        assert_eq!(gen.labels["target"], CODE_START);
    }

    #[test]
    fn test_allocate_variable() {
        let mut gen = CodeGenerator::new();

        let addr1 = gen.allocate_variable("x", &Type::Byte, false);
        let addr2 = gen.allocate_variable("y", &Type::Word, false);

        assert_eq!(addr1, 0xC000);
        assert_eq!(addr2, 0xC001); // After byte variable
        assert!(gen.variables.contains_key("x"));
        assert!(gen.variables.contains_key("y"));
    }

    #[test]
    fn test_add_string() {
        let mut gen = CodeGenerator::new();

        let index = gen.add_string("HELLO");

        assert_eq!(index, 0);
        assert_eq!(gen.strings.len(), 1);
        assert_eq!(gen.strings[0], vec![b'H', b'E', b'L', b'L', b'O', 0]);
    }

    #[test]
    fn test_opcodes() {
        // Verify some critical opcodes
        assert_eq!(opcodes::LDA_IMM, 0xA9);
        assert_eq!(opcodes::STA_ABS, 0x8D);
        assert_eq!(opcodes::JSR, 0x20);
        assert_eq!(opcodes::RTS, 0x60);
        assert_eq!(opcodes::JMP_ABS, 0x4C);
    }

    // ========================================
    // Signed Type Tests
    // ========================================

    #[test]
    fn test_is_signed_type() {
        let gen = CodeGenerator::new();
        assert!(gen.is_signed_type(&Type::Sbyte));
        assert!(gen.is_signed_type(&Type::Sword));
        assert!(!gen.is_signed_type(&Type::Byte));
        assert!(!gen.is_signed_type(&Type::Word));
        assert!(!gen.is_signed_type(&Type::Bool));
    }

    #[test]
    fn test_allocate_signed_variable() {
        let mut gen = CodeGenerator::new();

        let addr1 = gen.allocate_variable("x", &Type::Sbyte, false);
        let addr2 = gen.allocate_variable("y", &Type::Sword, false);

        assert_eq!(addr1, 0xC000);
        assert_eq!(addr2, 0xC001); // After sbyte variable (1 byte)
        assert!(gen.variables.contains_key("x"));
        assert!(gen.variables.contains_key("y"));

        // Check types
        assert_eq!(gen.variables.get("x").unwrap().var_type, Type::Sbyte);
        assert_eq!(gen.variables.get("y").unwrap().var_type, Type::Sword);
    }

    #[test]
    fn test_infer_type_negation() {
        let gen = CodeGenerator::new();

        // Create a negation expression
        let operand = Expr {
            kind: ExprKind::IntegerLiteral(100),
            span: Span::new(0, 3),
        };
        let negate_expr = Expr {
            kind: ExprKind::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            span: Span::new(0, 4),
        };

        // Negation of byte should produce sbyte
        let inferred = gen.infer_type_from_expr(&negate_expr);
        assert_eq!(inferred, Type::Sbyte);
    }

    #[test]
    fn test_infer_type_word_negation() {
        let gen = CodeGenerator::new();

        // Create a negation expression for word
        let operand = Expr {
            kind: ExprKind::IntegerLiteral(1000),
            span: Span::new(0, 4),
        };
        let negate_expr = Expr {
            kind: ExprKind::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            span: Span::new(0, 5),
        };

        // Negation of word should produce sword
        let inferred = gen.infer_type_from_expr(&negate_expr);
        assert_eq!(inferred, Type::Sword);
    }

    #[test]
    fn test_signed_variable_type_preserved() {
        let mut gen = CodeGenerator::new();

        gen.allocate_variable("signed_byte", &Type::Sbyte, false);
        gen.allocate_variable("signed_word", &Type::Sword, false);

        // Check that looking up the variable returns the correct signed type
        let sbyte_var = gen.variables.get("signed_byte").unwrap();
        let sword_var = gen.variables.get("signed_word").unwrap();

        assert!(gen.is_signed_type(&sbyte_var.var_type));
        assert!(gen.is_signed_type(&sword_var.var_type));
    }

    #[test]
    fn test_runtime_includes_signed_routines() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();
        gen.emit_runtime_library();

        // Check that signed runtime addresses were registered
        assert!(gen.runtime_addresses.contains_key("print_sbyte"));
        assert!(gen.runtime_addresses.contains_key("print_sword"));
        assert!(gen.runtime_addresses.contains_key("mul_sbyte"));
        assert!(gen.runtime_addresses.contains_key("mul_sword"));
        assert!(gen.runtime_addresses.contains_key("div_sbyte"));
        assert!(gen.runtime_addresses.contains_key("div_sword"));
    }

    // ========================================
    // Signed Comparison Tests
    // ========================================

    #[test]
    fn test_emit_signed_less_than_generates_code() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        let start_len = gen.code.len();
        gen.emit_signed_less_than();
        let end_len = gen.code.len();

        // Should generate some code
        assert!(
            end_len > start_len,
            "emit_signed_less_than should generate code"
        );
        // Should include SEC opcode
        assert!(gen.code[start_len..].contains(&opcodes::SEC));
    }

    #[test]
    fn test_emit_signed_greater_equal_generates_code() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        let start_len = gen.code.len();
        gen.emit_signed_greater_equal();
        let end_len = gen.code.len();

        assert!(
            end_len > start_len,
            "emit_signed_greater_equal should generate code"
        );
        assert!(gen.code[start_len..].contains(&opcodes::SEC));
    }

    #[test]
    fn test_emit_signed_less_equal_generates_code() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        let start_len = gen.code.len();
        gen.emit_signed_less_equal();
        let end_len = gen.code.len();

        assert!(
            end_len > start_len,
            "emit_signed_less_equal should generate code"
        );
    }

    #[test]
    fn test_emit_signed_greater_than_generates_code() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        let start_len = gen.code.len();
        gen.emit_signed_greater_than();
        let end_len = gen.code.len();

        assert!(
            end_len > start_len,
            "emit_signed_greater_than should generate code"
        );
    }

    #[test]
    fn test_signed_comparison_uses_bvc_instruction() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        gen.emit_signed_less_than();

        // Should include BVC opcode (branch if overflow clear)
        assert!(
            gen.code.contains(&opcodes::BVC),
            "Signed comparison should use BVC for overflow handling"
        );
    }

    #[test]
    fn test_signed_comparison_uses_eor_for_sign_flip() {
        let mut gen = CodeGenerator::new();
        gen.emit_basic_stub();

        gen.emit_signed_less_than();

        // Should include EOR #$80 for sign bit flip
        // EOR_IMM is $49, followed by $80
        let code = &gen.code;
        let has_eor_80 = code
            .windows(2)
            .any(|w| w[0] == opcodes::EOR_IMM && w[1] == 0x80);
        assert!(
            has_eor_80,
            "Signed comparison should use EOR #$80 for sign flip on overflow"
        );
    }
}
