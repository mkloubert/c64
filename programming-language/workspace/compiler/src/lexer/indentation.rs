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

//! Indentation handling for the lexer.
//!
//! This module handles Python-style significant whitespace including:
//! - INDENT tokens when indentation increases
//! - DEDENT tokens when indentation decreases
//! - Tracking the indentation stack
//! - Validating consistent indentation levels

use super::helpers::LexerHelpers;
use super::Lexer;
use super::Token;
use crate::error::{CompileError, ErrorCode, Span};

/// Trait for indentation handling operations.
pub trait IndentationHandler<'source> {
    /// Handle indentation at the start of a line.
    fn handle_line_start(&mut self) -> Result<Option<(Token, Span)>, CompileError>;

    /// Skip whitespace (spaces only, not newlines).
    fn skip_whitespace(&mut self);

    /// Skip a comment (from # to end of line).
    fn skip_comment(&mut self);
}

impl<'source> IndentationHandler<'source> for Lexer<'source> {
    fn handle_line_start(&mut self) -> Result<Option<(Token, Span)>, CompileError> {
        let start = self.position;

        // Count leading spaces
        let mut indent = 0;
        while let Some(c) = self.peek() {
            match c {
                ' ' => {
                    indent += 1;
                    self.advance();
                }
                '\t' => {
                    let span = self.span_from(start);
                    return Err(CompileError::new(
                        ErrorCode::TabNotAllowed,
                        "Tab character not allowed (use 4 spaces)",
                        span,
                    ));
                }
                '\n' => {
                    // Empty line, skip it
                    self.advance();
                    return self.next_token();
                }
                '#' => {
                    // Comment-only line - skip entire line (comment + trailing newline)
                    // This makes comment lines "invisible" to the token stream
                    self.skip_comment();
                    // Consume the newline at end of line (if present)
                    // Note: advance() on '\n' sets at_line_start = true automatically
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                    // Continue to next line
                    return self.next_token();
                }
                _ => break,
            }
        }

        self.at_line_start = false;

        if self.is_at_end() {
            // End of file, emit remaining DEDENTs
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                let span = Span::new(self.position, self.position);
                return Ok(Some((Token::Dedent, span)));
            }
            return Ok(None);
        }

        let current_indent = *self.indent_stack.last().unwrap();
        let span = self.span_from(start);

        if indent > current_indent {
            // Increased indentation
            self.indent_stack.push(indent);
            return Ok(Some((Token::Indent, span)));
        } else if indent < current_indent {
            // Decreased indentation - may need multiple DEDENTs
            while let Some(&level) = self.indent_stack.last() {
                if level <= indent {
                    break;
                }
                self.indent_stack.pop();
                self.pending_dedents += 1;
            }

            // Check for inconsistent indentation
            if *self.indent_stack.last().unwrap() != indent {
                return Err(CompileError::new(
                    ErrorCode::InconsistentIndentation,
                    format!(
                        "Inconsistent indentation (expected {} spaces, found {})",
                        self.indent_stack.last().unwrap(),
                        indent
                    ),
                    span,
                ));
            }

            if self.pending_dedents > 0 {
                self.pending_dedents -= 1;
                return Ok(Some((Token::Dedent, span)));
            }
        }

        // Same indentation level, continue with normal tokenization
        self.next_token()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }
}
