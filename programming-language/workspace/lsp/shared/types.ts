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
 * Built-in constant definition.
 */
export interface BuiltinConstant {
    name: string;
    type: string;
    value: string;
    description: string;
    examples: string[];
}

/**
 * All built-in constants in Cobra64.
 */
export const COBRA64_CONSTANTS: BuiltinConstant[] = [
    // C64 color constants (values 0-15 for VIC-II chip)
    {
        name: 'COLOR_BLACK', type: 'byte', value: '0', description: 'Black color (VIC-II palette 0)', examples: [
            '# set sprite to black (invisible on black bg)\nsprite_color(0, COLOR_BLACK)',
        ]
    },
    {
        name: 'COLOR_WHITE', type: 'byte', value: '1', description: 'White color (VIC-II palette 1)', examples: [
            '# $D020 = border color register\npoke($D020, COLOR_WHITE)',
            'sprite_color(0, COLOR_WHITE)  # bright sprite',
        ]
    },
    {
        name: 'COLOR_RED', type: 'byte', value: '2', description: 'Red color (VIC-II palette 2)', examples: [
            '# good for enemy sprites or warnings\nsprite_color(0, COLOR_RED)',
        ]
    },
    {
        name: 'COLOR_CYAN', type: 'byte', value: '3', description: 'Cyan color (VIC-II palette 3)', examples: [
            '# $D021 = background color register\npoke($D021, COLOR_CYAN)',
        ]
    },
    {
        name: 'COLOR_PURPLE', type: 'byte', value: '4', description: 'Purple color (VIC-II palette 4)', examples: [
            'sprite_color(1, COLOR_PURPLE)',
        ]
    },
    {
        name: 'COLOR_GREEN', type: 'byte', value: '5', description: 'Green color (VIC-II palette 5)', examples: [
            '# good for player or nature elements\nsprite_color(0, COLOR_GREEN)',
        ]
    },
    {
        name: 'COLOR_BLUE', type: 'byte', value: '6', description: 'Blue color (VIC-II palette 6)', examples: [
            '# C64 default border color\npoke($D020, COLOR_BLUE)',
        ]
    },
    {
        name: 'COLOR_YELLOW', type: 'byte', value: '7', description: 'Yellow color (VIC-II palette 7)', examples: [
            '# good for coins, stars, highlights\nsprite_color(0, COLOR_YELLOW)',
        ]
    },
    {
        name: 'COLOR_ORANGE', type: 'byte', value: '8', description: 'Orange color (VIC-II palette 8)', examples: [
            '# shared multicolor for all multicolor sprites\nsprite_multicolor1(COLOR_ORANGE)',
        ]
    },
    {
        name: 'COLOR_BROWN', type: 'byte', value: '9', description: 'Brown color (VIC-II palette 9)', examples: [
            '# good for ground, wood, earth tones\nsprite_color(2, COLOR_BROWN)',
        ]
    },
    {
        name: 'COLOR_LIGHT_RED', type: 'byte', value: '10', description: 'Light red color (VIC-II palette 10)', examples: [
            '# second shared multicolor\nsprite_multicolor2(COLOR_LIGHT_RED)',
        ]
    },
    {
        name: 'COLOR_DARK_GRAY', type: 'byte', value: '11', description: 'Dark gray color (VIC-II palette 11)', examples: [
            '# subtle dark background\npoke($D021, COLOR_DARK_GRAY)',
        ]
    },
    {
        name: 'COLOR_GRAY', type: 'byte', value: '12', description: 'Gray color (VIC-II palette 12)', examples: [
            'poke($D020, COLOR_GRAY)',
        ]
    },
    {
        name: 'COLOR_LIGHT_GREEN', type: 'byte', value: '13', description: 'Light green color (VIC-II palette 13)', examples: [
            'sprite_color(0, COLOR_LIGHT_GREEN)',
        ]
    },
    {
        name: 'COLOR_LIGHT_BLUE', type: 'byte', value: '14', description: 'Light blue color (VIC-II palette 14)', examples: [
            '# C64 default background color\npoke($D021, COLOR_LIGHT_BLUE)',
        ]
    },
    {
        name: 'COLOR_LIGHT_GRAY', type: 'byte', value: '15', description: 'Light gray color (VIC-II palette 15)', examples: [
            'sprite_color(7, COLOR_LIGHT_GRAY)',
        ]
    },
    // VIC-II sprite registers (directly control sprite hardware)
    {
        name: 'VIC_SPRITE_ENABLE', type: 'word', value: '$D015', description: 'Sprite enable register (53269)', examples: [
            '# each bit enables one sprite (bit 0 = sprite 0)\npoke(VIC_SPRITE_ENABLE, %00000001)  # only sprite 0',
            '# enable all 8 sprites at once\npoke(VIC_SPRITE_ENABLE, %11111111)',
        ]
    },
    {
        name: 'VIC_SPRITE_X_MSB', type: 'word', value: '$D010', description: 'Sprite X position MSB (53264)', examples: [
            '# X positions > 255 need this MSB register\n# set bit 0 when sprite 0 X > 255\npoke(VIC_SPRITE_X_MSB, %00000001)',
        ]
    },
    {
        name: 'VIC_SPRITE_EXPAND_Y', type: 'word', value: '$D017', description: 'Sprite Y expansion register (53271)', examples: [
            '# double the height of sprites 0 and 1\npoke(VIC_SPRITE_EXPAND_Y, %00000011)',
        ]
    },
    {
        name: 'VIC_SPRITE_PRIORITY', type: 'word', value: '$D01B', description: 'Sprite priority register (53275)', examples: [
            '# 1 = behind background, 0 = in front\n# sprite 0 appears behind background graphics\npoke(VIC_SPRITE_PRIORITY, %00000001)',
        ]
    },
    {
        name: 'VIC_SPRITE_MULTICOLOR', type: 'word', value: '$D01C', description: 'Sprite multicolor enable (53276)', examples: [
            '# multicolor = 4 colors but half X resolution\n# enable multicolor for sprite 0\npoke(VIC_SPRITE_MULTICOLOR, %00000001)',
        ]
    },
    {
        name: 'VIC_SPRITE_EXPAND_X', type: 'word', value: '$D01D', description: 'Sprite X expansion register (53277)', examples: [
            '# double the width of sprites 0 and 1\npoke(VIC_SPRITE_EXPAND_X, %00000011)',
        ]
    },
    {
        name: 'VIC_SPRITE_COLLISION_SPRITE', type: 'word', value: '$D01E', description: 'Sprite-sprite collision (53278)', examples: [
            '# read clears the register! save the value\n# each bit = that sprite collided\ncollisions: byte = peek(VIC_SPRITE_COLLISION_SPRITE)',
        ]
    },
    {
        name: 'VIC_SPRITE_COLLISION_BG', type: 'word', value: '$D01F', description: 'Sprite-background collision (53279)', examples: [
            '# detects sprite touching non-bg color pixels\n# useful for platform collision\nbg_hit: byte = peek(VIC_SPRITE_COLLISION_BG)',
        ]
    },
    {
        name: 'VIC_SPRITE_MULTICOLOR1', type: 'word', value: '$D025', description: 'Shared multicolor 1 (53285)', examples: [
            '# all multicolor sprites share these 2 colors\n# plus their individual color = 4 total\npoke(VIC_SPRITE_MULTICOLOR1, COLOR_RED)',
        ]
    },
    {
        name: 'VIC_SPRITE_MULTICOLOR2', type: 'word', value: '$D026', description: 'Shared multicolor 2 (53286)', examples: [
            '# second shared color for multicolor sprites\npoke(VIC_SPRITE_MULTICOLOR2, COLOR_WHITE)',
        ]
    },
    {
        name: 'VIC_SPRITE_POINTER_BASE', type: 'word', value: '$07F8', description: 'Sprite pointer base (2040)', examples: [
            '# pointer value * 64 = sprite data address\n# pointer 13 = address 832 ($0340)\npoke(VIC_SPRITE_POINTER_BASE, 13)',
            '# add sprite number for other sprites\n# sprite 1 pointer at $07F9, etc.\npoke(VIC_SPRITE_POINTER_BASE + 1, 14)',
        ]
    },
    // Individual sprite position registers (low byte only, see MSB for X > 255)
    {
        name: 'VIC_SPRITE0_X', type: 'word', value: '$D000', description: 'Sprite 0 X position (53248)', examples: [
            '# low 8 bits of X position (0-255)\n# for X > 255, also set MSB register\npoke(VIC_SPRITE0_X, 100)',
        ]
    },
    {
        name: 'VIC_SPRITE0_Y', type: 'word', value: '$D001', description: 'Sprite 0 Y position (53249)', examples: [
            '# visible range roughly 50-250\npoke(VIC_SPRITE0_Y, 150)',
        ]
    },
    { name: 'VIC_SPRITE1_X', type: 'word', value: '$D002', description: 'Sprite 1 X position (53250)', examples: ['poke(VIC_SPRITE1_X, 120)'] },
    { name: 'VIC_SPRITE1_Y', type: 'word', value: '$D003', description: 'Sprite 1 Y position (53251)', examples: ['poke(VIC_SPRITE1_Y, 100)'] },
    { name: 'VIC_SPRITE2_X', type: 'word', value: '$D004', description: 'Sprite 2 X position (53252)', examples: ['x: byte = peek(VIC_SPRITE2_X)'] },
    { name: 'VIC_SPRITE2_Y', type: 'word', value: '$D005', description: 'Sprite 2 Y position (53253)', examples: ['y: byte = peek(VIC_SPRITE2_Y)'] },
    { name: 'VIC_SPRITE3_X', type: 'word', value: '$D006', description: 'Sprite 3 X position (53254)', examples: ['poke(VIC_SPRITE3_X, 200)'] },
    { name: 'VIC_SPRITE3_Y', type: 'word', value: '$D007', description: 'Sprite 3 Y position (53255)', examples: ['poke(VIC_SPRITE3_Y, 100)'] },
    { name: 'VIC_SPRITE4_X', type: 'word', value: '$D008', description: 'Sprite 4 X position (53256)', examples: ['poke(VIC_SPRITE4_X, 50)'] },
    { name: 'VIC_SPRITE4_Y', type: 'word', value: '$D009', description: 'Sprite 4 Y position (53257)', examples: ['poke(VIC_SPRITE4_Y, 50)'] },
    { name: 'VIC_SPRITE5_X', type: 'word', value: '$D00A', description: 'Sprite 5 X position (53258)', examples: ['poke(VIC_SPRITE5_X, 180)'] },
    { name: 'VIC_SPRITE5_Y', type: 'word', value: '$D00B', description: 'Sprite 5 Y position (53259)', examples: ['poke(VIC_SPRITE5_Y, 200)'] },
    { name: 'VIC_SPRITE6_X', type: 'word', value: '$D00C', description: 'Sprite 6 X position (53260)', examples: ['poke(VIC_SPRITE6_X, 80)'] },
    { name: 'VIC_SPRITE6_Y', type: 'word', value: '$D00D', description: 'Sprite 6 Y position (53261)', examples: ['poke(VIC_SPRITE6_Y, 120)'] },
    { name: 'VIC_SPRITE7_X', type: 'word', value: '$D00E', description: 'Sprite 7 X position (53262)', examples: ['poke(VIC_SPRITE7_X, 160)'] },
    { name: 'VIC_SPRITE7_Y', type: 'word', value: '$D00F', description: 'Sprite 7 Y position (53263)', examples: ['poke(VIC_SPRITE7_Y, 180)'] },
    // Individual sprite color registers
    {
        name: 'VIC_SPRITE0_COLOR', type: 'word', value: '$D027', description: 'Sprite 0 color (53287)', examples: [
            '# individual sprite color (0-15)\n# in multicolor mode, this is the 3rd color\npoke(VIC_SPRITE0_COLOR, COLOR_WHITE)',
        ]
    },
    { name: 'VIC_SPRITE1_COLOR', type: 'word', value: '$D028', description: 'Sprite 1 color (53288)', examples: ['poke(VIC_SPRITE1_COLOR, COLOR_RED)'] },
    { name: 'VIC_SPRITE2_COLOR', type: 'word', value: '$D029', description: 'Sprite 2 color (53289)', examples: ['poke(VIC_SPRITE2_COLOR, COLOR_GREEN)'] },
    { name: 'VIC_SPRITE3_COLOR', type: 'word', value: '$D02A', description: 'Sprite 3 color (53290)', examples: ['poke(VIC_SPRITE3_COLOR, COLOR_BLUE)'] },
    { name: 'VIC_SPRITE4_COLOR', type: 'word', value: '$D02B', description: 'Sprite 4 color (53291)', examples: ['poke(VIC_SPRITE4_COLOR, COLOR_YELLOW)'] },
    { name: 'VIC_SPRITE5_COLOR', type: 'word', value: '$D02C', description: 'Sprite 5 color (53292)', examples: ['poke(VIC_SPRITE5_COLOR, COLOR_CYAN)'] },
    { name: 'VIC_SPRITE6_COLOR', type: 'word', value: '$D02D', description: 'Sprite 6 color (53293)', examples: ['poke(VIC_SPRITE6_COLOR, COLOR_PURPLE)'] },
    { name: 'VIC_SPRITE7_COLOR', type: 'word', value: '$D02E', description: 'Sprite 7 color (53294)', examples: ['poke(VIC_SPRITE7_COLOR, COLOR_ORANGE)'] },
];

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
    examples: string[];
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
        examples: [
            '# clear screen and move cursor to top-left\ncls()',
            '# typical program start\ndef main():\n    cls()  # start with clean screen\n    println("Hello!")',
        ],
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
        examples: [
            '# C64 screen is 40x25 characters\ncursor(0, 0)   # top-left corner\ncursor(39, 0)  # top-right corner',
            '# center text on screen\ncursor(15, 12)  # roughly centered\nprint("GAME OVER")',
        ],
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
        examples: [
            '# use print for same-line output\nprint("Score: ")  # no line break\nprint(score)       # continues on same line',
            '# works with different types\nprint(42)       # number\nprint(true)     # boolean\nprint("text")   # string',
        ],
    },
    {
        name: 'println',
        signature: 'println(value)',
        description: 'Prints a value followed by a newline.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to print (string, number, bool)' },
        ],
        returnType: null,
        examples: [
            '# each println starts a new line\nprintln("Line 1")\nprintln("Line 2")',
            '# print numbers in a loop\nfor i in 1 to 5:\n    println(i)  # prints 1, 2, 3, 4, 5',
        ],
    },
    // Input functions
    {
        name: 'get_key',
        signature: 'get_key() -> byte',
        description: 'Returns the current key being pressed, or 0 if no key.',
        parameters: [],
        returnType: 'byte',
        examples: [
            '# non-blocking: returns 0 if no key pressed\nkey: byte = get_key()\nif key != 0:\n    println("You pressed a key!")',
            '# game loop style - check without waiting\nwhile true:\n    key: byte = get_key()\n    if key == 81:  # Q key\n        break  # exit loop',
        ],
    },
    {
        name: 'read',
        signature: 'read() -> byte',
        description: 'Waits for a key press and returns it.',
        parameters: [],
        returnType: 'byte',
        examples: [
            '# blocking: waits until key is pressed\nprintln("Press any key to continue...")\nkey: byte = read()  # program pauses here',
            '# menu selection example\nprintln("Press 1, 2, or 3:")\nchoice: byte = read()\nif choice == 49:     # ASCII code for "1"\n    println("You chose option 1")',
        ],
    },
    {
        name: 'readln',
        signature: 'readln() -> string',
        description: 'Reads a line of text input from the user.',
        parameters: [],
        returnType: 'string',
        examples: [
            '# read text until RETURN is pressed\nprint("Enter your name: ")\nname: string = readln()\nprintln("Hello, " + name + "!")',
        ],
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
        examples: [
            '# common C64 memory locations:\n# $D020 (53280) = border color\n# $D021 (53281) = background color\npoke($D020, 0)  # black border\npoke($D021, 6)  # blue background',
            '# hex ($) and decimal both work\npoke(53280, COLOR_BLACK)  # same as $D020\npoke($D021, COLOR_BLUE)',
        ],
    },
    {
        name: 'peek',
        signature: 'peek(address: word) -> byte',
        description: 'Reads a byte from a memory address.',
        parameters: [
            { name: 'address', type: 'word', description: 'Memory address (0-65535)' },
        ],
        returnType: 'byte',
        examples: [
            '# read current border color\nborder: byte = peek($D020)',
            '# $DC01 = keyboard matrix register\n# value is 255 when no key pressed\nif peek($DC01) != 255:\n    println("A key is being pressed!")',
        ],
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
        examples: [
            '# string length (max 255 chars)\ntext: string = "Hello"\nsize: byte = len(text)  # returns 5',
            '# array length (can be larger)\ndata: byte[] = [1, 2, 3, 4, 5]\ncount: word = len(data)  # returns 5',
        ],
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
        examples: [
            '# returns PETSCII/ASCII code of character\ntext: string = "ABC"\nfirst: byte = str_at(text, 0)  # 65 = "A"\nsecond: byte = str_at(text, 1) # 66 = "B"',
            '# iterate through string characters\nfor i in 0 to len(text) - 1:\n    ch: byte = str_at(text, i)\n    println(ch)  # print ASCII code',
        ],
    },
    // Random functions
    {
        name: 'rand',
        signature: 'rand() -> fixed',
        description: 'Returns a random fixed-point number between 0.0 and 0.9375.',
        parameters: [],
        returnType: 'fixed',
        examples: [
            '# useful for probability checks\nif rand() > 0.5:\n    println("Heads!")  # 50% chance\nelse:\n    println("Tails!")',
            '# combine with math for ranges\n# e.g., random value 0-100:\nvalue: fixed = rand() * fixed(100)',
        ],
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
        examples: [
            '# dice roll (1-6)\ndice: byte = rand_byte(1, 6)',
            '# random C64 color (0-15)\ncolor: byte = rand_byte(0, 15)\nsprite_color(0, color)',
            '# random visible X position\n# 24-343 is visible screen area\nx: byte = rand_byte(24, 255)',
        ],
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
        examples: [
            '# random direction: -1, 0, or +1\ndir: sbyte = rand_sbyte(-1, 1)',
            '# random velocity with negative values\nspeed: sbyte = rand_sbyte(-5, 5)',
        ],
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
        examples: [
            '# random memory address\naddr: word = rand_word(1024, 2047)',
            '# random sprite X (full range 0-511)\nx: word = rand_word(24, 320)\nsprite_x(0, x)',
        ],
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
        examples: [
            '# random velocity with large range\nvelocity: sword = rand_sword(-100, 100)',
        ],
    },
    {
        name: 'seed',
        signature: 'seed()',
        description: 'Reseeds the random number generator from hardware entropy sources.',
        parameters: [],
        returnType: null,
        examples: [
            '# call once at program start for varied random numbers\n# uses C64 SID chip noise + timers for entropy\ndef main():\n    seed()  # initialize random generator\n    dice: byte = rand_byte(1, 6)\n    println(dice)',
        ],
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
        examples: [
            '# combine numbers with text using +\nscore: word = 1000\nprintln("Score: " + str(score))',
            '# works with any numeric type\nlives: byte = 3\ntext: string = "Lives: " + str(lives)\nprintln(text)',
        ],
    },
    {
        name: 'bool',
        signature: 'bool(value) -> bool',
        description: 'Converts a value to boolean. Returns true if value is non-zero.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert to boolean' },
        ],
        returnType: 'bool',
        examples: [
            '# 0 = false, non-zero = true\nbool(0)   # false\nbool(1)   # true\nbool(42)  # true',
            '# useful for checking if value exists\ncount: byte = 5\nif bool(count):\n    println("Has items")',
        ],
    },
    {
        name: 'byte',
        signature: 'byte(value) -> byte',
        description: 'Converts a value to byte. Truncates larger values to 8 bits. Parses strings as numbers.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to convert (numeric or string)' },
        ],
        returnType: 'byte',
        examples: [
            '# keeps only lower 8 bits\nbyte(256)  # 0 (256 = 0x100, lower 8 bits = 0)\nbyte(257)  # 1',
            '# parse string to number\nb: byte = byte("42")  # 42',
            '# extract low byte from word\naddr: word = $1234\nlow: byte = byte(addr)  # $34',
        ],
    },
    {
        name: 'word',
        signature: 'word(value) -> word',
        description: 'Converts a value to word. Truncates larger values to 16 bits. Parses strings as numbers.',
        parameters: [
            { name: 'value', type: 'any', description: 'Value to convert (numeric or string)' },
        ],
        returnType: 'word',
        examples: [
            '# expand byte to word for math\nsmall: byte = 100\nbig: word = word(small) * 256',
            '# combine high and low bytes into address\nhigh: byte = $12\nlow: byte = $34\naddr: word = word(high) * 256 + word(low)  # $1234',
        ],
    },
    {
        name: 'sbyte',
        signature: 'sbyte(value) -> sbyte',
        description: 'Converts a value to signed byte.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'sbyte',
        examples: [
            '# values > 127 become negative\nsbyte(200)  # -56 (wraps around)\nsbyte(128)  # -128',
            '# useful for signed arithmetic\ndelta: sbyte = sbyte(-5)  # move left\nsprite_x(0, sprite_get_x(0) + word(delta))',
        ],
    },
    {
        name: 'sword',
        signature: 'sword(value) -> sword',
        description: 'Converts a value to signed word.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'sword',
        examples: [
            '# for signed 16-bit math\npos: sword = sword(-1000)',
            '# values > 32767 become negative\nsword(40000)  # becomes negative',
        ],
    },
    {
        name: 'fixed',
        signature: 'fixed(value) -> fixed',
        description: 'Converts a value to fixed-point (12.4 format). Range: -2048.0 to 2047.9375.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'fixed',
        examples: [
            '# fixed-point allows fractional values\n# faster than float on 6502 CPU\nspeed: fixed = fixed(1) / fixed(4)  # 0.25',
            '# useful for smooth movement\npos_x: fixed = fixed(100)\npos_x = pos_x + 0.5  # move half-pixel',
        ],
    },
    {
        name: 'float',
        signature: 'float(value) -> float',
        description: 'Converts a value to IEEE-754 binary16 float.',
        parameters: [
            { name: 'value', type: 'numeric', description: 'Value to convert' },
        ],
        returnType: 'float',
        examples: [
            '# floating point - more precision but slower\nf: float = float(3.14159)',
            '# use for complex math where precision matters\nresult: float = float(a) / float(b)',
        ],
    },
    // Sprite Control functions
    {
        name: 'sprite_enable',
        signature: 'sprite_enable(num: byte, enable: bool)',
        description: 'Enables or disables a sprite. Sets or clears bit in VIC-II $D015.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'enable', type: 'bool', description: 'true to enable, false to disable' },
        ],
        returnType: null,
        examples: [
            '# C64 has 8 hardware sprites (0-7)\n# must enable before sprite is visible\nsprite_enable(0, true)   # show sprite 0\nsprite_enable(0, false)  # hide sprite 0',
            '# enable multiple sprites\nfor i in 0 to 3:\n    sprite_enable(i, true)  # enable sprites 0-3',
        ],
    },
    {
        name: 'sprites_enable',
        signature: 'sprites_enable(mask: byte)',
        description: 'Enables sprites by bitmask. Writes directly to VIC-II $D015.',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask (bit 0 = sprite 0, etc.)' },
        ],
        returnType: null,
        examples: [
            '# each bit = one sprite (faster than loop)\n# bit 0 = sprite 0, bit 1 = sprite 1, etc.\nsprites_enable(%00000001)  # only sprite 0\nsprites_enable(%00000011)  # sprites 0 and 1\nsprites_enable(%11111111)  # all 8 sprites\nsprites_enable(0)          # disable all',
        ],
    },
    // Sprite Positioning functions
    {
        name: 'sprite_x',
        signature: 'sprite_x(num: byte, x: word)',
        description: 'Sets the X position of a sprite (0-511). Handles 9-bit coordinate with MSB register.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'x', type: 'word', description: 'X position (0-511)' },
        ],
        returnType: null,
        examples: [
            '# X range is 0-511 (9 bits) for full screen\n# visible area is roughly 24-343\nsprite_x(0, 24)   # left edge of screen\nsprite_x(0, 160)  # center horizontally\nsprite_x(0, 300)  # works beyond 255!',
            '# move sprite (simple animation)\nsprite_x(0, sprite_get_x(0) + 1)  # move right',
        ],
    },
    {
        name: 'sprite_y',
        signature: 'sprite_y(num: byte, y: byte)',
        description: 'Sets the Y position of a sprite (0-255).',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'y', type: 'byte', description: 'Y position (0-255)' },
        ],
        returnType: null,
        examples: [
            '# Y range is 0-255 (8 bits)\n# visible area is roughly 50-229\nsprite_y(0, 50)   # near top of screen\nsprite_y(0, 140)  # center vertically\nsprite_y(0, 229)  # near bottom',
            '# move sprite down\nsprite_y(0, sprite_get_y(0) + 1)',
        ],
    },
    {
        name: 'sprite_pos',
        signature: 'sprite_pos(num: byte, x: word, y: byte)',
        description: 'Sets both X and Y position of a sprite.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'x', type: 'word', description: 'X position (0-511)' },
            { name: 'y', type: 'byte', description: 'Y position (0-255)' },
        ],
        returnType: null,
        examples: [
            '# set both coordinates at once (more efficient)\nsprite_pos(0, 160, 140)  # center of screen',
            '# position player sprite at start\nsprite_pos(0, 24, 200)   # bottom-left',
        ],
    },
    {
        name: 'sprite_get_x',
        signature: 'sprite_get_x(num: byte) -> word',
        description: 'Returns the X position of a sprite (0-511).',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'word',
        examples: [
            '# returns word because X can be > 255\nx: word = sprite_get_x(0)',
            '# screen wrapping example\nif sprite_get_x(0) > 343:\n    sprite_x(0, 24)  # wrap to left edge',
        ],
    },
    {
        name: 'sprite_get_y',
        signature: 'sprite_get_y(num: byte) -> byte',
        description: 'Returns the Y position of a sprite (0-255).',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'byte',
        examples: [
            'y: byte = sprite_get_y(0)',
            '# keep sprite on screen\nif sprite_get_y(0) > 229:\n    sprite_y(0, 50)  # wrap to top',
        ],
    },
    // Sprite Data & Color functions
    {
        name: 'sprite_data',
        signature: 'sprite_data(num: byte, pointer: byte)',
        description: 'Sets the sprite data pointer. Points to 64-byte block (pointer × 64).',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'pointer', type: 'byte', description: 'Pointer value (address = pointer × 64)' },
        ],
        returnType: null,
        examples: [
            '# sprite graphics are 64 bytes (24x21 pixels)\n# pointer value * 64 = memory address\n# example: pointer 13 = address 832\nsprite_data(0, 13)',
            '# simple sprite animation:\n# switch between different graphics\nframe: byte = 0\nwhile true:\n    sprite_data(0, 13 + frame)\n    frame = (frame + 1) % 4  # cycle 0-3',
        ],
    },
    {
        name: 'sprite_get_data',
        signature: 'sprite_get_data(num: byte) -> byte',
        description: 'Returns the sprite data pointer value.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'byte',
        examples: [
            'ptr: byte = sprite_get_data(0)\n# calculate actual address:\naddr: word = word(ptr) * 64',
        ],
    },
    {
        name: 'sprite_color',
        signature: 'sprite_color(num: byte, color: byte)',
        description: 'Sets the color of a sprite (0-15).',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'color', type: 'byte', description: 'Color (0-15, use COLOR_* constants)' },
        ],
        returnType: null,
        examples: [
            '# use COLOR_* constants for readability\nsprite_color(0, COLOR_WHITE)  # player\nsprite_color(1, COLOR_RED)    # enemy',
            '# flash effect: alternate colors\nsprite_color(0, COLOR_WHITE)\n# ... wait ...\nsprite_color(0, COLOR_YELLOW)',
        ],
    },
    {
        name: 'sprite_get_color',
        signature: 'sprite_get_color(num: byte) -> byte',
        description: 'Returns the color of a sprite.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'byte',
        examples: [
            'c: byte = sprite_get_color(0)',
            '# check sprite color for game logic\nif sprite_get_color(0) == COLOR_RED:\n    println("Damaged!")',
        ],
    },
    {
        name: 'sprite_multicolor1',
        signature: 'sprite_multicolor1(color: byte)',
        description: 'Sets shared multicolor 1 for all sprites (VIC-II $D025).',
        parameters: [
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            '# multicolor sprites have 4 colors:\n# 1. transparent (background shows through)\n# 2. multicolor1 (shared by all sprites)\n# 3. multicolor2 (shared by all sprites)\n# 4. individual sprite color\nsprite_multicolor1(COLOR_WHITE)\nsprite_multicolor2(COLOR_BLACK)',
        ],
    },
    {
        name: 'sprite_multicolor2',
        signature: 'sprite_multicolor2(color: byte)',
        description: 'Sets shared multicolor 2 for all sprites (VIC-II $D026).',
        parameters: [
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            '# second shared color for multicolor sprites\n# often used for outlines or shadows\nsprite_multicolor2(COLOR_BLACK)',
        ],
    },
    {
        name: 'sprite_get_multicolor1',
        signature: 'sprite_get_multicolor1() -> byte',
        description: 'Returns the shared multicolor 1 value.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'mc1: byte = sprite_get_multicolor1()',
        ],
    },
    {
        name: 'sprite_get_multicolor2',
        signature: 'sprite_get_multicolor2() -> byte',
        description: 'Returns the shared multicolor 2 value.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'mc2: byte = sprite_get_multicolor2()',
        ],
    },
    // Sprite Multicolor & Expansion functions
    {
        name: 'sprite_multicolor',
        signature: 'sprite_multicolor(num: byte, enable: bool)',
        description: 'Enables or disables multicolor mode for a sprite.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'enable', type: 'bool', description: 'true to enable multicolor mode' },
        ],
        returnType: null,
        examples: [
            '# multicolor mode: 4 colors, but half horizontal resolution\n# hi-res mode: 1 color, full resolution (24x21 pixels)\nsprite_multicolor(0, true)   # 4-color mode\nsprite_multicolor(0, false)  # hi-res mode',
        ],
    },
    {
        name: 'sprites_multicolor',
        signature: 'sprites_multicolor(mask: byte)',
        description: 'Sets multicolor mode by bitmask (VIC-II $D01C).',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask (bit 0 = sprite 0, etc.)' },
        ],
        returnType: null,
        examples: [
            '# set multiple sprites at once\nsprites_multicolor(%00001111)  # sprites 0-3 multicolor\nsprites_multicolor(0)          # all hi-res',
        ],
    },
    {
        name: 'sprite_is_multicolor',
        signature: 'sprite_is_multicolor(num: byte) -> bool',
        description: 'Returns true if sprite is in multicolor mode.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'bool',
        examples: [
            'if sprite_is_multicolor(0):\n    println("4-color mode")\nelse:\n    println("hi-res mode")',
        ],
    },
    {
        name: 'sprite_expand_x',
        signature: 'sprite_expand_x(num: byte, expand: bool)',
        description: 'Enables or disables 2x width expansion for a sprite.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'expand', type: 'bool', description: 'true to expand to double width' },
        ],
        returnType: null,
        examples: [
            '# expansion doubles sprite size (same data, bigger pixels)\n# normal sprite: 24x21 pixels\n# expanded: 48x21 (X) or 24x42 (Y) or 48x42 (both)\nsprite_expand_x(0, true)   # 48 pixels wide\nsprite_expand_x(0, false)  # normal 24 pixels',
        ],
    },
    {
        name: 'sprite_expand_y',
        signature: 'sprite_expand_y(num: byte, expand: bool)',
        description: 'Enables or disables 2x height expansion for a sprite.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'expand', type: 'bool', description: 'true to expand to double height' },
        ],
        returnType: null,
        examples: [
            'sprite_expand_y(0, true)  # 42 pixels tall',
            '# make sprite 4x larger (both directions)\nsprite_expand_x(0, true)\nsprite_expand_y(0, true)  # now 48x42 pixels',
        ],
    },
    {
        name: 'sprites_expand_x',
        signature: 'sprites_expand_x(mask: byte)',
        description: 'Sets X expansion by bitmask (VIC-II $D01D).',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask (bit 0 = sprite 0, etc.)' },
        ],
        returnType: null,
        examples: [
            '# expand multiple sprites at once\nsprites_expand_x(%00000011)  # sprites 0,1 double width\nsprites_expand_x(0)          # all normal width',
        ],
    },
    {
        name: 'sprites_expand_y',
        signature: 'sprites_expand_y(mask: byte)',
        description: 'Sets Y expansion by bitmask (VIC-II $D017).',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask (bit 0 = sprite 0, etc.)' },
        ],
        returnType: null,
        examples: [
            'sprites_expand_y(%00000011)  # sprites 0,1 double height',
        ],
    },
    {
        name: 'sprite_is_expanded_x',
        signature: 'sprite_is_expanded_x(num: byte) -> bool',
        description: 'Returns true if sprite has 2x width expansion enabled.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'bool',
        examples: [
            'if sprite_is_expanded_x(0):\n    println("48 pixels wide")',
        ],
    },
    {
        name: 'sprite_is_expanded_y',
        signature: 'sprite_is_expanded_y(num: byte) -> bool',
        description: 'Returns true if sprite has 2x height expansion enabled.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'bool',
        examples: [
            'if sprite_is_expanded_y(0):\n    println("42 pixels tall")',
        ],
    },
    // Sprite Priority & Collision functions
    {
        name: 'sprite_priority',
        signature: 'sprite_priority(num: byte, behind_bg: bool)',
        description: 'Sets sprite priority. If true, sprite appears behind background graphics.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
            { name: 'behind_bg', type: 'bool', description: 'true = behind background, false = in front' },
        ],
        returnType: null,
        examples: [
            '# useful for hide-behind-scenery effects\n# sprite goes behind non-background-color pixels\nsprite_priority(0, true)   # sprite behind scenery\nsprite_priority(0, false)  # sprite in front (default)',
        ],
    },
    {
        name: 'sprites_priority',
        signature: 'sprites_priority(mask: byte)',
        description: 'Sets priority by bitmask (VIC-II $D01B). Set bits = behind background.',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask (bit 0 = sprite 0, etc.)' },
        ],
        returnType: null,
        examples: [
            '# set multiple sprite priorities at once\nsprites_priority(%00000011)  # 0,1 behind background\nsprites_priority(0)          # all in front',
        ],
    },
    {
        name: 'sprite_get_priority',
        signature: 'sprite_get_priority(num: byte) -> bool',
        description: 'Returns true if sprite is set to appear behind background.',
        parameters: [
            { name: 'num', type: 'byte', description: 'Sprite number (0-7)' },
        ],
        returnType: 'bool',
        examples: [
            'behind: bool = sprite_get_priority(0)',
        ],
    },
    {
        name: 'sprite_collision_sprite',
        signature: 'sprite_collision_sprite() -> byte',
        description: 'Reads sprite-sprite collision register (VIC-II $D01E). Clears on read.',
        parameters: [],
        returnType: 'byte',
        examples: [
            '# IMPORTANT: register clears after read!\n# save value if checking multiple sprites\ncoll: byte = sprite_collision_sprite()\n\n# each bit = that sprite is colliding\nif coll and %00000001:\n    println("Sprite 0 hit something!")\nif coll and %00000010:\n    println("Sprite 1 hit something!")',
            '# check player (sprite 0) vs enemies (sprites 1-4)\ncoll: byte = sprite_collision_sprite()\nif coll and %00011110:  # bits 1-4\n    player_hit()',
        ],
    },
    {
        name: 'sprite_collision_bg',
        signature: 'sprite_collision_bg() -> byte',
        description: 'Reads sprite-background collision register (VIC-II $D01F). Clears on read.',
        parameters: [],
        returnType: 'byte',
        examples: [
            '# detects when sprite overlaps non-bg-color pixels\n# useful for platform games, walls, etc.\nbg: byte = sprite_collision_bg()\nif bg and %00000001:  # sprite 0\n    println("Hit wall or platform!")',
        ],
    },
    {
        name: 'sprite_collides',
        signature: 'sprite_collides(mask: byte) -> bool',
        description: 'Returns true if any sprite in mask has a collision.',
        parameters: [
            { name: 'mask', type: 'byte', description: 'Bitmask of sprites to check' },
        ],
        returnType: 'bool',
        examples: [
            '# simplified collision check for common cases\nif sprite_collides(%00000001):\n    # sprite 0 (player) hit something\n    lives = lives - 1',
            '# check if any enemy sprite hit player\nif sprite_collides(%00011110):  # sprites 1-4 (enemies)\n    game_over()',
        ],
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
