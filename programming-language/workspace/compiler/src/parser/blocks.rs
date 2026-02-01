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

//! Block and function parsing for the parser.
//!
//! This module provides block and function parsing functionality:
//! - Function definitions
//! - Parameter lists
//! - Statement blocks
//! - Top-level items

use super::helpers::ParserHelpers;
use super::statements::StatementParser;
use super::types::TypeParser;
use super::Parser;
use crate::ast::{Block, FunctionDef, Parameter, StatementKind, TopLevelItem};
use crate::error::{CompileError, ErrorCode};
use crate::lexer::{is_constant_name, Token};

/// Extension trait for block and function parsing.
pub trait BlockParser {
    /// Parse a top-level item (function, constant, or variable).
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, CompileError>;

    /// Parse a function definition.
    fn parse_function_def(&mut self) -> Result<FunctionDef, CompileError>;

    /// Parse a parameter list.
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, CompileError>;

    /// Parse a single parameter.
    fn parse_parameter(&mut self) -> Result<Parameter, CompileError>;

    /// Parse a block of statements.
    fn parse_block(&mut self) -> Result<Block, CompileError>;
}

impl<'a> BlockParser for Parser<'a> {
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, CompileError> {
        match self.peek() {
            Some(Token::Def) => {
                let func = self.parse_function_def()?;
                Ok(TopLevelItem::Function(func))
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                // Check if it's a constant (UPPERCASE name) or variable (lowercase name)
                if is_constant_name(&name) {
                    // Constant: NAME = value OR NAME: type = value
                    if matches!(self.peek_ahead(1), Some(Token::Equal))
                        || matches!(self.peek_ahead(1), Some(Token::Colon))
                    {
                        let decl = self.parse_const_decl()?;
                        Ok(TopLevelItem::Constant(decl))
                    } else {
                        Err(self.error(
                            ErrorCode::UnexpectedToken,
                            "Expected '=' or ':' after constant name",
                        ))
                    }
                } else {
                    // Variable: name: type = value OR name = value
                    if matches!(self.peek_ahead(1), Some(Token::Colon))
                        || matches!(self.peek_ahead(1), Some(Token::Equal))
                    {
                        let decl = self.parse_var_decl()?;
                        Ok(TopLevelItem::Variable(decl))
                    } else {
                        Err(self.error(
                            ErrorCode::UnexpectedToken,
                            "Expected function definition, constant, or variable declaration at top level",
                        ))
                    }
                }
            }
            Some(t) if t.is_type() => Err(self.error(
                ErrorCode::UnexpectedToken,
                "Variable declarations should use 'name: type' or 'name = value' syntax",
            )),
            _ => Err(self.error(
                ErrorCode::UnexpectedToken,
                "Expected function definition, constant, or variable declaration",
            )),
        }
    }

    fn parse_function_def(&mut self) -> Result<FunctionDef, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::Def, "Expected 'def'")?;

        // Function name
        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected function name")),
        };

        // Parameters
        self.expect(&Token::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_parameter_list()?;
        self.expect(&Token::RightParen, "Expected ')' after parameters")?;

        // Optional return type
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Colon before body
        self.expect(&Token::Colon, "Expected ':' before function body")?;
        self.expect(&Token::Newline, "Expected newline after ':'")?;

        // Function body
        let body = self.parse_block()?;

        let end_span = body.span.clone();
        let span = start_span.merge(&end_span);

        let mut func = FunctionDef::new(name, params, body, span);
        if let Some(ret_type) = return_type {
            func = func.with_return_type(ret_type);
        }

        Ok(func)
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, CompileError> {
        let mut params = Vec::new();

        if !self.check(&Token::RightParen) {
            loop {
                let param = self.parse_parameter()?;
                params.push(param);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_parameter(&mut self) -> Result<Parameter, CompileError> {
        let start_span = self.peek_span().unwrap();

        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected parameter name")),
        };

        self.expect(&Token::Colon, "Expected ':' after parameter name")?;
        let param_type = self.parse_type()?;

        let end_span = self.previous_span();
        let span = start_span.merge(&end_span);

        Ok(Parameter::new(name, param_type, span))
    }

    fn parse_block(&mut self) -> Result<Block, CompileError> {
        let start_span = self.peek_span().unwrap_or_else(|| self.previous_span());

        self.expect(&Token::Indent, "Expected indented block")?;

        let mut statements = Vec::new();

        while !self.check(&Token::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&Token::Dedent) || self.is_at_end() {
                break;
            }

            let stmt = self.parse_statement()?;

            // Block statements (if/while/for) don't require a newline after them
            // because their contained blocks already handle the newline/dedent
            let is_block_statement = matches!(
                &stmt.kind,
                StatementKind::If(_) | StatementKind::While(_) | StatementKind::For(_)
            );

            statements.push(stmt);

            // Consume newline after statement (unless it's a block statement)
            if !self.check(&Token::Dedent)
                && !self.is_at_end()
                && !is_block_statement
                && !self.match_token(&Token::Newline)
            {
                // Allow missing newline before dedent
                if !self.check(&Token::Dedent) {
                    return Err(self.error(
                        ErrorCode::ExpectedNewline,
                        "Expected newline after statement",
                    ));
                }
            }
            self.skip_newlines();
        }

        if !self.is_at_end() {
            self.expect(&Token::Dedent, "Expected dedent")?;
        }

        let end_span = self.previous_span();
        let span = start_span.merge(&end_span);

        Ok(Block::new(statements, span))
    }
}
