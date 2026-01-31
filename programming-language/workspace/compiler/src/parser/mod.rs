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

//! Parser module for the Cobra64 compiler.
//!
//! This module parses a token stream into an Abstract Syntax Tree (AST).
//! It uses recursive descent parsing with precedence climbing for expressions.

use crate::ast::{
    AssignOp, AssignTarget, Assignment, BinaryOp, Block, ConstDecl, Expr, ExprKind, ForStatement,
    FunctionDef, IfStatement, Parameter, Program, Statement, StatementKind, TopLevelItem, Type,
    UnaryOp, VarDecl, WhileStatement,
};
use crate::error::{CompileError, ErrorCode, Span};
use crate::lexer::Token;

/// The parser state.
pub struct Parser<'a> {
    /// The token stream to parse.
    tokens: &'a [(Token, Span)],
    /// Current position in the token stream.
    position: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given token stream.
    pub fn new(tokens: &'a [(Token, Span)]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    // ========================================
    // Helper Methods
    // ========================================

    /// Check if we've reached the end of the token stream.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Peek at the current token without advancing.
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|(t, _)| t)
    }

    /// Peek at the current token's span.
    pub fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.position).map(|(_, s)| s.clone())
    }

    /// Peek at a token ahead by n positions.
    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n).map(|(t, _)| t)
    }

    /// Get the previous token's span (for error reporting).
    fn previous_span(&self) -> Span {
        if self.position > 0 {
            self.tokens[self.position - 1].1.clone()
        } else if !self.tokens.is_empty() {
            self.tokens[0].1.clone()
        } else {
            Span::new(0, 0)
        }
    }

    /// Advance to the next token and return the current one.
    pub fn advance(&mut self) -> Option<(Token, Span)> {
        if self.is_at_end() {
            None
        } else {
            let result = self.tokens[self.position].clone();
            self.position += 1;
            Some(result)
        }
    }

    /// Check if the current token matches the expected type.
    pub fn check(&self, expected: &Token) -> bool {
        self.peek()
            .is_some_and(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
    }

    /// Check if the current token matches any of the expected types.
    #[allow(dead_code)]
    fn check_any(&self, expected: &[Token]) -> bool {
        expected.iter().any(|e| self.check(e))
    }

    /// Consume the current token if it matches the expected type.
    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect the current token to match, or return an error.
    fn expect(&mut self, expected: &Token, message: &str) -> Result<(Token, Span), CompileError> {
        if self.check(expected) {
            Ok(self.advance().unwrap())
        } else {
            let span = self.peek_span().unwrap_or_else(|| self.previous_span());
            let found = self
                .peek()
                .map_or("end of file".to_string(), |t| t.to_string());
            Err(CompileError::new(
                ErrorCode::UnexpectedToken,
                format!("{}, found {}", message, found),
                span,
            ))
        }
    }

    /// Skip newlines (but not INDENT/DEDENT).
    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    /// Create an error at the current position.
    fn error(&self, code: ErrorCode, message: impl Into<String>) -> CompileError {
        let span = self.peek_span().unwrap_or_else(|| self.previous_span());
        CompileError::new(code, message, span)
    }

    // ========================================
    // Program Parsing
    // ========================================

    /// Parse the complete program.
    pub fn parse(&mut self) -> Result<Program, CompileError> {
        let mut program = Program::new();

        self.skip_newlines();

        while !self.is_at_end() {
            let item = self.parse_top_level_item()?;
            program.add_item(item);
            self.skip_newlines();
        }

        Ok(program)
    }

    /// Parse a top-level item (function, constant, or variable).
    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, CompileError> {
        match self.peek() {
            Some(Token::Def) => {
                let func = self.parse_function_def()?;
                Ok(TopLevelItem::Function(func))
            }
            Some(Token::Const) => {
                let decl = self.parse_const_decl()?;
                Ok(TopLevelItem::Constant(decl))
            }
            Some(Token::Identifier(_)) => {
                // Could be a variable declaration (name: type) or a statement
                // Look ahead to check for colon
                if matches!(self.peek_ahead(1), Some(Token::Colon)) {
                    let decl = self.parse_var_decl()?;
                    Ok(TopLevelItem::Variable(decl))
                } else {
                    Err(self.error(
                        ErrorCode::UnexpectedToken,
                        "Expected function definition, constant, or variable declaration at top level",
                    ))
                }
            }
            Some(t) if t.is_type() => Err(self.error(
                ErrorCode::UnexpectedToken,
                "Variable declarations should use 'name: type' syntax",
            )),
            _ => Err(self.error(
                ErrorCode::UnexpectedToken,
                "Expected function definition, constant, or variable declaration",
            )),
        }
    }

    // ========================================
    // Function Parsing
    // ========================================

    /// Parse a function definition.
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

    /// Parse a parameter list.
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

    /// Parse a single parameter.
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

    /// Parse a type.
    fn parse_type(&mut self) -> Result<Type, CompileError> {
        let base_type = match self.peek() {
            Some(Token::Byte) => {
                self.advance();
                Type::Byte
            }
            Some(Token::Word) => {
                self.advance();
                Type::Word
            }
            Some(Token::Sbyte) => {
                self.advance();
                Type::Sbyte
            }
            Some(Token::Sword) => {
                self.advance();
                Type::Sword
            }
            Some(Token::Bool) => {
                self.advance();
                Type::Bool
            }
            Some(Token::StringType) => {
                self.advance();
                Type::String
            }
            Some(Token::Fixed) => {
                self.advance();
                Type::Fixed
            }
            Some(Token::Float) => {
                self.advance();
                Type::Float
            }
            _ => return Err(self.error(ErrorCode::ExpectedType, "Expected type")),
        };

        // Check for array type
        if self.match_token(&Token::LeftBracket) {
            let size = if self.check(&Token::Integer(0)) {
                if let Some((Token::Integer(n), _)) = self.advance() {
                    Some(n)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&Token::RightBracket, "Expected ']' after array size")?;

            match base_type {
                Type::Byte => Ok(Type::ByteArray(size)),
                Type::Word => Ok(Type::WordArray(size)),
                _ => Err(self.error(
                    ErrorCode::InvalidType,
                    "Only byte and word arrays are supported",
                )),
            }
        } else {
            Ok(base_type)
        }
    }

    // ========================================
    // Block Parsing
    // ========================================

    /// Parse a block of statements.
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

    // ========================================
    // Statement Parsing
    // ========================================

    /// Parse a statement.
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

    /// Parse a statement starting with an identifier.
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

    /// Convert an expression to an assignment target.
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

    /// Try to parse an assignment operator.
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

    /// Parse a variable declaration.
    fn parse_var_decl(&mut self) -> Result<VarDecl, CompileError> {
        let start_span = self.peek_span().unwrap();

        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected variable name")),
        };

        self.expect(&Token::Colon, "Expected ':' after variable name")?;
        let var_type = self.parse_type()?;

        // Check for array size in declaration
        let array_size = if matches!(
            var_type,
            Type::ByteArray(Some(_)) | Type::WordArray(Some(_))
        ) {
            match &var_type {
                Type::ByteArray(Some(n)) | Type::WordArray(Some(n)) => Some(*n),
                _ => None,
            }
        } else {
            None
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

    /// Parse a constant declaration.
    fn parse_const_decl(&mut self) -> Result<ConstDecl, CompileError> {
        let start_span = self.peek_span().unwrap();
        self.expect(&Token::Const, "Expected 'const'")?;

        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected constant name")),
        };

        self.expect(&Token::Equal, "Expected '=' after constant name")?;
        let value = self.parse_expression()?;

        let end_span = value.span.clone();
        let span = start_span.merge(&end_span);

        Ok(ConstDecl::new(name, value, span))
    }

    /// Parse an if statement.
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

    /// Parse a while statement.
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

    /// Parse a for statement.
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

    /// Parse a return statement.
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

    // ========================================
    // Expression Parsing (Precedence Climbing)
    // ========================================

    /// Parse an expression.
    fn parse_expression(&mut self) -> Result<Expr, CompileError> {
        self.parse_or_expression()
    }

    /// Parse an 'or' expression.
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

    /// Parse an 'and' expression.
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

    /// Parse a comparison expression.
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

    /// Try to parse a comparison operator.
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

    /// Parse a bitwise OR expression.
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

    /// Parse a bitwise XOR expression.
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

    /// Parse a bitwise AND expression.
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

    /// Parse a shift expression.
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

    /// Try to parse a shift operator.
    fn try_parse_shift_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::ShiftLeft => BinaryOp::ShiftLeft,
            Token::ShiftRight => BinaryOp::ShiftRight,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Parse an additive expression.
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

    /// Try to parse an additive operator.
    fn try_parse_additive_op(&mut self) -> Option<BinaryOp> {
        let op = match self.peek()? {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Sub,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    /// Parse a multiplicative expression.
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

    /// Try to parse a multiplicative operator.
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

    /// Parse a unary expression.
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

    /// Parse a postfix expression (function calls, array indexing).
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

    /// Parse a function call.
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

    /// Parse array indexing.
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

    /// Parse a primary expression.
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
            | Token::Float => {
                let target_type = match token {
                    Token::Byte => Type::Byte,
                    Token::Word => Type::Word,
                    Token::Sbyte => Type::Sbyte,
                    Token::Sword => Type::Sword,
                    Token::Fixed => Type::Fixed,
                    Token::Float => Type::Float,
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

            _ => Err(CompileError::new(
                ErrorCode::UnexpectedToken,
                format!("Unexpected token in expression: {}", token),
                span,
            )),
        }
    }
}

/// Parse a token stream into a program AST.
pub fn parse(tokens: &[(Token, Span)]) -> Result<Program, CompileError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    /// Helper to parse source code directly.
    fn parse_source(source: &str) -> Result<Program, CompileError> {
        let tokens = tokenize(source)?;
        parse(&tokens)
    }

    // ========================================
    // Parser Creation Tests
    // ========================================

    #[test]
    fn test_parser_creation() {
        let tokens = vec![];
        let parser = Parser::new(&tokens);
        assert!(parser.is_at_end());
    }

    #[test]
    fn test_parser_peek() {
        let tokens = vec![
            (Token::Integer(42), Span::new(0, 2)),
            (Token::Plus, Span::new(3, 4)),
        ];
        let parser = Parser::new(&tokens);
        assert_eq!(parser.peek(), Some(&Token::Integer(42)));
    }

    #[test]
    fn test_parser_advance() {
        let tokens = vec![
            (Token::Integer(42), Span::new(0, 2)),
            (Token::Plus, Span::new(3, 4)),
        ];
        let mut parser = Parser::new(&tokens);
        let first = parser.advance();
        assert!(matches!(first, Some((Token::Integer(42), _))));
        assert_eq!(parser.peek(), Some(&Token::Plus));
    }

    // ========================================
    // Expression Parsing Tests
    // ========================================

    #[test]
    fn test_parse_integer_literal() {
        let program = parse_source("def main():\n    42").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.body.statements.len(), 1);
    }

    #[test]
    fn test_parse_binary_expression() {
        let program = parse_source("def main():\n    1 + 2").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(expr.kind, ExprKind::BinaryOp { .. }));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let program = parse_source("def main():\n    1 + 2 * 3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            // Should parse as 1 + (2 * 3) due to precedence
            if let ExprKind::BinaryOp { op, right, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(
                    right.kind,
                    ExprKind::BinaryOp {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_unary_expression() {
        let program = parse_source("def main():\n    -42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_logical_expression() {
        let program = parse_source("def main():\n    true and false").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::And,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_comparison_expression() {
        let program = parse_source("def main():\n    x == 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::Equal,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let program = parse_source("def main():\n    (1 + 2) * 3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, left, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Mul);
                assert!(matches!(left.kind, ExprKind::Grouped(_)));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_function_call() {
        let program = parse_source("def main():\n    print(42)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::FunctionCall { name, args } = &expr.kind {
                assert_eq!(name, "print");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let program = parse_source("def main():\n    foo(1, 2, 3)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::FunctionCall { name, args } = &expr.kind {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 3);
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_array_index() {
        let program = parse_source("def main():\n    arr[0]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(expr.kind, ExprKind::ArrayIndex { .. }));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_type_cast() {
        let program = parse_source("def main():\n    byte(256)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Byte);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    // ========================================
    // Statement Parsing Tests
    // ========================================

    #[test]
    fn test_parse_assignment() {
        let program = parse_source("def main():\n    x = 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            assert!(matches!(assign.target, AssignTarget::Variable(ref n) if n == "x"));
            assert_eq!(assign.op, AssignOp::Assign);
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_compound_assignment() {
        let program = parse_source("def main():\n    x += 1").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            assert_eq!(assign.op, AssignOp::AddAssign);
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_var_decl() {
        let program = parse_source("def main():\n    x: byte = 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Type::Byte);
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_var_decl_no_init() {
        let program = parse_source("def main():\n    x: word").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Type::Word);
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_const_decl() {
        let program = parse_source("def main():\n    const MAX = 100").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::ConstDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "MAX");
        } else {
            panic!("Expected const decl");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let program = parse_source("def main():\n    if true:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert!(if_stmt.else_block.is_none());
            assert!(if_stmt.elif_branches.is_empty());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let source = "def main():\n    if true:\n        pass\n    else:\n        pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert!(if_stmt.else_block.is_some());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_if_elif_else() {
        let source = "def main():\n    if x == 1:\n        pass\n    elif x == 2:\n        pass\n    else:\n        pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert_eq!(if_stmt.elif_branches.len(), 1);
            assert!(if_stmt.else_block.is_some());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let program = parse_source("def main():\n    while true:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        assert!(matches!(
            main.body.statements[0].kind,
            StatementKind::While(_)
        ));
    }

    #[test]
    fn test_parse_for_statement() {
        let program = parse_source("def main():\n    for i in 0 to 10:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::For(for_stmt) = &main.body.statements[0].kind {
            assert_eq!(for_stmt.variable, "i");
            assert!(!for_stmt.descending);
        } else {
            panic!("Expected for statement");
        }
    }

    #[test]
    fn test_parse_for_downto() {
        let program = parse_source("def main():\n    for i in 10 downto 0:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::For(for_stmt) = &main.body.statements[0].kind {
            assert!(for_stmt.descending);
        } else {
            panic!("Expected for statement");
        }
    }

    #[test]
    fn test_parse_break() {
        let program = parse_source("def main():\n    while true:\n        break").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(while_stmt) = &main.body.statements[0].kind {
            assert!(matches!(
                while_stmt.body.statements[0].kind,
                StatementKind::Break
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    #[test]
    fn test_parse_continue() {
        let program = parse_source("def main():\n    while true:\n        continue").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(while_stmt) = &main.body.statements[0].kind {
            assert!(matches!(
                while_stmt.body.statements[0].kind,
                StatementKind::Continue
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    #[test]
    fn test_parse_return() {
        let program = parse_source("def main():\n    return").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Return(val) = &main.body.statements[0].kind {
            assert!(val.is_none());
        } else {
            panic!("Expected return statement");
        }
    }

    #[test]
    fn test_parse_return_value() {
        let program = parse_source("def main():\n    return 42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Return(val) = &main.body.statements[0].kind {
            assert!(val.is_some());
        } else {
            panic!("Expected return statement");
        }
    }

    #[test]
    fn test_parse_pass() {
        let program = parse_source("def main():\n    pass").unwrap();
        let main = program.main_function().unwrap();
        assert!(matches!(main.body.statements[0].kind, StatementKind::Pass));
    }

    // ========================================
    // Function Definition Tests
    // ========================================

    #[test]
    fn test_parse_function_no_params() {
        let program = parse_source("def main():\n    pass").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.name, "main");
        assert!(main.params.is_empty());
        assert!(main.return_type.is_none());
    }

    #[test]
    fn test_parse_function_with_params() {
        let program = parse_source("def add(a: byte, b: byte):\n    pass").unwrap();
        assert_eq!(program.items.len(), 1);
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.name, "add");
            assert_eq!(func.params.len(), 2);
            assert_eq!(func.params[0].name, "a");
            assert_eq!(func.params[0].param_type, Type::Byte);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let program = parse_source("def get_value() -> byte:\n    return 42").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Byte));
        } else {
            panic!("Expected function");
        }
    }

    // ========================================
    // Top-Level Tests
    // ========================================

    #[test]
    fn test_parse_top_level_const() {
        let program = parse_source("const MAX = 255\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        assert!(matches!(program.items[0], TopLevelItem::Constant(_)));
    }

    #[test]
    fn test_parse_top_level_var() {
        let program = parse_source("counter: word\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        assert!(matches!(program.items[0], TopLevelItem::Variable(_)));
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = "def helper():\n    pass\n\ndef main():\n    helper()";
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    // ========================================
    // Nested Block Tests
    // ========================================

    #[test]
    fn test_parse_nested_if() {
        let source = "def main():\n    if true:\n        if false:\n            pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(outer) = &main.body.statements[0].kind {
            assert!(matches!(
                outer.then_block.statements[0].kind,
                StatementKind::If(_)
            ));
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_nested_while() {
        let source = "def main():\n    while true:\n        while false:\n            pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(outer) = &main.body.statements[0].kind {
            assert!(matches!(
                outer.body.statements[0].kind,
                StatementKind::While(_)
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    // ========================================
    // Error Cases
    // ========================================

    #[test]
    fn test_parse_error_missing_colon() {
        let result = parse_source("def main()\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_paren() {
        let result = parse_source("def main:\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = parse_source("def main():\n    +++");
        assert!(result.is_err());
    }

    // ========================================
    // Bitwise and Shift Tests
    // ========================================

    #[test]
    fn test_parse_bitwise_and() {
        let program = parse_source("def main():\n    x & y").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::BitAnd,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_bitwise_or() {
        let program = parse_source("def main():\n    x | y").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::BitOr,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_shift_left() {
        let program = parse_source("def main():\n    x << 2").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::ShiftLeft,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_bitwise_not() {
        let program = parse_source("def main():\n    ~x").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::BitNot,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    // ========================================
    // Signed Type and Negative Literal Tests
    // ========================================

    #[test]
    fn test_parse_sbyte_var_decl() {
        let program = parse_source("def main():\n    x: sbyte = -100").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Type::Sbyte);
            assert!(decl.initializer.is_some());
            // The initializer should be a unary negation of 100
            if let Some(init) = &decl.initializer {
                assert!(matches!(
                    init.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_var_decl() {
        let program = parse_source("def main():\n    y: sword = -30000").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Type::Sword);
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sbyte_min_value() {
        // -128 is the minimum value for sbyte
        let program = parse_source("def main():\n    x: sbyte = -128").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Sbyte);
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(128)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_min_value() {
        // -32768 is the minimum value for sword
        let program = parse_source("def main():\n    y: sword = -32768").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Sword);
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(32768)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_negative_hex_literal() {
        // -$7F = -127
        let program = parse_source("def main():\n    x: sbyte = -$7F").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(127)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_negative_binary_literal() {
        // -%01111111 = -127
        let program = parse_source("def main():\n    x: sbyte = -%01111111").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(127)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_const_negative_value() {
        let program = parse_source("const MIN_SBYTE = -128\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "MIN_SBYTE");
            assert!(matches!(
                decl.value.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    ..
                }
            ));
        } else {
            panic!("Expected constant");
        }
    }

    #[test]
    fn test_parse_sbyte_function_param() {
        let program = parse_source("def process(val: sbyte):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Sbyte);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_sword_function_return() {
        let program = parse_source("def get_value() -> sword:\n    return -1").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Sword));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_sbyte_type_cast() {
        let program = parse_source("def main():\n    sbyte(-100)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Sbyte);
                // Inner expression is the negated literal
                assert!(matches!(
                    inner.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_in_expression() {
        let program = parse_source("def main():\n    x = 10 + -5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, right, .. } = &assign.value.kind {
                assert_eq!(*op, BinaryOp::Add);
                // Right side should be unary negate
                assert!(matches!(
                    right.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_double_negative() {
        // --42 should parse as negation of negation of 42
        let program = parse_source("def main():\n    --42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::UnaryOp { op, operand } = &expr.kind {
                assert_eq!(*op, UnaryOp::Negate);
                // Inner should also be a negation
                assert!(matches!(
                    operand.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected unary negate");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_zero() {
        let program = parse_source("def main():\n    x: sbyte = -0").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { operand, .. } = &init.kind {
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(0)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_subtraction_vs_negative() {
        // Ensure a - 1 is subtraction, not a followed by -1
        let program = parse_source("def main():\n    x - 1").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Sub);
            } else {
                panic!("Expected binary subtraction");
            }
        } else {
            panic!("Expected expression");
        }
    }

    // ========================================
    // Fixed-Point and Float Type Tests
    // ========================================

    #[test]
    fn test_parse_fixed_var_decl() {
        let program = parse_source("def main():\n    x: fixed = 3.75").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Type::Fixed);
            assert!(decl.initializer.is_some());
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "3.75"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_float_var_decl() {
        let program = parse_source("def main():\n    y: float = 3.14").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Type::Float);
            assert!(decl.initializer.is_some());
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_var_decl_no_init() {
        let program = parse_source("def main():\n    x: fixed").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Type::Fixed);
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_float_var_decl_no_init() {
        let program = parse_source("def main():\n    y: float").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Type::Float);
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_function_param() {
        let program = parse_source("def scale(val: fixed):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Fixed);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_float_function_param() {
        let program = parse_source("def process(val: float):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Float);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_fixed_function_return() {
        let program = parse_source("def get_pos() -> fixed:\n    return 0.0").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Fixed));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_float_function_return() {
        let program = parse_source("def compute() -> float:\n    return 3.14").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Float));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_decimal_literal_in_expression() {
        let program = parse_source("def main():\n    3.14 + 2.5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { left, op, right } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(left.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
                assert!(matches!(right.kind, ExprKind::DecimalLiteral(ref s) if s == "2.5"));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_scientific_notation() {
        let program = parse_source("def main():\n    x: float = 1.5e3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Float);
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "1.5e3"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_scientific_notation_negative_exponent() {
        let program = parse_source("def main():\n    x: float = 2.0e-5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Float);
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "2.0e-5"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_type_cast() {
        let program = parse_source("def main():\n    fixed(100)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Fixed);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_float_type_cast() {
        let program = parse_source("def main():\n    float(42)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Float);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_fixed_cast_from_decimal() {
        let program = parse_source("def main():\n    fixed(3.14)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Fixed);
                assert!(matches!(inner.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_byte_cast_from_fixed() {
        let program = parse_source("def main():\n    x: fixed = 3.5\n    byte(x)").unwrap();
        let main = program.main_function().unwrap();
        // Second statement should be the type cast
        if let StatementKind::Expression(expr) = &main.body.statements[1].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Byte);
                assert!(matches!(inner.kind, ExprKind::Identifier(ref name) if name == "x"));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_decimal_literal() {
        let program = parse_source("def main():\n    x: fixed = -3.5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Fixed);
            if let Some(init) = &decl.initializer {
                // Should be UnaryOp(Negate, DecimalLiteral("3.5"))
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::DecimalLiteral(ref s) if s == "3.5"));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_decimal_zero() {
        let program = parse_source("def main():\n    x: float = 0.0").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Type::Float);
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "0.0"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_top_level_fixed_var() {
        let program = parse_source("position: fixed\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "position");
            assert_eq!(decl.var_type, Type::Fixed);
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_top_level_float_var() {
        let program = parse_source("temperature: float\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "temperature");
            assert_eq!(decl.var_type, Type::Float);
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_mixed_fixed_float_expression() {
        // This tests that the parser can handle mixed types in expressions
        // Actual type checking will happen in the analyzer
        let program =
            parse_source("def main():\n    x: fixed = 1.0\n    y: float = 2.0\n    x + y").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.body.statements.len(), 3);
        if let StatementKind::Expression(expr) = &main.body.statements[2].kind {
            if let ExprKind::BinaryOp { op, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression");
        }
    }
}
