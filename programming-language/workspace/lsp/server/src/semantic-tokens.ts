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
    SemanticTokensBuilder,
    SemanticTokensLegend,
    SemanticTokenTypes,
    SemanticTokenModifiers,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { Token, TokenType, KEYWORDS, TYPE_KEYWORDS } from './lexer';
import { AnalyzerResult } from './analyzer';
import {
    COBRA64_BUILTINS,
    offsetToPosition,
} from '../../shared/types';

/**
 * Semantic token types used by Cobra64.
 */
export const tokenTypes = [
    'namespace',
    'type',
    'class',
    'enum',
    'interface',
    'struct',
    'typeParameter',
    'parameter',
    'variable',
    'property',
    'enumMember',
    'event',
    'function',
    'method',
    'macro',
    'keyword',
    'modifier',
    'comment',
    'string',
    'number',
    'regexp',
    'operator',
];

/**
 * Semantic token modifiers used by Cobra64.
 */
export const tokenModifiers = [
    'declaration',
    'definition',
    'readonly',
    'static',
    'deprecated',
    'abstract',
    'async',
    'modification',
    'documentation',
    'defaultLibrary',
];

/**
 * Create the semantic tokens legend.
 */
export function createSemanticTokensLegend(): SemanticTokensLegend {
    return {
        tokenTypes,
        tokenModifiers,
    };
}

/**
 * Get the index of a token type.
 */
function getTokenTypeIndex(type: string): number {
    return tokenTypes.indexOf(type);
}

/**
 * Get the modifier bitmask.
 */
function getModifierBitmask(modifiers: string[]): number {
    let result = 0;
    for (const modifier of modifiers) {
        const index = tokenModifiers.indexOf(modifier);
        if (index >= 0) {
            result |= (1 << index);
        }
    }
    return result;
}

/**
 * Build semantic tokens for a document.
 */
export function buildSemanticTokens(
    document: TextDocument,
    tokens: Token[],
    analyzerResult: AnalyzerResult | null
): number[] {
    const builder = new SemanticTokensBuilder();
    const text = document.getText();

    // Build a set of builtin function names for quick lookup
    const builtinNames = new Set(COBRA64_BUILTINS.map(b => b.name));

    // Build maps for quick symbol lookup
    const symbolMap = new Map<string, { kind: string; isConstant: boolean }>();
    const functionMap = new Map<string, { isBuiltin: boolean }>();

    if (analyzerResult) {
        for (const symbol of analyzerResult.symbols) {
            symbolMap.set(symbol.name, {
                kind: symbol.kind,
                isConstant: symbol.kind === 'constant',
            });
        }
        for (const func of analyzerResult.functions) {
            functionMap.set(func.name, { isBuiltin: func.isBuiltin });
        }
    }

    // Process each token
    for (const token of tokens) {
        const startPos = offsetToPosition(text, token.span.start);
        const length = token.span.end - token.span.start;

        let tokenType: string | null = null;
        let modifiers: string[] = [];

        switch (token.type) {
            case TokenType.Keyword:
                tokenType = 'keyword';
                break;

            case TokenType.Type:
                tokenType = 'type';
                break;

            case TokenType.BoolLiteral:
                tokenType = 'keyword';
                break;

            case TokenType.Identifier: {
                const name = token.value;

                // Check if it's a builtin function
                if (builtinNames.has(name)) {
                    tokenType = 'function';
                    modifiers = ['defaultLibrary'];
                }
                // Check if it's a user function
                else if (functionMap.has(name)) {
                    const funcInfo = functionMap.get(name)!;
                    tokenType = 'function';
                    if (funcInfo.isBuiltin) {
                        modifiers = ['defaultLibrary'];
                    }
                }
                // Check if it's a known symbol
                else if (symbolMap.has(name)) {
                    const symbolInfo = symbolMap.get(name)!;
                    if (symbolInfo.kind === 'constant') {
                        tokenType = 'variable';
                        modifiers = ['readonly'];
                    } else if (symbolInfo.kind === 'parameter') {
                        tokenType = 'parameter';
                    } else {
                        tokenType = 'variable';
                    }
                }
                // Default: treat as variable
                else {
                    tokenType = 'variable';
                }
                break;
            }

            case TokenType.Integer:
            case TokenType.Decimal:
                tokenType = 'number';
                break;

            case TokenType.String:
            case TokenType.Character:
                tokenType = 'string';
                break;

            case TokenType.Comment:
                tokenType = 'comment';
                break;

            // Operators
            case TokenType.Plus:
            case TokenType.Minus:
            case TokenType.Star:
            case TokenType.Slash:
            case TokenType.Percent:
            case TokenType.Ampersand:
            case TokenType.Pipe:
            case TokenType.Caret:
            case TokenType.Tilde:
            case TokenType.ShiftLeft:
            case TokenType.ShiftRight:
            case TokenType.Equal:
            case TokenType.NotEqual:
            case TokenType.Less:
            case TokenType.Greater:
            case TokenType.LessEqual:
            case TokenType.GreaterEqual:
                tokenType = 'operator';
                break;

            // Skip tokens that don't need semantic highlighting
            case TokenType.Newline:
            case TokenType.Indent:
            case TokenType.Dedent:
            case TokenType.EOF:
            case TokenType.Colon:
            case TokenType.Comma:
            case TokenType.LeftParen:
            case TokenType.RightParen:
            case TokenType.LeftBracket:
            case TokenType.RightBracket:
            case TokenType.Arrow:
            case TokenType.Assign:
            case TokenType.PlusAssign:
            case TokenType.MinusAssign:
            case TokenType.StarAssign:
            case TokenType.SlashAssign:
            case TokenType.PercentAssign:
            case TokenType.AmpersandAssign:
            case TokenType.PipeAssign:
            case TokenType.CaretAssign:
            case TokenType.ShiftLeftAssign:
            case TokenType.ShiftRightAssign:
            case TokenType.Error:
                tokenType = null;
                break;

            default:
                tokenType = null;
        }

        if (tokenType !== null) {
            const typeIndex = getTokenTypeIndex(tokenType);
            if (typeIndex >= 0) {
                builder.push(
                    startPos.line,
                    startPos.character,
                    length,
                    typeIndex,
                    getModifierBitmask(modifiers)
                );
            }
        }
    }

    return builder.build().data;
}

/**
 * Interface for semantic token data.
 */
export interface SemanticTokenData {
    line: number;
    character: number;
    length: number;
    tokenType: number;
    tokenModifiers: number;
}

/**
 * Build semantic tokens with detailed information for debugging.
 */
export function buildSemanticTokensDetailed(
    document: TextDocument,
    tokens: Token[],
    analyzerResult: AnalyzerResult | null
): SemanticTokenData[] {
    const result: SemanticTokenData[] = [];
    const text = document.getText();

    const builtinNames = new Set(COBRA64_BUILTINS.map(b => b.name));
    const symbolMap = new Map<string, { kind: string; isConstant: boolean }>();
    const functionMap = new Map<string, { isBuiltin: boolean }>();

    if (analyzerResult) {
        for (const symbol of analyzerResult.symbols) {
            symbolMap.set(symbol.name, {
                kind: symbol.kind,
                isConstant: symbol.kind === 'constant',
            });
        }
        for (const func of analyzerResult.functions) {
            functionMap.set(func.name, { isBuiltin: func.isBuiltin });
        }
    }

    for (const token of tokens) {
        const startPos = offsetToPosition(text, token.span.start);
        const length = token.span.end - token.span.start;

        let tokenType: string | null = null;
        let modifiers: string[] = [];

        switch (token.type) {
            case TokenType.Keyword:
                tokenType = 'keyword';
                break;

            case TokenType.Type:
                tokenType = 'type';
                break;

            case TokenType.BoolLiteral:
                tokenType = 'keyword';
                break;

            case TokenType.Identifier: {
                const name = token.value;
                if (builtinNames.has(name)) {
                    tokenType = 'function';
                    modifiers = ['defaultLibrary'];
                } else if (functionMap.has(name)) {
                    tokenType = 'function';
                    if (functionMap.get(name)!.isBuiltin) {
                        modifiers = ['defaultLibrary'];
                    }
                } else if (symbolMap.has(name)) {
                    const symbolInfo = symbolMap.get(name)!;
                    if (symbolInfo.kind === 'constant') {
                        tokenType = 'variable';
                        modifiers = ['readonly'];
                    } else if (symbolInfo.kind === 'parameter') {
                        tokenType = 'parameter';
                    } else {
                        tokenType = 'variable';
                    }
                } else {
                    tokenType = 'variable';
                }
                break;
            }

            case TokenType.Integer:
            case TokenType.Decimal:
                tokenType = 'number';
                break;

            case TokenType.String:
            case TokenType.Character:
                tokenType = 'string';
                break;

            case TokenType.Comment:
                tokenType = 'comment';
                break;

            default:
                tokenType = null;
        }

        if (tokenType !== null) {
            const typeIndex = getTokenTypeIndex(tokenType);
            if (typeIndex >= 0) {
                result.push({
                    line: startPos.line,
                    character: startPos.character,
                    length,
                    tokenType: typeIndex,
                    tokenModifiers: getModifierBitmask(modifiers),
                });
            }
        }
    }

    return result;
}
