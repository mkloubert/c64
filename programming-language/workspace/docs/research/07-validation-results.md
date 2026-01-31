# Validation Results

This document records the results of Phase 3: Practical validation of PRG and D64 file generation.

## Test Files Created

| File          | Size          | Description                                  |
| ------------- | ------------- | -------------------------------------------- |
| `minimal.prg` | 11 bytes      | Minimal PRG, sets border/background to black |
| `hello.prg`   | 43 bytes      | PRG with BASIC stub, cycles border colors    |
| `test.d64`    | 174,848 bytes | D64 disk image containing hello.prg          |

## PRG File Validation

### minimal.prg

```
Offset  Hex                                               ASCII
00000000  00 c0 a9 00 8d 20 d0 8d 21 d0 60                  ..... ..!.`
```

**Structure Analysis:**

- `00 c0` - Load address: $C000 (little-endian)
- `a9 00` - LDA #$00 (load black color)
- `8d 20 d0` - STA $D020 (store to border color)
- `8d 21 d0` - STA $D021 (store to background color)
- `60` - RTS (return)

**Verification:** Correct PRG structure with valid 6510 machine code.

### hello.prg

```
Offset  Hex                                               ASCII
00000000  01 08 0c 08 0a 00 9e 32 30 36 34 00 00 00 00 00   .......2064.....
00000010  00 a9 00 8d 20 d0 8d 21 d0 a2 00 a0 00 88 d0 fd   .... ..!........
00000020  ca d0 fa 18 69 01 29 0f 4c 12 08                  ....i.).L..
```

**Structure Analysis:**

- `01 08` - Load address: $0801 (BASIC program area)
- `0c 08` - Next line pointer: $080C
- `0a 00` - Line number: 10
- `9e` - SYS token
- `32 30 36 34` - "2064" (ASCII)
- `00` - End of line
- `00 00` - End of BASIC program
- `00 00 00` - Padding to reach $0810
- Machine code at $0810 (offset $0F in file)

**Machine Code Disassembly:**

```asm
$0810: A9 00     LDA #$00        ; Start color = black
$0812: 8D 20 D0  STA $D020       ; Set border
$0815: 8D 21 D0  STA $D021       ; Set background
$0818: A2 00     LDX #$00        ; Outer delay counter
$081A: A0 00     LDY #$00        ; Inner delay counter
$081C: 88        DEY             ; Delay loop
$081D: D0 FD     BNE $081C       ; Branch if Y != 0
$081F: CA        DEX
$0820: D0 FA     BNE $081C       ; Branch if X != 0
$0822: 18        CLC
$0823: 69 01     ADC #$01        ; Next color
$0825: 29 0F     AND #$0F        ; Keep in range 0-15
$0827: 4C 12 08  JMP $0812       ; Loop forever
```

**Verification:** Correct BASIC stub + valid 6510 machine code loop.

## D64 File Validation

### BAM (Track 18, Sector 0)

```
Offset    Hex                                               ASCII
00016500  12 01 41 00 14 fe ff 1f 15 ff ff 1f ...           ..A.............
...
00016590  48 45 4c 4c 4f 20 44 49 53 4b a0 a0 a0 a0 a0 a0   HELLO DISK......
000165a0  a0 a0 43 36 a0 32 41 a0 a0 a0 a0 ...              ..C6.2A.........
```

**Structure Analysis:**

- `12 01` - Directory at Track 18, Sector 1
- `41` - DOS version 'A'
- BAM entries for 35 tracks (4 bytes each)
- Disk name: "HELLO DISK" at offset $90
- Disk ID: "C6" at offset $A2
- DOS type: "2A" at offset $A5

**BAM Entry for Track 1:**

- `14 fe ff 1f` = 20 free sectors (one used), bitmap shows sector 0 used

### Directory (Track 18, Sector 1)

```
Offset    Hex                                               ASCII
00016600  00 ff 82 01 00 48 45 4c 4c 4f a0 a0 ...           .....HELLO......
```

**Directory Entry Analysis:**

- `00 ff` - No next directory sector
- `82` - File type: PRG + properly closed
- `01 00` - First data block: Track 1, Sector 0
- `HELLO` - Filename (padded with $A0)
- `01 00` - File size: 1 sector

### File Data (Track 1, Sector 0)

```
Offset    Hex                                               ASCII
00000000  00 2c 01 08 0c 08 0a 00 9e 32 30 36 34 ...        .,.......2064...
```

**Sector Chain Analysis:**

- `00` - Track 0 = last sector in chain
- `2c` - 44 bytes used ($2C = 44, includes 2-byte header overhead, actual data = 43 bytes)
- Remaining bytes: PRG file content

**Verification:** Valid D64 structure with correct BAM, directory, and file chain.

## File Size Verification

| Component   | Expected      | Actual        | Status |
| ----------- | ------------- | ------------- | ------ |
| PRG header  | 2 bytes       | 2 bytes       | OK     |
| D64 total   | 174,848 bytes | 174,848 bytes | OK     |
| D64 sectors | 683           | 683           | OK     |
| Sector size | 256 bytes     | 256 bytes     | OK     |

## Validation Summary

| Test                       | Result |
| -------------------------- | ------ |
| PRG load address encoding  | PASS   |
| PRG machine code validity  | PASS   |
| BASIC stub format          | PASS   |
| D64 file size              | PASS   |
| D64 BAM structure          | PASS   |
| D64 directory structure    | PASS   |
| D64 file chain             | PASS   |
| Sector allocation tracking | PASS   |

## Emulator Testing Instructions

Since no graphical environment is available, test externally:

### Using VICE (x64sc)

```bash
# Test PRG directly
x64sc -autostart hello.prg

# Test D64
x64sc test.d64
# Then in C64: LOAD "*",8,1 and RUN
```

### Using VICE Headless (if available)

```bash
# Run with virtual screen output
x64sc -console -sounddev dummy hello.prg
```

### Online Emulators

Upload files to:

- https://c64online.com/
- https://virtualconsoles.com/online-emulators/c64/

## Conclusion

All file formats are correctly implemented:

1. **PRG Generation** - Trivial (2-byte header + code)
2. **D64 Generation** - Achievable (fixed structure, well-documented)
3. **6510 Machine Code** - Correctly encoded

The feasibility analysis is **validated**. A Rust compiler can generate these formats.
