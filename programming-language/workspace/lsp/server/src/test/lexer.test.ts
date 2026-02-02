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

import * as assert from 'assert';
import { tokenize, TokenType } from '../lexer';

describe('Lexer', () => {
    describe('tokenize', () => {
        it('should tokenize keywords', () => {
            const result = tokenize('def if while for');
            const keywords = result.tokens.filter(t => t.type === TokenType.Keyword);
            assert.strictEqual(keywords.length, 4);
            assert.strictEqual(keywords[0].value, 'def');
            assert.strictEqual(keywords[1].value, 'if');
            assert.strictEqual(keywords[2].value, 'while');
            assert.strictEqual(keywords[3].value, 'for');
        });

        it('should tokenize type keywords', () => {
            const result = tokenize('byte word sbyte sword fixed float bool string');
            const types = result.tokens.filter(t => t.type === TokenType.Type);
            assert.strictEqual(types.length, 8);
        });

        it('should tokenize integer literals', () => {
            const result = tokenize('42 0 255');
            const integers = result.tokens.filter(t => t.type === TokenType.Integer);
            assert.strictEqual(integers.length, 3);
            assert.strictEqual(integers[0].value, '42');
            assert.strictEqual(integers[1].value, '0');
            assert.strictEqual(integers[2].value, '255');
        });

        it('should tokenize decimal literals', () => {
            const result = tokenize('3.14 0.5 123.456');
            const decimals = result.tokens.filter(t => t.type === TokenType.Decimal);
            assert.strictEqual(decimals.length, 3);
        });

        it('should tokenize hex literals', () => {
            const result = tokenize('$FF $0A $FFFF');
            const hexes = result.tokens.filter(t => t.type === TokenType.Integer);
            assert.strictEqual(hexes.length, 3);
        });

        it('should tokenize binary literals', () => {
            const result = tokenize('%1010 %11110000');
            const binaries = result.tokens.filter(t => t.type === TokenType.Integer);
            assert.strictEqual(binaries.length, 2);
        });

        it('should tokenize string literals', () => {
            const result = tokenize('"hello" "world"');
            const strings = result.tokens.filter(t => t.type === TokenType.String);
            assert.strictEqual(strings.length, 2);
            // Note: lexer keeps quotes in value, so check for quoted strings
            assert.ok(strings[0].value.includes('hello'));
            assert.ok(strings[1].value.includes('world'));
        });

        it('should tokenize character literals', () => {
            const result = tokenize("'a' 'b'");
            const chars = result.tokens.filter(t => t.type === TokenType.Character);
            assert.strictEqual(chars.length, 2);
        });

        it('should tokenize identifiers', () => {
            const result = tokenize('foo bar_baz my_var123');
            const ids = result.tokens.filter(t => t.type === TokenType.Identifier);
            assert.strictEqual(ids.length, 3);
            assert.strictEqual(ids[0].value, 'foo');
            assert.strictEqual(ids[1].value, 'bar_baz');
            assert.strictEqual(ids[2].value, 'my_var123');
        });

        it('should tokenize boolean literals', () => {
            const result = tokenize('true false');
            const bools = result.tokens.filter(t => t.type === TokenType.BoolLiteral);
            assert.strictEqual(bools.length, 2);
            assert.strictEqual(bools[0].value, 'true');
            assert.strictEqual(bools[1].value, 'false');
        });

        it('should tokenize operators', () => {
            const result = tokenize('+ - * / % == != < > <= >=');
            const operators = result.tokens.filter(t =>
                t.type === TokenType.Plus ||
                t.type === TokenType.Minus ||
                t.type === TokenType.Star ||
                t.type === TokenType.Slash ||
                t.type === TokenType.Percent ||
                t.type === TokenType.Equal ||
                t.type === TokenType.NotEqual ||
                t.type === TokenType.Less ||
                t.type === TokenType.Greater ||
                t.type === TokenType.LessEqual ||
                t.type === TokenType.GreaterEqual
            );
            assert.strictEqual(operators.length, 11);
        });

        it('should tokenize compound assignments', () => {
            const result = tokenize('+= -= *= /= %=');
            const compounds = result.tokens.filter(t =>
                t.type === TokenType.PlusAssign ||
                t.type === TokenType.MinusAssign ||
                t.type === TokenType.StarAssign ||
                t.type === TokenType.SlashAssign ||
                t.type === TokenType.PercentAssign
            );
            assert.strictEqual(compounds.length, 5);
        });

        it('should tokenize comments', () => {
            const result = tokenize('x = 1 # this is a comment\ny = 2');
            const comments = result.tokens.filter(t => t.type === TokenType.Comment);
            assert.strictEqual(comments.length, 1);
        });

        it('should track indentation', () => {
            const result = tokenize('def foo():\n    x = 1\n    y = 2\n');
            const indents = result.tokens.filter(t => t.type === TokenType.Indent);
            const dedents = result.tokens.filter(t => t.type === TokenType.Dedent);
            assert.strictEqual(indents.length, 1);
            assert.strictEqual(dedents.length, 1);
        });

        it('should report unterminated string error', () => {
            const result = tokenize('"unterminated');
            assert.ok(result.diagnostics.length > 0);
            assert.ok(result.diagnostics[0].message.includes('Unterminated'));
        });

        it('should report invalid character error', () => {
            const result = tokenize('@invalid');
            assert.ok(result.diagnostics.length > 0);
        });

        it('should handle empty input', () => {
            const result = tokenize('');
            assert.strictEqual(result.tokens.length, 1); // Just EOF
            assert.strictEqual(result.tokens[0].type, TokenType.EOF);
            assert.strictEqual(result.diagnostics.length, 0);
        });

        it('should handle escape sequences in strings', () => {
            const result = tokenize('"hello\\nworld"');
            const strings = result.tokens.filter(t => t.type === TokenType.String);
            assert.strictEqual(strings.length, 1);
            assert.ok(strings[0].value.includes('\\n'));
        });
    });
});
