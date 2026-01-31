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

//! Expression AST nodes for the Cobra64 compiler.

use crate::error::Span;

/// Convert a 12.4 fixed-point internal value to a display string.
///
/// # Examples
/// - 60 (3.75 × 16) → "3.75"
/// - -24 (-1.5 × 16) → "-1.5"
/// - 16 (1.0 × 16) → "1.0"
/// - 0 → "0.0"
pub fn fixed_to_string(value: i16) -> String {
    let is_negative = value < 0;
    let abs_value = value.unsigned_abs() as u32;

    // Integer part: upper 12 bits (value / 16)
    let int_part = abs_value >> 4;

    // Fractional part: lower 4 bits
    // Each bit represents: 0.5, 0.25, 0.125, 0.0625
    // Multiply by 625 to get 4 decimal digits (0.0625 × 10000 = 625)
    let frac_bits = abs_value & 0xF;
    let frac_decimal = frac_bits * 625; // 0-9375

    // Format with trailing zeros trimmed
    let frac_str = if frac_decimal == 0 {
        "0".to_string()
    } else {
        // Format as 4 digits, then trim trailing zeros
        let s = format!("{:04}", frac_decimal);
        s.trim_end_matches('0').to_string()
    };

    let sign = if is_negative { "-" } else { "" };
    format!("{}{}.{}", sign, int_part, frac_str)
}

/// Convert an IEEE-754 binary16 value to a display string.
///
/// # Format
/// - Sign: bit 15
/// - Exponent: bits 14-10 (bias 15)
/// - Mantissa: bits 9-0 (implicit leading 1 for normalized)
///
/// # Special values
/// - 0x0000: "0.0"
/// - 0x8000: "-0.0" (displayed as "0.0")
/// - 0x7C00: "inf"
/// - 0xFC00: "-inf"
/// - 0x7Cxx (mantissa != 0): "nan"
pub fn float_to_string(bits: u16) -> String {
    let sign = (bits >> 15) & 1;
    let exponent = (bits >> 10) & 0x1F;
    let mantissa = bits & 0x3FF;

    // Special cases
    if exponent == 0 && mantissa == 0 {
        // Zero (positive or negative, display as "0.0")
        return "0.0".to_string();
    }

    if exponent == 31 {
        // Infinity or NaN
        if mantissa == 0 {
            return if sign == 1 {
                "-inf".to_string()
            } else {
                "inf".to_string()
            };
        } else {
            return "nan".to_string();
        }
    }

    // Convert to f64 for display
    let value = binary16_to_f64(bits);

    // Format with reasonable precision
    let sign_str = if sign == 1 { "-" } else { "" };
    let abs_value = value.abs();

    // Use scientific notation for very small or very large values
    if abs_value != 0.0 && !(0.001..10000.0).contains(&abs_value) {
        format!("{}{:e}", sign_str, abs_value)
    } else {
        // Regular decimal notation, trim unnecessary trailing zeros
        let formatted = format!("{:.4}", abs_value);
        let trimmed = formatted.trim_end_matches('0');
        let result = if trimmed.ends_with('.') {
            format!("{}0", trimmed)
        } else {
            trimmed.to_string()
        };
        format!("{}{}", sign_str, result)
    }
}

/// Convert IEEE-754 binary16 bits to f64 for display purposes.
fn binary16_to_f64(bits: u16) -> f64 {
    let sign = ((bits >> 15) & 1) as i32;
    let exponent = ((bits >> 10) & 0x1F) as i32;
    let mantissa = (bits & 0x3FF) as u64;

    if exponent == 0 {
        if mantissa == 0 {
            // Zero
            return if sign == 1 { -0.0 } else { 0.0 };
        }
        // Subnormal number
        let value = (mantissa as f64) * 2.0_f64.powi(-24);
        return if sign == 1 { -value } else { value };
    }

    if exponent == 31 {
        if mantissa == 0 {
            // Infinity
            return if sign == 1 {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        // NaN
        return f64::NAN;
    }

    // Normalized number
    // value = (-1)^sign × 2^(exponent-15) × (1 + mantissa/1024)
    let mantissa_value = 1.0 + (mantissa as f64) / 1024.0;
    let value = mantissa_value * 2.0_f64.powi(exponent - 15);

    if sign == 1 {
        -value
    } else {
        value
    }
}

/// An expression in the Cobra64 language.
#[derive(Debug, Clone)]
pub struct Expr {
    /// The kind of expression.
    pub kind: ExprKind,
    /// The source span of this expression.
    pub span: Span,
}

impl Expr {
    /// Create a new expression.
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The kind of expression.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// An integer literal.
    IntegerLiteral(u16),

    /// A fixed-point literal (12.4 format).
    /// Stores the internal representation: value × 16.
    /// Range: -2048.0 to +2047.9375 (internal: -32768 to +32767).
    FixedLiteral(i16),

    /// A floating-point literal (IEEE-754 binary16).
    /// Stores the raw 16-bit IEEE-754 binary16 representation.
    /// Range: ±65504.
    FloatLiteral(u16),

    /// A decimal literal that hasn't been resolved to fixed or float yet.
    /// The parser creates this, and the analyzer converts it to
    /// FixedLiteral or FloatLiteral based on context.
    DecimalLiteral(String),

    /// A string literal.
    StringLiteral(String),

    /// A character literal.
    CharLiteral(char),

    /// A boolean literal.
    BoolLiteral(bool),

    /// A variable or constant reference.
    Identifier(String),

    /// A binary operation.
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    /// A unary operation.
    UnaryOp { op: UnaryOp, operand: Box<Expr> },

    /// A function call.
    FunctionCall { name: String, args: Vec<Expr> },

    /// An array index access.
    ArrayIndex { array: Box<Expr>, index: Box<Expr> },

    /// A type cast expression (e.g., `byte(x)`).
    TypeCast {
        target_type: super::Type,
        expr: Box<Expr>,
    },

    /// A parenthesized expression.
    Grouped(Box<Expr>),
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = binds tighter).
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual => 3,
            BinaryOp::BitOr => 4,
            BinaryOp::BitXor => 5,
            BinaryOp::BitAnd => 6,
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => 7,
            BinaryOp::Add | BinaryOp::Sub => 8,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 9,
        }
    }

    /// Check if this operator is left-associative.
    pub fn is_left_associative(&self) -> bool {
        true // All binary operators are left-associative
    }

    /// Get a string representation of this operator.
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Arithmetic negation (`-x`).
    Negate,
    /// Logical NOT (`not x`).
    Not,
    /// Bitwise NOT (`~x`).
    BitNot,
}

impl UnaryOp {
    /// Get a string representation of this operator.
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "not",
            UnaryOp::BitNot => "~",
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::fmt::Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprKind::IntegerLiteral(n) => write!(f, "{}", n),
            ExprKind::FixedLiteral(n) => write!(f, "{}", fixed_to_string(*n)),
            ExprKind::FloatLiteral(bits) => write!(f, "{}", float_to_string(*bits)),
            ExprKind::DecimalLiteral(s) => write!(f, "{}", s),
            ExprKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            ExprKind::CharLiteral(c) => write!(f, "'{}'", c),
            ExprKind::BoolLiteral(b) => write!(f, "{}", b),
            ExprKind::Identifier(name) => write!(f, "{}", name),
            ExprKind::BinaryOp { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            ExprKind::UnaryOp { op, operand } => {
                write!(f, "({}{})", op, operand)
            }
            ExprKind::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprKind::ArrayIndex { array, index } => {
                write!(f, "{}[{}]", array, index)
            }
            ExprKind::TypeCast { target_type, expr } => {
                write!(f, "{}({})", target_type, expr)
            }
            ExprKind::Grouped(expr) => write!(f, "({})", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::Add.precedence() > BinaryOp::Equal.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
    }

    #[test]
    fn test_expression_creation() {
        let span = Span::new(0, 5);
        let expr = Expr::new(ExprKind::IntegerLiteral(42), span);
        assert!(matches!(expr.kind, ExprKind::IntegerLiteral(42)));
    }

    #[test]
    fn test_display_integer_literal() {
        let expr = Expr::new(ExprKind::IntegerLiteral(42), Span::new(0, 2));
        assert_eq!(format!("{}", expr), "42");
    }

    #[test]
    fn test_display_string_literal() {
        let expr = Expr::new(
            ExprKind::StringLiteral("hello".to_string()),
            Span::new(0, 7),
        );
        assert_eq!(format!("{}", expr), "\"hello\"");
    }

    #[test]
    fn test_display_char_literal() {
        let expr = Expr::new(ExprKind::CharLiteral('A'), Span::new(0, 3));
        assert_eq!(format!("{}", expr), "'A'");
    }

    #[test]
    fn test_display_bool_literal() {
        let expr_true = Expr::new(ExprKind::BoolLiteral(true), Span::new(0, 4));
        let expr_false = Expr::new(ExprKind::BoolLiteral(false), Span::new(0, 5));
        assert_eq!(format!("{}", expr_true), "true");
        assert_eq!(format!("{}", expr_false), "false");
    }

    #[test]
    fn test_display_identifier() {
        let expr = Expr::new(ExprKind::Identifier("my_var".to_string()), Span::new(0, 6));
        assert_eq!(format!("{}", expr), "my_var");
    }

    #[test]
    fn test_display_binary_op() {
        let left = Box::new(Expr::new(ExprKind::IntegerLiteral(1), Span::new(0, 1)));
        let right = Box::new(Expr::new(ExprKind::IntegerLiteral(2), Span::new(4, 5)));
        let expr = Expr::new(
            ExprKind::BinaryOp {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        );
        assert_eq!(format!("{}", expr), "(1 + 2)");
    }

    #[test]
    fn test_display_unary_op() {
        let operand = Box::new(Expr::new(ExprKind::IntegerLiteral(5), Span::new(1, 2)));
        let expr = Expr::new(
            ExprKind::UnaryOp {
                op: UnaryOp::Negate,
                operand,
            },
            Span::new(0, 2),
        );
        assert_eq!(format!("{}", expr), "(-5)");
    }

    #[test]
    fn test_display_function_call() {
        let arg1 = Expr::new(ExprKind::IntegerLiteral(1), Span::new(4, 5));
        let arg2 = Expr::new(ExprKind::IntegerLiteral(2), Span::new(7, 8));
        let expr = Expr::new(
            ExprKind::FunctionCall {
                name: "foo".to_string(),
                args: vec![arg1, arg2],
            },
            Span::new(0, 9),
        );
        assert_eq!(format!("{}", expr), "foo(1, 2)");
    }

    #[test]
    fn test_display_array_index() {
        let array = Box::new(Expr::new(
            ExprKind::Identifier("arr".to_string()),
            Span::new(0, 3),
        ));
        let index = Box::new(Expr::new(ExprKind::IntegerLiteral(0), Span::new(4, 5)));
        let expr = Expr::new(ExprKind::ArrayIndex { array, index }, Span::new(0, 6));
        assert_eq!(format!("{}", expr), "arr[0]");
    }

    #[test]
    fn test_display_type_cast() {
        let inner = Box::new(Expr::new(ExprKind::IntegerLiteral(256), Span::new(5, 8)));
        let expr = Expr::new(
            ExprKind::TypeCast {
                target_type: super::super::Type::Byte,
                expr: inner,
            },
            Span::new(0, 9),
        );
        assert_eq!(format!("{}", expr), "byte(256)");
    }

    #[test]
    fn test_display_grouped() {
        let inner = Box::new(Expr::new(ExprKind::IntegerLiteral(42), Span::new(1, 3)));
        let expr = Expr::new(ExprKind::Grouped(inner), Span::new(0, 4));
        assert_eq!(format!("{}", expr), "(42)");
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(format!("{}", BinaryOp::Add), "+");
        assert_eq!(format!("{}", BinaryOp::Sub), "-");
        assert_eq!(format!("{}", BinaryOp::Mul), "*");
        assert_eq!(format!("{}", BinaryOp::Equal), "==");
        assert_eq!(format!("{}", BinaryOp::And), "and");
        assert_eq!(format!("{}", BinaryOp::Or), "or");
        assert_eq!(format!("{}", BinaryOp::BitAnd), "&");
        assert_eq!(format!("{}", BinaryOp::ShiftLeft), "<<");
    }

    #[test]
    fn test_unary_op_display() {
        assert_eq!(format!("{}", UnaryOp::Negate), "-");
        assert_eq!(format!("{}", UnaryOp::Not), "not");
        assert_eq!(format!("{}", UnaryOp::BitNot), "~");
    }

    #[test]
    fn test_all_binary_ops_have_as_str() {
        let ops = [
            BinaryOp::Add,
            BinaryOp::Sub,
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::Mod,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::Greater,
            BinaryOp::LessEqual,
            BinaryOp::GreaterEqual,
            BinaryOp::And,
            BinaryOp::Or,
            BinaryOp::BitAnd,
            BinaryOp::BitOr,
            BinaryOp::BitXor,
            BinaryOp::ShiftLeft,
            BinaryOp::ShiftRight,
        ];
        for op in &ops {
            assert!(!op.as_str().is_empty());
            assert!(op.is_left_associative());
        }
    }

    // ========================================
    // Fixed-Point Literal Tests
    // ========================================

    #[test]
    fn test_fixed_to_string_positive() {
        // 3.75 × 16 = 60
        assert_eq!(super::fixed_to_string(60), "3.75");
        // 1.0 × 16 = 16
        assert_eq!(super::fixed_to_string(16), "1.0");
        // 0.5 × 16 = 8
        assert_eq!(super::fixed_to_string(8), "0.5");
        // 0.25 × 16 = 4
        assert_eq!(super::fixed_to_string(4), "0.25");
        // 0.0625 × 16 = 1
        assert_eq!(super::fixed_to_string(1), "0.0625");
        // 100.5 × 16 = 1608
        assert_eq!(super::fixed_to_string(1608), "100.5");
    }

    #[test]
    fn test_fixed_to_string_negative() {
        // -1.5 × 16 = -24
        assert_eq!(super::fixed_to_string(-24), "-1.5");
        // -3.75 × 16 = -60
        assert_eq!(super::fixed_to_string(-60), "-3.75");
        // -0.5 × 16 = -8
        assert_eq!(super::fixed_to_string(-8), "-0.5");
    }

    #[test]
    fn test_fixed_to_string_zero() {
        assert_eq!(super::fixed_to_string(0), "0.0");
    }

    #[test]
    fn test_fixed_to_string_whole_numbers() {
        // 10.0 × 16 = 160
        assert_eq!(super::fixed_to_string(160), "10.0");
        // 255.0 × 16 = 4080
        assert_eq!(super::fixed_to_string(4080), "255.0");
    }

    #[test]
    fn test_fixed_to_string_boundaries() {
        // Maximum: 2047.9375 × 16 = 32767
        assert_eq!(super::fixed_to_string(32767), "2047.9375");
        // Minimum: -2048.0 × 16 = -32768
        assert_eq!(super::fixed_to_string(-32768), "-2048.0");
    }

    #[test]
    fn test_display_fixed_literal() {
        let expr = Expr::new(ExprKind::FixedLiteral(60), Span::new(0, 4));
        assert_eq!(format!("{}", expr), "3.75");
    }

    #[test]
    fn test_display_fixed_literal_negative() {
        let expr = Expr::new(ExprKind::FixedLiteral(-24), Span::new(0, 4));
        assert_eq!(format!("{}", expr), "-1.5");
    }

    // ========================================
    // Float Literal Tests
    // ========================================

    #[test]
    fn test_float_to_string_zero() {
        // Positive zero: 0x0000
        assert_eq!(super::float_to_string(0x0000), "0.0");
        // Negative zero: 0x8000 (displayed as "0.0")
        assert_eq!(super::float_to_string(0x8000), "0.0");
    }

    #[test]
    fn test_float_to_string_one() {
        // 1.0 in binary16: sign=0, exp=15 (01111), mantissa=0
        // bits = 0_01111_0000000000 = 0x3C00
        assert_eq!(super::float_to_string(0x3C00), "1.0");
    }

    #[test]
    fn test_float_to_string_negative_one() {
        // -1.0 in binary16: sign=1, exp=15 (01111), mantissa=0
        // bits = 1_01111_0000000000 = 0xBC00
        assert_eq!(super::float_to_string(0xBC00), "-1.0");
    }

    #[test]
    fn test_float_to_string_two() {
        // 2.0 in binary16: sign=0, exp=16 (10000), mantissa=0
        // bits = 0_10000_0000000000 = 0x4000
        assert_eq!(super::float_to_string(0x4000), "2.0");
    }

    #[test]
    fn test_float_to_string_half() {
        // 0.5 in binary16: sign=0, exp=14 (01110), mantissa=0
        // bits = 0_01110_0000000000 = 0x3800
        assert_eq!(super::float_to_string(0x3800), "0.5");
    }

    #[test]
    fn test_float_to_string_special_values() {
        // Positive infinity: 0x7C00
        assert_eq!(super::float_to_string(0x7C00), "inf");
        // Negative infinity: 0xFC00
        assert_eq!(super::float_to_string(0xFC00), "-inf");
        // NaN: 0x7C01 (any mantissa != 0 with exp=31)
        assert_eq!(super::float_to_string(0x7C01), "nan");
        assert_eq!(super::float_to_string(0x7E00), "nan");
    }

    #[test]
    fn test_float_to_string_pi_approx() {
        // π ≈ 3.140625 in binary16
        // sign=0, exp=16 (10000), mantissa=0x248 (584)
        // bits = 0_10000_1001001000 = 0x4248
        let s = super::float_to_string(0x4248);
        assert!(s.starts_with("3.14"));
    }

    #[test]
    fn test_display_float_literal() {
        let expr = Expr::new(ExprKind::FloatLiteral(0x3C00), Span::new(0, 3));
        assert_eq!(format!("{}", expr), "1.0");
    }

    #[test]
    fn test_display_float_literal_special() {
        let expr_inf = Expr::new(ExprKind::FloatLiteral(0x7C00), Span::new(0, 3));
        assert_eq!(format!("{}", expr_inf), "inf");

        let expr_nan = Expr::new(ExprKind::FloatLiteral(0x7C01), Span::new(0, 3));
        assert_eq!(format!("{}", expr_nan), "nan");
    }

    #[test]
    fn test_binary16_to_f64_normalized() {
        // 1.0
        assert_eq!(super::binary16_to_f64(0x3C00), 1.0);
        // 2.0
        assert_eq!(super::binary16_to_f64(0x4000), 2.0);
        // 0.5
        assert_eq!(super::binary16_to_f64(0x3800), 0.5);
        // -1.0
        assert_eq!(super::binary16_to_f64(0xBC00), -1.0);
    }

    #[test]
    fn test_binary16_to_f64_special() {
        // Zero
        assert_eq!(super::binary16_to_f64(0x0000), 0.0);
        // Infinity
        assert!(super::binary16_to_f64(0x7C00).is_infinite());
        assert!(super::binary16_to_f64(0x7C00).is_sign_positive());
        // Negative infinity
        assert!(super::binary16_to_f64(0xFC00).is_infinite());
        assert!(super::binary16_to_f64(0xFC00).is_sign_negative());
        // NaN
        assert!(super::binary16_to_f64(0x7C01).is_nan());
    }
}
