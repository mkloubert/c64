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
    CodeAction,
    CodeActionKind,
    Diagnostic,
    Range,
    TextEdit,
    WorkspaceEdit,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { Program, VarDecl } from './parser';
import { AnalyzerResult } from './analyzer';
import {
    COBRA64_BUILTINS,
    COBRA64_KEYWORDS,
    COBRA64_TYPES,
    offsetToPosition,
} from '../../shared/types';

/**
 * Get code actions for a range in a document.
 */
export function getCodeActions(
    document: TextDocument,
    range: Range,
    diagnostics: Diagnostic[],
    program: Program | null,
    analyzerResult: AnalyzerResult | null
): CodeAction[] {
    const actions: CodeAction[] = [];

    // Process diagnostics and generate quick fixes
    for (const diagnostic of diagnostics) {
        const diagnosticActions = getActionsForDiagnostic(document, diagnostic, program, analyzerResult);
        actions.push(...diagnosticActions);
    }

    // Add refactoring actions based on selection
    const refactoringActions = getRefactoringActions(document, range, program);
    actions.push(...refactoringActions);

    return actions;
}

/**
 * Get code actions for a specific diagnostic.
 */
function getActionsForDiagnostic(
    document: TextDocument,
    diagnostic: Diagnostic,
    program: Program | null,
    analyzerResult: AnalyzerResult | null
): CodeAction[] {
    const actions: CodeAction[] = [];
    const code = diagnostic.code as string;

    switch (code) {
        case 'E200': // Undefined variable
            actions.push(...getUndefinedVariableFixes(document, diagnostic, analyzerResult));
            break;

        case 'E201': // Undefined function
            actions.push(...getUndefinedFunctionFixes(document, diagnostic));
            break;

        case 'E207': // Wrong number of arguments
            // Could suggest fixing argument count
            break;
    }

    return actions;
}

/**
 * Get fixes for undefined variable errors.
 */
function getUndefinedVariableFixes(
    document: TextDocument,
    diagnostic: Diagnostic,
    analyzerResult: AnalyzerResult | null
): CodeAction[] {
    const actions: CodeAction[] = [];
    const text = document.getText();
    const range = diagnostic.range;

    // Extract the undefined variable name from the message
    const match = diagnostic.message.match(/Undefined variable '([^']+)'/);
    if (!match) return actions;

    const varName = match[1];

    // 1. Suggest similar variable names (typo fix)
    if (analyzerResult) {
        const similarNames = findSimilarNames(varName, analyzerResult.symbols.map(s => s.name));
        for (const similar of similarNames) {
            actions.push({
                title: `Change to '${similar}'`,
                kind: CodeActionKind.QuickFix,
                diagnostics: [diagnostic],
                isPreferred: similarNames[0] === similar,
                edit: {
                    changes: {
                        [document.uri]: [TextEdit.replace(range, similar)],
                    },
                },
            });
        }
    }

    // 2. Suggest declaring the variable
    const lineStart = document.offsetAt(range.start);
    let lineBegin = lineStart;
    while (lineBegin > 0 && text[lineBegin - 1] !== '\n') {
        lineBegin--;
    }
    const indentation = text.slice(lineBegin, lineStart).match(/^\s*/)?.[0] || '';

    actions.push({
        title: `Declare variable '${varName}'`,
        kind: CodeActionKind.QuickFix,
        diagnostics: [diagnostic],
        edit: {
            changes: {
                [document.uri]: [
                    TextEdit.insert(
                        { line: range.start.line, character: 0 },
                        `${indentation}${varName}: byte = 0\n`
                    ),
                ],
            },
        },
    });

    return actions;
}

/**
 * Get fixes for undefined function errors.
 */
function getUndefinedFunctionFixes(
    document: TextDocument,
    diagnostic: Diagnostic
): CodeAction[] {
    const actions: CodeAction[] = [];
    const range = diagnostic.range;

    // Extract the undefined function name
    const match = diagnostic.message.match(/Undefined function '([^']+)'/);
    if (!match) return actions;

    const funcName = match[1];

    // Suggest similar built-in function names
    const builtinNames = COBRA64_BUILTINS.map(b => b.name);
    const similarNames = findSimilarNames(funcName, builtinNames);

    for (const similar of similarNames) {
        actions.push({
            title: `Change to '${similar}'`,
            kind: CodeActionKind.QuickFix,
            diagnostics: [diagnostic],
            isPreferred: similarNames[0] === similar,
            edit: {
                changes: {
                    [document.uri]: [TextEdit.replace(range, similar)],
                },
            },
        });
    }

    // Suggest creating a function stub
    actions.push({
        title: `Create function '${funcName}'`,
        kind: CodeActionKind.QuickFix,
        diagnostics: [diagnostic],
        edit: {
            changes: {
                [document.uri]: [
                    TextEdit.insert(
                        { line: 0, character: 0 },
                        `def ${funcName}():\n    pass\n\n`
                    ),
                ],
            },
        },
    });

    return actions;
}

/**
 * Get refactoring actions based on selection.
 */
function getRefactoringActions(
    document: TextDocument,
    range: Range,
    program: Program | null
): CodeAction[] {
    const actions: CodeAction[] = [];

    if (!program) return actions;

    const text = document.getText();

    // Find variable declarations without type annotation in the selection
    for (const item of program.items) {
        if (item.kind === 'VarDecl' && !item.type && item.initializer) {
            const itemStart = offsetToPosition(text, item.span.start);
            const itemEnd = offsetToPosition(text, item.span.end);

            // Check if item is in range
            if (isPositionInRange(itemStart, range)) {
                const inferredType = inferType(item.initializer);
                if (inferredType) {
                    // Find position after variable name
                    let nameEnd = item.span.start;
                    while (nameEnd < text.length && /[a-zA-Z0-9_]/.test(text[nameEnd])) {
                        nameEnd++;
                    }
                    const insertPos = offsetToPosition(text, nameEnd);

                    actions.push({
                        title: `Add type annotation ': ${inferredType}'`,
                        kind: CodeActionKind.RefactorRewrite,
                        edit: {
                            changes: {
                                [document.uri]: [
                                    TextEdit.insert(
                                        { line: insertPos.line, character: insertPos.character },
                                        `: ${inferredType}`
                                    ),
                                ],
                            },
                        },
                    });
                }
            }
        }

        // Process function bodies for local variables
        if (item.kind === 'FunctionDef') {
            for (const stmt of item.body) {
                if (stmt.kind === 'VarDecl' && !stmt.type && stmt.initializer) {
                    const stmtStart = offsetToPosition(text, stmt.span.start);

                    if (isPositionInRange(stmtStart, range)) {
                        const inferredType = inferType(stmt.initializer);
                        if (inferredType) {
                            let nameEnd = stmt.span.start;
                            while (nameEnd < text.length && /[a-zA-Z0-9_]/.test(text[nameEnd])) {
                                nameEnd++;
                            }
                            const insertPos = offsetToPosition(text, nameEnd);

                            actions.push({
                                title: `Add type annotation ': ${inferredType}'`,
                                kind: CodeActionKind.RefactorRewrite,
                                edit: {
                                    changes: {
                                        [document.uri]: [
                                            TextEdit.insert(
                                                { line: insertPos.line, character: insertPos.character },
                                                `: ${inferredType}`
                                            ),
                                        ],
                                    },
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    return actions;
}

/**
 * Check if a position is within a range.
 */
function isPositionInRange(pos: { line: number; character: number }, range: Range): boolean {
    if (pos.line < range.start.line || pos.line > range.end.line) {
        return false;
    }
    if (pos.line === range.start.line && pos.character < range.start.character) {
        return false;
    }
    if (pos.line === range.end.line && pos.character > range.end.character) {
        return false;
    }
    return true;
}

/**
 * Find similar names using Levenshtein distance.
 */
function findSimilarNames(name: string, candidates: string[]): string[] {
    const maxDistance = Math.max(2, Math.floor(name.length / 3));
    const results: { name: string; distance: number }[] = [];

    for (const candidate of candidates) {
        if (candidate === name) continue;

        const distance = levenshteinDistance(name.toLowerCase(), candidate.toLowerCase());
        if (distance <= maxDistance) {
            results.push({ name: candidate, distance });
        }
    }

    // Sort by distance and return top 3
    results.sort((a, b) => a.distance - b.distance);
    return results.slice(0, 3).map(r => r.name);
}

/**
 * Calculate Levenshtein distance between two strings.
 */
function levenshteinDistance(a: string, b: string): number {
    const m = a.length;
    const n = b.length;

    if (m === 0) return n;
    if (n === 0) return m;

    const dp: number[][] = Array(m + 1).fill(null).map(() => Array(n + 1).fill(0));

    for (let i = 0; i <= m; i++) dp[i][0] = i;
    for (let j = 0; j <= n; j++) dp[0][j] = j;

    for (let i = 1; i <= m; i++) {
        for (let j = 1; j <= n; j++) {
            const cost = a[i - 1] === b[j - 1] ? 0 : 1;
            dp[i][j] = Math.min(
                dp[i - 1][j] + 1,      // deletion
                dp[i][j - 1] + 1,      // insertion
                dp[i - 1][j - 1] + cost // substitution
            );
        }
    }

    return dp[m][n];
}

/**
 * Infer type from expression.
 */
function inferType(expr: any): string | null {
    switch (expr.kind) {
        case 'IntegerLiteral': {
            const value = expr.value;
            if (value >= 0 && value <= 255) return 'byte';
            if (value >= 0 && value <= 65535) return 'word';
            if (value >= -128 && value < 0) return 'sbyte';
            return 'sword';
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
            const elemType = inferType(expr.elements[0]);
            return elemType ? `${elemType}[]` : null;
        default:
            return null;
    }
}
