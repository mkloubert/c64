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

//! Operator checking for the semantic analyzer.
//!
//! This module provides type checking for binary and unary operators:
//! - Arithmetic operators (+, -, *, /, %)
//! - Comparison operators (==, !=, <, >, <=, >=)
//! - Logical operators (and, or, not)
//! - Bitwise operators (&, |, ^, ~, <<, >>)

use super::Analyzer;
use crate::ast::{BinaryOp, Type, UnaryOp};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for operator type checking.
pub trait OperatorChecker {
    /// Check binary operator types and return result type.
    fn check_binary_op(
        &mut self,
        left: &Option<Type>,
        op: &BinaryOp,
        right: &Option<Type>,
        span: &Span,
    ) -> Option<Type>;

    /// Check binary operator types (with concrete types) and return result type.
    fn check_binary_op_types(
        &mut self,
        left: &Type,
        op: &BinaryOp,
        right: &Type,
        span: &Span,
    ) -> Option<Type>;

    /// Check unary operator types and return result type.
    fn check_unary_op(&mut self, op: &UnaryOp, operand: &Option<Type>, span: &Span)
        -> Option<Type>;
}

impl OperatorChecker for Analyzer {
    fn check_binary_op(
        &mut self,
        left: &Option<Type>,
        op: &BinaryOp,
        right: &Option<Type>,
        span: &Span,
    ) -> Option<Type> {
        let (left, right) = match (left, right) {
            (Some(l), Some(r)) => (l, r),
            _ => return None,
        };

        self.check_binary_op_types(left, op, right, span)
    }

    fn check_binary_op_types(
        &mut self,
        left: &Type,
        op: &BinaryOp,
        right: &Type,
        span: &Span,
    ) -> Option<Type> {
        match op {
            // Arithmetic operators - valid for all numeric types (integers, fixed, float)
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if !left.is_numeric() || !right.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Operator {} requires numeric operands, found {} and {}",
                            op, left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Modulo - valid for integers and fixed, but NOT for float
            BinaryOp::Mod => {
                if left.is_float() || right.is_float() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Modulo operator is not valid for float type, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                if !left.is_numeric() || !right.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Operator {} requires numeric operands, found {} and {}",
                            op, left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Comparison operators - valid for all numeric types
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual => {
                // Allow comparisons between any numeric types (promotion handled by binary_result_type)
                if left.is_numeric() && right.is_numeric() {
                    // Valid comparison
                } else if left != right && Type::binary_result_type(left, right).is_none() {
                    self.error(CompileError::new(
                        ErrorCode::CannotCompareTypes,
                        format!("Cannot compare {} and {}", left, right),
                        span.clone(),
                    ));
                }
                Some(Type::Bool)
            }

            // Logical operators - only valid for bool
            BinaryOp::And | BinaryOp::Or => {
                if *left != Type::Bool || *right != Type::Bool {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Logical operators require boolean operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Some(Type::Bool)
            }

            // Bitwise operators - only valid for integers (NOT fixed/float)
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                if !left.is_integer() || !right.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Bitwise operators require integer operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Shift operators - only valid for integers (NOT fixed/float)
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                if !left.is_integer() || !right.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Shift operators require integer operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Some(left.clone())
            }
        }
    }

    fn check_unary_op(
        &mut self,
        op: &UnaryOp,
        operand: &Option<Type>,
        span: &Span,
    ) -> Option<Type> {
        let operand = operand.as_ref()?;

        match op {
            UnaryOp::Negate => {
                if !operand.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("Cannot negate non-numeric type {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                // Negation promotes unsigned integers to signed type
                // Fixed and float stay the same (already signed)
                match operand {
                    Type::Byte => Some(Type::Sbyte),
                    Type::Word => Some(Type::Sword),
                    Type::Fixed => Some(Type::Fixed),
                    Type::Float => Some(Type::Float),
                    _ => Some(operand.clone()),
                }
            }
            UnaryOp::Not => {
                if *operand != Type::Bool {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("'not' requires boolean operand, found {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            UnaryOp::BitNot => {
                if !operand.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("Bitwise NOT requires integer operand, found {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                Some(operand.clone())
            }
        }
    }
}
