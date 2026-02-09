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

    /// String concatenation buffer (256 bytes at $C200).
    pub const STR_CONCAT_BUFFER: u16 = 0xC200;
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

    /// Additional temporaries ($02-$0D are safe on C64 when not using BASIC floats).
    /// TMP3/TMP3_HI form a 16-bit word for fixed-point operations.
    pub const TMP3: u8 = 0x02;
    pub const TMP3_HI: u8 = 0x03;
    pub const TMP4: u8 = 0x04;
    pub const TMP5: u8 = 0x05;

    /// Extended temporaries for complex operations (using $08-$0D).
    pub const TMP6: u8 = 0x08;
    pub const TMP7: u8 = 0x09;
    pub const TMP8: u8 = 0x0A;
    pub const TMP9: u8 = 0x0B;
    pub const TMP10: u8 = 0x0C;
    pub const TMP11: u8 = 0x0D;

    /// Pointer for string operations ($22-$23).
    pub const STR_PTR: u8 = 0x22;
    pub const STR_PTR_HI: u8 = 0x23;

    /// PRNG state (16-bit seed) - using $06/$07 which are safe on C64.
    pub const PRNG_LO: u8 = 0x06;
    pub const PRNG_HI: u8 = 0x07;
}

/// SID (Sound Interface Device) registers.
///
/// The SID chip provides 3 independent voices with full ADSR envelope control,
/// 4 waveforms (triangle, sawtooth, pulse, noise), and a programmable filter.
/// Memory mapped at $D400-$D41C.
#[allow(dead_code)]
pub mod sid {
    /// SID base address.
    pub const BASE: u16 = 0xD400;

    // =========================================================================
    // Voice 1 Registers ($D400-$D406)
    // =========================================================================

    /// Voice 1 frequency low byte.
    pub const VOICE1_FREQ_LO: u16 = 0xD400;
    /// Voice 1 frequency high byte.
    pub const VOICE1_FREQ_HI: u16 = 0xD401;
    /// Voice 1 pulse wave duty cycle low byte.
    pub const VOICE1_PULSE_LO: u16 = 0xD402;
    /// Voice 1 pulse wave duty cycle high byte (bits 0-3 only).
    pub const VOICE1_PULSE_HI: u16 = 0xD403;
    /// Voice 1 control register (waveform, gate, sync, ring mod, test).
    pub const VOICE1_CTRL: u16 = 0xD404;
    /// Voice 1 attack (high nibble) and decay (low nibble).
    pub const VOICE1_ATTACK_DECAY: u16 = 0xD405;
    /// Voice 1 sustain (high nibble) and release (low nibble).
    pub const VOICE1_SUSTAIN_RELEASE: u16 = 0xD406;

    // =========================================================================
    // Voice 2 Registers ($D407-$D40D)
    // =========================================================================

    /// Voice 2 frequency low byte.
    pub const VOICE2_FREQ_LO: u16 = 0xD407;
    /// Voice 2 frequency high byte.
    pub const VOICE2_FREQ_HI: u16 = 0xD408;
    /// Voice 2 pulse wave duty cycle low byte.
    pub const VOICE2_PULSE_LO: u16 = 0xD409;
    /// Voice 2 pulse wave duty cycle high byte (bits 0-3 only).
    pub const VOICE2_PULSE_HI: u16 = 0xD40A;
    /// Voice 2 control register (waveform, gate, sync, ring mod, test).
    pub const VOICE2_CTRL: u16 = 0xD40B;
    /// Voice 2 attack (high nibble) and decay (low nibble).
    pub const VOICE2_ATTACK_DECAY: u16 = 0xD40C;
    /// Voice 2 sustain (high nibble) and release (low nibble).
    pub const VOICE2_SUSTAIN_RELEASE: u16 = 0xD40D;

    // =========================================================================
    // Voice 3 Registers ($D40E-$D414)
    // =========================================================================

    /// Voice 3 frequency low byte.
    pub const VOICE3_FREQ_LO: u16 = 0xD40E;
    /// Voice 3 frequency high byte.
    pub const VOICE3_FREQ_HI: u16 = 0xD40F;
    /// Voice 3 pulse wave duty cycle low byte.
    pub const VOICE3_PULSE_LO: u16 = 0xD410;
    /// Voice 3 pulse wave duty cycle high byte (bits 0-3 only).
    pub const VOICE3_PULSE_HI: u16 = 0xD411;
    /// Voice 3 control register (waveform, gate, sync, ring mod, test).
    pub const VOICE3_CTRL: u16 = 0xD412;
    /// Voice 3 attack (high nibble) and decay (low nibble).
    pub const VOICE3_ATTACK_DECAY: u16 = 0xD413;
    /// Voice 3 sustain (high nibble) and release (low nibble).
    pub const VOICE3_SUSTAIN_RELEASE: u16 = 0xD414;

    // =========================================================================
    // Filter and Volume Registers ($D415-$D418)
    // =========================================================================

    /// Filter cutoff frequency low byte (bits 0-2 only).
    pub const FILTER_CUTOFF_LO: u16 = 0xD415;
    /// Filter cutoff frequency high byte.
    pub const FILTER_CUTOFF_HI: u16 = 0xD416;
    /// Filter resonance (high nibble) and voice routing (low nibble).
    pub const FILTER_RESONANCE: u16 = 0xD417;
    /// Filter mode (high nibble) and main volume (low nibble).
    pub const FILTER_MODE_VOLUME: u16 = 0xD418;

    // =========================================================================
    // Read-Only Registers ($D419-$D41C)
    // =========================================================================

    /// Potentiometer X (paddle) read.
    pub const POT_X: u16 = 0xD419;
    /// Potentiometer Y (paddle) read.
    pub const POT_Y: u16 = 0xD41A;
    /// Voice 3 oscillator output (random when noise waveform).
    pub const VOICE3_OSC: u16 = 0xD41B;
    /// Voice 3 envelope output.
    pub const VOICE3_ENV: u16 = 0xD41C;

    // =========================================================================
    // Waveform Bit Constants (for control register)
    // =========================================================================

    /// Triangle waveform (bit 4).
    pub const WAVEFORM_TRIANGLE: u8 = 0x10;
    /// Sawtooth waveform (bit 5).
    pub const WAVEFORM_SAWTOOTH: u8 = 0x20;
    /// Pulse/square waveform (bit 6).
    pub const WAVEFORM_PULSE: u8 = 0x40;
    /// Noise waveform (bit 7).
    pub const WAVEFORM_NOISE: u8 = 0x80;

    // =========================================================================
    // Control Register Bit Constants
    // =========================================================================

    /// Gate bit - start/release note (bit 0).
    pub const CTRL_GATE: u8 = 0x01;
    /// Sync bit - synchronize with previous voice (bit 1).
    pub const CTRL_SYNC: u8 = 0x02;
    /// Ring modulation bit - modulate with previous voice (bit 2).
    pub const CTRL_RING_MOD: u8 = 0x04;
    /// Test bit - disable oscillator (bit 3).
    pub const CTRL_TEST: u8 = 0x08;

    // =========================================================================
    // Filter Mode Bit Constants (for $D418 high nibble)
    // =========================================================================

    /// Low-pass filter mode (bit 4).
    pub const FILTER_LP: u8 = 0x10;
    /// Band-pass filter mode (bit 5).
    pub const FILTER_BP: u8 = 0x20;
    /// High-pass filter mode (bit 6).
    pub const FILTER_HP: u8 = 0x40;
    /// Disconnect voice 3 from audio output (bit 7).
    pub const FILTER_VOICE3_OFF: u8 = 0x80;

    // =========================================================================
    // Filter Routing Bit Constants (for $D417 low nibble)
    // =========================================================================

    /// Route voice 1 through filter (bit 0).
    pub const FILTER_VOICE1: u8 = 0x01;
    /// Route voice 2 through filter (bit 1).
    pub const FILTER_VOICE2: u8 = 0x02;
    /// Route voice 3 through filter (bit 2).
    pub const FILTER_VOICE3: u8 = 0x04;
    /// Route external audio input through filter (bit 3).
    pub const FILTER_EXT: u8 = 0x08;

    // =========================================================================
    // Voice Register Offsets (for calculating addresses)
    // =========================================================================

    /// Offset between voice register blocks (7 bytes per voice).
    pub const VOICE_OFFSET: u16 = 7;

    /// Offset from voice base to frequency low byte.
    pub const OFFSET_FREQ_LO: u16 = 0;
    /// Offset from voice base to frequency high byte.
    pub const OFFSET_FREQ_HI: u16 = 1;
    /// Offset from voice base to pulse width low byte.
    pub const OFFSET_PULSE_LO: u16 = 2;
    /// Offset from voice base to pulse width high byte.
    pub const OFFSET_PULSE_HI: u16 = 3;
    /// Offset from voice base to control register.
    pub const OFFSET_CTRL: u16 = 4;
    /// Offset from voice base to attack/decay register.
    pub const OFFSET_ATTACK_DECAY: u16 = 5;
    /// Offset from voice base to sustain/release register.
    pub const OFFSET_SUSTAIN_RELEASE: u16 = 6;

    // =========================================================================
    // SID Register Count
    // =========================================================================

    /// Total number of SID registers (for reset loop).
    pub const REGISTER_COUNT: u8 = 25;
}

/// VIC-II registers.
#[allow(dead_code)]
pub mod vic {
    // =========================================================================
    // VIC-II Base Address
    // =========================================================================

    /// VIC-II base address.
    pub const BASE: u16 = 0xD000;

    // =========================================================================
    // Control Registers
    // =========================================================================

    /// Control register 1 ($D011).
    /// Bits: RST8 | ECM | BMM | DEN | RSEL | YSCROLL(2-0)
    /// - Bit 7: RST8 - Raster line bit 8
    /// - Bit 6: ECM - Extended color mode
    /// - Bit 5: BMM - Bitmap mode
    /// - Bit 4: DEN - Display enable
    /// - Bit 3: RSEL - Row select (24/25 rows)
    /// - Bits 0-2: YSCROLL - Vertical scroll (0-7)
    pub const CONTROL1: u16 = 0xD011;

    /// Control register 2 ($D016).
    /// Bits: - | - | RES | MCM | CSEL | XSCROLL(2-0)
    /// - Bit 5: RES - Reset (unused)
    /// - Bit 4: MCM - Multicolor mode
    /// - Bit 3: CSEL - Column select (38/40 columns)
    /// - Bits 0-2: XSCROLL - Horizontal scroll (0-7)
    pub const CONTROL2: u16 = 0xD016;

    /// Memory control register ($D018).
    /// Bits: VM13-VM10 | CB13-CB11 | -
    /// - Bits 4-7: VM - Video matrix base address
    /// - Bits 1-3: CB - Character/bitmap base address
    pub const MEMORY: u16 = 0xD018;

    // =========================================================================
    // Raster Registers
    // =========================================================================

    /// Current raster line (bits 0-7).
    pub const RASTER: u16 = 0xD012;

    // =========================================================================
    // Color Registers
    // =========================================================================

    /// Border color register.
    pub const BORDER: u16 = 0xD020;

    /// Background color 0 (main background).
    pub const BACKGROUND0: u16 = 0xD021;

    /// Background color 1 (ECM/multicolor).
    pub const BACKGROUND1: u16 = 0xD022;

    /// Background color 2 (ECM/multicolor).
    pub const BACKGROUND2: u16 = 0xD023;

    /// Background color 3 (ECM only).
    pub const BACKGROUND3: u16 = 0xD024;

    // =========================================================================
    // Control Bit Constants
    // =========================================================================

    /// Bitmap mode bit (bit 5 of $D011).
    pub const BMM: u8 = 0x20;

    /// Extended color mode bit (bit 6 of $D011).
    pub const ECM: u8 = 0x40;

    /// Multicolor mode bit (bit 4 of $D016).
    pub const MCM: u8 = 0x10;

    /// Display enable bit (bit 4 of $D011).
    pub const DEN: u8 = 0x10;

    /// Row select bit - 25 rows (bit 3 of $D011).
    pub const RSEL: u8 = 0x08;

    /// Column select bit - 40 columns (bit 3 of $D016).
    pub const CSEL: u8 = 0x08;

    /// Y scroll mask (bits 0-2 of $D011).
    pub const YSCROLL_MASK: u8 = 0x07;

    /// X scroll mask (bits 0-2 of $D016).
    pub const XSCROLL_MASK: u8 = 0x07;

    // =========================================================================
    // Graphics Mode Constants (for gfx_mode function)
    // =========================================================================

    /// Standard text mode (ECM=0, BMM=0, MCM=0).
    pub const MODE_TEXT: u8 = 0;

    /// Multicolor text mode (ECM=0, BMM=0, MCM=1).
    pub const MODE_TEXT_MC: u8 = 1;

    /// Standard bitmap/hires mode (ECM=0, BMM=1, MCM=0).
    pub const MODE_BITMAP: u8 = 2;

    /// Multicolor bitmap mode (ECM=0, BMM=1, MCM=1).
    pub const MODE_BITMAP_MC: u8 = 3;

    /// Extended background color mode (ECM=1, BMM=0, MCM=0).
    pub const MODE_TEXT_ECM: u8 = 4;

    // =========================================================================
    // Memory Bank Constants (via CIA2 $DD00)
    // =========================================================================

    /// VIC bank 0: $0000-$3FFF (default).
    pub const BANK0: u8 = 0;

    /// VIC bank 1: $4000-$7FFF.
    pub const BANK1: u8 = 1;

    /// VIC bank 2: $8000-$BFFF.
    pub const BANK2: u8 = 2;

    /// VIC bank 3: $C000-$FFFF.
    pub const BANK3: u8 = 3;

    // =========================================================================
    // Default Values
    // =========================================================================

    /// Default value for $D011 (text mode, 25 rows, display enabled).
    pub const CONTROL1_DEFAULT: u8 = 0x1B;

    /// Default value for $D016 (no multicolor, 40 columns).
    pub const CONTROL2_DEFAULT: u8 = 0xC8;

    // =========================================================================
    // Raster Constants
    // =========================================================================

    /// First visible raster line (PAL).
    pub const RASTER_TOP: u16 = 50;

    /// Last visible raster line (PAL).
    pub const RASTER_BOTTOM: u16 = 250;

    /// Maximum raster line (PAL).
    pub const RASTER_MAX_PAL: u16 = 311;

    /// Maximum raster line (NTSC).
    pub const RASTER_MAX_NTSC: u16 = 261;

    // =========================================================================
    // Bitmap Memory Constants
    // =========================================================================

    /// Default bitmap address ($2000 = 8192).
    pub const BITMAP_DEFAULT: u16 = 0x2000;

    /// Bitmap size in bytes (320x200 / 8 = 8000 bytes).
    pub const BITMAP_SIZE: u16 = 8000;

    /// Screen width in pixels (hires mode).
    pub const SCREEN_WIDTH: u16 = 320;

    /// Screen height in pixels.
    pub const SCREEN_HEIGHT: u8 = 200;

    /// Screen width in pixels (multicolor mode).
    pub const SCREEN_WIDTH_MC: u8 = 160;
}

/// VIC-II sprite registers.
#[allow(dead_code)]
pub mod sprite {
    // Sprite position registers (X and Y pairs for sprites 0-7)
    /// Sprite 0 X position (low 8 bits).
    pub const SPRITE0_X: u16 = 0xD000;
    /// Sprite 0 Y position.
    pub const SPRITE0_Y: u16 = 0xD001;
    /// Sprite 1 X position (low 8 bits).
    pub const SPRITE1_X: u16 = 0xD002;
    /// Sprite 1 Y position.
    pub const SPRITE1_Y: u16 = 0xD003;
    /// Sprite 2 X position (low 8 bits).
    pub const SPRITE2_X: u16 = 0xD004;
    /// Sprite 2 Y position.
    pub const SPRITE2_Y: u16 = 0xD005;
    /// Sprite 3 X position (low 8 bits).
    pub const SPRITE3_X: u16 = 0xD006;
    /// Sprite 3 Y position.
    pub const SPRITE3_Y: u16 = 0xD007;
    /// Sprite 4 X position (low 8 bits).
    pub const SPRITE4_X: u16 = 0xD008;
    /// Sprite 4 Y position.
    pub const SPRITE4_Y: u16 = 0xD009;
    /// Sprite 5 X position (low 8 bits).
    pub const SPRITE5_X: u16 = 0xD00A;
    /// Sprite 5 Y position.
    pub const SPRITE5_Y: u16 = 0xD00B;
    /// Sprite 6 X position (low 8 bits).
    pub const SPRITE6_X: u16 = 0xD00C;
    /// Sprite 6 Y position.
    pub const SPRITE6_Y: u16 = 0xD00D;
    /// Sprite 7 X position (low 8 bits).
    pub const SPRITE7_X: u16 = 0xD00E;
    /// Sprite 7 Y position.
    pub const SPRITE7_Y: u16 = 0xD00F;

    /// Sprite X position MSB (bit 8 of X for each sprite, bits 0-7 = sprites 0-7).
    pub const X_MSB: u16 = 0xD010;

    /// Sprite enable register (bits 0-7 enable sprites 0-7).
    pub const ENABLE: u16 = 0xD015;

    /// Sprite Y expansion (bits 0-7 = double height for sprites 0-7).
    pub const EXPAND_Y: u16 = 0xD017;

    /// Sprite priority (bits 0-7: 1 = sprite behind background).
    pub const PRIORITY: u16 = 0xD01B;

    /// Sprite multicolor mode (bits 0-7: 1 = multicolor for sprite).
    pub const MULTICOLOR: u16 = 0xD01C;

    /// Sprite X expansion (bits 0-7 = double width for sprites 0-7).
    pub const EXPAND_X: u16 = 0xD01D;

    /// Sprite-sprite collision register (read clears, bits show collided sprites).
    pub const COLLISION_SPRITE: u16 = 0xD01E;

    /// Sprite-background collision register (read clears, bits show collided sprites).
    pub const COLLISION_BG: u16 = 0xD01F;

    /// Sprite multicolor shared color 1.
    pub const MULTICOLOR1: u16 = 0xD025;

    /// Sprite multicolor shared color 2.
    pub const MULTICOLOR2: u16 = 0xD026;

    // Individual sprite color registers
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

    /// Base address for sprite pointers (default screen at $0400).
    /// Sprite pointers are at screen + $3F8 to screen + $3FF.
    pub const POINTER_BASE: u16 = 0x07F8;

    /// Sprite data size (64 bytes per sprite shape).
    pub const DATA_SIZE: u8 = 64;
}

/// C64 color constants (VIC-II palette).
#[allow(dead_code)]
pub mod colors {
    pub const BLACK: u8 = 0;
    pub const WHITE: u8 = 1;
    pub const RED: u8 = 2;
    pub const CYAN: u8 = 3;
    pub const PURPLE: u8 = 4;
    pub const GREEN: u8 = 5;
    pub const BLUE: u8 = 6;
    pub const YELLOW: u8 = 7;
    pub const ORANGE: u8 = 8;
    pub const BROWN: u8 = 9;
    pub const LIGHT_RED: u8 = 10;
    pub const DARK_GRAY: u8 = 11;
    pub const GRAY: u8 = 12;
    pub const LIGHT_GREEN: u8 = 13;
    pub const LIGHT_BLUE: u8 = 14;
    pub const LIGHT_GRAY: u8 = 15;
}

/// CIA (Complex Interface Adapter) registers.
#[allow(dead_code)]
pub mod cia {
    /// CIA1 Timer A low byte (free-running counter).
    pub const CIA1_TIMER_A_LO: u16 = 0xDC04;
    /// CIA1 Timer A high byte.
    pub const CIA1_TIMER_A_HI: u16 = 0xDC05;
    /// CIA1 Timer B low byte.
    pub const CIA1_TIMER_B_LO: u16 = 0xDC06;
    /// CIA1 Timer B high byte.
    pub const CIA1_TIMER_B_HI: u16 = 0xDC07;

    /// CIA1 Port A - Joystick Port 2 and keyboard matrix columns.
    /// Reading this register returns joystick 2 state in bits 0-4:
    /// - Bit 0: Up (active low, 0 = pressed)
    /// - Bit 1: Down (active low)
    /// - Bit 2: Left (active low)
    /// - Bit 3: Right (active low)
    /// - Bit 4: Fire button (active low)
    pub const CIA1_PORTA: u16 = 0xDC00;

    /// CIA1 Port B - Joystick Port 1 and keyboard matrix rows.
    /// Reading this register returns joystick 1 state in bits 0-4:
    /// - Bit 0: Up (active low, 0 = pressed)
    /// - Bit 1: Down (active low)
    /// - Bit 2: Left (active low)
    /// - Bit 3: Right (active low)
    /// - Bit 4: Fire button (active low)
    pub const CIA1_PORTB: u16 = 0xDC01;

    /// Joystick bit mask for direction and fire bits (bits 0-4).
    pub const JOY_MASK: u8 = 0x1F;

    /// CIA2 Port A Data Direction Register.
    pub const CIA2_DDRA: u16 = 0xDD02;
    /// CIA2 Port A - VIC bank selection (bits 0-1).
    /// Bits 0-1 select VIC bank (inverted):
    /// - %11 = Bank 0 ($0000-$3FFF)
    /// - %10 = Bank 1 ($4000-$7FFF)
    /// - %01 = Bank 2 ($8000-$BFFF)
    /// - %00 = Bank 3 ($C000-$FFFF)
    pub const CIA2_PRA: u16 = 0xDD00;

    /// VIC bank mask (bits 0-1 of CIA2 Port A).
    pub const VIC_BANK_MASK: u8 = 0x03;
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
