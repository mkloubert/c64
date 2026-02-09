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

//! Data block parsing for the Cobra64 compiler.
//!
//! This module handles parsing of data blocks which allow embedding
//! raw binary data directly in source code.
//!
//! # Syntax
//!
//! ```text
//! data BLOCK_NAME:
//!     $00, $3C, $00      # Hex bytes
//!     255, 128, 64       # Decimal bytes
//!     %11110000          # Binary bytes
//!     include "file.bin" # Include external file
//!     include "file.bin", $100, $200  # With offset and length
//! end
//! ```

use super::helpers::ParserHelpers;
use super::Parser;
use crate::ast::{DataBlock, DataEntry};
use crate::error::{CompileError, ErrorCode};
use crate::lexer::Token;

/// Extension trait for data block parsing.
pub trait DataBlockParser {
    /// Parse a data block definition.
    fn parse_data_block(&mut self) -> Result<DataBlock, CompileError>;

    /// Parse data entries until 'end' keyword.
    fn parse_data_entries(&mut self) -> Result<Vec<DataEntry>, CompileError>;

    /// Parse a single data entry (bytes or include directive).
    fn parse_data_entry(&mut self) -> Result<DataEntry, CompileError>;

    /// Parse inline byte values (hex, decimal, or binary).
    fn parse_inline_bytes(&mut self) -> Result<Vec<u8>, CompileError>;

    /// Parse an include directive.
    fn parse_include_directive(&mut self) -> Result<DataEntry, CompileError>;
}

impl<'a> DataBlockParser for Parser<'a> {
    fn parse_data_block(&mut self) -> Result<DataBlock, CompileError> {
        let start_span = self.peek_span().unwrap();

        // Consume 'data' keyword
        self.expect(&Token::Data, "Expected 'data'")?;

        // Parse block name
        let name = match self.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => return Err(self.error(ErrorCode::ExpectedIdentifier, "Expected data block name")),
        };

        // Expect colon after name
        self.expect(&Token::Colon, "Expected ':' after data block name")?;

        // Expect newline after colon
        self.expect(&Token::Newline, "Expected newline after ':'")?;

        // Consume INDENT token if present (from indented data entries)
        let has_indent = self.match_token(&Token::Indent);

        // Parse data entries
        let entries = self.parse_data_entries()?;

        // Consume DEDENT token if we had an indent
        if has_indent {
            self.match_token(&Token::Dedent);
        }

        // Expect 'end' keyword
        self.expect(&Token::End, "Expected 'end' to close data block")?;

        let end_span = self.previous_span();
        let span = start_span.merge(&end_span);

        Ok(DataBlock::new(name, span).with_entries(entries))
    }

    fn parse_data_entries(&mut self) -> Result<Vec<DataEntry>, CompileError> {
        let mut entries = Vec::new();

        // Skip leading newlines
        self.skip_newlines();

        while !self.check(&Token::End) && !self.check(&Token::Dedent) && !self.is_at_end() {
            // Skip any indentation/newline tokens
            while self.check(&Token::Newline) || self.check(&Token::Indent) {
                self.advance();
            }

            // Check for end of block after skipping whitespace
            if self.check(&Token::End) || self.check(&Token::Dedent) || self.is_at_end() {
                break;
            }

            let entry = self.parse_data_entry()?;
            entries.push(entry);

            // Consume separator (newline or comma)
            // Allow multiple newlines
            if self.match_token(&Token::Newline) {
                self.skip_newlines();
            }
        }

        Ok(entries)
    }

    fn parse_data_entry(&mut self) -> Result<DataEntry, CompileError> {
        // Check if this is an include directive
        if self.check(&Token::Include) {
            return self.parse_include_directive();
        }

        // Otherwise, parse inline byte values
        let bytes = self.parse_inline_bytes()?;
        Ok(DataEntry::bytes(bytes))
    }

    fn parse_inline_bytes(&mut self) -> Result<Vec<u8>, CompileError> {
        let mut bytes = Vec::new();

        loop {
            // Parse a byte value
            let byte = match self.peek() {
                Some(Token::Integer(n)) => {
                    let value = *n;
                    self.advance();

                    if value > 255 {
                        return Err(self.error(
                            ErrorCode::ValueOutOfRange,
                            &format!("Byte value {} exceeds maximum 255", value),
                        ));
                    }
                    value as u8
                }
                Some(Token::End) | Some(Token::Newline) | Some(Token::Include)
                | Some(Token::Dedent) => {
                    // End of this entry
                    break;
                }
                Some(t) => {
                    return Err(self.error(
                        ErrorCode::UnexpectedToken,
                        &format!("Expected byte value, got {}", t.name()),
                    ))
                }
                None => break,
            };

            bytes.push(byte);

            // Check for comma separator
            if self.match_token(&Token::Comma) {
                // Skip any newlines after comma (allows multi-line data)
                self.skip_newlines();

                // Check if next token is 'end', 'dedent', or another directive
                if self.check(&Token::End)
                    || self.check(&Token::Include)
                    || self.check(&Token::Dedent)
                {
                    break;
                }
            } else {
                // No comma, end of this line of bytes
                break;
            }
        }

        if bytes.is_empty() {
            return Err(self.error(
                ErrorCode::UnexpectedToken,
                "Expected at least one byte value",
            ));
        }

        Ok(bytes)
    }

    fn parse_include_directive(&mut self) -> Result<DataEntry, CompileError> {
        let start_span = self.peek_span().unwrap();

        // Consume 'include' keyword
        self.expect(&Token::Include, "Expected 'include'")?;

        // Parse file path (string literal)
        let path = match self.advance() {
            Some((Token::String(s), _)) => s,
            _ => return Err(self.error(ErrorCode::ExpectedString, "Expected file path string")),
        };

        // Check for optional offset and length
        // Syntax: include "file" [, offset [, length]]
        let (offset, length) = if self.match_token(&Token::Comma) {
            // Parse offset
            let offset = match self.advance() {
                Some((Token::Integer(n), _)) => n as u32,
                _ => {
                    return Err(self.error(
                        ErrorCode::ExpectedInteger,
                        "Expected offset value after ','",
                    ))
                }
            };

            // Check for optional length
            let length = if self.match_token(&Token::Comma) {
                // Parse length
                match self.advance() {
                    Some((Token::Integer(n), _)) => Some(n as u32),
                    _ => {
                        return Err(self.error(ErrorCode::ExpectedInteger, "Expected length value"))
                    }
                }
            } else {
                // Length is optional - read from offset to end of file
                None
            };

            (Some(offset), length)
        } else {
            (None, None)
        };

        let end_span = self.previous_span();
        let span = start_span.merge(&end_span);

        match (offset, length) {
            (Some(off), Some(len)) => Ok(DataEntry::include_with_range(path, off, len, span)),
            (Some(off), None) => Ok(DataEntry::include_with_offset(path, off, span)),
            (None, _) => Ok(DataEntry::include(path, span)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn parse_source(source: &str) -> Result<crate::ast::Program, CompileError> {
        let tokens = tokenize(source)?;
        parse(&tokens)
    }

    #[test]
    fn test_parse_simple_data_block() {
        let source = r#"
data SPRITE:
    $00, $3C, $00
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_decimal_values() {
        let source = r#"
data VALUES:
    255, 128, 64, 32
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_multi_line() {
        let source = r#"
data SPRITE_DATA:
    $00, $3C, $00
    $00, $7E, $00
    $00, $FF, $00
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_with_include() {
        let source = r#"
data FONT:
    include "font.bin"
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_include_with_range() {
        let source = r#"
data MUSIC:
    include "music.sid", $7E, $1000
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_mixed_entries() {
        let source = r#"
data MIXED:
    $00, $01, $02
    include "extra.bin"
    $FF, $FE, $FD
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_binary_values() {
        let source = r#"
data BITMAP:
    %11110000, %00001111
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_data_block_value_out_of_range() {
        let source = r#"
data BAD:
    256
end

def main():
    pass
"#;
        let result = parse_source(source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::ValueOutOfRange);
    }

    #[test]
    fn test_parse_data_block_missing_end() {
        let source = r#"
data INCOMPLETE:
    $00, $01

def main():
    pass
"#;
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_data_blocks() {
        let source = r#"
data SPRITE1:
    $00, $01, $02
end

data SPRITE2:
    $10, $11, $12
end

def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 3);
    }
}
