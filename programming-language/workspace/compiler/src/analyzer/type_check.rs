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

//! Type checking utilities for the semantic analyzer.
//!
//! This module provides type checking functionality:
//! - Expression type determination (for code generation)
//! - Assignment compatibility checking
//! - Constant evaluation at compile time
//! - Value range checking
//!
//! Note: Type inference for declarations has been removed.
//! All variable and constant declarations now require explicit type annotations.

use super::Analyzer;
use crate::ast::{BinaryOp, Expr, ExprKind, Type, UnaryOp};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for type checking utilities.
pub trait TypeChecker {
    /// Determine the natural type of an expression based on its value.
    ///
    /// This is used by the code generator to determine appropriate
    /// machine code when the expression type needs to be inferred
    /// from the value (e.g., for array literals or intermediate calculations).
    ///
    /// Note: This is NOT used for variable/constant declarations,
    /// which now require explicit type annotations.
    ///
    /// For literals:
    /// - Integer literal → byte (0-255) or word (256-65535)
    /// - Negative integer → sbyte (-128 to -1) or sword
    /// - Decimal literal → float (default)
    /// - Boolean → bool
    /// - String → string
    fn infer_type_from_expr(&self, expr: &Expr, expr_type: &Type) -> Type;

    /// Check if an expression can be assigned to a target type.
    ///
    /// This handles the special case of `DecimalLiteral` which can be
    /// assigned to both `fixed` and `float` types (type is determined by context).
    fn is_expr_assignable_to(
        &self,
        expr_kind: &ExprKind,
        value_type: &Type,
        target_type: &Type,
    ) -> bool;

    /// Try to evaluate a constant expression at compile time.
    /// Returns the evaluated value as i64 to handle both signed and unsigned.
    fn try_eval_constant(&self, expr: &Expr) -> Option<i64>;

    /// Check if a value is within the valid range for a type.
    fn check_value_in_range(&mut self, value: i64, target_type: &Type, span: &Span);
}

impl TypeChecker for Analyzer {
    fn infer_type_from_expr(&self, expr: &Expr, expr_type: &Type) -> Type {
        match &expr.kind {
            ExprKind::IntegerLiteral(v) => {
                // Apply type inference rules based on value range
                if *v <= 255 {
                    Type::Byte
                } else {
                    Type::Word
                }
            }
            ExprKind::UnaryOp {
                op: UnaryOp::Negate,
                operand,
            } => {
                // Handle negative literals
                if let ExprKind::IntegerLiteral(v) = &operand.kind {
                    let neg_value = -(*v as i32);
                    if neg_value >= -128 {
                        Type::Sbyte
                    } else {
                        Type::Sword
                    }
                } else if let ExprKind::DecimalLiteral(_) = &operand.kind {
                    // Negative decimal → float
                    Type::Float
                } else {
                    // For other negated expressions, use the analyzed type
                    expr_type.clone()
                }
            }
            ExprKind::DecimalLiteral(_) => {
                // Decimal literals default to float for type inference
                Type::Float
            }
            ExprKind::BoolLiteral(_) => Type::Bool,
            ExprKind::StringLiteral(_) => Type::String,
            ExprKind::CharLiteral(_) => Type::Byte,
            _ => {
                // For complex expressions, use the analyzed type
                expr_type.clone()
            }
        }
    }

    fn is_expr_assignable_to(
        &self,
        expr_kind: &ExprKind,
        value_type: &Type,
        target_type: &Type,
    ) -> bool {
        // DecimalLiteral can be assigned to both fixed and float
        if matches!(expr_kind, ExprKind::DecimalLiteral(_))
            && (target_type.is_fixed() || target_type.is_float())
        {
            return true;
        }

        // Handle negated decimal literals: -3.5 etc.
        if let ExprKind::UnaryOp {
            op: UnaryOp::Negate,
            operand,
        } = expr_kind
        {
            if matches!(operand.kind, ExprKind::DecimalLiteral(_))
                && (target_type.is_fixed() || target_type.is_float())
            {
                return true;
            }
        }

        // Integer literals can be assigned to compatible signed types if in range
        // This allows `x: sbyte = 127` (literal fits in sbyte range)
        // but prevents `x: sbyte = byte_var` (variable could be 128-255)
        if let ExprKind::IntegerLiteral(value) = expr_kind {
            let v = *value as i64;
            match target_type {
                Type::Sbyte if (-128..=127).contains(&v) => return true,
                Type::Sword if (-32768..=32767).contains(&v) => return true,
                Type::Fixed if (-2048..=2047).contains(&v) => return true,
                _ => {}
            }
        }

        // Handle negated integer literals: -10, etc.
        if let ExprKind::UnaryOp {
            op: UnaryOp::Negate,
            operand,
        } = expr_kind
        {
            if let ExprKind::IntegerLiteral(value) = &operand.kind {
                let v = -(*value as i64);
                match target_type {
                    Type::Sbyte if (-128..=127).contains(&v) => return true,
                    Type::Sword if (-32768..=32767).contains(&v) => return true,
                    Type::Fixed if (-2048..=2047).contains(&v) => return true,
                    _ => {}
                }
            }
        }

        // Default to standard type assignability
        value_type.is_assignable_to(target_type)
    }

    fn try_eval_constant(&self, expr: &Expr) -> Option<i64> {
        match &expr.kind {
            ExprKind::IntegerLiteral(n) => Some(*n as i64),
            ExprKind::BoolLiteral(b) => Some(if *b { 1 } else { 0 }),
            ExprKind::CharLiteral(c) => Some(*c as i64),
            ExprKind::UnaryOp { op, operand } => {
                let operand_val = self.try_eval_constant(operand)?;
                match op {
                    UnaryOp::Negate => Some(-operand_val),
                    UnaryOp::Not => Some(if operand_val == 0 { 1 } else { 0 }),
                    UnaryOp::BitNot => Some(!operand_val),
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                let left_val = self.try_eval_constant(left)?;
                let right_val = self.try_eval_constant(right)?;
                match op {
                    BinaryOp::Add => Some(left_val.wrapping_add(right_val)),
                    BinaryOp::Sub => Some(left_val.wrapping_sub(right_val)),
                    BinaryOp::Mul => Some(left_val.wrapping_mul(right_val)),
                    BinaryOp::Div => {
                        if right_val == 0 {
                            None
                        } else {
                            Some(left_val / right_val)
                        }
                    }
                    BinaryOp::Mod => {
                        if right_val == 0 {
                            None
                        } else {
                            Some(left_val % right_val)
                        }
                    }
                    BinaryOp::BitAnd => Some(left_val & right_val),
                    BinaryOp::BitOr => Some(left_val | right_val),
                    BinaryOp::BitXor => Some(left_val ^ right_val),
                    BinaryOp::ShiftLeft => Some(left_val << (right_val & 0x3F)),
                    BinaryOp::ShiftRight => Some(left_val >> (right_val & 0x3F)),
                    _ => None, // Comparison/logical ops don't produce numeric results
                }
            }
            ExprKind::Grouped(inner) => self.try_eval_constant(inner),
            ExprKind::Identifier(name) => {
                // Try to look up constant value
                if let Some(symbol) = self.symbols.lookup(name) {
                    if symbol.is_constant {
                        // For now, we don't track constant values, return None
                        None
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_value_in_range(&mut self, value: i64, target_type: &Type, span: &Span) {
        let (min, max, type_name) = match target_type {
            Type::Byte => (0, 255, "byte"),
            Type::Word => (0, 65535, "word"),
            Type::Sbyte => (-128, 127, "sbyte"),
            Type::Sword => (-32768, 32767, "sword"),
            Type::Bool => (0, 1, "bool"),
            _ => return, // No range check for other types
        };

        if value < min || value > max {
            self.error(
                CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    format!(
                        "Value {} is out of range for {} ({} to {})",
                        value, type_name, min, max
                    ),
                    span.clone(),
                )
                .with_hint(format!(
                    "Valid range for {} is {} to {}",
                    type_name, min, max
                )),
            );
        }
    }
}
