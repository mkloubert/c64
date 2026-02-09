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
    Hover,
    MarkupContent,
    MarkupKind,
    Position,
    Location,
    Range,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { Token, TokenType, tokenize, KEYWORDS, TYPE_KEYWORDS } from './lexer';
import {
    Program,
    FunctionDef,
    Expression,
    Statement,
    TopLevelItem,
} from './parser';
import { AnalyzerResult } from './analyzer';
import {
    COBRA64_BUILTINS,
    COBRA64_CONSTANTS,
    COBRA64_TYPES,
    COBRA64_KEYWORDS,
    positionToOffset,
    offsetToPosition,
    Span,
} from '../../shared/types';

/**
 * Document analysis cache for features.
 */
export interface DocumentAnalysis {
    program: Program | null;
    analyzerResult: AnalyzerResult | null;
    tokens: Token[];
}

/**
 * Keyword documentation.
 */
const KEYWORD_DOCS: Record<string, { description: string; example: string }> = {
    def: {
        description: 'Defines a function.',
        example: 'def add(a: byte, b: byte) -> byte:\n    return a + b',
    },
    if: {
        description: 'Conditional statement. Executes block if condition is true.',
        example: 'if x > 10:\n    println("large")',
    },
    elif: {
        description: 'Else-if branch. Checked if previous conditions were false.',
        example: 'if x > 10:\n    println("large")\nelif x > 5:\n    println("medium")',
    },
    else: {
        description: 'Else branch. Executes if all previous conditions were false.',
        example: 'if x > 10:\n    println("large")\nelse:\n    println("small")',
    },
    while: {
        description: 'While loop. Repeats block while condition is true.',
        example: 'while i < 10:\n    println(i)\n    i = i + 1',
    },
    for: {
        description: 'For loop. Iterates over a range of values.',
        example: 'for i in 0 to 9:\n    println(i)',
    },
    in: {
        description: 'Part of for loop syntax. Specifies the range start.',
        example: 'for i in 0 to 9:',
    },
    to: {
        description: 'Ascending range in for loop (inclusive).',
        example: 'for i in 0 to 9:  # 0, 1, 2, ..., 9',
    },
    downto: {
        description: 'Descending range in for loop (inclusive).',
        example: 'for i in 10 downto 1:  # 10, 9, 8, ..., 1',
    },
    break: {
        description: 'Exits the innermost loop immediately.',
        example: 'while true:\n    if done:\n        break',
    },
    continue: {
        description: 'Skips to the next iteration of the loop.',
        example: 'for i in 0 to 9:\n    if i % 2 == 0:\n        continue\n    println(i)',
    },
    return: {
        description: 'Returns a value from a function.',
        example: 'def double(x: byte) -> byte:\n    return x * 2',
    },
    pass: {
        description: 'Placeholder statement. Does nothing.',
        example: 'def todo():\n    pass',
    },
    and: {
        description: 'Logical AND operator. True if both operands are true.',
        example: 'if x > 0 and x < 10:',
    },
    or: {
        description: 'Logical OR operator. True if either operand is true.',
        example: 'if x == 0 or x == 10:',
    },
    not: {
        description: 'Logical NOT operator. Inverts boolean value.',
        example: 'if not done:',
    },
    true: {
        description: 'Boolean literal representing true.',
        example: 'flag: bool = true',
    },
    false: {
        description: 'Boolean literal representing false.',
        example: 'flag: bool = false',
    },
    data: {
        description: 'Defines a data block for embedding raw binary data.',
        example: 'data SPRITE:\n    $00, $3C, $00\n    $00, $7E, $00\nend',
    },
    end: {
        description: 'Terminates a data block definition.',
        example: 'data SPRITE:\n    $FF, $FF\nend',
    },
    include: {
        description: 'Includes an external binary file in a data block.',
        example: 'data MUSIC:\n    include "music.sid", $7E  # skip header\nend',
    },
};

/**
 * Type documentation.
 */
const TYPE_DOCS: Record<string, { description: string; range: string }> = {
    byte: {
        description: 'Unsigned 8-bit integer',
        range: '0 to 255',
    },
    word: {
        description: 'Unsigned 16-bit integer',
        range: '0 to 65535',
    },
    sbyte: {
        description: 'Signed 8-bit integer',
        range: '-128 to 127',
    },
    sword: {
        description: 'Signed 16-bit integer',
        range: '-32768 to 32767',
    },
    fixed: {
        description: 'Fixed-point decimal (12.4 format)',
        range: '-2048.0 to +2047.9375',
    },
    float: {
        description: 'IEEE-754 binary16 floating point',
        range: '±65504 (±6.1e-5 minimum)',
    },
    bool: {
        description: 'Boolean value',
        range: 'true or false',
    },
    string: {
        description: 'Text string',
        range: 'Variable length',
    },
};

/**
 * Find the token at a given position.
 */
export function findTokenAtPosition(
    document: TextDocument,
    position: Position,
    tokens: Token[]
): Token | null {
    const offset = positionToOffset(document.getText(), {
        line: position.line,
        character: position.character,
    });

    for (const token of tokens) {
        if (offset >= token.span.start && offset < token.span.end) {
            return token;
        }
    }

    return null;
}

/**
 * Find word at position (for identifiers that might not be tokenized).
 */
export function getWordAtPosition(
    document: TextDocument,
    position: Position
): { word: string; range: Range } | null {
    const text = document.getText();
    const offset = positionToOffset(text, {
        line: position.line,
        character: position.character,
    });

    if (offset >= text.length) return null;

    // Find word boundaries
    let start = offset;
    let end = offset;

    while (start > 0 && isIdentifierChar(text[start - 1])) {
        start--;
    }

    while (end < text.length && isIdentifierChar(text[end])) {
        end++;
    }

    if (start === end) return null;

    const word = text.slice(start, end);
    const startPos = offsetToPosition(text, start);
    const endPos = offsetToPosition(text, end);

    return {
        word,
        range: {
            start: { line: startPos.line, character: startPos.character },
            end: { line: endPos.line, character: endPos.character },
        },
    };
}

function isIdentifierChar(c: string): boolean {
    return /[a-zA-Z0-9_]/.test(c);
}

/**
 * Get hover information for a position.
 */
export function getHover(
    document: TextDocument,
    position: Position,
    analysis: DocumentAnalysis
): Hover | null {
    const wordInfo = getWordAtPosition(document, position);
    if (!wordInfo) return null;

    const { word, range } = wordInfo;

    // Check if it's a keyword
    if (KEYWORDS.has(word) || word === 'true' || word === 'false') {
        const doc = KEYWORD_DOCS[word];
        if (doc) {
            return {
                contents: {
                    kind: MarkupKind.Markdown,
                    value: `**${word}** (keyword)\n\n${doc.description}\n\n\`\`\`python\n${doc.example}\n\`\`\``,
                },
                range,
            };
        }
    }

    // Check if it's a type
    if (TYPE_KEYWORDS.has(word)) {
        const doc = TYPE_DOCS[word];
        if (doc) {
            return {
                contents: {
                    kind: MarkupKind.Markdown,
                    value: `**${word}** (type)\n\n${doc.description}\n\nRange: \`${doc.range}\``,
                },
                range,
            };
        }
    }

    // Check if it's a built-in function
    const builtin = COBRA64_BUILTINS.find(b => b.name === word);
    if (builtin) {
        const params = builtin.parameters.map(p => `- \`${p.name}: ${p.type}\` - ${p.description}`).join('\n');
        const returnInfo = builtin.returnType ? `\n\n**Returns:** \`${builtin.returnType}\`` : '';
        const examplesSection = builtin.examples.length > 0
            ? '\n\n**Examples:**\n```python\n' + builtin.examples.join('\n\n') + '\n```'
            : '';

        return {
            contents: {
                kind: MarkupKind.Markdown,
                value: `**${builtin.name}** (built-in function)\n\n\`\`\`python\n${builtin.signature}\n\`\`\`\n\n${builtin.description}${params ? '\n\n**Parameters:**\n' + params : ''}${returnInfo}${examplesSection}`,
            },
            range,
        };
    }

    // Check if it's a built-in constant
    const constant = COBRA64_CONSTANTS.find(c => c.name === word);
    if (constant) {
        const examplesSection = constant.examples.length > 0
            ? '\n\n**Examples:**\n```python\n' + constant.examples.join('\n') + '\n```'
            : '';

        return {
            contents: {
                kind: MarkupKind.Markdown,
                value: `**${constant.name}** (built-in constant)\n\n\`\`\`python\nconst ${constant.name}: ${constant.type} = ${constant.value}\n\`\`\`\n\n${constant.description}${examplesSection}`,
            },
            range,
        };
    }

    // Check analyzer results for symbols
    if (analysis.analyzerResult) {
        // Check functions
        const func = analysis.analyzerResult.functions.find(f => f.name === word && !f.isBuiltin);
        if (func) {
            const params = func.params.map(p => `${p.name}: ${p.type}`).join(', ');
            const returnType = func.returnType ? ` -> ${func.returnType}` : '';

            return {
                contents: {
                    kind: MarkupKind.Markdown,
                    value: `**${func.name}** (function)\n\n\`\`\`python\ndef ${func.name}(${params})${returnType}\n\`\`\``,
                },
                range,
            };
        }

        // Check symbols (variables/constants/data blocks)
        const symbol = analysis.analyzerResult.symbols.find(s => s.name === word);
        if (symbol) {
            if (symbol.kind === 'dataBlock') {
                return {
                    contents: {
                        kind: MarkupKind.Markdown,
                        value: `**${symbol.name}** (data block)\n\nType: \`word\` (address)\n\n*Data blocks embed raw binary data. Reference returns the address.*`,
                    },
                    range,
                };
            }

            const kindLabel = symbol.kind === 'constant' ? 'constant' :
                symbol.kind === 'parameter' ? 'parameter' : 'variable';

            let value = `**${symbol.name}** (${kindLabel})\n\n`;
            value += `Type: \`${symbol.type}\``;

            if (symbol.kind === 'constant') {
                value += '\n\n*Constants are immutable and must have UPPERCASE names.*';
            }

            return {
                contents: {
                    kind: MarkupKind.Markdown,
                    value,
                },
                range,
            };
        }
    }

    return null;
}

/**
 * Get definition location for a position.
 */
export function getDefinition(
    document: TextDocument,
    position: Position,
    analysis: DocumentAnalysis
): Location | null {
    const wordInfo = getWordAtPosition(document, position);
    if (!wordInfo) return null;

    const { word } = wordInfo;

    // Keywords and types don't have definitions
    if (KEYWORDS.has(word) || TYPE_KEYWORDS.has(word) || word === 'true' || word === 'false') {
        return null;
    }

    // Built-in functions don't have definitions in the source
    const builtin = COBRA64_BUILTINS.find(b => b.name === word);
    if (builtin) {
        return null;
    }

    if (!analysis.analyzerResult) return null;

    const text = document.getText();

    // Check functions
    const func = analysis.analyzerResult.functions.find(f => f.name === word && !f.isBuiltin);
    if (func) {
        return {
            uri: document.uri,
            range: spanToRange(text, func.span),
        };
    }

    // Check symbols
    const symbol = analysis.analyzerResult.symbols.find(s => s.name === word);
    if (symbol) {
        return {
            uri: document.uri,
            range: spanToRange(text, symbol.definitionSpan),
        };
    }

    return null;
}

/**
 * Convert span to LSP Range.
 */
function spanToRange(source: string, span: Span): Range {
    const startPos = offsetToPosition(source, span.start);
    const endPos = offsetToPosition(source, span.end);

    return {
        start: { line: startPos.line, character: startPos.character },
        end: { line: endPos.line, character: endPos.character },
    };
}

/**
 * Find all references to a symbol.
 */
export function findReferences(
    document: TextDocument,
    position: Position,
    analysis: DocumentAnalysis,
    includeDeclaration: boolean
): Location[] {
    const wordInfo = getWordAtPosition(document, position);
    if (!wordInfo) return [];

    const { word } = wordInfo;
    const text = document.getText();
    const locations: Location[] = [];

    // Find all occurrences of the word in tokens
    for (const token of analysis.tokens) {
        if (token.type === TokenType.Identifier && token.value === word) {
            locations.push({
                uri: document.uri,
                range: spanToRange(text, token.span),
            });
        }
    }

    // If not including declaration, filter it out
    if (!includeDeclaration && analysis.analyzerResult) {
        const symbol = analysis.analyzerResult.symbols.find(s => s.name === word);
        if (symbol) {
            const declRange = spanToRange(text, symbol.definitionSpan);
            return locations.filter(loc =>
                loc.range.start.line !== declRange.start.line ||
                loc.range.start.character !== declRange.start.character
            );
        }
    }

    return locations;
}

/**
 * Get document symbols for outline.
 */
export function getDocumentSymbols(analysis: DocumentAnalysis): {
    name: string;
    kind: number; // SymbolKind
    range: Range;
    selectionRange: Range;
    children?: any[];
}[] {
    if (!analysis.program) return [];

    const symbols: any[] = [];
    const source = ''; // We need the source to convert spans

    // This is a simplified version - would need source text for proper ranges
    for (const item of analysis.program.items) {
        if (item.kind === 'FunctionDef') {
            symbols.push({
                name: item.name,
                kind: 12, // Function
                range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
                selectionRange: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            });
        } else if (item.kind === 'VarDecl') {
            symbols.push({
                name: item.name,
                kind: 13, // Variable
                range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
                selectionRange: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            });
        } else if (item.kind === 'ConstDecl') {
            symbols.push({
                name: item.name,
                kind: 14, // Constant
                range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
                selectionRange: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            });
        } else if (item.kind === 'DataBlockDef') {
            symbols.push({
                name: item.name,
                kind: 14, // Constant (data blocks act as constants)
                range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
                selectionRange: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } },
            });
        }
    }

    return symbols;
}
