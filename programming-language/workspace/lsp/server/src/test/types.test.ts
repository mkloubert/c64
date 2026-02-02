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
import {
    offsetToPosition,
    positionToOffset,
    spanToRange,
} from '../../../shared/types';

describe('Position Conversion', () => {
    describe('offsetToPosition', () => {
        it('should convert offset 0 to line 0, character 0', () => {
            const pos = offsetToPosition('hello', 0);
            assert.strictEqual(pos.line, 0);
            assert.strictEqual(pos.character, 0);
        });

        it('should convert mid-line offset correctly', () => {
            const pos = offsetToPosition('hello', 3);
            assert.strictEqual(pos.line, 0);
            assert.strictEqual(pos.character, 3);
        });

        it('should handle newlines correctly', () => {
            const source = 'line1\nline2\nline3';

            // Start of line 2
            const pos1 = offsetToPosition(source, 6);
            assert.strictEqual(pos1.line, 1);
            assert.strictEqual(pos1.character, 0);

            // Middle of line 2
            const pos2 = offsetToPosition(source, 8);
            assert.strictEqual(pos2.line, 1);
            assert.strictEqual(pos2.character, 2);

            // Start of line 3
            const pos3 = offsetToPosition(source, 12);
            assert.strictEqual(pos3.line, 2);
            assert.strictEqual(pos3.character, 0);
        });

        it('should handle empty source', () => {
            const pos = offsetToPosition('', 0);
            assert.strictEqual(pos.line, 0);
            assert.strictEqual(pos.character, 0);
        });

        it('should handle offset beyond source length', () => {
            const pos = offsetToPosition('hi', 10);
            // Should not throw, and return valid position
            assert.ok(pos.line >= 0);
            assert.ok(pos.character >= 0);
        });

        it('should handle source with only newlines', () => {
            const source = '\n\n\n';
            const pos = offsetToPosition(source, 2);
            assert.strictEqual(pos.line, 2);
            assert.strictEqual(pos.character, 0);
        });
    });

    describe('positionToOffset', () => {
        it('should convert line 0, character 0 to offset 0', () => {
            const offset = positionToOffset('hello', { line: 0, character: 0 });
            assert.strictEqual(offset, 0);
        });

        it('should convert mid-line position correctly', () => {
            const offset = positionToOffset('hello', { line: 0, character: 3 });
            assert.strictEqual(offset, 3);
        });

        it('should handle newlines correctly', () => {
            const source = 'line1\nline2\nline3';

            // Start of line 2
            const offset1 = positionToOffset(source, { line: 1, character: 0 });
            assert.strictEqual(offset1, 6);

            // Middle of line 2
            const offset2 = positionToOffset(source, { line: 1, character: 2 });
            assert.strictEqual(offset2, 8);
        });

        it('should round-trip with offsetToPosition', () => {
            const source = 'def main():\n    x = 1\n    y = 2';
            const testOffsets = [0, 5, 12, 18, 25];

            for (const offset of testOffsets) {
                const pos = offsetToPosition(source, offset);
                const backToOffset = positionToOffset(source, pos);
                assert.strictEqual(backToOffset, offset, `Round-trip failed for offset ${offset}`);
            }
        });
    });

    describe('spanToRange', () => {
        it('should convert single-line span', () => {
            const source = 'hello world';
            const range = spanToRange(source, { start: 0, end: 5 });

            assert.strictEqual(range.start.line, 0);
            assert.strictEqual(range.start.character, 0);
            assert.strictEqual(range.end.line, 0);
            assert.strictEqual(range.end.character, 5);
        });

        it('should convert multi-line span', () => {
            const source = 'line1\nline2\nline3';
            const range = spanToRange(source, { start: 0, end: 12 });

            assert.strictEqual(range.start.line, 0);
            assert.strictEqual(range.start.character, 0);
            assert.strictEqual(range.end.line, 2);
            assert.strictEqual(range.end.character, 0);
        });

        it('should handle empty span', () => {
            const source = 'hello';
            const range = spanToRange(source, { start: 2, end: 2 });

            assert.strictEqual(range.start.line, range.end.line);
            assert.strictEqual(range.start.character, range.end.character);
        });
    });
});
