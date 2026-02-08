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
//!
//! # Module Structure
//!
//! - `builtins` - Built-in function registration (BuiltinRegistry trait)
//! - `context` - Analysis context for tracking state
//! - `control_flow` - Control flow analysis (ControlFlowAnalyzer trait)
//! - `expressions` - Expression analysis (ExpressionAnalyzer trait)
//! - `functions` - Function analysis (FunctionAnalyzer trait)
//! - `operators` - Operator type checking (OperatorChecker trait)
//! - `scope` - Scope management for lexical scoping
//! - `statements` - Statement analysis (StatementAnalyzer trait)
//! - `symbol` - Symbol and symbol type definitions
//! - `symbol_table` - Symbol table with nested scope support
//! - `type_check` - Type inference and checking utilities (TypeChecker trait)

// Submodules
pub mod builtins;
pub mod context;
pub mod control_flow;
pub mod expressions;
pub mod functions;
pub mod operators;
pub mod scope;
pub mod statements;
pub mod symbol;
pub mod symbol_table;
pub mod type_check;

// Re-exports for public API
pub use context::AnalysisContext;
pub use scope::Scope;
pub use symbol::{Symbol, SymbolType};
pub use symbol_table::SymbolTable;

// Internal imports from submodules
use builtins::BuiltinRegistry;
use functions::FunctionAnalyzer;
use statements::StatementAnalyzer;

use crate::ast::{Block, Program, TopLevelItem, Type};
use crate::error::{CompileError, ErrorCode, Span};

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

    /// Analyze a block of statements.
    fn analyze_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.analyze_statement(stmt);
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
        let result = analyze_source("const MAX: byte = 100\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_constant_in_function() {
        let result = analyze_source("def main():\n    const LOCAL: word = 500");
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
        let result = analyze_source("const X: byte = 5\ndef main():\n    X = 10");
        assert!(has_error_code(&result, ErrorCode::CannotAssignToConstant));
    }

    #[test]
    fn test_error_assign_to_local_constant() {
        let result = analyze_source("def main():\n    const Y: word = 100\n    Y = 200");
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

    #[test]
    fn test_array_literal_byte_inference() {
        // All values 0-255 should infer byte[]
        let result = analyze_source("def main():\n    arr: byte[] = [1, 2, 255]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_literal_word_inference() {
        // Any value > 255 should infer word[]
        let result = analyze_source("def main():\n    arr: word[] = [1, 1000, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_literal_bool_inference() {
        // All booleans should infer bool[]
        let result = analyze_source("def main():\n    flags: bool[] = [true, false, true]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_array_literal_empty() {
        // Empty array should require type annotation
        let result = analyze_source("def main():\n    arr = []");
        assert!(has_error_code(&result, ErrorCode::CannotInferArrayType));
    }

    #[test]
    fn test_error_array_literal_mixed_types() {
        // Mixing bools and integers should fail
        let result = analyze_source("def main():\n    arr: byte[] = [1, true]");
        assert!(has_error_code(&result, ErrorCode::ArrayElementTypeMismatch));
    }

    #[test]
    fn test_error_array_literal_float_element() {
        // Float elements should fail
        let result = analyze_source("def main():\n    arr: byte[] = [1.5, 2.0]");
        assert!(has_error_code(&result, ErrorCode::ArrayElementTypeMismatch));
    }

    #[test]
    fn test_error_array_literal_string_element() {
        // String elements should fail
        let result = analyze_source("def main():\n    arr: byte[] = [\"hello\"]");
        assert!(has_error_code(&result, ErrorCode::ArrayElementTypeMismatch));
    }

    #[test]
    fn test_error_array_size_too_many() {
        // Array literal with more elements than declared
        let result = analyze_source("def main():\n    arr: byte[2] = [1, 2, 3]");
        assert!(has_error_code(&result, ErrorCode::ArrayInitTooManyElements));
    }

    #[test]
    fn test_error_array_size_too_few() {
        // Array literal with fewer elements than declared
        let result = analyze_source("def main():\n    arr: byte[5] = [1, 2]");
        assert!(has_error_code(&result, ErrorCode::ArrayInitTooFewElements));
    }

    #[test]
    fn test_array_size_exact_match() {
        // Array literal size matches declaration
        let result = analyze_source("def main():\n    arr: byte[3] = [1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_array_byte_value_out_of_range() {
        // Value > 255 in byte array
        let result = analyze_source("def main():\n    arr: byte[2] = [1, 1000]");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_array_negative_value_in_byte_array() {
        // Negative values in explicitly typed byte[] should fail
        let result = analyze_source("def main():\n    arr: byte[] = [-1, 2]");
        // Type mismatch: inferred sbyte[] cannot be assigned to byte[]
        assert!(result.is_err());
    }

    #[test]
    fn test_array_word_large_values() {
        // Word array with values > 255
        let result = analyze_source("def main():\n    arr: word[3] = [1000, 2000, 65535]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_sbyte_inference() {
        // Negative values in sbyte range should infer sbyte[]
        let result = analyze_source("def main():\n    arr: sbyte[] = [-50, 0, 100]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_sword_inference() {
        // Negative values outside sbyte range should infer sword[]
        let result = analyze_source("def main():\n    arr: sword[] = [-1000, 500]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_sbyte_sized() {
        // Sized sbyte array
        let result = analyze_source("def main():\n    arr: sbyte[5]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_sword_sized() {
        // Sized sword array
        let result = analyze_source("def main():\n    arr: sword[10]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_sbyte_array_value_out_of_range() {
        // Value outside sbyte range (-128 to 127)
        let result = analyze_source("def main():\n    arr: sbyte[2] = [-129, 0]");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
    }

    #[test]
    fn test_error_sword_array_value_out_of_range() {
        // Value outside sword range
        let result = analyze_source("def main():\n    arr: sword[2] = [-40000, 0]");
        assert!(has_error_code(&result, ErrorCode::ConstantValueOutOfRange));
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
        let result = analyze_source("MIN: sbyte = -128\ndef main():\n    pass");
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

    // ========================================
    // Explicit Type Annotation Tests
    // ========================================

    #[test]
    fn test_explicit_type_var_byte() {
        // Variable with explicit byte type
        let result = analyze_source("x: byte = 10\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_byte_max() {
        // Variable with explicit byte type at max value
        let result = analyze_source("x: byte = 255\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_word() {
        // Variable with explicit word type
        let result = analyze_source("x: word = 1000\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_word_boundary() {
        // Variable with explicit word type at boundary value
        let result = analyze_source("x: word = 256\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_negative_sbyte() {
        // Variable with explicit sbyte type for negative value
        let result = analyze_source("x: sbyte = -50\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_negative_sword() {
        // Variable with explicit sword type for large negative value
        let result = analyze_source("x: sword = -1000\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_float() {
        // Variable with explicit float type for decimal literal
        let result = analyze_source("x: float = 3.14\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_bool() {
        // Variable with explicit bool type
        let result = analyze_source("x: bool = true\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_string() {
        // Variable with explicit string type
        let result = analyze_source("x: string = \"hello\"\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_const_word() {
        // Constant with explicit type
        let result = analyze_source("MAX: word = 255\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_const_fixed() {
        // Constant with explicit fixed type
        let result = analyze_source("PI: fixed = 3.14\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_const_float() {
        // Constant with explicit float type
        let result = analyze_source("E: float = 2.718\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_const_byte() {
        // Constant with explicit byte type
        let result = analyze_source("MIN: byte = 0\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_var_use_in_expression() {
        // Use an explicitly typed variable in an expression
        let result = analyze_source("x: byte = 10\ndef main():\n    y: byte = x + 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_const_use_in_expression() {
        // Use a constant with explicit type in an expression
        let result = analyze_source("MAX: word = 1000\ndef main():\n    y: word = MAX + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_mixed_declarations() {
        // Multiple declarations with explicit types
        let result = analyze_source(
            "MAX: word = 255\ncount: byte = 0\nPI: fixed = 3.14\nE: float = 2.718\ndef main():\n    pass",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_type_error_without_type() {
        // Variable without type annotation should fail at parser level
        let result = analyze_source("x\ndef main():\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_explicit_type_const_type_mismatch() {
        // Constant with explicit type that doesn't match value type
        let result = analyze_source("PI: byte = 3.14\ndef main():\n    pass");
        assert!(result.is_err());
    }

    #[test]
    fn test_explicit_type_negative_decimal() {
        // Negative decimal with explicit float type
        let result = analyze_source("x: float = -3.14\ndef main():\n    pass");
        assert!(result.is_ok());
    }

    // ========================================================================
    // len() function tests
    // ========================================================================

    #[test]
    fn test_len_byte_array() {
        let result =
            analyze_source("def main():\n    arr: byte[] = [1, 2, 3]\n    x: word = len(arr)");
        assert!(result.is_ok(), "len() on byte array should work");
    }

    #[test]
    fn test_len_word_array() {
        let result =
            analyze_source("def main():\n    arr: word[] = [1000, 2000]\n    x: word = len(arr)");
        assert!(result.is_ok(), "len() on word array should work");
    }

    #[test]
    fn test_len_bool_array() {
        let result = analyze_source(
            "def main():\n    flags: bool[] = [true, false]\n    x: word = len(flags)",
        );
        assert!(result.is_ok(), "len() on bool array should work");
    }

    #[test]
    fn test_len_sbyte_array() {
        let result =
            analyze_source("def main():\n    arr: sbyte[] = [-10, 20]\n    x: word = len(arr)");
        assert!(result.is_ok(), "len() on sbyte array should work");
    }

    #[test]
    fn test_len_sword_array() {
        let result =
            analyze_source("def main():\n    arr: sword[] = [-1000, 500]\n    x: word = len(arr)");
        assert!(result.is_ok(), "len() on sword array should work");
    }

    #[test]
    fn test_len_sized_array() {
        let result =
            analyze_source("def main():\n    buffer: byte[100]\n    x: word = len(buffer)");
        assert!(result.is_ok(), "len() on sized array should work");
    }

    #[test]
    fn test_len_non_array_error() {
        let result = analyze_source("def main():\n    x: byte = 5\n    y: word = len(x)");
        assert!(result.is_err(), "len() on non-array should fail");
    }

    #[test]
    fn test_len_wrong_arg_count_zero() {
        let result = analyze_source("def main():\n    x: word = len()");
        assert!(result.is_err(), "len() with no arguments should fail");
    }

    #[test]
    fn test_len_wrong_arg_count_two() {
        let result = analyze_source(
            "def main():\n    a: byte[] = [1, 2]\n    b: byte[] = [3, 4]\n    x: word = len(a, b)",
        );
        assert!(result.is_err(), "len() with two arguments should fail");
    }
}
