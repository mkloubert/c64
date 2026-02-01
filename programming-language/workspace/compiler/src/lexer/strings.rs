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

//! String and character scanning for the lexer.
//!
//! This module handles scanning of:
//! - String literals with escape sequences
//! - Character literals with escape sequences

use super::helpers::LexerHelpers;
use super::Lexer;
use super::Token;
use crate::error::{CompileError, ErrorCode, Span};

/// Trait for string and character scanning operations.
pub trait StringScanner<'source> {
    /// Scan a string literal.
    fn scan_string(&mut self) -> Result<(Token, Span), CompileError>;

    /// Scan a character literal.
    fn scan_char(&mut self) -> Result<(Token, Span), CompileError>;
}

impl<'source> StringScanner<'source> for Lexer<'source> {
    fn scan_string(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        self.advance(); // consume opening "

        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(CompileError::new(
                        ErrorCode::UnterminatedString,
                        "Unterminated string literal",
                        self.span_from(start),
                    ));
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    let escaped = match self.peek() {
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some('\'') => '\'',
                        Some('0') => '\0',
                        Some(c) => {
                            return Err(CompileError::new(
                                ErrorCode::InvalidEscapeSequence,
                                format!("Invalid escape sequence '\\{}'", c),
                                self.span_from(start),
                            ));
                        }
                        None => {
                            return Err(CompileError::new(
                                ErrorCode::UnterminatedString,
                                "Unterminated string literal",
                                self.span_from(start),
                            ));
                        }
                    };
                    self.advance();
                    value.push(escaped);
                }
                Some(c) => {
                    self.advance();
                    value.push(c);
                }
            }
        }

        if value.len() > 255 {
            return Err(CompileError::new(
                ErrorCode::StringTooLong,
                "String exceeds maximum length of 255 characters",
                self.span_from(start),
            ));
        }

        Ok((Token::String(value), self.span_from(start)))
    }

    fn scan_char(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        self.advance(); // consume opening '

        let value = match self.peek() {
            None | Some('\n') => {
                return Err(CompileError::new(
                    ErrorCode::UnterminatedCharLiteral,
                    "Unterminated character literal",
                    self.span_from(start),
                ));
            }
            Some('\'') => {
                return Err(CompileError::new(
                    ErrorCode::EmptyCharLiteral,
                    "Empty character literal",
                    self.span_from(start),
                ));
            }
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\\') => '\\',
                    Some('"') => '"',
                    Some('\'') => '\'',
                    Some('0') => '\0',
                    Some(c) => {
                        return Err(CompileError::new(
                            ErrorCode::InvalidEscapeSequence,
                            format!("Invalid escape sequence '\\{}'", c),
                            self.span_from(start),
                        ));
                    }
                    None => {
                        return Err(CompileError::new(
                            ErrorCode::UnterminatedCharLiteral,
                            "Unterminated character literal",
                            self.span_from(start),
                        ));
                    }
                }
            }
            Some(c) => c,
        };

        self.advance();

        match self.peek() {
            Some('\'') => {
                self.advance(); // consume closing '
            }
            None | Some('\n') => {
                return Err(CompileError::new(
                    ErrorCode::UnterminatedCharLiteral,
                    "Unterminated character literal",
                    self.span_from(start),
                ));
            }
            Some(_) => {
                return Err(CompileError::new(
                    ErrorCode::CharLiteralTooLong,
                    "Character literal too long (expected single character)",
                    self.span_from(start),
                ));
            }
        }

        Ok((Token::Char(value), self.span_from(start)))
    }
}
