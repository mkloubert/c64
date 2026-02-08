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

//! Type utilities for code generation.
//!
//! This module provides methods for determining types from expressions
//! during code generation. It includes:
//! - Expression type determination (for intermediate calculations)
//! - Array length extraction
//! - Helper utilities for array literal analysis
//!
//! Note: Type inference for declarations has been removed.
//! All variable and constant declarations now require explicit type annotations.
//! The `infer_type_from_expr` function is kept for determining types of
//! sub-expressions during code generation (e.g., array literals, function calls).

use super::variables::VariableManager;
use super::CodeGenerator;
use crate::ast::{Expr, ExprKind, Type, UnaryOp};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for type operations during code generation.
///
/// This trait provides methods for determining types from expressions
/// and extracting array information. Note that this is NOT used for
/// variable/constant declarations (which require explicit types).
pub trait TypeInference {
    /// Determine the type of an expression.
    ///
    /// This is used during code generation to determine the appropriate
    /// machine code to emit for sub-expressions (e.g., array literals,
    /// binary operations, function calls).
    ///
    /// Note: This is NOT used for variable/constant declarations,
    /// which now require explicit type annotations.
    fn infer_type_from_expr(&self, expr: &Expr) -> Type;

    /// Get the length of an array from an expression.
    ///
    /// Returns the array size as a u16. The argument should be an
    /// identifier referencing an array variable.
    fn get_array_length(&self, expr: &Expr, span: &Span) -> Result<u16, CompileError>;

    /// Check if all array elements are zero (or false for bools).
    ///
    /// This is used for optimization - zero-filled arrays can be
    /// initialized more efficiently.
    fn all_elements_are_zero(&self, elements: &[Expr]) -> bool;
}

impl TypeInference for CodeGenerator {
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
                if let Some(var) = self.get_variable(name) {
                    var.var_type
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
            ExprKind::ArrayLiteral { elements } => {
                // Infer array type from elements
                if elements.is_empty() {
                    Type::ByteArray(Some(0))
                } else {
                    let first_type = self.infer_type_from_expr(&elements[0]);
                    let len = elements.len() as u16;
                    match first_type {
                        Type::Bool => Type::BoolArray(Some(len)),
                        Type::Word => Type::WordArray(Some(len)),
                        Type::Sbyte => Type::SbyteArray(Some(len)),
                        Type::Sword => Type::SwordArray(Some(len)),
                        Type::Fixed => Type::FixedArray(Some(len)),
                        Type::Float => Type::FloatArray(Some(len)),
                        _ => Type::ByteArray(Some(len)),
                    }
                }
            }
            ExprKind::ArrayIndex { array, .. } => {
                // Return the element type of the array
                if let ExprKind::Identifier(name) = &array.kind {
                    if let Some(var) = self.get_variable(name) {
                        var.var_type.element_type().unwrap_or(Type::Byte)
                    } else {
                        Type::Byte
                    }
                } else {
                    Type::Byte
                }
            }
            ExprKind::FunctionCall { name, args } => {
                // Return type for built-in functions
                match name.as_str() {
                    "rand" | "rand_fixed" => Type::Fixed,
                    "rand_byte" | "peek" | "get_key" => Type::Byte,
                    "rand_sbyte" => Type::Sbyte,
                    "rand_word" | "peek_word" | "len" => Type::Word,
                    "rand_sword" => Type::Sword,
                    "read" | "readln" => Type::String,
                    _ => {
                        // For user-defined functions, check the function table
                        if let Some(func) = self.functions.get(name) {
                            func.return_type.clone().unwrap_or(Type::Byte)
                        } else {
                            // For functions with typed arguments, infer from first arg
                            if !args.is_empty() {
                                self.infer_type_from_expr(&args[0])
                            } else {
                                Type::Byte
                            }
                        }
                    }
                }
            }
            _ => Type::Byte, // Default to byte
        }
    }

    fn get_array_length(&self, expr: &Expr, span: &Span) -> Result<u16, CompileError> {
        // The argument should be an identifier referencing an array variable
        if let ExprKind::Identifier(name) = &expr.kind {
            if let Some(var) = self.get_variable(name) {
                // Get the size from the array type
                match &var.var_type {
                    Type::ByteArray(Some(size))
                    | Type::WordArray(Some(size))
                    | Type::BoolArray(Some(size))
                    | Type::SbyteArray(Some(size))
                    | Type::SwordArray(Some(size))
                    | Type::FixedArray(Some(size))
                    | Type::FloatArray(Some(size)) => Ok(*size),
                    Type::ByteArray(None)
                    | Type::WordArray(None)
                    | Type::BoolArray(None)
                    | Type::SbyteArray(None)
                    | Type::SwordArray(None)
                    | Type::FixedArray(None)
                    | Type::FloatArray(None) => {
                        // Array without known size - should not happen after analysis
                        Err(CompileError::new(
                            ErrorCode::TypeMismatch,
                            format!("Cannot determine size of array '{}'", name),
                            span.clone(),
                        ))
                    }
                    _ => Err(CompileError::new(
                        ErrorCode::TypeMismatch,
                        format!("'{}' is not an array", name),
                        span.clone(),
                    )),
                }
            } else {
                Err(CompileError::new(
                    ErrorCode::UndefinedVariable,
                    format!("Undefined variable '{}'", name),
                    span.clone(),
                ))
            }
        } else {
            Err(CompileError::new(
                ErrorCode::TypeMismatch,
                "len() requires an array variable".to_string(),
                span.clone(),
            ))
        }
    }

    fn all_elements_are_zero(&self, elements: &[Expr]) -> bool {
        elements.iter().all(|elem| match &elem.kind {
            ExprKind::IntegerLiteral(v) => *v == 0,
            ExprKind::BoolLiteral(b) => !b,
            _ => false,
        })
    }
}
