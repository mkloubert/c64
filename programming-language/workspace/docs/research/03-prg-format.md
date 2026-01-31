# PRG File Format

This document describes the PRG (Program) file format used by Commodore 64 programs.

## Overview

PRG is the native executable format for Commodore 64 programs. It is extremely simple.

### Key Properties

- **File Extension**: `.prg`
- **Header Size**: 2 bytes
- **Content**: Raw binary data
- **No metadata**: Just address + code/data

---

## File Structure

```
+--------+--------+------------------+
| Byte 0 | Byte 1 | Bytes 2 to N     |
+--------+--------+------------------+
| Load address    | Program data     |
| (little-endian) |                  |
+--------+--------+------------------+
```

### Load Address

The first two bytes specify where in memory the program should be loaded:

- **Byte 0**: Low byte of address
- **Byte 1**: High byte of address

Example: `$01 $08` means load address `$0801` (2049 decimal)

### Program Data

Everything after the first two bytes is copied directly into C64 memory, starting at the load address.

---

## Common Load Addresses

| Address | Hex     | Purpose                      |
| ------- | ------- | ---------------------------- |
| 2049    | `$0801` | BASIC program area (default) |
| 4096    | `$1000` | Common for ML programs       |
| 8192    | `$2000` | After graphics memory        |
| 32768   | `$8000` | Upper RAM                    |
| 49152   | `$C000` | Free RAM area                |

### BASIC Program Area ($0801)

Most programs use `$0801` because:

- It is where BASIC programs normally load
- The BASIC `RUN` command works automatically
- System variables expect programs here

---

## Loading Methods

### From BASIC

```basic
LOAD "PROGRAM",8      ; Loads to $0801 regardless of file header
LOAD "PROGRAM",8,1    ; Loads to address specified in PRG header
```

The `,1` parameter tells the KERNAL to use the embedded load address.

### From Machine Language

Using KERNAL routines:

1. `SETLFS` ($FFBA) - Set file parameters
2. `SETNAM` ($FFBD) - Set filename
3. `LOAD` ($FFD5) - Perform load

---

## Creating PRG Files

### For Assembly Programs

```asm
; Example: Program at $C000
* = $C000

start:
    lda #$00
    sta $d020    ; Set border to black
    rts
```

When assembled, output PRG file:

```
00 C0        ; Load address $C000
A9 00        ; LDA #$00
8D 20 D0     ; STA $D020
60           ; RTS
```

### For BASIC-Launchable Programs

To create a program that can be started with `RUN`:

```
; PRG file structure for autostart
$0801: Load address bytes ($01 $08)
$0801: BASIC stub to call machine code
$0810: Actual machine code
```

Example BASIC stub:

```asm
* = $0801

; BASIC line: 10 SYS 2064
.byte $0C, $08      ; Pointer to next line ($080C)
.byte $0A, $00      ; Line number 10
.byte $9E           ; SYS token
.byte " 2064"       ; Address as ASCII
.byte $00           ; End of line
.byte $00, $00      ; End of program (null pointer)

; Machine code starts at $0810 = 2064
* = $0810
    ; Your code here
```

### Binary Structure

```
Offset  Content
------  -------
$00     $01         ; Low byte of load address
$01     $08         ; High byte ($0801)
$02     $0C         ; Next line pointer low
$03     $08         ; Next line pointer high
$04     $0A         ; Line number low (10)
$05     $00         ; Line number high
$06     $9E         ; SYS token
$07-$0B " 2064"     ; Target address
$0C     $00         ; End of BASIC line
$0D     $00         ; End of program (null pointer low)
$0E     $00         ; End of program (null pointer high)
$0F     ...         ; Machine code starts here
```

---

## Memory After Loading

When a PRG is loaded with `LOAD "FILE",8,1`:

```
Memory:
$0800: (System area)
$0801: First byte of PRG data
$0802: Second byte of PRG data
...
$0801+N: Last byte of PRG data
```

The load address bytes are NOT stored in memory - they are consumed by the loader.

---

## Practical Example

### Creating a PRG in Python

```python
def create_prg(load_address: int, data: bytes) -> bytes:
    """Create a PRG file with load address and data."""
    low_byte = load_address & 0xFF
    high_byte = (load_address >> 8) & 0xFF
    return bytes([low_byte, high_byte]) + data

# Example: Simple program at $C000
# LDA #$00; STA $D020; RTS
code = bytes([0xA9, 0x00, 0x8D, 0x20, 0xD0, 0x60])
prg = create_prg(0xC000, code)

with open("test.prg", "wb") as f:
    f.write(prg)
```

### Creating PRG with BASIC Stub

```python
def create_basic_stub(sys_address: int) -> bytes:
    """Create BASIC stub that calls SYS address."""
    addr_str = str(sys_address).encode('ascii')

    # BASIC line at $0801
    stub = bytearray()

    # Pointer to next line (will be end of this line)
    next_line = 0x0801 + 4 + 1 + len(addr_str) + 1
    stub.extend([next_line & 0xFF, (next_line >> 8) & 0xFF])

    # Line number 10
    stub.extend([0x0A, 0x00])

    # SYS token
    stub.append(0x9E)

    # Address as ASCII
    stub.extend(addr_str)

    # End of line
    stub.append(0x00)

    # End of program (null pointer)
    stub.extend([0x00, 0x00])

    return bytes(stub)

def create_autostart_prg(code: bytes) -> bytes:
    """Create PRG with BASIC stub for autostart."""
    load_address = 0x0801
    stub = create_basic_stub(0x0801 + len(create_basic_stub(0)))

    # Adjust stub to point to actual code location
    code_start = 0x0801 + len(stub)
    stub = create_basic_stub(code_start)

    return create_prg(load_address, stub + code)
```

---

## Related Formats

| Format | Description                   |
| ------ | ----------------------------- |
| `.prg` | Standard program file         |
| `.seq` | Sequential data file          |
| `.p00` | PC64 format (PRG with header) |
| `.t64` | Tape image (can contain PRG)  |

---

## Sources

- [Commodore 64 binary executable](http://justsolve.archiveteam.org/wiki/Commodore_64_binary_executable)
- [What is a .PRG file anyway? - Lemon64](https://www.lemon64.com/forum/viewtopic.php?t=69720)
- [VICE Manual - File Formats](https://vice-emu.sourceforge.io/vice_17.html)
