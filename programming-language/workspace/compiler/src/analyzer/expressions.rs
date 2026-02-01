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

//! Expression analysis for the semantic analyzer.
//!
//! This module provides expression analysis functionality:
//! - Expression type analysis
//! - Array literal analysis and type inference
//! - Array size and element validation

use super::functions::FunctionAnalyzer;
use super::operators::OperatorChecker;
use super::type_check::TypeChecker;
use super::Analyzer;
use crate::ast::{Expr, ExprKind, Type};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for expression analysis.
pub trait ExpressionAnalyzer {
    /// Analyze an expression and return its type.
    fn analyze_expression(&mut self, expr: &Expr) -> Option<Type>;

    /// Analyze an array literal and infer its type.
    fn analyze_array_literal(&mut self, elements: &[Expr], span: &Span) -> Option<Type>;

    /// Check that array literal size matches declared array size.
    fn check_array_literal_size(&mut self, array_type: &Type, literal_len: usize, span: &Span);

    /// Check that array literal elements are compatible with declared element type.
    fn check_array_literal_elements(&mut self, array_type: &Type, elements: &[Expr]);
}

impl ExpressionAnalyzer for Analyzer {
    fn analyze_expression(&mut self, expr: &Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::IntegerLiteral(n) => {
                if *n <= 255 {
                    Some(Type::Byte)
                } else {
                    Some(Type::Word)
                }
            }
            ExprKind::StringLiteral(_) => Some(Type::String),
            ExprKind::CharLiteral(_) => Some(Type::Byte),
            ExprKind::BoolLiteral(_) => Some(Type::Bool),
            ExprKind::Identifier(name) => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    symbol.get_type().cloned()
                } else {
                    self.error(CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        expr.span.clone(),
                    ));
                    None
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);
                self.check_binary_op(&left_type, op, &right_type, &expr.span)
            }
            ExprKind::UnaryOp { op, operand } => {
                let operand_type = self.analyze_expression(operand);
                self.check_unary_op(op, &operand_type, &expr.span)
            }
            ExprKind::FunctionCall { name, args } => {
                self.analyze_function_call(name, args, &expr.span)
            }
            ExprKind::ArrayIndex { array, index } => {
                let array_type = self.analyze_expression(array);
                let index_type = self.analyze_expression(index);

                // Check index is integer
                if let Some(index_type) = index_type {
                    if !index_type.is_integer() {
                        self.error(CompileError::new(
                            ErrorCode::ArrayIndexMustBeInteger,
                            format!("Array index must be an integer, found {}", index_type),
                            index.span.clone(),
                        ));
                    }
                }

                // Return element type
                array_type.and_then(|t| t.element_type())
            }
            ExprKind::TypeCast {
                target_type,
                expr: inner,
            } => {
                self.analyze_expression(inner);
                Some(target_type.clone())
            }
            ExprKind::FixedLiteral(_) => Some(Type::Fixed),
            ExprKind::FloatLiteral(_) => Some(Type::Float),
            ExprKind::DecimalLiteral(_) => {
                // DecimalLiteral defaults to float type during analysis.
                // Context-based conversion (e.g., for fixed variables) will be
                // handled in a later phase during assignment checking.
                Some(Type::Float)
            }
            ExprKind::Grouped(inner) => self.analyze_expression(inner),
            ExprKind::ArrayLiteral { elements } => self.analyze_array_literal(elements, &expr.span),
        }
    }

    fn analyze_array_literal(&mut self, elements: &[Expr], span: &Span) -> Option<Type> {
        if elements.is_empty() {
            self.error(CompileError::new(
                ErrorCode::CannotInferArrayType,
                "Empty array literal requires explicit type annotation",
                span.clone(),
            ));
            return None;
        }

        // Track element info for type inference
        let mut has_bool = false;
        let mut has_integer = false;
        let mut has_negative = false;
        let mut max_value: i64 = 0;
        let mut min_value: i64 = 0;
        let mut has_error = false;

        for elem in elements {
            if let Some(elem_type) = self.analyze_expression(elem) {
                match elem_type {
                    Type::Bool => {
                        has_bool = true;
                    }
                    Type::Byte => {
                        has_integer = true;
                        // Try to get the actual value for better inference
                        if let Some(val) = self.try_eval_constant(elem) {
                            if val < 0 {
                                has_negative = true;
                                min_value = min_value.min(val);
                            } else {
                                max_value = max_value.max(val);
                            }
                        }
                    }
                    Type::Word => {
                        has_integer = true;
                        if let Some(val) = self.try_eval_constant(elem) {
                            if val < 0 {
                                has_negative = true;
                                min_value = min_value.min(val);
                            } else {
                                max_value = max_value.max(val);
                            }
                        } else {
                            // Assume word-sized if we can't evaluate
                            max_value = max_value.max(256);
                        }
                    }
                    Type::Sbyte | Type::Sword => {
                        has_integer = true;
                        if let Some(val) = self.try_eval_constant(elem) {
                            if val < 0 {
                                has_negative = true;
                                min_value = min_value.min(val);
                            } else {
                                max_value = max_value.max(val);
                            }
                        } else {
                            has_negative = true; // Assume might be negative
                        }
                    }
                    Type::Fixed | Type::Float => {
                        self.error(CompileError::new(
                            ErrorCode::ArrayElementTypeMismatch,
                            format!("Array literals do not support {} elements", elem_type),
                            elem.span.clone(),
                        ));
                        has_error = true;
                    }
                    Type::String => {
                        self.error(CompileError::new(
                            ErrorCode::ArrayElementTypeMismatch,
                            "Array literals do not support string elements",
                            elem.span.clone(),
                        ));
                        has_error = true;
                    }
                    _ => {
                        self.error(CompileError::new(
                            ErrorCode::ArrayElementTypeMismatch,
                            format!("Unsupported element type in array literal: {}", elem_type),
                            elem.span.clone(),
                        ));
                        has_error = true;
                    }
                }
            } else {
                has_error = true;
            }
        }

        if has_error {
            return None;
        }

        // Check for mixed types (bools and integers)
        if has_bool && has_integer {
            self.error(CompileError::new(
                ErrorCode::ArrayElementTypeMismatch,
                "Cannot mix boolean and integer elements in array literal",
                span.clone(),
            ));
            return None;
        }

        let array_len = elements.len() as u16;

        // Determine array type based on value ranges
        if has_bool {
            Some(Type::BoolArray(Some(array_len)))
        } else if has_negative {
            // Signed array type inference:
            // - All values fit in sbyte (-128 to 127) → sbyte[]
            // - Otherwise → sword[]
            if min_value >= -128 && max_value <= 127 {
                Some(Type::SbyteArray(Some(array_len)))
            } else if min_value >= -32768 && max_value <= 32767 {
                Some(Type::SwordArray(Some(array_len)))
            } else {
                self.error(CompileError::new(
                    ErrorCode::ArrayElementTypeMismatch,
                    format!(
                        "Array literal values out of range: {} to {} (max sword range is -32768 to 32767)",
                        min_value, max_value
                    ),
                    span.clone(),
                ));
                None
            }
        } else if max_value <= 255 {
            Some(Type::ByteArray(Some(array_len)))
        } else if max_value <= 65535 {
            Some(Type::WordArray(Some(array_len)))
        } else {
            self.error(CompileError::new(
                ErrorCode::ArrayElementTypeMismatch,
                format!(
                    "Array literal value {} exceeds maximum word value (65535)",
                    max_value
                ),
                span.clone(),
            ));
            None
        }
    }

    fn check_array_literal_size(&mut self, array_type: &Type, literal_len: usize, span: &Span) {
        let declared_size = match array_type {
            Type::ByteArray(Some(n))
            | Type::WordArray(Some(n))
            | Type::BoolArray(Some(n))
            | Type::SbyteArray(Some(n))
            | Type::SwordArray(Some(n)) => Some(*n as usize),
            _ => None,
        };

        if let Some(declared) = declared_size {
            if literal_len > declared {
                self.error(CompileError::new(
                    ErrorCode::ArrayInitTooManyElements,
                    format!(
                        "Array literal has {} elements but array is declared with size {}",
                        literal_len, declared
                    ),
                    span.clone(),
                ));
            } else if literal_len < declared {
                self.error(CompileError::new(
                    ErrorCode::ArrayInitTooFewElements,
                    format!(
                        "Array literal has {} elements but array is declared with size {}",
                        literal_len, declared
                    ),
                    span.clone(),
                ));
            }
        }
    }

    fn check_array_literal_elements(&mut self, array_type: &Type, elements: &[Expr]) {
        let element_type = match array_type.element_type() {
            Some(t) => t,
            None => return,
        };

        for elem in elements {
            // Get the value if it's a constant
            if let Some(val) = self.try_eval_constant(elem) {
                match element_type {
                    Type::Byte => {
                        if !(0..=255).contains(&val) {
                            self.error(CompileError::new(
                                ErrorCode::ConstantValueOutOfRange,
                                format!("Value {} is out of range for byte (0-255)", val),
                                elem.span.clone(),
                            ));
                        }
                    }
                    Type::Word => {
                        if !(0..=65535).contains(&val) {
                            self.error(CompileError::new(
                                ErrorCode::ConstantValueOutOfRange,
                                format!("Value {} is out of range for word (0-65535)", val),
                                elem.span.clone(),
                            ));
                        }
                    }
                    Type::Sbyte => {
                        if !(-128..=127).contains(&val) {
                            self.error(CompileError::new(
                                ErrorCode::ConstantValueOutOfRange,
                                format!("Value {} is out of range for sbyte (-128 to 127)", val),
                                elem.span.clone(),
                            ));
                        }
                    }
                    Type::Sword => {
                        if !(-32768..=32767).contains(&val) {
                            self.error(CompileError::new(
                                ErrorCode::ConstantValueOutOfRange,
                                format!(
                                    "Value {} is out of range for sword (-32768 to 32767)",
                                    val
                                ),
                                elem.span.clone(),
                            ));
                        }
                    }
                    Type::Bool => {
                        // Bool elements should already be validated by type checking
                    }
                    _ => {}
                }
            }
        }
    }
}
