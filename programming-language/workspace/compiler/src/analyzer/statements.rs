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

//! Statement analysis for the semantic analyzer.
//!
//! This module provides statement analysis functionality:
//! - Variable and constant declarations
//! - Assignment statements (simple and compound)
//! - Statement dispatch to appropriate handlers

use super::control_flow::ControlFlowAnalyzer;
use super::expressions::ExpressionAnalyzer;
use super::operators::OperatorChecker;
use super::symbol::Symbol;
use super::type_check::TypeChecker;
use super::Analyzer;
use crate::ast::{
    AssignOp, AssignTarget, Assignment, BinaryOp, ConstDecl, ExprKind, Statement, StatementKind,
    Type, VarDecl,
};
use crate::error::{CompileError, ErrorCode, Span};

/// Extension trait for statement analysis.
pub trait StatementAnalyzer {
    /// Analyze a statement.
    fn analyze_statement(&mut self, stmt: &Statement);

    /// Analyze a variable declaration.
    fn analyze_var_decl(&mut self, decl: &VarDecl);

    /// Analyze a constant declaration.
    fn analyze_const_decl(&mut self, decl: &ConstDecl);

    /// Analyze an assignment statement.
    fn analyze_assignment(&mut self, assign: &Assignment);

    /// Convert an assignment operator to the corresponding binary operator.
    fn assign_op_to_binary_op(&self, op: &AssignOp) -> Option<BinaryOp>;

    /// Analyze an assignment target.
    fn analyze_assign_target(&mut self, target: &AssignTarget, span: &Span) -> Option<Type>;
}

impl StatementAnalyzer for Analyzer {
    fn analyze_statement(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::VarDecl(decl) => {
                self.analyze_var_decl(decl);
            }
            StatementKind::ConstDecl(decl) => {
                self.analyze_const_decl(decl);
            }
            StatementKind::Assignment(assign) => {
                self.analyze_assignment(assign);
            }
            StatementKind::If(if_stmt) => {
                self.analyze_if_statement(if_stmt);
            }
            StatementKind::While(while_stmt) => {
                self.analyze_while_statement(while_stmt);
            }
            StatementKind::For(for_stmt) => {
                self.analyze_for_statement(for_stmt);
            }
            StatementKind::Break => {
                if !self.context.in_loop {
                    self.error(CompileError::new(
                        ErrorCode::BreakOutsideLoop,
                        "'break' can only be used inside a loop",
                        stmt.span.clone(),
                    ));
                }
            }
            StatementKind::Continue => {
                if !self.context.in_loop {
                    self.error(CompileError::new(
                        ErrorCode::ContinueOutsideLoop,
                        "'continue' can only be used inside a loop",
                        stmt.span.clone(),
                    ));
                }
            }
            StatementKind::Return(value) => {
                self.analyze_return_statement(value, &stmt.span);
            }
            StatementKind::Pass => {
                // Nothing to analyze
            }
            StatementKind::Expression(expr) => {
                self.analyze_expression(expr);
            }
        }
    }

    fn analyze_var_decl(&mut self, decl: &VarDecl) {
        // Determine the variable type (explicit or inferred)
        let var_type = if let Some(ref explicit_type) = decl.var_type {
            // Explicit type provided
            explicit_type.clone()
        } else {
            // Type inference - infer from initializer
            if let Some(init) = &decl.initializer {
                let init_type = self.analyze_expression(init);
                if let Some(inferred) = init_type {
                    self.infer_type_from_expr(init, &inferred)
                } else {
                    self.error(CompileError::new(
                        ErrorCode::TypeMismatch,
                        "Cannot infer type from expression",
                        decl.span.clone(),
                    ));
                    Type::Byte // Fallback
                }
            } else {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    "Variable declaration without type requires an initializer for type inference",
                    decl.span.clone(),
                ));
                Type::Byte // Fallback
            }
        };

        // Check initializer type if present
        if let Some(init) = &decl.initializer {
            let init_type = self.analyze_expression(init);
            if let Some(init_type) = init_type {
                if !self.is_expr_assignable_to(&init.kind, &init_type, &var_type) {
                    self.error(CompileError::new(
                        ErrorCode::TypeMismatch,
                        format!(
                            "Cannot assign {} to variable of type {}",
                            init_type, var_type
                        ),
                        init.span.clone(),
                    ));
                }
            }

            // Check compile-time range for integer types
            if var_type.is_integer() {
                if let Some(value) = self.try_eval_constant(init) {
                    self.check_value_in_range(value, &var_type, &init.span);
                }
            }

            // Check array literal size matches declared size
            if var_type.is_array() {
                if let ExprKind::ArrayLiteral { elements } = &init.kind {
                    self.check_array_literal_size(&var_type, elements.len(), &init.span);
                    self.check_array_literal_elements(&var_type, elements);
                }
            }
        }

        // Add to symbol table
        let symbol = Symbol::variable(decl.name.clone(), var_type, false, decl.span.clone());
        if let Err(existing) = self.symbols.define(symbol) {
            self.error(
                CompileError::new(
                    ErrorCode::VariableAlreadyDefined,
                    format!("Variable '{}' is already defined in this scope", decl.name),
                    decl.span.clone(),
                )
                .with_hint(format!(
                    "Previously defined at position {}",
                    existing.span.start
                )),
            );
        }
    }

    fn analyze_const_decl(&mut self, decl: &ConstDecl) {
        // Analyze the value expression
        let value_type = self.analyze_expression(&decl.value);

        // Determine the constant type (explicit or inferred)
        let const_type = if let Some(ref explicit_type) = decl.const_type {
            // Explicit type provided - check compatibility
            if let Some(ref val_type) = value_type {
                if !self.is_expr_assignable_to(&decl.value.kind, val_type, explicit_type) {
                    self.error(CompileError::new(
                        ErrorCode::TypeMismatch,
                        format!(
                            "Cannot assign {} to constant of type {}",
                            val_type, explicit_type
                        ),
                        decl.value.span.clone(),
                    ));
                }
            }
            explicit_type.clone()
        } else {
            // Type inference from value
            if let Some(ref val_type) = value_type {
                self.infer_type_from_expr(&decl.value, val_type)
            } else {
                Type::Word // Fallback
            }
        };

        // Add to symbol table
        let symbol = Symbol::variable(decl.name.clone(), const_type, true, decl.span.clone());
        if let Err(existing) = self.symbols.define(symbol) {
            self.error(
                CompileError::new(
                    ErrorCode::VariableAlreadyDefined,
                    format!("Constant '{}' is already defined in this scope", decl.name),
                    decl.span.clone(),
                )
                .with_hint(format!(
                    "Previously defined at position {}",
                    existing.span.start
                )),
            );
        }
    }

    fn analyze_assignment(&mut self, assign: &Assignment) {
        let target_type = self.analyze_assign_target(&assign.target, &assign.span);
        let value_type = self.analyze_expression(&assign.value);

        // Check for constant assignment
        if let AssignTarget::Variable(name) = &assign.target {
            if let Some(symbol) = self.symbols.lookup(name) {
                if symbol.is_constant {
                    self.error(CompileError::new(
                        ErrorCode::CannotAssignToConstant,
                        format!("Cannot assign to constant '{}'", name),
                        assign.span.clone(),
                    ));
                }
            }
        }

        // Type check assignment
        if let (Some(target_type), Some(value_type)) = (target_type, value_type) {
            // For compound assignment, check operation validity
            if assign.op != AssignOp::Assign {
                let binary_op = self.assign_op_to_binary_op(&assign.op);
                if let Some(binary_op) = binary_op {
                    self.check_binary_op_types(&target_type, &binary_op, &value_type, &assign.span);
                }
            }

            if !self.is_expr_assignable_to(&assign.value.kind, &value_type, &target_type) {
                self.error(CompileError::new(
                    ErrorCode::TypeMismatch,
                    format!("Cannot assign {} to {}", value_type, target_type),
                    assign.value.span.clone(),
                ));
            }
        }
    }

    fn assign_op_to_binary_op(&self, op: &AssignOp) -> Option<BinaryOp> {
        match op {
            AssignOp::Assign => None,
            AssignOp::AddAssign => Some(BinaryOp::Add),
            AssignOp::SubAssign => Some(BinaryOp::Sub),
            AssignOp::MulAssign => Some(BinaryOp::Mul),
            AssignOp::DivAssign => Some(BinaryOp::Div),
            AssignOp::ModAssign => Some(BinaryOp::Mod),
            AssignOp::BitAndAssign => Some(BinaryOp::BitAnd),
            AssignOp::BitOrAssign => Some(BinaryOp::BitOr),
            AssignOp::BitXorAssign => Some(BinaryOp::BitXor),
            AssignOp::ShiftLeftAssign => Some(BinaryOp::ShiftLeft),
            AssignOp::ShiftRightAssign => Some(BinaryOp::ShiftRight),
        }
    }

    fn analyze_assign_target(&mut self, target: &AssignTarget, span: &Span) -> Option<Type> {
        match target {
            AssignTarget::Variable(name) => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    symbol.get_type().cloned()
                } else {
                    self.error(CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        span.clone(),
                    ));
                    None
                }
            }
            AssignTarget::ArrayElement { name, index } => {
                // Check array exists
                let element_type = if let Some(symbol) = self.symbols.lookup(name) {
                    if let Some(arr_type) = symbol.get_type() {
                        arr_type.element_type()
                    } else {
                        self.error(CompileError::new(
                            ErrorCode::CannotIndexNonArray,
                            format!("Cannot index non-array '{}'", name),
                            span.clone(),
                        ));
                        None
                    }
                } else {
                    self.error(CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        span.clone(),
                    ));
                    None
                };

                // Check index is integer
                let index_type = self.analyze_expression(index);
                if let Some(index_type) = index_type {
                    if !index_type.is_integer() {
                        self.error(CompileError::new(
                            ErrorCode::ArrayIndexMustBeInteger,
                            format!("Array index must be an integer, found {}", index_type),
                            index.span.clone(),
                        ));
                    }
                }

                element_type
            }
        }
    }
}
