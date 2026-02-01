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

//! Parser helper methods for token stream navigation and error handling.
//!
//! This module provides utility methods for the parser including:
//! - Token stream navigation (peek, advance, check)
//! - Token matching and expectation
//! - Error creation

use super::Parser;
use crate::error::{CompileError, ErrorCode, Span};
use crate::lexer::Token;

/// Trait for parser helper operations.
pub trait ParserHelpers<'a> {
    /// Check if we've reached the end of the token stream.
    fn is_at_end(&self) -> bool;

    /// Peek at the current token without advancing.
    fn peek(&self) -> Option<&Token>;

    /// Peek at the current token's span.
    fn peek_span(&self) -> Option<Span>;

    /// Peek at a token ahead by n positions.
    fn peek_ahead(&self, n: usize) -> Option<&Token>;

    /// Get the previous token's span (for error reporting).
    fn previous_span(&self) -> Span;

    /// Advance to the next token and return the current one.
    fn advance(&mut self) -> Option<(Token, Span)>;

    /// Check if the current token matches the expected type.
    fn check(&self, expected: &Token) -> bool;

    /// Check if the current token matches any of the expected types.
    fn check_any(&self, expected: &[Token]) -> bool;

    /// Consume the current token if it matches the expected type.
    fn match_token(&mut self, expected: &Token) -> bool;

    /// Expect the current token to match, or return an error.
    fn expect(&mut self, expected: &Token, message: &str) -> Result<(Token, Span), CompileError>;

    /// Skip newlines (but not INDENT/DEDENT).
    fn skip_newlines(&mut self);

    /// Create an error at the current position.
    fn error(&self, code: ErrorCode, message: impl Into<String>) -> CompileError;
}

impl<'a> ParserHelpers<'a> for Parser<'a> {
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|(t, _)| t)
    }

    fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.position).map(|(_, s)| s.clone())
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.position + n).map(|(t, _)| t)
    }

    fn previous_span(&self) -> Span {
        if self.position > 0 {
            self.tokens[self.position - 1].1.clone()
        } else if !self.tokens.is_empty() {
            self.tokens[0].1.clone()
        } else {
            Span::new(0, 0)
        }
    }

    fn advance(&mut self) -> Option<(Token, Span)> {
        if self.is_at_end() {
            None
        } else {
            let result = self.tokens[self.position].clone();
            self.position += 1;
            Some(result)
        }
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek()
            .is_some_and(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
    }

    fn check_any(&self, expected: &[Token]) -> bool {
        expected.iter().any(|e| self.check(e))
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

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

    fn skip_newlines(&mut self) {
        while self.check(&Token::Newline) {
            self.advance();
        }
    }

    fn error(&self, code: ErrorCode, message: impl Into<String>) -> CompileError {
        let span = self.peek_span().unwrap_or_else(|| self.previous_span());
        CompileError::new(code, message, span)
    }
}
