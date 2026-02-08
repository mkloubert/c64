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
use crate::error::{CompileError, CompileWarning, ErrorCode, Span, WarningCode};

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

    /// Check for potentially dangerous type casts and emit warnings.
    fn check_type_cast_warnings(
        &mut self,
        source_type: &Type,
        target_type: &Type,
        inner_expr: &Expr,
        span: &Span,
    );
}

/// Check if an expression is a DecimalLiteral (including through groupings).
fn is_decimal_literal(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::DecimalLiteral(_) => true,
        ExprKind::Grouped(inner) => is_decimal_literal(inner),
        _ => false,
    }
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
                // Handle DecimalLiteral type adaptation:
                // If one operand is a DecimalLiteral and the other is Fixed,
                // treat the DecimalLiteral as Fixed to preserve type consistency.
                let left_is_decimal = is_decimal_literal(left);
                let right_is_decimal = is_decimal_literal(right);

                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                // Adapt DecimalLiteral type based on the other operand
                let (adapted_left, adapted_right) = match (&left_type, &right_type) {
                    (Some(Type::Float), Some(Type::Fixed)) if left_is_decimal => {
                        // Left is DecimalLiteral, right is Fixed -> treat left as Fixed
                        (Some(Type::Fixed), right_type)
                    }
                    (Some(Type::Fixed), Some(Type::Float)) if right_is_decimal => {
                        // Left is Fixed, right is DecimalLiteral -> treat right as Fixed
                        (left_type, Some(Type::Fixed))
                    }
                    _ => (left_type, right_type),
                };

                self.check_binary_op(&adapted_left, op, &adapted_right, &expr.span)
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
                let source_type = self.analyze_expression(inner);

                // Check for potentially dangerous casts and emit warnings
                if let Some(ref src_type) = source_type {
                    self.check_type_cast_warnings(src_type, target_type, inner, &expr.span);
                }

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
        let mut has_fixed = false;
        let mut has_float = false;
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
                    Type::Fixed => {
                        has_fixed = true;
                    }
                    Type::Float => {
                        has_float = true;
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

        // Check for mixed types
        let type_count = [has_bool, has_integer, has_fixed, has_float]
            .iter()
            .filter(|&&x| x)
            .count();

        if type_count > 1 {
            // Determine what types are mixed
            let mut types_present = Vec::new();
            if has_bool {
                types_present.push("bool");
            }
            if has_integer {
                types_present.push("integer");
            }
            if has_fixed {
                types_present.push("fixed");
            }
            if has_float {
                types_present.push("float");
            }
            self.error(CompileError::new(
                ErrorCode::ArrayElementTypeMismatch,
                format!(
                    "Cannot mix {} elements in array literal",
                    types_present.join(" and ")
                ),
                span.clone(),
            ));
            return None;
        }

        let array_len = elements.len() as u16;

        // Determine array type based on element types
        if has_bool {
            Some(Type::BoolArray(Some(array_len)))
        } else if has_fixed {
            Some(Type::FixedArray(Some(array_len)))
        } else if has_float {
            Some(Type::FloatArray(Some(array_len)))
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
            | Type::SwordArray(Some(n))
            | Type::FixedArray(Some(n))
            | Type::FloatArray(Some(n)) => Some(*n as usize),
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

    fn check_type_cast_warnings(
        &mut self,
        source_type: &Type,
        target_type: &Type,
        inner_expr: &Expr,
        span: &Span,
    ) {
        // Try to get constant value for literal checks
        let const_value = self.try_eval_constant(inner_expr);

        match (source_type, target_type) {
            // Warning: Truncation when casting to smaller type
            (Type::Word, Type::Byte) | (Type::Sword, Type::Sbyte) => {
                if let Some(val) = const_value {
                    let truncated = (val & 0xFF) as i8 as i64;
                    if val != truncated && val > 255 {
                        self.warning(CompileWarning::new(
                            WarningCode::LiteralTruncation,
                            format!(
                                "Value {} will be truncated to {} when cast to {}",
                                val,
                                val & 0xFF,
                                target_type
                            ),
                            span.clone(),
                        ));
                    }
                }
            }

            // Warning: Large integer to float may lose precision
            (Type::Word, Type::Float) | (Type::Sword, Type::Float) => {
                if let Some(val) = const_value {
                    // IEEE-754 binary16 has 11 bits of mantissa precision
                    // Values > 2048 may lose precision
                    if val.abs() > 2048 {
                        self.warning(CompileWarning::new(
                            WarningCode::PrecisionLoss,
                            format!(
                                "Large value {} may lose precision when converted to float",
                                val
                            ),
                            span.clone(),
                        ));
                    }
                }
            }

            // Warning: Value overflows fixed-point range
            (Type::Word, Type::Fixed) | (Type::Sword, Type::Fixed) => {
                if let Some(val) = const_value {
                    if !(-2048..=2047).contains(&val) {
                        self.warning(CompileWarning::new(
                            WarningCode::FixedPointOverflow,
                            format!(
                                "Value {} overflows fixed-point range (-2048 to 2047)",
                                val
                            ),
                            span.clone(),
                        ));
                    }
                }
            }

            // Warning: Negative value cast to unsigned (same size)
            (Type::Sbyte, Type::Byte) | (Type::Sword, Type::Word) => {
                if let Some(val) = const_value {
                    if val < 0 {
                        let wrapped = if *target_type == Type::Byte {
                            (val as i8 as u8) as i64
                        } else {
                            (val as i16 as u16) as i64
                        };
                        self.warning(CompileWarning::new(
                            WarningCode::NegativeToUnsigned,
                            format!(
                                "Negative value {} will wrap to {} when cast to {}",
                                val, wrapped, target_type
                            ),
                            span.clone(),
                        ));
                    }
                } else {
                    // Variable conversion - warn about potential issues
                    self.warning(CompileWarning::new(
                        WarningCode::SignedToUnsigned,
                        format!(
                            "Converting {} to {} may produce unexpected results for negative values",
                            source_type, target_type
                        ),
                        span.clone(),
                    ));
                }
            }

            // Warning: Signed to wider unsigned may change value (for variables)
            (Type::Sbyte, Type::Word) => {
                if const_value.is_none() {
                    self.warning(CompileWarning::new(
                        WarningCode::SignedToUnsigned,
                        format!(
                            "Converting {} to {} may produce unexpected results for negative values",
                            source_type, target_type
                        ),
                        span.clone(),
                    ));
                }
            }

            _ => {}
        }
    }
}
