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
    DataBlock = 'dataBlock',
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
    'data',
    'def',
    'end',
    'if',
    'elif',
    'else',
    'include',
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
    // SID Sound Waveform Constants
    {
        name: 'WAVE_TRIANGLE', type: 'byte', value: '16', description: 'Triangle waveform for SID voice (soft, mellow tone)', examples: [
            '# smooth, flute-like sound\nsid_waveform(0, WAVE_TRIANGLE)',
        ]
    },
    {
        name: 'WAVE_SAWTOOTH', type: 'byte', value: '32', description: 'Sawtooth waveform for SID voice (bright, harsh tone)', examples: [
            '# bright, buzzy sound - good for bass\nsid_waveform(0, WAVE_SAWTOOTH)',
        ]
    },
    {
        name: 'WAVE_PULSE', type: 'byte', value: '64', description: 'Pulse/square waveform for SID voice (hollow, rich tone)', examples: [
            '# classic 8-bit sound\n# vary pulse width for different tones\nsid_waveform(0, WAVE_PULSE)\nsid_pulse_width(0, 2048)  # 50% = square wave',
        ]
    },
    {
        name: 'WAVE_NOISE', type: 'byte', value: '128', description: 'White noise waveform for SID voice (drums, explosions)', examples: [
            '# use for drums, explosions, wind, laser sounds\nsid_waveform(0, WAVE_NOISE)',
        ]
    },
    // SID Filter Mode Constants
    {
        name: 'FILTER_LOWPASS', type: 'byte', value: '16', description: 'Low-pass filter mode (passes low frequencies)', examples: [
            '# cuts high frequencies - warm, muffled sound\nsid_filter_mode(FILTER_LOWPASS)\nsid_filter_cutoff(512)',
        ]
    },
    {
        name: 'FILTER_BANDPASS', type: 'byte', value: '32', description: 'Band-pass filter mode (passes mid frequencies)', examples: [
            '# passes frequencies around cutoff\n# nasal, telephone-like sound\nsid_filter_mode(FILTER_BANDPASS)',
        ]
    },
    {
        name: 'FILTER_HIGHPASS', type: 'byte', value: '64', description: 'High-pass filter mode (passes high frequencies)', examples: [
            '# cuts low frequencies - thin, bright sound\nsid_filter_mode(FILTER_HIGHPASS)',
        ]
    },
    // SID Note Constants
    {
        name: 'NOTE_C', type: 'byte', value: '0', description: 'Musical note C (for play_note function)', examples: [
            '# play middle C (C4)\nplay_note(0, NOTE_C, 4)',
        ]
    },
    {
        name: 'NOTE_CS', type: 'byte', value: '1', description: 'Musical note C#/Db (for play_note function)', examples: [
            'play_note(0, NOTE_CS, 4)',
        ]
    },
    {
        name: 'NOTE_D', type: 'byte', value: '2', description: 'Musical note D (for play_note function)', examples: [
            'play_note(0, NOTE_D, 4)',
        ]
    },
    {
        name: 'NOTE_DS', type: 'byte', value: '3', description: 'Musical note D#/Eb (for play_note function)', examples: [
            'play_note(0, NOTE_DS, 4)',
        ]
    },
    {
        name: 'NOTE_E', type: 'byte', value: '4', description: 'Musical note E (for play_note function)', examples: [
            'play_note(0, NOTE_E, 4)',
        ]
    },
    {
        name: 'NOTE_F', type: 'byte', value: '5', description: 'Musical note F (for play_note function)', examples: [
            'play_note(0, NOTE_F, 4)',
        ]
    },
    {
        name: 'NOTE_FS', type: 'byte', value: '6', description: 'Musical note F#/Gb (for play_note function)', examples: [
            'play_note(0, NOTE_FS, 4)',
        ]
    },
    {
        name: 'NOTE_G', type: 'byte', value: '7', description: 'Musical note G (for play_note function)', examples: [
            'play_note(0, NOTE_G, 4)',
        ]
    },
    {
        name: 'NOTE_GS', type: 'byte', value: '8', description: 'Musical note G#/Ab (for play_note function)', examples: [
            'play_note(0, NOTE_GS, 4)',
        ]
    },
    {
        name: 'NOTE_A', type: 'byte', value: '9', description: 'Musical note A (for play_note function)', examples: [
            '# A4 = 440 Hz (concert pitch)\nplay_note(0, NOTE_A, 4)',
        ]
    },
    {
        name: 'NOTE_AS', type: 'byte', value: '10', description: 'Musical note A#/Bb (for play_note function)', examples: [
            'play_note(0, NOTE_AS, 4)',
        ]
    },
    {
        name: 'NOTE_B', type: 'byte', value: '11', description: 'Musical note B (for play_note function)', examples: [
            'play_note(0, NOTE_B, 4)',
        ]
    },
    // SID Register Address Constants
    {
        name: 'SID_BASE', type: 'word', value: '$D400', description: 'SID chip base address (54272)', examples: [
            '# all SID registers start at $D400\n# voices are at 7-byte offsets',
        ]
    },
    { name: 'SID_VOICE1_FREQ_LO', type: 'word', value: '$D400', description: 'Voice 1 frequency low byte (54272)', examples: ['poke(SID_VOICE1_FREQ_LO, $22)'] },
    { name: 'SID_VOICE1_FREQ_HI', type: 'word', value: '$D401', description: 'Voice 1 frequency high byte (54273)', examples: ['poke(SID_VOICE1_FREQ_HI, $1C)'] },
    { name: 'SID_VOICE1_PW_LO', type: 'word', value: '$D402', description: 'Voice 1 pulse width low byte (54274)', examples: ['# 12-bit pulse width (0-4095)'] },
    { name: 'SID_VOICE1_PW_HI', type: 'word', value: '$D403', description: 'Voice 1 pulse width high byte (54275)', examples: ['# upper 4 bits of pulse width'] },
    { name: 'SID_VOICE1_CTRL', type: 'word', value: '$D404', description: 'Voice 1 control register (54276)', examples: ['# gate, sync, ring mod, test, waveform'] },
    { name: 'SID_VOICE1_AD', type: 'word', value: '$D405', description: 'Voice 1 attack/decay (54277)', examples: ['# upper 4 bits = attack, lower = decay'] },
    { name: 'SID_VOICE1_SR', type: 'word', value: '$D406', description: 'Voice 1 sustain/release (54278)', examples: ['# upper 4 bits = sustain, lower = release'] },
    { name: 'SID_VOICE2_FREQ_LO', type: 'word', value: '$D407', description: 'Voice 2 frequency low byte (54279)', examples: ['poke(SID_VOICE2_FREQ_LO, freq_lo)'] },
    { name: 'SID_VOICE2_FREQ_HI', type: 'word', value: '$D408', description: 'Voice 2 frequency high byte (54280)', examples: ['poke(SID_VOICE2_FREQ_HI, freq_hi)'] },
    { name: 'SID_VOICE2_PW_LO', type: 'word', value: '$D409', description: 'Voice 2 pulse width low byte (54281)', examples: ['poke(SID_VOICE2_PW_LO, pw_lo)'] },
    { name: 'SID_VOICE2_PW_HI', type: 'word', value: '$D40A', description: 'Voice 2 pulse width high byte (54282)', examples: ['poke(SID_VOICE2_PW_HI, pw_hi)'] },
    { name: 'SID_VOICE2_CTRL', type: 'word', value: '$D40B', description: 'Voice 2 control register (54283)', examples: ['poke(SID_VOICE2_CTRL, $21)'] },
    { name: 'SID_VOICE2_AD', type: 'word', value: '$D40C', description: 'Voice 2 attack/decay (54284)', examples: ['poke(SID_VOICE2_AD, $00)'] },
    { name: 'SID_VOICE2_SR', type: 'word', value: '$D40D', description: 'Voice 2 sustain/release (54285)', examples: ['poke(SID_VOICE2_SR, $F9)'] },
    { name: 'SID_VOICE3_FREQ_LO', type: 'word', value: '$D40E', description: 'Voice 3 frequency low byte (54286)', examples: ['poke(SID_VOICE3_FREQ_LO, freq_lo)'] },
    { name: 'SID_VOICE3_FREQ_HI', type: 'word', value: '$D40F', description: 'Voice 3 frequency high byte (54287)', examples: ['poke(SID_VOICE3_FREQ_HI, freq_hi)'] },
    { name: 'SID_VOICE3_PW_LO', type: 'word', value: '$D410', description: 'Voice 3 pulse width low byte (54288)', examples: ['poke(SID_VOICE3_PW_LO, pw_lo)'] },
    { name: 'SID_VOICE3_PW_HI', type: 'word', value: '$D411', description: 'Voice 3 pulse width high byte (54289)', examples: ['poke(SID_VOICE3_PW_HI, pw_hi)'] },
    { name: 'SID_VOICE3_CTRL', type: 'word', value: '$D412', description: 'Voice 3 control register (54290)', examples: ['poke(SID_VOICE3_CTRL, $21)'] },
    { name: 'SID_VOICE3_AD', type: 'word', value: '$D413', description: 'Voice 3 attack/decay (54291)', examples: ['poke(SID_VOICE3_AD, $00)'] },
    { name: 'SID_VOICE3_SR', type: 'word', value: '$D414', description: 'Voice 3 sustain/release (54292)', examples: ['poke(SID_VOICE3_SR, $F9)'] },
    {
        name: 'SID_FILTER_CUTOFF_LO', type: 'word', value: '$D415', description: 'Filter cutoff low byte - only lower 3 bits used (54293)', examples: [
            '# 11-bit filter cutoff frequency (0-2047)',
        ]
    },
    { name: 'SID_FILTER_CUTOFF_HI', type: 'word', value: '$D416', description: 'Filter cutoff high byte (54294)', examples: ['poke(SID_FILTER_CUTOFF_HI, $40)'] },
    {
        name: 'SID_FILTER_CTRL', type: 'word', value: '$D417', description: 'Filter control - resonance and voice routing (54295)', examples: [
            '# upper 4 bits = resonance (0-15)\n# lower 4 bits = voice routing mask',
        ]
    },
    {
        name: 'SID_VOLUME', type: 'word', value: '$D418', description: 'Master volume and filter mode (54296)', examples: [
            '# lower 4 bits = volume (0-15)\n# upper 4 bits = filter mode\npoke(SID_VOLUME, 15)  # max volume',
        ]
    },
    // VIC-II Graphics Register Constants
    {
        name: 'VIC_CONTROL1', type: 'word', value: '$D011', description: 'VIC-II control register 1 (53265)', examples: [
            '# bits: YSCROLL, RSEL, DEN, BMM, ECM, RST8',
        ]
    },
    {
        name: 'VIC_CONTROL2', type: 'word', value: '$D016', description: 'VIC-II control register 2 (53270)', examples: [
            '# bits: XSCROLL, CSEL, MCM',
        ]
    },
    {
        name: 'VIC_MEMORY', type: 'word', value: '$D018', description: 'VIC-II memory control register (53272)', examples: [
            '# controls screen, char, and bitmap addresses',
        ]
    },
    {
        name: 'VIC_RASTER', type: 'word', value: '$D012', description: 'VIC-II raster line register (53266)', examples: [
            'line: byte = peek(VIC_RASTER)',
        ]
    },
    {
        name: 'VIC_BORDER', type: 'word', value: '$D020', description: 'VIC-II border color register (53280)', examples: [
            'poke(VIC_BORDER, COLOR_BLACK)',
        ]
    },
    {
        name: 'VIC_BACKGROUND', type: 'word', value: '$D021', description: 'VIC-II background color register (53281)', examples: [
            'poke(VIC_BACKGROUND, COLOR_BLUE)',
        ]
    },
    { name: 'VIC_BACKGROUND1', type: 'word', value: '$D022', description: 'VIC-II extra background 1 (53282)', examples: ['# ECM background 1'] },
    { name: 'VIC_BACKGROUND2', type: 'word', value: '$D023', description: 'VIC-II extra background 2 (53283)', examples: ['# ECM background 2'] },
    { name: 'VIC_BACKGROUND3', type: 'word', value: '$D024', description: 'VIC-II extra background 3 (53284)', examples: ['# ECM background 3'] },
    {
        name: 'COLOR_RAM', type: 'word', value: '$D800', description: 'Color RAM start address (55296)', examples: [
            'poke(COLOR_RAM, COLOR_WHITE)  # first cell color',
        ]
    },
    // Graphics Mode Constants
    { name: 'GFX_TEXT', type: 'byte', value: '0', description: 'Standard character mode (40x25)', examples: ['gfx_mode(GFX_TEXT)'] },
    { name: 'GFX_TEXT_MC', type: 'byte', value: '1', description: 'Multicolor character mode', examples: ['gfx_mode(GFX_TEXT_MC)'] },
    { name: 'GFX_BITMAP', type: 'byte', value: '2', description: 'Standard bitmap mode (320x200)', examples: ['gfx_mode(GFX_BITMAP)'] },
    { name: 'GFX_BITMAP_MC', type: 'byte', value: '3', description: 'Multicolor bitmap mode (160x200)', examples: ['gfx_mode(GFX_BITMAP_MC)'] },
    { name: 'GFX_TEXT_ECM', type: 'byte', value: '4', description: 'Extended background color mode', examples: ['gfx_mode(GFX_TEXT_ECM)'] },
    // VIC Bank Constants
    { name: 'VIC_BANK0', type: 'byte', value: '0', description: 'VIC Bank 0: $0000-$3FFF', examples: ['vic_bank(VIC_BANK0)'] },
    { name: 'VIC_BANK1', type: 'byte', value: '1', description: 'VIC Bank 1: $4000-$7FFF', examples: ['vic_bank(VIC_BANK1)'] },
    { name: 'VIC_BANK2', type: 'byte', value: '2', description: 'VIC Bank 2: $8000-$BFFF', examples: ['vic_bank(VIC_BANK2)'] },
    { name: 'VIC_BANK3', type: 'byte', value: '3', description: 'VIC Bank 3: $C000-$FFFF', examples: ['vic_bank(VIC_BANK3)'] },
    // Raster Constants
    { name: 'RASTER_TOP', type: 'word', value: '50', description: 'First visible raster line', examples: ['wait_raster(RASTER_TOP)'] },
    { name: 'RASTER_BOTTOM', type: 'word', value: '250', description: 'Last visible raster line', examples: ['wait_raster(RASTER_BOTTOM)'] },
    { name: 'RASTER_MAX_PAL', type: 'word', value: '311', description: 'Maximum raster line (PAL)', examples: ['# PAL has 312 lines (0-311)'] },
    { name: 'RASTER_MAX_NTSC', type: 'word', value: '261', description: 'Maximum raster line (NTSC)', examples: ['# NTSC has 262 lines (0-261)'] },
    // Joystick Constants
    {
        name: 'JOY_UP', type: 'byte', value: '1', description: 'Joystick up direction (bit 0)', examples: [
            '# test if joystick is pushed up\njoy: byte = joystick(2)\nif (joy & JOY_UP) != 0:\n    y = y - 1',
        ]
    },
    {
        name: 'JOY_DOWN', type: 'byte', value: '2', description: 'Joystick down direction (bit 1)', examples: [
            '# test if joystick is pushed down\nif (joystick(2) & JOY_DOWN) != 0:\n    y = y + 1',
        ]
    },
    {
        name: 'JOY_LEFT', type: 'byte', value: '4', description: 'Joystick left direction (bit 2)', examples: [
            '# test if joystick is pushed left\nif (joystick(2) & JOY_LEFT) != 0:\n    x = x - 1',
        ]
    },
    {
        name: 'JOY_RIGHT', type: 'byte', value: '8', description: 'Joystick right direction (bit 3)', examples: [
            '# test if joystick is pushed right\nif (joystick(2) & JOY_RIGHT) != 0:\n    x = x + 1',
        ]
    },
    {
        name: 'JOY_FIRE', type: 'byte', value: '16', description: 'Joystick fire button (bit 4)', examples: [
            '# wait for fire button\nwhile (joystick(2) & JOY_FIRE) == 0:\n    pass\nprintln("FIRE!")',
            '# exit loop on fire\nwhile (joystick(2) & JOY_FIRE) == 0:\n    # game logic here\n    pass',
        ]
    },
    { name: 'CIA1_PORTA', type: 'word', value: '$DC00', description: 'CIA1 Port A - Joystick Port 2 (56320)', examples: ['# direct hardware access\njoy: byte = peek(CIA1_PORTA)'] },
    { name: 'CIA1_PORTB', type: 'word', value: '$DC01', description: 'CIA1 Port B - Joystick Port 1 (56321)', examples: ['# direct hardware access (may conflict with keyboard)\njoy: byte = peek(CIA1_PORTB)'] },
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
    {
        name: 'joystick',
        signature: 'joystick(port: byte) -> byte',
        description: 'Reads the state of a joystick from the specified port. Returns a byte with bits set for pressed directions/button. Use JOY_UP, JOY_DOWN, JOY_LEFT, JOY_RIGHT, JOY_FIRE constants to test bits.',
        parameters: [
            { name: 'port', type: 'byte', description: 'Joystick port number (1 or 2). Port 2 is recommended.' },
        ],
        returnType: 'byte',
        examples: [
            '# read joystick port 2\njoy: byte = joystick(2)\nif (joy & JOY_UP) != 0:\n    println("UP")\nif (joy & JOY_FIRE) != 0:\n    println("FIRE")',
            '# control a sprite with joystick\nwhile (joystick(2) & JOY_FIRE) == 0:\n    joy: byte = joystick(2)\n    if (joy & JOY_UP) != 0:\n        y = y - 1\n    if (joy & JOY_DOWN) != 0:\n        y = y + 1\n    sprite_pos(0, x, y)',
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
    // SID Sound Functions - Basic Control
    {
        name: 'sid_reset',
        signature: 'sid_reset()',
        description: 'Clears all 25 SID registers, silencing all sound.',
        parameters: [],
        returnType: null,
        examples: [
            '# initialize SID to silence at program start\ndef main():\n    sid_reset()\n    sid_volume(15)',
        ],
    },
    {
        name: 'sid_volume',
        signature: 'sid_volume(vol: byte)',
        description: 'Sets the master volume (0-15). 0 = silent, 15 = maximum.',
        parameters: [
            { name: 'vol', type: 'byte', description: 'Volume level (0-15)' },
        ],
        returnType: null,
        examples: [
            '# set maximum volume\nsid_volume(15)',
            '# fade out effect\nfor v in 15 downto 0:\n    sid_volume(v)\n    # delay',
        ],
    },
    {
        name: 'sid_frequency',
        signature: 'sid_frequency(voice: byte, freq: word)',
        description: 'Sets the 16-bit frequency for a voice (0-2). Hz = freq × 0.0596 (PAL).',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'freq', type: 'word', description: '16-bit frequency value' },
        ],
        returnType: null,
        examples: [
            '# voice 0 at ~440 Hz (A4)\n# 440 / 0.0596 ≈ 7383\nsid_frequency(0, 7383)',
            '# sweep frequency up\nfreq: word = 1000\nwhile freq < 10000:\n    sid_frequency(0, freq)\n    freq = freq + 100',
        ],
    },
    {
        name: 'sid_waveform',
        signature: 'sid_waveform(voice: byte, wave: byte)',
        description: 'Sets the waveform for a voice. Use WAVE_* constants. Preserves gate bit.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'wave', type: 'byte', description: 'Waveform (WAVE_TRIANGLE/SAWTOOTH/PULSE/NOISE)' },
        ],
        returnType: null,
        examples: [
            '# set different waveforms\nsid_waveform(0, WAVE_PULSE)    # square wave\nsid_waveform(1, WAVE_SAWTOOTH) # bright, buzzy\nsid_waveform(2, WAVE_NOISE)    # drums/effects',
        ],
    },
    {
        name: 'sid_gate',
        signature: 'sid_gate(voice: byte, on: byte)',
        description: 'Controls the gate bit to start/stop the ADSR envelope.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'on', type: 'byte', description: '1 = start note (attack), 0 = release note' },
        ],
        returnType: null,
        examples: [
            '# play a note\nsid_gate(0, 1)  # start attack phase\n# wait for note duration\nsid_gate(0, 0)  # start release phase',
        ],
    },
    // SID Sound Functions - ADSR Envelope
    {
        name: 'sid_attack',
        signature: 'sid_attack(voice: byte, val: byte)',
        description: 'Sets attack time (0-15). Higher = longer attack. 0=2ms, 15=8s.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'val', type: 'byte', description: 'Attack time (0-15)' },
        ],
        returnType: null,
        examples: [
            '# quick attack for percussive sounds\nsid_attack(0, 0)',
            '# slow attack for pads\nsid_attack(0, 10)',
        ],
    },
    {
        name: 'sid_decay',
        signature: 'sid_decay(voice: byte, val: byte)',
        description: 'Sets decay time (0-15). Higher = longer decay.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'val', type: 'byte', description: 'Decay time (0-15)' },
        ],
        returnType: null,
        examples: [
            'sid_decay(0, 5)  # medium decay',
        ],
    },
    {
        name: 'sid_sustain',
        signature: 'sid_sustain(voice: byte, val: byte)',
        description: 'Sets sustain level (0-15). 15 = full volume, 0 = silent.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'val', type: 'byte', description: 'Sustain level (0-15)' },
        ],
        returnType: null,
        examples: [
            '# sustained organ-like sound\nsid_sustain(0, 15)',
            '# pluck/piano-like (no sustain)\nsid_sustain(0, 0)',
        ],
    },
    {
        name: 'sid_release',
        signature: 'sid_release(voice: byte, val: byte)',
        description: 'Sets release time (0-15). Higher = longer release after gate off.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'val', type: 'byte', description: 'Release time (0-15)' },
        ],
        returnType: null,
        examples: [
            '# short release for staccato\nsid_release(0, 2)',
            '# long release for reverb-like effect\nsid_release(0, 12)',
        ],
    },
    {
        name: 'sid_envelope',
        signature: 'sid_envelope(voice: byte, a: byte, d: byte, s: byte, r: byte)',
        description: 'Sets the full ADSR envelope at once.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'a', type: 'byte', description: 'Attack (0-15)' },
            { name: 'd', type: 'byte', description: 'Decay (0-15)' },
            { name: 's', type: 'byte', description: 'Sustain (0-15)' },
            { name: 'r', type: 'byte', description: 'Release (0-15)' },
        ],
        returnType: null,
        examples: [
            '# piano-like: quick attack, medium decay, no sustain\nsid_envelope(0, 0, 5, 0, 8)',
            '# pad: slow attack, full sustain\nsid_envelope(0, 10, 5, 15, 10)',
            '# percussion: instant attack/decay, no sustain\nsid_envelope(0, 0, 2, 0, 2)',
        ],
    },
    // SID Sound Functions - Pulse Width
    {
        name: 'sid_pulse_width',
        signature: 'sid_pulse_width(voice: byte, width: word)',
        description: 'Sets the 12-bit pulse width (0-4095) for pulse waveform. 2048 = 50% duty cycle.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'width', type: 'word', description: 'Pulse width (0-4095)' },
        ],
        returnType: null,
        examples: [
            '# square wave (50% duty cycle)\nsid_waveform(0, WAVE_PULSE)\nsid_pulse_width(0, 2048)',
            '# thin sound (12.5% duty cycle)\nsid_pulse_width(0, 512)',
            '# PWM effect - sweep pulse width\nfor pw in 512 to 3584:\n    sid_pulse_width(0, pw)',
        ],
    },
    // SID Sound Functions - Advanced Voice Control
    {
        name: 'sid_ring_mod',
        signature: 'sid_ring_mod(voice: byte, enable: byte)',
        description: 'Enables ring modulation. Voice N modulated by voice N-1 (0 by 2).',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'enable', type: 'byte', description: '1 = enable, 0 = disable' },
        ],
        returnType: null,
        examples: [
            '# ring modulation creates bell/metallic tones\n# voice 0 is modulated by voice 2\nsid_ring_mod(0, 1)',
        ],
    },
    {
        name: 'sid_sync',
        signature: 'sid_sync(voice: byte, enable: byte)',
        description: 'Enables hard oscillator sync. Voice N syncs to voice N-1.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'enable', type: 'byte', description: '1 = enable, 0 = disable' },
        ],
        returnType: null,
        examples: [
            '# sync creates harsh, metallic timbres\n# classic lead synth sound\nsid_sync(1, 1)',
        ],
    },
    {
        name: 'sid_test',
        signature: 'sid_test(voice: byte, enable: byte)',
        description: 'Controls the test bit. Resets and holds oscillator at zero.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'enable', type: 'byte', description: '1 = hold at zero, 0 = normal' },
        ],
        returnType: null,
        examples: [
            '# used for special effects\nsid_test(0, 1)  # silence oscillator\nsid_test(0, 0)  # release',
        ],
    },
    // SID Sound Functions - Filter Control
    {
        name: 'sid_filter_cutoff',
        signature: 'sid_filter_cutoff(freq: word)',
        description: 'Sets the 11-bit filter cutoff frequency (0-2047).',
        parameters: [
            { name: 'freq', type: 'word', description: 'Filter cutoff (0-2047)' },
        ],
        returnType: null,
        examples: [
            '# low cutoff = dark, muffled sound\nsid_filter_cutoff(256)',
            '# high cutoff = bright sound\nsid_filter_cutoff(1800)',
            '# filter sweep effect\nfor cut in 0 to 2047:\n    sid_filter_cutoff(cut)',
        ],
    },
    {
        name: 'sid_filter_resonance',
        signature: 'sid_filter_resonance(val: byte)',
        description: 'Sets filter resonance (0-15). Higher = more pronounced resonance peak.',
        parameters: [
            { name: 'val', type: 'byte', description: 'Resonance (0-15)' },
        ],
        returnType: null,
        examples: [
            '# subtle filtering\nsid_filter_resonance(4)',
            '# screaming resonance\nsid_filter_resonance(15)',
        ],
    },
    {
        name: 'sid_filter_route',
        signature: 'sid_filter_route(voices: byte)',
        description: 'Routes voices through the filter using a bitmask.',
        parameters: [
            { name: 'voices', type: 'byte', description: 'Bitmask (bit 0 = voice 0, etc.)' },
        ],
        returnType: null,
        examples: [
            '# filter only voice 0\nsid_filter_route(1)',
            '# filter voices 0 and 1\nsid_filter_route(%00000011)',
            '# filter all voices\nsid_filter_route(7)',
        ],
    },
    {
        name: 'sid_filter_mode',
        signature: 'sid_filter_mode(mode: byte)',
        description: 'Sets the filter mode using FILTER_* constants. Can combine modes.',
        parameters: [
            { name: 'mode', type: 'byte', description: 'Filter mode (FILTER_LOWPASS/BANDPASS/HIGHPASS)' },
        ],
        returnType: null,
        examples: [
            '# low-pass for warm bass\nsid_filter_mode(FILTER_LOWPASS)',
            '# band-pass for vocal/nasal sound\nsid_filter_mode(FILTER_BANDPASS)',
            '# notch filter (low + high)\nsid_filter_mode(FILTER_LOWPASS | FILTER_HIGHPASS)',
        ],
    },
    // SID Sound Functions - High-Level Music
    {
        name: 'play_note',
        signature: 'play_note(voice: byte, note: byte, octave: byte)',
        description: 'Plays a musical note using NOTE_* constants and octave (0-7).',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'note', type: 'byte', description: 'Note constant (NOTE_C through NOTE_B)' },
            { name: 'octave', type: 'byte', description: 'Octave (0-7, 4 = middle octave)' },
        ],
        returnType: null,
        examples: [
            '# play middle C\nplay_note(0, NOTE_C, 4)',
            '# play C major chord\nplay_note(0, NOTE_C, 4)\nplay_note(1, NOTE_E, 4)\nplay_note(2, NOTE_G, 4)',
            '# simple melody\nplay_note(0, NOTE_C, 4)\ndelay(10)\nplay_note(0, NOTE_E, 4)\ndelay(10)\nplay_note(0, NOTE_G, 4)',
        ],
    },
    {
        name: 'play_tone',
        signature: 'play_tone(voice: byte, freq: word, wave: byte, duration: byte)',
        description: 'Plays a tone with automatic gate control. Duration is in frames (~1/60s).',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
            { name: 'freq', type: 'word', description: 'Frequency value' },
            { name: 'wave', type: 'byte', description: 'Waveform constant' },
            { name: 'duration', type: 'byte', description: 'Duration in frames' },
        ],
        returnType: null,
        examples: [
            '# play a tone for ~1 second\nplay_tone(0, 7383, WAVE_PULSE, 60)',
        ],
    },
    {
        name: 'sound_off',
        signature: 'sound_off()',
        description: 'Silences all voices by clearing gate bits and waveforms.',
        parameters: [],
        returnType: null,
        examples: [
            '# stop all sound immediately\nsound_off()',
        ],
    },
    {
        name: 'sound_off_voice',
        signature: 'sound_off_voice(voice: byte)',
        description: 'Silences a specific voice.',
        parameters: [
            { name: 'voice', type: 'byte', description: 'Voice number (0-2)' },
        ],
        returnType: null,
        examples: [
            '# stop only voice 0\nsound_off_voice(0)',
        ],
    },
    // Graphics - Display Control
    {
        name: 'border_color',
        signature: 'border_color(color: byte)',
        description: 'Sets the border color (0-15).',
        parameters: [
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            'border_color(0)   # black border\nborder_color(COLOR_BLUE)',
        ],
    },
    {
        name: 'background_color',
        signature: 'background_color(color: byte)',
        description: 'Sets the background color (0-15).',
        parameters: [
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            'background_color(6)  # blue background',
        ],
    },
    {
        name: 'get_border_color',
        signature: 'get_border_color() -> byte',
        description: 'Returns the current border color.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'c: byte = get_border_color()',
        ],
    },
    {
        name: 'get_background_color',
        signature: 'get_background_color() -> byte',
        description: 'Returns the current background color.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'c: byte = get_background_color()',
        ],
    },
    // Graphics - Mode Switching
    {
        name: 'gfx_mode',
        signature: 'gfx_mode(mode: byte)',
        description: 'Switches graphics mode. Use GFX_* constants (0-4).',
        parameters: [
            { name: 'mode', type: 'byte', description: 'Graphics mode (0-4)' },
        ],
        returnType: null,
        examples: [
            'gfx_mode(GFX_BITMAP)  # 320x200 hires\ngfx_mode(GFX_TEXT)    # text mode',
        ],
    },
    {
        name: 'get_gfx_mode',
        signature: 'get_gfx_mode() -> byte',
        description: 'Returns the current graphics mode.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'mode: byte = get_gfx_mode()',
        ],
    },
    {
        name: 'gfx_text',
        signature: 'gfx_text()',
        description: 'Switches to standard text mode (shortcut for gfx_mode(0)).',
        parameters: [],
        returnType: null,
        examples: [
            'gfx_text()  # return to text mode',
        ],
    },
    {
        name: 'gfx_hires',
        signature: 'gfx_hires()',
        description: 'Switches to hires bitmap mode 320x200 (shortcut for gfx_mode(2)).',
        parameters: [],
        returnType: null,
        examples: [
            'gfx_hires()     # 320x200 bitmap\nbitmap_clear()  # clear screen',
        ],
    },
    {
        name: 'gfx_multicolor',
        signature: 'gfx_multicolor()',
        description: 'Switches to multicolor bitmap mode 160x200 (shortcut for gfx_mode(3)).',
        parameters: [],
        returnType: null,
        examples: [
            'gfx_multicolor()  # 160x200, 4 colors per cell',
        ],
    },
    {
        name: 'screen_columns',
        signature: 'screen_columns(cols: byte)',
        description: 'Sets 38 or 40 column mode. 38 columns shows border on sides.',
        parameters: [
            { name: 'cols', type: 'byte', description: '38 or 40' },
        ],
        returnType: null,
        examples: [
            'screen_columns(38)  # for smooth scrolling\nscreen_columns(40)  # normal',
        ],
    },
    {
        name: 'screen_rows',
        signature: 'screen_rows(rows: byte)',
        description: 'Sets 24 or 25 row mode. 24 rows shows border on top/bottom.',
        parameters: [
            { name: 'rows', type: 'byte', description: '24 or 25' },
        ],
        returnType: null,
        examples: [
            'screen_rows(24)  # for smooth scrolling',
        ],
    },
    // Graphics - Memory Configuration
    {
        name: 'vic_bank',
        signature: 'vic_bank(bank: byte)',
        description: 'Sets VIC memory bank (0-3). Use VIC_BANK* constants.',
        parameters: [
            { name: 'bank', type: 'byte', description: 'Bank number (0-3)' },
        ],
        returnType: null,
        examples: [
            'vic_bank(VIC_BANK1)  # use $4000-$7FFF',
        ],
    },
    {
        name: 'get_vic_bank',
        signature: 'get_vic_bank() -> byte',
        description: 'Returns the current VIC bank.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'bank: byte = get_vic_bank()',
        ],
    },
    {
        name: 'screen_address',
        signature: 'screen_address(addr: word)',
        description: 'Sets screen RAM address (relative to VIC bank).',
        parameters: [
            { name: 'addr', type: 'word', description: 'Address within bank' },
        ],
        returnType: null,
        examples: [
            'screen_address($0400)  # default location',
        ],
    },
    {
        name: 'bitmap_address',
        signature: 'bitmap_address(addr: word)',
        description: 'Sets bitmap address. Only $0000 or $2000 within bank.',
        parameters: [
            { name: 'addr', type: 'word', description: 'Address ($0000 or $2000)' },
        ],
        returnType: null,
        examples: [
            'bitmap_address($2000)  # default location',
        ],
    },
    {
        name: 'charset_address',
        signature: 'charset_address(addr: word)',
        description: 'Sets character set address (relative to VIC bank).',
        parameters: [
            { name: 'addr', type: 'word', description: 'Address within bank' },
        ],
        returnType: null,
        examples: [
            'charset_address($0800)  # custom charset',
        ],
    },
    // Graphics - Pixel Operations
    {
        name: 'plot',
        signature: 'plot(x: word, y: byte)',
        description: 'Sets a pixel in hires bitmap mode.',
        parameters: [
            { name: 'x', type: 'word', description: 'X coordinate (0-319)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
        ],
        returnType: null,
        examples: [
            'plot(160, 100)  # pixel at center',
        ],
    },
    {
        name: 'unplot',
        signature: 'unplot(x: word, y: byte)',
        description: 'Clears a pixel in hires bitmap mode.',
        parameters: [
            { name: 'x', type: 'word', description: 'X coordinate (0-319)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
        ],
        returnType: null,
        examples: [
            'unplot(160, 100)  # clear pixel',
        ],
    },
    {
        name: 'point',
        signature: 'point(x: word, y: byte) -> bool',
        description: 'Tests if a pixel is set in hires bitmap mode.',
        parameters: [
            { name: 'x', type: 'word', description: 'X coordinate (0-319)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
        ],
        returnType: 'bool',
        examples: [
            'if point(160, 100):\n    println("PIXEL SET")',
        ],
    },
    {
        name: 'plot_mc',
        signature: 'plot_mc(x: byte, y: byte, color: byte)',
        description: 'Sets a pixel in multicolor bitmap mode (4 colors per cell).',
        parameters: [
            { name: 'x', type: 'byte', description: 'X coordinate (0-159)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
            { name: 'color', type: 'byte', description: 'Color (0-3)' },
        ],
        returnType: null,
        examples: [
            '# colors: 0=bg, 1=screen hi, 2=screen lo, 3=color RAM\nplot_mc(80, 100, 2)',
        ],
    },
    {
        name: 'point_mc',
        signature: 'point_mc(x: byte, y: byte) -> byte',
        description: 'Gets pixel color in multicolor bitmap mode.',
        parameters: [
            { name: 'x', type: 'byte', description: 'X coordinate (0-159)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
        ],
        returnType: 'byte',
        examples: [
            'c: byte = point_mc(80, 100)  # 0-3',
        ],
    },
    {
        name: 'bitmap_clear',
        signature: 'bitmap_clear()',
        description: 'Clears the bitmap (fills with 0).',
        parameters: [],
        returnType: null,
        examples: [
            'gfx_hires()\nbitmap_clear()',
        ],
    },
    {
        name: 'bitmap_fill',
        signature: 'bitmap_fill(pattern: byte)',
        description: 'Fills the bitmap with a pattern byte.',
        parameters: [
            { name: 'pattern', type: 'byte', description: 'Fill pattern' },
        ],
        returnType: null,
        examples: [
            'bitmap_fill($55)  # vertical stripes',
        ],
    },
    // Graphics - Drawing Primitives
    {
        name: 'line',
        signature: 'line(x1: word, y1: byte, x2: word, y2: byte)',
        description: 'Draws a line in hires bitmap mode (Bresenham algorithm).',
        parameters: [
            { name: 'x1', type: 'word', description: 'Start X (0-319)' },
            { name: 'y1', type: 'byte', description: 'Start Y (0-199)' },
            { name: 'x2', type: 'word', description: 'End X (0-319)' },
            { name: 'y2', type: 'byte', description: 'End Y (0-199)' },
        ],
        returnType: null,
        examples: [
            'line(0, 0, 319, 199)  # diagonal',
        ],
    },
    {
        name: 'hline',
        signature: 'hline(x: word, y: byte, length: word)',
        description: 'Draws a fast horizontal line.',
        parameters: [
            { name: 'x', type: 'word', description: 'Start X (0-319)' },
            { name: 'y', type: 'byte', description: 'Y coordinate (0-199)' },
            { name: 'length', type: 'word', description: 'Line length' },
        ],
        returnType: null,
        examples: [
            'hline(0, 100, 320)  # full width line',
        ],
    },
    {
        name: 'vline',
        signature: 'vline(x: word, y: byte, length: byte)',
        description: 'Draws a fast vertical line.',
        parameters: [
            { name: 'x', type: 'word', description: 'X coordinate (0-319)' },
            { name: 'y', type: 'byte', description: 'Start Y (0-199)' },
            { name: 'length', type: 'byte', description: 'Line length' },
        ],
        returnType: null,
        examples: [
            'vline(160, 0, 200)  # full height line',
        ],
    },
    {
        name: 'rect',
        signature: 'rect(x: word, y: byte, width: word, height: byte)',
        description: 'Draws a rectangle outline.',
        parameters: [
            { name: 'x', type: 'word', description: 'Left X (0-319)' },
            { name: 'y', type: 'byte', description: 'Top Y (0-199)' },
            { name: 'width', type: 'word', description: 'Width' },
            { name: 'height', type: 'byte', description: 'Height' },
        ],
        returnType: null,
        examples: [
            'rect(10, 10, 100, 80)',
        ],
    },
    {
        name: 'rect_fill',
        signature: 'rect_fill(x: word, y: byte, width: word, height: byte)',
        description: 'Draws a filled rectangle.',
        parameters: [
            { name: 'x', type: 'word', description: 'Left X (0-319)' },
            { name: 'y', type: 'byte', description: 'Top Y (0-199)' },
            { name: 'width', type: 'word', description: 'Width' },
            { name: 'height', type: 'byte', description: 'Height' },
        ],
        returnType: null,
        examples: [
            'rect_fill(10, 10, 100, 80)',
        ],
    },
    // Graphics - Cell Color Control
    {
        name: 'cell_color',
        signature: 'cell_color(cx: byte, cy: byte, fg: byte, bg: byte)',
        description: 'Sets cell foreground and background colors (bitmap mode).',
        parameters: [
            { name: 'cx', type: 'byte', description: 'Cell X (0-39)' },
            { name: 'cy', type: 'byte', description: 'Cell Y (0-24)' },
            { name: 'fg', type: 'byte', description: 'Foreground color (0-15)' },
            { name: 'bg', type: 'byte', description: 'Background color (0-15)' },
        ],
        returnType: null,
        examples: [
            'cell_color(20, 12, 1, 0)  # white on black',
        ],
    },
    {
        name: 'get_cell_color',
        signature: 'get_cell_color(cx: byte, cy: byte) -> byte',
        description: 'Gets cell colors (fg in high nibble, bg in low nibble).',
        parameters: [
            { name: 'cx', type: 'byte', description: 'Cell X (0-39)' },
            { name: 'cy', type: 'byte', description: 'Cell Y (0-24)' },
        ],
        returnType: 'byte',
        examples: [
            'c: byte = get_cell_color(20, 12)',
        ],
    },
    {
        name: 'color_ram',
        signature: 'color_ram(cx: byte, cy: byte, color: byte)',
        description: 'Sets color RAM at cell position.',
        parameters: [
            { name: 'cx', type: 'byte', description: 'Cell X (0-39)' },
            { name: 'cy', type: 'byte', description: 'Cell Y (0-24)' },
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            'color_ram(20, 12, 5)  # green',
        ],
    },
    {
        name: 'get_color_ram',
        signature: 'get_color_ram(cx: byte, cy: byte) -> byte',
        description: 'Gets color RAM value at cell position.',
        parameters: [
            { name: 'cx', type: 'byte', description: 'Cell X (0-39)' },
            { name: 'cy', type: 'byte', description: 'Cell Y (0-24)' },
        ],
        returnType: 'byte',
        examples: [
            'c: byte = get_color_ram(20, 12)',
        ],
    },
    {
        name: 'fill_colors',
        signature: 'fill_colors(fg: byte, bg: byte)',
        description: 'Fills all cells with foreground/background colors.',
        parameters: [
            { name: 'fg', type: 'byte', description: 'Foreground color (0-15)' },
            { name: 'bg', type: 'byte', description: 'Background color (0-15)' },
        ],
        returnType: null,
        examples: [
            'fill_colors(1, 0)  # white on black',
        ],
    },
    {
        name: 'fill_color_ram',
        signature: 'fill_color_ram(color: byte)',
        description: 'Fills color RAM with a single color.',
        parameters: [
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            'fill_color_ram(1)  # all white',
        ],
    },
    // Graphics - Hardware Scrolling
    {
        name: 'scroll_x',
        signature: 'scroll_x(offset: byte)',
        description: 'Sets horizontal scroll offset (0-7 pixels).',
        parameters: [
            { name: 'offset', type: 'byte', description: 'Scroll offset (0-7)' },
        ],
        returnType: null,
        examples: [
            'scroll_x(4)  # scroll 4 pixels right',
        ],
    },
    {
        name: 'scroll_y',
        signature: 'scroll_y(offset: byte)',
        description: 'Sets vertical scroll offset (0-7 pixels).',
        parameters: [
            { name: 'offset', type: 'byte', description: 'Scroll offset (0-7)' },
        ],
        returnType: null,
        examples: [
            'scroll_y(4)  # scroll 4 pixels down',
        ],
    },
    {
        name: 'get_scroll_x',
        signature: 'get_scroll_x() -> byte',
        description: 'Gets current horizontal scroll offset.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'sx: byte = get_scroll_x()',
        ],
    },
    {
        name: 'get_scroll_y',
        signature: 'get_scroll_y() -> byte',
        description: 'Gets current vertical scroll offset.',
        parameters: [],
        returnType: 'byte',
        examples: [
            'sy: byte = get_scroll_y()',
        ],
    },
    // Graphics - Raster Functions
    {
        name: 'raster',
        signature: 'raster() -> word',
        description: 'Returns current raster line (0-311 PAL, 0-261 NTSC).',
        parameters: [],
        returnType: 'word',
        examples: [
            'r: word = raster()',
        ],
    },
    {
        name: 'wait_raster',
        signature: 'wait_raster(line: word)',
        description: 'Waits until raster reaches the specified line.',
        parameters: [
            { name: 'line', type: 'word', description: 'Raster line (0-311)' },
        ],
        returnType: null,
        examples: [
            'wait_raster(250)  # sync to line 250\nborder_color(2)   # change color',
        ],
    },
    // Graphics - Extended Color Mode
    {
        name: 'ecm_background',
        signature: 'ecm_background(index: byte, color: byte)',
        description: 'Sets ECM background color (index 0-3 maps to $D021-$D024).',
        parameters: [
            { name: 'index', type: 'byte', description: 'Background index (0-3)' },
            { name: 'color', type: 'byte', description: 'Color (0-15)' },
        ],
        returnType: null,
        examples: [
            '# ECM uses 4 background colors\necm_background(0, 0)  # black\necm_background(1, 2)  # red',
        ],
    },
    {
        name: 'get_ecm_background',
        signature: 'get_ecm_background(index: byte) -> byte',
        description: 'Gets ECM background color.',
        parameters: [
            { name: 'index', type: 'byte', description: 'Background index (0-3)' },
        ],
        returnType: 'byte',
        examples: [
            'c: byte = get_ecm_background(1)',
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
