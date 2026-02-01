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

//! Token definitions for the Cobra64 language.

/// A token in the Cobra64 language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Integer literal (decimal, hex, or binary).
    Integer(u16),
    /// Decimal literal (number with decimal point or scientific notation).
    /// Stored as string for later conversion to fixed or float based on context.
    /// Examples: "3.14", "0.5", "1.5e-3", ".25"
    Decimal(String),
    /// String literal.
    String(String),
    /// Character literal.
    Char(char),
    /// Identifier (variable or function name).
    Identifier(String),

    // Type keywords
    /// `byte` - 8-bit unsigned integer type.
    Byte,
    /// `word` - 16-bit unsigned integer type.
    Word,
    /// `sbyte` - 8-bit signed integer type.
    Sbyte,
    /// `sword` - 16-bit signed integer type.
    Sword,
    /// `bool` - boolean type.
    Bool,
    /// `string` - string type.
    StringType,
    /// `fixed` - 12.4 fixed-point type.
    Fixed,
    /// `float` - IEEE-754 binary16 floating-point type.
    Float,

    // Definition keywords
    /// `def` - function definition.
    Def,

    // Control flow keywords
    /// `if` - conditional statement.
    If,
    /// `elif` - else-if branch.
    Elif,
    /// `else` - else branch.
    Else,
    /// `while` - while loop.
    While,
    /// `for` - for loop.
    For,
    /// `in` - range iteration.
    In,
    /// `to` - ascending range.
    To,
    /// `downto` - descending range.
    Downto,
    /// `break` - exit loop.
    Break,
    /// `continue` - skip to next iteration.
    Continue,
    /// `return` - return from function.
    Return,
    /// `pass` - no-operation placeholder.
    Pass,

    // Logical keywords
    /// `and` - logical AND.
    And,
    /// `or` - logical OR.
    Or,
    /// `not` - logical NOT.
    Not,

    // Boolean literals
    /// `true` - boolean true value.
    True,
    /// `false` - boolean false value.
    False,

    // Arithmetic operators
    /// `+` - addition.
    Plus,
    /// `-` - subtraction.
    Minus,
    /// `*` - multiplication.
    Star,
    /// `/` - division.
    Slash,
    /// `%` - modulo.
    Percent,

    // Comparison operators
    /// `==` - equal.
    EqualEqual,
    /// `!=` - not equal.
    BangEqual,
    /// `<` - less than.
    Less,
    /// `>` - greater than.
    Greater,
    /// `<=` - less or equal.
    LessEqual,
    /// `>=` - greater or equal.
    GreaterEqual,

    // Bitwise operators
    /// `&` - bitwise AND.
    Ampersand,
    /// `|` - bitwise OR.
    Pipe,
    /// `^` - bitwise XOR.
    Caret,
    /// `~` - bitwise NOT.
    Tilde,
    /// `<<` - left shift.
    ShiftLeft,
    /// `>>` - right shift.
    ShiftRight,

    // Assignment operators
    /// `=` - assignment.
    Equal,
    /// `+=` - add assign.
    PlusAssign,
    /// `-=` - subtract assign.
    MinusAssign,
    /// `*=` - multiply assign.
    StarAssign,
    /// `/=` - divide assign.
    SlashAssign,
    /// `%=` - modulo assign.
    PercentAssign,
    /// `&=` - AND assign.
    AmpersandAssign,
    /// `|=` - OR assign.
    PipeAssign,
    /// `^=` - XOR assign.
    CaretAssign,
    /// `<<=` - left shift assign.
    ShiftLeftAssign,
    /// `>>=` - right shift assign.
    ShiftRightAssign,

    // Punctuation
    /// `(` - left parenthesis.
    LeftParen,
    /// `)` - right parenthesis.
    RightParen,
    /// `[` - left bracket.
    LeftBracket,
    /// `]` - right bracket.
    RightBracket,
    /// `:` - colon.
    Colon,
    /// `,` - comma.
    Comma,
    /// `->` - arrow (for return type).
    Arrow,

    // Indentation tokens
    /// Increased indentation level.
    Indent,
    /// Decreased indentation level.
    Dedent,
    /// End of line.
    Newline,
}

impl Token {
    /// Check if this token is a type keyword.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Token::Byte
                | Token::Word
                | Token::Sbyte
                | Token::Sword
                | Token::Bool
                | Token::StringType
                | Token::Fixed
                | Token::Float
        )
    }

    /// Check if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Byte
                | Token::Word
                | Token::Sbyte
                | Token::Sword
                | Token::Bool
                | Token::StringType
                | Token::Fixed
                | Token::Float
                | Token::Def
                | Token::If
                | Token::Elif
                | Token::Else
                | Token::While
                | Token::For
                | Token::In
                | Token::To
                | Token::Downto
                | Token::Break
                | Token::Continue
                | Token::Return
                | Token::Pass
                | Token::And
                | Token::Or
                | Token::Not
                | Token::True
                | Token::False
        )
    }

    /// Check if this token is a comparison operator.
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Token::EqualEqual
                | Token::BangEqual
                | Token::Less
                | Token::Greater
                | Token::LessEqual
                | Token::GreaterEqual
        )
    }

    /// Check if this token is an assignment operator.
    pub fn is_assignment(&self) -> bool {
        matches!(
            self,
            Token::Equal
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::StarAssign
                | Token::SlashAssign
                | Token::PercentAssign
                | Token::AmpersandAssign
                | Token::PipeAssign
                | Token::CaretAssign
                | Token::ShiftLeftAssign
                | Token::ShiftRightAssign
        )
    }

    /// Convert a keyword string to a token, or return an identifier.
    pub fn from_keyword_or_identifier(s: &str) -> Token {
        match s {
            // Type keywords
            "byte" => Token::Byte,
            "word" => Token::Word,
            "sbyte" => Token::Sbyte,
            "sword" => Token::Sword,
            "bool" => Token::Bool,
            "string" => Token::StringType,
            "fixed" => Token::Fixed,
            "float" => Token::Float,

            // Definition keywords
            "def" => Token::Def,

            // Control flow keywords
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "to" => Token::To,
            "downto" => Token::Downto,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "pass" => Token::Pass,

            // Logical keywords
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,

            // Boolean literals
            "true" => Token::True,
            "false" => Token::False,

            // Not a keyword, return as identifier
            _ => Token::Identifier(s.to_string()),
        }
    }

    /// Get a human-readable name for this token type.
    pub fn name(&self) -> &'static str {
        match self {
            Token::Integer(_) => "integer",
            Token::Decimal(_) => "decimal",
            Token::String(_) => "string",
            Token::Char(_) => "character",
            Token::Identifier(_) => "identifier",
            Token::Byte => "'byte'",
            Token::Word => "'word'",
            Token::Sbyte => "'sbyte'",
            Token::Sword => "'sword'",
            Token::Bool => "'bool'",
            Token::StringType => "'string'",
            Token::Fixed => "'fixed'",
            Token::Float => "'float'",
            Token::Def => "'def'",
            Token::If => "'if'",
            Token::Elif => "'elif'",
            Token::Else => "'else'",
            Token::While => "'while'",
            Token::For => "'for'",
            Token::In => "'in'",
            Token::To => "'to'",
            Token::Downto => "'downto'",
            Token::Break => "'break'",
            Token::Continue => "'continue'",
            Token::Return => "'return'",
            Token::Pass => "'pass'",
            Token::And => "'and'",
            Token::Or => "'or'",
            Token::Not => "'not'",
            Token::True => "'true'",
            Token::False => "'false'",
            Token::Plus => "'+'",
            Token::Minus => "'-'",
            Token::Star => "'*'",
            Token::Slash => "'/'",
            Token::Percent => "'%'",
            Token::EqualEqual => "'=='",
            Token::BangEqual => "'!='",
            Token::Less => "'<'",
            Token::Greater => "'>'",
            Token::LessEqual => "'<='",
            Token::GreaterEqual => "'>='",
            Token::Ampersand => "'&'",
            Token::Pipe => "'|'",
            Token::Caret => "'^'",
            Token::Tilde => "'~'",
            Token::ShiftLeft => "'<<'",
            Token::ShiftRight => "'>>'",
            Token::Equal => "'='",
            Token::PlusAssign => "'+='",
            Token::MinusAssign => "'-='",
            Token::StarAssign => "'*='",
            Token::SlashAssign => "'/='",
            Token::PercentAssign => "'%='",
            Token::AmpersandAssign => "'&='",
            Token::PipeAssign => "'|='",
            Token::CaretAssign => "'^='",
            Token::ShiftLeftAssign => "'<<='",
            Token::ShiftRightAssign => "'>>='",
            Token::LeftParen => "'('",
            Token::RightParen => "')'",
            Token::LeftBracket => "'['",
            Token::RightBracket => "']'",
            Token::Colon => "':'",
            Token::Comma => "','",
            Token::Arrow => "'->'",
            Token::Indent => "INDENT",
            Token::Dedent => "DEDENT",
            Token::Newline => "NEWLINE",
        }
    }
}

/// Result of identifier naming validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierKind {
    /// A constant (first letter uppercase, all letters uppercase).
    Constant,
    /// A variable (first letter lowercase).
    Variable,
}

/// Validate an identifier name and determine if it's a constant or variable.
///
/// Rules:
/// - Must start with `_` or letter (a-z, A-Z)
/// - After that: `_`, letters (a-z, A-Z), digits (0-9)
/// - Only `_` is invalid
/// - If first letter is uppercase, ALL letters must be uppercase (constant)
/// - If first letter is lowercase, it's a variable
///
/// Returns `Ok(IdentifierKind)` or `Err(error_message)`.
pub fn validate_identifier_naming(name: &str) -> Result<IdentifierKind, &'static str> {
    if name.is_empty() {
        return Err("Identifier cannot be empty");
    }

    // Check if it's only underscores
    if name.chars().all(|c| c == '_') {
        return Err("Identifier cannot consist only of underscores");
    }

    // Find the first letter in the name
    let first_letter = name.chars().find(|c| c.is_ascii_alphabetic());

    match first_letter {
        Some(c) if c.is_ascii_uppercase() => {
            // First letter is uppercase - check that ALL letters are uppercase
            for ch in name.chars() {
                if ch.is_ascii_lowercase() {
                    return Err("Constant names must have all letters in uppercase");
                }
            }
            Ok(IdentifierKind::Constant)
        }
        Some(_) => {
            // First letter is lowercase - it's a variable
            Ok(IdentifierKind::Variable)
        }
        None => {
            // No letters at all (only underscores and digits)
            // This shouldn't happen due to the underscore-only check above,
            // but let's handle it: treat as variable
            Ok(IdentifierKind::Variable)
        }
    }
}

/// Check if an identifier name represents a constant.
pub fn is_constant_name(name: &str) -> bool {
    matches!(
        validate_identifier_naming(name),
        Ok(IdentifierKind::Constant)
    )
}

/// Check if an identifier name represents a variable.
pub fn is_variable_name(name: &str) -> bool {
    matches!(
        validate_identifier_naming(name),
        Ok(IdentifierKind::Variable)
    )
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Decimal(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Char(c) => write!(f, "'{}'", c),
            Token::Identifier(s) => write!(f, "{}", s),
            _ => write!(f, "{}", self.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_recognition() {
        assert!(matches!(Token::from_keyword_or_identifier("if"), Token::If));
        assert!(matches!(
            Token::from_keyword_or_identifier("while"),
            Token::While
        ));
        assert!(matches!(
            Token::from_keyword_or_identifier("byte"),
            Token::Byte
        ));
    }

    #[test]
    fn test_identifier_recognition() {
        match Token::from_keyword_or_identifier("foo") {
            Token::Identifier(s) => assert_eq!(s, "foo"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_is_type() {
        assert!(Token::Byte.is_type());
        assert!(Token::Word.is_type());
        assert!(Token::Bool.is_type());
        assert!(Token::Fixed.is_type());
        assert!(Token::Float.is_type());
        assert!(!Token::If.is_type());
    }

    #[test]
    fn test_is_keyword() {
        assert!(Token::If.is_keyword());
        assert!(Token::While.is_keyword());
        assert!(Token::True.is_keyword());
        assert!(Token::Fixed.is_keyword());
        assert!(Token::Float.is_keyword());
        assert!(!Token::Plus.is_keyword());
    }

    #[test]
    fn test_fixed_float_keyword_recognition() {
        assert!(matches!(
            Token::from_keyword_or_identifier("fixed"),
            Token::Fixed
        ));
        assert!(matches!(
            Token::from_keyword_or_identifier("float"),
            Token::Float
        ));
    }

    #[test]
    fn test_decimal_token_display() {
        let token = Token::Decimal("3.14".to_string());
        assert_eq!(format!("{}", token), "3.14");
    }

    #[test]
    fn test_decimal_token_name() {
        let token = Token::Decimal("3.14".to_string());
        assert_eq!(token.name(), "decimal");
    }

    // ========================================
    // Identifier Naming Validation Tests
    // ========================================

    #[test]
    fn test_validate_constant_names() {
        // Valid constants
        assert_eq!(
            validate_identifier_naming("MY_CONST"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("_MY_CONST"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("_3MY_CONST"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("A"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("B2"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("C_3"),
            Ok(IdentifierKind::Constant)
        );
        assert_eq!(
            validate_identifier_naming("_4D"),
            Ok(IdentifierKind::Constant)
        );
    }

    #[test]
    fn test_validate_variable_names() {
        // Valid variables
        assert_eq!(
            validate_identifier_naming("myVar"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("myVar_5"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("a"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("b2"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("c_3"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("_4d"),
            Ok(IdentifierKind::Variable)
        );
        assert_eq!(
            validate_identifier_naming("_e666"),
            Ok(IdentifierKind::Variable)
        );
    }

    #[test]
    fn test_validate_invalid_mixed_case() {
        // Invalid: first letter uppercase but mixed case
        assert!(validate_identifier_naming("MyConst").is_err());
        assert!(validate_identifier_naming("_My_Const").is_err());
        assert!(validate_identifier_naming("_3My_Const").is_err());
        assert!(validate_identifier_naming("MY_Const").is_err());
    }

    #[test]
    fn test_validate_underscore_only() {
        // Invalid: only underscores
        assert!(validate_identifier_naming("_").is_err());
        assert!(validate_identifier_naming("__").is_err());
        assert!(validate_identifier_naming("___").is_err());
    }

    #[test]
    fn test_is_constant_name_helper() {
        assert!(is_constant_name("MY_CONST"));
        assert!(is_constant_name("A"));
        assert!(is_constant_name("_4D"));
        assert!(!is_constant_name("myVar"));
        assert!(!is_constant_name("_4d"));
        assert!(!is_constant_name("_"));
    }

    #[test]
    fn test_is_variable_name_helper() {
        assert!(is_variable_name("myVar"));
        assert!(is_variable_name("a"));
        assert!(is_variable_name("_4d"));
        assert!(!is_variable_name("MY_CONST"));
        assert!(!is_variable_name("_4D"));
        assert!(!is_variable_name("_"));
    }
}
