// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! MOS 6510 CPU instruction encoding for the Cobra64 compiler.
//!
//! This module provides opcode constants and instruction encoding
//! for the 6510 CPU used in the Commodore 64.

/// Opcodes for the 6510 CPU.
///
/// Naming convention: INSTR_MODE where MODE is:
/// - IMM: Immediate (#$xx)
/// - ZP: Zero Page ($xx)
/// - ZPX: Zero Page,X ($xx,X)
/// - ZPY: Zero Page,Y ($xx,Y)
/// - ABS: Absolute ($xxxx)
/// - ABX: Absolute,X ($xxxx,X)
/// - ABY: Absolute,Y ($xxxx,Y)
/// - IND: Indirect (($xxxx))
/// - IZX: Indexed Indirect (($xx,X))
/// - IZY: Indirect Indexed (($xx),Y)
/// - IMP: Implied
/// - ACC: Accumulator
/// - REL: Relative (branches)
#[allow(dead_code)]
pub mod opcodes {
    // Load/Store Operations
    pub const LDA_IMM: u8 = 0xA9;
    pub const LDA_ZP: u8 = 0xA5;
    pub const LDA_ZPX: u8 = 0xB5;
    pub const LDA_ABS: u8 = 0xAD;
    pub const LDA_ABX: u8 = 0xBD;
    pub const LDA_ABY: u8 = 0xB9;
    pub const LDA_IZX: u8 = 0xA1;
    pub const LDA_IZY: u8 = 0xB1;

    pub const LDX_IMM: u8 = 0xA2;
    pub const LDX_ZP: u8 = 0xA6;
    pub const LDX_ZPY: u8 = 0xB6;
    pub const LDX_ABS: u8 = 0xAE;
    pub const LDX_ABY: u8 = 0xBE;

    pub const LDY_IMM: u8 = 0xA0;
    pub const LDY_ZP: u8 = 0xA4;
    pub const LDY_ZPX: u8 = 0xB4;
    pub const LDY_ABS: u8 = 0xAC;
    pub const LDY_ABX: u8 = 0xBC;

    pub const STA_ZP: u8 = 0x85;
    pub const STA_ZPX: u8 = 0x95;
    pub const STA_ABS: u8 = 0x8D;
    pub const STA_ABX: u8 = 0x9D;
    pub const STA_ABY: u8 = 0x99;
    pub const STA_IZX: u8 = 0x81;
    pub const STA_IZY: u8 = 0x91;

    pub const STX_ZP: u8 = 0x86;
    pub const STX_ZPY: u8 = 0x96;
    pub const STX_ABS: u8 = 0x8E;

    pub const STY_ZP: u8 = 0x84;
    pub const STY_ZPX: u8 = 0x94;
    pub const STY_ABS: u8 = 0x8C;

    // Arithmetic Operations
    pub const ADC_IMM: u8 = 0x69;
    pub const ADC_ZP: u8 = 0x65;
    pub const ADC_ZPX: u8 = 0x75;
    pub const ADC_ABS: u8 = 0x6D;
    pub const ADC_ABX: u8 = 0x7D;
    pub const ADC_ABY: u8 = 0x79;
    pub const ADC_IZX: u8 = 0x61;
    pub const ADC_IZY: u8 = 0x71;

    pub const SBC_IMM: u8 = 0xE9;
    pub const SBC_ZP: u8 = 0xE5;
    pub const SBC_ZPX: u8 = 0xF5;
    pub const SBC_ABS: u8 = 0xED;
    pub const SBC_ABX: u8 = 0xFD;
    pub const SBC_ABY: u8 = 0xF9;
    pub const SBC_IZX: u8 = 0xE1;
    pub const SBC_IZY: u8 = 0xF1;

    // Increment/Decrement
    pub const INC_ZP: u8 = 0xE6;
    pub const INC_ZPX: u8 = 0xF6;
    pub const INC_ABS: u8 = 0xEE;
    pub const INC_ABX: u8 = 0xFE;

    pub const DEC_ZP: u8 = 0xC6;
    pub const DEC_ZPX: u8 = 0xD6;
    pub const DEC_ABS: u8 = 0xCE;
    pub const DEC_ABX: u8 = 0xDE;

    pub const INX: u8 = 0xE8;
    pub const INY: u8 = 0xC8;
    pub const DEX: u8 = 0xCA;
    pub const DEY: u8 = 0x88;

    // Logical Operations
    pub const AND_IMM: u8 = 0x29;
    pub const AND_ZP: u8 = 0x25;
    pub const AND_ZPX: u8 = 0x35;
    pub const AND_ABS: u8 = 0x2D;
    pub const AND_ABX: u8 = 0x3D;
    pub const AND_ABY: u8 = 0x39;
    pub const AND_IZX: u8 = 0x21;
    pub const AND_IZY: u8 = 0x31;

    pub const ORA_IMM: u8 = 0x09;
    pub const ORA_ZP: u8 = 0x05;
    pub const ORA_ZPX: u8 = 0x15;
    pub const ORA_ABS: u8 = 0x0D;
    pub const ORA_ABX: u8 = 0x1D;
    pub const ORA_ABY: u8 = 0x19;
    pub const ORA_IZX: u8 = 0x01;
    pub const ORA_IZY: u8 = 0x11;

    pub const EOR_IMM: u8 = 0x49;
    pub const EOR_ZP: u8 = 0x45;
    pub const EOR_ZPX: u8 = 0x55;
    pub const EOR_ABS: u8 = 0x4D;
    pub const EOR_ABX: u8 = 0x5D;
    pub const EOR_ABY: u8 = 0x59;
    pub const EOR_IZX: u8 = 0x41;
    pub const EOR_IZY: u8 = 0x51;

    // Shift and Rotate
    pub const ASL_ACC: u8 = 0x0A;
    pub const ASL_ZP: u8 = 0x06;
    pub const ASL_ZPX: u8 = 0x16;
    pub const ASL_ABS: u8 = 0x0E;
    pub const ASL_ABX: u8 = 0x1E;

    pub const LSR_ACC: u8 = 0x4A;
    pub const LSR_ZP: u8 = 0x46;
    pub const LSR_ZPX: u8 = 0x56;
    pub const LSR_ABS: u8 = 0x4E;
    pub const LSR_ABX: u8 = 0x5E;

    pub const ROL_ACC: u8 = 0x2A;
    pub const ROL_ZP: u8 = 0x26;
    pub const ROL_ZPX: u8 = 0x36;
    pub const ROL_ABS: u8 = 0x2E;
    pub const ROL_ABX: u8 = 0x3E;

    pub const ROR_ACC: u8 = 0x6A;
    pub const ROR_ZP: u8 = 0x66;
    pub const ROR_ZPX: u8 = 0x76;
    pub const ROR_ABS: u8 = 0x6E;
    pub const ROR_ABX: u8 = 0x7E;

    // Compare Operations
    pub const CMP_IMM: u8 = 0xC9;
    pub const CMP_ZP: u8 = 0xC5;
    pub const CMP_ZPX: u8 = 0xD5;
    pub const CMP_ABS: u8 = 0xCD;
    pub const CMP_ABX: u8 = 0xDD;
    pub const CMP_ABY: u8 = 0xD9;
    pub const CMP_IZX: u8 = 0xC1;
    pub const CMP_IZY: u8 = 0xD1;

    pub const CPX_IMM: u8 = 0xE0;
    pub const CPX_ZP: u8 = 0xE4;
    pub const CPX_ABS: u8 = 0xEC;

    pub const CPY_IMM: u8 = 0xC0;
    pub const CPY_ZP: u8 = 0xC4;
    pub const CPY_ABS: u8 = 0xCC;

    // Branch Operations (all relative)
    pub const BCC: u8 = 0x90; // Branch if Carry Clear
    pub const BCS: u8 = 0xB0; // Branch if Carry Set
    pub const BEQ: u8 = 0xF0; // Branch if Equal (Zero set)
    pub const BNE: u8 = 0xD0; // Branch if Not Equal (Zero clear)
    pub const BMI: u8 = 0x30; // Branch if Minus (Negative set)
    pub const BPL: u8 = 0x10; // Branch if Plus (Negative clear)
    pub const BVC: u8 = 0x50; // Branch if Overflow Clear
    pub const BVS: u8 = 0x70; // Branch if Overflow Set

    // Jump and Subroutine
    pub const JMP_ABS: u8 = 0x4C;
    pub const JMP_IND: u8 = 0x6C;
    pub const JSR: u8 = 0x20;
    pub const RTS: u8 = 0x60;
    pub const RTI: u8 = 0x40;

    // Stack Operations
    pub const PHA: u8 = 0x48;
    pub const PLA: u8 = 0x68;
    pub const PHP: u8 = 0x08;
    pub const PLP: u8 = 0x28;

    // Status Flag Operations
    pub const CLC: u8 = 0x18;
    pub const SEC: u8 = 0x38;
    pub const CLI: u8 = 0x58;
    pub const SEI: u8 = 0x78;
    pub const CLV: u8 = 0xB8;
    pub const CLD: u8 = 0xD8;
    pub const SED: u8 = 0xF8;

    // Transfer Operations
    pub const TAX: u8 = 0xAA;
    pub const TXA: u8 = 0x8A;
    pub const TAY: u8 = 0xA8;
    pub const TYA: u8 = 0x98;
    pub const TSX: u8 = 0xBA;
    pub const TXS: u8 = 0x9A;

    // Bit Test
    pub const BIT_ZP: u8 = 0x24;
    pub const BIT_ABS: u8 = 0x2C;

    // Miscellaneous
    pub const NOP: u8 = 0xEA;
    pub const BRK: u8 = 0x00;
}

/// C64 KERNAL ROM routine addresses.
#[allow(dead_code)]
pub mod kernal {
    /// Output a character to the current output device.
    /// Input: A = character (PETSCII)
    pub const CHROUT: u16 = 0xFFD2;

    /// Get a character from the keyboard buffer.
    /// Output: A = character (0 if none available)
    pub const GETIN: u16 = 0xFFE4;

    /// Set or get cursor position.
    /// Input: Carry clear = set position, X = column, Y = row
    /// Input: Carry set = get position
    /// Output: X = column, Y = row
    pub const PLOT: u16 = 0xFFF0;

    /// Clear the screen.
    pub const CLRSCR: u16 = 0xE544;

    /// Input a line from keyboard.
    /// Stores input at address in $7A-$7B.
    pub const CHRIN: u16 = 0xFFCF;

    /// Open a logical file.
    pub const OPEN: u16 = 0xFFC0;

    /// Close a logical file.
    pub const CLOSE: u16 = 0xFFC3;

    /// Set input channel.
    pub const CHKIN: u16 = 0xFFC6;

    /// Set output channel.
    pub const CHKOUT: u16 = 0xFFC9;

    /// Restore default I/O.
    pub const CLRCHN: u16 = 0xFFCC;

    /// Read/set system status.
    pub const READST: u16 = 0xFFB7;
}

/// C64 memory locations.
#[allow(dead_code)]
pub mod c64 {
    /// Screen memory start.
    pub const SCREEN_RAM: u16 = 0x0400;

    /// Color memory start.
    pub const COLOR_RAM: u16 = 0xD800;

    /// Border color register.
    pub const BORDER_COLOR: u16 = 0xD020;

    /// Background color register.
    pub const BACKGROUND_COLOR: u16 = 0xD021;

    /// Current cursor column (0-39).
    pub const CURSOR_COL: u16 = 0x00D3;

    /// Current cursor row (0-24).
    pub const CURSOR_ROW: u16 = 0x00D6;

    /// Keyboard buffer.
    pub const KEYBOARD_BUFFER: u16 = 0x0277;

    /// Number of characters in keyboard buffer.
    pub const KEYBOARD_BUFFER_LEN: u16 = 0x00C6;

    /// Input buffer for readln (256 bytes at $C100).
    pub const INPUT_BUFFER: u16 = 0xC100;
}

/// Zero page locations for temporary storage.
/// We use $22-$23 for string pointer (BASIC text pointer, safe in ML).
/// $FB-$FE may be used by KERNAL CHROUT!
#[allow(dead_code)]
pub mod zeropage {
    /// Temporary storage 1 (word) - using $22/$23 instead of $FB/$FC.
    pub const TMP1: u8 = 0x22;
    pub const TMP1_HI: u8 = 0x23;

    /// Temporary storage 2 (word).
    pub const TMP2: u8 = 0xFD;
    pub const TMP2_HI: u8 = 0xFE;

    /// Additional temporaries ($02-$05 are safe on C64).
    /// TMP3/TMP3_HI form a 16-bit word for fixed-point operations.
    pub const TMP3: u8 = 0x02;
    pub const TMP3_HI: u8 = 0x03;
    pub const TMP4: u8 = 0x04;
    pub const TMP5: u8 = 0x05;

    /// Pointer for string operations ($22-$23).
    pub const STR_PTR: u8 = 0x22;
    pub const STR_PTR_HI: u8 = 0x23;

    /// PRNG state (16-bit seed) - using $06/$07 which are safe on C64.
    pub const PRNG_LO: u8 = 0x06;
    pub const PRNG_HI: u8 = 0x07;
}

/// SID (Sound Interface Device) registers.
#[allow(dead_code)]
pub mod sid {
    /// SID base address.
    pub const BASE: u16 = 0xD400;

    // Voice 1 registers
    /// Voice 1 frequency low byte.
    pub const VOICE1_FREQ_LO: u16 = 0xD400;
    /// Voice 1 frequency high byte.
    pub const VOICE1_FREQ_HI: u16 = 0xD401;
    /// Voice 1 pulse width low byte.
    pub const VOICE1_PW_LO: u16 = 0xD402;
    /// Voice 1 pulse width high byte (bits 0-3 only).
    pub const VOICE1_PW_HI: u16 = 0xD403;
    /// Voice 1 control register (waveform, gate, sync, ring, test).
    pub const VOICE1_CTRL: u16 = 0xD404;
    /// Voice 1 attack/decay (high nibble = attack, low nibble = decay).
    pub const VOICE1_AD: u16 = 0xD405;
    /// Voice 1 sustain/release (high nibble = sustain, low nibble = release).
    pub const VOICE1_SR: u16 = 0xD406;

    // Voice 2 registers
    /// Voice 2 frequency low byte.
    pub const VOICE2_FREQ_LO: u16 = 0xD407;
    /// Voice 2 frequency high byte.
    pub const VOICE2_FREQ_HI: u16 = 0xD408;
    /// Voice 2 pulse width low byte.
    pub const VOICE2_PW_LO: u16 = 0xD409;
    /// Voice 2 pulse width high byte (bits 0-3 only).
    pub const VOICE2_PW_HI: u16 = 0xD40A;
    /// Voice 2 control register.
    pub const VOICE2_CTRL: u16 = 0xD40B;
    /// Voice 2 attack/decay.
    pub const VOICE2_AD: u16 = 0xD40C;
    /// Voice 2 sustain/release.
    pub const VOICE2_SR: u16 = 0xD40D;

    // Voice 3 registers
    /// Voice 3 frequency low byte.
    pub const VOICE3_FREQ_LO: u16 = 0xD40E;
    /// Voice 3 frequency high byte.
    pub const VOICE3_FREQ_HI: u16 = 0xD40F;
    /// Voice 3 pulse width low byte.
    pub const VOICE3_PW_LO: u16 = 0xD410;
    /// Voice 3 pulse width high byte (bits 0-3 only).
    pub const VOICE3_PW_HI: u16 = 0xD411;
    /// Voice 3 control register.
    pub const VOICE3_CTRL: u16 = 0xD412;
    /// Voice 3 attack/decay.
    pub const VOICE3_AD: u16 = 0xD413;
    /// Voice 3 sustain/release.
    pub const VOICE3_SR: u16 = 0xD414;

    // Filter and volume registers
    /// Filter cutoff frequency low byte (bits 0-2 only).
    pub const FILTER_CUTOFF_LO: u16 = 0xD415;
    /// Filter cutoff frequency high byte.
    pub const FILTER_CUTOFF_HI: u16 = 0xD416;
    /// Filter resonance and routing.
    pub const FILTER_CTRL: u16 = 0xD417;
    /// Volume and filter mode (bits 0-3 = volume, bits 4-7 = filter mode).
    pub const VOLUME_FILTER: u16 = 0xD418;

    // Read-only registers
    /// Voice 3 oscillator output (random when noise waveform).
    pub const VOICE3_OSC: u16 = 0xD41B;
    /// Voice 3 envelope output.
    pub const VOICE3_ENV: u16 = 0xD41C;

    // Waveform bits for control register
    /// Waveform: Triangle.
    pub const WAVEFORM_TRIANGLE: u8 = 0x10;
    /// Waveform: Sawtooth.
    pub const WAVEFORM_SAWTOOTH: u8 = 0x20;
    /// Waveform: Pulse/Square.
    pub const WAVEFORM_PULSE: u8 = 0x40;
    /// Waveform: Noise.
    pub const WAVEFORM_NOISE: u8 = 0x80;

    // Control register bits
    /// Gate bit (start/stop envelope).
    pub const GATE: u8 = 0x01;
    /// Sync bit (synchronize with voice 3/1/2).
    pub const SYNC: u8 = 0x02;
    /// Ring modulation bit.
    pub const RING_MOD: u8 = 0x04;
    /// Test bit (resets oscillator).
    pub const TEST: u8 = 0x08;
}

/// VIC-II registers.
#[allow(dead_code)]
pub mod vic {
    /// VIC-II base address.
    pub const BASE: u16 = 0xD000;

    // Sprite position registers
    /// Sprite 0 X position (bits 0-7).
    pub const SPRITE0_X: u16 = 0xD000;
    /// Sprite 0 Y position.
    pub const SPRITE0_Y: u16 = 0xD001;
    /// Sprite 1 X position (bits 0-7).
    pub const SPRITE1_X: u16 = 0xD002;
    /// Sprite 1 Y position.
    pub const SPRITE1_Y: u16 = 0xD003;
    /// Sprite 2 X position (bits 0-7).
    pub const SPRITE2_X: u16 = 0xD004;
    /// Sprite 2 Y position.
    pub const SPRITE2_Y: u16 = 0xD005;
    /// Sprite 3 X position (bits 0-7).
    pub const SPRITE3_X: u16 = 0xD006;
    /// Sprite 3 Y position.
    pub const SPRITE3_Y: u16 = 0xD007;
    /// Sprite 4 X position (bits 0-7).
    pub const SPRITE4_X: u16 = 0xD008;
    /// Sprite 4 Y position.
    pub const SPRITE4_Y: u16 = 0xD009;
    /// Sprite 5 X position (bits 0-7).
    pub const SPRITE5_X: u16 = 0xD00A;
    /// Sprite 5 Y position.
    pub const SPRITE5_Y: u16 = 0xD00B;
    /// Sprite 6 X position (bits 0-7).
    pub const SPRITE6_X: u16 = 0xD00C;
    /// Sprite 6 Y position.
    pub const SPRITE6_Y: u16 = 0xD00D;
    /// Sprite 7 X position (bits 0-7).
    pub const SPRITE7_X: u16 = 0xD00E;
    /// Sprite 7 Y position.
    pub const SPRITE7_Y: u16 = 0xD00F;

    /// Sprite X position MSB (bit 8 for all sprites).
    /// Bit 0 = Sprite 0, Bit 7 = Sprite 7.
    pub const SPRITE_X_MSB: u16 = 0xD010;

    // Control registers
    /// Control register 1 (screen height, bitmap mode, etc.).
    pub const CONTROL1: u16 = 0xD011;
    /// Current raster line (bits 0-7).
    pub const RASTER: u16 = 0xD012;
    /// Light pen X position.
    pub const LIGHTPEN_X: u16 = 0xD013;
    /// Light pen Y position.
    pub const LIGHTPEN_Y: u16 = 0xD014;
    /// Sprite enable register. Bit 0 = Sprite 0, etc.
    pub const SPRITE_ENABLE: u16 = 0xD015;
    /// Control register 2 (screen width, multicolor mode).
    pub const CONTROL2: u16 = 0xD016;
    /// Sprite Y expansion (double height). Bit 0 = Sprite 0, etc.
    pub const SPRITE_EXPAND_Y: u16 = 0xD017;
    /// Memory pointers (screen and character memory).
    pub const MEMORY_POINTERS: u16 = 0xD018;
    /// Interrupt register.
    pub const INTERRUPT: u16 = 0xD019;
    /// Interrupt enable register.
    pub const INTERRUPT_ENABLE: u16 = 0xD01A;
    /// Sprite to background priority. Bit=1: background in front.
    pub const SPRITE_PRIORITY: u16 = 0xD01B;
    /// Sprite multicolor mode. Bit 0 = Sprite 0, etc.
    pub const SPRITE_MULTICOLOR: u16 = 0xD01C;
    /// Sprite X expansion (double width). Bit 0 = Sprite 0, etc.
    pub const SPRITE_EXPAND_X: u16 = 0xD01D;
    /// Sprite-sprite collision register (read clears).
    pub const SPRITE_COLLISION: u16 = 0xD01E;
    /// Sprite-background collision register (read clears).
    pub const SPRITE_BG_COLLISION: u16 = 0xD01F;

    // Color registers
    /// Border color.
    pub const BORDER_COLOR: u16 = 0xD020;
    /// Background color 0.
    pub const BACKGROUND_COLOR: u16 = 0xD021;
    /// Background color 1 (multicolor mode).
    pub const BACKGROUND_COLOR1: u16 = 0xD022;
    /// Background color 2 (multicolor mode).
    pub const BACKGROUND_COLOR2: u16 = 0xD023;
    /// Background color 3 (multicolor mode).
    pub const BACKGROUND_COLOR3: u16 = 0xD024;
    /// Sprite multicolor 0 (shared color for bit pattern "01").
    pub const SPRITE_MULTICOLOR0: u16 = 0xD025;
    /// Sprite multicolor 1 (shared color for bit pattern "11").
    pub const SPRITE_MULTICOLOR1: u16 = 0xD026;

    // Individual sprite colors
    /// Sprite 0 color.
    pub const SPRITE0_COLOR: u16 = 0xD027;
    /// Sprite 1 color.
    pub const SPRITE1_COLOR: u16 = 0xD028;
    /// Sprite 2 color.
    pub const SPRITE2_COLOR: u16 = 0xD029;
    /// Sprite 3 color.
    pub const SPRITE3_COLOR: u16 = 0xD02A;
    /// Sprite 4 color.
    pub const SPRITE4_COLOR: u16 = 0xD02B;
    /// Sprite 5 color.
    pub const SPRITE5_COLOR: u16 = 0xD02C;
    /// Sprite 6 color.
    pub const SPRITE6_COLOR: u16 = 0xD02D;
    /// Sprite 7 color.
    pub const SPRITE7_COLOR: u16 = 0xD02E;

    // Sprite data pointers (in default screen memory at $0400)
    /// Sprite pointer base address (screen RAM + $03F8).
    pub const SPRITE_POINTERS: u16 = 0x07F8;
}

/// CIA (Complex Interface Adapter) registers.
#[allow(dead_code)]
pub mod cia {
    /// CIA1 base address.
    pub const CIA1_BASE: u16 = 0xDC00;

    /// CIA1 Port A - Joystick Port 2 and keyboard columns.
    /// Read: Bit 0-3 = Joystick directions, Bit 4 = Fire button (active low)
    /// Joystick bits: 0=Up, 1=Down, 2=Left, 3=Right, 4=Fire (0 = pressed)
    pub const CIA1_PORT_A: u16 = 0xDC00;

    /// CIA1 Port B - Joystick Port 1 and keyboard rows.
    /// Read: Bit 0-3 = Joystick directions, Bit 4 = Fire button (active low)
    /// Joystick bits: 0=Up, 1=Down, 2=Left, 3=Right, 4=Fire (0 = pressed)
    pub const CIA1_PORT_B: u16 = 0xDC01;

    /// CIA1 Data Direction Register A.
    pub const CIA1_DDR_A: u16 = 0xDC02;

    /// CIA1 Data Direction Register B.
    pub const CIA1_DDR_B: u16 = 0xDC03;

    /// CIA1 Timer A low byte (free-running counter).
    pub const CIA1_TIMER_A_LO: u16 = 0xDC04;
    /// CIA1 Timer A high byte.
    pub const CIA1_TIMER_A_HI: u16 = 0xDC05;
    /// CIA1 Timer B low byte.
    pub const CIA1_TIMER_B_LO: u16 = 0xDC06;
    /// CIA1 Timer B high byte.
    pub const CIA1_TIMER_B_HI: u16 = 0xDC07;

    /// Joystick bit masks (directly from CIA port, active-low).
    /// Note: 0 = pressed, 1 = not pressed
    pub const JOY_UP_MASK: u8 = 0x01;
    pub const JOY_DOWN_MASK: u8 = 0x02;
    pub const JOY_LEFT_MASK: u8 = 0x04;
    pub const JOY_RIGHT_MASK: u8 = 0x08;
    pub const JOY_FIRE_MASK: u8 = 0x10;
}

/// PETSCII character codes.
#[allow(dead_code)]
pub mod petscii {
    pub const RETURN: u8 = 0x0D;
    pub const CLEAR_SCREEN: u8 = 0x93;
    pub const HOME: u8 = 0x13;
    pub const CURSOR_DOWN: u8 = 0x11;
    pub const CURSOR_UP: u8 = 0x91;
    pub const CURSOR_RIGHT: u8 = 0x1D;
    pub const CURSOR_LEFT: u8 = 0x9D;
    pub const SPACE: u8 = 0x20;
    pub const DELETE: u8 = 0x14;
}

#[cfg(test)]
mod tests {
    use super::opcodes::*;

    #[test]
    fn test_load_opcodes() {
        assert_eq!(LDA_IMM, 0xA9);
        assert_eq!(LDA_ABS, 0xAD);
        assert_eq!(LDX_IMM, 0xA2);
        assert_eq!(LDY_IMM, 0xA0);
    }

    #[test]
    fn test_store_opcodes() {
        assert_eq!(STA_ABS, 0x8D);
        assert_eq!(STX_ABS, 0x8E);
        assert_eq!(STY_ABS, 0x8C);
    }

    #[test]
    fn test_arithmetic_opcodes() {
        assert_eq!(ADC_IMM, 0x69);
        assert_eq!(SBC_IMM, 0xE9);
        assert_eq!(CLC, 0x18);
        assert_eq!(SEC, 0x38);
    }

    #[test]
    fn test_branch_opcodes() {
        assert_eq!(BEQ, 0xF0);
        assert_eq!(BNE, 0xD0);
        assert_eq!(BCC, 0x90);
        assert_eq!(BCS, 0xB0);
    }

    #[test]
    fn test_jump_opcodes() {
        assert_eq!(JMP_ABS, 0x4C);
        assert_eq!(JSR, 0x20);
        assert_eq!(RTS, 0x60);
    }

    #[test]
    fn test_logical_opcodes() {
        assert_eq!(AND_IMM, 0x29);
        assert_eq!(ORA_IMM, 0x09);
        assert_eq!(EOR_IMM, 0x49);
    }

    #[test]
    fn test_shift_opcodes() {
        assert_eq!(ASL_ACC, 0x0A);
        assert_eq!(LSR_ACC, 0x4A);
        assert_eq!(ROL_ACC, 0x2A);
        assert_eq!(ROR_ACC, 0x6A);
    }
}
