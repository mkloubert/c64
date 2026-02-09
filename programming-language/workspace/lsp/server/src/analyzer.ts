/*
Cobra64 - A concept for a modern Python-like compiler creating C64 binaries

Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

import {
    Program,
    FunctionDef,
    VarDecl,
    ConstDecl,
    DataBlockDef,
    Statement,
    Expression,
    TopLevelItem,
} from './parser';
import {
    Span,
    CompilerDiagnostic,
    DiagnosticSeverity,
    COBRA64_BUILTINS,
    SymbolKind,
    Symbol as Cobra64Symbol,
} from '../../shared/types';

/**
 * Symbol information for the analyzer.
 */
interface SymbolInfo {
    name: string;
    kind: SymbolKind;
    type: string;
    span: Span;
    definitionSpan: Span;
    isConstant: boolean;
    value?: unknown;
}

/**
 * Data block information.
 */
interface DataBlockInfo {
    name: string;
    byteCount: number;
    hasInclude: boolean;
    span: Span;
}

/**
 * Function signature information.
 */
interface FunctionInfo {
    name: string;
    params: { name: string; type: string }[];
    returnType: string | null;
    span: Span;
    isBuiltin: boolean;
}

/**
 * Scope for variable lookups.
 */
class Scope {
    private symbols: Map<string, SymbolInfo> = new Map();
    public parent: Scope | null;

    constructor(parent: Scope | null = null) {
        this.parent = parent;
    }

    define(symbol: SymbolInfo): void {
        this.symbols.set(symbol.name, symbol);
    }

    lookup(name: string): SymbolInfo | null {
        const symbol = this.symbols.get(name);
        if (symbol) return symbol;
        if (this.parent) return this.parent.lookup(name);
        return null;
    }

    lookupLocal(name: string): SymbolInfo | null {
        return this.symbols.get(name) || null;
    }

    getAllSymbols(): SymbolInfo[] {
        return Array.from(this.symbols.values());
    }
}

/**
 * Analyzer result.
 */
export interface AnalyzerResult {
    diagnostics: CompilerDiagnostic[];
    symbols: Cobra64Symbol[];
    functions: FunctionInfo[];
}

/**
 * Semantic analyzer for Cobra64.
 */
export class Analyzer {
    private diagnostics: CompilerDiagnostic[] = [];
    private globalScope: Scope = new Scope();
    private currentScope: Scope = this.globalScope;
    private functions: Map<string, FunctionInfo> = new Map();
    private dataBlocks: Map<string, DataBlockInfo> = new Map();
    private allSymbols: Cobra64Symbol[] = [];
    private inLoop: number = 0;
    private currentFunction: FunctionInfo | null = null;
    private hasMain: boolean = false;

    constructor() {
        this.registerBuiltins();
    }

    /**
     * Analyze the program AST.
     */
    analyze(program: Program): AnalyzerResult {
        // First pass: collect all top-level declarations
        this.collectDeclarations(program);

        // Second pass: analyze all items
        for (const item of program.items) {
            this.analyzeTopLevelItem(item);
        }

        // Check for main function
        if (!this.hasMain) {
            this.addError(program.span, 'E204', "No 'main()' function defined");
        }

        return {
            diagnostics: this.diagnostics,
            symbols: this.allSymbols,
            functions: Array.from(this.functions.values()),
        };
    }

    private registerBuiltins(): void {
        for (const builtin of COBRA64_BUILTINS) {
            this.functions.set(builtin.name, {
                name: builtin.name,
                params: builtin.parameters.map(p => ({ name: p.name, type: p.type })),
                returnType: builtin.returnType,
                span: { start: 0, end: 0 },
                isBuiltin: true,
            });
        }
    }

    private collectDeclarations(program: Program): void {
        for (const item of program.items) {
            if (item.kind === 'FunctionDef') {
                if (this.functions.has(item.name) && !this.functions.get(item.name)!.isBuiltin) {
                    this.addError(item.span, 'E203', `Function '${item.name}' is already defined`);
                } else {
                    this.functions.set(item.name, {
                        name: item.name,
                        params: item.params.map(p => ({ name: p.name, type: p.type })),
                        returnType: item.returnType,
                        span: item.span,
                        isBuiltin: false,
                    });

                    if (item.name === 'main') {
                        this.hasMain = true;
                    }
                }
            } else if (item.kind === 'DataBlockDef') {
                this.collectDataBlock(item);
            } else if (item.kind === 'VarDecl' || item.kind === 'ConstDecl') {
                if (this.globalScope.lookupLocal(item.name)) {
                    this.addError(item.span, 'E203', `'${item.name}' is already defined`);
                } else {
                    const type = item.type || this.inferType(item.kind === 'ConstDecl' ? item.value : item.initializer);
                    this.globalScope.define({
                        name: item.name,
                        kind: item.kind === 'ConstDecl' ? SymbolKind.Constant : SymbolKind.Variable,
                        type,
                        span: item.span,
                        definitionSpan: item.span,
                        isConstant: item.kind === 'ConstDecl',
                    });

                    this.allSymbols.push({
                        name: item.name,
                        kind: item.kind === 'ConstDecl' ? SymbolKind.Constant : SymbolKind.Variable,
                        type,
                        span: item.span,
                        definitionSpan: item.span,
                    });
                }
            }
        }
    }

    private collectDataBlock(block: DataBlockDef): void {
        // Check for duplicate names
        if (this.dataBlocks.has(block.name)) {
            this.addError(block.span, 'E203', `Data block '${block.name}' is already defined`);
            return;
        }
        if (this.globalScope.lookupLocal(block.name)) {
            this.addError(block.span, 'E203', `'${block.name}' is already defined as a variable or constant`);
            return;
        }

        // Calculate byte count from entries
        let byteCount = 0;
        let hasInclude = false;

        for (const entry of block.entries) {
            if (entry.kind === 'DataEntryBytes') {
                byteCount += entry.values.length;
            } else if (entry.kind === 'DataEntryInclude') {
                hasInclude = true;
                // We can't know the size without reading the file
            }
        }

        this.dataBlocks.set(block.name, {
            name: block.name,
            byteCount,
            hasInclude,
            span: block.span,
        });

        // Register as a constant (data blocks have word address type)
        this.globalScope.define({
            name: block.name,
            kind: SymbolKind.DataBlock,
            type: 'word',
            span: block.span,
            definitionSpan: block.span,
            isConstant: true,
        });

        this.allSymbols.push({
            name: block.name,
            kind: SymbolKind.DataBlock,
            type: 'word',
            span: block.span,
            definitionSpan: block.span,
        });
    }

    private analyzeTopLevelItem(item: TopLevelItem): void {
        switch (item.kind) {
            case 'FunctionDef':
                this.analyzeFunction(item);
                break;
            case 'DataBlockDef':
                // Data blocks are already collected, nothing more to analyze
                break;
            case 'VarDecl':
                if (item.initializer) {
                    this.analyzeExpression(item.initializer);
                }
                break;
            case 'ConstDecl':
                this.analyzeExpression(item.value);
                break;
        }
    }

    private analyzeFunction(func: FunctionDef): void {
        const funcInfo = this.functions.get(func.name);
        if (!funcInfo) return;

        this.currentFunction = funcInfo;

        // Create function scope
        const functionScope = new Scope(this.globalScope);
        this.currentScope = functionScope;

        // Add parameters to scope
        for (const param of func.params) {
            if (functionScope.lookupLocal(param.name)) {
                this.addError(param.span, 'E203', `Parameter '${param.name}' is already defined`);
            } else {
                functionScope.define({
                    name: param.name,
                    kind: SymbolKind.Parameter,
                    type: param.type,
                    span: param.span,
                    definitionSpan: param.span,
                    isConstant: false,
                });
            }
        }

        // Add function symbol
        this.allSymbols.push({
            name: func.name,
            kind: SymbolKind.Function,
            type: this.formatFunctionType(funcInfo),
            span: func.span,
            definitionSpan: func.span,
            children: func.params.map(p => ({
                name: p.name,
                kind: SymbolKind.Parameter,
                type: p.type,
                span: p.span,
                definitionSpan: p.span,
            })),
        });

        // Analyze body
        for (const stmt of func.body) {
            this.analyzeStatement(stmt);
        }

        this.currentScope = this.globalScope;
        this.currentFunction = null;
    }

    private analyzeStatement(stmt: Statement): void {
        switch (stmt.kind) {
            case 'VarDecl':
                this.analyzeVarDecl(stmt);
                break;

            case 'Assignment':
                this.analyzeAssignment(stmt);
                break;

            case 'IfStatement':
                this.analyzeExpression(stmt.condition);
                for (const s of stmt.thenBranch) {
                    this.analyzeStatement(s);
                }
                for (const elif of stmt.elifBranches) {
                    this.analyzeExpression(elif.condition);
                    for (const s of elif.body) {
                        this.analyzeStatement(s);
                    }
                }
                if (stmt.elseBranch) {
                    for (const s of stmt.elseBranch) {
                        this.analyzeStatement(s);
                    }
                }
                break;

            case 'WhileStatement':
                this.analyzeExpression(stmt.condition);
                this.inLoop++;
                for (const s of stmt.body) {
                    this.analyzeStatement(s);
                }
                this.inLoop--;
                break;

            case 'ForStatement':
                this.analyzeExpression(stmt.start);
                this.analyzeExpression(stmt.end);

                // Create scope for loop variable
                const loopScope = new Scope(this.currentScope);
                loopScope.define({
                    name: stmt.variable,
                    kind: SymbolKind.Variable,
                    type: 'byte', // For loop variables are typically byte
                    span: stmt.span,
                    definitionSpan: stmt.span,
                    isConstant: false,
                });

                const prevScope = this.currentScope;
                this.currentScope = loopScope;
                this.inLoop++;

                for (const s of stmt.body) {
                    this.analyzeStatement(s);
                }

                this.inLoop--;
                this.currentScope = prevScope;
                break;

            case 'ReturnStatement':
                if (stmt.value) {
                    this.analyzeExpression(stmt.value);
                }
                break;

            case 'BreakStatement':
                if (this.inLoop === 0) {
                    this.addError(stmt.span, 'E205', "'break' can only be used inside a loop");
                }
                break;

            case 'ContinueStatement':
                if (this.inLoop === 0) {
                    this.addError(stmt.span, 'E205', "'continue' can only be used inside a loop");
                }
                break;

            case 'PassStatement':
                // Nothing to analyze
                break;

            case 'ExpressionStatement':
                this.analyzeExpression(stmt.expression);
                break;
        }
    }

    private analyzeVarDecl(decl: VarDecl): void {
        // Check for redefinition in current scope
        if (this.currentScope.lookupLocal(decl.name)) {
            this.addError(decl.span, 'E203', `Variable '${decl.name}' is already defined in this scope`);
            return;
        }

        // Analyze initializer
        if (decl.initializer) {
            this.analyzeExpression(decl.initializer);
        }

        // Determine type
        const type = decl.type || this.inferType(decl.initializer);

        // Check that local vars in functions have types
        if (!decl.type && !decl.initializer && this.currentScope !== this.globalScope) {
            this.addError(decl.span, 'E200', 'Local variables must have a type annotation or initializer');
        }

        this.currentScope.define({
            name: decl.name,
            kind: SymbolKind.Variable,
            type,
            span: decl.span,
            definitionSpan: decl.span,
            isConstant: false,
        });
    }

    private analyzeAssignment(stmt: { target: string; index: Expression | null; value: Expression; span: Span }): void {
        // Check if target exists
        const symbol = this.currentScope.lookup(stmt.target);
        if (!symbol) {
            this.addError(stmt.span, 'E200', `Undefined variable '${stmt.target}'`);
        } else if (symbol.isConstant) {
            this.addError(stmt.span, 'E201', `Cannot assign to constant '${stmt.target}'`);
        }

        if (stmt.index) {
            this.analyzeExpression(stmt.index);
        }

        this.analyzeExpression(stmt.value);
    }

    private analyzeExpression(expr: Expression): string {
        switch (expr.kind) {
            case 'IntegerLiteral':
                return this.inferIntegerType(expr.value);

            case 'DecimalLiteral':
                return 'float';

            case 'StringLiteral':
                return 'string';

            case 'CharLiteral':
                return 'byte';

            case 'BoolLiteral':
                return 'bool';

            case 'Identifier': {
                const symbol = this.currentScope.lookup(expr.name);
                if (!symbol) {
                    this.addError(expr.span, 'E200', `Undefined variable '${expr.name}'`);
                    return 'byte';
                }
                return symbol.type;
            }

            case 'ArrayLiteral': {
                if (expr.elements.length === 0) {
                    return 'byte[]';
                }
                const elementType = this.analyzeExpression(expr.elements[0]);
                for (let i = 1; i < expr.elements.length; i++) {
                    this.analyzeExpression(expr.elements[i]);
                }
                return `${elementType}[]`;
            }

            case 'ArrayIndex': {
                const arrayType = this.analyzeExpression(expr.array);
                this.analyzeExpression(expr.index);
                if (arrayType.endsWith('[]')) {
                    return arrayType.slice(0, -2);
                }
                this.addError(expr.span, 'E202', 'Cannot index non-array type');
                return 'byte';
            }

            case 'FunctionCall': {
                const func = this.functions.get(expr.name);
                if (!func) {
                    this.addError(expr.span, 'E201', `Undefined function '${expr.name}'`);
                    return 'byte';
                }

                // Check argument count
                if (expr.args.length !== func.params.length) {
                    this.addError(expr.span, 'E207',
                        `Function '${expr.name}' expects ${func.params.length} arguments, got ${expr.args.length}`);
                }

                // Analyze arguments
                for (const arg of expr.args) {
                    this.analyzeExpression(arg);
                }

                return func.returnType || 'void';
            }

            case 'UnaryOp': {
                const operandType = this.analyzeExpression(expr.operand);
                if (expr.operator === 'not') {
                    return 'bool';
                }
                return operandType;
            }

            case 'BinaryOp': {
                const leftType = this.analyzeExpression(expr.left);
                const rightType = this.analyzeExpression(expr.right);

                // Comparison operators always return bool
                if (['==', '!=', '<', '>', '<=', '>='].includes(expr.operator)) {
                    return 'bool';
                }

                // Logical operators always return bool
                if (['and', 'or'].includes(expr.operator)) {
                    return 'bool';
                }

                // Type promotion for arithmetic
                return this.promoteTypes(leftType, rightType);
            }

            case 'TypeCast': {
                this.analyzeExpression(expr.expression);
                return expr.targetType;
            }

            default:
                return 'byte';
        }
    }

    private inferType(expr: Expression | null): string {
        if (!expr) return 'byte';
        return this.analyzeExpression(expr);
    }

    private inferIntegerType(value: number): string {
        if (value >= 0 && value <= 255) return 'byte';
        if (value >= 0 && value <= 65535) return 'word';
        if (value >= -128 && value < 0) return 'sbyte';
        if (value >= -32768 && value < -128) return 'sword';
        return 'word';
    }

    private promoteTypes(left: string, right: string): string {
        // Float is highest
        if (left === 'float' || right === 'float') return 'float';

        // Then fixed
        if (left === 'fixed' || right === 'fixed') return 'fixed';

        // Then sword
        if (left === 'sword' || right === 'sword') return 'sword';

        // Then word
        if (left === 'word' || right === 'word') return 'word';

        // Then sbyte
        if (left === 'sbyte' || right === 'sbyte') return 'sbyte';

        // Default to byte
        return 'byte';
    }

    private formatFunctionType(func: FunctionInfo): string {
        const params = func.params.map(p => `${p.name}: ${p.type}`).join(', ');
        const ret = func.returnType ? ` -> ${func.returnType}` : '';
        return `(${params})${ret}`;
    }

    private addError(span: Span, code: string, message: string): void {
        this.diagnostics.push({
            code,
            message,
            span,
            severity: DiagnosticSeverity.Error,
        });
    }
}

/**
 * Analyze a program AST.
 */
export function analyze(program: Program): AnalyzerResult {
    const analyzer = new Analyzer();
    return analyzer.analyze(program);
}
