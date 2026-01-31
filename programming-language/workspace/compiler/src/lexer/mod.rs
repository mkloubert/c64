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

//! Lexer module for the Cobra64 compiler.
//!
//! This module tokenizes Cobra64 source code into a stream of tokens.
//! It handles:
//! - Keywords and identifiers
//! - Number literals (decimal, hex with $, binary with %)
//! - String and character literals
//! - Operators and punctuation
//! - Indentation tracking (INDENT/DEDENT tokens)
//! - Comments (starting with #)

mod tokens;

pub use tokens::Token;

use crate::error::{CompileError, ErrorCode, Span};

/// The lexer state for tokenizing source code.
pub struct Lexer<'source> {
    /// The source code being tokenized.
    source: &'source str,
    /// Current byte position in the source.
    position: usize,
    /// Current line number (1-indexed).
    line: usize,
    /// Current column number (1-indexed).
    column: usize,
    /// Stack of indentation levels.
    indent_stack: Vec<usize>,
    /// Pending DEDENT tokens to emit.
    pending_dedents: usize,
    /// Whether we're at the start of a line.
    at_line_start: bool,
}

impl<'source> Lexer<'source> {
    /// Create a new lexer for the given source code.
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
        }
    }

    /// Get the current position in the source.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the current line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get the current column number.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Check if we've reached the end of the source.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Peek at the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Peek at the next character without advancing.
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.position..].chars();
        chars.next();
        chars.next()
    }

    /// Advance to the next character and return it.
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

    /// Create a span from start position to current position.
    fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.position)
    }

    /// Get the next token from the source.
    pub fn next_token(&mut self) -> Result<Option<(Token, Span)>, CompileError> {
        // First, emit any pending DEDENT tokens
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            let span = Span::new(self.position, self.position);
            return Ok(Some((Token::Dedent, span)));
        }

        // Skip whitespace (but handle indentation at line start)
        if self.at_line_start {
            return self.handle_line_start();
        }

        self.skip_whitespace();

        if self.is_at_end() {
            // Emit remaining DEDENT tokens at end of file
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                let span = Span::new(self.position, self.position);
                return Ok(Some((Token::Dedent, span)));
            }
            return Ok(None);
        }

        let start = self.position;
        let c = self.peek().unwrap();

        // Comments
        if c == '#' {
            self.skip_comment();
            return self.next_token();
        }

        // Newline
        if c == '\n' {
            self.advance();
            let span = self.span_from(start);
            return Ok(Some((Token::Newline, span)));
        }

        // String literal
        if c == '"' {
            return self.scan_string().map(Some);
        }

        // Character literal
        if c == '\'' {
            return self.scan_char().map(Some);
        }

        // Number literals
        if c == '$' {
            return self.scan_hex_number().map(Some);
        }
        if c == '%' && self.peek_next().is_some_and(|c| c == '0' || c == '1') {
            return self.scan_binary_number().map(Some);
        }
        if c.is_ascii_digit() {
            return self.scan_decimal_number().map(Some);
        }

        // Identifiers and keywords
        if c.is_ascii_alphabetic() || c == '_' {
            return Ok(Some(self.scan_identifier()));
        }

        // Operators and punctuation
        self.scan_operator_or_punctuation()
    }

    /// Handle indentation at the start of a line.
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

    /// Skip whitespace (spaces only, not newlines).
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a comment (from # to end of line).
    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Scan a string literal.
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

    /// Scan a character literal.
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

    /// Scan a decimal number literal.
    fn scan_decimal_number(&mut self) -> Result<(Token, Span), CompileError> {
        let start = self.position;
        let mut value: u64 = 0;

        while let Some(c) = self.peek() {
            if let Some(digit) = c.to_digit(10) {
                self.advance();
                value = value.saturating_mul(10).saturating_add(digit as u64);
            } else if c.is_ascii_alphanumeric() || c == '_' {
                return Err(CompileError::new(
                    ErrorCode::InvalidDigitInNumber,
                    "Invalid digit in number literal",
                    self.span_from(start),
                ));
            } else {
                break;
            }
        }

        if value > 65535 {
            return Err(CompileError::new(
                ErrorCode::IntegerTooLargeForWord,
                "Integer literal too large for word (max 65535)",
                self.span_from(start),
            ));
        }

        Ok((Token::Integer(value as u16), self.span_from(start)))
    }

    /// Scan a hexadecimal number literal (starting with $).
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

    /// Scan a binary number literal (starting with %).
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

    /// Scan an identifier or keyword.
    fn scan_identifier(&mut self) -> (Token, Span) {
        let start = self.position;

        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        let token = Token::from_keyword_or_identifier(text);
        (token, self.span_from(start))
    }

    /// Scan an operator or punctuation.
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

/// Tokenize source code into a vector of tokens with spans.
pub fn tokenize(source: &str) -> Result<Vec<(Token, Span)>, CompileError> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    while let Some(token_span) = lexer.next_token()? {
        tokens.push(token_span);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    // ========================================
    // Basic Token Tests
    // ========================================

    #[test]
    fn test_arithmetic_operators() {
        let tokens = tokenize("+ - * / %").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].0, Token::Plus));
        assert!(matches!(tokens[1].0, Token::Minus));
        assert!(matches!(tokens[2].0, Token::Star));
        assert!(matches!(tokens[3].0, Token::Slash));
        assert!(matches!(tokens[4].0, Token::Percent));
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("== != < > <= >=").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].0, Token::EqualEqual));
        assert!(matches!(tokens[1].0, Token::BangEqual));
        assert!(matches!(tokens[2].0, Token::Less));
        assert!(matches!(tokens[3].0, Token::Greater));
        assert!(matches!(tokens[4].0, Token::LessEqual));
        assert!(matches!(tokens[5].0, Token::GreaterEqual));
    }

    #[test]
    fn test_bitwise_operators() {
        let tokens = tokenize("& | ^ ~ << >>").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].0, Token::Ampersand));
        assert!(matches!(tokens[1].0, Token::Pipe));
        assert!(matches!(tokens[2].0, Token::Caret));
        assert!(matches!(tokens[3].0, Token::Tilde));
        assert!(matches!(tokens[4].0, Token::ShiftLeft));
        assert!(matches!(tokens[5].0, Token::ShiftRight));
    }

    #[test]
    fn test_assignment_operators() {
        let tokens = tokenize("= += -= *= /= %= &= |= ^= <<= >>=").unwrap();
        assert_eq!(tokens.len(), 11);
        assert!(matches!(tokens[0].0, Token::Equal));
        assert!(matches!(tokens[1].0, Token::PlusAssign));
        assert!(matches!(tokens[2].0, Token::MinusAssign));
        assert!(matches!(tokens[3].0, Token::StarAssign));
        assert!(matches!(tokens[4].0, Token::SlashAssign));
        assert!(matches!(tokens[5].0, Token::PercentAssign));
        assert!(matches!(tokens[6].0, Token::AmpersandAssign));
        assert!(matches!(tokens[7].0, Token::PipeAssign));
        assert!(matches!(tokens[8].0, Token::CaretAssign));
        assert!(matches!(tokens[9].0, Token::ShiftLeftAssign));
        assert!(matches!(tokens[10].0, Token::ShiftRightAssign));
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize("( ) [ ] : , ->").unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0].0, Token::LeftParen));
        assert!(matches!(tokens[1].0, Token::RightParen));
        assert!(matches!(tokens[2].0, Token::LeftBracket));
        assert!(matches!(tokens[3].0, Token::RightBracket));
        assert!(matches!(tokens[4].0, Token::Colon));
        assert!(matches!(tokens[5].0, Token::Comma));
        assert!(matches!(tokens[6].0, Token::Arrow));
    }

    // ========================================
    // Keyword Tests
    // ========================================

    #[test]
    fn test_type_keywords() {
        let tokens = tokenize("byte word sbyte sword bool string").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].0, Token::Byte));
        assert!(matches!(tokens[1].0, Token::Word));
        assert!(matches!(tokens[2].0, Token::Sbyte));
        assert!(matches!(tokens[3].0, Token::Sword));
        assert!(matches!(tokens[4].0, Token::Bool));
        assert!(matches!(tokens[5].0, Token::StringType));
    }

    #[test]
    fn test_control_flow_keywords() {
        let tokens =
            tokenize("if elif else while for in to downto break continue return pass").unwrap();
        assert_eq!(tokens.len(), 12);
        assert!(matches!(tokens[0].0, Token::If));
        assert!(matches!(tokens[1].0, Token::Elif));
        assert!(matches!(tokens[2].0, Token::Else));
        assert!(matches!(tokens[3].0, Token::While));
        assert!(matches!(tokens[4].0, Token::For));
        assert!(matches!(tokens[5].0, Token::In));
        assert!(matches!(tokens[6].0, Token::To));
        assert!(matches!(tokens[7].0, Token::Downto));
        assert!(matches!(tokens[8].0, Token::Break));
        assert!(matches!(tokens[9].0, Token::Continue));
        assert!(matches!(tokens[10].0, Token::Return));
        assert!(matches!(tokens[11].0, Token::Pass));
    }

    #[test]
    fn test_logical_keywords() {
        let tokens = tokenize("and or not").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].0, Token::And));
        assert!(matches!(tokens[1].0, Token::Or));
        assert!(matches!(tokens[2].0, Token::Not));
    }

    #[test]
    fn test_definition_keywords() {
        let tokens = tokenize("const def").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Const));
        assert!(matches!(tokens[1].0, Token::Def));
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = tokenize("true false").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::True));
        assert!(matches!(tokens[1].0, Token::False));
    }

    // ========================================
    // Identifier Tests
    // ========================================

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("foo bar _baz x1 player_score MAX_VALUE").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "foo"));
        assert!(matches!(&tokens[1].0, Token::Identifier(s) if s == "bar"));
        assert!(matches!(&tokens[2].0, Token::Identifier(s) if s == "_baz"));
        assert!(matches!(&tokens[3].0, Token::Identifier(s) if s == "x1"));
        assert!(matches!(&tokens[4].0, Token::Identifier(s) if s == "player_score"));
        assert!(matches!(&tokens[5].0, Token::Identifier(s) if s == "MAX_VALUE"));
    }

    #[test]
    fn test_identifier_starting_with_keyword() {
        let tokens = tokenize("iffy whileloop bypass").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "iffy"));
        assert!(matches!(&tokens[1].0, Token::Identifier(s) if s == "whileloop"));
        assert!(matches!(&tokens[2].0, Token::Identifier(s) if s == "bypass"));
    }

    // ========================================
    // Number Literal Tests
    // ========================================

    #[test]
    fn test_decimal_numbers() {
        let tokens = tokenize("0 1 42 255 256 65535").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].0, Token::Integer(0)));
        assert!(matches!(tokens[1].0, Token::Integer(1)));
        assert!(matches!(tokens[2].0, Token::Integer(42)));
        assert!(matches!(tokens[3].0, Token::Integer(255)));
        assert!(matches!(tokens[4].0, Token::Integer(256)));
        assert!(matches!(tokens[5].0, Token::Integer(65535)));
    }

    #[test]
    fn test_hex_numbers() {
        let tokens = tokenize("$0 $FF $ff $0801 $FFFF").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].0, Token::Integer(0)));
        assert!(matches!(tokens[1].0, Token::Integer(255)));
        assert!(matches!(tokens[2].0, Token::Integer(255)));
        assert!(matches!(tokens[3].0, Token::Integer(0x0801)));
        assert!(matches!(tokens[4].0, Token::Integer(65535)));
    }

    #[test]
    fn test_binary_numbers() {
        let tokens = tokenize("%0 %1 %1010 %11111111 %1111111111111111").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].0, Token::Integer(0)));
        assert!(matches!(tokens[1].0, Token::Integer(1)));
        assert!(matches!(tokens[2].0, Token::Integer(10)));
        assert!(matches!(tokens[3].0, Token::Integer(255)));
        assert!(matches!(tokens[4].0, Token::Integer(65535)));
    }

    #[test]
    fn test_number_overflow() {
        let result = tokenize("65536");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IntegerTooLargeForWord);
    }

    #[test]
    fn test_hex_overflow() {
        let result = tokenize("$10000");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IntegerTooLargeForWord);
    }

    #[test]
    fn test_binary_overflow() {
        let result = tokenize("%10000000000000000");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IntegerTooLargeForWord);
    }

    #[test]
    fn test_invalid_decimal_digit() {
        let result = tokenize("123abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidDigitInNumber);
    }

    #[test]
    fn test_invalid_hex_digit() {
        let result = tokenize("$FFG");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidHexDigit);
    }

    #[test]
    fn test_invalid_binary_digit() {
        let result = tokenize("%1012");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidBinaryDigit);
    }

    #[test]
    fn test_empty_hex_number() {
        let result = tokenize("$ ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::EmptyNumberLiteral);
    }

    // ========================================
    // String Literal Tests
    // ========================================

    #[test]
    fn test_string_simple() {
        let tokens = tokenize(r#""hello""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello"));
    }

    #[test]
    fn test_string_empty() {
        let tokens = tokenize(r#""""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s.is_empty()));
    }

    #[test]
    fn test_string_with_spaces() {
        let tokens = tokenize(r#""hello world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello world"));
    }

    #[test]
    fn test_string_escape_newline() {
        let tokens = tokenize(r#""hello\nworld""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\nworld"));
    }

    #[test]
    fn test_string_escape_tab() {
        let tokens = tokenize(r#""hello\tworld""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\tworld"));
    }

    #[test]
    fn test_string_escape_carriage_return() {
        let tokens = tokenize(r#""hello\rworld""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\rworld"));
    }

    #[test]
    fn test_string_escape_backslash() {
        let tokens = tokenize(r#""hello\\world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\\world"));
    }

    #[test]
    fn test_string_escape_quote() {
        let tokens = tokenize(r#""hello\"world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\"world"));
    }

    #[test]
    fn test_string_escape_null() {
        let tokens = tokenize(r#""hello\0world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "hello\0world"));
    }

    #[test]
    fn test_string_all_escapes() {
        let tokens = tokenize(r#""\n\r\t\\\"\'\0""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "\n\r\t\\\"'\0"));
    }

    #[test]
    fn test_unterminated_string() {
        let result = tokenize(r#""hello"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::UnterminatedString);
    }

    #[test]
    fn test_unterminated_string_newline() {
        let result = tokenize("\"hello\nworld\"");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::UnterminatedString);
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let result = tokenize(r#""hello\xworld""#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidEscapeSequence);
    }

    #[test]
    fn test_string_too_long() {
        let long_string = format!(r#""{}""#, "a".repeat(256));
        let result = tokenize(&long_string);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::StringTooLong);
    }

    // ========================================
    // Character Literal Tests
    // ========================================

    #[test]
    fn test_char_simple() {
        let tokens = tokenize("'a'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('a')));
    }

    #[test]
    fn test_char_digit() {
        let tokens = tokenize("'5'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('5')));
    }

    #[test]
    fn test_char_escape_newline() {
        let tokens = tokenize(r"'\n'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('\n')));
    }

    #[test]
    fn test_char_escape_tab() {
        let tokens = tokenize(r"'\t'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('\t')));
    }

    #[test]
    fn test_char_escape_backslash() {
        let tokens = tokenize(r"'\\'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('\\')));
    }

    #[test]
    fn test_char_escape_single_quote() {
        let tokens = tokenize(r"'\''").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('\'')));
    }

    #[test]
    fn test_char_escape_null() {
        let tokens = tokenize(r"'\0'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Char('\0')));
    }

    #[test]
    fn test_empty_char() {
        let result = tokenize("''");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::EmptyCharLiteral);
    }

    #[test]
    fn test_char_too_long() {
        let result = tokenize("'ab'");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::CharLiteralTooLong);
    }

    #[test]
    fn test_unterminated_char() {
        let result = tokenize("'a");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::UnterminatedCharLiteral);
    }

    // ========================================
    // Comment Tests
    // ========================================

    #[test]
    fn test_comment_skipped() {
        let tokens = tokenize("x # this is a comment\ny").unwrap();
        // Should have: x, NEWLINE, y
        let non_newline: Vec<_> = tokens
            .iter()
            .filter(|(t, _)| !matches!(t, Token::Newline))
            .collect();
        assert_eq!(non_newline.len(), 2);
        assert!(matches!(&non_newline[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(&non_newline[1].0, Token::Identifier(s) if s == "y"));
    }

    #[test]
    fn test_comment_at_line_start() {
        let tokens = tokenize("# comment\nx").unwrap();
        let non_newline: Vec<_> = tokens
            .iter()
            .filter(|(t, _)| !matches!(t, Token::Newline))
            .collect();
        assert_eq!(non_newline.len(), 1);
        assert!(matches!(&non_newline[0].0, Token::Identifier(s) if s == "x"));
    }

    #[test]
    fn test_comment_only_file() {
        let tokens = tokenize("# just a comment").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_hash_in_string_not_comment() {
        let tokens = tokenize(r##""# not a comment""##).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::String(s) if s == "# not a comment"));
    }

    // ========================================
    // Indentation Tests
    // ========================================

    #[test]
    fn test_indent() {
        let source = "if x:\n    y";
        let tokens = tokenize(source).unwrap();
        let token_types: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert!(token_types.contains(&Token::Indent));
    }

    #[test]
    fn test_dedent() {
        let source = "if x:\n    y\nz";
        let tokens = tokenize(source).unwrap();
        let token_types: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert!(token_types.contains(&Token::Indent));
        assert!(token_types.contains(&Token::Dedent));
    }

    #[test]
    fn test_multiple_indent_levels() {
        let source = "if x:\n    if y:\n        z";
        let tokens = tokenize(source).unwrap();
        let indent_count = tokens
            .iter()
            .filter(|(t, _)| matches!(t, Token::Indent))
            .count();
        assert_eq!(indent_count, 2);
    }

    #[test]
    fn test_multiple_dedents() {
        let source = "if x:\n    if y:\n        z\na";
        let tokens = tokenize(source).unwrap();
        let dedent_count = tokens
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent))
            .count();
        assert_eq!(dedent_count, 2);
    }

    #[test]
    fn test_tab_error() {
        let result = tokenize("if x:\n\ty");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::TabNotAllowed);
    }

    #[test]
    fn test_inconsistent_indentation() {
        let source = "if x:\n    y\n  z"; // 4 spaces then 2 spaces
        let result = tokenize(source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InconsistentIndentation);
    }

    #[test]
    fn test_empty_lines_ignored() {
        let source = "x\n\n\ny";
        let tokens = tokenize(source).unwrap();
        let ids: Vec<_> = tokens
            .iter()
            .filter(|(t, _)| matches!(t, Token::Identifier(_)))
            .collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_dedent_at_eof() {
        let source = "if x:\n    y";
        let tokens = tokenize(source).unwrap();
        // Should emit DEDENT at end of file
        let dedent_count = tokens
            .iter()
            .filter(|(t, _)| matches!(t, Token::Dedent))
            .count();
        assert_eq!(dedent_count, 1);
    }

    // ========================================
    // Invalid Character Tests
    // ========================================

    #[test]
    fn test_invalid_character() {
        let result = tokenize("@");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidCharacter);
    }

    #[test]
    fn test_lone_bang() {
        let result = tokenize("!");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidCharacter);
    }

    // ========================================
    // Source Location Tests
    // ========================================

    #[test]
    fn test_span_tracking() {
        let tokens = tokenize("foo bar").unwrap();
        assert_eq!(tokens[0].1.start, 0);
        assert_eq!(tokens[0].1.end, 3);
        assert_eq!(tokens[1].1.start, 4);
        assert_eq!(tokens[1].1.end, 7);
    }

    #[test]
    fn test_multiline_span() {
        let tokens = tokenize("x\ny").unwrap();
        let y_token = tokens
            .iter()
            .find(|(t, _)| matches!(t, Token::Identifier(s) if s == "y"))
            .unwrap();
        assert_eq!(y_token.1.start, 2);
    }

    // ========================================
    // Complex Expression Tests
    // ========================================

    #[test]
    fn test_complex_expression() {
        let tokens = tokenize("a + b * c - d / e").unwrap();
        assert_eq!(tokens.len(), 9);
    }

    #[test]
    fn test_function_call() {
        let tokens = tokenize("print(x, y)").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "print"));
        assert!(matches!(tokens[1].0, Token::LeftParen));
        assert!(matches!(&tokens[2].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[3].0, Token::Comma));
        assert!(matches!(&tokens[4].0, Token::Identifier(s) if s == "y"));
        assert!(matches!(tokens[5].0, Token::RightParen));
    }

    #[test]
    fn test_array_access() {
        let tokens = tokenize("arr[i]").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "arr"));
        assert!(matches!(tokens[1].0, Token::LeftBracket));
        assert!(matches!(&tokens[2].0, Token::Identifier(s) if s == "i"));
        assert!(matches!(tokens[3].0, Token::RightBracket));
    }

    #[test]
    fn test_function_definition() {
        let tokens = tokenize("def main() -> byte:").unwrap();
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0].0, Token::Def));
        assert!(matches!(&tokens[1].0, Token::Identifier(s) if s == "main"));
        assert!(matches!(tokens[2].0, Token::LeftParen));
        assert!(matches!(tokens[3].0, Token::RightParen));
        assert!(matches!(tokens[4].0, Token::Arrow));
        assert!(matches!(tokens[5].0, Token::Byte));
        assert!(matches!(tokens[6].0, Token::Colon));
    }

    #[test]
    fn test_complete_program() {
        let source = r#"
def main():
    byte x = 42
    if x > 0:
        print("positive")
    else:
        print("zero or negative")
"#;
        let result = tokenize(source);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
    }
}
