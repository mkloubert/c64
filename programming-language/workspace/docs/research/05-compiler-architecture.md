# Compiler Architecture

This document describes the general architecture of compilers and how it applies to building a C64 cross-compiler in Rust.

## Overview

A compiler transforms source code in one language into another (usually machine code).

```
Source Code  →  [Compiler]  →  Target Code
```

For a C64 cross-compiler:

```
New Language  →  [Rust Compiler Binary]  →  6510 Machine Code (PRG/D64)
```

---

## Compiler Phases

### Traditional Three-Phase Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND                                 │
│  ┌──────────┐   ┌──────────┐   ┌──────────────┐                 │
│  │  Lexer   │ → │  Parser  │ → │ Semantic     │                 │
│  │(Scanner) │   │          │   │ Analysis     │                 │
│  └──────────┘   └──────────┘   └──────────────┘                 │
│       ↓              ↓               ↓                          │
│    Tokens          AST         Typed AST                        │
└─────────────────────────────────────────────────────────────────┘
                           ↓
                  Intermediate Representation (IR)
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│                        OPTIMIZER (optional)                     │
│  Constant folding, dead code elimination, etc.                  │
└─────────────────────────────────────────────────────────────────┘
                           ↓
                    Optimized IR
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│                        BACKEND                                  │
│  ┌──────────────────┐   ┌──────────────────┐                    │
│  │ Code Generator   │ → │ Output Generator │                    │
│  └──────────────────┘   └──────────────────┘                    │
│           ↓                       ↓                             │
│    6510 Machine Code        PRG / D64 File                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Frontend Components

### 1. Lexer (Scanner)

Converts source text into tokens.

**Input:** `x = 42 + y`

**Output:**

```
Token::Identifier("x")
Token::Equals
Token::Number(42)
Token::Plus
Token::Identifier("y")
```

### 2. Parser

Builds an Abstract Syntax Tree (AST) from tokens.

**Input:** Token stream

**Output:** AST

```
    Assignment
    ├── Variable: x
    └── BinaryOp: +
        ├── Number: 42
        └── Variable: y
```

### 3. Semantic Analysis

Validates the AST and adds type information.

**Tasks:**

- Symbol table management
- Type checking
- Scope resolution
- Error detection

---

## Intermediate Representation (IR)

A language-independent form that is easier to optimize and translate.

### For 6502/6510

A simple three-address or stack-based IR works well because:

- 6502 has limited registers (A, X, Y)
- Most operations go through accumulator
- Stack operations are common

**Three-Address Code Example:**

```
t1 = 42
t2 = load(y)
t3 = add(t1, t2)
store(x, t3)
```

---

## Backend Components

### 1. Code Generator

Translates IR to 6510 machine code.

**Note:** We do NOT use LLVM or Cranelift. These target modern CPUs (x86, ARM). Instead, we write a custom 6510 code generator that directly emits machine code bytes.

**Example transformation:**

```
IR: x = 42 + y

6510 Machine Code:
    A9 2A        ; LDA #42
    18           ; CLC
    6D xx xx     ; ADC y (absolute address)
    8D xx xx     ; STA x (absolute address)
```

### 2. Register Allocation

Decides which values go in registers vs memory.

**For 6502:**

- A (accumulator): Primary computation
- X, Y: Indexing, loop counters
- Zero page: "Pseudo-registers" for temporary values

### 3. Output Generator

Creates the final binary/file format.

**For C64:**

1. Generate PRG file (2-byte header + machine code)
2. Optionally wrap in D64 disk image

---

## Rust Implementation Tools

### Parser Libraries for Rust

| Library      | Type                   | Best For                 |
| ------------ | ---------------------- | ------------------------ |
| **logos**    | Lexer generator        | Fast tokenization        |
| **lalrpop**  | LR(1) parser generator | Programming languages    |
| **pest**     | PEG parser             | Readable grammars        |
| **nom**      | Parser combinators     | Binary formats, flexible |
| **chumsky**  | Parser combinators     | Good error recovery      |
| Hand-written | Recursive descent      | Full control             |

### Recommended: logos + lalrpop

**logos** for lexing:

```rust
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Number(i64),

    #[token("+")]
    Plus,

    #[token("=")]
    Equals,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}
```

**lalrpop** for parsing:

```rust
// grammar.lalrpop
use crate::ast::*;

grammar;

pub Program: Vec<Statement> = {
    <Statement*>
};

Statement: Statement = {
    <name:Identifier> "=" <expr:Expr> => Statement::Assignment(name, expr),
};

Expr: Expr = {
    <left:Expr> "+" <right:Term> => Expr::BinaryOp(Box::new(left), Op::Add, Box::new(right)),
    Term,
};
```

### Alternative: Hand-Written Parser

For full control and better error messages:

```rust
pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = self.advance();
            let right = self.parse_term()?;
            left = Expr::BinaryOp(Box::new(left), op.into(), Box::new(right));
        }

        Ok(left)
    }
}
```

---

## Suggested Architecture for C64 Compiler (Rust)

### Project Structure

```
c64-compiler/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── lexer/
│   │   ├── mod.rs
│   │   └── tokens.rs        # Token definitions (logos)
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs           # AST node definitions
│   │   └── grammar.lalrpop  # Grammar (if using lalrpop)
│   ├── analyzer/
│   │   ├── mod.rs
│   │   ├── symbols.rs       # Symbol table
│   │   └── types.rs         # Type checking
│   ├── ir/
│   │   ├── mod.rs
│   │   └── instructions.rs  # IR definitions
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── mos6510.rs       # 6510 instruction encoding
│   │   └── generator.rs     # IR to machine code
│   ├── output/
│   │   ├── mod.rs
│   │   ├── prg.rs           # PRG file writer
│   │   └── d64.rs           # D64 disk image writer
│   └── error.rs             # Error types
└── tests/
    └── ...
```

### Pipeline

```
Source File (.c64)
       ↓
   ┌─────────────────┐
   │ Lexer (logos)   │ → Vec<Token>
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ Parser          │ → AST
   │ (lalrpop/hand)  │
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ Analyzer        │ → Typed AST + Symbol Table
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ IR Generator    │ → Vec<IRInstruction>
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ Code Generator  │ → Vec<u8> (machine code)
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ PRG Writer      │ → .prg file
   └─────────────────┘
       ↓
   ┌─────────────────┐
   │ D64 Writer      │ → .d64 file
   └─────────────────┘
```

### Key Rust Types

```rust
// AST nodes
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Identifier(String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(String, Option<Type>, Expr),
    Assignment(String, Expr),
    If(Expr, Block, Option<Block>),
    While(Expr, Block),
    Return(Option<Expr>),
}

// IR instructions
#[derive(Debug, Clone)]
pub enum IRInst {
    LoadImm(Reg, u8),
    Load(Reg, Address),
    Store(Address, Reg),
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Jump(Label),
    JumpIf(Condition, Label),
    Call(Label),
    Return,
}

// 6510 instructions
#[derive(Debug, Clone)]
pub enum Opcode {
    LDA, LDX, LDY,
    STA, STX, STY,
    ADC, SBC,
    AND, ORA, EOR,
    ASL, LSR, ROL, ROR,
    INC, DEC, INX, INY, DEX, DEY,
    CMP, CPX, CPY,
    BEQ, BNE, BCC, BCS, BMI, BPL,
    JMP, JSR, RTS, RTI,
    // ... etc
}
```

---

## 6510 Code Generator (Custom)

Unlike compilers targeting modern CPUs, we do NOT use LLVM or Cranelift. The 6510 is simple enough to generate code directly.

### Opcode Encoding

```rust
impl Opcode {
    pub fn encode(&self, mode: AddressMode, operand: Option<u16>) -> Vec<u8> {
        match (self, mode) {
            (Opcode::LDA, AddressMode::Immediate) => {
                vec![0xA9, operand.unwrap() as u8]
            }
            (Opcode::LDA, AddressMode::Absolute) => {
                let addr = operand.unwrap();
                vec![0xAD, (addr & 0xFF) as u8, (addr >> 8) as u8]
            }
            (Opcode::STA, AddressMode::Absolute) => {
                let addr = operand.unwrap();
                vec![0x8D, (addr & 0xFF) as u8, (addr >> 8) as u8]
            }
            // ... etc
        }
    }
}
```

### PRG File Generation

```rust
pub fn write_prg(path: &Path, load_address: u16, code: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Write 2-byte load address (little-endian)
    file.write_all(&[(load_address & 0xFF) as u8, (load_address >> 8) as u8])?;

    // Write machine code
    file.write_all(code)?;

    Ok(())
}
```

---

## Rust Crate Dependencies

```toml
[dependencies]
logos = "0.14"           # Lexer generator
lalrpop-util = "0.20"    # Parser runtime (if using lalrpop)
thiserror = "1.0"        # Error handling
clap = { version = "4", features = ["derive"] }  # CLI

[build-dependencies]
lalrpop = "0.20"         # Parser generator (if using lalrpop)

[dev-dependencies]
pretty_assertions = "1.4"
```

---

## Optimization Opportunities

### Compiler Optimizations

| Optimization          | Description                      |
| --------------------- | -------------------------------- |
| Constant folding      | `2 + 3` → `5` at compile time    |
| Dead code elimination | Remove unreachable code          |
| Common subexpression  | Reuse computed values            |
| Strength reduction    | `x * 2` → `x << 1`               |
| Inlining              | Replace function calls with body |

### 6502-Specific Optimizations

| Optimization           | Description                      |
| ---------------------- | -------------------------------- |
| Zero-page allocation   | Put frequently used vars in ZP   |
| Branch optimization    | Use short branches when possible |
| Peephole optimization  | Replace instruction sequences    |
| Tail call optimization | JMP instead of JSR+RTS           |

---

## Why Rust?

| Advantage          | Description                       |
| ------------------ | --------------------------------- |
| **Performance**    | Native binary, fast compilation   |
| **Safety**         | Memory safety without GC          |
| **Tooling**        | Cargo, rustfmt, clippy            |
| **Error handling** | Result/Option types               |
| **Cross-platform** | Compiles to Linux, Windows, macOS |
| **Ecosystem**      | Good parsing libraries available  |

---

## Sources

- [LALRPOP GitHub](https://github.com/lalrpop/lalrpop)
- [Logos - Lexer Generator](https://docs.rs/logos/latest/logos/)
- [Pest PEG Parser](https://pest.rs/)
- [Building a Rust Parser using Pest](https://blog.logrocket.com/building-rust-parser-pest-peg/)
- [Parsing Strategies in Rust](https://willcrichton.net/notes/parsing-strategies-in-rust/)
- [Rust Langdev Libraries](https://github.com/Kixiron/rust-langdev)
