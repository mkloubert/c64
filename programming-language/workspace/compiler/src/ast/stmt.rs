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

//! Statement AST nodes for the Cobra64 compiler.

use crate::error::Span;

use super::{Block, Expr, Type};

/// A statement in the Cobra64 language.
#[derive(Debug, Clone)]
pub struct Statement {
    /// The kind of statement.
    pub kind: StatementKind,
    /// The source span of this statement.
    pub span: Span,
}

impl Statement {
    /// Create a new statement.
    pub fn new(kind: StatementKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The kind of statement.
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// A variable declaration.
    VarDecl(VarDecl),

    /// A constant declaration.
    ConstDecl(ConstDecl),

    /// An assignment statement.
    Assignment(Assignment),

    /// An if statement.
    If(IfStatement),

    /// A while loop.
    While(WhileStatement),

    /// A for loop.
    For(ForStatement),

    /// A break statement.
    Break,

    /// A continue statement.
    Continue,

    /// A return statement.
    Return(Option<Expr>),

    /// A pass (no-op) statement.
    Pass,

    /// An expression statement (function call as statement).
    Expression(Expr),
}

/// A variable declaration.
#[derive(Debug, Clone)]
pub struct VarDecl {
    /// The variable name.
    pub name: String,
    /// The declared type (always required).
    pub var_type: Option<Type>,
    /// Optional initial value.
    pub initializer: Option<Expr>,
    /// Optional array size.
    pub array_size: Option<u16>,
    /// The source span.
    pub span: Span,
}

impl VarDecl {
    /// Create a new variable declaration with explicit type.
    pub fn new(name: String, var_type: Type, span: Span) -> Self {
        Self {
            name,
            var_type: Some(var_type),
            initializer: None,
            array_size: None,
            span,
        }
    }

    /// Create a new variable declaration with type inference.
    ///
    /// # Deprecated
    /// Type inference has been removed. Use `VarDecl::new()` with explicit type instead.
    #[deprecated(since = "0.2.0", note = "Type inference removed. Use VarDecl::new() with explicit type.")]
    pub fn new_inferred(name: String, span: Span) -> Self {
        Self {
            name,
            var_type: None,
            initializer: None,
            array_size: None,
            span,
        }
    }

    /// Add an initializer to this declaration.
    pub fn with_initializer(mut self, expr: Expr) -> Self {
        self.initializer = Some(expr);
        self
    }

    /// Make this an array declaration.
    pub fn with_array_size(mut self, size: u16) -> Self {
        self.array_size = Some(size);
        self
    }

    /// Check if this is an array declaration.
    pub fn is_array(&self) -> bool {
        self.array_size.is_some()
    }
}

/// A constant declaration.
#[derive(Debug, Clone)]
pub struct ConstDecl {
    /// The constant name.
    pub name: String,
    /// The explicit type (always required).
    pub const_type: Option<Type>,
    /// The constant value.
    pub value: Expr,
    /// The source span.
    pub span: Span,
}

impl ConstDecl {
    /// Create a new constant declaration with type inference.
    ///
    /// # Deprecated
    /// Type inference has been removed. Use `ConstDecl::new_typed()` with explicit type instead.
    #[deprecated(since = "0.2.0", note = "Type inference removed. Use ConstDecl::new_typed() with explicit type.")]
    pub fn new(name: String, value: Expr, span: Span) -> Self {
        Self {
            name,
            const_type: None,
            value,
            span,
        }
    }

    /// Create a new constant declaration with explicit type.
    pub fn new_typed(name: String, const_type: Type, value: Expr, span: Span) -> Self {
        Self {
            name,
            const_type: Some(const_type),
            value,
            span,
        }
    }
}

/// An assignment statement.
#[derive(Debug, Clone)]
pub struct Assignment {
    /// The target (variable name or array access).
    pub target: AssignTarget,
    /// The assignment operator.
    pub op: AssignOp,
    /// The value being assigned.
    pub value: Expr,
    /// The source span.
    pub span: Span,
}

/// The target of an assignment.
#[derive(Debug, Clone)]
pub enum AssignTarget {
    /// A simple variable.
    Variable(String),
    /// An array element.
    ArrayElement { name: String, index: Box<Expr> },
}

/// An assignment operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    /// Simple assignment `=`.
    Assign,
    /// Add and assign `+=`.
    AddAssign,
    /// Subtract and assign `-=`.
    SubAssign,
    /// Multiply and assign `*=`.
    MulAssign,
    /// Divide and assign `/=`.
    DivAssign,
    /// Modulo and assign `%=`.
    ModAssign,
    /// Bitwise AND and assign `&=`.
    BitAndAssign,
    /// Bitwise OR and assign `|=`.
    BitOrAssign,
    /// Bitwise XOR and assign `^=`.
    BitXorAssign,
    /// Left shift and assign `<<=`.
    ShiftLeftAssign,
    /// Right shift and assign `>>=`.
    ShiftRightAssign,
}

/// An if statement.
#[derive(Debug, Clone)]
pub struct IfStatement {
    /// The condition.
    pub condition: Expr,
    /// The then-block.
    pub then_block: Block,
    /// Optional elif branches.
    pub elif_branches: Vec<(Expr, Block)>,
    /// Optional else block.
    pub else_block: Option<Block>,
    /// The source span.
    pub span: Span,
}

/// A while loop.
#[derive(Debug, Clone)]
pub struct WhileStatement {
    /// The loop condition.
    pub condition: Expr,
    /// The loop body.
    pub body: Block,
    /// The source span.
    pub span: Span,
}

/// A for loop.
#[derive(Debug, Clone)]
pub struct ForStatement {
    /// The loop variable name.
    pub variable: String,
    /// The start value.
    pub start: Expr,
    /// The end value.
    pub end: Expr,
    /// Whether this is a downto loop (descending).
    pub descending: bool,
    /// The loop body.
    pub body: Block,
    /// The source span.
    pub span: Span,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// The function name.
    pub name: String,
    /// The function parameters.
    pub params: Vec<Parameter>,
    /// The return type (None for void).
    pub return_type: Option<Type>,
    /// The function body.
    pub body: Block,
    /// The source span.
    pub span: Span,
}

impl FunctionDef {
    /// Create a new function definition.
    pub fn new(name: String, params: Vec<Parameter>, body: Block, span: Span) -> Self {
        Self {
            name,
            params,
            return_type: None,
            body,
            span,
        }
    }

    /// Set the return type.
    pub fn with_return_type(mut self, return_type: Type) -> Self {
        self.return_type = Some(return_type);
        self
    }

    /// Check if this function returns a value.
    pub fn has_return_value(&self) -> bool {
        self.return_type.is_some()
    }
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The parameter name.
    pub name: String,
    /// The parameter type.
    pub param_type: Type,
    /// The source span.
    pub span: Span,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(name: String, param_type: Type, span: Span) -> Self {
        Self {
            name,
            param_type,
            span,
        }
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::fmt::Display for StatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementKind::VarDecl(decl) => write!(f, "{}", decl),
            StatementKind::ConstDecl(decl) => write!(f, "{}", decl),
            StatementKind::Assignment(assign) => write!(f, "{}", assign),
            StatementKind::If(if_stmt) => write!(f, "{}", if_stmt),
            StatementKind::While(while_stmt) => write!(f, "{}", while_stmt),
            StatementKind::For(for_stmt) => write!(f, "{}", for_stmt),
            StatementKind::Break => write!(f, "break"),
            StatementKind::Continue => write!(f, "continue"),
            StatementKind::Return(Some(expr)) => write!(f, "return {}", expr),
            StatementKind::Return(None) => write!(f, "return"),
            StatementKind::Pass => write!(f, "pass"),
            StatementKind::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

impl std::fmt::Display for VarDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref var_type) = self.var_type {
            write!(f, "{}: {}", self.name, var_type)?;
            if let Some(size) = self.array_size {
                write!(f, "[{}]", size)?;
            }
        } else {
            write!(f, "{}", self.name)?;
        }
        if let Some(init) = &self.initializer {
            write!(f, " = {}", init)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ConstDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref const_type) = self.const_type {
            write!(f, "const {}: {} = {}", self.name, const_type, self.value)
        } else {
            write!(f, "const {} = {}", self.name, self.value)
        }
    }
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.target, self.op, self.value)
    }
}

impl std::fmt::Display for AssignTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignTarget::Variable(name) => write!(f, "{}", name),
            AssignTarget::ArrayElement { name, index } => write!(f, "{}[{}]", name, index),
        }
    }
}

impl std::fmt::Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AssignOp::Assign => "=",
            AssignOp::AddAssign => "+=",
            AssignOp::SubAssign => "-=",
            AssignOp::MulAssign => "*=",
            AssignOp::DivAssign => "/=",
            AssignOp::ModAssign => "%=",
            AssignOp::BitAndAssign => "&=",
            AssignOp::BitOrAssign => "|=",
            AssignOp::BitXorAssign => "^=",
            AssignOp::ShiftLeftAssign => "<<=",
            AssignOp::ShiftRightAssign => ">>=",
        };
        write!(f, "{}", s)
    }
}

impl std::fmt::Display for IfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {}:", self.condition)?;
        for (cond, _block) in &self.elif_branches {
            write!(f, " elif {}:", cond)?;
        }
        if self.else_block.is_some() {
            write!(f, " else:")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "while {}:", self.condition)
    }
}

impl std::fmt::Display for ForStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = if self.descending { "downto" } else { "to" };
        write!(
            f,
            "for {} in {} {} {}:",
            self.variable, self.start, direction, self.end
        )
    }
}

impl std::fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "def {}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        write!(f, ")")?;
        if let Some(ret_type) = &self.return_type {
            write!(f, " -> {}", ret_type)?;
        }
        write!(f, ":")
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.param_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ExprKind;

    #[test]
    fn test_var_decl() {
        let span = Span::new(0, 10);
        let decl = VarDecl::new("x".to_string(), Type::Byte, span);
        assert_eq!(decl.name, "x");
        assert!(!decl.is_array());
    }

    #[test]
    fn test_function_def() {
        let span = Span::new(0, 50);
        let body = Block::empty(span.clone());
        let func = FunctionDef::new("main".to_string(), vec![], body, span);
        assert_eq!(func.name, "main");
        assert!(!func.has_return_value());
    }

    #[test]
    fn test_var_decl_with_initializer() {
        let span = Span::new(0, 10);
        let init = Expr::new(ExprKind::IntegerLiteral(42), Span::new(8, 10));
        let decl = VarDecl::new("x".to_string(), Type::Byte, span).with_initializer(init);
        assert!(decl.initializer.is_some());
    }

    #[test]
    fn test_var_decl_array() {
        let span = Span::new(0, 15);
        let decl = VarDecl::new("arr".to_string(), Type::Byte, span).with_array_size(10);
        assert!(decl.is_array());
        assert_eq!(decl.array_size, Some(10));
    }

    #[test]
    fn test_function_with_return_type() {
        let span = Span::new(0, 50);
        let body = Block::empty(span.clone());
        let func = FunctionDef::new("get_value".to_string(), vec![], body, span)
            .with_return_type(Type::Byte);
        assert!(func.has_return_value());
        assert_eq!(func.return_type, Some(Type::Byte));
    }

    #[test]
    fn test_display_var_decl() {
        let span = Span::new(0, 10);
        let decl = VarDecl::new("x".to_string(), Type::Byte, span);
        assert_eq!(format!("{}", decl), "x: byte");
    }

    #[test]
    fn test_display_var_decl_with_init() {
        let span = Span::new(0, 10);
        let init = Expr::new(ExprKind::IntegerLiteral(42), Span::new(8, 10));
        let decl = VarDecl::new("x".to_string(), Type::Byte, span).with_initializer(init);
        assert_eq!(format!("{}", decl), "x: byte = 42");
    }

    #[test]
    #[allow(deprecated)]
    fn test_display_var_decl_inferred() {
        // This test uses deprecated API - kept for backwards compatibility testing
        let span = Span::new(0, 10);
        let init = Expr::new(ExprKind::IntegerLiteral(42), Span::new(4, 6));
        let decl = VarDecl::new_inferred("x".to_string(), span).with_initializer(init);
        assert_eq!(format!("{}", decl), "x = 42");
    }

    #[test]
    #[allow(deprecated)]
    fn test_display_const_decl() {
        // This test uses deprecated API - kept for backwards compatibility testing
        let span = Span::new(0, 15);
        let value = Expr::new(ExprKind::IntegerLiteral(100), Span::new(12, 15));
        let decl = ConstDecl::new("MAX".to_string(), value, span);
        assert_eq!(format!("{}", decl), "const MAX = 100");
    }

    #[test]
    fn test_display_const_decl_typed() {
        let span = Span::new(0, 20);
        let value = Expr::new(ExprKind::IntegerLiteral(255), Span::new(16, 19));
        let decl = ConstDecl::new_typed("MAX".to_string(), Type::Word, value, span);
        assert_eq!(format!("{}", decl), "const MAX: word = 255");
    }

    #[test]
    fn test_display_assignment() {
        let span = Span::new(0, 10);
        let value = Expr::new(ExprKind::IntegerLiteral(5), Span::new(4, 5));
        let assign = Assignment {
            target: AssignTarget::Variable("x".to_string()),
            op: AssignOp::Assign,
            value,
            span,
        };
        assert_eq!(format!("{}", assign), "x = 5");
    }

    #[test]
    fn test_display_compound_assignment() {
        let span = Span::new(0, 10);
        let value = Expr::new(ExprKind::IntegerLiteral(1), Span::new(5, 6));
        let assign = Assignment {
            target: AssignTarget::Variable("x".to_string()),
            op: AssignOp::AddAssign,
            value,
            span,
        };
        assert_eq!(format!("{}", assign), "x += 1");
    }

    #[test]
    fn test_display_array_assignment() {
        let span = Span::new(0, 15);
        let index = Box::new(Expr::new(ExprKind::IntegerLiteral(0), Span::new(4, 5)));
        let value = Expr::new(ExprKind::IntegerLiteral(42), Span::new(10, 12));
        let assign = Assignment {
            target: AssignTarget::ArrayElement {
                name: "arr".to_string(),
                index,
            },
            op: AssignOp::Assign,
            value,
            span,
        };
        assert_eq!(format!("{}", assign), "arr[0] = 42");
    }

    #[test]
    fn test_display_if_statement() {
        let span = Span::new(0, 20);
        let cond = Expr::new(ExprKind::BoolLiteral(true), Span::new(3, 7));
        let body = Block::empty(Span::new(8, 20));
        let if_stmt = IfStatement {
            condition: cond,
            then_block: body,
            elif_branches: vec![],
            else_block: None,
            span,
        };
        assert_eq!(format!("{}", if_stmt), "if true:");
    }

    #[test]
    fn test_display_while_statement() {
        let span = Span::new(0, 20);
        let cond = Expr::new(ExprKind::BoolLiteral(true), Span::new(6, 10));
        let body = Block::empty(Span::new(11, 20));
        let while_stmt = WhileStatement {
            condition: cond,
            body,
            span,
        };
        assert_eq!(format!("{}", while_stmt), "while true:");
    }

    #[test]
    fn test_display_for_statement() {
        let span = Span::new(0, 25);
        let start = Expr::new(ExprKind::IntegerLiteral(0), Span::new(9, 10));
        let end = Expr::new(ExprKind::IntegerLiteral(10), Span::new(14, 16));
        let body = Block::empty(Span::new(17, 25));
        let for_stmt = ForStatement {
            variable: "i".to_string(),
            start,
            end,
            descending: false,
            body,
            span,
        };
        assert_eq!(format!("{}", for_stmt), "for i in 0 to 10:");
    }

    #[test]
    fn test_display_for_downto() {
        let span = Span::new(0, 25);
        let start = Expr::new(ExprKind::IntegerLiteral(10), Span::new(9, 11));
        let end = Expr::new(ExprKind::IntegerLiteral(0), Span::new(20, 21));
        let body = Block::empty(Span::new(22, 25));
        let for_stmt = ForStatement {
            variable: "i".to_string(),
            start,
            end,
            descending: true,
            body,
            span,
        };
        assert_eq!(format!("{}", for_stmt), "for i in 10 downto 0:");
    }

    #[test]
    fn test_display_function_def() {
        let span = Span::new(0, 30);
        let body = Block::empty(Span::new(10, 30));
        let func = FunctionDef::new("main".to_string(), vec![], body, span);
        assert_eq!(format!("{}", func), "def main():");
    }

    #[test]
    fn test_display_function_with_params() {
        let span = Span::new(0, 40);
        let body = Block::empty(Span::new(25, 40));
        let params = vec![
            Parameter::new("x".to_string(), Type::Byte, Span::new(8, 15)),
            Parameter::new("y".to_string(), Type::Word, Span::new(17, 24)),
        ];
        let func = FunctionDef::new("add".to_string(), params, body, span);
        assert_eq!(format!("{}", func), "def add(x: byte, y: word):");
    }

    #[test]
    fn test_display_function_with_return() {
        let span = Span::new(0, 45);
        let body = Block::empty(Span::new(30, 45));
        let func =
            FunctionDef::new("get".to_string(), vec![], body, span).with_return_type(Type::Word);
        assert_eq!(format!("{}", func), "def get() -> word:");
    }

    #[test]
    fn test_display_parameter() {
        let param = Parameter::new("value".to_string(), Type::Byte, Span::new(0, 11));
        assert_eq!(format!("{}", param), "value: byte");
    }

    #[test]
    fn test_all_assign_ops_display() {
        let ops = [
            (AssignOp::Assign, "="),
            (AssignOp::AddAssign, "+="),
            (AssignOp::SubAssign, "-="),
            (AssignOp::MulAssign, "*="),
            (AssignOp::DivAssign, "/="),
            (AssignOp::ModAssign, "%="),
            (AssignOp::BitAndAssign, "&="),
            (AssignOp::BitOrAssign, "|="),
            (AssignOp::BitXorAssign, "^="),
            (AssignOp::ShiftLeftAssign, "<<="),
            (AssignOp::ShiftRightAssign, ">>="),
        ];
        for (op, expected) in &ops {
            assert_eq!(format!("{}", op), *expected);
        }
    }

    #[test]
    fn test_statement_kinds_display() {
        let span = Span::new(0, 10);

        // Break
        let stmt = Statement::new(StatementKind::Break, span.clone());
        assert_eq!(format!("{}", stmt), "break");

        // Continue
        let stmt = Statement::new(StatementKind::Continue, span.clone());
        assert_eq!(format!("{}", stmt), "continue");

        // Pass
        let stmt = Statement::new(StatementKind::Pass, span.clone());
        assert_eq!(format!("{}", stmt), "pass");

        // Return without value
        let stmt = Statement::new(StatementKind::Return(None), span.clone());
        assert_eq!(format!("{}", stmt), "return");

        // Return with value
        let expr = Expr::new(ExprKind::IntegerLiteral(42), Span::new(7, 9));
        let stmt = Statement::new(StatementKind::Return(Some(expr)), span);
        assert_eq!(format!("{}", stmt), "return 42");
    }
}
