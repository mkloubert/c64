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

//! Expression code generation.
//!
//! This module provides code generation for all expression types:
//! - Literals (integer, bool, char, string, fixed, float)
//! - Identifiers (variable access)
//! - Binary and unary operations
//! - Function calls
//! - Type casts
//! - Array indexing

use super::binary_ops::BinaryOpsEmitter;
use super::conversions::TypeConversions;
use super::emit::EmitHelpers;
use super::functions::FunctionCallEmitter;
use super::mos6510::{opcodes, zeropage};
use super::strings::StringManager;
use super::type_inference::TypeInference;
use super::types::decimal_string_to_binary16;
use super::unary_ops::UnaryOpsEmitter;
use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{Expr, ExprKind, Type};
use crate::error::{CompileError, ErrorCode};

/// Extension trait for expression code generation.
pub trait ExpressionEmitter {
    /// Generate code for an expression.
    ///
    /// Result is left in A register (for byte) or A/X (for word, A=low, X=high).
    fn generate_expression(&mut self, expr: &Expr) -> Result<(), CompileError>;
}

impl ExpressionEmitter for CodeGenerator {
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
                let var = self.get_variable(name).ok_or_else(|| {
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
                    let var = self.get_variable(name).ok_or_else(|| {
                        CompileError::new(
                            ErrorCode::UndefinedVariable,
                            format!("Undefined array '{}'", name),
                            expr.span.clone(),
                        )
                    })?;

                    let is_word_array =
                        matches!(var.var_type, Type::WordArray(_) | Type::SwordArray(_));

                    // Generate index expression
                    self.generate_expression(index)?;

                    if is_word_array {
                        // For word arrays, multiply index by 2 (ASL A)
                        self.emit_byte(opcodes::ASL_ACC);
                    }

                    self.emit_byte(opcodes::TAY); // Y = index (or index*2 for word arrays)

                    // Load base address into TMP1
                    self.emit_imm(opcodes::LDA_IMM, (var.address & 0xFF) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1);
                    self.emit_imm(opcodes::LDA_IMM, (var.address >> 8) as u8);
                    self.emit_byte(opcodes::STA_ZP);
                    self.emit_byte(zeropage::TMP1_HI);

                    if is_word_array {
                        // Load 16-bit value: low byte to A, high byte to X
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::PHA); // Save low byte
                        self.emit_byte(opcodes::INY); // Point to high byte
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                        self.emit_byte(opcodes::TAX); // X = high byte
                        self.emit_byte(opcodes::PLA); // A = low byte
                    } else {
                        // Load 8-bit value (byte, bool)
                        self.emit_byte(opcodes::LDA_IZY);
                        self.emit_byte(zeropage::TMP1);
                    }
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
            ExprKind::ArrayLiteral { .. } => {
                // Array literals are handled during variable initialization.
                // If we reach here, it means the array literal is used in an
                // expression context that isn't supported yet.
                return Err(CompileError::new(
                    ErrorCode::NotImplemented,
                    "Array literals in expression context not yet implemented",
                    expr.span.clone(),
                ));
            }
        }
        Ok(())
    }
}
