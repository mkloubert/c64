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
//!
//! # Module Structure
//!
//! - `helpers` - Character navigation and span creation (LexerHelpers trait)
//! - `identifiers` - Identifier and keyword scanning (IdentifierScanner trait)
//! - `indentation` - Indentation handling (IndentationHandler trait)
//! - `numbers` - Number literal scanning (NumberScanner trait)
//! - `operators` - Operator and punctuation scanning (OperatorScanner trait)
//! - `strings` - String and character literal scanning (StringScanner trait)
//! - `tokens` - Token definitions and identifier validation

mod helpers;
mod identifiers;
mod indentation;
mod numbers;
mod operators;
mod strings;
mod tokens;

pub use tokens::Token;

use crate::error::{CompileError, Span};
use helpers::LexerHelpers;
use identifiers::IdentifierScanner;
use indentation::IndentationHandler;
use numbers::NumberScanner;
use operators::OperatorScanner;
use strings::StringScanner;

/// The lexer state for tokenizing source code.
pub struct Lexer<'source> {
    /// The source code being tokenized.
    pub(crate) source: &'source str,
    /// Current byte position in the source.
    pub(crate) position: usize,
    /// Current line number (1-indexed).
    pub(crate) line: usize,
    /// Current column number (1-indexed).
    pub(crate) column: usize,
    /// Stack of indentation levels.
    pub(crate) indent_stack: Vec<usize>,
    /// Pending DEDENT tokens to emit.
    pub(crate) pending_dedents: usize,
    /// Whether we're at the start of a line.
    pub(crate) at_line_start: bool,
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
            return self.scan_identifier().map(Some);
        }

        // Operators and punctuation
        self.scan_operator_or_punctuation()
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
        let tokens = tokenize("def").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Def));
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

    // ========================================
    // Negative Literal Tests (for signed types)
    // ========================================

    #[test]
    fn test_negative_decimal_literal() {
        // Negative literals are tokenized as [Minus, Integer]
        let tokens = tokenize("-128").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(128)));
    }

    #[test]
    fn test_negative_decimal_literal_word() {
        let tokens = tokenize("-32768").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(32768)));
    }

    #[test]
    fn test_negative_hex_literal() {
        // -$7F = -127
        let tokens = tokenize("-$7F").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(127)));
    }

    #[test]
    fn test_negative_hex_literal_word() {
        // -$7FFF = -32767
        let tokens = tokenize("-$7FFF").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(32767)));
    }

    #[test]
    fn test_negative_binary_literal() {
        // -%01111111 = -127
        let tokens = tokenize("-%01111111").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(127)));
    }

    #[test]
    fn test_negative_binary_literal_word() {
        // -%0111111111111111 = -32767
        let tokens = tokenize("-%0111111111111111").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(32767)));
    }

    #[test]
    fn test_negative_zero() {
        let tokens = tokenize("-0").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(0)));
    }

    #[test]
    fn test_negative_one() {
        let tokens = tokenize("-1").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(tokens[1].0, Token::Integer(1)));
    }

    #[test]
    fn test_negative_literal_in_expression() {
        // Ensure negative literals work in expressions
        let tokens = tokenize("x = -100").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[1].0, Token::Equal));
        assert!(matches!(tokens[2].0, Token::Minus));
        assert!(matches!(tokens[3].0, Token::Integer(100)));
    }

    #[test]
    fn test_subtraction_vs_negative() {
        // a - 1 should be [a, Minus, 1] - subtraction
        let tokens = tokenize("a - 1").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "a"));
        assert!(matches!(tokens[1].0, Token::Minus));
        assert!(matches!(tokens[2].0, Token::Integer(1)));
    }

    #[test]
    fn test_negative_literal_no_space() {
        // -128 without space should still tokenize correctly
        let tokens = tokenize("x=-128").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[1].0, Token::Equal));
        assert!(matches!(tokens[2].0, Token::Minus));
        assert!(matches!(tokens[3].0, Token::Integer(128)));
    }

    #[test]
    fn test_signed_type_declaration_with_negative() {
        let tokens = tokenize("x: sbyte = -100").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[1].0, Token::Colon));
        assert!(matches!(tokens[2].0, Token::Sbyte));
        assert!(matches!(tokens[3].0, Token::Equal));
        assert!(matches!(tokens[4].0, Token::Minus));
        assert!(matches!(tokens[5].0, Token::Integer(100)));
    }

    #[test]
    fn test_sword_type_declaration_with_negative() {
        let tokens = tokenize("y: sword = -30000").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "y"));
        assert!(matches!(tokens[1].0, Token::Colon));
        assert!(matches!(tokens[2].0, Token::Sword));
        assert!(matches!(tokens[3].0, Token::Equal));
        assert!(matches!(tokens[4].0, Token::Minus));
        assert!(matches!(tokens[5].0, Token::Integer(30000)));
    }

    // ========================================
    // Decimal Literal Tests (for fixed/float)
    // ========================================

    #[test]
    fn test_decimal_simple() {
        let tokens = tokenize("3.14").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "3.14"));
    }

    #[test]
    fn test_decimal_zero_prefix() {
        let tokens = tokenize("0.5").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "0.5"));
    }

    #[test]
    fn test_decimal_multiple_fractional_digits() {
        let tokens = tokenize("3.14159").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "3.14159"));
    }

    #[test]
    fn test_decimal_zero() {
        let tokens = tokenize("0.0").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "0.0"));
    }

    #[test]
    fn test_decimal_large_integer_part() {
        let tokens = tokenize("1234.5678").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "1234.5678"));
    }

    #[test]
    fn test_decimal_scientific_notation() {
        let tokens = tokenize("1.5e3").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "1.5e3"));
    }

    #[test]
    fn test_decimal_scientific_notation_uppercase() {
        let tokens = tokenize("1.5E3").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "1.5E3"));
    }

    #[test]
    fn test_decimal_scientific_notation_negative_exp() {
        let tokens = tokenize("2.0e-5").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "2.0e-5"));
    }

    #[test]
    fn test_decimal_scientific_notation_positive_exp() {
        let tokens = tokenize("1.0e+10").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "1.0e+10"));
    }

    #[test]
    fn test_decimal_scientific_no_decimal_point() {
        let tokens = tokenize("1e5").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Decimal(s) if s == "1e5"));
    }

    #[test]
    fn test_integer_not_decimal() {
        // Regular integers should not become decimals
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].0, Token::Integer(42)));
    }

    #[test]
    fn test_decimal_in_expression() {
        let tokens = tokenize("x = 3.14 + 2.0").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[1].0, Token::Equal));
        assert!(matches!(&tokens[2].0, Token::Decimal(s) if s == "3.14"));
        assert!(matches!(tokens[3].0, Token::Plus));
        assert!(matches!(&tokens[4].0, Token::Decimal(s) if s == "2.0"));
    }

    #[test]
    fn test_decimal_negative() {
        // Negative decimals are tokenized as [Minus, Decimal]
        let tokens = tokenize("-3.14").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Minus));
        assert!(matches!(&tokens[1].0, Token::Decimal(s) if s == "3.14"));
    }

    #[test]
    fn test_decimal_with_fixed_type() {
        let tokens = tokenize("x: fixed = 3.75").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[1].0, Token::Colon));
        assert!(matches!(tokens[2].0, Token::Fixed));
        assert!(matches!(tokens[3].0, Token::Equal));
        assert!(matches!(&tokens[4].0, Token::Decimal(s) if s == "3.75"));
    }

    #[test]
    fn test_decimal_with_float_type() {
        let tokens = tokenize("y: float = 3.14").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "y"));
        assert!(matches!(tokens[1].0, Token::Colon));
        assert!(matches!(tokens[2].0, Token::Float));
        assert!(matches!(tokens[3].0, Token::Equal));
        assert!(matches!(&tokens[4].0, Token::Decimal(s) if s == "3.14"));
    }

    #[test]
    fn test_fixed_float_keywords() {
        let tokens = tokenize("fixed float").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].0, Token::Fixed));
        assert!(matches!(tokens[1].0, Token::Float));
    }

    #[test]
    fn test_decimal_invalid_exponent() {
        let result = tokenize("1.5e");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidDecimalLiteral);
    }

    #[test]
    fn test_decimal_invalid_trailing_char() {
        let result = tokenize("3.14x");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidDecimalLiteral);
    }

    #[test]
    fn test_decimal_scientific_invalid_trailing_char() {
        let result = tokenize("1e5x");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidDecimalLiteral);
    }

    // ========================================
    // Identifier Naming Convention Tests
    // ========================================

    #[test]
    fn test_uppercase_identifiers_valid() {
        // Uppercase identifiers are valid
        let tokens = tokenize("MY_CONST").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "MY_CONST"));

        let tokens = tokenize("_MY_CONST").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_MY_CONST"));

        let tokens = tokenize("_3MY_CONST").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_3MY_CONST"));

        let tokens = tokenize("A").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "A"));

        let tokens = tokenize("B2").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "B2"));

        let tokens = tokenize("C_3").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "C_3"));

        let tokens = tokenize("_______MY_CONST6").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_______MY_CONST6"));
    }

    #[test]
    fn test_lowercase_identifiers_valid() {
        // Lowercase and mixed case identifiers are valid
        let tokens = tokenize("myVar").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "myVar"));

        let tokens = tokenize("myVar_5").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "myVar_5"));

        let tokens = tokenize("a").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "a"));

        let tokens = tokenize("b2").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "b2"));

        let tokens = tokenize("c_3").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "c_3"));

        let tokens = tokenize("_4d").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_4d"));

        let tokens = tokenize("_e666").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_e666"));

        let tokens = tokenize("_f_777").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_f_777"));
    }

    #[test]
    fn test_mixed_case_identifiers_valid() {
        // Mixed case identifiers are now valid (no naming convention enforced)
        let tokens = tokenize("MyConst").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "MyConst"));

        let tokens = tokenize("_My_Const").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_My_Const"));

        let tokens = tokenize("_3My_Const").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_3My_Const"));

        let tokens = tokenize("MY_Const").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "MY_Const"));
    }

    #[test]
    fn test_underscore_only_invalid() {
        // Only underscores is invalid
        let result = tokenize("_");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IdentifierOnlyUnderscore);

        let result = tokenize("__");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IdentifierOnlyUnderscore);

        let result = tokenize("___");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IdentifierOnlyUnderscore);
    }

    #[test]
    fn test_identifier_with_leading_underscore_and_digit() {
        // Identifiers can start with underscore followed by digits and letters
        let tokens = tokenize("_4D").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_4D"));

        let tokens = tokenize("_4d").unwrap();
        assert!(matches!(&tokens[0].0, Token::Identifier(s) if s == "_4d"));
    }
}
