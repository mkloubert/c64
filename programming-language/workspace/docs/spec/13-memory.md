# Memory Management and Register Allocation

This document describes memory management and register allocation strategies.

---

## Register Allocation

### 6510 Registers

| Register | Size  | Primary Use                           |
| -------- | ----- | ------------------------------------- |
| A        | 8-bit | Accumulator, arithmetic, return value |
| X        | 8-bit | Index, loop counter, return high byte |
| Y        | 8-bit | Index for indirect addressing         |
| SP       | 8-bit | Stack pointer (hardware managed)      |
| P        | 8-bit | Status flags (automatic)              |

### Register Allocation Strategy

**Simple approach:** No complex register allocation. Use A for most operations, X/Y for indexing.

### Register Assignment by Operation

| Operation               | Registers Used          |
| ----------------------- | ----------------------- |
| Load variable           | A                       |
| Store variable          | A                       |
| Arithmetic (+, -, etc.) | A                       |
| Array index             | X or Y                  |
| Function call           | A (param), A/X (return) |
| Loop counter (for)      | Memory + X              |
| Comparison              | A                       |
| Word operations         | A + memory              |

### Caller-Save Convention

All registers are caller-save:

- Caller must save A, X, Y if needed after function call
- Functions can freely use all registers

```
# If caller needs X preserved across call:
STX _save_x
JSR some_function
LDX _save_x
```

---

## Stack Usage

### Hardware Stack ($0100-$01FF)

The 6502 stack is used ONLY for:

- Return addresses (JSR/RTS)
- Saved registers (PHA/PLA, PHX/PLX, PHY/PLY)

**Stack is NOT used for:**

- Local variables (fixed memory instead)
- Parameters (fixed memory instead)
- Temporary values (memory temporaries instead)

### Stack Depth

Maximum call depth depends on stack usage:

- Each JSR uses 2 bytes
- Each register save uses 1 byte
- ~100 nested calls possible (conservative estimate)

### No Recursion

Due to fixed variable allocation, recursion is not supported:

```
# NOT ALLOWED - will corrupt variables
def factorial(byte n) -> word:
    if n <= 1:
        return 1
    return n * factorial(n - 1)  # ERROR!
```

**Workaround:** Use iterative algorithms:

```
def factorial(byte n) -> word:
    word result = 1
    for i in 1 to n:
        result = result * i
    return result
```

---

## Zero Page Usage

### Reserved by System

| Range   | Usage                  |
| ------- | ---------------------- |
| $00-$01 | CPU I/O port           |
| $03-$8F | BASIC/KERNAL workspace |
| $90-$FA | BASIC/KERNAL variables |
| $FF     | Temp storage           |

### Available for Compiler

| Address | Usage               |
| ------- | ------------------- |
| $02     | General temporary   |
| $FB     | Pointer 1 low byte  |
| $FC     | Pointer 1 high byte |
| $FD     | Pointer 2 low byte  |
| $FE     | Pointer 2 high byte |

### Extended Zero Page (Without BASIC)

If BASIC ROM is disabled, more zero page is available:

| Range   | Size      | Notes           |
| ------- | --------- | --------------- |
| $02-$8F | 142 bytes | BASIC workspace |
| $FB-$FE | 4 bytes   | Always free     |

### Zero Page Allocation Priority

1. **Pointers for indirect addressing** ($FB-$FE)
2. **Frequently accessed variables** (loop counters)
3. **Temporary values** during expression evaluation

---

## Variable Allocation

### Allocation Algorithm

1. **Count global variables** - Calculate total bytes needed
2. **Assign addresses** - Sequential allocation after code
3. **Generate symbol table** - Map names to addresses
4. **Count local variables per function** - Each function gets region
5. **Assign function variable regions** - Non-overlapping

### Example Allocation

```
# Source
byte player_x = 100
byte player_y = 100
word score = 0
byte enemies[8]

def update():
    byte dx = 0
    byte dy = 0
    # ...

def draw():
    byte i = 0
    # ...
```

```
# Generated symbol table
GLOBALS_START = $2000
player_x      = $2000   ; 1 byte
player_y      = $2001   ; 1 byte
score         = $2002   ; 2 bytes
enemies       = $2004   ; 8 bytes
GLOBALS_END   = $200C

update_dx     = $200C   ; 1 byte
update_dy     = $200D   ; 1 byte

draw_i        = $200E   ; 1 byte
```

### String Variable Allocation

Strings reserve full buffer:

```
string name             ; 256 bytes default
string[32] short_name   ; 33 bytes (32 + null)
```

---

## Memory Optimization

### Byte vs Word

Prefer `byte` over `word` when possible:

| Operation | Byte Cost | Word Cost |
| --------- | --------- | --------- |
| Load      | 2 bytes   | 4 bytes   |
| Store     | 2 bytes   | 4 bytes   |
| Add       | 3 bytes   | 8 bytes   |
| Compare   | 2 bytes   | 8+ bytes  |

### Array Size Limits

Keep arrays under 256 bytes for efficient indexing:

```
byte data[256]      # Good: can use X/Y indexing
byte big[300]       # Slower: needs indirect indexing
```

### Variable Reuse

Non-overlapping local variables could share memory:

```
def func1():
    byte temp = 0       # At $2000

def func2():
    byte counter = 0    # Could also use $2000
```

---

## Expression Temporaries

### Temporary Variables

Complex expressions need temporary storage:

```
# x = (a + b) * (c + d)
# Needs: temp1 = a + b, temp2 = c + d, then multiply
```

### Temporary Allocation

```
_temp1 = $XX    ; Expression temporary 1
_temp2 = $XX    ; Expression temporary 2
_temp3 = $XX    ; Expression temporary 3
```

### Temporary Pool

Estimate maximum expression depth:

- Most expressions need 1-2 temporaries
- Deeply nested: up to 4-5 temporaries
- Allocate fixed pool of 8 bytes for temporaries

---

## Function Memory Layout

### Per-Function Allocation

Each function gets dedicated memory region:

```
function_name:
    ; Code
    ...
    RTS

function_name_param1:   .byte 0
function_name_param2:   .byte 0
function_name_local1:   .byte 0
function_name_local2:   .word 0
```

### Parameter Passing Example

```
# Source
def draw_at(byte x, byte y, byte char):
    cursor(x, y)
    print(char)

# Caller
draw_at(10, 5, '*')

# Generated caller code
LDA #10
STA draw_at_x
LDA #5
STA draw_at_y
LDA #'*'
STA draw_at_char
JSR draw_at
```

---

## Memory Map Generation

### Output Files

The compiler generates:

1. **Assembly source** (.asm) - Human-readable 64tass code
2. **Symbol file** (.sym) - Address map for debugging
3. **Memory map** (.map) - Memory layout summary

### Symbol File Format

```
; Symbol file for game.c64
; Generated by compiler

; Global variables
player_x        = $2000
player_y        = $2001
score           = $2002

; Functions
main            = $0810
update          = $0850
draw            = $08A0

; Constants
SCREEN          = $0400
VIC             = $D000
```

### Memory Map Format

```
Memory Map for game.c64
=======================

Code segment:    $0801 - $09FF (510 bytes)
Data segment:    $0A00 - $0A1F (32 bytes)
BSS segment:     $0A20 - $0AFF (224 bytes)

Free RAM:        $0B00 - $9FFF (38,656 bytes)

Total program:   766 bytes
Available RAM:   38,656 bytes
```

---

## Runtime Checks (Debug Mode)

### Optional Bounds Checking

In debug builds, add array bounds checks:

```
# array[index] with check
LDA index
CMP #ARRAY_SIZE
BCS _bounds_error
; Normal access
LDA array,X
JMP _bounds_ok
_bounds_error:
    ; Error handler
_bounds_ok:
```

### Stack Overflow Detection

Monitor stack depth:

```
; At function entry (debug mode)
TSX
CPX #$20            ; Stack getting low?
BCC _stack_overflow
```

### Disabled in Release

Debug checks are removed in release builds for performance.
