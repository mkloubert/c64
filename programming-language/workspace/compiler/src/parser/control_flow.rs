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

//! Control flow statement parsing for the parser.
//!
//! This module provides control flow statement parsing:
//! - If/elif/else statements
//! - While loops
//! - For loops
//! - Return statements

use super::blocks::BlockParser;
use super::expressions::ExpressionParser;
use super::helpers::ParserHelpers;
use super::Parser;
use crate::ast::{ForStatement, IfStatement, Statement, StatementKind, WhileStatement};
use crate::error::{CompileError, ErrorCode};
use crate::lexer::Token;

/// Extension trait for control flow parsing.
pub trait ControlFlowParser {
    /// Parse an if statement.
    fn parse_if_statement(&mut self) -> Result<Statement, CompileError>;

    /// Parse a while statement.
    fn parse_while_statement(&mut self) -> Result<Statement, CompileError>;

    /// Parse a for statement.
    fn parse_for_statement(&mut self) -> Result<Statement, CompileError>;

    /// Parse a return statement.
    fn parse_return_statement(&mut self) -> Result<Statement, CompileError>;
}

impl<'a> ControlFlowParser for Parser<'a> {
    fn parse_if_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::If, "Expected 'if'")?;

        let condition = self.parse_expression()?;
        self.expect(&Token::Colon, "Expected ':' after condition")?;
        self.expect(&Token::Newline, "Expected newline after ':'")?;

        let then_block = self.parse_block()?;

        // Parse elif branches
        let mut elif_branches = Vec::new();
        self.skip_newlines();
        while self.check(&Token::Elif) {
            self.advance();
            let elif_cond = self.parse_expression()?;
            self.expect(&Token::Colon, "Expected ':' after elif condition")?;
            self.expect(&Token::Newline, "Expected newline after ':'")?;
            let elif_block = self.parse_block()?;
            elif_branches.push((elif_cond, elif_block));
            self.skip_newlines();
        }

        // Parse optional else block
        let else_block = if self.check(&Token::Else) {
            self.advance();
            self.expect(&Token::Colon, "Expected ':' after 'else'")?;
            self.expect(&Token::Newline, "Expected newline after ':'")?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let end_span = else_block
            .as_ref()
            .map(|b| b.span.clone())
            .or_else(|| elif_branches.last().map(|(_, b)| b.span.clone()))
            .unwrap_or_else(|| then_block.span.clone());
        let span = start_span.merge(&end_span);

        let if_stmt = IfStatement {
            condition,
            then_block,
            elif_branches,
            else_block,
            span: span.clone(),
        };

        Ok(Statement::new(StatementKind::If(if_stmt), span))
    }

    fn parse_while_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::While, "Expected 'while'")?;

        let condition = self.parse_expression()?;
        self.expect(&Token::Colon, "Expected ':' after condition")?;
        self.expect(&Token::Newline, "Expected newline after ':'")?;

        let body = self.parse_block()?;

        let end_span = body.span.clone();
        let span = start_span.merge(&end_span);

        let while_stmt = WhileStatement {
            condition,
            body,
            span: span.clone(),
        };

        Ok(Statement::new(StatementKind::While(while_stmt), span))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::For, "Expected 'for'")?;

        let variable = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => {
                return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected loop variable name"))
            }
        };

        self.expect(&Token::In, "Expected 'in' after loop variable")?;

        let start = self.parse_expression()?;

        let descending = if self.match_token(&Token::To) {
            false
        } else if self.match_token(&Token::Downto) {
            true
        } else {
            return Err(self.error(ErrorCode::UnexpectedToken, "Expected 'to' or 'downto'"));
        };

        let end = self.parse_expression()?;

        self.expect(&Token::Colon, "Expected ':' after range")?;
        self.expect(&Token::Newline, "Expected newline after ':'")?;

        let body = self.parse_block()?;

        let end_span = body.span.clone();
        let span = start_span.merge(&end_span);

        let for_stmt = ForStatement {
            variable,
            start,
            end,
            descending,
            body,
            span: span.clone(),
        };

        Ok(Statement::new(StatementKind::For(for_stmt), span))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::Return, "Expected 'return'")?;

        // Check for optional return value
        let value =
            if !self.check(&Token::Newline) && !self.check(&Token::Dedent) && !self.is_at_end() {
                Some(self.parse_expression()?)
            } else {
                None
            };

        let end_span = value
            .as_ref()
            .map(|e| e.span.clone())
            .unwrap_or_else(|| start_span.clone());
        let span = start_span.merge(&end_span);

        Ok(Statement::new(StatementKind::Return(value), span))
    }
}
