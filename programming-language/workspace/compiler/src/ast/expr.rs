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
}
