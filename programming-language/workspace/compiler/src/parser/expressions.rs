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

//! Expression parsing for the parser.
//!
//! This module provides expression parsing functionality:
//! - Precedence climbing for binary operators
//! - Unary operators
//! - Primary expressions (literals, identifiers, grouping)
//! - Postfix expressions (function calls, array indexing)

use super::helpers::ParserHelpers;
use super::Parser;
use crate::ast::{BinaryOp, Expr, ExprKind, Type, UnaryOp};
use crate::error::{CompileError, ErrorCode};
use crate::lexer::Token;

/// Extension trait for expression parsing.
pub trait ExpressionParser {
    /// Parse an expression.
    fn parse_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse an 'or' expression.
    fn parse_or_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse an 'and' expression.
    fn parse_and_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a comparison expression.
    fn parse_comparison_expression(&mut self) -> Result<Expr, CompileError>;

    /// Try to parse a comparison operator.
    fn try_parse_comparison_op(&mut self) -> Option<BinaryOp>;

    /// Parse a bitwise OR expression.
    fn parse_bitor_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a bitwise XOR expression.
    fn parse_bitxor_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a bitwise AND expression.
    fn parse_bitand_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a shift expression.
    fn parse_shift_expression(&mut self) -> Result<Expr, CompileError>;

    /// Try to parse a shift operator.
    fn try_parse_shift_op(&mut self) -> Option<BinaryOp>;

    /// Parse an additive expression.
    fn parse_additive_expression(&mut self) -> Result<Expr, CompileError>;

    /// Try to parse an additive operator.
    fn try_parse_additive_op(&mut self) -> Option<BinaryOp>;

    /// Parse a multiplicative expression.
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, CompileError>;

    /// Try to parse a multiplicative operator.
    fn try_parse_multiplicative_op(&mut self) -> Option<BinaryOp>;

    /// Parse a unary expression.
    fn parse_unary_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a postfix expression (function calls, array indexing).
    fn parse_postfix_expression(&mut self) -> Result<Expr, CompileError>;

    /// Parse a function call.
    fn parse_function_call(&mut self, callee: Expr) -> Result<Expr, CompileError>;

    /// Parse array indexing.
    fn parse_array_index(&mut self, array: Expr) -> Result<Expr, CompileError>;

    /// Parse a primary expression.
    fn parse_primary_expression(&mut self) -> Result<Expr, CompileError>;
}

impl<'a> ExpressionParser for Parser<'a> {
    fn parse_expression(&mut self) -> Result<Expr, CompileError> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_and_expression()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::Or,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_comparison_expression()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_comparison_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::And,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_bitor_expression()?;

        while let Some(op) = self.try_parse_comparison_op() {
            let right = self.parse_bitor_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn try_parse_comparison_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::EqualEqual => BinaryOp::Equal,
            Token::BangEqual => BinaryOp::NotEqual,
            Token::Less => BinaryOp::Less,
            Token::Greater => BinaryOp::Greater,
            Token::LessEqual => BinaryOp::LessEqual,
            Token::GreaterEqual => BinaryOp::GreaterEqual,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_bitor_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_bitxor_expression()?;

        while self.check(&Token::Pipe) {
            self.advance();
            let right = self.parse_bitxor_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::BitOr,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_bitxor_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_bitand_expression()?;

        while self.check(&Token::Caret) {
            self.advance();
            let right = self.parse_bitand_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::BitXor,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_bitand_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_shift_expression()?;

        while self.check(&Token::Ampersand) {
            self.advance();
            let right = self.parse_shift_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOp::BitAnd,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn parse_shift_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_additive_expression()?;

        while let Some(op) = self.try_parse_shift_op() {
            let right = self.parse_additive_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn try_parse_shift_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::ShiftLeft => BinaryOp::ShiftLeft,
            Token::ShiftRight => BinaryOp::ShiftRight,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_additive_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.try_parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn try_parse_additive_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Sub,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_unary_expression()?;

        while let Some(op) = self.try_parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = left.span.merge(&right.span);
            left = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(left)
    }

    fn try_parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::Star => BinaryOp::Mul,
            Token::Slash => BinaryOp::Div,
            Token::Percent => BinaryOp::Mod,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_unary_expression(&mut self) -> Result<Expr, CompileError> {
        let start_span = self.peek_span().unwrap_or_else(|| self.previous_span());

        if self.check(&Token::Minus) {
            self.advance();
            let operand = self.parse_unary_expression()?;
            let span = start_span.merge(&operand.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        if self.check(&Token::Not) {
            self.advance();
            let operand = self.parse_unary_expression()?;
            let span = start_span.merge(&operand.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        if self.check(&Token::Tilde) {
            self.advance();
            let operand = self.parse_unary_expression()?;
            let span = start_span.merge(&operand.span);
            return Ok(Expr::new(
                ExprKind::UnaryOp {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                },
                span,
            ));
        }

        self.parse_postfix_expression()
    }

    fn parse_postfix_expression(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if self.check(&Token::LeftParen) {
                // Function call
                expr = self.parse_function_call(expr)?;
            } else if self.check(&Token::LeftBracket) {
                // Array indexing
                expr = self.parse_array_index(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_function_call(&mut self, callee: Expr) -> Result<Expr, CompileError> {
        let name = match &callee.kind {
            ExprKind::Identifier(name) => name.clone(),
            _ => {
                return Err(CompileError::new(
                    ErrorCode::InvalidFunctionCall,
                    "Expected function name",
                    callee.span,
                ))
            }
        };

        self.expect(&Token::LeftParen, "Expected '(' for function call")?;

        let mut args = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        let (_, end_span) = self.expect(&Token::RightParen, "Expected ')' after arguments")?;
        let span = callee.span.merge(&end_span);

        Ok(Expr::new(ExprKind::FunctionCall { name, args }, span))
    }

    fn parse_array_index(&mut self, array: Expr) -> Result<Expr, CompileError> {
        self.expect(&Token::LeftBracket, "Expected '['")?;
        let index = self.parse_expression()?;
        let (_, end_span) = self.expect(&Token::RightBracket, "Expected ']'")?;

        let span = array.span.merge(&end_span);

        Ok(Expr::new(
            ExprKind::ArrayIndex {
                array: Box::new(array),
                index: Box::new(index),
            },
            span,
        ))
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, CompileError> {
        let (token, span) = self
            .advance()
            .ok_or_else(|| self.error(ErrorCode::UnexpectedEndOfFile, "Unexpected end of file"))?;

        match token {
            Token::Integer(n) => Ok(Expr::new(ExprKind::IntegerLiteral(n), span)),
            Token::Decimal(s) => Ok(Expr::new(ExprKind::DecimalLiteral(s), span)),
            Token::String(s) => Ok(Expr::new(ExprKind::StringLiteral(s), span)),
            Token::Char(c) => Ok(Expr::new(ExprKind::CharLiteral(c), span)),
            Token::True => Ok(Expr::new(ExprKind::BoolLiteral(true), span)),
            Token::False => Ok(Expr::new(ExprKind::BoolLiteral(false), span)),
            Token::Identifier(name) => Ok(Expr::new(ExprKind::Identifier(name), span)),

            // Type cast expressions
            Token::Byte
            | Token::Word
            | Token::Sbyte
            | Token::Sword
            | Token::Fixed
            | Token::Float
            | Token::Bool
            | Token::Str => {
                let target_type = match token {
                    Token::Byte => Type::Byte,
                    Token::Word => Type::Word,
                    Token::Sbyte => Type::Sbyte,
                    Token::Sword => Type::Sword,
                    Token::Fixed => Type::Fixed,
                    Token::Float => Type::Float,
                    Token::Bool => Type::Bool,
                    Token::Str => Type::String,
                    _ => unreachable!(),
                };

                self.expect(&Token::LeftParen, "Expected '(' for type cast")?;
                let expr = self.parse_expression()?;
                let (_, end_span) = self.expect(&Token::RightParen, "Expected ')'")?;

                let full_span = span.merge(&end_span);
                Ok(Expr::new(
                    ExprKind::TypeCast {
                        target_type,
                        expr: Box::new(expr),
                    },
                    full_span,
                ))
            }

            // Parenthesized expression
            Token::LeftParen => {
                let inner = self.parse_expression()?;
                let (_, end_span) = self.expect(&Token::RightParen, "Expected ')'")?;
                let full_span = span.merge(&end_span);
                Ok(Expr::new(ExprKind::Grouped(Box::new(inner)), full_span))
            }

            // Array literal
            Token::LeftBracket => {
                let mut elements = Vec::new();

                // Check for empty array
                if !self.check(&Token::RightBracket) {
                    // Parse first element
                    elements.push(self.parse_expression()?);

                    // Parse remaining elements
                    while self.match_token(&Token::Comma) {
                        // Allow trailing comma
                        if self.check(&Token::RightBracket) {
                            break;
                        }
                        elements.push(self.parse_expression()?);
                    }
                }

                let (_, end_span) = self.expect(&Token::RightBracket, "Expected ']'")?;
                let full_span = span.merge(&end_span);
                Ok(Expr::new(ExprKind::ArrayLiteral { elements }, full_span))
            }

            _ => Err(CompileError::new(
                ErrorCode::UnexpectedToken,
                format!("Unexpected token in expression: {}", token),
                span,
            )),
        }
    }
}
