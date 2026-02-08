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

//! Identifier and keyword scanning for the lexer.
//!
//! This module handles scanning of:
//! - Identifiers (variable and function names)
//! - Keywords (reserved words)

use super::helpers::LexerHelpers;
use super::Lexer;
use super::Token;
use crate::error::{CompileError, ErrorCode, Span};

/// Trait for identifier scanning operations.
pub trait IdentifierScanner<'source> {
    /// Scan an identifier or keyword.
    fn scan_identifier(&mut self) -> Result<(Token, Span), CompileError>;
}

impl<'source> IdentifierScanner<'source> for Lexer<'source> {
    fn scan_identifier(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        let span = self.span_from(start);

        // First check if it's a keyword
        let token = Token::from_keyword_or_identifier(text);

        // If it's an identifier (not a keyword), check for underscore-only names
        if let Token::Identifier(ref name) = token {
            if name.chars().all(|c| c == '_') {
                return Err(CompileError::new(
                    ErrorCode::IdentifierOnlyUnderscore,
                    "Identifier cannot consist only of underscores",
                    span,
                ));
            }
        }

        Ok((token, span))
    }
}
