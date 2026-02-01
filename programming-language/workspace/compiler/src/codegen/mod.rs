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

//! Code generation module for the Cobra64 compiler.
//!
//! This module generates 6510 machine code from the analyzed AST.
//! It handles:
//! - Instruction encoding
//! - Memory layout
//! - Expression evaluation
//! - Control flow
//! - Built-in functions
//!
//! # Module Structure
//!
//! The code generator is organized into the following submodules:
//! - `assignments` - Assignment code generation (AssignmentEmitter trait)
//! - `binary_ops` - Binary operation code generation (BinaryOpsEmitter trait)
//! - `comparisons` - Signed and fixed-point comparison helpers (ComparisonHelpers trait)
//! - `constants` - Memory layout constants for the C64
//! - `control_flow` - Control flow code generation (ControlFlowEmitter trait)
//! - `conversions` - Type conversion code generation (TypeConversions trait)
//! - `declarations` - Declaration code generation (DeclarationEmitter trait)
//! - `emit` - Low-level byte emission helpers (EmitHelpers trait)
//! - `expressions` - Expression code generation (ExpressionEmitter trait)
//! - `float_runtime` - IEEE-754 float runtime routines
//! - `functions` - Function call code generation (FunctionCallEmitter trait)
//! - `labels` - Label and branch management (LabelManager trait)
//! - `mos6510` - 6510 CPU opcodes, addresses and constants
//! - `runtime` - C64 runtime library routines (RuntimeEmitter trait)
//! - `strings` - String literal management (StringManager trait)
//! - `type_inference` - Type inference utilities (TypeInference trait)
//! - `types` - Type conversion utilities
//! - `unary_ops` - Unary operation code generation (UnaryOpsEmitter trait)
//! - `variables` - Variable and function structures (VariableManager trait)

// Submodules
pub mod assignments;
pub mod binary_ops;
pub mod comparisons;
pub mod constants;
pub mod control_flow;
pub mod conversions;
pub mod declarations;
pub mod emit;
pub mod expressions;
pub mod float_runtime;
pub mod functions;
pub mod labels;
pub mod mos6510;
pub mod runtime;
pub mod strings;
pub mod type_inference;
pub mod types;
pub mod unary_ops;
pub mod variables;

// Re-exports for public API
pub use constants::{BASIC_STUB_SIZE, CODE_START, PROGRAM_START};
pub use types::{decimal_string_to_binary16, f64_to_binary16};

// Internal imports from submodules
use assignments::AssignmentEmitter;
use control_flow::ControlFlowEmitter;
use declarations::DeclarationEmitter;
use emit::EmitHelpers;
use expressions::ExpressionEmitter;
use labels::{LabelManager, LoopContext, PendingBranch, PendingJump};
use runtime::RuntimeEmitter;
use strings::{PendingStringRef, StringManager};
use type_inference::TypeInference;
use variables::{Function, Variable, VariableManager};

use crate::ast::{Block, Program, Statement, StatementKind, TopLevelItem, Type};
use crate::error::CompileError;
use mos6510::opcodes;
use std::collections::HashMap;

/// The code generator for the 6510 CPU.
pub struct CodeGenerator {
    /// The generated machine code.
    pub(crate) code: Vec<u8>,
    /// Current code address.
    pub(crate) current_address: u16,
    /// Variable allocation table.
    pub(crate) variables: HashMap<String, Variable>,
    /// Function table.
    pub(crate) functions: HashMap<String, Function>,
    /// String literals (just the bytes, addresses resolved later).
    pub(crate) strings: Vec<Vec<u8>>,
    /// Pending string references to resolve.
    pub(crate) pending_string_refs: Vec<PendingStringRef>,
    /// Label addresses (resolved).
    pub(crate) labels: HashMap<String, u16>,
    /// Pending branches to resolve.
    pub(crate) pending_branches: Vec<PendingBranch>,
    /// Pending jumps to resolve.
    pub(crate) pending_jumps: Vec<PendingJump>,
    /// Next variable address (grows from end of code).
    pub(crate) next_var_address: u16,
    /// Label counter for generating unique labels.
    pub(crate) label_counter: u32,
    /// Current loop context stack.
    pub(crate) loop_stack: Vec<LoopContext>,
    /// Runtime library included flag.
    pub(crate) runtime_included: bool,
    /// Runtime routine addresses.
    pub(crate) runtime_addresses: HashMap<String, u16>,
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

    /// Generate code for a program.
    pub fn generate(&mut self, program: &Program) -> Result<Vec<u8>, CompileError> {
        // Emit BASIC stub
        self.emit_basic_stub();

        // First pass: collect function and variable info
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => {
                    // Allocate parameter addresses and register function
                    let mut param_addresses = Vec::new();
                    for param in &func.params {
                        let addr = self.allocate_variable(&param.name, &param.param_type, false);
                        param_addresses.push(addr);
                    }
                    self.functions.insert(
                        func.name.clone(),
                        Function {
                            address: 0,
                            params: func.params.iter().map(|p| p.param_type.clone()).collect(),
                            param_addresses,
                            return_type: func.return_type.clone(),
                        },
                    );
                }
                TopLevelItem::Variable(decl) => {
                    // Use explicit type or infer from initializer
                    let var_type = if let Some(ref t) = decl.var_type {
                        t.clone()
                    } else if let Some(ref init) = decl.initializer {
                        self.infer_type_from_expr(init)
                    } else {
                        Type::Byte // Fallback (analyzer should have caught this)
                    };
                    self.allocate_variable(&decl.name, &var_type, false);
                }
                TopLevelItem::Constant(decl) => {
                    // Use explicit type or infer from value
                    let const_type = if let Some(ref t) = decl.const_type {
                        t.clone()
                    } else {
                        self.infer_type_from_expr(&decl.value)
                    };
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
                        let var = self.get_variable(&decl.name).unwrap();
                        self.emit_store_to_address(var.address, &var.var_type);
                    }
                }
                TopLevelItem::Constant(decl) => {
                    self.generate_expression(&decl.value)?;
                    let var = self.get_variable(&decl.name).unwrap();
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
    use crate::ast::{Expr, ExprKind, UnaryOp};
    use crate::codegen::comparisons::ComparisonHelpers;
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
