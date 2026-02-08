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

/**
 * Span represents a range in the source code using byte offsets.
 */
export interface Span {
    start: number;
    end: number;
}

/**
 * Position in a document (line and character are 0-based).
 */
export interface Position {
    line: number;
    character: number;
}

/**
 * Range in a document.
 */
export interface Range {
    start: Position;
    end: Position;
}

/**
 * Error codes from the Cobra64 compiler.
 * Lexical errors: E001-E026
 * Syntax errors: E100-E146
 * Semantic errors: E200-E246
 */
export enum ErrorCode {
    // Lexical errors
    InvalidCharacter = 'E001',
    UnterminatedString = 'E002',
    InvalidEscapeSequence = 'E003',
    NumberOverflow = 'E004',
    TabsNotAllowed = 'E005',
    IdentifierOnlyUnderscore = 'E026',

    // Syntax errors
    UnexpectedToken = 'E100',
    ExpectedExpression = 'E101',
    ExpectedColon = 'E102',
    ExpectedParenthesis = 'E103',
    DuplicateParameterName = 'E104',

    // Semantic errors
    UndefinedVariable = 'E200',
    UndefinedFunction = 'E201',
    TypeMismatch = 'E202',
    DuplicateDefinition = 'E203',
    NoMainFunction = 'E204',
    BreakOutsideLoop = 'E205',
    ValueOutOfRange = 'E206',
    WrongNumberOfArguments = 'E207',
}

/**
 * Severity level for diagnostics.
 */
export enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/**
 * A compiler diagnostic/error.
 */
export interface CompilerDiagnostic {
    code: string;
    message: string;
    span: Span;
    severity: DiagnosticSeverity;
    hint?: string;
}

/**
 * Symbol kind for document symbols.
 */
export enum SymbolKind {
    Function = 'function',
    Variable = 'variable',
    Constant = 'constant',
    Parameter = 'parameter',
}

/**
 * A symbol in the source code.
 */
export interface Symbol {
    name: string;
    kind: SymbolKind;
    type: string;
    span: Span;
    definitionSpan: Span;
    children?: Symbol[];
}

/**
 * Cobra64 data types.
 */
export const COBRA64_TYPES = [
    'byte',
    'word',
    'sbyte',
    'sword',
    'fixed',
    'float',
    'bool',
    'string',
] as const;

export type Cobra64Type = typeof COBRA64_TYPES[number];

/**
 * Cobra64 keywords.
 */
export const COBRA64_KEYWORDS = [
    'const',
    'def',
    'if',
    'elif',
    'else',
    'while',
    'for',
    'in',
    'to',
    'downto',
    'break',
    'continue',
    'return',
    'pass',
    'and',
    'or',
    'not',
    'true',
    'false',
] as const;

export type Cobra64Keyword = typeof COBRA64_KEYWORDS[number];

/**
 * Built-in function signature.
 */
export interface BuiltinFunction {
    name: string;
    signature: string;
    description: string;
    parameters: {
        name: string;
        type: string;
        description: string;
    }[];
    returnType: string | null;
}

/**
 * All built-in functions in Cobra64.
 */
export const COBRA64_BUILTINS: BuiltinFunction[] = [
    // Screen functions
    {
        name: 'cls',
        signature: 'cls()',
        description: 'Clears the screen.',
        parameters: [],
        returnType: null,
    },
    {
        name: 'cursor',
        signature: 'cursor(x: byte, y: byte)',
        description: 'Moves the cursor to position (x, y). x: Column (0-39), y: Row (0-24).',
        parameters: [
            { name: 'x', type: 'byte', description: 'Column (0-39)' },
            { name: 'y', type: 'byte', description: 'Row (0-24)' },
        ],
        returnType: null,
    },
    // Output functions
    {
        name: 'print',
        signature: 'print(value)',
        description: 'Prints a value without newline.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to print (string, number, bool)' },
        ],
        returnType: null,
    },
    {
        name: 'println',
        signature: 'println(value)',
        description: 'Prints a value followed by a newline.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to print (string, number, bool)' },
        ],
        returnType: null,
    },
    // Input functions
    {
        name: 'get_key',
        signature: 'get_key() -> byte',
        description: 'Returns the current key being pressed, or 0 if no key.',
        parameters: [],
        returnType: 'byte',
    },
    {
        name: 'read',
        signature: 'read() -> byte',
        description: 'Waits for a key press and returns it.',
        parameters: [],
        returnType: 'byte',
    },
    {
        name: 'readln',
        signature: 'readln() -> string',
        description: 'Reads a line of text input from the user.',
        parameters: [],
        returnType: 'string',
    },
    // Memory functions
    {
        name: 'poke',
        signature: 'poke(address: word, value: byte)',
        description: 'Writes a byte to a memory address.',
        parameters: [
            { name: 'address', type: 'word', description: 'Memory address (0-65535)' },
            { name: 'value', type: 'byte', description: 'Value to write (0-255)' },
        ],
        returnType: null,
    },
    {
        name: 'peek',
        signature: 'peek(address: word) -> byte',
        description: 'Reads a byte from a memory address.',
        parameters: [
            { name: 'address', type: 'word', description: 'Memory address (0-65535)' },
        ],
        returnType: 'byte',
    },
    // Array/String functions
    {
        name: 'len',
        signature: 'len(value) -> byte/word',
        description: 'Returns the length. For strings: returns byte (0-255). For arrays: returns word (0-65535).',
        parameters: [
            { name: 'value', type: 'array/string', description: 'Array or string to get length of' },
        ],
        returnType: 'byte/word',
    },
    // String functions
    {
        name: 'str_at',
        signature: 'str_at(s: string, i: byte) -> byte',
        description: 'Returns the character (byte value) at position i in the string.',
        parameters: [
            { name: 's', type: 'string', description: 'String to get character from' },
            { name: 'i', type: 'byte', description: 'Index (0-based)' },
        ],
        returnType: 'byte',
    },
    // Random functions
    {
        name: 'rand',
        signature: 'rand() -> fixed',
        description: 'Returns a random fixed-point number between 0.0 and 0.9375.',
        parameters: [],
        returnType: 'fixed',
    },
    {
        name: 'rand_byte',
        signature: 'rand_byte(from: byte, to: byte) -> byte',
        description: 'Returns a random byte in the range [from, to] (inclusive).',
        parameters: [
            { name: 'from', type: 'byte', description: 'Minimum value' },
            { name: 'to', type: 'byte', description: 'Maximum value' },
        ],
        returnType: 'byte',
    },
    {
        name: 'rand_sbyte',
        signature: 'rand_sbyte(from: sbyte, to: sbyte) -> sbyte',
        description: 'Returns a random signed byte in the range [from, to] (inclusive).',
        parameters: [
            { name: 'from', type: 'sbyte', description: 'Minimum value' },
            { name: 'to', type: 'sbyte', description: 'Maximum value' },
        ],
        returnType: 'sbyte',
    },
    {
        name: 'rand_word',
        signature: 'rand_word(from: word, to: word) -> word',
        description: 'Returns a random 16-bit word in the range [from, to] (inclusive).',
        parameters: [
            { name: 'from', type: 'word', description: 'Minimum value' },
            { name: 'to', type: 'word', description: 'Maximum value' },
        ],
        returnType: 'word',
    },
    {
        name: 'rand_sword',
        signature: 'rand_sword(from: sword, to: sword) -> sword',
        description: 'Returns a random signed 16-bit word in the range [from, to] (inclusive).',
        parameters: [
            { name: 'from', type: 'sword', description: 'Minimum value' },
            { name: 'to', type: 'sword', description: 'Maximum value' },
        ],
        returnType: 'sword',
    },
    {
        name: 'seed',
        signature: 'seed()',
        description: 'Reseeds the random number generator from hardware entropy sources.',
        parameters: [],
        returnType: null,
    },
    // Type conversion functions
    {
        name: 'str',
        signature: 'str(value) -> string',
        description: 'Converts a numeric value to its string representation.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert (byte, word, sbyte, sword, fixed, float, bool)' },
        ],
        returnType: 'string',
    },
    {
        name: 'bool',
        signature: 'bool(value) -> bool',
        description: 'Converts a value to boolean. Returns true if value is non-zero.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert to boolean' },
        ],
        returnType: 'bool',
    },
    {
        name: 'byte',
        signature: 'byte(value) -> byte',
        description: 'Converts a value to byte. Truncates larger values to 8 bits. Parses strings as numbers.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to convert (numeric or string)' },
        ],
        returnType: 'byte',
    },
    {
        name: 'word',
        signature: 'word(value) -> word',
        description: 'Converts a value to word. Truncates larger values to 16 bits. Parses strings as numbers.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to convert (numeric or string)' },
        ],
        returnType: 'word',
    },
    {
        name: 'sbyte',
        signature: 'sbyte(value) -> sbyte',
        description: 'Converts a value to signed byte.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'sbyte',
    },
    {
        name: 'sword',
        signature: 'sword(value) -> sword',
        description: 'Converts a value to signed word.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'sword',
    },
    {
        name: 'fixed',
        signature: 'fixed(value) -> fixed',
        description: 'Converts a value to fixed-point (12.4 format). Range: -2048.0 to 2047.9375.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'fixed',
    },
    {
        name: 'float',
        signature: 'float(value) -> float',
        description: 'Converts a value to IEEE-754 binary16 float.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'float',
    },
];

/**
 * Convert byte offset to line/column position.
 */
export function offsetToPosition(source: string, offset: number): Position {
    let line = 0;
    let character = 0;

    for (let i = 0; i < offset && i < source.length; i++) {
        if (source[i] === '\n') {
            line++;
            character = 0;
        } else {
            character++;
        }
    }

    return { line, character };
}

/**
 * Convert span to range using source text.
 */
export function spanToRange(source: string, span: Span): Range {
    return {
        start: offsetToPosition(source, span.start),
        end: offsetToPosition(source, span.end),
    };
}

/**
 * Convert line/column position to byte offset.
 */
export function positionToOffset(source: string, position: Position): number {
    let offset = 0;
    let line = 0;

    while (line < position.line && offset < source.length) {
        if (source[offset] === '\n') {
            line++;
        }
        offset++;
    }

    return offset + position.character;
}
