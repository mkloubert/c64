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

//! Error types for the Cobra64 compiler.
//!
//! This module defines all error types used throughout the compiler,
//! including lexical, syntax, and semantic errors.

use std::ops::Range;
use thiserror::Error;

/// A source span representing a range in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    /// Create a new span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Create a span from a range.
    pub fn from_range(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }

    /// Get the length of this span.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans into one that covers both.
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self::from_range(range)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

/// Error codes for the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Lexical errors (E001-E030)
    InvalidCharacter,
    InvalidCharacterInIdentifier,
    InvalidDigitInNumber,
    UnterminatedString,
    UnterminatedCharLiteral,
    InvalidEscapeSequence,
    EmptyCharLiteral,
    CharLiteralTooLong,
    StringTooLong,
    InvalidHexEscape,
    IntegerTooLargeForByte,
    IntegerTooLargeForWord,
    InvalidBinaryDigit,
    InvalidHexDigit,
    EmptyNumberLiteral,
    InvalidDecimalLiteral,

    // Syntax errors (E100-E146)
    UnexpectedToken,
    UnexpectedEndOfFile,
    ExpectedToken,
    ExpectedExpression,
    ExpectedStatement,
    ExpectedIdentifier,
    ExpectedType,
    ExpectedNewline,
    InvalidType,
    InvalidAssignmentTarget,
    InvalidFunctionCall,
    ExpectedIndentedBlock,
    UnexpectedIndentation,
    InconsistentIndentation,
    TabNotAllowed,
    MixedTabsAndSpaces,
    ExpectedTypeName,
    ExpectedVariableName,
    ExpectedConstantValue,
    ArraySizeMustBePositive,
    ArraySizeTooLarge,
    ExpectedFunctionName,
    ExpectedOpenParen,
    ExpectedCloseParen,
    ExpectedColonAfterSignature,
    ExpectedArrowBeforeReturnType,
    DuplicateParameterName,
    ExpectedColonAfterCondition,
    ElifWithoutIf,
    ElseWithoutIf,
    ExpectedToOrDownto,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,

    // Semantic errors (E200-E242)
    UndefinedVariable,
    VariableAlreadyDefined,
    CannotAssignToConstant,
    VariableUsedBeforeInit,
    TypeMismatch,
    CannotConvert,
    InvalidOperatorForType,
    CannotCompareTypes,
    ArrayIndexMustBeInteger,
    CannotIndexNonArray,
    UndefinedFunction,
    FunctionAlreadyDefined,
    WrongNumberOfArguments,
    ArgumentTypeMismatch,
    MissingReturnStatement,
    CannotReturnValueFromVoid,
    MissingReturnValue,
    ConstantExpressionRequired,
    ArraySizeMustBeConstant,
    ConstantValueOutOfRange,
    ArrayIndexOutOfBounds,
    ArrayInitTooManyElements,
    ArrayInitTooFewElements,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl ErrorCode {
    /// Get the numeric code for this error.
    pub fn code(&self) -> &'static str {
        match self {
            // Lexical errors
            ErrorCode::InvalidCharacter => "E001",
            ErrorCode::InvalidCharacterInIdentifier => "E002",
            ErrorCode::InvalidDigitInNumber => "E003",
            ErrorCode::UnterminatedString => "E010",
            ErrorCode::UnterminatedCharLiteral => "E011",
            ErrorCode::InvalidEscapeSequence => "E012",
            ErrorCode::EmptyCharLiteral => "E013",
            ErrorCode::CharLiteralTooLong => "E014",
            ErrorCode::StringTooLong => "E015",
            ErrorCode::InvalidHexEscape => "E016",
            ErrorCode::IntegerTooLargeForByte => "E020",
            ErrorCode::IntegerTooLargeForWord => "E021",
            ErrorCode::InvalidBinaryDigit => "E022",
            ErrorCode::InvalidHexDigit => "E023",
            ErrorCode::EmptyNumberLiteral => "E024",
            ErrorCode::InvalidDecimalLiteral => "E025",

            // Syntax errors
            ErrorCode::UnexpectedToken => "E100",
            ErrorCode::UnexpectedEndOfFile => "E101",
            ErrorCode::ExpectedToken => "E102",
            ErrorCode::ExpectedExpression => "E103",
            ErrorCode::ExpectedStatement => "E104",
            ErrorCode::ExpectedIdentifier => "E105",
            ErrorCode::ExpectedType => "E106",
            ErrorCode::ExpectedNewline => "E107",
            ErrorCode::InvalidType => "E108",
            ErrorCode::InvalidAssignmentTarget => "E109",
            ErrorCode::InvalidFunctionCall => "E110",
            ErrorCode::ExpectedIndentedBlock => "E111",
            ErrorCode::UnexpectedIndentation => "E112",
            ErrorCode::InconsistentIndentation => "E113",
            ErrorCode::TabNotAllowed => "E114",
            ErrorCode::MixedTabsAndSpaces => "E115",
            ErrorCode::ExpectedTypeName => "E120",
            ErrorCode::ExpectedVariableName => "E121",
            ErrorCode::ExpectedConstantValue => "E122",
            ErrorCode::ArraySizeMustBePositive => "E123",
            ErrorCode::ArraySizeTooLarge => "E124",
            ErrorCode::ExpectedFunctionName => "E130",
            ErrorCode::ExpectedOpenParen => "E131",
            ErrorCode::ExpectedCloseParen => "E132",
            ErrorCode::ExpectedColonAfterSignature => "E133",
            ErrorCode::ExpectedArrowBeforeReturnType => "E134",
            ErrorCode::DuplicateParameterName => "E135",
            ErrorCode::ExpectedColonAfterCondition => "E140",
            ErrorCode::ElifWithoutIf => "E141",
            ErrorCode::ElseWithoutIf => "E142",
            ErrorCode::ExpectedToOrDownto => "E143",
            ErrorCode::BreakOutsideLoop => "E144",
            ErrorCode::ContinueOutsideLoop => "E145",
            ErrorCode::ReturnOutsideFunction => "E146",

            // Semantic errors
            ErrorCode::UndefinedVariable => "E200",
            ErrorCode::VariableAlreadyDefined => "E201",
            ErrorCode::CannotAssignToConstant => "E202",
            ErrorCode::VariableUsedBeforeInit => "E203",
            ErrorCode::TypeMismatch => "E210",
            ErrorCode::CannotConvert => "E211",
            ErrorCode::InvalidOperatorForType => "E212",
            ErrorCode::CannotCompareTypes => "E213",
            ErrorCode::ArrayIndexMustBeInteger => "E214",
            ErrorCode::CannotIndexNonArray => "E215",
            ErrorCode::UndefinedFunction => "E220",
            ErrorCode::FunctionAlreadyDefined => "E221",
            ErrorCode::WrongNumberOfArguments => "E222",
            ErrorCode::ArgumentTypeMismatch => "E223",
            ErrorCode::MissingReturnStatement => "E224",
            ErrorCode::CannotReturnValueFromVoid => "E225",
            ErrorCode::MissingReturnValue => "E226",
            ErrorCode::ConstantExpressionRequired => "E230",
            ErrorCode::ArraySizeMustBeConstant => "E231",
            ErrorCode::ConstantValueOutOfRange => "E232",
            ErrorCode::ArrayIndexOutOfBounds => "E240",
            ErrorCode::ArrayInitTooManyElements => "E241",
            ErrorCode::ArrayInitTooFewElements => "E242",
        }
    }
}

/// A compiler error with source location.
#[derive(Debug, Error)]
#[error("[{code}] {message}")]
pub struct CompileError {
    /// The error code.
    pub code: ErrorCode,
    /// The error message.
    pub message: String,
    /// The source span where the error occurred.
    pub span: Span,
    /// Optional hint for fixing the error.
    pub hint: Option<String>,
}

impl CompileError {
    /// Create a new compile error.
    pub fn new(code: ErrorCode, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            hint: None,
        }
    }

    /// Add a hint to this error.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Get the error code string.
    pub fn code_str(&self) -> &'static str {
        self.code.code()
    }
}

/// Result type for compiler operations.
pub type Result<T> = std::result::Result<T, CompileError>;

/// Source location with line and column information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// The content of the line.
    pub line_content: String,
}

impl SourceLocation {
    /// Calculate line and column from a byte offset in source code.
    pub fn from_offset(source: &str, offset: usize) -> Self {
        let offset = offset.min(source.len());
        let before = &source[..offset];

        let line = before.chars().filter(|&c| c == '\n').count() + 1;

        let last_newline = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = before[last_newline..].chars().count() + 1;

        // Extract the line content
        let line_start = last_newline;
        let line_end = source[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(source.len());
        let line_content = source[line_start..line_end].to_string();

        Self {
            line,
            column,
            line_content,
        }
    }
}

/// Format an error with source context.
pub fn format_error(error: &CompileError, source: &str, filename: Option<&str>) -> String {
    let loc = SourceLocation::from_offset(source, error.span.start);
    let filename = filename.unwrap_or("<input>");

    let mut output = String::new();

    // Error header
    output.push_str(&format!("error[{}]: {}\n", error.code_str(), error.message));

    // Location
    output.push_str(&format!("  --> {}:{}:{}\n", filename, loc.line, loc.column));

    // Source context
    let line_num_width = loc.line.to_string().len();
    output.push_str(&format!("{:>width$} |\n", "", width = line_num_width));
    output.push_str(&format!(
        "{:>width$} | {}\n",
        loc.line,
        loc.line_content,
        width = line_num_width
    ));

    // Underline the error span
    let underline_start = loc.column - 1;
    let underline_len = (error.span.end - error.span.start)
        .max(1)
        .min(loc.line_content.len().saturating_sub(underline_start));
    output.push_str(&format!(
        "{:>width$} | {:>start$}{}\n",
        "",
        "",
        "^".repeat(underline_len),
        width = line_num_width,
        start = underline_start
    ));

    // Hint if available
    if let Some(hint) = &error.hint {
        output.push_str(&format!(
            "{:>width$} = hint: {}\n",
            "",
            hint,
            width = line_num_width
        ));
    }

    output
}

/// A collection of compile errors.
#[derive(Debug, Default)]
pub struct Errors {
    errors: Vec<CompileError>,
}

impl Errors {
    /// Create a new empty error collection.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add an error to the collection.
    pub fn push(&mut self, error: CompileError) {
        self.errors.push(error);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get an iterator over the errors.
    pub fn iter(&self) -> impl Iterator<Item = &CompileError> {
        self.errors.iter()
    }

    /// Convert into a vector of errors.
    pub fn into_vec(self) -> Vec<CompileError> {
        self.errors
    }
}

impl IntoIterator for Errors {
    type Item = CompileError;
    type IntoIter = std::vec::IntoIter<CompileError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(15, 20);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_error_code() {
        assert_eq!(ErrorCode::InvalidCharacter.code(), "E001");
        assert_eq!(ErrorCode::UnexpectedToken.code(), "E100");
        assert_eq!(ErrorCode::UndefinedVariable.code(), "E200");
    }

    #[test]
    fn test_compile_error() {
        let error = CompileError::new(
            ErrorCode::UndefinedVariable,
            "Undefined variable 'foo'",
            Span::new(0, 3),
        )
        .with_hint("Did you mean 'bar'?");

        assert_eq!(error.code_str(), "E200");
        assert!(error.hint.is_some());
    }
}
