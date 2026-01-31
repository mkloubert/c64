# C64 Hardware Architecture

This document describes the hardware architecture of the Commodore 64 home computer.

## CPU: MOS 6510

The C64 uses the **MOS Technology 6510** microprocessor, which is almost identical to the 6502.

### Key Specifications

- **Architecture**: 8-bit
- **Clock Speed**:
  - NTSC: 1.023 MHz
  - PAL: 0.985 MHz
- **Address Bus**: 16-bit (can address 64KB)
- **Data Bus**: 8-bit

### Differences from 6502

The 6510 has a built-in 8-bit I/O port at addresses `$00` and `$01`:

- **$00**: Data Direction Register (DDR)
- **$01**: Port Register - Controls memory banking

The bottom 3 bits of `$01` control memory layout:

- Bit 0 (LORAM): BASIC ROM at `$A000-$BFFF`
- Bit 1 (HIRAM): KERNAL ROM at `$E000-$FFFF`
- Bit 2 (CHAREN): Character ROM / I/O visibility

Setting bits 0 and 1 to zero gives a full 64KB RAM machine.

### Instruction Set

The 6510 has **56 official instructions** with **151 valid opcodes**. Key categories:

| Category     | Instructions                           |
| ------------ | -------------------------------------- |
| Load/Store   | LDA, LDX, LDY, STA, STX, STY           |
| Arithmetic   | ADC, SBC, INC, DEC                     |
| Logic        | AND, ORA, EOR                          |
| Shift/Rotate | ASL, LSR, ROL, ROR                     |
| Branch       | BEQ, BNE, BCC, BCS, BMI, BPL, BVC, BVS |
| Jump         | JMP, JSR, RTS, RTI                     |
| Stack        | PHA, PLA, PHP, PLP                     |
| Flags        | CLC, SEC, CLI, SEI, CLV, CLD, SED      |
| Compare      | CMP, CPX, CPY                          |
| Transfer     | TAX, TXA, TAY, TYA, TSX, TXS           |
| Other        | NOP, BRK, BIT                          |

### Addressing Modes

| Mode         | Example       | Description                  |
| ------------ | ------------- | ---------------------------- |
| Immediate    | `LDA #$20`    | Operand is the value         |
| Zero Page    | `LDA $20`     | 8-bit address in page zero   |
| Zero Page,X  | `LDA $20,X`   | Zero page + X register       |
| Zero Page,Y  | `LDX $20,Y`   | Zero page + Y register       |
| Absolute     | `LDA $2000`   | 16-bit address               |
| Absolute,X   | `LDA $2000,X` | Absolute + X register        |
| Absolute,Y   | `LDA $2000,Y` | Absolute + Y register        |
| Indirect     | `JMP ($2000)` | Address at memory location   |
| (Indirect,X) | `LDA ($20,X)` | Indexed indirect             |
| (Indirect),Y | `LDA ($20),Y` | Indirect indexed             |
| Relative     | `BEQ label`   | Branch offset (-128 to +127) |
| Implied      | `TAX`         | No operand                   |
| Accumulator  | `ASL A`       | Operates on accumulator      |

---

## Memory Map

### Overview

```
$FFFF +------------------+
      |   KERNAL ROM     | 8KB
$E000 +------------------+
      |   I/O or RAM     | 4KB
$D000 +------------------+
      | Char ROM or RAM  | 4KB
$C000 +------------------+
      |   BASIC ROM      | 8KB
$A000 +------------------+
      |                  |
      |      RAM         | 38KB (free for programs)
      |                  |
$0800 +------------------+
      | Screen RAM etc.  | 2KB
$0000 +------------------+
```

### Important Memory Locations

| Address       | Description                  |
| ------------- | ---------------------------- |
| `$0000`       | 6510 DDR                     |
| `$0001`       | 6510 Port (bank switching)   |
| `$0002-$00FF` | Zero Page (fast access)      |
| `$0100-$01FF` | Stack                        |
| `$0200-$03FF` | System variables             |
| `$0400-$07FF` | Default Screen RAM           |
| `$0800-$9FFF` | BASIC program area           |
| `$A000-$BFFF` | BASIC ROM / RAM              |
| `$C000-$CFFF` | Free RAM                     |
| `$D000-$D3FF` | VIC-II registers             |
| `$D400-$D7FF` | SID registers                |
| `$D800-$DBFF` | Color RAM                    |
| `$DC00-$DCFF` | CIA 1 (keyboard, joystick)   |
| `$DD00-$DDFF` | CIA 2 (serial bus, VIC bank) |
| `$E000-$FFFF` | KERNAL ROM / RAM             |

---

## VIC-II Video Chip

The **MOS 6567 (NTSC) / 6569 (PAL)** generates all video output.

### Specifications

- **Resolution**: 320x200 (hi-res) or 160x200 (multicolor)
- **Colors**: 16 fixed palette
- **Sprites**: 8 hardware sprites (24x21 pixels)
- **Address Space**: 16KB for video data
- **Registers**: Located at `$D000-$D3FF`

### Graphics Modes

| Mode                | Resolution  | Colors | Description                            |
| ------------------- | ----------- | ------ | -------------------------------------- |
| Standard Text       | 40x25 chars | 16     | One color per character                |
| Multicolor Text     | 40x25 chars | 4      | 4 colors, half horizontal resolution   |
| Hi-Res Bitmap       | 320x200     | 2      | One foreground/background per 8x8 cell |
| Multicolor Bitmap   | 160x200     | 4      | 4 colors per 4x8 cell                  |
| Extended Background | 40x25 chars | 4      | 4 background colors, 64 characters     |

### Important VIC-II Registers

| Address       | Register      | Description                 |
| ------------- | ------------- | --------------------------- |
| `$D000-$D00F` | Sprite X/Y    | Sprite positions            |
| `$D010`       | Sprite X MSB  | High bits of X positions    |
| `$D011`       | Control 1     | Screen control, Y scroll    |
| `$D012`       | Raster        | Current/trigger raster line |
| `$D016`       | Control 2     | Multicolor mode, X scroll   |
| `$D018`       | Memory        | Screen/char memory pointers |
| `$D019`       | IRQ flags     | Interrupt status            |
| `$D01A`       | IRQ enable    | Interrupt enable mask       |
| `$D020`       | Border        | Border color                |
| `$D021`       | Background    | Background color 0          |
| `$D025-$D026` | Sprite MC     | Sprite multicolors          |
| `$D027-$D02E` | Sprite colors | Individual sprite colors    |

### Color Palette

| Value | Color       |
| ----- | ----------- |
| 0     | Black       |
| 1     | White       |
| 2     | Red         |
| 3     | Cyan        |
| 4     | Purple      |
| 5     | Green       |
| 6     | Blue        |
| 7     | Yellow      |
| 8     | Orange      |
| 9     | Brown       |
| 10    | Light Red   |
| 11    | Dark Grey   |
| 12    | Grey        |
| 13    | Light Green |
| 14    | Light Blue  |
| 15    | Light Grey  |

---

## SID Sound Chip

The **MOS 6581 / 8580** is a programmable synthesizer chip.

### Specifications

- **Voices**: 3 independent voices
- **Waveforms**: Triangle, Sawtooth, Pulse (variable width), Noise
- **Frequency Range**: 0-4000 Hz (16-bit control)
- **ADSR**: Attack, Decay, Sustain, Release per voice
- **Filter**: 12dB multimode (lowpass, highpass, bandpass)
- **Volume**: 4-bit master (0-15)
- **Registers**: Located at `$D400-$D418`

### Register Map

| Offset    | Voice | Description                       |
| --------- | ----- | --------------------------------- |
| `$00-$01` | 1     | Frequency (16-bit, little-endian) |
| `$02-$03` | 1     | Pulse Width                       |
| `$04`     | 1     | Control (waveform, gate)          |
| `$05`     | 1     | Attack/Decay                      |
| `$06`     | 1     | Sustain/Release                   |
| `$07-$0D` | 2     | Same as voice 1                   |
| `$0E-$14` | 3     | Same as voice 1                   |
| `$15-$16` | -     | Filter cutoff                     |
| `$17`     | -     | Filter resonance/routing          |
| `$18`     | -     | Volume/filter mode                |

### Control Register Bits (offset $04, $0B, $12)

| Bit | Description               |
| --- | ------------------------- |
| 0   | Gate (1=start, 0=release) |
| 1   | Sync                      |
| 2   | Ring Modulation           |
| 3   | Test                      |
| 4   | Triangle                  |
| 5   | Sawtooth                  |
| 6   | Pulse                     |
| 7   | Noise                     |

---

## KERNAL ROM

The KERNAL provides system routines accessible via a jump table at `$FF81-$FFF3`.

### Common KERNAL Routines

| Address | Name   | Description                 |
| ------- | ------ | --------------------------- |
| `$FF81` | CINT   | Initialize screen editor    |
| `$FF84` | IOINIT | Initialize I/O devices      |
| `$FF87` | RAMTAS | Initialize RAM, tape buffer |
| `$FFD2` | CHROUT | Output character            |
| `$FFCF` | CHRIN  | Input character             |
| `$FFE4` | GETIN  | Get character from keyboard |
| `$FFE7` | CLALL  | Close all files             |
| `$FFBA` | SETLFS | Set file parameters         |
| `$FFBD` | SETNAM | Set filename                |
| `$FFD5` | LOAD   | Load from device            |
| `$FFD8` | SAVE   | Save to device              |

---

## Sources

- [MOS Technology 6510 - Wikipedia](https://en.wikipedia.org/wiki/MOS_Technology_6510)
- [6502 Instruction Set - Masswerk](https://www.masswerk.at/6502/6502_instruction_set.html)
- [C64 Memory Map - sta.c64.org](https://sta.c64.org/cbm64mem.html)
- [Memory Map - Ultimate C64 Reference](https://www.pagetable.com/c64ref/c64mem/)
- [VIC-II - C64-Wiki](https://www.c64-wiki.com/wiki/VIC)
- [SID - C64-Wiki](https://www.c64-wiki.com/wiki/SID)
- [KERNAL API - Ultimate C64 Reference](https://www.pagetable.com/c64ref/kernal/)
