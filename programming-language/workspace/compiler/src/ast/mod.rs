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

//! Abstract Syntax Tree (AST) definitions for the Cobra64 compiler.
//!
//! This module defines the data structures that represent a parsed Cobra64 program.

mod expr;
mod stmt;
mod types;

pub use expr::*;
pub use stmt::*;
pub use types::*;

use crate::error::Span;

/// A complete Cobra64 program.
#[derive(Debug, Clone)]
pub struct Program {
    /// Top-level statements (constants, variables, functions).
    pub items: Vec<TopLevelItem>,
}

impl Program {
    /// Create a new empty program.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a top-level item to the program.
    pub fn add_item(&mut self, item: TopLevelItem) {
        self.items.push(item);
    }

    /// Find the main function in the program.
    pub fn main_function(&self) -> Option<&FunctionDef> {
        for item in &self.items {
            if let TopLevelItem::Function(func) = item {
                if func.name == "main" {
                    return Some(func);
                }
            }
        }
        None
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

/// A top-level item in a program.
#[derive(Debug, Clone)]
pub enum TopLevelItem {
    /// A constant declaration.
    Constant(ConstDecl),
    /// A variable declaration.
    Variable(VarDecl),
    /// A function definition.
    Function(FunctionDef),
    /// A data block definition.
    DataBlock(DataBlock),
}

/// A block of statements.
#[derive(Debug, Clone)]
pub struct Block {
    /// The statements in this block.
    pub statements: Vec<Statement>,
    /// The source span of this block.
    pub span: Span,
}

impl Block {
    /// Create a new block.
    pub fn new(statements: Vec<Statement>, span: Span) -> Self {
        Self { statements, span }
    }

    /// Create an empty block.
    pub fn empty(span: Span) -> Self {
        Self {
            statements: Vec::new(),
            span,
        }
    }

    /// Check if this block is empty.
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for TopLevelItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelItem::Constant(decl) => write!(f, "{}", decl),
            TopLevelItem::Variable(decl) => write!(f, "{}", decl),
            TopLevelItem::Function(func) => write!(f, "{}", func),
            TopLevelItem::DataBlock(data_block) => write!(f, "{}", data_block),
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.statements.is_empty() {
            write!(f, "pass")
        } else {
            for (i, stmt) in self.statements.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                write!(f, "    {}", stmt)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let program = Program::new();
        assert!(program.items.is_empty());
        assert!(program.main_function().is_none());
    }

    #[test]
    fn test_block_creation() {
        let span = Span::new(0, 10);
        let block = Block::empty(span.clone());
        assert!(block.is_empty());
        assert_eq!(block.span.start, 0);
    }

    #[test]
    fn test_program_add_item() {
        let mut program = Program::new();
        let span = Span::new(0, 30);
        let body = Block::empty(Span::new(10, 30));
        let func = FunctionDef::new("main".to_string(), vec![], body, span);
        program.add_item(TopLevelItem::Function(func));
        assert_eq!(program.items.len(), 1);
        assert!(program.main_function().is_some());
    }

    #[test]
    fn test_program_main_function() {
        let mut program = Program::new();

        // Add a non-main function
        let span = Span::new(0, 30);
        let body = Block::empty(Span::new(10, 30));
        let func = FunctionDef::new("helper".to_string(), vec![], body, span);
        program.add_item(TopLevelItem::Function(func));
        assert!(program.main_function().is_none());

        // Add main function
        let span = Span::new(30, 60);
        let body = Block::empty(Span::new(40, 60));
        let main_func = FunctionDef::new("main".to_string(), vec![], body, span);
        program.add_item(TopLevelItem::Function(main_func));
        assert!(program.main_function().is_some());
        assert_eq!(program.main_function().unwrap().name, "main");
    }

    #[test]
    fn test_block_with_statements() {
        let span = Span::new(0, 20);
        let stmt = Statement::new(StatementKind::Pass, Span::new(4, 8));
        let block = Block::new(vec![stmt], span);
        assert!(!block.is_empty());
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn test_display_empty_block() {
        let block = Block::empty(Span::new(0, 10));
        assert_eq!(format!("{}", block), "pass");
    }

    #[test]
    fn test_display_block_with_statements() {
        let stmt1 = Statement::new(StatementKind::Pass, Span::new(4, 8));
        let stmt2 = Statement::new(StatementKind::Break, Span::new(12, 17));
        let block = Block::new(vec![stmt1, stmt2], Span::new(0, 20));
        assert_eq!(format!("{}", block), "    pass\n    break");
    }

    #[test]
    fn test_display_top_level_function() {
        let span = Span::new(0, 30);
        let body = Block::empty(Span::new(10, 30));
        let func = FunctionDef::new("main".to_string(), vec![], body, span);
        let item = TopLevelItem::Function(func);
        assert_eq!(format!("{}", item), "def main():");
    }

    #[test]
    #[allow(deprecated)]
    fn test_display_top_level_constant() {
        let span = Span::new(0, 15);
        let value = Expr::new(ExprKind::IntegerLiteral(100), Span::new(12, 15));
        let decl = ConstDecl::new("MAX".to_string(), value, span);
        let item = TopLevelItem::Constant(decl);
        assert_eq!(format!("{}", item), "const MAX = 100");
    }

    #[test]
    fn test_display_top_level_variable() {
        let span = Span::new(0, 12);
        let decl = VarDecl::new("counter".to_string(), Type::Word, span);
        let item = TopLevelItem::Variable(decl);
        assert_eq!(format!("{}", item), "counter: word");
    }

    #[test]
    fn test_display_program() {
        let mut program = Program::new();

        // Add a constant
        let const_value = Expr::new(ExprKind::IntegerLiteral(255), Span::new(12, 15));
        let const_decl =
            ConstDecl::new_typed("MAX".to_string(), Type::Byte, const_value, Span::new(0, 15));
        program.add_item(TopLevelItem::Constant(const_decl));

        // Add a function
        let body = Block::empty(Span::new(30, 50));
        let func = FunctionDef::new("main".to_string(), vec![], body, Span::new(20, 50));
        program.add_item(TopLevelItem::Function(func));

        let output = format!("{}", program);
        assert!(output.contains("const MAX: byte = 255"));
        assert!(output.contains("def main():"));
    }
}
