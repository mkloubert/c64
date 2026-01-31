# Data Types for the C64 Language

This document describes the data types that are possible on the C64 based on the 6502/6510 CPU architecture.

## Hardware Constraints

The 6510 CPU is an 8-bit processor with:

- 8-bit accumulator (A register)
- 8-bit X and Y index registers
- 16-bit address bus (can address 65,536 bytes)
- Little-endian byte order for 16-bit values

## Recommended Data Types

### Primitive Types

| Type    | Size    | Range            | Description                             |
| ------- | ------- | ---------------- | --------------------------------------- |
| `byte`  | 1 byte  | 0-255            | Unsigned 8-bit integer                  |
| `sbyte` | 1 byte  | -128 to +127     | Signed 8-bit integer (two's complement) |
| `word`  | 2 bytes | 0-65535          | Unsigned 16-bit integer                 |
| `sword` | 2 bytes | -32768 to +32767 | Signed 16-bit integer                   |
| `bool`  | 1 byte  | true/false       | Boolean value (0=false, non-zero=true)  |

### Complex Types (Optional/Future)

| Type     | Size     | Notes                                                     |
| -------- | -------- | --------------------------------------------------------- |
| `float`  | 5 bytes  | Uses C64 BASIC ROM routines, slow                         |
| `string` | variable | Null-terminated or length-prefixed                        |
| `array`  | variable | Fixed-size arrays only (max 256 elements with byte index) |

## Memory Layout

### Little-Endian Storage

16-bit values are stored with the low byte first:

- Value `$1234` stored at address `$0000`:
  - `$0000` contains `$34` (low byte)
  - `$0001` contains `$12` (high byte)

### Zero Page ($00-$FF)

The first 256 bytes of memory are special:

- Faster access (fewer cycles)
- Required for indirect addressing modes
- Most used by KERNAL/BASIC, but some are free:
  - `$02` - Free
  - `$FB-$FE` - Free (4 consecutive bytes, good for pointers)

## Design Recommendations

### Keep It Simple

From the prog8 project: "dealing with anything but bytes on the 6502 quickly turns into a mess"

Therefore our language should:

1. Prefer `byte` as the default integer type
2. Make `word` available but warn about performance cost
3. Avoid floating-point unless absolutely necessary
4. Limit array sizes to 256 elements for simple indexing

### Type Inference

Consider automatic type inference to reduce verbosity:

```
x = 42        # Inferred as byte (fits in 0-255)
y = 1000      # Inferred as word (needs 16 bits)
z = true      # Inferred as bool
```

### Implicit Conversions

Allow safe implicit conversions (smaller to larger):

- `byte` -> `word` (safe)
- `sbyte` -> `sword` (safe)
- `word` -> `byte` (warning or error, data loss)

## References

- [6502 Assembly Wikibooks](https://en.wikibooks.org/wiki/6502_Assembly)
- [C64 Memory Map](https://sta.c64.org/cbm64mem.html)
- [C64 Zero Page](https://www.c64-wiki.com/wiki/Zeropage)
- [prog8 Data Types](https://prog8.readthedocs.io/en/stable/programming.html)
