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

//! Operator and punctuation scanning for the lexer.
//!
//! This module handles scanning of:
//! - Arithmetic operators (+, -, *, /, %)
//! - Comparison operators (==, !=, <, >, <=, >=)
//! - Assignment operators (=, +=, -=, etc.)
//! - Bitwise operators (&, |, ^, ~, <<, >>)
//! - Punctuation (parentheses, brackets, colon, comma)

use super::helpers::LexerHelpers;
use super::Lexer;
use super::Token;
use crate::error::{CompileError, ErrorCode, Span};

/// Trait for operator scanning operations.
pub trait OperatorScanner<'source> {
    /// Scan an operator or punctuation.
    fn scan_operator_or_punctuation(&mut self) -> Result<Option<(Token, Span)>, CompileError>;
}

impl<'source> OperatorScanner<'source> for Lexer<'source> {
    fn scan_operator_or_punctuation(&mut self) -> Result<Option<(Token, Span)>, CompileError> {
        let start = self.position;
        let c = self.advance().unwrap();

        let token = match c {
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::PlusAssign
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::MinusAssign
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::StarAssign
                } else {
                    Token::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::SlashAssign
                } else {
                    Token::Slash
                }
            }
            '%' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::PercentAssign
                } else {
                    Token::Percent
                }
            }
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqualEqual
                } else {
                    Token::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::BangEqual
                } else {
                    return Err(CompileError::new(
                        ErrorCode::InvalidCharacter,
                        format!("Invalid character '{}' (use 'not' for logical NOT)", c),
                        self.span_from(start),
                    ));
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else if self.peek() == Some('<') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::ShiftLeftAssign
                    } else {
                        Token::ShiftLeft
                    }
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::ShiftRightAssign
                    } else {
                        Token::ShiftRight
                    }
                } else {
                    Token::Greater
                }
            }
            '&' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::AmpersandAssign
                } else {
                    Token::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::PipeAssign
                } else {
                    Token::Pipe
                }
            }
            '^' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::CaretAssign
                } else {
                    Token::Caret
                }
            }
            '~' => Token::Tilde,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ':' => Token::Colon,
            ',' => Token::Comma,
            _ => {
                return Err(CompileError::new(
                    ErrorCode::InvalidCharacter,
                    format!("Invalid character '{}'", c),
                    self.span_from(start),
                ));
            }
        };

        Ok(Some((token, self.span_from(start))))
    }
}
