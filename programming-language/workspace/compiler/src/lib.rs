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

//! Cobra64 Compiler Library
//!
//! This library provides all the components needed to compile Cobra64 source code
//! into executable programs for the Commodore 64.
//!
//! # Modules
//!
//! - [`error`] - Error types and error reporting
//! - [`lexer`] - Tokenization of source code
//! - [`parser`] - Parsing tokens into an AST
//! - [`ast`] - Abstract Syntax Tree definitions
//! - [`analyzer`] - Semantic analysis and type checking
//! - [`codegen`] - 6510 machine code generation
//! - [`output`] - PRG and D64 file writing
//!
//! # Example
//!
//! ```no_run
//! use cobra64::{lexer, parser, analyzer, codegen, output};
//! use std::path::Path;
//!
//! fn compile(source: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
//!     // Tokenize
//!     let tokens = lexer::tokenize(source)?;
//!
//!     // Parse
//!     let ast = parser::parse(&tokens)?;
//!
//!     // Analyze
//!     analyzer::analyze(&ast).map_err(|e| e.into_iter().next().unwrap())?;
//!
//!     // Generate code
//!     let code = codegen::generate(&ast)?;
//!
//!     // Write output
//!     output::write_prg(&code, output_path)?;
//!
//!     Ok(())
//! }
//! ```

pub mod analyzer;
pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod output;
pub mod parser;

// Re-export commonly used types
pub use ast::{Program, Type};
pub use error::{format_error, CompileError, ErrorCode, Result, SourceLocation, Span};
pub use lexer::Token;

/// The version of the Cobra64 compiler.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the compiler.
pub const NAME: &str = "Cobra64";

/// Compile source code to machine code.
///
/// This is the main entry point for compiling Cobra64 source code.
/// It performs all compilation stages: lexing, parsing, analysis, and code generation.
///
/// # Arguments
///
/// * `source` - The source code to compile
///
/// # Returns
///
/// Returns the generated machine code as a byte vector, or an error if compilation fails.
///
/// # Example
///
/// ```no_run
/// let source = r#"
/// def main():
///     println("Hello, World!")
/// "#;
///
/// match cobra64::compile(source) {
///     Ok(code) => println!("Generated {} bytes of code", code.len()),
///     Err(e) => eprintln!("Compilation error: {}", e),
/// }
/// ```
pub fn compile(source: &str) -> std::result::Result<Vec<u8>, CompileError> {
    // Tokenize
    let tokens = lexer::tokenize(source)?;

    // Parse
    let ast = parser::parse(&tokens)?;

    // Analyze
    analyzer::analyze(&ast)
        .map_err(|errors| errors.into_iter().next().expect("At least one error"))?;

    // Generate code
    codegen::generate(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "Cobra64");
    }
}
