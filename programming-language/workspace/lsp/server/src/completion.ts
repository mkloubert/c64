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
    CompletionItem,
    CompletionItemKind,
    InsertTextFormat,
    MarkupKind,
    SignatureHelp,
    SignatureInformation,
    ParameterInformation,
    Position,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { DocumentAnalysis } from './features';
import {
    COBRA64_BUILTINS,
    COBRA64_TYPES,
    COBRA64_KEYWORDS,
    positionToOffset,
} from '../../shared/types';

/**
 * Completion context to determine what kind of completions to provide.
 */
interface CompletionContext {
    isAfterColon: boolean;      // After ":" for type annotation
    isAfterDef: boolean;        // After "def" keyword
    isInFunctionCall: boolean;  // Inside function call parentheses
    functionName: string | null; // Name of function being called
    parameterIndex: number;     // Which parameter we're at
    linePrefix: string;         // Text before cursor on current line
    wordPrefix: string;         // Current word being typed
}

/**
 * Analyze the context at the cursor position.
 */
function getCompletionContext(
    document: TextDocument,
    position: Position
): CompletionContext {
    const text = document.getText();
    const offset = positionToOffset(text, { line: position.line, character: position.character });

    // Get text from start of line to cursor
    const lineStart = text.lastIndexOf('\n', offset - 1) + 1;
    const linePrefix = text.slice(lineStart, offset);

    // Get current word prefix
    let wordStart = offset;
    while (wordStart > lineStart && /[a-zA-Z0-9_]/.test(text[wordStart - 1])) {
        wordStart--;
    }
    const wordPrefix = text.slice(wordStart, offset);

    // Check if after colon (type annotation)
    const colonMatch = linePrefix.match(/:\s*([a-zA-Z]*)$/);
    const isAfterColon = colonMatch !== null;

    // Check if after "def"
    const isAfterDef = /^\s*def\s+$/.test(linePrefix);

    // Check if inside function call
    let isInFunctionCall = false;
    let functionName: string | null = null;
    let parameterIndex = 0;

    // Count parentheses to determine if we're inside a function call
    let parenDepth = 0;
    let funcStart = -1;

    for (let i = offset - 1; i >= lineStart; i--) {
        const char = text[i];
        if (char === ')') {
            parenDepth++;
        } else if (char === '(') {
            if (parenDepth === 0) {
                isInFunctionCall = true;
                funcStart = i;
                break;
            }
            parenDepth--;
        }
    }

    if (isInFunctionCall && funcStart > lineStart) {
        // Extract function name
        let nameEnd = funcStart;
        let nameStart = funcStart - 1;
        while (nameStart >= lineStart && /[a-zA-Z0-9_]/.test(text[nameStart])) {
            nameStart--;
        }
        nameStart++;
        functionName = text.slice(nameStart, nameEnd);

        // Count commas to determine parameter index
        const argsText = text.slice(funcStart + 1, offset);
        parameterIndex = (argsText.match(/,/g) || []).length;
    }

    return {
        isAfterColon,
        isAfterDef,
        isInFunctionCall,
        functionName,
        parameterIndex,
        linePrefix,
        wordPrefix,
    };
}

/**
 * Get completion items for the current position.
 */
export function getCompletions(
    document: TextDocument,
    position: Position,
    analysis: DocumentAnalysis
): CompletionItem[] {
    const context = getCompletionContext(document, position);
    const items: CompletionItem[] = [];

    // After colon: suggest types
    if (context.isAfterColon) {
        items.push(...getTypeCompletions());
        return items;
    }

    // After "def": don't suggest anything (user types function name)
    if (context.isAfterDef) {
        return items;
    }

    // General completions
    items.push(...getKeywordCompletions(context.wordPrefix));
    items.push(...getTypeCompletions());
    items.push(...getBuiltinFunctionCompletions());
    items.push(...getUserFunctionCompletions(analysis, context.wordPrefix));
    items.push(...getVariableCompletions(analysis, context.wordPrefix));

    // Filter by prefix if present
    if (context.wordPrefix) {
        const prefix = context.wordPrefix.toLowerCase();
        return items.filter(item =>
            item.label.toLowerCase().startsWith(prefix)
        );
    }

    return items;
}

/**
 * Get keyword completions.
 */
function getKeywordCompletions(prefix: string): CompletionItem[] {
    const keywords = [
        { label: 'def', snippet: 'def ${1:name}(${2:params}):\n    ${0:pass}', doc: 'Define a function' },
        { label: 'if', snippet: 'if ${1:condition}:\n    ${0:pass}', doc: 'Conditional statement' },
        { label: 'elif', snippet: 'elif ${1:condition}:\n    ${0:pass}', doc: 'Else-if branch' },
        { label: 'else', snippet: 'else:\n    ${0:pass}', doc: 'Else branch' },
        { label: 'while', snippet: 'while ${1:condition}:\n    ${0:pass}', doc: 'While loop' },
        { label: 'for', snippet: 'for ${1:i} in ${2:0} to ${3:9}:\n    ${0:pass}', doc: 'For loop' },
        { label: 'break', snippet: 'break', doc: 'Exit the loop' },
        { label: 'continue', snippet: 'continue', doc: 'Skip to next iteration' },
        { label: 'return', snippet: 'return ${0}', doc: 'Return from function' },
        { label: 'pass', snippet: 'pass', doc: 'Do nothing (placeholder)' },
        { label: 'and', snippet: 'and ', doc: 'Logical AND' },
        { label: 'or', snippet: 'or ', doc: 'Logical OR' },
        { label: 'not', snippet: 'not ', doc: 'Logical NOT' },
        { label: 'true', snippet: 'true', doc: 'Boolean true' },
        { label: 'false', snippet: 'false', doc: 'Boolean false' },
        { label: 'in', snippet: 'in ', doc: 'Part of for loop' },
        { label: 'to', snippet: 'to ', doc: 'Ascending range' },
        { label: 'downto', snippet: 'downto ', doc: 'Descending range' },
    ];

    return keywords.map((kw, index) => ({
        label: kw.label,
        kind: CompletionItemKind.Keyword,
        insertText: kw.snippet,
        insertTextFormat: InsertTextFormat.Snippet,
        detail: kw.doc,
        sortText: `0${index.toString().padStart(2, '0')}`, // Keywords first
    }));
}

/**
 * Get type completions.
 */
function getTypeCompletions(): CompletionItem[] {
    const types = [
        { label: 'byte', doc: 'Unsigned 8-bit (0-255)' },
        { label: 'word', doc: 'Unsigned 16-bit (0-65535)' },
        { label: 'sbyte', doc: 'Signed 8-bit (-128 to 127)' },
        { label: 'sword', doc: 'Signed 16-bit (-32768 to 32767)' },
        { label: 'fixed', doc: 'Fixed-point (-2048.0 to 2047.9375)' },
        { label: 'float', doc: 'IEEE-754 binary16 (±65504)' },
        { label: 'bool', doc: 'Boolean (true/false)' },
        { label: 'string', doc: 'Text string' },
        // Array types
        { label: 'byte[]', doc: 'Byte array' },
        { label: 'word[]', doc: 'Word array' },
        { label: 'sbyte[]', doc: 'Signed byte array' },
        { label: 'sword[]', doc: 'Signed word array' },
        { label: 'bool[]', doc: 'Boolean array' },
        { label: 'fixed[]', doc: 'Fixed-point array (12.4 format)' },
        { label: 'float[]', doc: 'IEEE-754 float array' },
    ];

    return types.map((t, index) => ({
        label: t.label,
        kind: CompletionItemKind.TypeParameter,
        detail: t.doc,
        sortText: `1${index.toString().padStart(2, '0')}`, // Types second
    }));
}

/**
 * Get built-in function completions.
 */
function getBuiltinFunctionCompletions(): CompletionItem[] {
    return COBRA64_BUILTINS.map((builtin, index) => {
        // Create snippet with parameter placeholders
        let snippet = builtin.name + '(';
        if (builtin.parameters.length > 0) {
            snippet += builtin.parameters.map((p, i) =>
                `\${${i + 1}:${p.name}}`
            ).join(', ');
        }
        snippet += ')';

        const paramList = builtin.parameters.map(p => `${p.name}: ${p.type}`).join(', ');
        const signature = `${builtin.name}(${paramList})${builtin.returnType ? ' -> ' + builtin.returnType : ''}`;

        return {
            label: builtin.name,
            kind: CompletionItemKind.Function,
            insertText: snippet,
            insertTextFormat: InsertTextFormat.Snippet,
            detail: signature,
            documentation: {
                kind: MarkupKind.Markdown,
                value: `**${builtin.name}** (built-in)\n\n${builtin.description}`,
            },
            sortText: `2${index.toString().padStart(2, '0')}`, // Built-ins third
        };
    });
}

/**
 * Get user-defined function completions.
 */
function getUserFunctionCompletions(
    analysis: DocumentAnalysis,
    prefix: string
): CompletionItem[] {
    if (!analysis.analyzerResult) return [];

    return analysis.analyzerResult.functions
        .filter(f => !f.isBuiltin)
        .map((func, index) => {
            // Create snippet with parameter placeholders
            let snippet = func.name + '(';
            if (func.params.length > 0) {
                snippet += func.params.map((p, i) =>
                    `\${${i + 1}:${p.name}}`
                ).join(', ');
            }
            snippet += ')';

            const paramList = func.params.map(p => `${p.name}: ${p.type}`).join(', ');
            const signature = `${func.name}(${paramList})${func.returnType ? ' -> ' + func.returnType : ''}`;

            return {
                label: func.name,
                kind: CompletionItemKind.Function,
                insertText: snippet,
                insertTextFormat: InsertTextFormat.Snippet,
                detail: signature,
                sortText: `3${index.toString().padStart(2, '0')}`, // User functions fourth
            };
        });
}

/**
 * Get variable completions from analysis.
 */
function getVariableCompletions(
    analysis: DocumentAnalysis,
    prefix: string
): CompletionItem[] {
    if (!analysis.analyzerResult) return [];

    return analysis.analyzerResult.symbols
        .filter(s => s.kind === 'variable' || s.kind === 'constant' || s.kind === 'parameter')
        .map((symbol, index) => ({
            label: symbol.name,
            kind: symbol.kind === 'constant' ? CompletionItemKind.Constant :
                symbol.kind === 'parameter' ? CompletionItemKind.Variable :
                    CompletionItemKind.Variable,
            detail: symbol.type,
            sortText: `4${index.toString().padStart(2, '0')}`, // Variables last
        }));
}

/**
 * Resolve additional details for a completion item.
 */
export function resolveCompletionItem(item: CompletionItem): CompletionItem {
    // Add more documentation if needed
    const builtin = COBRA64_BUILTINS.find(b => b.name === item.label);
    if (builtin && !item.documentation) {
        const params = builtin.parameters.map(p =>
            `- \`${p.name}: ${p.type}\` - ${p.description}`
        ).join('\n');

        item.documentation = {
            kind: MarkupKind.Markdown,
            value: `**${builtin.name}**\n\n${builtin.description}${params ? '\n\n**Parameters:**\n' + params : ''}${builtin.returnType ? '\n\n**Returns:** `' + builtin.returnType + '`' : ''}`,
        };
    }

    return item;
}

/**
 * Get signature help for function calls.
 */
export function getSignatureHelp(
    document: TextDocument,
    position: Position,
    analysis: DocumentAnalysis
): SignatureHelp | null {
    const context = getCompletionContext(document, position);

    if (!context.isInFunctionCall || !context.functionName) {
        return null;
    }

    // Find the function
    const builtin = COBRA64_BUILTINS.find(b => b.name === context.functionName);
    if (builtin) {
        return createSignatureHelp(
            builtin.name,
            builtin.parameters.map(p => ({ name: p.name, type: p.type, doc: p.description })),
            builtin.returnType,
            builtin.description,
            context.parameterIndex
        );
    }

    // Check user functions
    if (analysis.analyzerResult) {
        const userFunc = analysis.analyzerResult.functions.find(
            f => f.name === context.functionName && !f.isBuiltin
        );
        if (userFunc) {
            return createSignatureHelp(
                userFunc.name,
                userFunc.params.map(p => ({ name: p.name, type: p.type, doc: '' })),
                userFunc.returnType,
                '',
                context.parameterIndex
            );
        }
    }

    return null;
}

/**
 * Create a SignatureHelp object.
 */
function createSignatureHelp(
    name: string,
    params: { name: string; type: string; doc: string }[],
    returnType: string | null,
    description: string,
    activeParameter: number
): SignatureHelp {
    const paramLabels = params.map(p => `${p.name}: ${p.type}`);
    const signatureLabel = `${name}(${paramLabels.join(', ')})${returnType ? ' -> ' + returnType : ''}`;

    // Calculate parameter label offsets
    const parameters: ParameterInformation[] = [];
    let currentOffset = name.length + 1; // After "name("

    for (let i = 0; i < params.length; i++) {
        const paramLabel = paramLabels[i];
        const startOffset = currentOffset;
        const endOffset = currentOffset + paramLabel.length;

        parameters.push({
            label: [startOffset, endOffset],
            documentation: params[i].doc || undefined,
        });

        currentOffset = endOffset + 2; // Skip ", "
    }

    const signature: SignatureInformation = {
        label: signatureLabel,
        documentation: description ? {
            kind: MarkupKind.Markdown,
            value: description,
        } : undefined,
        parameters,
    };

    return {
        signatures: [signature],
        activeSignature: 0,
        activeParameter: Math.min(activeParameter, params.length - 1),
    };
}
