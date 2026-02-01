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

//! Lexer helper methods for character navigation and span creation.
//!
//! This module provides utility methods for the lexer including:
//! - Character stream navigation (peek, peek_next, advance)
//! - Position tracking (position, line, column)
//! - Span creation

use super::Lexer;
use crate::error::Span;

/// Trait for lexer helper operations.
#[allow(dead_code)]
pub trait LexerHelpers<'source> {
    /// Get the current position in the source.
    fn position(&self) -> usize;

    /// Get the current line number.
    fn line(&self) -> usize;

    /// Get the current column number.
    fn column(&self) -> usize;

    /// Check if we've reached the end of the source.
    fn is_at_end(&self) -> bool;

    /// Peek at the current character without advancing.
    fn peek(&self) -> Option<char>;

    /// Peek at the next character without advancing.
    fn peek_next(&self) -> Option<char>;

    /// Advance to the next character and return it.
    fn advance(&mut self) -> Option<char>;

    /// Create a span from start position to current position.
    fn span_from(&self, start: usize) -> Span;
}

impl<'source> LexerHelpers<'source> for Lexer<'source> {
    fn position(&self) -> usize {
        self.position
    }

    fn line(&self) -> usize {
        self.line
    }

    fn column(&self) -> usize {
        self.column
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.position..].chars();
        chars.next();
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.position += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
            self.at_line_start = true;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.position)
    }
}
