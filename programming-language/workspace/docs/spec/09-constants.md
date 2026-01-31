# Constants and Compile-Time Evaluation

This document defines constant expressions and their evaluation.

---

## Constant Declaration

### Syntax

```
const NAME = expression
```

### Examples

```
const MAX_LIVES = 3
const SCREEN_WIDTH = 40
const SCREEN_HEIGHT = 25
const SCREEN_SIZE = SCREEN_WIDTH * SCREEN_HEIGHT
const START_ADDR = $0801
const BIT_MASK = %10101010
```

### Naming Convention

Constants use UPPER_SNAKE_CASE by convention.

---

## Constant Properties

| Property     | Description                        |
| ------------ | ---------------------------------- |
| Immutable    | Cannot be changed after definition |
| Compile-time | Evaluated during compilation       |
| No storage   | Do not occupy runtime memory       |
| Global scope | Visible from point of definition   |

### Example: No Runtime Storage

```
const OFFSET = 100

def example():
    byte x = OFFSET     # Compiled as: byte x = 100
    # OFFSET does not use any RAM at runtime
```

---

## Constant Expressions

A constant expression can contain:

### Allowed in Constant Expressions

| Element              | Example                          |
| -------------------- | -------------------------------- | ----------------------- |
| Integer literals     | `42`, `$FF`, `%1010`             |
| Other constants      | `MAX + 1`                        |
| Arithmetic operators | `+`, `-`, `*`, `/`, `%`          |
| Bitwise operators    | `&`, `                           | `, `^`, `~`, `<<`, `>>` |
| Comparison operators | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical operators    | `and`, `or`, `not`               |
| Parentheses          | `(A + B) * C`                    |
| Boolean literals     | `true`, `false`                  |

### NOT Allowed in Constant Expressions

| Element         | Reason                    |
| --------------- | ------------------------- |
| Variables       | Not known at compile time |
| Function calls  | Runtime evaluation        |
| Array elements  | Runtime memory access     |
| String literals | Not a numeric constant    |

### Examples

```
# Valid constant expressions
const A = 10
const B = 20
const C = A + B             # 30
const D = A * B + 5         # 205
const E = (A + B) * 2       # 60
const F = A << 2            # 40
const G = $FF & $0F         # 15
const H = A > B             # false (0)
const I = A < B and B < 50  # true (1)

# Invalid constant expressions
byte x = 5
const BAD1 = x              # Error: variable not allowed
const BAD2 = strlen("HI")   # Error: function call not allowed
const BAD3 = data[0]        # Error: array access not allowed
```

---

## Constant Type Inference

### Automatic Type Selection

Constants get the smallest type that fits the value:

| Value Range    | Inferred Type |
| -------------- | ------------- |
| 0 to 255       | `byte`        |
| 256 to 65535   | `word`        |
| -128 to -1     | `sbyte`       |
| -32768 to -129 | `sword`       |
| true/false     | `bool`        |

### Examples

```
const SMALL = 100           # byte
const MEDIUM = 1000         # word
const NEGATIVE = -50        # sbyte
const BIG_NEG = -1000       # sword
const FLAG = true           # bool
```

### Type Adaptation

When used in expressions, constants adapt to context:

```
const VALUE = 100           # Type: byte

byte b = VALUE              # Used as byte
word w = VALUE              # Used as word (promoted)
word sum = VALUE + 1000     # VALUE promoted to word for addition
```

---

## Constant Evaluation Rules

### Arithmetic

Standard integer arithmetic, evaluated with full precision:

```
const A = 200
const B = 200
const C = A + B     # C = 400 (word, not overflow)

const D = 255
const E = D + 1     # E = 256 (word)
```

### Division

Integer division, truncates toward zero:

```
const A = 7 / 2     # A = 3
const B = -7 / 2    # B = -3 (toward zero)
const C = 100 / 3   # C = 33
```

### Modulo

Remainder after division:

```
const A = 7 % 3     # A = 1
const B = 100 % 10  # B = 0
const C = 17 % 5    # C = 2
```

### Bit Shifts

```
const A = 1 << 4    # A = 16
const B = 128 >> 3  # B = 16
const C = $FF << 8  # C = $FF00 (word)
```

### Bitwise Operations

```
const A = $F0 | $0F         # A = $FF
const B = $FF & $0F         # B = $0F
const C = $AA ^ $55         # C = $FF
const D = ~$0F & $FF        # D = $F0
```

### Logical Operations

Return 0 (false) or 1 (true):

```
const A = 10 > 5            # A = 1 (true)
const B = 10 < 5            # B = 0 (false)
const C = true and false    # C = 0
const D = true or false     # D = 1
const E = not false         # E = 1
```

---

## Constant Usage

### In Variable Declarations

```
const MAX = 100
byte value = MAX            # Initialized to 100
```

### In Array Sizes

```
const SIZE = 10
byte data[SIZE]             # Array of 10 bytes
```

### In Expressions

```
const OFFSET = 32
byte adjusted = input + OFFSET
```

### In Control Flow

```
const MAX_ITER = 100
for i in 0 to MAX_ITER - 1:
    process(i)
```

### In Function Calls

```
const DEFAULT_COLOR = WHITE
text_color(DEFAULT_COLOR)
```

---

## Forward References

Constants must be defined before use:

```
# Error: SECOND used before definition
const FIRST = SECOND + 1    # Error!
const SECOND = 10

# Correct order
const SECOND = 10
const FIRST = SECOND + 1    # OK: 11
```

---

## Constant Visibility

Constants are visible from their definition point to end of file:

```
# TOP cannot use BOTTOM (not yet defined)
const TOP = 10

def function1():
    byte x = TOP        # OK
    byte y = BOTTOM     # OK (defined before function body executes)

const BOTTOM = 20

def function2():
    byte z = TOP + BOTTOM   # OK
```

---

## Built-in Constants

### Color Constants

```
const BLACK = 0
const WHITE = 1
const RED = 2
const CYAN = 3
const PURPLE = 4
const GREEN = 5
const BLUE = 6
const YELLOW = 7
const ORANGE = 8
const BROWN = 9
const LIGHTRED = 10
const DARKGREY = 11
const GREY = 12
const LIGHTGREEN = 13
const LIGHTBLUE = 14
const LIGHTGREY = 15
```

### Joystick Constants

```
const JOY_UP = 1
const JOY_DOWN = 2
const JOY_LEFT = 4
const JOY_RIGHT = 8
const JOY_FIRE = 16
```

### Waveform Constants

```
const WAVE_TRIANGLE = 16
const WAVE_SAW = 32
const WAVE_PULSE = 64
const WAVE_NOISE = 128
```

### System Constants

```
const SCREEN_ADDR = $0400
const COLOR_ADDR = $D800
const VIC_BASE = $D000
const SID_BASE = $D400
const CIA1_BASE = $DC00
const CIA2_BASE = $DD00
```

---

## Constant Errors

### Error: Constant Required

```
byte x = 10
const BAD = x           # Error E230: Constant expression required
```

### Error: Out of Range

```
const TOO_BIG = 100000  # Error E232: Constant value out of range
```

### Error: Undefined Constant

```
const A = B + 1         # Error E200: Undefined 'B'
```

### Error: Circular Definition

```
const A = B + 1
const B = A + 1         # Error: Circular dependency
```

---

## Implementation Notes

### Constant Folding

The compiler evaluates constants during compilation:

```
# Source
const A = 10
const B = 20
byte x = A + B * 2

# After constant folding
byte x = 50             # Computed at compile time
```

### No Memory Allocation

Constants do not generate any runtime code or data:

```
const SIZE = 100
# No LDA, STA, or memory reservation for SIZE
```

### Inline Substitution

Constants are substituted directly into code:

```
const BORDER = $D020
poke(BORDER, RED)

# Compiles to:
# LDA #2          ; RED
# STA $D020       ; BORDER substituted
```
