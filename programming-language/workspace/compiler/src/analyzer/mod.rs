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

//! Semantic analyzer module for the Cobra64 compiler.
//!
//! This module performs semantic analysis on the AST:
//! - Symbol resolution (variables, functions)
//! - Type checking
//! - Constant evaluation
//! - Error detection (undefined variables, type mismatches, etc.)

use crate::ast::{
    AssignOp, AssignTarget, Assignment, BinaryOp, Block, ConstDecl, Expr, ExprKind, ForStatement,
    FunctionDef, IfStatement, Program, Statement, StatementKind, TopLevelItem, Type, UnaryOp,
    VarDecl, WhileStatement,
};
use crate::error::{CompileError, ErrorCode, Span};
use std::collections::HashMap;

/// Symbol table entry.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The symbol name.
    pub name: String,
    /// The symbol type.
    pub symbol_type: SymbolType,
    /// Whether this is a constant.
    pub is_constant: bool,
    /// The memory address (assigned during code generation).
    pub address: Option<u16>,
    /// The span where this symbol was defined.
    pub span: Span,
}

impl Symbol {
    /// Create a new variable symbol.
    pub fn variable(name: String, var_type: Type, is_constant: bool, span: Span) -> Self {
        Self {
            name,
            symbol_type: SymbolType::Variable(var_type),
            is_constant,
            address: None,
            span,
        }
    }

    /// Create a new function symbol.
    pub fn function(
        name: String,
        params: Vec<Type>,
        return_type: Option<Type>,
        span: Span,
    ) -> Self {
        Self {
            name,
            symbol_type: SymbolType::Function {
                params,
                return_type,
            },
            is_constant: true, // Functions are always immutable
            address: None,
            span,
        }
    }

    /// Get the type of a variable symbol.
    pub fn get_type(&self) -> Option<&Type> {
        match &self.symbol_type {
            SymbolType::Variable(t) => Some(t),
            SymbolType::Function { .. } => None,
        }
    }
}

/// The type of a symbol.
#[derive(Debug, Clone)]
pub enum SymbolType {
    /// A variable or constant.
    Variable(Type),
    /// A function.
    Function {
        params: Vec<Type>,
        return_type: Option<Type>,
    },
}

/// A scope in the symbol table.
#[derive(Debug, Default)]
pub struct Scope {
    /// Symbols defined in this scope.
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a symbol in this scope.
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Symbol> {
        if let Some(existing) = self.symbols.get(&symbol.name) {
            return Err(existing.clone());
        }
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    /// Look up a symbol in this scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Look up a symbol in this scope (mutable).
    #[allow(dead_code)]
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }
}

/// The symbol table for semantic analysis.
#[derive(Debug)]
pub struct SymbolTable {
    /// The scope stack (innermost scope last).
    scopes: Vec<Scope>,
}

impl SymbolTable {
    /// Create a new symbol table with a global scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    /// Push a new scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope from the stack.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a symbol in the current scope.
    pub fn define(&mut self, symbol: Symbol) -> Result<(), Symbol> {
        self.scopes
            .last_mut()
            .expect("No scope available")
            .define(symbol)
    }

    /// Look up a symbol, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.lookup(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Check if we're in the global scope.
    pub fn is_global_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Get the current scope depth.
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for semantic analysis.
#[derive(Debug, Clone, Default)]
pub struct AnalysisContext {
    /// Whether we're inside a loop (for break/continue validation).
    pub in_loop: bool,
    /// Whether we're inside a function.
    pub in_function: bool,
    /// The expected return type of the current function.
    pub return_type: Option<Type>,
    /// The current function name (for error messages).
    pub function_name: Option<String>,
}

/// The semantic analyzer.
pub struct Analyzer {
    /// The symbol table.
    pub symbols: SymbolTable,
    /// Collected errors.
    errors: Vec<CompileError>,
    /// Analysis context.
    context: AnalysisContext,
}

impl Analyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        let mut analyzer = Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            context: AnalysisContext::default(),
        };
        analyzer.register_builtins();
        analyzer
    }

    /// Register built-in functions.
    fn register_builtins(&mut self) {
        // cls() - clear screen
        self.define_builtin("cls", vec![], None);

        // print(value) - print without newline
        self.define_builtin("print", vec![Type::String], None);

        // println(value) - print with newline
        self.define_builtin("println", vec![Type::String], None);

        // cursor(x, y) - set cursor position
        self.define_builtin("cursor", vec![Type::Byte, Type::Byte], None);

        // get_key() -> byte - get key without waiting
        self.define_builtin("get_key", vec![], Some(Type::Byte));

        // wait_for_key() -> byte - wait for key press
        self.define_builtin("wait_for_key", vec![], Some(Type::Byte));

        // readln() -> string - read a line of input
        self.define_builtin("readln", vec![], Some(Type::String));

        // poke(addr, value) - write to memory
        self.define_builtin("poke", vec![Type::Word, Type::Byte], None);

        // peek(addr) -> byte - read from memory
        self.define_builtin("peek", vec![Type::Word], Some(Type::Byte));
    }

    /// Define a built-in function.
    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Option<Type>) {
        let symbol = Symbol::function(name.to_string(), params, return_type, Span::new(0, 0));
        let _ = self.symbols.define(symbol);
    }

    /// Analyze a program.
    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<CompileError>> {
        // First pass: collect all top-level declarations
        self.collect_declarations(program);

        // Check for main function
        if self.symbols.lookup("main").is_none() {
            self.error(CompileError::new(
                ErrorCode::UndefinedFunction,
                "Missing main() function",
                Span::new(0, 0),
            ));
        }

        // Second pass: analyze all items
        for item in &program.items {
            self.analyze_top_level_item(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Collect all top-level declarations (first pass).
    fn collect_declarations(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => {
                    let param_types: Vec<Type> =
                        func.params.iter().map(|p| p.param_type.clone()).collect();
                    let symbol = Symbol::function(
                        func.name.clone(),
                        param_types,
                        func.return_type.clone(),
                        func.span.clone(),
                    );
                    if let Err(existing) = self.symbols.define(symbol) {
                        self.error(
                            CompileError::new(
                                ErrorCode::FunctionAlreadyDefined,
                                format!("Function '{}' is already defined", func.name),
                                func.span.clone(),
                            )
                            .with_hint(format!(
                                "Previously defined at position {}",
                                existing.span.start
                            )),
                        );
                    }
                }
                TopLevelItem::Constant(decl) => {
                    self.analyze_const_decl(decl);
                }
                TopLevelItem::Variable(decl) => {
                    self.analyze_var_decl(decl);
                }
            }
        }
    }

    /// Analyze a top-level item.
    fn analyze_top_level_item(&mut self, item: &TopLevelItem) {
        match item {
            TopLevelItem::Function(func) => {
                self.analyze_function(func);
            }
            TopLevelItem::Constant(_) | TopLevelItem::Variable(_) => {
                // Already handled in first pass
            }
        }
    }

    /// Analyze a function definition.
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

    /// Check if a block definitely returns a value.
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

    /// Analyze a block of statements.
    fn analyze_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.analyze_statement(stmt);
        }
    }

    /// Analyze a statement.
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

    /// Analyze a variable declaration.
    fn analyze_var_decl(&mut self, decl: &VarDecl) {
        // Check initializer type if present
        if let Some(init) = &decl.initializer {
            let init_type = self.analyze_expression(init);
            if let Some(init_type) = init_type {
                if !self.is_expr_assignable_to(&init.kind, &init_type, &decl.var_type) {
                    self.error(CompileError::new(
                        ErrorCode::TypeMismatch,
                        format!(
                            "Cannot assign {} to variable of type {}",
                            init_type, decl.var_type
                        ),
                        init.span.clone(),
                    ));
                }
            }

            // Check compile-time range for integer types
            if decl.var_type.is_integer() {
                if let Some(value) = self.try_eval_constant(init) {
                    self.check_value_in_range(value, &decl.var_type, &init.span);
                }
            }
        }

        // Add to symbol table
        let symbol = Symbol::variable(
            decl.name.clone(),
            decl.var_type.clone(),
            false,
            decl.span.clone(),
        );
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

    /// Analyze a constant declaration.
    fn analyze_const_decl(&mut self, decl: &ConstDecl) {
        // Analyze the value expression
        let value_type = self.analyze_expression(&decl.value);

        // Infer type from value
        let const_type = value_type.unwrap_or(Type::Word);

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

    /// Analyze an assignment statement.
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

    /// Convert an assignment operator to the corresponding binary operator.
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

    /// Analyze an assignment target.
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

    /// Analyze an if statement.
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

    /// Analyze a while statement.
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

    /// Analyze a for statement.
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

    /// Analyze a return statement.
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

    /// Analyze an expression and return its type.
    fn analyze_expression(&mut self, expr: &Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::IntegerLiteral(n) => {
                if *n <= 255 {
                    Some(Type::Byte)
                } else {
                    Some(Type::Word)
                }
            }
            ExprKind::StringLiteral(_) => Some(Type::String),
            ExprKind::CharLiteral(_) => Some(Type::Byte),
            ExprKind::BoolLiteral(_) => Some(Type::Bool),
            ExprKind::Identifier(name) => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    symbol.get_type().cloned()
                } else {
                    self.error(CompileError::new(
                        ErrorCode::UndefinedVariable,
                        format!("Undefined variable '{}'", name),
                        expr.span.clone(),
                    ));
                    None
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);
                self.check_binary_op(&left_type, op, &right_type, &expr.span)
            }
            ExprKind::UnaryOp { op, operand } => {
                let operand_type = self.analyze_expression(operand);
                self.check_unary_op(op, &operand_type, &expr.span)
            }
            ExprKind::FunctionCall { name, args } => {
                self.analyze_function_call(name, args, &expr.span)
            }
            ExprKind::ArrayIndex { array, index } => {
                let array_type = self.analyze_expression(array);
                let index_type = self.analyze_expression(index);

                // Check index is integer
                if let Some(index_type) = index_type {
                    if !index_type.is_integer() {
                        self.error(CompileError::new(
                            ErrorCode::ArrayIndexMustBeInteger,
                            format!("Array index must be an integer, found {}", index_type),
                            index.span.clone(),
                        ));
                    }
                }

                // Return element type
                array_type.and_then(|t| t.element_type())
            }
            ExprKind::TypeCast {
                target_type,
                expr: inner,
            } => {
                self.analyze_expression(inner);
                Some(target_type.clone())
            }
            ExprKind::FixedLiteral(_) => Some(Type::Fixed),
            ExprKind::FloatLiteral(_) => Some(Type::Float),
            ExprKind::DecimalLiteral(_) => {
                // DecimalLiteral defaults to float type during analysis.
                // Context-based conversion (e.g., for fixed variables) will be
                // handled in a later phase during assignment checking.
                Some(Type::Float)
            }
            ExprKind::Grouped(inner) => self.analyze_expression(inner),
        }
    }

    /// Check binary operator types and return result type.
    fn check_binary_op(
        &mut self,
        left: &Option<Type>,
        op: &BinaryOp,
        right: &Option<Type>,
        span: &Span,
    ) -> Option<Type> {
        let (left, right) = match (left, right) {
            (Some(l), Some(r)) => (l, r),
            _ => return None,
        };

        self.check_binary_op_types(left, op, right, span)
    }

    /// Check binary operator types and return result type.
    fn check_binary_op_types(
        &mut self,
        left: &Type,
        op: &BinaryOp,
        right: &Type,
        span: &Span,
    ) -> Option<Type> {
        match op {
            // Arithmetic operators - valid for all numeric types (integers, fixed, float)
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                if !left.is_numeric() || !right.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Operator {} requires numeric operands, found {} and {}",
                            op, left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Modulo - valid for integers and fixed, but NOT for float
            BinaryOp::Mod => {
                if left.is_float() || right.is_float() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Modulo operator is not valid for float type, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                if !left.is_numeric() || !right.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Operator {} requires numeric operands, found {} and {}",
                            op, left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Comparison operators - valid for all numeric types
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual => {
                // Allow comparisons between any numeric types (promotion handled by binary_result_type)
                if left.is_numeric() && right.is_numeric() {
                    // Valid comparison
                } else if left != right && Type::binary_result_type(left, right).is_none() {
                    self.error(CompileError::new(
                        ErrorCode::CannotCompareTypes,
                        format!("Cannot compare {} and {}", left, right),
                        span.clone(),
                    ));
                }
                Some(Type::Bool)
            }

            // Logical operators - only valid for bool
            BinaryOp::And | BinaryOp::Or => {
                if *left != Type::Bool || *right != Type::Bool {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Logical operators require boolean operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Some(Type::Bool)
            }

            // Bitwise operators - only valid for integers (NOT fixed/float)
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                if !left.is_integer() || !right.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Bitwise operators require integer operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Type::binary_result_type(left, right)
            }

            // Shift operators - only valid for integers (NOT fixed/float)
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                if !left.is_integer() || !right.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!(
                            "Shift operators require integer operands, found {} and {}",
                            left, right
                        ),
                        span.clone(),
                    ));
                    return None;
                }
                Some(left.clone())
            }
        }
    }

    /// Check unary operator types and return result type.
    fn check_unary_op(
        &mut self,
        op: &UnaryOp,
        operand: &Option<Type>,
        span: &Span,
    ) -> Option<Type> {
        let operand = operand.as_ref()?;

        match op {
            UnaryOp::Negate => {
                if !operand.is_numeric() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("Cannot negate non-numeric type {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                // Negation promotes unsigned integers to signed type
                // Fixed and float stay the same (already signed)
                match operand {
                    Type::Byte => Some(Type::Sbyte),
                    Type::Word => Some(Type::Sword),
                    Type::Fixed => Some(Type::Fixed),
                    Type::Float => Some(Type::Float),
                    _ => Some(operand.clone()),
                }
            }
            UnaryOp::Not => {
                if *operand != Type::Bool {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("'not' requires boolean operand, found {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            UnaryOp::BitNot => {
                if !operand.is_integer() {
                    self.error(CompileError::new(
                        ErrorCode::InvalidOperatorForType,
                        format!("Bitwise NOT requires integer operand, found {}", operand),
                        span.clone(),
                    ));
                    return None;
                }
                Some(operand.clone())
            }
        }
    }

    /// Check if an expression can be assigned to a target type.
    ///
    /// This handles the special case of `DecimalLiteral` which can be
    /// assigned to both `fixed` and `float` types (type is determined by context).
    fn is_expr_assignable_to(
        &self,
        expr_kind: &ExprKind,
        value_type: &Type,
        target_type: &Type,
    ) -> bool {
        // DecimalLiteral can be assigned to both fixed and float
        if matches!(expr_kind, ExprKind::DecimalLiteral(_))
            && (target_type.is_fixed() || target_type.is_float())
        {
            return true;
        }

        // Handle negated decimal literals: -3.5 etc.
        if let ExprKind::UnaryOp {
            op: UnaryOp::Negate,
            operand,
        } = expr_kind
        {
            if matches!(operand.kind, ExprKind::DecimalLiteral(_))
                && (target_type.is_fixed() || target_type.is_float())
            {
                return true;
            }
        }

        // Default to standard type assignability
        value_type.is_assignable_to(target_type)
    }

    /// Analyze a function call.
    fn analyze_function_call(&mut self, name: &str, args: &[Expr], span: &Span) -> Option<Type> {
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

    /// Add an error to the error list.
    pub fn error(&mut self, error: CompileError) {
        self.errors.push(error);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Try to evaluate a constant expression at compile time.
    /// Returns the evaluated value as i64 to handle both signed and unsigned.
    fn try_eval_constant(&self, expr: &Expr) -> Option<i64> {
        match &expr.kind {
            ExprKind::IntegerLiteral(n) => Some(*n as i64),
            ExprKind::BoolLiteral(b) => Some(if *b { 1 } else { 0 }),
            ExprKind::CharLiteral(c) => Some(*c as i64),
            ExprKind::UnaryOp { op, operand } => {
                let operand_val = self.try_eval_constant(operand)?;
                match op {
                    UnaryOp::Negate => Some(-operand_val),
                    UnaryOp::Not => Some(if operand_val == 0 { 1 } else { 0 }),
                    UnaryOp::BitNot => Some(!operand_val),
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                let left_val = self.try_eval_constant(left)?;
                let right_val = self.try_eval_constant(right)?;
                match op {
                    BinaryOp::Add => Some(left_val.wrapping_add(right_val)),
                    BinaryOp::Sub => Some(left_val.wrapping_sub(right_val)),
                    BinaryOp::Mul => Some(left_val.wrapping_mul(right_val)),
                    BinaryOp::Div => {
                        if right_val == 0 {
                            None
                        } else {
                            Some(left_val / right_val)
                        }
                    }
                    BinaryOp::Mod => {
                        if right_val == 0 {
                            None
                        } else {
                            Some(left_val % right_val)
                        }
                    }
                    BinaryOp::BitAnd => Some(left_val & right_val),
                    BinaryOp::BitOr => Some(left_val | right_val),
                    BinaryOp::BitXor => Some(left_val ^ right_val),
                    BinaryOp::ShiftLeft => Some(left_val << (right_val & 0x3F)),
                    BinaryOp::ShiftRight => Some(left_val >> (right_val & 0x3F)),
                    _ => None, // Comparison/logical ops don't produce numeric results
                }
            }
            ExprKind::Grouped(inner) => self.try_eval_constant(inner),
            ExprKind::Identifier(name) => {
                // Try to look up constant value
                if let Some(symbol) = self.symbols.lookup(name) {
                    if symbol.is_constant {
                        // For now, we don't track constant values, return None
                        None
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a value is within the valid range for a type.
    fn check_value_in_range(&mut self, value: i64, target_type: &Type, span: &Span) {
        let (min, max, type_name) = match target_type {
            Type::Byte => (0, 255, "byte"),
            Type::Word => (0, 65535, "word"),
            Type::Sbyte => (-128, 127, "sbyte"),
            Type::Sword => (-32768, 32767, "sword"),
            Type::Bool => (0, 1, "bool"),
            _ => return, // No range check for other types
        };

        if value < min || value > max {
            self.error(
                CompileError::new(
                    ErrorCode::ConstantValueOutOfRange,
                    format!(
                        "Value {} is out of range for {} ({} to {})",
                        value, type_name, min, max
                    ),
                    span.clone(),
                )
                .with_hint(format!(
                    "Valid range for {} is {} to {}",
                    type_name, min, max
                )),
            );
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyze a program for semantic correctness.
pub fn analyze(program: &Program) -> Result<(), Vec<CompileError>> {
    let mut analyzer = Analyzer::new();
    analyzer.analyze(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    /// Helper to analyze source code directly.
    fn analyze_source(source: &str) -> Result<(), Vec<CompileError>> {
        let tokens = tokenize(source).map_err(|e| vec![e])?;
        let program = parse(&tokens).map_err(|e| vec![e])?;
        analyze(&program)
    }

    /// Helper to check if analysis produces a specific error code.
    fn has_error_code(result: &Result<(), Vec<CompileError>>, code: ErrorCode) -> bool {
        match result {
            Err(errors) => errors.iter().any(|e| e.code == code),
            Ok(_) => false,
        }
    }

    // ========================================
    // Symbol Table Tests
    // ========================================

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();

        let sym = Symbol::variable("x".to_string(), Type::Byte, false, Span::new(0, 1));
        table.define(sym).unwrap();
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_scope_nesting() {
        let mut table = SymbolTable::new();

        let global_sym = Symbol::variable("x".to_string(), Type::Byte, false, Span::new(0, 1));
        table.define(global_sym).unwrap();

        table.push_scope();

        let local_sym = Symbol::variable("y".to_string(), Type::Word, false, Span::new(5, 6));
        table.define(local_sym).unwrap();

        // Both should be visible
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_some());

        table.pop_scope();

        // Only global should be visible now
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }

    #[test]
    fn test_symbol_shadowing() {
        let mut table = SymbolTable::new();

        let global_sym = Symbol::variable("x".to_string(), Type::Byte, false, Span::new(0, 1));
        table.define(global_sym).unwrap();

        table.push_scope();

        // Define same name in inner scope (shadowing)
        let local_sym = Symbol::variable("x".to_string(), Type::Word, false, Span::new(5, 6));
        table.define(local_sym).unwrap();

        // Should find the local one
        let found = table.lookup("x").unwrap();
        assert!(matches!(
            found.symbol_type,
            SymbolType::Variable(Type::Word)
        ));

        table.pop_scope();

        // Now should find the global one
        let found = table.lookup("x").unwrap();
        assert!(matches!(
            found.symbol_type,
            SymbolType::Variable(Type::Byte)
        ));
    }

    // ========================================
    // Valid Programs
    // ========================================

    #[test]
    fn test_valid_minimal_program() {
        let result = analyze_source("def main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_variable_declaration() {
        let result = analyze_source("def main():\n    x: byte = 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_constant_declaration() {
        let result = analyze_source("const MAX = 100\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_function_with_params() {
        let result = analyze_source(
            "def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    pass",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_if_statement() {
        let result = analyze_source("def main():\n    if true:\n        pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_while_loop() {
        let result = analyze_source("def main():\n    while true:\n        break");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_for_loop() {
        let result = analyze_source("def main():\n    for i in 0 to 10:\n        pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_function_call() {
        let result = analyze_source("def main():\n    cls()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_print_call() {
        let result = analyze_source("def main():\n    println(\"hello\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_type_promotion() {
        let result = analyze_source("def main():\n    x: word = 5");
        assert!(result.is_ok()); // byte literal 5 is assignable to word
    }

    // ========================================
    // Missing Main Function
    // ========================================

    #[test]
    fn test_error_missing_main() {
        let result = analyze_source("def helper():\n    pass");
        assert!(has_error_code(&result, ErrorCode::UndefinedFunction));
    }

    // ========================================
    // Undefined Variables
    // ========================================

    #[test]
    fn test_error_undefined_variable() {
        let result = analyze_source("def main():\n    x = 5");
        assert!(has_error_code(&result, ErrorCode::UndefinedVariable));
    }

    #[test]
    fn test_error_undefined_in_expression() {
        let result = analyze_source("def main():\n    y: byte = x + 1");
        assert!(has_error_code(&result, ErrorCode::UndefinedVariable));
    }

    // ========================================
    // Duplicate Definitions
    // ========================================

    #[test]
    fn test_error_duplicate_variable() {
        let result = analyze_source("def main():\n    x: byte\n    x: word");
        assert!(has_error_code(&result, ErrorCode::VariableAlreadyDefined));
    }

    #[test]
    fn test_error_duplicate_function() {
        let result =
            analyze_source("def foo():\n    pass\n\ndef foo():\n    pass\n\ndef main():\n    pass");
        assert!(has_error_code(&result, ErrorCode::FunctionAlreadyDefined));
    }

    #[test]
    fn test_error_duplicate_parameter() {
        let result =
            analyze_source("def foo(x: byte, x: byte):\n    pass\n\ndef main():\n    pass");
        assert!(has_error_code(&result, ErrorCode::DuplicateParameterName));
    }

    // ========================================
    // Type Mismatches
    // ========================================

    #[test]
    fn test_error_type_mismatch_assignment() {
        let result = analyze_source("def main():\n    x: byte = \"hello\"");
        assert!(has_error_code(&result, ErrorCode::TypeMismatch));
    }

    #[test]
    fn test_error_type_mismatch_condition() {
        let result = analyze_source("def main():\n    if 42:\n        pass");
        assert!(has_error_code(&result, ErrorCode::TypeMismatch));
    }

    #[test]
    fn test_error_type_mismatch_while() {
        let result = analyze_source("def main():\n    while 1:\n        pass");
        assert!(has_error_code(&result, ErrorCode::TypeMismatch));
    }

    // ========================================
    // Operator Type Errors
    // ========================================

    #[test]
    fn test_error_add_string() {
        let result = analyze_source("def main():\n    x: byte = \"a\" + \"b\"");
        assert!(has_error_code(&result, ErrorCode::InvalidOperatorForType));
    }

    #[test]
    fn test_error_logical_on_integers() {
        let result = analyze_source("def main():\n    x: bool = 1 and 2");
        assert!(has_error_code(&result, ErrorCode::InvalidOperatorForType));
    }

    #[test]
    fn test_error_not_on_integer() {
        let result = analyze_source("def main():\n    x: bool = not 5");
        assert!(has_error_code(&result, ErrorCode::InvalidOperatorForType));
    }

    // ========================================
    // Constant Assignment
    // ========================================

    #[test]
    fn test_error_assign_to_constant() {
        let result = analyze_source("const X = 5\ndef main():\n    X = 10");
        assert!(has_error_code(&result, ErrorCode::CannotAssignToConstant));
    }

    // ========================================
    // Break/Continue Outside Loop
    // ========================================

    #[test]
    fn test_error_break_outside_loop() {
        let result = analyze_source("def main():\n    break");
        assert!(has_error_code(&result, ErrorCode::BreakOutsideLoop));
    }

    #[test]
    fn test_error_continue_outside_loop() {
        let result = analyze_source("def main():\n    continue");
        assert!(has_error_code(&result, ErrorCode::ContinueOutsideLoop));
    }

    #[test]
    fn test_valid_break_in_nested_function() {
        // Break inside a loop is valid
        let result = analyze_source("def main():\n    while true:\n        break");
        assert!(result.is_ok());
    }

    // ========================================
    // Return Statement Errors
    // ========================================

    #[test]
    fn test_error_return_outside_function() {
        // This is actually hard to test since parsing requires functions
        // Skip this test as it requires special setup
    }

    #[test]
    fn test_error_return_value_from_void() {
        let result = analyze_source("def main():\n    return 5");
        assert!(has_error_code(
            &result,
            ErrorCode::CannotReturnValueFromVoid
        ));
    }

    #[test]
    fn test_error_missing_return_value() {
        let result = analyze_source("def get() -> byte:\n    return\n\ndef main():\n    pass");
        assert!(has_error_code(&result, ErrorCode::MissingReturnValue));
    }

    #[test]
    fn test_error_return_type_mismatch() {
        let result =
            analyze_source("def get() -> byte:\n    return \"hello\"\n\ndef main():\n    pass");
        assert!(has_error_code(&result, ErrorCode::TypeMismatch));
    }

    // ========================================
    // Function Call Errors
    // ========================================

    #[test]
    fn test_error_undefined_function_call() {
        let result = analyze_source("def main():\n    foo()");
        assert!(has_error_code(&result, ErrorCode::UndefinedFunction));
    }

    #[test]
    fn test_error_wrong_argument_count() {
        let result = analyze_source("def main():\n    cursor(1)");
        assert!(has_error_code(&result, ErrorCode::WrongNumberOfArguments));
    }

    #[test]
    fn test_error_argument_type_mismatch() {
        let result = analyze_source("def main():\n    cursor(\"x\", \"y\")");
        assert!(has_error_code(&result, ErrorCode::ArgumentTypeMismatch));
    }

    // ========================================
    // Array Errors
    // ========================================

    #[test]
    fn test_error_array_index_not_integer() {
        let result = analyze_source("def main():\n    arr: byte[10]\n    x: byte = arr[true]");
        assert!(has_error_code(&result, ErrorCode::ArrayIndexMustBeInteger));
    }

    // ========================================
    // Expression Type Tests
    // ========================================

    #[test]
    fn test_comparison_returns_bool() {
        let result = analyze_source("def main():\n    x: bool = 1 == 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_logical_expression() {
        let result = analyze_source("def main():\n    x: bool = true and false");
        assert!(result.is_ok());
    }

    #[test]
    fn test_arithmetic_expression() {
        let result = analyze_source("def main():\n    x: byte = 1 + 2 * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitwise_expression() {
        let result = analyze_source("def main():\n    x: byte = 1 & 2 | 3");
        assert!(result.is_ok());
    }

    // ========================================
    // Built-in Functions
    // ========================================

    #[test]
    fn test_builtin_cls() {
        let result = analyze_source("def main():\n    cls()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_cursor() {
        let result = analyze_source("def main():\n    cursor(10, 5)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_peek_poke() {
        let result = analyze_source("def main():\n    poke(53280, 0)\n    x: byte = peek(53280)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_get_key() {
        let result = analyze_source("def main():\n    k: byte = get_key()");
        assert!(result.is_ok());
    }

    // ========================================
    // Scope Tests
    // ========================================

    #[test]
    fn test_variable_scope_in_if() {
        // Variable defined in if block should not be visible outside
        let result =
            analyze_source("def main():\n    if true:\n        x: byte = 5\n    y: byte = x\n");
        assert!(has_error_code(&result, ErrorCode::UndefinedVariable));
    }

    #[test]
    fn test_variable_scope_in_while() {
        let result = analyze_source(
            "def main():\n    while true:\n        x: byte = 5\n        break\n    y: byte = x\n",
        );
        assert!(has_error_code(&result, ErrorCode::UndefinedVariable));
    }

    #[test]
    fn test_for_loop_variable_scope() {
        // Loop variable should be visible inside the loop
        let result = analyze_source("def main():\n    for i in 0 to 10:\n        x: byte = i");
        assert!(result.is_ok());
    }

    // ========================================
    // User-defined Function Calls
    // ========================================

    #[test]
    fn test_call_user_function() {
        let result = analyze_source("def helper():\n    pass\n\ndef main():\n    helper()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_with_return() {
        let result = analyze_source(
            "def get_five() -> byte:\n    return 5\n\ndef main():\n    x: byte = get_five()",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_function_with_args() {
        let result = analyze_source(
            "def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    x: byte = add(1, 2)",
        );
        assert!(result.is_ok());
    }

    // ========================================
    // Signed Type Tests
    // ========================================

    #[test]
    fn test_valid_sbyte_declaration() {
        let result = analyze_source("def main():\n    x: sbyte = -100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sbyte_min_value() {
        let result = analyze_source("def main():\n    x: sbyte = -128");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sbyte_max_value() {
        let result = analyze_source("def main():\n    x: sbyte = 127");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sword_declaration() {
        let result = analyze_source("def main():\n    y: sword = -30000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sword_min_value() {
        let result = analyze_source("def main():\n    y: sword = -32768");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sword_max_value() {
        let result = analyze_source("def main():\n    y: sword = 32767");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_sbyte_overflow_positive() {
        let result = analyze_source("def main():\n    x: sbyte = 128");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_sbyte_overflow_negative() {
        let result = analyze_source("def main():\n    x: sbyte = -129");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_sword_overflow_positive() {
        let result = analyze_source("def main():\n    y: sword = 32768");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_sword_overflow_negative() {
        let result = analyze_source("def main():\n    y: sword = -32769");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_valid_sbyte_zero() {
        let result = analyze_source("def main():\n    x: sbyte = 0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sbyte_negative_zero() {
        let result = analyze_source("def main():\n    x: sbyte = -0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sbyte_negative_one() {
        let result = analyze_source("def main():\n    x: sbyte = -1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_negation_type_promotion() {
        // Negating a byte should produce sbyte
        let result = analyze_source("def main():\n    x: byte = 5\n    y: sbyte = -x");
        // This should work because -x on byte becomes sbyte
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sbyte_function_param() {
        let result =
            analyze_source("def process(val: sbyte):\n    pass\n\ndef main():\n    process(-100)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_sword_function_return() {
        let result = analyze_source(
            "def get_value() -> sword:\n    return -1000\n\ndef main():\n    y: sword = get_value()",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_sbyte_to_sword_promotion() {
        // sbyte should be assignable to sword
        let result = analyze_source("def main():\n    x: sbyte = -100\n    y: sword = x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_byte_to_sword_promotion() {
        // byte should be assignable to sword
        let result = analyze_source("def main():\n    x: byte = 100\n    y: sword = x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_negative_to_unsigned() {
        // Negative value should not be assignable to byte
        let result = analyze_source("def main():\n    x: byte = -1");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_negative_to_word() {
        // Negative value should not be assignable to word
        let result = analyze_source("def main():\n    x: word = -1");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_valid_const_negative() {
        let result = analyze_source("const MIN = -128\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sbyte_arithmetic() {
        let result = analyze_source(
            "def main():\n    x: sbyte = -50\n    y: sbyte = -50\n    z: sbyte = x + y",
        );
        // Note: This might overflow at runtime, but compile-time doesn't check this
        assert!(result.is_ok());
    }

    #[test]
    fn test_sbyte_comparison() {
        let result = analyze_source("def main():\n    x: sbyte = -50\n    if x < 0:\n        pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sword_comparison() {
        let result =
            analyze_source("def main():\n    x: sword = -1000\n    if x > -2000:\n        pass");
        assert!(result.is_ok());
    }

    // ========================================
    // Fixed-Point and Float Type Tests
    // ========================================

    #[test]
    fn test_fixed_declaration() {
        let result = analyze_source("def main():\n    x: fixed = 3.75");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_declaration() {
        let result = analyze_source("def main():\n    x: float = 3.14");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_arithmetic_add() {
        let result = analyze_source(
            "def main():\n    x: fixed = 1.5\n    y: fixed = 2.5\n    z: fixed = x + y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_arithmetic_sub() {
        let result = analyze_source(
            "def main():\n    x: fixed = 5.0\n    y: fixed = 2.5\n    z: fixed = x - y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_arithmetic_mul() {
        let result = analyze_source(
            "def main():\n    x: fixed = 2.0\n    y: fixed = 3.5\n    z: fixed = x * y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_arithmetic_div() {
        let result = analyze_source(
            "def main():\n    x: fixed = 10.0\n    y: fixed = 2.0\n    z: fixed = x / y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_modulo() {
        let result = analyze_source(
            "def main():\n    x: fixed = 10.5\n    y: fixed = 3.0\n    z: fixed = x % y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_arithmetic_add() {
        let result = analyze_source(
            "def main():\n    x: float = 1.5\n    y: float = 2.5\n    z: float = x + y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_arithmetic_mul() {
        let result = analyze_source(
            "def main():\n    x: float = 2.0\n    y: float = 3.5\n    z: float = x * y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_arithmetic_div() {
        let result = analyze_source(
            "def main():\n    x: float = 10.0\n    y: float = 2.0\n    z: float = x / y",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_comparison() {
        let result =
            analyze_source("def main():\n    x: fixed = 3.5\n    if x > 2.0:\n        pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_comparison() {
        let result =
            analyze_source("def main():\n    x: float = 3.14\n    if x < 4.0:\n        pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_negation() {
        let result = analyze_source("def main():\n    x: fixed = 3.5\n    y: fixed = -x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_negation() {
        let result = analyze_source("def main():\n    x: float = 3.14\n    y: float = -x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_type_cast_from_integer() {
        let result = analyze_source("def main():\n    x: byte = 10\n    y: fixed = fixed(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_type_cast_from_integer() {
        let result = analyze_source("def main():\n    x: word = 1000\n    y: float = float(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_byte_cast_from_fixed() {
        let result = analyze_source("def main():\n    x: fixed = 3.5\n    y: byte = byte(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_byte_cast_from_float() {
        let result = analyze_source("def main():\n    x: float = 3.14\n    y: byte = byte(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_plus_integer() {
        // Integer should be promoted to fixed
        let result = analyze_source("def main():\n    x: fixed = 3.5\n    y: fixed = x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_plus_integer() {
        // Integer should be promoted to float
        let result = analyze_source("def main():\n    x: float = 3.14\n    y: float = x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_float_modulo() {
        // Modulo is not valid for float
        let result = analyze_source(
            "def main():\n    x: float = 10.0\n    y: float = 3.0\n    z: float = x % y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_fixed_bitwise_and() {
        // Bitwise AND is not valid for fixed
        let result = analyze_source(
            "def main():\n    x: fixed = 10.0\n    y: fixed = 3.0\n    z: fixed = x & y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_fixed_bitwise_or() {
        // Bitwise OR is not valid for fixed
        let result = analyze_source(
            "def main():\n    x: fixed = 10.0\n    y: fixed = 3.0\n    z: fixed = x | y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_float_bitwise_xor() {
        // Bitwise XOR is not valid for float
        let result = analyze_source(
            "def main():\n    x: float = 10.0\n    y: float = 3.0\n    z: float = x ^ y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_fixed_shift_left() {
        // Shift left is not valid for fixed
        let result = analyze_source("def main():\n    x: fixed = 10.0\n    z: fixed = x << 2");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_float_shift_right() {
        // Shift right is not valid for float
        let result = analyze_source("def main():\n    x: float = 10.0\n    z: float = x >> 2");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_fixed_logical_and() {
        // Logical AND is not valid for fixed
        let result = analyze_source(
            "def main():\n    x: fixed = 1.0\n    y: fixed = 2.0\n    z: bool = x and y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_float_logical_or() {
        // Logical OR is not valid for float
        let result = analyze_source(
            "def main():\n    x: float = 1.0\n    y: float = 2.0\n    z: bool = x or y",
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_fixed_logical_not() {
        // Logical NOT is not valid for fixed
        let result = analyze_source("def main():\n    x: fixed = 1.0\n    y: bool = not x");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_error_float_bitwise_not() {
        // Bitwise NOT is not valid for float
        let result = analyze_source("def main():\n    x: float = 1.0\n    y: float = ~x");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_fixed_function_param() {
        // Note: Using fixed(2.0) to keep the result as fixed type
        // Without the cast, 2.0 would be float and val*2.0 would be float
        let result = analyze_source("def scale(val: fixed) -> fixed:\n    return val * fixed(2.0)\ndef main():\n    x: fixed = scale(1.5)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float_function_param() {
        let result = analyze_source("def compute(val: float) -> float:\n    return val + 1.0\ndef main():\n    x: float = compute(3.14)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_fixed_float_promotes_to_float() {
        // When mixing fixed and float, result should be float
        let result = analyze_source(
            "def main():\n    x: fixed = 1.5\n    y: float = 2.5\n    z: float = x + y",
        );
        assert!(result.is_ok());
    }
}
