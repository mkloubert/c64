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

pub mod mos6510;

use crate::ast::{
    AssignOp, AssignTarget, Assignment, BinaryOp, Block, ConstDecl, Expr, ExprKind, ForStatement,
    FunctionDef, IfStatement, Program, Statement, StatementKind, TopLevelItem, Type, UnaryOp,
    VarDecl, WhileStatement,
};
use crate::error::{CompileError, ErrorCode, Span};
use mos6510::{kernal, opcodes, petscii, zeropage};
use std::collections::HashMap;

/// Memory layout constants.
const PROGRAM_START: u16 = 0x0801; // Standard C64 BASIC program start
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
    /// Whether this is a constant.
    is_const: bool,
}

/// Function information for code generation.
#[derive(Debug, Clone)]
struct Function {
    /// Address where the function code starts.
    address: u16,
    /// Parameter types.
    params: Vec<Type>,
    /// Return type (None for void).
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

    /// Emit code to load a string address into TMP1/TMP1_HI.
    /// The actual address will be patched later when string positions are known.
    fn emit_string_ref(&mut self, string_index: usize) {
        // LDA #<string_addr (placeholder)
        self.emit_byte(opcodes::LDA_IMM);
        let code_offset_lo = self.code.len();
        self.emit_byte(0); // Placeholder for low byte

        // STA TMP1
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1);

        // LDA #>string_addr (placeholder)
        self.emit_byte(opcodes::LDA_IMM);
        let code_offset_hi = self.code.len();
        self.emit_byte(0); // Placeholder for high byte

        // STA TMP1_HI
        self.emit_byte(opcodes::STA_ZP);
        self.emit_byte(zeropage::TMP1_HI);

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

        // Multiply routines
        self.emit_multiply_byte_routine();
        self.emit_multiply_word_routine();

        // Divide routines
        self.emit_divide_byte_routine();
        self.emit_divide_word_routine();
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

    /// Emit print word routine (simplified).
    fn emit_print_word_routine(&mut self) {
        self.define_label("__print_word");
        self.runtime_addresses
            .insert("print_word".to_string(), self.current_address);

        // For now, just print low byte
        // A proper implementation would handle full 16-bit
        self.emit_jsr_label("__print_byte");

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
            _ => Type::Byte, // Default to byte
        }
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
                self.generate_expression(inner)?;
                // Type casting is mostly a compile-time concept
                // At runtime, we might need to extend or truncate
                let _ = target_type; // Use the cast type as needed
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
                self.emit_byte(opcodes::CLC);
                self.emit_byte(opcodes::ADC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Sub => {
                self.emit_byte(opcodes::SEC);
                self.emit_byte(opcodes::SBC_ZP);
                self.emit_byte(zeropage::TMP1);
            }
            BinaryOp::Mul => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_jsr_label("__mul_byte");
            }
            BinaryOp::Div => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_jsr_label("__div_byte");
            }
            BinaryOp::Mod => {
                self.emit_byte(opcodes::LDX_ZP);
                self.emit_byte(zeropage::TMP1);
                self.emit_jsr_label("__div_byte");
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
                // Compare and set result
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
                // A < TMP1: carry clear after CMP if A < TMP1
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
            BinaryOp::LessEqual => {
                // A <= TMP1: A < TMP1 or A == TMP1
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
            BinaryOp::Greater => {
                // A > TMP1: carry set and not equal
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
            BinaryOp::GreaterEqual => {
                // A >= TMP1: carry set after CMP
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

    /// Generate code for a unary operation.
    fn generate_unary_op(&mut self, op: UnaryOp, operand: &Expr) -> Result<(), CompileError> {
        self.generate_expression(operand)?;

        match op {
            UnaryOp::Negate => {
                // Two's complement negation: EOR #$FF, CLC, ADC #1
                self.emit_imm(opcodes::EOR_IMM, 0xFF);
                self.emit_byte(opcodes::CLC);
                self.emit_imm(opcodes::ADC_IMM, 1);
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
                            self.emit_jsr_label("__print_str");
                        }
                        Type::Word => {
                            self.emit_jsr_label("__print_word");
                        }
                        Type::Byte | Type::Bool | Type::Sbyte => {
                            self.emit_jsr_label("__print_byte");
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
                    // X position
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::TAX);
                    // Y position
                    self.generate_expression(&args[1])?;
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
                // Simple input routine - read until RETURN
                // For now, just wait for a key
                let read_label = self.make_label("readln");
                self.define_label(&read_label);
                self.emit_abs(opcodes::JSR, kernal::GETIN);
                self.emit_byte(opcodes::CMP_IMM);
                self.emit_byte(petscii::RETURN);
                self.emit_branch(opcodes::BNE, &read_label);
            }
            "poke" => {
                if args.len() >= 2 {
                    // Value to poke
                    self.generate_expression(&args[1])?;
                    self.emit_byte(opcodes::PHA);
                    // Address
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    // High byte of address (if word)
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_byte(opcodes::STA_ZP);
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
                    // Address
                    self.generate_expression(&args[0])?;
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::LDA_IMM, 0);
                    self.emit_byte(opcodes::STA_ZP);
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

    /// Emit code to load a value from an address into A.
    fn emit_load_from_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::LDA_ABS, address);
            }
            Type::Word | Type::Sword => {
                self.emit_abs(opcodes::LDA_ABS, address);
                self.emit_abs(opcodes::LDX_ABS, address.wrapping_add(1));
            }
            _ => {
                // For other types, just load the address
                self.emit_abs(opcodes::LDA_ABS, address);
            }
        }
    }

    /// Emit code to store A to an address.
    fn emit_store_to_address(&mut self, address: u16, var_type: &Type) {
        match var_type {
            Type::Byte | Type::Sbyte | Type::Bool => {
                self.emit_abs(opcodes::STA_ABS, address);
            }
            Type::Word | Type::Sword => {
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
}
