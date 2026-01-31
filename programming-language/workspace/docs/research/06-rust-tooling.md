# Rust Tooling for Compiler Development

This document describes Rust libraries and tools suitable for building the C64 compiler.

## Lexer Libraries

### logos (Recommended)

Fast lexer generator using procedural macros.

| Property        | Value                                  |
| --------------- | -------------------------------------- |
| **Crate**       | `logos`                                |
| **Version**     | 0.14.x                                 |
| **Type**        | Derive macro                           |
| **Performance** | Very fast (comparable to hand-written) |

**Example:**

```rust
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\r]+")]  // Skip whitespace
pub enum Token {
    // Keywords
    #[token("var")]
    Var,

    #[token("if")]
    If,

    #[token("while")]
    While,

    #[token("function")]
    Function,

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Number(i64),

    #[regex(r"\$[0-9a-fA-F]+", |lex| i64::from_str_radix(&lex.slice()[1..], 16).ok())]
    HexNumber(i64),

    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("=")]
    Equals,

    #[token("==")]
    EqualsEquals,

    #[token("!=")]
    NotEquals,

    // Delimiters
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,
}
```

**Usage:**

```rust
let source = "var x = 42";
let mut lexer = Token::lexer(source);

while let Some(token) = lexer.next() {
    println!("{:?} at {:?}", token, lexer.span());
}
```

---

## Parser Libraries

### lalrpop (Recommended for complex grammars)

LR(1) parser generator with excellent Rust integration.

| Property    | Value                                       |
| ----------- | ------------------------------------------- |
| **Crate**   | `lalrpop` (build), `lalrpop-util` (runtime) |
| **Version** | 0.20.x                                      |
| **Type**    | Build-time code generator                   |
| **Grammar** | LALR(1) / LR(1)                             |

**Cargo.toml:**

```toml
[dependencies]
lalrpop-util = { version = "0.20", features = ["lexer"] }

[build-dependencies]
lalrpop = "0.20"
```

**build.rs:**

```rust
fn main() {
    lalrpop::process_root().unwrap();
}
```

**Grammar file (src/parser/grammar.lalrpop):**

```rust
use crate::ast::*;
use crate::lexer::Token;

grammar<'input>;

extern {
    type Location = usize;
    type Error = ();

    enum Token {
        "var" => Token::Var,
        "if" => Token::If,
        "while" => Token::While,
        "+" => Token::Plus,
        "-" => Token::Minus,
        "=" => Token::Equals,
        "(" => Token::LParen,
        ")" => Token::RParen,
        "{" => Token::LBrace,
        "}" => Token::RBrace,
        ";" => Token::Semicolon,
        "number" => Token::Number(<i64>),
        "ident" => Token::Identifier(<String>),
    }
}

pub Program: Program = {
    <statements:Statement*> => Program { statements }
};

Statement: Statement = {
    "var" <name:"ident"> "=" <expr:Expr> ";" => {
        Statement::VarDecl { name, value: expr }
    },
    <name:"ident"> "=" <expr:Expr> ";" => {
        Statement::Assignment { name, value: expr }
    },
};

Expr: Expr = {
    <left:Expr> "+" <right:Term> => Expr::BinaryOp {
        left: Box::new(left),
        op: BinOp::Add,
        right: Box::new(right),
    },
    <left:Expr> "-" <right:Term> => Expr::BinaryOp {
        left: Box::new(left),
        op: BinOp::Sub,
        right: Box::new(right),
    },
    Term,
};

Term: Expr = {
    <n:"number"> => Expr::Number(n),
    <name:"ident"> => Expr::Identifier(name),
    "(" <Expr> ")",
};
```

### pest (Alternative - PEG-based)

Parser using Parsing Expression Grammars.

| Property    | Value                       |
| ----------- | --------------------------- |
| **Crate**   | `pest`, `pest_derive`       |
| **Version** | 2.7.x                       |
| **Type**    | Derive macro + grammar file |
| **Grammar** | PEG                         |

**Grammar file (grammar.pest):**

```pest
program = { SOI ~ statement* ~ EOI }

statement = { var_decl | assignment }

var_decl = { "var" ~ identifier ~ "=" ~ expr ~ ";" }
assignment = { identifier ~ "=" ~ expr ~ ";" }

expr = { term ~ (("+" | "-") ~ term)* }
term = { number | identifier | "(" ~ expr ~ ")" }

identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
number = @{ ASCII_DIGIT+ }

WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }
```

### Hand-Written Recursive Descent (Maximum Control)

For best error messages and full control:

```rust
pub struct Parser<'a> {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokens: Vec<_> = Token::lexer(source)
            .spanned()
            .filter_map(|(tok, span)| tok.ok().map(|t| (t, span)))
            .collect();

        Self { tokens, pos: 0, source }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).map(|(t, _)| t.clone());
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.current() {
            Some(t) if t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: expected.clone(),
                found: t.clone(),
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while self.current().is_some() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current() {
            Some(Token::Var) => self.parse_var_decl(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Identifier(_)) => self.parse_assignment(),
            _ => Err(ParseError::ExpectedStatement),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while matches!(self.current(), Some(Token::Plus | Token::Minus)) {
            let op = match self.advance() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name)),
            Some(Token::LParen) => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::ExpectedExpression),
        }
    }
}
```

---

## CLI Library

### clap (Recommended)

Command-line argument parser.

| Property     | Value    |
| ------------ | -------- |
| **Crate**    | `clap`   |
| **Version**  | 4.x      |
| **Features** | `derive` |

**Example:**

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "c64c")]
#[command(about = "C64 Compiler - Compile source to PRG/D64")]
struct Cli {
    /// Source file to compile
    #[arg(required = true)]
    input: PathBuf,

    /// Output file (default: input with .prg extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Create D64 disk image
    #[arg(short, long)]
    disk: bool,

    /// Disk name (for D64 output)
    #[arg(long, default_value = "PROGRAM")]
    disk_name: String,

    /// Show generated assembly
    #[arg(long)]
    emit_asm: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Use cli.input, cli.output, etc.
}
```

---

## Error Handling

### thiserror

Derive macro for custom error types.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Lexer error at {location}: {message}")]
    LexerError { location: usize, message: String },

    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### miette (Pretty error reporting)

For user-friendly error messages with source code snippets:

```rust
use miette::{Diagnostic, SourceSpan};

#[derive(Error, Debug, Diagnostic)]
#[error("Undefined variable")]
#[diagnostic(code(c64c::undefined_var), help("Did you mean to declare this with 'var'?"))]
pub struct UndefinedVariable {
    #[source_code]
    pub src: String,

    #[label("this variable is not defined")]
    pub span: SourceSpan,

    pub name: String,
}
```

---

## Testing

### pretty_assertions

Better diff output for test failures.

```rust
#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_parse_expression() {
    let source = "1 + 2 * 3";
    let expected = Expr::BinaryOp { /* ... */ };
    let result = parse(source).unwrap();
    assert_eq!(result, expected);
}
```

### insta (Snapshot testing)

For testing compiler output:

```rust
use insta::assert_snapshot;

#[test]
fn test_codegen() {
    let source = "var x = 42";
    let asm = compile_to_asm(source).unwrap();
    assert_snapshot!(asm);
}
```

---

## Recommended Cargo.toml

```toml
[package]
name = "c64c"
version = "0.1.0"
edition = "2021"
description = "A modern programming language compiler for the Commodore 64"

[dependencies]
logos = "0.14"
lalrpop-util = { version = "0.20", features = ["lexer"] }
clap = { version = "4", features = ["derive"] }
thiserror = "1.0"
miette = { version = "7", features = ["fancy"] }

[build-dependencies]
lalrpop = "0.20"

[dev-dependencies]
pretty_assertions = "1.4"
insta = "1.34"

[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## Project Structure

```
c64c/
├── Cargo.toml
├── build.rs                 # lalrpop build script
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── lexer.rs             # Token definitions (logos)
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs           # AST node types
│   │   └── grammar.lalrpop  # Grammar (if using lalrpop)
│   ├── analyzer/
│   │   ├── mod.rs
│   │   ├── symbols.rs       # Symbol table
│   │   └── types.rs         # Type system
│   ├── ir/
│   │   ├── mod.rs
│   │   └── instructions.rs  # IR types
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── mos6510.rs       # 6510 opcodes
│   │   └── emitter.rs       # Code emission
│   ├── output/
│   │   ├── mod.rs
│   │   ├── prg.rs           # PRG writer
│   │   └── d64.rs           # D64 writer
│   └── error.rs             # Error types
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    └── integration_tests.rs
```

---

## Sources

- [logos Documentation](https://docs.rs/logos/latest/logos/)
- [LALRPOP Book](https://lalrpop.github.io/lalrpop/)
- [pest Book](https://pest.rs/book/)
- [clap Documentation](https://docs.rs/clap/latest/clap/)
- [thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/)
- [miette Documentation](https://docs.rs/miette/latest/miette/)
- [Rust Langdev Libraries](https://github.com/Kixiron/rust-langdev)
