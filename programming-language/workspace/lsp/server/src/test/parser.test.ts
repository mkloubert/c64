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
import { tokenize } from '../lexer';
import { parse } from '../parser';

describe('Parser', () => {
    function parseCode(code: string) {
        const lexerResult = tokenize(code);
        return parse(lexerResult.tokens);
    }

    describe('Variable Declarations', () => {
        it('should parse variable with type annotation', () => {
            const result = parseCode('x: byte = 42\ndef main():\n    pass');
            assert.ok(result.program);
            assert.strictEqual(result.program.items.length, 2);
            assert.strictEqual(result.program.items[0].kind, 'VarDecl');
            const varDecl = result.program.items[0] as any;
            assert.strictEqual(varDecl.name, 'x');
            assert.strictEqual(varDecl.type, 'byte');
        });

        it('should report error for variable without type', () => {
            const result = parseCode('x = 42\ndef main():\n    pass');
            assert.ok(result.program);
            // Variable is still parsed for error recovery
            const varDecl = result.program.items[0] as any;
            assert.strictEqual(varDecl.name, 'x');
            assert.strictEqual(varDecl.type, null);
            // But an error should be reported (E147)
            assert.ok(result.diagnostics.length > 0);
            assert.ok(result.diagnostics.some((d: any) => d.code === 'E147'));
        });
    });

    describe('Constant Declarations', () => {
        it('should parse constant declaration', () => {
            const result = parseCode('MAX_VALUE: byte = 255\ndef main():\n    pass');
            assert.ok(result.program);
            const constDecl = result.program.items[0] as any;
            assert.strictEqual(constDecl.kind, 'ConstDecl');
            assert.strictEqual(constDecl.name, 'MAX_VALUE');
        });
    });

    describe('Function Definitions', () => {
        it('should parse simple function', () => {
            const result = parseCode('def main():\n    pass');
            assert.ok(result.program);
            assert.strictEqual(result.program.items.length, 1);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.kind, 'FunctionDef');
            assert.strictEqual(func.name, 'main');
            assert.strictEqual(func.params.length, 0);
        });

        it('should parse function with parameters', () => {
            const result = parseCode('def add(a: byte, b: byte) -> byte:\n    return a + b\ndef main():\n    pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.kind, 'FunctionDef');
            assert.strictEqual(func.name, 'add');
            assert.strictEqual(func.params.length, 2);
            assert.strictEqual(func.params[0].name, 'a');
            assert.strictEqual(func.params[0].type, 'byte');
            assert.strictEqual(func.returnType, 'byte');
        });

        it('should parse function with no return type', () => {
            const result = parseCode('def greet():\n    println("Hello")\ndef main():\n    pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.returnType, null);
        });
    });

    describe('Statements', () => {
        it('should parse if statement', () => {
            const result = parseCode('def main():\n    if x > 0:\n        pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.body[0].kind, 'IfStatement');
        });

        it('should parse if-elif-else statement', () => {
            const result = parseCode('def main():\n    if x > 0:\n        pass\n    elif x < 0:\n        pass\n    else:\n        pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const ifStmt = func.body[0];
            assert.strictEqual(ifStmt.kind, 'IfStatement');
            assert.strictEqual(ifStmt.elifBranches.length, 1);
            assert.ok(ifStmt.elseBranch);
        });

        it('should parse while loop', () => {
            const result = parseCode('def main():\n    while x < 10:\n        x = x + 1');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.body[0].kind, 'WhileStatement');
        });

        it('should parse for loop with to', () => {
            const result = parseCode('def main():\n    for i in 0 to 9:\n        pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const forStmt = func.body[0];
            assert.strictEqual(forStmt.kind, 'ForStatement');
            assert.strictEqual(forStmt.variable, 'i');
            assert.strictEqual(forStmt.direction, 'to');
        });

        it('should parse for loop with downto', () => {
            const result = parseCode('def main():\n    for i in 10 downto 1:\n        pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const forStmt = func.body[0];
            assert.strictEqual(forStmt.direction, 'downto');
        });

        it('should parse return statement', () => {
            const result = parseCode('def add(a: byte, b: byte) -> byte:\n    return a + b\ndef main():\n    pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.strictEqual(func.body[0].kind, 'ReturnStatement');
        });

        it('should parse break statement', () => {
            const result = parseCode('def main():\n    while true:\n        break');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const whileStmt = func.body[0];
            assert.strictEqual(whileStmt.body[0].kind, 'BreakStatement');
        });

        it('should parse continue statement', () => {
            const result = parseCode('def main():\n    while true:\n        continue');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const whileStmt = func.body[0];
            assert.strictEqual(whileStmt.body[0].kind, 'ContinueStatement');
        });
    });

    describe('Expressions', () => {
        it('should parse binary expressions', () => {
            const result = parseCode('x = 1 + 2 * 3\ndef main():\n    pass');
            assert.ok(result.program);
            const varDecl = result.program.items[0] as any;
            assert.strictEqual(varDecl.initializer.kind, 'BinaryOp');
        });

        it('should parse function calls', () => {
            const result = parseCode('def main():\n    println("Hello")');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const exprStmt = func.body[0];
            assert.strictEqual(exprStmt.kind, 'ExpressionStatement');
            assert.strictEqual(exprStmt.expression.kind, 'FunctionCall');
            assert.strictEqual(exprStmt.expression.name, 'println');
        });

        it('should parse array literals', () => {
            const result = parseCode('arr: byte[] = [1, 2, 3]\ndef main():\n    pass');
            assert.ok(result.program);
            const varDecl = result.program.items[0] as any;
            assert.strictEqual(varDecl.initializer.kind, 'ArrayLiteral');
            assert.strictEqual(varDecl.initializer.elements.length, 3);
        });

        it('should parse array indexing', () => {
            const result = parseCode('arr: byte[] = [1, 2, 3]\ndef main():\n    x: byte = arr[0]');
            assert.ok(result.program);
            const func = result.program.items[1] as any; // main is second item
            assert.ok(func.body.length > 0, 'Function body should not be empty');
            const stmt = func.body[0];
            // Check if it's a variable declaration or assignment
            if (stmt.kind === 'VarDecl') {
                assert.ok(stmt.initializer, 'VarDecl should have initializer');
                assert.strictEqual(stmt.initializer.kind, 'ArrayIndex');
            } else if (stmt.kind === 'Assignment') {
                assert.strictEqual(stmt.value.kind, 'ArrayIndex');
            } else {
                assert.fail('Expected VarDecl or Assignment');
            }
        });

        it('should parse unary expressions', () => {
            const result = parseCode('def main():\n    x: sbyte = -5');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            assert.ok(func.body.length > 0);
            const varDecl = func.body[0];
            assert.ok(varDecl.initializer);
            assert.strictEqual(varDecl.initializer.kind, 'UnaryOp');
            assert.strictEqual(varDecl.initializer.operator, '-');
        });

        it('should parse logical expressions', () => {
            const result = parseCode('def main():\n    if x > 0 and x < 10:\n        pass');
            assert.ok(result.program);
            const func = result.program.items[0] as any;
            const ifStmt = func.body[0];
            assert.strictEqual(ifStmt.condition.kind, 'BinaryOp');
            assert.strictEqual(ifStmt.condition.operator, 'and');
        });
    });

    describe('Error Recovery', () => {
        it('should report syntax errors', () => {
            const result = parseCode('def ():\n    pass');
            assert.ok(result.diagnostics.length > 0);
        });

        it('should continue parsing after error', () => {
            const result = parseCode('def bad(\ndef main():\n    pass');
            // Should still find main function despite error
            assert.ok(result.program);
        });
    });

    describe('Data Blocks', () => {
        it('should parse simple data block', () => {
            const result = parseCode('data SPRITE:\n    $00, $3C, $00\nend\ndef main():\n    pass');
            assert.ok(result.program);
            assert.strictEqual(result.program.items.length, 2);
            assert.strictEqual(result.program.items[0].kind, 'DataBlockDef');
            const dataBlock = result.program.items[0] as any;
            assert.strictEqual(dataBlock.name, 'SPRITE');
            assert.strictEqual(dataBlock.entries.length, 1);
            assert.strictEqual(dataBlock.entries[0].kind, 'DataEntryBytes');
            assert.strictEqual(dataBlock.entries[0].values.length, 3);
        });

        it('should parse data block with include', () => {
            const result = parseCode('data MUSIC:\n    include "song.sid"\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            assert.strictEqual(dataBlock.kind, 'DataBlockDef');
            assert.strictEqual(dataBlock.entries.length, 1);
            assert.strictEqual(dataBlock.entries[0].kind, 'DataEntryInclude');
            assert.strictEqual(dataBlock.entries[0].path, 'song.sid');
        });

        it('should parse data block include with offset', () => {
            const result = parseCode('data MUSIC:\n    include "song.sid", $7E\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            const entry = dataBlock.entries[0];
            assert.strictEqual(entry.kind, 'DataEntryInclude');
            assert.strictEqual(entry.offset, 0x7E);
            assert.strictEqual(entry.length, null);
        });

        it('should parse data block include with offset and length', () => {
            const result = parseCode('data MUSIC:\n    include "song.sid", $7E, $1000\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            const entry = dataBlock.entries[0];
            assert.strictEqual(entry.kind, 'DataEntryInclude');
            assert.strictEqual(entry.offset, 0x7E);
            assert.strictEqual(entry.length, 0x1000);
        });

        it('should parse data block with multiple lines', () => {
            const result = parseCode('data SPRITE:\n    $00, $3C, $00\n    $00, $7E, $00\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            // Each line becomes a separate entry
            assert.strictEqual(dataBlock.entries.length, 2);
        });

        it('should parse decimal values in data block', () => {
            const result = parseCode('data DATA:\n    0, 60, 255\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            assert.strictEqual(dataBlock.entries[0].values[0], 0);
            assert.strictEqual(dataBlock.entries[0].values[1], 60);
            assert.strictEqual(dataBlock.entries[0].values[2], 255);
        });

        it('should parse binary values in data block', () => {
            const result = parseCode('data DATA:\n    %11111111, %00000000\nend\ndef main():\n    pass');
            assert.ok(result.program);
            const dataBlock = result.program.items[0] as any;
            assert.strictEqual(dataBlock.entries[0].values[0], 255);
            assert.strictEqual(dataBlock.entries[0].values[1], 0);
        });
    });
});
