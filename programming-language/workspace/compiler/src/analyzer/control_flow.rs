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

//! Control flow analysis for the semantic analyzer.
//!
//! This module provides control flow statement analysis:
//! - If/elif/else statements
//! - While loops
//! - For loops
//! - Return statements

use super::expressions::ExpressionAnalyzer;
use super::symbol::Symbol;
use super::type_check::TypeChecker;
use super::Analyzer;
use crate::ast::{Expr, ForStatement, IfStatement, Type, WhileStatement};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for control flow analysis.
pub trait ControlFlowAnalyzer {
    /// Analyze an if statement.
    fn analyze_if_statement(&mut self, if_stmt: &IfStatement);

    /// Analyze a while statement.
    fn analyze_while_statement(&mut self, while_stmt: &WhileStatement);

    /// Analyze a for statement.
    fn analyze_for_statement(&mut self, for_stmt: &ForStatement);

    /// Analyze a return statement.
    fn analyze_return_statement(&mut self, value: &Option<Expr>, span: &Span);
}

impl ControlFlowAnalyzer for Analyzer {
    fn analyze_if_statement(&mut self, if_stmt: &IfStatement) {
        // Check condition is boolean
        let cond_type = self.analyze_expression(&if_stmt.condition);
        if let Some(cond_type) = cond_type {
            if cond_type != Type::Bool {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    format!("Condition must be boolean, found {}", cond_type),
                    if_stmt.condition.span.clone(),
                ));
            }
        }

        // Analyze then block
        self.symbols.push_scope();
        self.analyze_block(&if_stmt.then_block);
        self.symbols.pop_scope();

        // Analyze elif branches
        for (elif_cond, elif_block) in &if_stmt.elif_branches {
            let elif_type = self.analyze_expression(elif_cond);
            if let Some(elif_type) = elif_type {
                if elif_type != Type::Bool {
                    self.error(CompileError::new(
                        ErrorCode::TypeMismatch,
                        format!("Condition must be boolean, found {}", elif_type),
                        elif_cond.span.clone(),
                    ));
                }
            }
            self.symbols.push_scope();
            self.analyze_block(elif_block);
            self.symbols.pop_scope();
        }

        // Analyze else block
        if let Some(else_block) = &if_stmt.else_block {
            self.symbols.push_scope();
            self.analyze_block(else_block);
            self.symbols.pop_scope();
        }
    }

    fn analyze_while_statement(&mut self, while_stmt: &WhileStatement) {
        // Check condition is boolean
        let cond_type = self.analyze_expression(&while_stmt.condition);
        if let Some(cond_type) = cond_type {
            if cond_type != Type::Bool {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    format!("Condition must be boolean, found {}", cond_type),
                    while_stmt.condition.span.clone(),
                ));
            }
        }

        // Analyze body with loop context
        let old_in_loop = self.context.in_loop;
        self.context.in_loop = true;
        self.symbols.push_scope();
        self.analyze_block(&while_stmt.body);
        self.symbols.pop_scope();
        self.context.in_loop = old_in_loop;
    }

    fn analyze_for_statement(&mut self, for_stmt: &ForStatement) {
        // Analyze start and end expressions
        let start_type = self.analyze_expression(&for_stmt.start);
        let end_type = self.analyze_expression(&for_stmt.end);

        // Check both are integers
        if let Some(start_type) = &start_type {
            if !start_type.is_integer() {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    format!("Range start must be an integer, found {}", start_type),
                    for_stmt.start.span.clone(),
                ));
            }
        }
        if let Some(end_type) = &end_type {
            if !end_type.is_integer() {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    format!("Range end must be an integer, found {}", end_type),
                    for_stmt.end.span.clone(),
                ));
            }
        }

        // Infer loop variable type
        let loop_var_type = start_type.or(end_type).unwrap_or(Type::Byte);

        // Analyze body with loop variable in scope
        let old_in_loop = self.context.in_loop;
        self.context.in_loop = true;
        self.symbols.push_scope();

        // Add loop variable to scope
        let symbol = Symbol::variable(
            for_stmt.variable.clone(),
            loop_var_type,
            true, // Loop variable is effectively constant within the loop
            for_stmt.span.clone(),
        );
        let _ = self.symbols.define(symbol);

        self.analyze_block(&for_stmt.body);
        self.symbols.pop_scope();
        self.context.in_loop = old_in_loop;
    }

    fn analyze_return_statement(&mut self, value: &Option<Expr>, span: &Span) {
        if !self.context.in_function {
            self.error(CompileError::new(
                ErrorCode::ReturnOutsideFunction,
                "'return' can only be used inside a function",
                span.clone(),
            ));
            return;
        }

        // Clone return_type to avoid borrow checker issues
        let expected_return_type = self.context.return_type.clone();

        match (expected_return_type, value) {
            (Some(expected), Some(expr)) => {
                let actual = self.analyze_expression(expr);
                if let Some(actual) = actual {
                    if !self.is_expr_assignable_to(&expr.kind, &actual, &expected) {
                        self.error(CompileError::new(
                            ErrorCode::TypeMismatch,
                            format!("Expected return type {}, found {}", expected, actual),
                            expr.span.clone(),
                        ));
                    }
                }
            }
            (Some(expected), None) => {
                self.error(CompileError::new(
                    ErrorCode::MissingReturnValue,
                    format!("Expected return value of type {}", expected),
                    span.clone(),
                ));
            }
            (None, Some(expr)) => {
                self.analyze_expression(expr);
                self.error(CompileError::new(
                    ErrorCode::CannotReturnValueFromVoid,
                    "Cannot return a value from a void function",
                    expr.span.clone(),
                ));
            }
            (None, None) => {
                // OK - void function returning nothing
            }
        }
    }
}
