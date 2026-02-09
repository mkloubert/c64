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

import { Token, TokenType } from './lexer';
import { Span, CompilerDiagnostic, DiagnosticSeverity } from '../../shared/types';

/**
 * AST Node types.
 */
export type AstNode =
    | Program
    | FunctionDef
    | VarDecl
    | ConstDecl
    | Assignment
    | IfStatement
    | WhileStatement
    | ForStatement
    | ReturnStatement
    | BreakStatement
    | ContinueStatement
    | PassStatement
    | ExpressionStatement
    | Expression;

export interface Program {
    kind: 'Program';
    items: TopLevelItem[];
    span: Span;
}

export type TopLevelItem = FunctionDef | VarDecl | ConstDecl | DataBlockDef;

export interface FunctionDef {
    kind: 'FunctionDef';
    name: string;
    params: Parameter[];
    returnType: string | null;
    body: Statement[];
    span: Span;
}

export interface Parameter {
    name: string;
    type: string;
    span: Span;
}

export interface VarDecl {
    kind: 'VarDecl';
    name: string;
    type: string | null;
    arraySize: number | null;
    initializer: Expression | null;
    span: Span;
}

export interface ConstDecl {
    kind: 'ConstDecl';
    name: string;
    type: string | null;
    value: Expression;
    span: Span;
}

export interface DataBlockDef {
    kind: 'DataBlockDef';
    name: string;
    entries: DataEntry[];
    span: Span;
}

export type DataEntry =
    | DataEntryBytes
    | DataEntryInclude;

export interface DataEntryBytes {
    kind: 'DataEntryBytes';
    values: number[];
    span: Span;
}

export interface DataEntryInclude {
    kind: 'DataEntryInclude';
    path: string;
    offset: number | null;
    length: number | null;
    span: Span;
}

export type Statement =
    | VarDecl
    | Assignment
    | IfStatement
    | WhileStatement
    | ForStatement
    | ReturnStatement
    | BreakStatement
    | ContinueStatement
    | PassStatement
    | ExpressionStatement;

export interface Assignment {
    kind: 'Assignment';
    target: string;
    index: Expression | null; // For array assignment
    operator: string;
    value: Expression;
    span: Span;
}

export interface IfStatement {
    kind: 'IfStatement';
    condition: Expression;
    thenBranch: Statement[];
    elifBranches: { condition: Expression; body: Statement[] }[];
    elseBranch: Statement[] | null;
    span: Span;
}

export interface WhileStatement {
    kind: 'WhileStatement';
    condition: Expression;
    body: Statement[];
    span: Span;
}

export interface ForStatement {
    kind: 'ForStatement';
    variable: string;
    start: Expression;
    end: Expression;
    direction: 'to' | 'downto';
    body: Statement[];
    span: Span;
}

export interface ReturnStatement {
    kind: 'ReturnStatement';
    value: Expression | null;
    span: Span;
}

export interface BreakStatement {
    kind: 'BreakStatement';
    span: Span;
}

export interface ContinueStatement {
    kind: 'ContinueStatement';
    span: Span;
}

export interface PassStatement {
    kind: 'PassStatement';
    span: Span;
}

export interface ExpressionStatement {
    kind: 'ExpressionStatement';
    expression: Expression;
    span: Span;
}

export type Expression =
    | IntegerLiteral
    | DecimalLiteral
    | StringLiteral
    | CharLiteral
    | BoolLiteral
    | Identifier
    | ArrayLiteral
    | ArrayIndex
    | FunctionCall
    | UnaryOp
    | BinaryOp
    | TypeCast;

export interface IntegerLiteral {
    kind: 'IntegerLiteral';
    value: number;
    raw: string;
    span: Span;
}

export interface DecimalLiteral {
    kind: 'DecimalLiteral';
    value: number;
    span: Span;
}

export interface StringLiteral {
    kind: 'StringLiteral';
    value: string;
    span: Span;
}

export interface CharLiteral {
    kind: 'CharLiteral';
    value: string;
    span: Span;
}

export interface BoolLiteral {
    kind: 'BoolLiteral';
    value: boolean;
    span: Span;
}

export interface Identifier {
    kind: 'Identifier';
    name: string;
    span: Span;
}

export interface ArrayLiteral {
    kind: 'ArrayLiteral';
    elements: Expression[];
    span: Span;
}

export interface ArrayIndex {
    kind: 'ArrayIndex';
    array: Expression;
    index: Expression;
    span: Span;
}

export interface FunctionCall {
    kind: 'FunctionCall';
    name: string;
    args: Expression[];
    span: Span;
}

export interface UnaryOp {
    kind: 'UnaryOp';
    operator: string;
    operand: Expression;
    span: Span;
}

export interface BinaryOp {
    kind: 'BinaryOp';
    operator: string;
    left: Expression;
    right: Expression;
    span: Span;
}

export interface TypeCast {
    kind: 'TypeCast';
    targetType: string;
    expression: Expression;
    span: Span;
}

/**
 * Parser result containing AST and any errors.
 */
export interface ParserResult {
    program: Program | null;
    diagnostics: CompilerDiagnostic[];
}

/**
 * Parser for Cobra64 source code.
 */
export class Parser {
    private tokens: Token[];
    private pos: number = 0;
    private diagnostics: CompilerDiagnostic[] = [];

    constructor(tokens: Token[]) {
        // Filter out comments and newlines (except for structure)
        this.tokens = tokens.filter(t =>
            t.type !== TokenType.Comment
        );
    }

    /**
     * Parse the tokens into an AST.
     */
    parse(): ParserResult {
        try {
            const items: TopLevelItem[] = [];
            const start = this.current().span.start;

            this.skipNewlines();

            while (!this.isAtEnd()) {
                const item = this.parseTopLevelItem();
                if (item) {
                    items.push(item);
                }
                this.skipNewlines();
            }

            const end = this.previous().span.end;

            return {
                program: {
                    kind: 'Program',
                    items,
                    span: { start, end },
                },
                diagnostics: this.diagnostics,
            };
        } catch (e) {
            return {
                program: null,
                diagnostics: this.diagnostics,
            };
        }
    }

    private parseTopLevelItem(): TopLevelItem | null {
        this.skipNewlines();

        if (this.check(TokenType.Keyword)) {
            if (this.current().value === 'def') {
                return this.parseFunctionDef();
            }
            if (this.current().value === 'data') {
                return this.parseDataBlock();
            }
        }

        if (this.check(TokenType.Identifier)) {
            return this.parseTopLevelDeclaration();
        }

        // Skip unexpected tokens
        if (!this.isAtEnd()) {
            this.addError(this.current().span, 'E100',
                `Unexpected token '${this.current().value}' at top level`);
            this.advance();
        }

        return null;
    }

    private parseFunctionDef(): FunctionDef {
        const start = this.current().span.start;
        this.advance(); // 'def'

        if (!this.check(TokenType.Identifier)) {
            this.addError(this.current().span, 'E101', 'Expected function name after def');
            return this.createDummyFunction(start);
        }

        const name = this.current().value;
        this.advance();

        if (!this.match(TokenType.LeftParen)) {
            this.addError(this.current().span, 'E103', "Expected '(' after function name");
        }

        const params = this.parseParameters();

        if (!this.match(TokenType.RightParen)) {
            this.addError(this.current().span, 'E103', "Expected ')' after parameters");
        }

        let returnType: string | null = null;
        if (this.match(TokenType.Arrow)) {
            if (this.check(TokenType.Type)) {
                returnType = this.current().value;
                this.advance();
            } else {
                this.addError(this.current().span, 'E101', 'Expected return type after ->');
            }
        }

        if (!this.match(TokenType.Colon)) {
            this.addError(this.current().span, 'E102', "Expected ':' after function signature");
        }

        this.skipNewlines();

        const body = this.parseBlock();

        return {
            kind: 'FunctionDef',
            name,
            params,
            returnType,
            body,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseDataBlock(): DataBlockDef {
        const start = this.current().span.start;
        this.advance(); // 'data'

        if (!this.check(TokenType.Identifier)) {
            this.addError(this.current().span, 'E101', 'Expected data block name after data');
            return this.createDummyDataBlock(start);
        }

        const name = this.current().value;
        this.advance();

        if (!this.match(TokenType.Colon)) {
            this.addError(this.current().span, 'E102', "Expected ':' after data block name");
        }

        this.skipNewlinesAndIndent();

        const entries: DataEntry[] = [];

        // Parse data entries until we see 'end' or Dedent
        while (!this.isAtEnd()) {
            this.skipNewlinesAndIndent();

            // Check for end of data block
            if (this.check(TokenType.Keyword) && this.current().value === 'end') {
                break;
            }
            if (this.check(TokenType.Dedent)) {
                this.advance();
                break;
            }

            const entry = this.parseDataEntry();
            if (entry) {
                entries.push(entry);
            } else {
                // If we couldn't parse an entry, skip to next line to avoid infinite loop
                if (!this.isAtEnd() && !this.check(TokenType.Newline) &&
                    !this.check(TokenType.Dedent) &&
                    !(this.check(TokenType.Keyword) && this.current().value === 'end')) {
                    this.advance();
                }
            }
        }

        // Skip any remaining dedent
        this.match(TokenType.Dedent);

        if (this.check(TokenType.Keyword) && this.current().value === 'end') {
            this.advance(); // 'end'
        }

        return {
            kind: 'DataBlockDef',
            name,
            entries,
            span: { start, end: this.previous().span.end },
        };
    }

    private skipNewlinesAndIndent(): void {
        while (this.check(TokenType.Newline) || this.check(TokenType.Indent)) {
            this.advance();
        }
    }

    private parseDataEntry(): DataEntry | null {
        const start = this.current().span.start;

        // Check for include directive
        if (this.check(TokenType.Keyword) && this.current().value === 'include') {
            this.advance(); // 'include'

            if (!this.check(TokenType.String)) {
                this.addError(this.current().span, 'E101', 'Expected filename string after include');
                return null;
            }

            // Extract path from string (remove quotes)
            const raw = this.current().value;
            const path = raw.slice(1, -1);
            this.advance();

            let offset: number | null = null;
            let length: number | null = null;

            // Check for offset
            if (this.match(TokenType.Comma)) {
                if (this.check(TokenType.Integer)) {
                    offset = this.parseIntegerValue(this.current().value);
                    this.advance();
                }

                // Check for length
                if (this.match(TokenType.Comma)) {
                    if (this.check(TokenType.Integer)) {
                        length = this.parseIntegerValue(this.current().value);
                        this.advance();
                    }
                }
            }

            return {
                kind: 'DataEntryInclude',
                path,
                offset,
                length,
                span: { start, end: this.previous().span.end },
            };
        }

        // Parse inline byte values
        const values: number[] = [];

        while (!this.isAtEnd() && !this.check(TokenType.Newline) &&
               !(this.check(TokenType.Keyword) && this.current().value === 'end')) {
            if (this.check(TokenType.Integer)) {
                values.push(this.parseIntegerValue(this.current().value));
                this.advance();
            } else if (this.match(TokenType.Comma)) {
                // Skip comma separator
                continue;
            } else {
                break;
            }
        }

        if (values.length === 0) {
            return null;
        }

        return {
            kind: 'DataEntryBytes',
            values,
            span: { start, end: this.previous().span.end },
        };
    }

    private createDummyDataBlock(start: number): DataBlockDef {
        return {
            kind: 'DataBlockDef',
            name: '__error__',
            entries: [],
            span: { start, end: this.previous().span.end },
        };
    }

    private parseParameters(): Parameter[] {
        const params: Parameter[] = [];
        const seenNames = new Set<string>();

        if (this.check(TokenType.RightParen)) {
            return params;
        }

        do {
            if (this.check(TokenType.Identifier)) {
                const paramStart = this.current().span.start;
                const name = this.current().value;
                this.advance();

                if (seenNames.has(name)) {
                    this.addError({ start: paramStart, end: this.previous().span.end },
                        'E104', `Duplicate parameter name '${name}'`);
                }
                seenNames.add(name);

                if (!this.match(TokenType.Colon)) {
                    this.addError(this.current().span, 'E102', "Expected ':' after parameter name");
                }

                let type = 'byte'; // Default type
                if (this.check(TokenType.Type)) {
                    type = this.current().value;
                    this.advance();

                    // Check for array type
                    if (this.match(TokenType.LeftBracket)) {
                        if (!this.match(TokenType.RightBracket)) {
                            this.addError(this.current().span, 'E103', "Expected ']' for array type");
                        }
                        type += '[]';
                    }
                } else {
                    this.addError(this.current().span, 'E101', 'Expected type for parameter');
                }

                params.push({
                    name,
                    type,
                    span: { start: paramStart, end: this.previous().span.end },
                });
            }
        } while (this.match(TokenType.Comma));

        return params;
    }

    private parseTopLevelDeclaration(): VarDecl | ConstDecl {
        const start = this.current().span.start;
        const name = this.current().value;
        this.advance();

        // Check if it's a constant (UPPERCASE naming)
        const isConstant = this.isConstantName(name);

        let type: string | null = null;
        let arraySize: number | null = null;

        // Type annotation is required (no type inference)
        if (this.match(TokenType.Colon)) {
            if (this.check(TokenType.Type)) {
                type = this.current().value;
                this.advance();

                // Check for array type
                if (this.match(TokenType.LeftBracket)) {
                    if (this.check(TokenType.Integer)) {
                        arraySize = this.parseIntegerValue(this.current().value);
                        this.advance();
                    }
                    if (!this.match(TokenType.RightBracket)) {
                        this.addError(this.current().span, 'E103', "Expected ']'");
                    }
                    type += '[]';
                }
            } else {
                this.addError(this.current().span, 'E101', 'Expected type after ":"');
            }
        } else {
            // No colon found - type annotation is required
            this.addError({ start, end: this.previous().span.end },
                'E147', `${isConstant ? 'Constant' : 'Variable'} declaration requires explicit type annotation`);
        }

        let initializer: Expression | null = null;
        if (this.match(TokenType.Assign)) {
            initializer = this.parseExpression();
        }

        if (isConstant) {
            if (!initializer) {
                this.addError({ start, end: this.previous().span.end },
                    'E200', 'Constants must have an initializer');
                initializer = { kind: 'IntegerLiteral', value: 0, raw: '0', span: { start, end: start } };
            }
            return {
                kind: 'ConstDecl',
                name,
                type,
                value: initializer,
                span: { start, end: this.previous().span.end },
            };
        }

        return {
            kind: 'VarDecl',
            name,
            type,
            arraySize,
            initializer,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseBlock(): Statement[] {
        const statements: Statement[] = [];

        if (!this.match(TokenType.Indent)) {
            this.addError(this.current().span, 'E105', 'Expected indented block');
            return statements;
        }

        while (!this.check(TokenType.Dedent) && !this.isAtEnd()) {
            this.skipNewlines();
            if (this.check(TokenType.Dedent) || this.isAtEnd()) break;

            const stmt = this.parseStatement();
            if (stmt) {
                statements.push(stmt);
            }
            this.skipNewlines();
        }

        this.match(TokenType.Dedent);

        return statements;
    }

    private parseStatement(): Statement | null {
        this.skipNewlines();

        if (this.check(TokenType.Keyword)) {
            const keyword = this.current().value;

            switch (keyword) {
                case 'if':
                    return this.parseIfStatement();
                case 'while':
                    return this.parseWhileStatement();
                case 'for':
                    return this.parseForStatement();
                case 'return':
                    return this.parseReturnStatement();
                case 'break':
                    return this.parseBreakStatement();
                case 'continue':
                    return this.parseContinueStatement();
                case 'pass':
                    return this.parsePassStatement();
            }
        }

        if (this.check(TokenType.Identifier)) {
            return this.parseIdentifierStatement();
        }

        // Expression statement
        const expr = this.parseExpression();
        return {
            kind: 'ExpressionStatement',
            expression: expr,
            span: expr.span,
        };
    }

    private parseIdentifierStatement(): Statement {
        const start = this.current().span.start;
        const name = this.current().value;
        this.advance();

        // Variable declaration
        if (this.match(TokenType.Colon)) {
            let type: string | null = null;
            let arraySize: number | null = null;

            if (this.check(TokenType.Type)) {
                type = this.current().value;
                this.advance();

                if (this.match(TokenType.LeftBracket)) {
                    if (this.check(TokenType.Integer)) {
                        arraySize = this.parseIntegerValue(this.current().value);
                        this.advance();
                    }
                    if (!this.match(TokenType.RightBracket)) {
                        this.addError(this.current().span, 'E103', "Expected ']'");
                    }
                    type += '[]';
                }
            } else {
                this.addError(this.current().span, 'E101', 'Expected type after ":"');
            }

            let initializer: Expression | null = null;
            if (this.match(TokenType.Assign)) {
                initializer = this.parseExpression();
            }

            return {
                kind: 'VarDecl',
                name,
                type,
                arraySize,
                initializer,
                span: { start, end: this.previous().span.end },
            };
        }

        // Array assignment
        if (this.match(TokenType.LeftBracket)) {
            const index = this.parseExpression();
            if (!this.match(TokenType.RightBracket)) {
                this.addError(this.current().span, 'E103', "Expected ']'");
            }

            const operator = this.parseAssignmentOperator();
            if (!operator) {
                this.addError(this.current().span, 'E100', 'Expected assignment operator');
                return this.createDummyAssignment(start, name);
            }

            const value = this.parseExpression();
            return {
                kind: 'Assignment',
                target: name,
                index,
                operator,
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // Simple assignment
        const operator = this.parseAssignmentOperator();
        if (operator) {
            const value = this.parseExpression();
            return {
                kind: 'Assignment',
                target: name,
                index: null,
                operator,
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // Function call
        if (this.check(TokenType.LeftParen)) {
            // Reparse as expression
            this.pos--; // Go back to identifier
            const expr = this.parseExpression();
            return {
                kind: 'ExpressionStatement',
                expression: expr,
                span: expr.span,
            };
        }

        this.addError(this.current().span, 'E100',
            `Unexpected token after identifier '${name}'`);
        return this.createDummyAssignment(start, name);
    }

    private parseAssignmentOperator(): string | null {
        const assignOps: TokenType[] = [
            TokenType.Assign,
            TokenType.PlusAssign,
            TokenType.MinusAssign,
            TokenType.StarAssign,
            TokenType.SlashAssign,
            TokenType.PercentAssign,
            TokenType.AmpersandAssign,
            TokenType.PipeAssign,
            TokenType.CaretAssign,
            TokenType.ShiftLeftAssign,
            TokenType.ShiftRightAssign,
        ];

        for (const op of assignOps) {
            if (this.check(op)) {
                const value = this.current().value;
                this.advance();
                return value;
            }
        }

        return null;
    }

    private parseIfStatement(): IfStatement {
        const start = this.current().span.start;
        this.advance(); // 'if'

        const condition = this.parseExpression();

        if (!this.match(TokenType.Colon)) {
            this.addError(this.current().span, 'E102', "Expected ':' after if condition");
        }

        this.skipNewlines();
        const thenBranch = this.parseBlock();

        const elifBranches: { condition: Expression; body: Statement[] }[] = [];

        this.skipNewlines();
        while (this.check(TokenType.Keyword) && this.current().value === 'elif') {
            this.advance(); // 'elif'
            const elifCondition = this.parseExpression();

            if (!this.match(TokenType.Colon)) {
                this.addError(this.current().span, 'E102', "Expected ':' after elif condition");
            }

            this.skipNewlines();
            const elifBody = this.parseBlock();
            elifBranches.push({ condition: elifCondition, body: elifBody });
            this.skipNewlines();
        }

        let elseBranch: Statement[] | null = null;
        if (this.check(TokenType.Keyword) && this.current().value === 'else') {
            this.advance(); // 'else'

            if (!this.match(TokenType.Colon)) {
                this.addError(this.current().span, 'E102', "Expected ':' after else");
            }

            this.skipNewlines();
            elseBranch = this.parseBlock();
        }

        return {
            kind: 'IfStatement',
            condition,
            thenBranch,
            elifBranches,
            elseBranch,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseWhileStatement(): WhileStatement {
        const start = this.current().span.start;
        this.advance(); // 'while'

        const condition = this.parseExpression();

        if (!this.match(TokenType.Colon)) {
            this.addError(this.current().span, 'E102', "Expected ':' after while condition");
        }

        this.skipNewlines();
        const body = this.parseBlock();

        return {
            kind: 'WhileStatement',
            condition,
            body,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseForStatement(): ForStatement {
        const start = this.current().span.start;
        this.advance(); // 'for'

        if (!this.check(TokenType.Identifier)) {
            this.addError(this.current().span, 'E101', 'Expected loop variable after for');
        }

        const variable = this.current().value;
        this.advance();

        if (!this.check(TokenType.Keyword) || this.current().value !== 'in') {
            this.addError(this.current().span, 'E100', "Expected 'in' after loop variable");
        }
        this.advance(); // 'in'

        const startExpr = this.parseExpression();

        let direction: 'to' | 'downto' = 'to';
        if (this.check(TokenType.Keyword)) {
            if (this.current().value === 'to') {
                direction = 'to';
                this.advance();
            } else if (this.current().value === 'downto') {
                direction = 'downto';
                this.advance();
            } else {
                this.addError(this.current().span, 'E100', "Expected 'to' or 'downto'");
            }
        }

        const endExpr = this.parseExpression();

        if (!this.match(TokenType.Colon)) {
            this.addError(this.current().span, 'E102', "Expected ':' after for range");
        }

        this.skipNewlines();
        const body = this.parseBlock();

        return {
            kind: 'ForStatement',
            variable,
            start: startExpr,
            end: endExpr,
            direction,
            body,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseReturnStatement(): ReturnStatement {
        const start = this.current().span.start;
        this.advance(); // 'return'

        let value: Expression | null = null;
        if (!this.check(TokenType.Newline) && !this.check(TokenType.Dedent) && !this.isAtEnd()) {
            value = this.parseExpression();
        }

        return {
            kind: 'ReturnStatement',
            value,
            span: { start, end: this.previous().span.end },
        };
    }

    private parseBreakStatement(): BreakStatement {
        const span = this.current().span;
        this.advance(); // 'break'
        return { kind: 'BreakStatement', span };
    }

    private parseContinueStatement(): ContinueStatement {
        const span = this.current().span;
        this.advance(); // 'continue'
        return { kind: 'ContinueStatement', span };
    }

    private parsePassStatement(): PassStatement {
        const span = this.current().span;
        this.advance(); // 'pass'
        return { kind: 'PassStatement', span };
    }

    private parseExpression(): Expression {
        return this.parseOr();
    }

    private parseOr(): Expression {
        let left = this.parseAnd();

        while (this.check(TokenType.Keyword) && this.current().value === 'or') {
            const operator = this.current().value;
            this.advance();
            const right = this.parseAnd();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseAnd(): Expression {
        let left = this.parseComparison();

        while (this.check(TokenType.Keyword) && this.current().value === 'and') {
            const operator = this.current().value;
            this.advance();
            const right = this.parseComparison();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseComparison(): Expression {
        let left = this.parseBitwiseOr();

        const compOps = [
            TokenType.Equal, TokenType.NotEqual,
            TokenType.Less, TokenType.Greater,
            TokenType.LessEqual, TokenType.GreaterEqual,
        ];

        while (compOps.some(op => this.check(op))) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseBitwiseOr();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseBitwiseOr(): Expression {
        let left = this.parseBitwiseXor();

        while (this.check(TokenType.Pipe)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseBitwiseXor();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseBitwiseXor(): Expression {
        let left = this.parseBitwiseAnd();

        while (this.check(TokenType.Caret)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseBitwiseAnd();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseBitwiseAnd(): Expression {
        let left = this.parseShift();

        while (this.check(TokenType.Ampersand)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseShift();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseShift(): Expression {
        let left = this.parseAdditive();

        while (this.check(TokenType.ShiftLeft) || this.check(TokenType.ShiftRight)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseAdditive();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseAdditive(): Expression {
        let left = this.parseMultiplicative();

        while (this.check(TokenType.Plus) || this.check(TokenType.Minus)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseMultiplicative();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseMultiplicative(): Expression {
        let left = this.parseUnary();

        while (this.check(TokenType.Star) || this.check(TokenType.Slash) || this.check(TokenType.Percent)) {
            const operator = this.current().value;
            this.advance();
            const right = this.parseUnary();
            left = {
                kind: 'BinaryOp',
                operator,
                left,
                right,
                span: { start: left.span.start, end: right.span.end },
            };
        }

        return left;
    }

    private parseUnary(): Expression {
        if (this.check(TokenType.Minus) || this.check(TokenType.Tilde) ||
            (this.check(TokenType.Keyword) && this.current().value === 'not')) {
            const start = this.current().span.start;
            const operator = this.current().value;
            this.advance();
            const operand = this.parseUnary();
            return {
                kind: 'UnaryOp',
                operator,
                operand,
                span: { start, end: operand.span.end },
            };
        }

        return this.parsePostfix();
    }

    private parsePostfix(): Expression {
        let expr = this.parsePrimary();

        while (true) {
            if (this.match(TokenType.LeftBracket)) {
                const index = this.parseExpression();
                if (!this.match(TokenType.RightBracket)) {
                    this.addError(this.current().span, 'E103', "Expected ']'");
                }
                expr = {
                    kind: 'ArrayIndex',
                    array: expr,
                    index,
                    span: { start: expr.span.start, end: this.previous().span.end },
                };
            } else if (this.match(TokenType.LeftParen)) {
                // Function call or type cast
                if (expr.kind === 'Identifier') {
                    const args = this.parseArguments();
                    if (!this.match(TokenType.RightParen)) {
                        this.addError(this.current().span, 'E103', "Expected ')'");
                    }

                    // Check if it's a type cast
                    if (this.isTypeName(expr.name) && args.length === 1) {
                        expr = {
                            kind: 'TypeCast',
                            targetType: expr.name,
                            expression: args[0],
                            span: { start: expr.span.start, end: this.previous().span.end },
                        };
                    } else {
                        expr = {
                            kind: 'FunctionCall',
                            name: expr.name,
                            args,
                            span: { start: expr.span.start, end: this.previous().span.end },
                        };
                    }
                } else {
                    this.addError(expr.span, 'E100', 'Expected function name');
                    this.parseArguments();
                    this.match(TokenType.RightParen);
                }
            } else {
                break;
            }
        }

        return expr;
    }

    private parsePrimary(): Expression {
        const start = this.current().span.start;

        // Integer literal
        if (this.check(TokenType.Integer)) {
            const raw = this.current().value;
            const value = this.parseIntegerValue(raw);
            this.advance();
            return {
                kind: 'IntegerLiteral',
                value,
                raw,
                span: { start, end: this.previous().span.end },
            };
        }

        // Decimal literal
        if (this.check(TokenType.Decimal)) {
            const value = parseFloat(this.current().value);
            this.advance();
            return {
                kind: 'DecimalLiteral',
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // String literal
        if (this.check(TokenType.String)) {
            const value = this.current().value;
            this.advance();
            return {
                kind: 'StringLiteral',
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // Character literal
        if (this.check(TokenType.Character)) {
            const value = this.current().value;
            this.advance();
            return {
                kind: 'CharLiteral',
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // Boolean literal
        if (this.check(TokenType.BoolLiteral)) {
            const value = this.current().value === 'true';
            this.advance();
            return {
                kind: 'BoolLiteral',
                value,
                span: { start, end: this.previous().span.end },
            };
        }

        // Identifier or type
        if (this.check(TokenType.Identifier) || this.check(TokenType.Type)) {
            const name = this.current().value;
            this.advance();
            return {
                kind: 'Identifier',
                name,
                span: { start, end: this.previous().span.end },
            };
        }

        // Array literal
        if (this.match(TokenType.LeftBracket)) {
            const elements: Expression[] = [];

            if (!this.check(TokenType.RightBracket)) {
                do {
                    elements.push(this.parseExpression());
                } while (this.match(TokenType.Comma));
            }

            if (!this.match(TokenType.RightBracket)) {
                this.addError(this.current().span, 'E103', "Expected ']'");
            }

            return {
                kind: 'ArrayLiteral',
                elements,
                span: { start, end: this.previous().span.end },
            };
        }

        // Parenthesized expression
        if (this.match(TokenType.LeftParen)) {
            const expr = this.parseExpression();
            if (!this.match(TokenType.RightParen)) {
                this.addError(this.current().span, 'E103', "Expected ')'");
            }
            return expr;
        }

        // Error - unexpected token
        this.addError(this.current().span, 'E101', `Expected expression, got '${this.current().value}'`);
        this.advance();
        return {
            kind: 'IntegerLiteral',
            value: 0,
            raw: '0',
            span: { start, end: start },
        };
    }

    private parseArguments(): Expression[] {
        const args: Expression[] = [];

        if (!this.check(TokenType.RightParen)) {
            do {
                args.push(this.parseExpression());
            } while (this.match(TokenType.Comma));
        }

        return args;
    }

    private parseIntegerValue(raw: string): number {
        if (raw.startsWith('$')) {
            return parseInt(raw.slice(1), 16);
        } else if (raw.startsWith('%')) {
            return parseInt(raw.slice(1), 2);
        }
        return parseInt(raw, 10);
    }

    private isConstantName(name: string): boolean {
        const firstLetter = name.match(/[a-zA-Z]/);
        if (!firstLetter) return false;
        return firstLetter[0] === firstLetter[0].toUpperCase() &&
            name.split('').filter(c => /[a-zA-Z]/.test(c)).every(c => c === c.toUpperCase());
    }

    private isTypeName(name: string): boolean {
        return ['byte', 'word', 'sbyte', 'sword', 'fixed', 'float', 'bool', 'string'].includes(name);
    }

    private skipNewlines(): void {
        while (this.check(TokenType.Newline)) {
            this.advance();
        }
    }

    private current(): Token {
        return this.tokens[this.pos] || { type: TokenType.EOF, value: '', span: { start: 0, end: 0 } };
    }

    private previous(): Token {
        return this.tokens[this.pos - 1] || this.current();
    }

    private check(type: TokenType): boolean {
        return this.current().type === type;
    }

    private match(type: TokenType): boolean {
        if (this.check(type)) {
            this.advance();
            return true;
        }
        return false;
    }

    private advance(): Token {
        if (!this.isAtEnd()) this.pos++;
        return this.previous();
    }

    private isAtEnd(): boolean {
        return this.current().type === TokenType.EOF;
    }

    private addError(span: Span, code: string, message: string): void {
        this.diagnostics.push({
            code,
            message,
            span,
            severity: DiagnosticSeverity.Error,
        });
    }

    private createDummyFunction(start: number): FunctionDef {
        return {
            kind: 'FunctionDef',
            name: '__error__',
            params: [],
            returnType: null,
            body: [],
            span: { start, end: this.previous().span.end },
        };
    }

    private createDummyAssignment(start: number, name: string): Assignment {
        return {
            kind: 'Assignment',
            target: name,
            index: null,
            operator: '=',
            value: { kind: 'IntegerLiteral', value: 0, raw: '0', span: { start, end: start } },
            span: { start, end: this.previous().span.end },
        };
    }
}

/**
 * Parse tokens into an AST.
 */
export function parse(tokens: Token[]): ParserResult {
    const parser = new Parser(tokens);
    return parser.parse();
}
