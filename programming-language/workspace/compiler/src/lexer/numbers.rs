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

//! Number scanning for the lexer.
//!
//! This module handles scanning of number literals including:
//! - Decimal numbers (integers and floating-point)
//! - Hexadecimal numbers ($ prefix)
//! - Binary numbers (% prefix)

use super::helpers::LexerHelpers;
use super::Lexer;
use super::Token;
use crate::error::{CompileError, ErrorCode, Span};

/// Trait for number scanning operations.
pub trait NumberScanner<'source> {
    /// Scan a decimal number literal (integer or decimal with fractional part).
    fn scan_decimal_number(&mut self) -> Result<(Token, Span), CompileError>;

    /// Scan a decimal literal (number with decimal point or scientific notation).
    /// Called when we've detected a '.' or 'e'/'E' in a number.
    fn scan_decimal_literal(&mut self, start: usize) -> Result<(Token, Span), CompileError>;

    /// Scan a hexadecimal number literal (starting with $).
    fn scan_hex_number(&mut self) -> Result<(Token, Span), CompileError>;

    /// Scan a binary number literal (starting with %).
    fn scan_binary_number(&mut self) -> Result<(Token, Span), CompileError>;
}

impl<'source> NumberScanner<'source> for Lexer<'source> {
    fn scan_decimal_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        let mut value: u64 = 0;
        let mut has_decimal_point = false;
        let mut has_exponent = false;

        // Scan integer part
        while let Some(c) = self.peek() {
            if let Some(digit) = c.to_digit(10) {
                self.advance();
                value = value.saturating_mul(10).saturating_add(digit as u64);
            } else if c == '.' {
                // Check if this is a decimal point (not a method call like "123.to_string")
                if let Some(next) = self.peek_next() {
                    if next.is_ascii_digit() {
                        has_decimal_point = true;
                        break;
                    }
                }
                // Not a decimal number, just an integer followed by a dot
                break;
            } else if c == 'e' || c == 'E' {
                // Scientific notation without decimal point (e.g., "1e5")
                has_exponent = true;
                break;
            } else if c.is_ascii_alphabetic() || c == '_' {
                return Err(CompileError::new(
                    ErrorCode::InvalidDigitInNumber,
                    "Invalid digit in number literal",
                    self.span_from(start),
                ));
            } else {
                break;
            }
        }

        // If we have a decimal point or exponent, scan as decimal literal
        if has_decimal_point || has_exponent {
            return self.scan_decimal_literal(start);
        }

        // Otherwise, it's an integer
        if value > 65535 {
            return Err(CompileError::new(
                ErrorCode::IntegerTooLargeForWord,
                "Integer literal too large for word (max 65535)",
                self.span_from(start),
            ));
        }

        Ok((Token::Integer(value as u16), self.span_from(start)))
    }

    fn scan_decimal_literal(&mut self, start: usize) -> Result<(Token, Span), CompileError> {
        // Reset position to start and re-scan the entire number as a string
        self.position = start;
        self.column = self.column.saturating_sub(self.position - start);

        let mut literal = String::new();
        let mut has_decimal_point = false;
        let mut has_exponent = false;
        let mut has_digit = false;

        // Scan integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                has_digit = true;
                literal.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Scan decimal point and fractional part
        if self.peek() == Some('.') {
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    has_decimal_point = true;
                    literal.push('.');
                    self.advance(); // consume '.'

                    // Scan fractional digits
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            has_digit = true;
                            literal.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Scan exponent part (e.g., e+10, E-5, e3)
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' {
                has_exponent = true;
                literal.push(c);
                self.advance();

                // Optional sign
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        literal.push(sign);
                        self.advance();
                    }
                }

                // Exponent digits (required)
                let mut has_exp_digit = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        has_exp_digit = true;
                        literal.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if !has_exp_digit {
                    return Err(CompileError::new(
                        ErrorCode::InvalidDecimalLiteral,
                        "Exponent requires at least one digit",
                        self.span_from(start),
                    ));
                }
            }
        }

        // Validate: must have at least one digit and either decimal point or exponent
        if !has_digit {
            return Err(CompileError::new(
                ErrorCode::InvalidDecimalLiteral,
                "Decimal literal requires at least one digit",
                self.span_from(start),
            ));
        }

        if !has_decimal_point && !has_exponent {
            return Err(CompileError::new(
                ErrorCode::InvalidDecimalLiteral,
                "Expected decimal point or exponent",
                self.span_from(start),
            ));
        }

        // Check for invalid trailing characters
        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() || c == '_' {
                return Err(CompileError::new(
                    ErrorCode::InvalidDecimalLiteral,
                    format!("Invalid character '{}' in decimal literal", c),
                    self.span_from(start),
                ));
            }
        }

        Ok((Token::Decimal(literal), self.span_from(start)))
    }

    fn scan_hex_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        self.advance(); // consume $

        let mut value: u64 = 0;
        let mut has_digits = false;

        while let Some(c) = self.peek() {
            if let Some(digit) = c.to_digit(16) {
                self.advance();
                has_digits = true;
                value = value.saturating_mul(16).saturating_add(digit as u64);
            } else if c.is_ascii_alphanumeric() {
                return Err(CompileError::new(
                    ErrorCode::InvalidHexDigit,
                    "Invalid hexadecimal digit",
                    self.span_from(start),
                ));
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(CompileError::new(
                ErrorCode::EmptyNumberLiteral,
                "Number literal cannot be empty",
                self.span_from(start),
            ));
        }

        if value > 65535 {
            return Err(CompileError::new(
                ErrorCode::IntegerTooLargeForWord,
                "Integer literal too large for word (max $FFFF)",
                self.span_from(start),
            ));
        }

        Ok((Token::Integer(value as u16), self.span_from(start)))
    }

    fn scan_binary_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        self.advance(); // consume %

        let mut value: u64 = 0;
        let mut has_digits = false;

        while let Some(c) = self.peek() {
            match c {
                '0' => {
                    self.advance();
                    has_digits = true;
                    value = value.saturating_mul(2);
                }
                '1' => {
                    self.advance();
                    has_digits = true;
                    value = value.saturating_mul(2).saturating_add(1);
                }
                c if c.is_ascii_alphanumeric() => {
                    return Err(CompileError::new(
                        ErrorCode::InvalidBinaryDigit,
                        "Invalid binary digit (expected 0 or 1)",
                        self.span_from(start),
                    ));
                }
                _ => break,
            }
        }

        if !has_digits {
            return Err(CompileError::new(
                ErrorCode::EmptyNumberLiteral,
                "Number literal cannot be empty",
                self.span_from(start),
            ));
        }

        if value > 65535 {
            return Err(CompileError::new(
                ErrorCode::IntegerTooLargeForWord,
                "Integer literal too large for word (max %1111111111111111)",
                self.span_from(start),
            ));
        }

        Ok((Token::Integer(value as u16), self.span_from(start)))
    }
}
