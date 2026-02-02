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
import { TextDocument } from 'vscode-languageserver-textdocument';
import { CompletionItemKind } from 'vscode-languageserver/node';
import { getCompletions, getSignatureHelp } from '../completion';
import { tokenize } from '../lexer';
import { parse } from '../parser';
import { analyze } from '../analyzer';

function createTestDocument(content: string): TextDocument {
    return TextDocument.create('file:///test.cb64', 'cobra64', 1, content);
}

function getAnalysis(content: string) {
    const lexerResult = tokenize(content);
    const parserResult = parse(lexerResult.tokens);
    const analyzerResult = parserResult.program ? analyze(parserResult.program) : null;

    return {
        program: parserResult.program,
        analyzerResult,
        tokens: lexerResult.tokens,
    };
}

describe('Completion', () => {
    describe('getCompletions', () => {
        it('should provide keyword completions', () => {
            const doc = createTestDocument('def main():\n    ');
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 1, character: 4 }, analysis);

            const keywords = completions.filter(c => c.kind === CompletionItemKind.Keyword);
            assert.ok(keywords.length > 0);

            const defCompletion = keywords.find(k => k.label === 'def');
            assert.ok(defCompletion, 'Should have def keyword');

            const ifCompletion = keywords.find(k => k.label === 'if');
            assert.ok(ifCompletion, 'Should have if keyword');
        });

        it('should provide type completions after colon', () => {
            const doc = createTestDocument('x:');
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 0, character: 2 }, analysis);

            const types = completions.filter(c => c.kind === CompletionItemKind.TypeParameter);
            assert.ok(types.length > 0);

            const byteType = types.find(t => t.label === 'byte');
            assert.ok(byteType, 'Should have byte type');

            const wordType = types.find(t => t.label === 'word');
            assert.ok(wordType, 'Should have word type');
        });

        it('should provide built-in function completions', () => {
            const doc = createTestDocument('def main():\n    ');
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 1, character: 4 }, analysis);

            const functions = completions.filter(c => c.kind === CompletionItemKind.Function);
            assert.ok(functions.length > 0);

            const printlnFunc = functions.find(f => f.label === 'println');
            assert.ok(printlnFunc, 'Should have println function');

            const clsFunc = functions.find(f => f.label === 'cls');
            assert.ok(clsFunc, 'Should have cls function');
        });

        it('should provide user function completions', () => {
            const code = 'def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    ';
            const doc = createTestDocument(code);
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 4, character: 4 }, analysis);

            const functions = completions.filter(c => c.kind === CompletionItemKind.Function);
            const addFunc = functions.find(f => f.label === 'add');
            assert.ok(addFunc, 'Should have user-defined add function');
        });

        it('should provide variable completions', () => {
            const code = 'counter: byte = 0\n\ndef main():\n    ';
            const doc = createTestDocument(code);
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 3, character: 4 }, analysis);

            const variables = completions.filter(c =>
                c.kind === CompletionItemKind.Variable ||
                c.kind === CompletionItemKind.Constant
            );
            const counterVar = variables.find(v => v.label === 'counter');
            assert.ok(counterVar, 'Should have counter variable');
        });

        it('should filter completions by prefix', () => {
            const doc = createTestDocument('def main():\n    pr');
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 1, character: 6 }, analysis);

            // All completions should start with 'pr'
            const filtered = completions.filter(c => c.label.toLowerCase().startsWith('pr'));
            assert.ok(filtered.length > 0);
            assert.ok(filtered.some(c => c.label === 'print'));
            assert.ok(filtered.some(c => c.label === 'println'));
        });

        it('should not suggest after def keyword', () => {
            const doc = createTestDocument('def ');
            const analysis = getAnalysis(doc.getText());
            const completions = getCompletions(doc, { line: 0, character: 4 }, analysis);

            // Should return empty - user is naming a function
            assert.strictEqual(completions.length, 0);
        });
    });

    describe('getSignatureHelp', () => {
        it('should provide signature for built-in function', () => {
            const doc = createTestDocument('def main():\n    println(');
            const analysis = getAnalysis(doc.getText());
            const help = getSignatureHelp(doc, { line: 1, character: 12 }, analysis);

            assert.ok(help, 'Should provide signature help');
            assert.strictEqual(help.signatures.length, 1);
            assert.ok(help.signatures[0].label.includes('println'));
        });

        it('should provide signature for user function', () => {
            const code = 'def add(a: byte, b: byte) -> byte:\n    return a + b\n\ndef main():\n    add(';
            const doc = createTestDocument(code);
            const analysis = getAnalysis(doc.getText());
            const help = getSignatureHelp(doc, { line: 4, character: 8 }, analysis);

            assert.ok(help, 'Should provide signature help');
            assert.strictEqual(help.signatures.length, 1);
            assert.ok(help.signatures[0].label.includes('add'));
            assert.ok(help.signatures[0].parameters);
            assert.strictEqual(help.signatures[0].parameters!.length, 2);
        });

        it('should highlight correct parameter', () => {
            const code = 'def main():\n    cursor(1, ';
            const doc = createTestDocument(code);
            const analysis = getAnalysis(doc.getText());
            const help = getSignatureHelp(doc, { line: 1, character: 14 }, analysis);

            assert.ok(help, 'Should provide signature help');
            assert.strictEqual(help.activeParameter, 1); // Second parameter
        });

        it('should return null when not in function call', () => {
            const doc = createTestDocument('def main():\n    x = 1');
            const analysis = getAnalysis(doc.getText());
            const help = getSignatureHelp(doc, { line: 1, character: 8 }, analysis);

            assert.strictEqual(help, null);
        });
    });
});
