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
    InlayHint,
    InlayHintKind,
    Position,
    Range,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { Program, VarDecl, FunctionDef, Expression } from './parser';
import { AnalyzerResult } from './analyzer';
import {
    COBRA64_BUILTINS,
    offsetToPosition,
} from '../../shared/types';

/**
 * Get inlay hints for a document.
 */
export function getInlayHints(
    document: TextDocument,
    range: Range,
    program: Program | null,
    analyzerResult: AnalyzerResult | null
): InlayHint[] {
    if (!program) return [];

    const hints: InlayHint[] = [];
    const text = document.getText();

    // Build function map for parameter hints
    const functionMap = new Map<string, { params: { name: string; type: string }[] }>();

    if (analyzerResult) {
        for (const func of analyzerResult.functions) {
            functionMap.set(func.name, { params: func.params });
        }
    }

    // Also add built-ins
    for (const builtin of COBRA64_BUILTINS) {
        functionMap.set(builtin.name, {
            params: builtin.parameters.map(p => ({ name: p.name, type: p.type })),
        });
    }

    // Process all items
    for (const item of program.items) {
        if (item.kind === 'VarDecl') {
            // Add type hint for inferred variables
            addVarDeclTypeHint(hints, item, text);
        } else if (item.kind === 'FunctionDef') {
            // Process function body for hints
            addFunctionHints(hints, item, text, functionMap);
        }
    }

    return hints;
}

/**
 * Add type hint for variable declarations without explicit type.
 *
 * Note: Since type inference was removed in v0.6.0, all declarations
 * now require explicit types. This function will rarely match, but is
 * kept for backwards compatibility with files that have parse errors.
 */
function addVarDeclTypeHint(hints: InlayHint[], decl: VarDecl, text: string): void {
    // Only add hint if type is missing (parser error case)
    if (decl.type) return;
    if (!decl.initializer) return;

    // Infer type from initializer for the hint
    const inferredType = inferExpressionType(decl.initializer);
    if (!inferredType) return;

    // Find position after variable name
    const pos = offsetToPosition(text, decl.span.start);

    // Find the '=' sign position to place hint before it
    let nameEnd = decl.span.start;
    while (nameEnd < text.length && /[a-zA-Z0-9_]/.test(text[nameEnd])) {
        nameEnd++;
    }

    const hintPos = offsetToPosition(text, nameEnd);

    hints.push({
        position: Position.create(hintPos.line, hintPos.character),
        label: `: ${inferredType}`,
        kind: InlayHintKind.Type,
        paddingLeft: false,
        paddingRight: true,
    });
}

/**
 * Add hints for function body.
 */
function addFunctionHints(
    hints: InlayHint[],
    func: FunctionDef,
    text: string,
    functionMap: Map<string, { params: { name: string; type: string }[] }>
): void {
    // Process all statements in function body
    for (const stmt of func.body) {
        processStatementForHints(hints, stmt, text, functionMap);
    }
}

/**
 * Process a statement for inlay hints.
 */
function processStatementForHints(
    hints: InlayHint[],
    stmt: any,
    text: string,
    functionMap: Map<string, { params: { name: string; type: string }[] }>
): void {
    switch (stmt.kind) {
        case 'VarDecl':
            addVarDeclTypeHint(hints, stmt, text);
            if (stmt.initializer) {
                processExpressionForHints(hints, stmt.initializer, text, functionMap);
            }
            break;

        case 'Assignment':
            processExpressionForHints(hints, stmt.value, text, functionMap);
            if (stmt.index) {
                processExpressionForHints(hints, stmt.index, text, functionMap);
            }
            break;

        case 'IfStatement':
            processExpressionForHints(hints, stmt.condition, text, functionMap);
            for (const s of stmt.thenBranch) {
                processStatementForHints(hints, s, text, functionMap);
            }
            for (const elif of stmt.elifBranches) {
                processExpressionForHints(hints, elif.condition, text, functionMap);
                for (const s of elif.body) {
                    processStatementForHints(hints, s, text, functionMap);
                }
            }
            if (stmt.elseBranch) {
                for (const s of stmt.elseBranch) {
                    processStatementForHints(hints, s, text, functionMap);
                }
            }
            break;

        case 'WhileStatement':
            processExpressionForHints(hints, stmt.condition, text, functionMap);
            for (const s of stmt.body) {
                processStatementForHints(hints, s, text, functionMap);
            }
            break;

        case 'ForStatement':
            processExpressionForHints(hints, stmt.start, text, functionMap);
            processExpressionForHints(hints, stmt.end, text, functionMap);
            for (const s of stmt.body) {
                processStatementForHints(hints, s, text, functionMap);
            }
            break;

        case 'ReturnStatement':
            if (stmt.value) {
                processExpressionForHints(hints, stmt.value, text, functionMap);
            }
            break;

        case 'ExpressionStatement':
            processExpressionForHints(hints, stmt.expression, text, functionMap);
            break;
    }
}

/**
 * Process an expression for inlay hints.
 */
function processExpressionForHints(
    hints: InlayHint[],
    expr: Expression,
    text: string,
    functionMap: Map<string, { params: { name: string; type: string }[] }>
): void {
    switch (expr.kind) {
        case 'FunctionCall':
            // Add parameter name hints
            addParameterHints(hints, expr, text, functionMap);
            // Process nested expressions
            for (const arg of expr.args) {
                processExpressionForHints(hints, arg, text, functionMap);
            }
            break;

        case 'BinaryOp':
            processExpressionForHints(hints, expr.left, text, functionMap);
            processExpressionForHints(hints, expr.right, text, functionMap);
            break;

        case 'UnaryOp':
            processExpressionForHints(hints, expr.operand, text, functionMap);
            break;

        case 'ArrayLiteral':
            for (const elem of expr.elements) {
                processExpressionForHints(hints, elem, text, functionMap);
            }
            break;

        case 'ArrayIndex':
            processExpressionForHints(hints, expr.array, text, functionMap);
            processExpressionForHints(hints, expr.index, text, functionMap);
            break;

        case 'TypeCast':
            processExpressionForHints(hints, expr.expression, text, functionMap);
            break;
    }
}

/**
 * Add parameter name hints for function calls.
 */
function addParameterHints(
    hints: InlayHint[],
    call: { name: string; args: Expression[]; span: { start: number; end: number } },
    text: string,
    functionMap: Map<string, { params: { name: string; type: string }[] }>
): void {
    const funcInfo = functionMap.get(call.name);
    if (!funcInfo) return;

    // Only add hints if function has parameters
    if (funcInfo.params.length === 0) return;

    // Add hint for each argument
    for (let i = 0; i < call.args.length && i < funcInfo.params.length; i++) {
        const arg = call.args[i];
        const param = funcInfo.params[i];

        // Skip if argument is already a named literal matching param name
        if (arg.kind === 'Identifier' && arg.name === param.name) {
            continue;
        }

        // Get position at start of argument
        const argPos = offsetToPosition(text, arg.span.start);

        hints.push({
            position: Position.create(argPos.line, argPos.character),
            label: `${param.name}:`,
            kind: InlayHintKind.Parameter,
            paddingLeft: false,
            paddingRight: true,
        });
    }
}

/**
 * Infer the type of an expression.
 */
function inferExpressionType(expr: Expression): string | null {
    switch (expr.kind) {
        case 'IntegerLiteral': {
            const value = expr.value;
            if (value >= 0 && value <= 255) return 'byte';
            if (value >= 0 && value <= 65535) return 'word';
            if (value >= -128 && value < 0) return 'sbyte';
            if (value >= -32768 && value < -128) return 'sword';
            return 'word';
        }

        case 'DecimalLiteral':
            return 'float';

        case 'StringLiteral':
            return 'string';

        case 'CharLiteral':
            return 'byte';

        case 'BoolLiteral':
            return 'bool';

        case 'ArrayLiteral':
            if (expr.elements.length === 0) return 'byte[]';
            const elemType = inferExpressionType(expr.elements[0]);
            return elemType ? `${elemType}[]` : null;

        default:
            return null;
    }
}
