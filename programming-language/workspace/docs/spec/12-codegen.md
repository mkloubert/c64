# Code Generation Strategy

This document defines how source code maps to 6510 assembly.

---

## Memory Layout

### Program Memory Map

```
$0000-$00FF   Zero Page (CPU registers, pointers, temp vars)
$0100-$01FF   Hardware Stack (6502 stack, 256 bytes)
$0200-$03FF   System variables, input buffer
$0400-$07FF   Screen memory (default)
$0800-$0800   Unused byte
$0801-$XXXX   Program code (BASIC stub + machine code)
$XXXX-$9FFF   Program data, variables, arrays
$A000-$BFFF   BASIC ROM (can be banked out)
$C000-$CFFF   Free RAM (4KB)
$D000-$DFFF   I/O registers (VIC, SID, CIA)
$E000-$FFFF   KERNAL ROM
```

### Compiler Memory Allocation

| Region           | Address       | Size    | Usage                   |
| ---------------- | ------------- | ------- | ----------------------- |
| Zero Page Vars   | $02, $FB-$FE  | 5 bytes | Compiler temp variables |
| Program Start    | $0801         | -       | BASIC stub + code       |
| Global Variables | After code    | -       | Static variable storage |
| String Literals  | After globals | -       | Constant strings        |
| Arrays           | After strings | -       | Array storage           |
| Heap/Free        | Remaining     | -       | Available RAM           |

---

## Variable Storage

### Zero Page Variables

Reserved zero page locations for compiler use:

| Address | Usage              |
| ------- | ------------------ |
| $02     | General temp       |
| $FB-$FC | Pointer 1 (16-bit) |
| $FD-$FE | Pointer 2 (16-bit) |

### Global Variables

Global variables are stored in RAM after program code:

```
# Source
byte score = 0
word high_score = 10000
bool game_over = false

# Memory layout (example at $2000)
# $2000: score (1 byte)
# $2001: high_score low byte
# $2002: high_score high byte
# $2003: game_over (1 byte)
```

### Local Variables

Local variables in functions use fixed memory locations (not stack):

```
def calculate(byte a, byte b) -> byte:
    byte temp = a + b       # temp at fixed address
    return temp

# Each function has its own variable region
# calculate.a     = $2100
# calculate.b     = $2101
# calculate.temp  = $2102
```

**Note:** This means functions are NOT reentrant (no recursion).

### Array Storage

Arrays are stored contiguously:

```
byte data[10]
# Stored as 10 consecutive bytes

word scores[5]
# Stored as 10 consecutive bytes (5 words × 2 bytes)
# scores[0] at base+0, base+1
# scores[1] at base+2, base+3
# etc.
```

---

## Function Calling Convention

### Overview

- Parameters passed via fixed memory locations
- Return value in A register (byte) or A/X (word: A=low, X=high)
- Caller saves registers if needed
- No stack frames (fixed allocation)

### Parameter Passing

```
def add(byte a, byte b) -> byte:
    return a + b

# Calling: result = add(10, 20)
# Generated:
#   LDA #10
#   STA add_param_a
#   LDA #20
#   STA add_param_b
#   JSR add
#   STA result          ; Return value in A
```

### Return Values

| Type   | Location            |
| ------ | ------------------- |
| `byte` | A register          |
| `word` | A (low), X (high)   |
| `bool` | A register (0 or 1) |
| `void` | -                   |

### Register Usage

| Register | Usage                                             |
| -------- | ------------------------------------------------- |
| A        | Primary work register, return value (low)         |
| X        | Secondary work, return value (high), loop counter |
| Y        | Array indexing, temp                              |
| SP       | Hardware stack (JSR/RTS only)                     |

### Example Function

```
# Source
def multiply_by_2(byte value) -> byte:
    return value << 1

# Generated Assembly
multiply_by_2:
    LDA multiply_by_2_value    ; Load parameter
    ASL A                       ; Shift left = × 2
    RTS                         ; Return (result in A)

multiply_by_2_value:
    .byte 0                     ; Parameter storage
```

---

## Expression Evaluation

### Strategy: Accumulator-Based

Expressions are evaluated using the A register as primary accumulator, with memory temporaries for complex expressions.

### Simple Expressions

```
# x = a + b
LDA a
CLC
ADC b
STA x
```

### Complex Expressions

```
# x = (a + b) * c
# Strategy: evaluate left-to-right, save intermediates

LDA a
CLC
ADC b           ; A = a + b
STA _temp1      ; Save intermediate
LDA _temp1
; Call multiply routine with c
JSR _mul_byte
STA x
```

### Operator Code Generation

| Operator | Byte Code  | Notes               |
| -------- | ---------- | ------------------- | ---------- |
| `+`      | `CLC; ADC` | Clear carry, add    |
| `-`      | `SEC; SBC` | Set carry, subtract |
| `&`      | `AND`      | Bitwise AND         |
| `        | `          | `ORA`               | Bitwise OR |
| `^`      | `EOR`      | Bitwise XOR         |
| `<<`     | `ASL` (×n) | Shift left          |
| `>>`     | `LSR` (×n) | Shift right         |
| `*`      | `JSR _mul` | Software multiply   |
| `/`      | `JSR _div` | Software divide     |

### Comparison Code Generation

```
# if a > b:
LDA a
CMP b
BEQ _skip       ; Equal? Skip
BCC _skip       ; Less? Skip
; ... then block ...
_skip:
```

| Comparison | Branch Sequence              |
| ---------- | ---------------------------- |
| `==`       | `CMP; BNE skip`              |
| `!=`       | `CMP; BEQ skip`              |
| `<`        | `CMP; BCS skip`              |
| `>=`       | `CMP; BCC skip`              |
| `>`        | `CMP; BCC skip; BEQ skip`    |
| `<=`       | `CMP; BEQ ok; BCS skip; ok:` |

---

## Control Flow Generation

### If Statement

```
# Source
if condition:
    then_block
else:
    else_block

# Generated
    ; Evaluate condition into A
    BEQ _else           ; Branch if false (zero)
    ; then_block code
    JMP _endif
_else:
    ; else_block code
_endif:
```

### While Loop

```
# Source
while condition:
    body

# Generated
_while_start:
    ; Evaluate condition
    BEQ _while_end      ; Exit if false
    ; body code
    JMP _while_start
_while_end:
```

### For Loop

```
# Source
for i in 0 to 9:
    body

# Generated
    LDA #0
    STA i
_for_start:
    ; body code
    INC i
    LDA i
    CMP #10             ; end + 1
    BNE _for_start
_for_end:
```

### For Downto Loop

```
# Source
for i in 9 downto 0:
    body

# Generated
    LDA #9
    STA i
_for_start:
    ; body code
    DEC i
    BPL _for_start      ; Continue while >= 0
_for_end:
```

### Break and Continue

```
# break
JMP _loop_end

# continue
JMP _loop_start
```

---

## Word (16-bit) Operations

### Word Addition

```
# result = a + b (both words)
CLC
LDA a           ; Low byte
ADC b
STA result
LDA a+1         ; High byte
ADC b+1
STA result+1
```

### Word Subtraction

```
# result = a - b
SEC
LDA a
SBC b
STA result
LDA a+1
SBC b+1
STA result+1
```

### Word Comparison

```
# if a < b (unsigned)
LDA a+1         ; Compare high bytes first
CMP b+1
BCC _less       ; a_hi < b_hi
BNE _not_less   ; a_hi > b_hi
LDA a           ; High bytes equal, compare low
CMP b
BCC _less
_not_less:
    ; a >= b
    JMP _done
_less:
    ; a < b
_done:
```

### Word Increment/Decrement

```
# word++
INC word
BNE _done
INC word+1
_done:

# word--
LDA word
BNE _no_borrow
DEC word+1
_no_borrow:
DEC word
```

---

## Array Access

### Byte Array Read

```
# value = array[index]
LDX index
LDA array,X
STA value
```

### Byte Array Write

```
# array[index] = value
LDX index
LDA value
STA array,X
```

### Word Array Read

```
# value = words[index]
LDA index
ASL A               ; × 2 (word size)
TAX
LDA words,X
STA value
LDA words+1,X
STA value+1
```

### Variable Index (Indirect)

For arrays larger than 256 bytes or variable base:

```
# Using zero page pointer
LDA #<array
STA $FB
LDA #>array
STA $FC
LDY index
LDA ($FB),Y
```

---

## String Handling

### String Literals

Stored in ROM/data section:

```
string msg = "HELLO"

; Data section
_str_1:
    .byte "HELLO", 0

; Variable holds pointer
msg:
    .word _str_1
```

### Print String

```
# print(msg)
LDA msg         ; Low byte of pointer
STA $FB
LDA msg+1       ; High byte
STA $FC
LDY #0
_loop:
    LDA ($FB),Y
    BEQ _done       ; Null terminator
    JSR $FFD2       ; CHROUT
    INY
    BNE _loop
_done:
```

---

## Runtime Library

### Required Routines

| Routine       | Purpose               |
| ------------- | --------------------- |
| `_mul_byte`   | 8-bit multiplication  |
| `_mul_word`   | 16-bit multiplication |
| `_div_byte`   | 8-bit division        |
| `_div_word`   | 16-bit division       |
| `_print_byte` | Print byte as decimal |
| `_print_word` | Print word as decimal |

### Multiplication (8-bit)

```asm
; A × X -> A (low), X (high)
_mul_byte:
    STA _mul_a
    STX _mul_b
    LDA #0
    LDX #8
_mul_loop:
    LSR _mul_b
    BCC _mul_skip
    CLC
    ADC _mul_a
_mul_skip:
    ASL _mul_a
    DEX
    BNE _mul_loop
    RTS
```

---

## BASIC Stub

Programs start with a BASIC stub for autostart:

```asm
    * = $0801

    ; BASIC line: 10 SYS 2062
    .word _next_line    ; Pointer to next line
    .word 10            ; Line number
    .byte $9E           ; SYS token
    .text "2062"        ; Address as text
    .byte 0             ; End of line
_next_line:
    .word 0             ; End of program

    ; Machine code starts here ($080E = 2062)
start:
    ; Program code...
```

---

## Optimization Opportunities

### Constant Folding

```
# Compile time
const X = 10 + 20       ; Becomes 30

byte a = 5 * 4          ; Becomes 20
```

### Strength Reduction

```
# x * 2 -> x << 1
ASL A

# x * 4 -> x << 2
ASL A
ASL A

# x / 2 -> x >> 1
LSR A
```

### Dead Code Elimination

```
# Unreachable code after return
def foo():
    return 5
    print("never")      ; Eliminated
```

### Peephole Optimization

```
# Before
LDA x
STA temp
LDA temp

# After
LDA x
STA temp
; Second LDA eliminated (A already has value)
```
