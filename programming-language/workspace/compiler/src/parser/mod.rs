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

//! Parser module for the Cobra64 compiler.
//!
//! This module parses a token stream into an Abstract Syntax Tree (AST).
//! It uses recursive descent parsing with precedence climbing for expressions.
//!
//! # Module Structure
//!
//! - `blocks` - Block and function parsing (BlockParser trait)
//! - `control_flow` - Control flow statement parsing (ControlFlowParser trait)
//! - `data_blocks` - Data block parsing (DataBlockParser trait)
//! - `expressions` - Expression parsing (ExpressionParser trait)
//! - `helpers` - Token stream navigation and error handling (ParserHelpers trait)
//! - `statements` - Statement parsing (StatementParser trait)
//! - `types` - Type parsing (TypeParser trait)

// Submodules
pub mod blocks;
pub mod control_flow;
pub mod data_blocks;
pub mod expressions;
pub mod helpers;
pub mod statements;
pub mod types;

// Internal imports from submodules
use blocks::BlockParser;
use helpers::ParserHelpers;

use crate::ast::Program;
use crate::error::{CompileError, Span};
use crate::lexer::Token;

/// The parser state.
pub struct Parser<'a> {
    /// The token stream to parse.
    pub(crate) tokens: &'a [(Token, Span)],
    /// Current position in the token stream.
    pub(crate) position: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given token stream.
    pub fn new(tokens: &'a [(Token, Span)]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    // ========================================
    // Program Parsing
    // ========================================

    /// Parse the complete program.
    pub fn parse(&mut self) -> Result<Program, CompileError> {
        let mut program = Program::new();

        self.skip_newlines();

        while !self.is_at_end() {
            let item = self.parse_top_level_item()?;
            program.add_item(item);
            self.skip_newlines();
        }

        Ok(program)
    }
}

/// Parse a token stream into a program AST.
pub fn parse(tokens: &[(Token, Span)]) -> Result<Program, CompileError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        AssignOp, AssignTarget, BinaryOp, ExprKind, StatementKind, TopLevelItem, Type, UnaryOp,
    };
    use crate::error::ErrorCode;
    use crate::lexer::tokenize;

    /// Helper to parse source code directly.
    fn parse_source(source: &str) -> Result<Program, CompileError> {
        let tokens = tokenize(source)?;
        parse(&tokens)
    }

    // ========================================
    // Parser Creation Tests
    // ========================================

    #[test]
    fn test_parser_creation() {
        let tokens = vec![];
        let parser = Parser::new(&tokens);
        assert!(parser.is_at_end());
    }

    #[test]
    fn test_parser_peek() {
        let tokens = vec![
            (Token::Integer(42), Span::new(0, 2)),
            (Token::Plus, Span::new(3, 4)),
        ];
        let parser = Parser::new(&tokens);
        assert_eq!(parser.peek(), Some(&Token::Integer(42)));
    }

    #[test]
    fn test_parser_advance() {
        let tokens = vec![
            (Token::Integer(42), Span::new(0, 2)),
            (Token::Plus, Span::new(3, 4)),
        ];
        let mut parser = Parser::new(&tokens);
        let first = parser.advance();
        assert!(matches!(first, Some((Token::Integer(42), _))));
        assert_eq!(parser.peek(), Some(&Token::Plus));
    }

    // ========================================
    // Expression Parsing Tests
    // ========================================

    #[test]
    fn test_parse_integer_literal() {
        let program = parse_source("def main():\n    42").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.body.statements.len(), 1);
    }

    #[test]
    fn test_parse_binary_expression() {
        let program = parse_source("def main():\n    1 + 2").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(expr.kind, ExprKind::BinaryOp { .. }));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let program = parse_source("def main():\n    1 + 2 * 3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            // Should parse as 1 + (2 * 3) due to precedence
            if let ExprKind::BinaryOp { op, right, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(
                    right.kind,
                    ExprKind::BinaryOp {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_unary_expression() {
        let program = parse_source("def main():\n    -42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_logical_expression() {
        let program = parse_source("def main():\n    true and false").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::And,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_comparison_expression() {
        let program = parse_source("def main():\n    x == 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::Equal,
                    ..
                }
            ));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let program = parse_source("def main():\n    (1 + 2) * 3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, left, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Mul);
                assert!(matches!(left.kind, ExprKind::Grouped(_)));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_function_call() {
        let program = parse_source("def main():\n    print(42)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::FunctionCall { name, args } = &expr.kind {
                assert_eq!(name, "print");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let program = parse_source("def main():\n    foo(1, 2, 3)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::FunctionCall { name, args } = &expr.kind {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 3);
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_array_index() {
        let program = parse_source("def main():\n    arr[0]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(expr.kind, ExprKind::ArrayIndex { .. }));
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_array_literal_integers() {
        let program = parse_source("def main():\n    x: byte[] = [1, 2, 3]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            if let Some(init) = &decl.initializer {
                if let ExprKind::ArrayLiteral { elements } = &init.kind {
                    assert_eq!(elements.len(), 3);
                } else {
                    panic!("Expected array literal");
                }
            } else {
                panic!("Expected initializer");
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_array_literal_bools() {
        let program = parse_source("def main():\n    flags: bool[] = [true, false, true]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "flags");
            assert_eq!(decl.var_type, Some(Type::BoolArray(None)));
            if let Some(init) = &decl.initializer {
                if let ExprKind::ArrayLiteral { elements } = &init.kind {
                    assert_eq!(elements.len(), 3);
                } else {
                    panic!("Expected array literal");
                }
            } else {
                panic!("Expected initializer");
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_array_literal_empty() {
        let program = parse_source("def main():\n    x: byte[] = []").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::ArrayLiteral { elements } = &init.kind {
                    assert!(elements.is_empty());
                } else {
                    panic!("Expected array literal");
                }
            } else {
                panic!("Expected initializer");
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_array_literal_trailing_comma() {
        let program = parse_source("def main():\n    x: byte[] = [1, 2, 3,]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::ArrayLiteral { elements } = &init.kind {
                    assert_eq!(elements.len(), 3);
                } else {
                    panic!("Expected array literal");
                }
            } else {
                panic!("Expected initializer");
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_array_type_with_size() {
        let program = parse_source("def main():\n    buffer: byte[100]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "buffer");
            assert_eq!(decl.var_type, Some(Type::ByteArray(Some(100))));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_bool_array_type() {
        let program = parse_source("def main():\n    flags: bool[8]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "flags");
            assert_eq!(decl.var_type, Some(Type::BoolArray(Some(8))));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sbyte_array_type() {
        let program = parse_source("def main():\n    temps: sbyte[10]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "temps");
            assert_eq!(decl.var_type, Some(Type::SbyteArray(Some(10))));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_array_type() {
        let program = parse_source("def main():\n    values: sword[5]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "values");
            assert_eq!(decl.var_type, Some(Type::SwordArray(Some(5))));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sbyte_array_unsized() {
        let program = parse_source("def main():\n    arr: sbyte[] = [-10, 0, 10]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "arr");
            assert_eq!(decl.var_type, Some(Type::SbyteArray(None)));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_array_unsized() {
        let program = parse_source("def main():\n    arr: sword[] = [-1000, 500]").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "arr");
            assert_eq!(decl.var_type, Some(Type::SwordArray(None)));
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_type_cast() {
        let program = parse_source("def main():\n    byte(256)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Byte);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    // ========================================
    // Statement Parsing Tests
    // ========================================

    #[test]
    fn test_parse_assignment() {
        let program = parse_source("def main():\n    x = 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            assert!(matches!(assign.target, AssignTarget::Variable(ref n) if n == "x"));
            assert_eq!(assign.op, AssignOp::Assign);
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_compound_assignment() {
        let program = parse_source("def main():\n    x += 1").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            assert_eq!(assign.op, AssignOp::AddAssign);
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_var_decl() {
        let program = parse_source("def main():\n    x: byte = 5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Byte));
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_var_decl_no_init() {
        let program = parse_source("def main():\n    x: word").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Word));
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_const_decl() {
        // Constant declarations use 'const' keyword
        let program = parse_source("const MAX: byte = 100\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "MAX");
            assert_eq!(decl.const_type, Some(Type::Byte));
        } else {
            panic!("Expected const decl at top-level");
        }
    }

    #[test]
    fn test_parse_const_decl_in_function() {
        // Constant declarations are allowed inside functions
        let program = parse_source("def main():\n    const LOCAL_MAX: word = 1000").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::ConstDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "LOCAL_MAX");
            assert_eq!(decl.const_type, Some(Type::Word));
        } else {
            panic!("Expected const decl in function body");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let program = parse_source("def main():\n    if true:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert!(if_stmt.else_block.is_none());
            assert!(if_stmt.elif_branches.is_empty());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let source = "def main():\n    if true:\n        pass\n    else:\n        pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert!(if_stmt.else_block.is_some());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_if_elif_else() {
        let source = "def main():\n    if x == 1:\n        pass\n    elif x == 2:\n        pass\n    else:\n        pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(if_stmt) = &main.body.statements[0].kind {
            assert_eq!(if_stmt.elif_branches.len(), 1);
            assert!(if_stmt.else_block.is_some());
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let program = parse_source("def main():\n    while true:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        assert!(matches!(
            main.body.statements[0].kind,
            StatementKind::While(_)
        ));
    }

    #[test]
    fn test_parse_for_statement() {
        let program = parse_source("def main():\n    for i in 0 to 10:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::For(for_stmt) = &main.body.statements[0].kind {
            assert_eq!(for_stmt.variable, "i");
            assert!(!for_stmt.descending);
        } else {
            panic!("Expected for statement");
        }
    }

    #[test]
    fn test_parse_for_downto() {
        let program = parse_source("def main():\n    for i in 10 downto 0:\n        pass").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::For(for_stmt) = &main.body.statements[0].kind {
            assert!(for_stmt.descending);
        } else {
            panic!("Expected for statement");
        }
    }

    #[test]
    fn test_parse_break() {
        let program = parse_source("def main():\n    while true:\n        break").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(while_stmt) = &main.body.statements[0].kind {
            assert!(matches!(
                while_stmt.body.statements[0].kind,
                StatementKind::Break
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    #[test]
    fn test_parse_continue() {
        let program = parse_source("def main():\n    while true:\n        continue").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(while_stmt) = &main.body.statements[0].kind {
            assert!(matches!(
                while_stmt.body.statements[0].kind,
                StatementKind::Continue
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    #[test]
    fn test_parse_return() {
        let program = parse_source("def main():\n    return").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Return(val) = &main.body.statements[0].kind {
            assert!(val.is_none());
        } else {
            panic!("Expected return statement");
        }
    }

    #[test]
    fn test_parse_return_value() {
        let program = parse_source("def main():\n    return 42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Return(val) = &main.body.statements[0].kind {
            assert!(val.is_some());
        } else {
            panic!("Expected return statement");
        }
    }

    #[test]
    fn test_parse_pass() {
        let program = parse_source("def main():\n    pass").unwrap();
        let main = program.main_function().unwrap();
        assert!(matches!(main.body.statements[0].kind, StatementKind::Pass));
    }

    // ========================================
    // Function Definition Tests
    // ========================================

    #[test]
    fn test_parse_function_no_params() {
        let program = parse_source("def main():\n    pass").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.name, "main");
        assert!(main.params.is_empty());
        assert!(main.return_type.is_none());
    }

    #[test]
    fn test_parse_function_with_params() {
        let program = parse_source("def add(a: byte, b: byte):\n    pass").unwrap();
        assert_eq!(program.items.len(), 1);
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.name, "add");
            assert_eq!(func.params.len(), 2);
            assert_eq!(func.params[0].name, "a");
            assert_eq!(func.params[0].param_type, Type::Byte);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let program = parse_source("def get_value() -> byte:\n    return 42").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Byte));
        } else {
            panic!("Expected function");
        }
    }

    // ========================================
    // Top-Level Tests
    // ========================================

    #[test]
    fn test_parse_top_level_const() {
        let program = parse_source("const MAX: byte = 255\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        assert!(matches!(program.items[0], TopLevelItem::Constant(_)));
    }

    #[test]
    fn test_parse_top_level_var() {
        let program = parse_source("counter: word\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        assert!(matches!(program.items[0], TopLevelItem::Variable(_)));
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = "def helper():\n    pass\n\ndef main():\n    helper()";
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    // ========================================
    // Nested Block Tests
    // ========================================

    #[test]
    fn test_parse_nested_if() {
        let source = "def main():\n    if true:\n        if false:\n            pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::If(outer) = &main.body.statements[0].kind {
            assert!(matches!(
                outer.then_block.statements[0].kind,
                StatementKind::If(_)
            ));
        } else {
            panic!("Expected if statement");
        }
    }

    #[test]
    fn test_parse_nested_while() {
        let source = "def main():\n    while true:\n        while false:\n            pass";
        let program = parse_source(source).unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::While(outer) = &main.body.statements[0].kind {
            assert!(matches!(
                outer.body.statements[0].kind,
                StatementKind::While(_)
            ));
        } else {
            panic!("Expected while statement");
        }
    }

    // ========================================
    // Error Cases
    // ========================================

    #[test]
    fn test_parse_error_missing_colon() {
        let result = parse_source("def main()\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_missing_paren() {
        let result = parse_source("def main:\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = parse_source("def main():\n    +++");
        assert!(result.is_err());
    }

    // ========================================
    // Bitwise and Shift Tests
    // ========================================

    #[test]
    fn test_parse_bitwise_and() {
        let program = parse_source("def main():\n    x & y").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::BitAnd,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_bitwise_or() {
        let program = parse_source("def main():\n    x | y").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::BitOr,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_shift_left() {
        let program = parse_source("def main():\n    x << 2").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::BinaryOp {
                    op: BinaryOp::ShiftLeft,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_bitwise_not() {
        let program = parse_source("def main():\n    ~x").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            assert!(matches!(
                expr.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::BitNot,
                    ..
                }
            ));
        } else {
            panic!("Expected expression");
        }
    }

    // ========================================
    // Signed Type and Negative Literal Tests
    // ========================================

    #[test]
    fn test_parse_sbyte_var_decl() {
        let program = parse_source("def main():\n    x: sbyte = -100").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Sbyte));
            assert!(decl.initializer.is_some());
            // The initializer should be a unary negation of 100
            if let Some(init) = &decl.initializer {
                assert!(matches!(
                    init.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_var_decl() {
        let program = parse_source("def main():\n    y: sword = -30000").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Some(Type::Sword));
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sbyte_min_value() {
        // -128 is the minimum value for sbyte
        let program = parse_source("def main():\n    x: sbyte = -128").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Sbyte));
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(128)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_sword_min_value() {
        // -32768 is the minimum value for sword
        let program = parse_source("def main():\n    y: sword = -32768").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Sword));
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(32768)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_negative_hex_literal() {
        // -$7F = -127
        let program = parse_source("def main():\n    x: sbyte = -$7F").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(127)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_negative_binary_literal() {
        // -%01111111 = -127
        let program = parse_source("def main():\n    x: sbyte = -%01111111").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(127)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_const_negative_value() {
        let program = parse_source("const MIN_SBYTE: sbyte = -128\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "MIN_SBYTE");
            assert_eq!(decl.const_type, Some(Type::Sbyte));
            assert!(matches!(
                decl.value.kind,
                ExprKind::UnaryOp {
                    op: UnaryOp::Negate,
                    ..
                }
            ));
        } else {
            panic!("Expected constant");
        }
    }

    #[test]
    fn test_parse_sbyte_function_param() {
        let program = parse_source("def process(val: sbyte):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Sbyte);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_sword_function_return() {
        let program = parse_source("def get_value() -> sword:\n    return -1").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Sword));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_sbyte_type_cast() {
        let program = parse_source("def main():\n    sbyte(-100)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Sbyte);
                // Inner expression is the negated literal
                assert!(matches!(
                    inner.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_in_expression() {
        let program = parse_source("def main():\n    x = 10 + -5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Assignment(assign) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, right, .. } = &assign.value.kind {
                assert_eq!(*op, BinaryOp::Add);
                // Right side should be unary negate
                assert!(matches!(
                    right.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_double_negative() {
        // --42 should parse as negation of negation of 42
        let program = parse_source("def main():\n    --42").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::UnaryOp { op, operand } = &expr.kind {
                assert_eq!(*op, UnaryOp::Negate);
                // Inner should also be a negation
                assert!(matches!(
                    operand.kind,
                    ExprKind::UnaryOp {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
            } else {
                panic!("Expected unary negate");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_zero() {
        let program = parse_source("def main():\n    x: sbyte = -0").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            if let Some(init) = &decl.initializer {
                if let ExprKind::UnaryOp { operand, .. } = &init.kind {
                    assert!(matches!(operand.kind, ExprKind::IntegerLiteral(0)));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_subtraction_vs_negative() {
        // Ensure a - 1 is subtraction, not a followed by -1
        let program = parse_source("def main():\n    x - 1").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { op, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Sub);
            } else {
                panic!("Expected binary subtraction");
            }
        } else {
            panic!("Expected expression");
        }
    }

    // ========================================
    // Fixed-Point and Float Type Tests
    // ========================================

    #[test]
    fn test_parse_fixed_var_decl() {
        let program = parse_source("def main():\n    x: fixed = 3.75").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Fixed));
            assert!(decl.initializer.is_some());
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "3.75"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_float_var_decl() {
        let program = parse_source("def main():\n    y: float = 3.14").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Some(Type::Float));
            assert!(decl.initializer.is_some());
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_var_decl_no_init() {
        let program = parse_source("def main():\n    x: fixed").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Fixed));
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_float_var_decl_no_init() {
        let program = parse_source("def main():\n    y: float").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Some(Type::Float));
            assert!(decl.initializer.is_none());
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_function_param() {
        let program = parse_source("def scale(val: fixed):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Fixed);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_float_function_param() {
        let program = parse_source("def process(val: float):\n    pass").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.params.len(), 1);
            assert_eq!(func.params[0].param_type, Type::Float);
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_fixed_function_return() {
        let program = parse_source("def get_pos() -> fixed:\n    return 0.0").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Fixed));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_float_function_return() {
        let program = parse_source("def compute() -> float:\n    return 3.14").unwrap();
        if let TopLevelItem::Function(func) = &program.items[0] {
            assert_eq!(func.return_type, Some(Type::Float));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_decimal_literal_in_expression() {
        let program = parse_source("def main():\n    3.14 + 2.5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::BinaryOp { left, op, right } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(left.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
                assert!(matches!(right.kind, ExprKind::DecimalLiteral(ref s) if s == "2.5"));
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression statement");
        }
    }

    #[test]
    fn test_parse_scientific_notation() {
        let program = parse_source("def main():\n    x: float = 1.5e3").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Float));
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "1.5e3"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_scientific_notation_negative_exponent() {
        let program = parse_source("def main():\n    x: float = 2.0e-5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Float));
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "2.0e-5"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_fixed_type_cast() {
        let program = parse_source("def main():\n    fixed(100)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Fixed);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_float_type_cast() {
        let program = parse_source("def main():\n    float(42)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast { target_type, .. } = &expr.kind {
                assert_eq!(*target_type, Type::Float);
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_fixed_cast_from_decimal() {
        let program = parse_source("def main():\n    fixed(3.14)").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::Expression(expr) = &main.body.statements[0].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Fixed);
                assert!(matches!(inner.kind, ExprKind::DecimalLiteral(ref s) if s == "3.14"));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_byte_cast_from_fixed() {
        let program = parse_source("def main():\n    x: fixed = 3.5\n    byte(x)").unwrap();
        let main = program.main_function().unwrap();
        // Second statement should be the type cast
        if let StatementKind::Expression(expr) = &main.body.statements[1].kind {
            if let ExprKind::TypeCast {
                target_type,
                expr: inner,
            } = &expr.kind
            {
                assert_eq!(*target_type, Type::Byte);
                assert!(matches!(inner.kind, ExprKind::Identifier(ref name) if name == "x"));
            } else {
                panic!("Expected type cast");
            }
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_negative_decimal_literal() {
        let program = parse_source("def main():\n    x: fixed = -3.5").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Fixed));
            if let Some(init) = &decl.initializer {
                // Should be UnaryOp(Negate, DecimalLiteral("3.5"))
                if let ExprKind::UnaryOp { op, operand } = &init.kind {
                    assert_eq!(*op, UnaryOp::Negate);
                    assert!(matches!(operand.kind, ExprKind::DecimalLiteral(ref s) if s == "3.5"));
                } else {
                    panic!("Expected unary negate");
                }
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_decimal_zero() {
        let program = parse_source("def main():\n    x: float = 0.0").unwrap();
        let main = program.main_function().unwrap();
        if let StatementKind::VarDecl(decl) = &main.body.statements[0].kind {
            assert_eq!(decl.var_type, Some(Type::Float));
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::DecimalLiteral(ref s) if s == "0.0"));
            }
        } else {
            panic!("Expected var decl");
        }
    }

    #[test]
    fn test_parse_top_level_fixed_var() {
        let program = parse_source("position: fixed\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "position");
            assert_eq!(decl.var_type, Some(Type::Fixed));
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_top_level_float_var() {
        let program = parse_source("temperature: float\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "temperature");
            assert_eq!(decl.var_type, Some(Type::Float));
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_mixed_fixed_float_expression() {
        // This tests that the parser can handle mixed types in expressions
        // Actual type checking will happen in the analyzer
        let program =
            parse_source("def main():\n    x: fixed = 1.0\n    y: float = 2.0\n    x + y").unwrap();
        let main = program.main_function().unwrap();
        assert_eq!(main.body.statements.len(), 3);
        if let StatementKind::Expression(expr) = &main.body.statements[2].kind {
            if let ExprKind::BinaryOp { op, .. } = &expr.kind {
                assert_eq!(*op, BinaryOp::Add);
            } else {
                panic!("Expected binary op");
            }
        } else {
            panic!("Expected expression");
        }
    }

    // ========================================
    // Explicit Type Annotation Tests
    // ========================================

    #[test]
    fn test_parse_var_decl_requires_explicit_type() {
        // Variable without explicit type should fail
        let result = parse_source("x = 10\ndef main():\n    pass");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingTypeAnnotation);
    }

    #[test]
    fn test_parse_var_decl_with_explicit_type() {
        // Variable with explicit type should work
        let program = parse_source("x: byte = 10\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "x");
            assert_eq!(decl.var_type, Some(Type::Byte));
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_var_decl_word_with_explicit_type() {
        // Variable with explicit word type
        let program = parse_source("y: word = 1000\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "y");
            assert_eq!(decl.var_type, Some(Type::Word));
            if let Some(init) = &decl.initializer {
                assert!(matches!(init.kind, ExprKind::IntegerLiteral(1000)));
            } else {
                panic!("Expected initializer");
            }
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_var_decl_negative_with_explicit_type() {
        // Variable with negative value and explicit signed type
        let program = parse_source("temp: sbyte = -50\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Variable(decl) = &program.items[0] {
            assert_eq!(decl.name, "temp");
            assert_eq!(decl.var_type, Some(Type::Sbyte));
            assert!(decl.initializer.is_some());
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_parse_const_decl_explicit_type() {
        // Constant with const keyword: const name: type = value
        let program = parse_source("const MAX: word = 255\ndef main():\n    pass").unwrap();
        assert_eq!(program.items.len(), 2);
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "MAX");
            assert_eq!(decl.const_type, Some(Type::Word));
        } else {
            panic!("Expected constant declaration");
        }
    }

    #[test]
    fn test_parse_const_decl_explicit_type_fixed() {
        // Constant with explicit fixed type
        let program = parse_source("const PI: fixed = 3.14\ndef main():\n    pass").unwrap();
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "PI");
            assert_eq!(decl.const_type, Some(Type::Fixed));
        } else {
            panic!("Expected constant");
        }
    }

    #[test]
    fn test_parse_const_decl_requires_explicit_type() {
        // Constant without explicit type should fail
        let result = parse_source("const MIN = 0\ndef main():\n    pass");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::MissingTypeAnnotation);
    }

    #[test]
    fn test_parse_mixed_explicit_type_declarations() {
        // All declarations with explicit types (constants use 'const' keyword)
        let source = r#"
const MAX: word = 255
count: byte = 0
const PI: fixed = 3.14
const E: float = 2.718
def main():
    pass
"#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 5);

        // const MAX: word = 255
        if let TopLevelItem::Constant(decl) = &program.items[0] {
            assert_eq!(decl.name, "MAX");
            assert_eq!(decl.const_type, Some(Type::Word));
        } else {
            panic!("Expected constant MAX");
        }

        // count: byte = 0 (variable)
        if let TopLevelItem::Variable(decl) = &program.items[1] {
            assert_eq!(decl.name, "count");
            assert_eq!(decl.var_type, Some(Type::Byte));
        } else {
            panic!("Expected variable count");
        }

        // const PI: fixed = 3.14
        if let TopLevelItem::Constant(decl) = &program.items[2] {
            assert_eq!(decl.name, "PI");
            assert_eq!(decl.const_type, Some(Type::Fixed));
        } else {
            panic!("Expected constant PI");
        }

        // const E: float = 2.718
        if let TopLevelItem::Constant(decl) = &program.items[3] {
            assert_eq!(decl.name, "E");
            assert_eq!(decl.const_type, Some(Type::Float));
        } else {
            panic!("Expected constant E");
        }
    }

    #[test]
    fn test_parse_var_decl_without_type_fails() {
        // Variable without type annotation should fail
        let result = parse_source("x\ndef main():\n    pass");
        assert!(result.is_err());
    }
}
