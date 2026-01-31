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
