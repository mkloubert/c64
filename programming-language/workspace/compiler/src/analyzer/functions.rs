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

//! Function analysis for the semantic analyzer.
//!
//! This module provides function-related analysis:
//! - Function definition analysis
//! - Function call analysis
//! - Return statement handling
//! - Built-in function support (len)

use super::expressions::ExpressionAnalyzer;
use super::symbol::{Symbol, SymbolType};
use super::type_check::TypeChecker;
use super::Analyzer;
use crate::ast::{Block, Expr, FunctionDef, StatementKind, Type};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for function analysis.
pub trait FunctionAnalyzer {
    /// Analyze a function definition.
    fn analyze_function(&mut self, func: &FunctionDef);

    /// Check if a block definitely returns a value.
    fn block_has_return(&self, block: &Block) -> bool;

    /// Analyze a function call.
    fn analyze_function_call(&mut self, name: &str, args: &[Expr], span: &Span) -> Option<Type>;

    /// Analyze the built-in len() function call.
    fn analyze_len_call(&mut self, args: &[Expr], span: &Span) -> Option<Type>;
}

impl FunctionAnalyzer for Analyzer {
    fn analyze_function(&mut self, func: &FunctionDef) {
        // Set up context for function body
        let old_context = self.context.clone();
        self.context.in_function = true;
        self.context.return_type = func.return_type.clone();
        self.context.function_name = Some(func.name.clone());

        // Create new scope for function body
        self.symbols.push_scope();

        // Add parameters to scope
        for param in &func.params {
            let symbol = Symbol::variable(
                param.name.clone(),
                param.param_type.clone(),
                false,
                param.span.clone(),
            );
            if let Err(existing) = self.symbols.define(symbol) {
                self.error(
                    CompileError::new(
                        ErrorCode::DuplicateParameterName,
                        format!("Duplicate parameter name '{}'", param.name),
                        param.span.clone(),
                    )
                    .with_hint(format!(
                        "Previously defined at position {}",
                        existing.span.start
                    )),
                );
            }
        }

        // Analyze function body
        self.analyze_block(&func.body);

        // Check for missing return statement in functions with return type
        if func.return_type.is_some() && !self.block_has_return(&func.body) {
            self.error(CompileError::new(
                ErrorCode::MissingReturnStatement,
                format!(
                    "Function '{}' may not return a value on all paths",
                    func.name
                ),
                func.span.clone(),
            ));
        }

        // Pop function scope
        self.symbols.pop_scope();

        // Restore context
        self.context = old_context;
    }

    fn block_has_return(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            match &stmt.kind {
                StatementKind::Return(Some(_)) => return true,
                StatementKind::If(if_stmt) => {
                    // If-else must return on both branches
                    if let Some(else_block) = &if_stmt.else_block {
                        let then_returns = self.block_has_return(&if_stmt.then_block);
                        let else_returns = self.block_has_return(else_block);
                        if then_returns && else_returns {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn analyze_function_call(&mut self, name: &str, args: &[Expr], span: &Span) -> Option<Type> {
        // Handle built-in len() function specially
        if name == "len" {
            return self.analyze_len_call(args, span);
        }

        let symbol = self.symbols.lookup(name).cloned();

        match symbol {
            Some(Symbol {
                symbol_type:
                    SymbolType::Function {
                        params,
                        return_type,
                    },
                ..
            }) => {
                // Check argument count
                if args.len() != params.len() {
                    self.error(CompileError::new(
                        ErrorCode::WrongNumberOfArguments,
                        format!(
                            "Function '{}' expects {} arguments, but {} were provided",
                            name,
                            params.len(),
                            args.len()
                        ),
                        span.clone(),
                    ));
                }

                // Check argument types
                for (i, (arg, expected_type)) in args.iter().zip(params.iter()).enumerate() {
                    let arg_type = self.analyze_expression(arg);
                    if let Some(arg_type) = arg_type {
                        // Allow string literals for print functions (implicit conversion)
                        let compatible = if *expected_type == Type::String {
                            arg_type == Type::String
                                || arg_type.is_integer()
                                || arg_type == Type::Bool
                                || arg_type.is_fixed()
                                || arg_type.is_float()
                        } else {
                            // Use expr_assignable_to for DecimalLiteral handling
                            self.is_expr_assignable_to(&arg.kind, &arg_type, expected_type)
                        };

                        if !compatible {
                            self.error(CompileError::new(
                                ErrorCode::ArgumentTypeMismatch,
                                format!(
                                    "Argument {} of '{}' expects {}, found {}",
                                    i + 1,
                                    name,
                                    expected_type,
                                    arg_type
                                ),
                                arg.span.clone(),
                            ));
                        }
                    }
                }

                return_type
            }
            Some(_) => {
                self.error(CompileError::new(
                    ErrorCode::InvalidFunctionCall,
                    format!("'{}' is not a function", name),
                    span.clone(),
                ));
                None
            }
            None => {
                self.error(CompileError::new(
                    ErrorCode::UndefinedFunction,
                    format!("Undefined function '{}'", name),
                    span.clone(),
                ));
                None
            }
        }
    }

    fn analyze_len_call(&mut self, args: &[Expr], span: &Span) -> Option<Type> {
        // len() requires exactly one argument
        if args.len() != 1 {
            self.error(CompileError::new(
                ErrorCode::WrongNumberOfArguments,
                format!(
                    "Function 'len' expects 1 argument, but {} were provided",
                    args.len()
                ),
                span.clone(),
            ));
            return None;
        }

        // Analyze the argument and check it's an array type
        let arg_type = self.analyze_expression(&args[0])?;

        if !arg_type.is_array() {
            self.error(CompileError::new(
                ErrorCode::TypeMismatch,
                format!(
                    "Function 'len' expects an array argument, found {}",
                    arg_type
                ),
                span.clone(),
            ));
            return None;
        }

        // Return type is word (16-bit to support arrays up to 65535 elements)
        Some(Type::Word)
    }
}
