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

//! Statement parsing for the parser.
//!
//! This module provides statement parsing functionality:
//! - Variable declarations (with explicit type annotation)
//! - Constant declarations (with explicit type annotation)
//! - Assignment statements (simple and compound)
//! - Expression statements

use super::control_flow::ControlFlowParser;
use super::expressions::ExpressionParser;
use super::helpers::ParserHelpers;
use super::types::TypeParser;
use super::Parser;
use crate::ast::{
    AssignOp, AssignTarget, Assignment, ConstDecl, Expr, ExprKind, Statement, StatementKind, Type,
    VarDecl,
};
use crate::error::{CompileError, ErrorCode};
use crate::lexer::Token;

/// Extension trait for statement parsing.
pub trait StatementParser {
    /// Parse a statement.
    fn parse_statement(&mut self) -> Result<Statement, CompileError>;

    /// Parse a statement starting with an identifier.
    fn parse_identifier_statement(&mut self) -> Result<Statement, CompileError>;

    /// Convert an expression to an assignment target.
    fn expr_to_assign_target(&self, expr: Expr) -> Result<AssignTarget, CompileError>;

    /// Try to parse an assignment operator.
    fn try_parse_assign_op(&mut self) -> Option<AssignOp>;

    /// Parse a variable declaration.
    fn parse_var_decl(&mut self) -> Result<VarDecl, CompileError>;

    /// Parse a constant declaration.
    fn parse_const_decl(&mut self) -> Result<ConstDecl, CompileError>;
}

impl<'a> StatementParser for Parser<'a> {
    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();

        match self.peek() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::While) => self.parse_while_statement(),
            Some(Token::For) => self.parse_for_statement(),
            Some(Token::Break) => {
                self.advance();
                let span = start_span;
                Ok(Statement::new(StatementKind::Break, span))
            }
            Some(Token::Continue) => {
                self.advance();
                let span = start_span;
                Ok(Statement::new(StatementKind::Continue, span))
            }
            Some(Token::Return) => self.parse_return_statement(),
            Some(Token::Pass) => {
                self.advance();
                let span = start_span;
                Ok(Statement::new(StatementKind::Pass, span))
            }
            Some(Token::Const) => {
                // Constant declaration inside function
                let decl = self.parse_const_decl()?;
                let span = decl.span.clone();
                Ok(Statement::new(StatementKind::ConstDecl(decl), span))
            }
            Some(Token::Identifier(_)) => {
                // Could be variable declaration, assignment, or expression
                self.parse_identifier_statement()
            }
            _ => {
                // Try to parse as expression statement
                let expr = self.parse_expression()?;
                let span = expr.span.clone();
                Ok(Statement::new(StatementKind::Expression(expr), span))
            }
        }
    }

    fn parse_identifier_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();

        // Check if it's a variable declaration (name: type)
        if matches!(self.peek_ahead(1), Some(Token::Colon)) {
            // Check if the token after colon is a type
            if matches!(
                self.peek_ahead(2),
                Some(Token::Byte)
                    | Some(Token::Word)
                    | Some(Token::Sbyte)
                    | Some(Token::Sword)
                    | Some(Token::Bool)
                    | Some(Token::StringType)
                    | Some(Token::Fixed)
                    | Some(Token::Float)
            ) {
                let decl = self.parse_var_decl()?;
                let span = decl.span.clone();
                return Ok(Statement::new(StatementKind::VarDecl(decl), span));
            }
        }

        // Parse as expression first
        let expr = self.parse_expression()?;

        // Check for assignment
        if let Some(assign_op) = self.try_parse_assign_op() {
            let target = self.expr_to_assign_target(expr)?;
            let value = self.parse_expression()?;
            let end_span = value.span.clone();
            let span = start_span.merge(&end_span);

            let assignment = Assignment {
                target,
                op: assign_op,
                value,
                span: span.clone(),
            };

            Ok(Statement::new(StatementKind::Assignment(assignment), span))
        } else {
            // Just an expression statement
            let span = expr.span.clone();
            Ok(Statement::new(StatementKind::Expression(expr), span))
        }
    }

    fn expr_to_assign_target(&self, expr: Expr) -> Result<AssignTarget, CompileError> {
        match expr.kind {
            ExprKind::Identifier(name) => Ok(AssignTarget::Variable(name)),
            ExprKind::ArrayIndex { array, index } => {
                if let ExprKind::Identifier(name) = array.kind {
                    Ok(AssignTarget::ArrayElement { name, index })
                } else {
                    Err(CompileError::new(
                        ErrorCode::InvalidAssignmentTarget,
                        "Invalid assignment target",
                        expr.span,
                    ))
                }
            }
            _ => Err(CompileError::new(
                ErrorCode::InvalidAssignmentTarget,
                "Invalid assignment target",
                expr.span,
            )),
        }
    }

    fn try_parse_assign_op(&mut self) -> Option<AssignOp> {
        let op = match self.peek()? {
            Token::Equal => AssignOp::Assign,
            Token::PlusAssign => AssignOp::AddAssign,
            Token::MinusAssign => AssignOp::SubAssign,
            Token::StarAssign => AssignOp::MulAssign,
            Token::SlashAssign => AssignOp::DivAssign,
            Token::PercentAssign => AssignOp::ModAssign,
            Token::AmpersandAssign => AssignOp::BitAndAssign,
            Token::PipeAssign => AssignOp::BitOrAssign,
            Token::CaretAssign => AssignOp::BitXorAssign,
            Token::ShiftLeftAssign => AssignOp::ShiftLeftAssign,
            Token::ShiftRightAssign => AssignOp::ShiftRightAssign,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_var_decl(&mut self) -> Result<VarDecl, CompileError> {
        let start_span = self.peek_span().unwrap();

        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected variable name")),
        };

        // Type annotation is now required
        if !self.match_token(&Token::Colon) {
            return Err(self.error(
                ErrorCode::MissingTypeAnnotation,
                "Variable declaration requires explicit type annotation",
            ).with_hint("Add a type annotation, e.g.: x: byte = 10"));
        }

        let var_type = self.parse_type()?;

        // Check for array size in declaration
        let array_size = match &var_type {
            Type::ByteArray(Some(n)) | Type::WordArray(Some(n)) => Some(*n),
            _ => None,
        };

        // Optional initializer
        let initializer = if self.match_token(&Token::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.previous_span();
        let span = start_span.merge(&end_span);

        let mut decl = VarDecl::new(name, var_type, span);

        if let Some(init) = initializer {
            decl = decl.with_initializer(init);
        }
        if let Some(size) = array_size {
            decl = decl.with_array_size(size);
        }

        Ok(decl)
    }

    fn parse_const_decl(&mut self) -> Result<ConstDecl, CompileError> {
        let start_span = self.peek_span().unwrap();

        // Consume 'const' keyword
        self.expect(&Token::Const, "Expected 'const' keyword")?;

        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected constant name")),
        };

        // Type annotation is now required
        if !self.match_token(&Token::Colon) {
            return Err(self.error(
                ErrorCode::MissingTypeAnnotation,
                "Constant declaration requires explicit type annotation",
            ).with_hint("Add a type annotation, e.g.: MAX_VALUE: byte = 255"));
        }

        let const_type = self.parse_type()?;

        self.expect(&Token::Equal, "Expected '=' after constant type")?;
        let value = self.parse_expression()?;

        let end_span = value.span.clone();
        let span = start_span.merge(&end_span);

        Ok(ConstDecl::new_typed(name, const_type, value, span))
    }
}
