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

import { Span, CompilerDiagnostic, DiagnosticSeverity } from '../../shared/types';

/**
 * Token types for Cobra64 language.
 */
export enum TokenType {
    // Literals
    Integer,
    Decimal,
    String,
    Character,

    // Identifiers and keywords
    Identifier,
    Keyword,
    Type,
    BoolLiteral,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    ShiftLeft,
    ShiftRight,

    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Assignment
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    AmpersandAssign,
    PipeAssign,
    CaretAssign,
    ShiftLeftAssign,
    ShiftRightAssign,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Arrow,

    // Structure
    Newline,
    Indent,
    Dedent,
    Comment,

    // Special
    EOF,
    Error,
}

/**
 * Keywords in Cobra64.
 */
export const KEYWORDS = new Set([
    'def', 'if', 'elif', 'else', 'while', 'for', 'in', 'to', 'downto',
    'break', 'continue', 'return', 'pass', 'and', 'or', 'not',
    'data', 'end', 'include',
]);

/**
 * Type keywords in Cobra64.
 */
export const TYPE_KEYWORDS = new Set([
    'byte', 'word', 'sbyte', 'sword', 'fixed', 'float', 'bool', 'string',
]);

/**
 * Boolean literals.
 */
export const BOOL_LITERALS = new Set(['true', 'false']);

/**
 * A token from the lexer.
 */
export interface Token {
    type: TokenType;
    value: string;
    span: Span;
}

/**
 * Lexer result containing tokens and any errors.
 */
export interface LexerResult {
    tokens: Token[];
    diagnostics: CompilerDiagnostic[];
}

/**
 * Lexer for Cobra64 source code.
 */
export class Lexer {
    private source: string;
    private pos: number = 0;
    private tokens: Token[] = [];
    private diagnostics: CompilerDiagnostic[] = [];
    private indentStack: number[] = [0];
    private atLineStart: boolean = true;

    constructor(source: string) {
        this.source = source;
    }

    /**
     * Tokenize the source code.
     */
    tokenize(): LexerResult {
        while (!this.isAtEnd()) {
            this.scanToken();
        }

        // Emit remaining dedents at end of file
        while (this.indentStack.length > 1) {
            this.indentStack.pop();
            this.tokens.push({
                type: TokenType.Dedent,
                value: '',
                span: { start: this.pos, end: this.pos },
            });
        }

        this.tokens.push({
            type: TokenType.EOF,
            value: '',
            span: { start: this.pos, end: this.pos },
        });

        return {
            tokens: this.tokens,
            diagnostics: this.diagnostics,
        };
    }

    private isAtEnd(): boolean {
        return this.pos >= this.source.length;
    }

    private peek(): string {
        if (this.isAtEnd()) return '\0';
        return this.source[this.pos];
    }

    private peekNext(): string {
        if (this.pos + 1 >= this.source.length) return '\0';
        return this.source[this.pos + 1];
    }

    private advance(): string {
        return this.source[this.pos++];
    }

    private match(expected: string): boolean {
        if (this.isAtEnd()) return false;
        if (this.source[this.pos] !== expected) return false;
        this.pos++;
        return true;
    }

    private scanToken(): void {
        // Handle line start (indentation)
        if (this.atLineStart) {
            this.handleIndentation();
            this.atLineStart = false;
            if (this.isAtEnd()) return;
        }

        const start = this.pos;
        const c = this.advance();

        switch (c) {
            // Whitespace (not at line start)
            case ' ':
                break;

            // Tab - not allowed
            case '\t':
                this.addError(start, 'E005', 'Tabs are not allowed. Use 4 spaces for indentation.');
                break;

            // Newline
            case '\n':
                this.tokens.push({
                    type: TokenType.Newline,
                    value: '\n',
                    span: { start, end: this.pos },
                });
                this.atLineStart = true;
                break;

            case '\r':
                if (this.match('\n')) {
                    this.tokens.push({
                        type: TokenType.Newline,
                        value: '\r\n',
                        span: { start, end: this.pos },
                    });
                    this.atLineStart = true;
                }
                break;

            // Comment
            case '#':
                this.scanComment(start);
                break;

            // String
            case '"':
                this.scanString(start);
                break;

            // Character
            case "'":
                this.scanCharacter(start);
                break;

            // Operators and punctuation
            case '(':
                this.addToken(TokenType.LeftParen, '(', start);
                break;
            case ')':
                this.addToken(TokenType.RightParen, ')', start);
                break;
            case '[':
                this.addToken(TokenType.LeftBracket, '[', start);
                break;
            case ']':
                this.addToken(TokenType.RightBracket, ']', start);
                break;
            case ':':
                this.addToken(TokenType.Colon, ':', start);
                break;
            case ',':
                this.addToken(TokenType.Comma, ',', start);
                break;
            case '~':
                this.addToken(TokenType.Tilde, '~', start);
                break;

            case '+':
                if (this.match('=')) {
                    this.addToken(TokenType.PlusAssign, '+=', start);
                } else {
                    this.addToken(TokenType.Plus, '+', start);
                }
                break;

            case '-':
                if (this.match('=')) {
                    this.addToken(TokenType.MinusAssign, '-=', start);
                } else if (this.match('>')) {
                    this.addToken(TokenType.Arrow, '->', start);
                } else {
                    this.addToken(TokenType.Minus, '-', start);
                }
                break;

            case '*':
                if (this.match('=')) {
                    this.addToken(TokenType.StarAssign, '*=', start);
                } else {
                    this.addToken(TokenType.Star, '*', start);
                }
                break;

            case '/':
                if (this.match('=')) {
                    this.addToken(TokenType.SlashAssign, '/=', start);
                } else {
                    this.addToken(TokenType.Slash, '/', start);
                }
                break;

            case '%':
                if (this.match('=')) {
                    this.addToken(TokenType.PercentAssign, '%=', start);
                } else if (this.isDigit(this.peek())) {
                    // Binary literal
                    this.scanBinaryNumber(start);
                } else {
                    this.addToken(TokenType.Percent, '%', start);
                }
                break;

            case '&':
                if (this.match('=')) {
                    this.addToken(TokenType.AmpersandAssign, '&=', start);
                } else {
                    this.addToken(TokenType.Ampersand, '&', start);
                }
                break;

            case '|':
                if (this.match('=')) {
                    this.addToken(TokenType.PipeAssign, '|=', start);
                } else {
                    this.addToken(TokenType.Pipe, '|', start);
                }
                break;

            case '^':
                if (this.match('=')) {
                    this.addToken(TokenType.CaretAssign, '^=', start);
                } else {
                    this.addToken(TokenType.Caret, '^', start);
                }
                break;

            case '=':
                if (this.match('=')) {
                    this.addToken(TokenType.Equal, '==', start);
                } else {
                    this.addToken(TokenType.Assign, '=', start);
                }
                break;

            case '!':
                if (this.match('=')) {
                    this.addToken(TokenType.NotEqual, '!=', start);
                } else {
                    this.addError(start, 'E001', `Unexpected character '!'`);
                }
                break;

            case '<':
                if (this.match('<')) {
                    if (this.match('=')) {
                        this.addToken(TokenType.ShiftLeftAssign, '<<=', start);
                    } else {
                        this.addToken(TokenType.ShiftLeft, '<<', start);
                    }
                } else if (this.match('=')) {
                    this.addToken(TokenType.LessEqual, '<=', start);
                } else {
                    this.addToken(TokenType.Less, '<', start);
                }
                break;

            case '>':
                if (this.match('>')) {
                    if (this.match('=')) {
                        this.addToken(TokenType.ShiftRightAssign, '>>=', start);
                    } else {
                        this.addToken(TokenType.ShiftRight, '>>', start);
                    }
                } else if (this.match('=')) {
                    this.addToken(TokenType.GreaterEqual, '>=', start);
                } else {
                    this.addToken(TokenType.Greater, '>', start);
                }
                break;

            case '$':
                // Hexadecimal literal
                this.scanHexNumber(start);
                break;

            default:
                if (this.isDigit(c)) {
                    this.scanNumber(start);
                } else if (this.isAlpha(c)) {
                    this.scanIdentifier(start);
                } else {
                    this.addError(start, 'E001', `Unexpected character '${c}'`);
                }
                break;
        }
    }

    private handleIndentation(): void {
        let indent = 0;

        // Skip blank lines and comments
        while (!this.isAtEnd()) {
            if (this.peek() === ' ') {
                indent++;
                this.advance();
            } else if (this.peek() === '\t') {
                this.addError(this.pos, 'E005', 'Tabs are not allowed. Use 4 spaces for indentation.');
                this.advance();
                indent += 4; // Treat tab as 4 spaces for recovery
            } else if (this.peek() === '\n') {
                // Blank line - reset indent and emit newline
                indent = 0;
                return;
            } else if (this.peek() === '\r' && this.peekNext() === '\n') {
                indent = 0;
                return;
            } else if (this.peek() === '#') {
                // Comment line - don't change indentation
                return;
            } else {
                break;
            }
        }

        if (this.isAtEnd()) return;

        const currentIndent = this.indentStack[this.indentStack.length - 1];

        if (indent > currentIndent) {
            this.indentStack.push(indent);
            this.tokens.push({
                type: TokenType.Indent,
                value: '',
                span: { start: this.pos - indent, end: this.pos },
            });
        } else if (indent < currentIndent) {
            while (this.indentStack.length > 1 && this.indentStack[this.indentStack.length - 1] > indent) {
                this.indentStack.pop();
                this.tokens.push({
                    type: TokenType.Dedent,
                    value: '',
                    span: { start: this.pos - indent, end: this.pos },
                });
            }

            if (this.indentStack[this.indentStack.length - 1] !== indent) {
                this.addError(this.pos - indent, 'E006', 'Inconsistent indentation');
            }
        }
    }

    private scanComment(start: number): void {
        while (!this.isAtEnd() && this.peek() !== '\n') {
            this.advance();
        }
        this.addToken(TokenType.Comment, this.source.slice(start, this.pos), start);
    }

    private scanString(start: number): void {
        while (!this.isAtEnd() && this.peek() !== '"' && this.peek() !== '\n') {
            if (this.peek() === '\\') {
                this.advance(); // Skip backslash
                if (!this.isAtEnd()) {
                    const escaped = this.peek();
                    if (!['n', 'r', 't', '\\', '"', '0'].includes(escaped)) {
                        this.addError(this.pos, 'E003', `Invalid escape sequence '\\${escaped}'`);
                    }
                    this.advance(); // Skip escaped char
                }
            } else {
                this.advance();
            }
        }

        if (this.isAtEnd() || this.peek() === '\n') {
            this.addError(start, 'E002', 'Unterminated string literal');
            return;
        }

        this.advance(); // Closing "
        this.addToken(TokenType.String, this.source.slice(start, this.pos), start);
    }

    private scanCharacter(start: number): void {
        if (this.isAtEnd() || this.peek() === '\n') {
            this.addError(start, 'E002', 'Unterminated character literal');
            return;
        }

        if (this.peek() === '\\') {
            this.advance(); // Skip backslash
            if (!this.isAtEnd() && this.peek() !== '\n') {
                this.advance(); // Skip escaped char
            }
        } else {
            this.advance(); // The character
        }

        if (this.isAtEnd() || this.peek() !== "'") {
            this.addError(start, 'E002', 'Unterminated character literal');
            return;
        }

        this.advance(); // Closing '
        this.addToken(TokenType.Character, this.source.slice(start, this.pos), start);
    }

    private scanNumber(start: number): void {
        // Back up one position since we already consumed the first digit
        this.pos = start;

        while (this.isDigit(this.peek())) {
            this.advance();
        }

        // Check for decimal
        if (this.peek() === '.' && this.isDigit(this.peekNext())) {
            this.advance(); // Consume '.'
            while (this.isDigit(this.peek())) {
                this.advance();
            }

            // Check for exponent
            if (this.peek() === 'e' || this.peek() === 'E') {
                this.advance();
                if (this.peek() === '+' || this.peek() === '-') {
                    this.advance();
                }
                if (!this.isDigit(this.peek())) {
                    this.addError(start, 'E004', 'Invalid number: expected digits after exponent');
                    return;
                }
                while (this.isDigit(this.peek())) {
                    this.advance();
                }
            }

            this.addToken(TokenType.Decimal, this.source.slice(start, this.pos), start);
        } else {
            const value = this.source.slice(start, this.pos);
            const num = parseInt(value, 10);
            if (num > 65535) {
                this.addError(start, 'E004', `Number ${value} is too large (max 65535)`);
            }
            this.addToken(TokenType.Integer, value, start);
        }
    }

    private scanHexNumber(start: number): void {
        if (!this.isHexDigit(this.peek())) {
            this.addError(start, 'E004', 'Expected hexadecimal digits after $');
            return;
        }

        while (this.isHexDigit(this.peek())) {
            this.advance();
        }

        const value = this.source.slice(start, this.pos);
        const hexPart = value.slice(1); // Remove $
        const num = parseInt(hexPart, 16);
        if (num > 65535) {
            this.addError(start, 'E004', `Hexadecimal number ${value} is too large (max $FFFF)`);
        }
        this.addToken(TokenType.Integer, value, start);
    }

    private scanBinaryNumber(start: number): void {
        while (this.peek() === '0' || this.peek() === '1') {
            this.advance();
        }

        const value = this.source.slice(start, this.pos);
        const binPart = value.slice(1); // Remove %
        if (binPart.length === 0) {
            this.addError(start, 'E004', 'Expected binary digits after %');
            return;
        }
        const num = parseInt(binPart, 2);
        if (num > 65535) {
            this.addError(start, 'E004', `Binary number ${value} is too large (max 16 bits)`);
        }
        this.addToken(TokenType.Integer, value, start);
    }

    private scanIdentifier(start: number): void {
        // Back up one position since we already consumed the first character
        this.pos = start;

        while (this.isAlphaNumeric(this.peek())) {
            this.advance();
        }

        const value = this.source.slice(start, this.pos);

        // Check for keywords
        if (KEYWORDS.has(value)) {
            this.addToken(TokenType.Keyword, value, start);
        } else if (TYPE_KEYWORDS.has(value)) {
            this.addToken(TokenType.Type, value, start);
        } else if (BOOL_LITERALS.has(value)) {
            this.addToken(TokenType.BoolLiteral, value, start);
        } else {
            // Validate identifier naming convention
            this.validateIdentifier(value, start);
            this.addToken(TokenType.Identifier, value, start);
        }
    }

    private validateIdentifier(name: string, start: number): void {
        // Check for underscore-only names
        if (/^_+$/.test(name)) {
            this.addError(start, 'E027', 'Identifier cannot consist only of underscores');
            return;
        }

        // Find first letter
        const firstLetter = name.match(/[a-zA-Z]/);
        if (!firstLetter) return; // No letters, just underscores and numbers

        const isFirstUppercase = firstLetter[0] === firstLetter[0].toUpperCase();

        if (isFirstUppercase) {
            // If first letter is uppercase, all letters must be uppercase (constant)
            const letters = name.match(/[a-zA-Z]/g) || [];
            const allUppercase = letters.every(c => c === c.toUpperCase());
            if (!allUppercase) {
                this.addError(start, 'E026',
                    `Invalid identifier '${name}': if first letter is uppercase, all letters must be uppercase (constant naming convention)`);
            }
        }
    }

    private isDigit(c: string): boolean {
        return c >= '0' && c <= '9';
    }

    private isHexDigit(c: string): boolean {
        return this.isDigit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
    }

    private isAlpha(c: string): boolean {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c === '_';
    }

    private isAlphaNumeric(c: string): boolean {
        return this.isAlpha(c) || this.isDigit(c);
    }

    private addToken(type: TokenType, value: string, start: number): void {
        this.tokens.push({
            type,
            value,
            span: { start, end: this.pos },
        });
    }

    private addError(pos: number, code: string, message: string): void {
        this.diagnostics.push({
            code,
            message,
            span: { start: pos, end: pos + 1 },
            severity: DiagnosticSeverity.Error,
        });
    }
}

/**
 * Tokenize source code.
 */
export function tokenize(source: string): LexerResult {
    const lexer = new Lexer(source);
    return lexer.tokenize();
}
