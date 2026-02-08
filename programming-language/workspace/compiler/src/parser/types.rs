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

//! Type parsing for the parser.
//!
//! This module handles parsing of type annotations including:
//! - Primitive types (byte, word, sbyte, sword, bool, string, fixed, float)
//! - Array types with optional size specifiers

use super::helpers::ParserHelpers;
use super::Parser;
use crate::ast::Type;
use crate::error::{CompileError, ErrorCode};
use crate::lexer::Token;

/// Trait for type parsing operations.
pub trait TypeParser<'a> {
    /// Parse a type annotation.
    fn parse_type(&mut self) -> Result<Type, CompileError>;
}

impl<'a> TypeParser<'a> for Parser<'a> {
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
                Type::Bool => Ok(Type::BoolArray(size)),
                Type::Sbyte => Ok(Type::SbyteArray(size)),
                Type::Sword => Ok(Type::SwordArray(size)),
                Type::Fixed => Ok(Type::FixedArray(size)),
                Type::Float => Ok(Type::FloatArray(size)),
                _ => Err(self.error(
                    ErrorCode::InvalidType,
                    "Only byte, word, bool, sbyte, sword, fixed, and float arrays are supported",
                )),
            }
        } else {
            Ok(base_type)
        }
    }
}
