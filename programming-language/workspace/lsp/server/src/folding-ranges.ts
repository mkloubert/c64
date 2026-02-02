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
    FoldingRange,
    FoldingRangeKind,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { Program, Statement, FunctionDef } from './parser';
import { Token, TokenType } from './lexer';
import { offsetToPosition } from '../../shared/types';

/**
 * Get folding ranges for a document.
 */
export function getFoldingRanges(
    document: TextDocument,
    program: Program | null,
    tokens: Token[]
): FoldingRange[] {
    const ranges: FoldingRange[] = [];
    const text = document.getText();

    // Add comment block folding
    addCommentFoldingRanges(ranges, tokens, text);

    // Add code block folding from AST
    if (program) {
        addProgramFoldingRanges(ranges, program, text);
    }

    return ranges;
}

/**
 * Add folding ranges for consecutive comment lines.
 */
function addCommentFoldingRanges(ranges: FoldingRange[], tokens: Token[], text: string): void {
    let commentBlockStart: number | null = null;
    let commentBlockEnd: number | null = null;
    let lastCommentLine: number = -2;

    for (const token of tokens) {
        if (token.type === TokenType.Comment) {
            const startPos = offsetToPosition(text, token.span.start);
            const endPos = offsetToPosition(text, token.span.end);

            if (startPos.line === lastCommentLine + 1) {
                // Continue the block
                commentBlockEnd = endPos.line;
            } else {
                // End previous block if exists
                if (commentBlockStart !== null && commentBlockEnd !== null && commentBlockEnd > commentBlockStart) {
                    ranges.push({
                        startLine: commentBlockStart,
                        endLine: commentBlockEnd,
                        kind: FoldingRangeKind.Comment,
                    });
                }
                // Start new block
                commentBlockStart = startPos.line;
                commentBlockEnd = endPos.line;
            }

            lastCommentLine = endPos.line;
        }
    }

    // Don't forget the last block
    if (commentBlockStart !== null && commentBlockEnd !== null && commentBlockEnd > commentBlockStart) {
        ranges.push({
            startLine: commentBlockStart,
            endLine: commentBlockEnd,
            kind: FoldingRangeKind.Comment,
        });
    }
}

/**
 * Add folding ranges for program items.
 */
function addProgramFoldingRanges(ranges: FoldingRange[], program: Program, text: string): void {
    for (const item of program.items) {
        if (item.kind === 'FunctionDef') {
            addFunctionFoldingRange(ranges, item, text);
        }
    }
}

/**
 * Add folding range for a function definition.
 */
function addFunctionFoldingRange(ranges: FoldingRange[], func: FunctionDef, text: string): void {
    const startPos = offsetToPosition(text, func.span.start);
    const endPos = offsetToPosition(text, func.span.end);

    // Only add if spans multiple lines
    if (endPos.line > startPos.line) {
        ranges.push({
            startLine: startPos.line,
            endLine: endPos.line,
            kind: FoldingRangeKind.Region,
        });
    }

    // Process function body for nested blocks
    for (const stmt of func.body) {
        addStatementFoldingRange(ranges, stmt, text);
    }
}

/**
 * Add folding range for a statement.
 */
function addStatementFoldingRange(ranges: FoldingRange[], stmt: Statement, text: string): void {
    switch (stmt.kind) {
        case 'IfStatement': {
            const startPos = offsetToPosition(text, stmt.span.start);
            const endPos = offsetToPosition(text, stmt.span.end);

            if (endPos.line > startPos.line) {
                ranges.push({
                    startLine: startPos.line,
                    endLine: endPos.line,
                    kind: FoldingRangeKind.Region,
                });
            }

            // Process then branch
            for (const s of stmt.thenBranch) {
                addStatementFoldingRange(ranges, s, text);
            }

            // Process elif branches
            for (const elif of stmt.elifBranches) {
                // Elif branches don't have their own span, so we calculate from the body
                if (elif.body.length > 0) {
                    const firstStmt = elif.body[0];
                    const lastStmt = elif.body[elif.body.length - 1];
                    const bodyStart = offsetToPosition(text, firstStmt.span.start);
                    const bodyEnd = offsetToPosition(text, lastStmt.span.end);

                    // Only fold if body spans multiple lines
                    if (bodyEnd.line > bodyStart.line) {
                        ranges.push({
                            startLine: bodyStart.line,
                            endLine: bodyEnd.line,
                            kind: FoldingRangeKind.Region,
                        });
                    }

                    for (const s of elif.body) {
                        addStatementFoldingRange(ranges, s, text);
                    }
                }
            }

            // Process else branch
            if (stmt.elseBranch && stmt.elseBranch.length > 0) {
                for (const s of stmt.elseBranch) {
                    addStatementFoldingRange(ranges, s, text);
                }
            }
            break;
        }

        case 'WhileStatement': {
            const startPos = offsetToPosition(text, stmt.span.start);
            const endPos = offsetToPosition(text, stmt.span.end);

            if (endPos.line > startPos.line) {
                ranges.push({
                    startLine: startPos.line,
                    endLine: endPos.line,
                    kind: FoldingRangeKind.Region,
                });
            }

            for (const s of stmt.body) {
                addStatementFoldingRange(ranges, s, text);
            }
            break;
        }

        case 'ForStatement': {
            const startPos = offsetToPosition(text, stmt.span.start);
            const endPos = offsetToPosition(text, stmt.span.end);

            if (endPos.line > startPos.line) {
                ranges.push({
                    startLine: startPos.line,
                    endLine: endPos.line,
                    kind: FoldingRangeKind.Region,
                });
            }

            for (const s of stmt.body) {
                addStatementFoldingRange(ranges, s, text);
            }
            break;
        }
    }
}
