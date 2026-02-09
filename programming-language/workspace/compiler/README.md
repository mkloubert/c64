# Cobra64 Compiler Documentation

<p align="center">
  <img src="cobra64_logo.png" alt="Cobra64 Logo" width="256">
</p>

A modern programming language and compiler for the Commodore 64.

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Language Reference](#language-reference)
   - [Data Types](#data-types)
   - [Variables](#variables)
   - [Constants](#constants)
   - [Arrays](#arrays)
   - [Operators](#operators)
   - [Type Casting](#type-casting)
   - [Control Flow](#control-flow)
   - [Functions](#functions)
   - [Comments](#comments)
5. [Built-in Functions](#built-in-functions)
   - [Screen Functions](#screen-functions)
   - [Output Functions](#output-functions)
   - [Input Functions](#input-functions)
   - [Memory Functions](#memory-functions)
   - [Array and String Functions](#array-and-string-functions)
   - [Random Number Functions](#random-number-functions)
   - [Sprite Functions](#sprite-functions)
   - [Sound Functions](#sound-functions)
6. [Example Programs](#example-programs)
7. [Error Messages](#error-messages)
8. [Limitations](#limitations)

---

## Overview

Cobra64 is a compiled programming language designed specifically for the Commodore 64 8-bit home computer. It features:

- **Python-like syntax** - Clean, indentation-based structure
- **Strong typing** - Catch errors at compile time
- **Direct hardware access** - Work with C64 memory and I/O
- **Modern tooling** - Compile to PRG or D64 disk images

The compiler generates native 6510 machine code that runs directly on the C64 (or emulators like VICE).

### Target Platform

- **CPU:** MOS 6510 (6502 compatible)
- **Memory:** 64KB RAM
- **Output formats:** PRG files, D64 disk images

---

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

### Building from Source

```bash
cd /workspace/compiler
cargo build --release
```

The compiler binary will be at `target/release/cobra64`.

### Running

```bash
# Compile to PRG file
./target/release/cobra64 program.cb64 -o program.prg

# Compile to D64 disk image
./target/release/cobra64 program.cb64 -o program.d64

# Verbose output
./target/release/cobra64 program.cb64 -o program.prg -v
```

---

## Quick Start

### Hello World

Create a file named `hello.cb64`:

```python
def main():
    cls()
    println("HELLO WORLD")
```

Compile and run:

```bash
# Compile
cobra64 hello.cb64 -o hello.prg

# Run in VICE emulator
x64sc hello.prg
```

### Program Structure

Every Cobra64 program needs a `main()` function as the entry point:

```python
def main():
    # Your code here
    pass
```

### Using Arrays

Arrays store multiple values of the same type:

```python
def main():
    # Create and initialize an array
    scores: word[] = [100, 250, 500]

    # Access elements by index
    first: word = scores[0]
    scores[1] = 300

    # Use arrays in loops
    i: byte = 0
    while i < 3:
        poke(1024 + i, byte(scores[i]))
        i = i + 1
```

---

## IDE Support

### VS Code Extension

A full-featured VS Code extension is available in the `/workspace/lsp` directory providing:

- **Syntax Highlighting** - Full support for all Cobra64 constructs
- **Real-time Diagnostics** - Error checking as you type
- **IntelliSense** - Auto-completion for keywords, types, and functions
- **Hover Information** - Type info and documentation
- **Go to Definition** - Navigate to declarations
- **Find All References** - Locate all symbol usages
- **Rename Symbol** - Safely rename across files
- **Signature Help** - Parameter hints for function calls

To use the extension:

```bash
cd /workspace/lsp
npm install
npm run compile
# Press F5 in VS Code to launch Extension Development Host
```

See `/workspace/lsp/README.md` for full documentation.

---

## Language Reference

### Data Types

#### Integer Types

| Type    | Size    | Range           | Description     |
| ------- | ------- | --------------- | --------------- |
| `byte`  | 1 byte  | 0 to 255        | Unsigned 8-bit  |
| `sbyte` | 1 byte  | -128 to 127     | Signed 8-bit    |
| `word`  | 2 bytes | 0 to 65535      | Unsigned 16-bit |
| `sword` | 2 bytes | -32768 to 32767 | Signed 16-bit   |

#### Decimal Types

| Type    | Size    | Range                 | Precision         | Description             |
| ------- | ------- | --------------------- | ----------------- | ----------------------- |
| `fixed` | 2 bytes | -2048.0 to +2047.9375 | 0.0625 (1/16)     | Fixed-point 12.4 format |
| `float` | 2 bytes | ±65504 (±6.1e-5 min)  | ~3 decimal digits | IEEE-754 binary16       |

#### Other Types

| Type     | Size   | Range             | Description   |
| -------- | ------ | ----------------- | ------------- |
| `bool`   | 1 byte | `true` or `false` | Boolean value |
| `string` | varies | -                 | Text string   |

#### Signed vs Unsigned Types

Use signed types (`sbyte`, `sword`) when you need negative numbers:

```python
def main():
    # Signed types can hold negative values
    temperature: sbyte = -10
    altitude: sword = -500

    # Unsigned types cannot (will cause compile error)
    # x: byte = -1  # Error: out of range for byte
```

#### Fixed vs Float Types

Both `fixed` and `float` support decimal numbers but have different trade-offs:

**Fixed-point (12.4 format):**

- Constant precision of 0.0625 (1/16) across entire range
- Fast operations (similar speed to integers)
- Best for: screen coordinates, smooth scrolling, game physics
- Limited range: -2048.0 to +2047.9375

**Float (IEEE-754 binary16):**

- Relative precision (~3 decimal digits)
- Slower operations (software emulated)
- Best for: scientific calculations, large value ranges
- Wide range: ±65504

```python
def main():
    # Use fixed for game coordinates with subpixel precision
    x: fixed = 100.5
    speed: fixed = 0.125

    # Use float for calculations with large numbers
    distance: float = 50000.0
    scale: float = 0.001
```

**Performance comparison (approximate cycles):**

| Operation | Integer | Fixed | Float |
| --------- | ------- | ----- | ----- |
| Add/Sub   | ~10     | ~20   | ~300  |
| Multiply  | ~100    | ~150  | ~1200 |
| Divide    | ~200    | ~250  | ~2500 |

#### Literals

```python
# Integer literals
x: byte = 42
y: word = 1000

# Negative numbers (for signed types)
temp: sbyte = -50
offset: sword = -1000

# Hexadecimal (prefix $)
addr: word = $D020      # VIC border color register
neg_hex: sbyte = -$7F   # -127

# Binary (prefix %)
mask: byte = %10101010
neg_bin: sbyte = -%01111111  # -127

# Fixed-point literals (decimal with type annotation)
pos: fixed = 100.5
velocity: fixed = -2.25
fraction: fixed = 0.0625    # Smallest fixed fraction (1/16)

# Float literals (decimal notation)
pi: float = 3.14159
tiny: float = 0.001
big: float = 50000.0

# Scientific notation (for float)
light_speed: float = 3.0e8   # 300000000
planck: float = 6.6e-10      # Very small number

# Strings
msg: string = "HELLO"

# Characters
ch: byte = 'A'          # PETSCII value 65

# Booleans
flag: bool = true
```

**Note:** Decimal literals (like `3.14`) are inferred as `float` by default. Use explicit type annotation to get `fixed`:

```python
f: fixed = 3.14    # fixed-point
g: float = 3.14    # float (default for decimals)
```

### Variables

All variable declarations require an explicit type annotation:

```python
# Top-level variables
counter: byte = 0
score: word = 1000
temperature: sbyte = -10
offset: sword = -1000
pi: float = 3.14
active: bool = true

def main():
    # Variables inside functions
    a: byte = 1
    b: byte = 2
    c: byte = a + b
```

**Note:** Explicit type annotations are always required for variable declarations.

### Constants

Constants are declared using the `const` keyword. All constant declarations require an explicit type annotation.

```python
# Constants with explicit types using 'const' keyword
const MAX_SCORE: byte = 255
const SCREEN_WIDTH: byte = 40
const LARGE_VALUE: word = 1000
const NEGATIVE: sbyte = -100
const PI: fixed = 3.14159
const E: float = 2.718

def main():
    # Local constants inside functions
    const HEADER: string = "SCORE:"

    x: byte = MAX_SCORE
    y: word = LARGE_VALUE + 1
```

#### Naming Rules

| Type         | Rule                                      | Examples                |
| ------------ | ----------------------------------------- | ----------------------- |
| **Constant** | Use `const` keyword                       | `const MAX: byte = 255` |
| **Variable** | No keyword, just type annotation          | `counter: byte = 0`     |
| **Invalid**  | Identifier consisting only of underscores | `_`, `__`, `___`        |

```python
# Valid constants (use 'const' keyword)
const MAX_VALUE: byte = 255
const VIC_BASE: word = $D000
const _SPRITE_COUNT: byte = 8
const _4K: word = 4096

# Valid variables (no 'const' keyword)
counter: byte = 0
myScore: word = 1000
_temp: byte = 5
MaxValue: byte = 100    # Mixed case is now valid for variables

# INVALID - will cause compile error
# _: byte = 0           # Error: Identifier cannot be only underscores
```

**Important:**

- Constants are declared with the `const` keyword
- Constants can be declared at **top-level or inside functions**
- Constants are **immutable** - they cannot be reassigned
- Constants can have any type (byte, word, sbyte, sword, fixed, float, bool, string)

### Arrays

Arrays allow storing multiple values of the same type in contiguous memory.

#### Array Types

| Type      | Element Size | Element Range        | Description           |
| --------- | ------------ | -------------------- | --------------------- |
| `byte[]`  | 1 byte       | 0-255                | Unsigned 8-bit array  |
| `word[]`  | 2 bytes      | 0-65535              | Unsigned 16-bit array |
| `bool[]`  | 1 byte       | true/false           | Boolean array         |
| `sbyte[]` | 1 byte       | -128 to 127          | Signed 8-bit array    |
| `sword[]` | 2 bytes      | -32768 to 32767      | Signed 16-bit array   |
| `fixed[]` | 2 bytes      | -2048.0 to 2047.9375 | Fixed-point array     |
| `float[]` | 2 bytes      | ±65504               | IEEE-754 float array  |

#### Array Declaration

```python
def main():
    # Array with initializer (size inferred from elements)
    data: byte[] = [10, 20, 30, 40, 50]

    # Sized array without initializer
    buffer: byte[100]

    # Word array for larger values
    scores: word[] = [1000, 2500, 5000]

    # Bool array
    flags: bool[] = [true, false, true]

    # Signed arrays (type inferred from negative values)
    temps: sbyte[] = [-10, 0, 25, 50]
    offsets: sword[] = [-1000, 500, 2000]

    # Fixed-point array (for game coordinates, speeds, etc.)
    positions: fixed[] = [0.0, 1.5, 3.25, 5.0]

    # Float array (for scientific calculations)
    values: float[] = [3.14, 2.718, 1.414]
```

#### Array Access

```python
def main():
    arr: byte[] = [10, 20, 30]

    # Read element
    x: byte = arr[0]    # x = 10
    y: byte = arr[2]    # y = 30

    # Write element
    arr[1] = 25         # arr is now [10, 25, 30]

    # Index can be a variable
    i: byte = 1
    z: byte = arr[i]    # z = 25
```

#### Array Type Inference

Array literal types are inferred from element values:

```python
def main():
    # All values 0-255: byte[]
    a: byte[] = [1, 2, 255]

    # Any value > 255: word[]
    b: word[] = [1000, 50, 3]

    # Boolean values: bool[]
    c: bool[] = [true, false]
```

#### Arrays in Loops

```python
def main():
    # Initialize array with loop
    arr: byte[10]
    i: byte = 0
    while i < len(arr):
        arr[i] = i * 2
        i = i + 1

    # Sum array elements
    total: word = 0
    j: byte = 0
    while j < len(arr):
        total = total + arr[j]
        j = j + 1
```

#### Array Function Parameters

Arrays can be passed to functions:

```python
def sum_array(arr: byte[], size: byte) -> word:
    total: word = 0
    i: byte = 0
    while i < size:
        total = total + arr[i]
        i = i + 1
    return total

def main():
    data: byte[] = [10, 20, 30]
    result: word = sum_array(data, 3)
```

#### Array Limitations

- **No nested arrays** - Multi-dimensional arrays not supported
- **No dynamic sizing** - Size must be known at compile time
- **No bounds checking** - Accessing out-of-bounds indices is undefined
- **Maximum size** - Limited by available memory (~38KB)
- **Index type** - Index must be `byte` or `word` (integer types only)
- **len() on parameters** - `len()` only works on local arrays, not function parameters (size unknown)

### Operators

#### Arithmetic Operators

| Operator | Description    | Example | Types                  |
| -------- | -------------- | ------- | ---------------------- |
| `+`      | Addition       | `a + b` | All numeric            |
| `-`      | Subtraction    | `a - b` | All numeric            |
| `*`      | Multiplication | `a * b` | All numeric            |
| `/`      | Division       | `a / b` | All numeric            |
| `%`      | Modulo         | `a % b` | Integer and fixed only |

**Note:** Modulo (`%`) is not supported for `float` type.

#### Comparison Operators

| Operator | Description      | Example  |
| -------- | ---------------- | -------- |
| `==`     | Equal            | `a == b` |
| `!=`     | Not equal        | `a != b` |
| `<`      | Less than        | `a < b`  |
| `>`      | Greater than     | `a > b`  |
| `<=`     | Less or equal    | `a <= b` |
| `>=`     | Greater or equal | `a >= b` |

#### Logical Operators

| Operator | Description | Example    |
| -------- | ----------- | ---------- |
| `and`    | Logical AND | `a and b`  |
| `or`     | Logical OR  | `a or b`   |
| `not`    | Logical NOT | `not flag` |

#### Bitwise Operators

| Operator | Description | Example  |
| -------- | ----------- | -------- |
| `&`      | Bitwise AND | `a & b`  |
| `\|`     | Bitwise OR  | `a \| b` |
| `^`      | Bitwise XOR | `a ^ b`  |
| `~`      | Bitwise NOT | `~a`     |
| `<<`     | Left shift  | `a << 2` |
| `>>`     | Right shift | `a >> 2` |

**Note:** Bitwise operators only work with integer types (`byte`, `sbyte`, `word`, `sword`). They are not supported for `fixed` or `float`.

#### Compound Assignment Operators

Compound assignment operators combine an operation with assignment:

| Operator | Description         | Equivalent   |
| -------- | ------------------- | ------------ |
| `+=`     | Add and assign      | `x = x + y`  |
| `-=`     | Subtract and assign | `x = x - y`  |
| `*=`     | Multiply and assign | `x = x * y`  |
| `/=`     | Divide and assign   | `x = x / y`  |
| `%=`     | Modulo and assign   | `x = x % y`  |
| `&=`     | Bitwise AND assign  | `x = x & y`  |
| `\|=`    | Bitwise OR assign   | `x = x \| y` |
| `^=`     | Bitwise XOR assign  | `x = x ^ y`  |
| `<<=`    | Left shift assign   | `x = x << y` |
| `>>=`    | Right shift assign  | `x = x >> y` |

**Usage:**

```python
def main():
    # Variable compound assignment
    x: byte = 10
    x += 5      # x is now 15
    x *= 2      # x is now 30
    x &= 15     # x is now 14

    # Array element compound assignment
    arr: byte[5]
    arr[0] = 10
    arr[0] += 5     # arr[0] is now 15

    # Word array compound assignment
    scores: word[3]
    scores[0] = 1000
    scores[0] += 500    # scores[0] is now 1500

    # In loops (common pattern)
    sum: word = 0
    i: byte = 1
    while i <= 10:
        sum += i    # Accumulate sum
        i += 1      # Increment counter
```

**Note:** Compound assignment works with all integer types (`byte`, `sbyte`, `word`, `sword`) on both variables and array elements. Bitwise compound operators (`&=`, `|=`, `^=`, `<<=`, `>>=`) only work with integer types.

#### Operator Precedence (highest to lowest)

1. `()` - Parentheses
2. `not`, `~`, unary `-` - Unary operators
3. `*`, `/`, `%` - Multiplication, division
4. `+`, `-` - Addition, subtraction
5. `<<`, `>>` - Bit shifts
6. `&` - Bitwise AND
7. `^` - Bitwise XOR
8. `|` - Bitwise OR
9. `==`, `!=`, `<`, `>`, `<=`, `>=` - Comparisons
10. `and` - Logical AND
11. `or` - Logical OR

### Type Casting

Convert values between types using the type name as a function.

#### Basic Syntax

```python
def main():
    # Integer conversions
    w: word = 1000
    b: byte = byte(w)     # word to byte (truncates to low byte)

    s: sbyte = -50
    w2: word = word(s)    # sbyte to word (sign-extends)

    # Decimal conversions
    f: float = 3.14
    fx: fixed = fixed(f)  # float to fixed
    i: word = word(f)     # float to word (truncates decimal)

    # Boolean conversions
    x: byte = 42
    flag: bool = bool(x)  # true if x != 0
    num: byte = byte(flag) # 1 if true, 0 if false

    # String conversions
    n: word = 12345
    s1: string = str(n)   # "12345"
    s2: string = "99"
    b2: byte = byte(s2)   # 99
```

#### Implicit Conversions (Automatic)

These conversions happen automatically when assigning or in expressions:

| From    | To      | Notes                            |
| ------- | ------- | -------------------------------- |
| `byte`  | `word`  | Zero-extends to 16 bits          |
| `byte`  | `sword` | Zero-extends to 16 bits          |
| `sbyte` | `sword` | Sign-extends to 16 bits          |
| `byte`  | `fixed` | Integer to 12.4 format (value.0) |
| `sbyte` | `fixed` | Integer to 12.4 format (value.0) |
| `byte`  | `float` | Integer to IEEE-754 binary16     |
| `sbyte` | `float` | Integer to IEEE-754 binary16     |
| `fixed` | `float` | Widening conversion              |
| `bool`  | `byte`  | true=1, false=0                  |
| `bool`  | `word`  | true=1, false=0                  |
| `bool`  | `sbyte` | true=1, false=0                  |
| `bool`  | `sword` | true=1, false=0                  |

```python
def main():
    b: byte = 100
    w: word = b           # Implicit: byte → word (zero-extend)

    s: sbyte = -50
    sw: sword = s         # Implicit: sbyte → sword (sign-extend)

    i: byte = 42
    f: fixed = i          # Implicit: byte → fixed (42.0)
    fl: float = f         # Implicit: fixed → float

    flag: bool = true
    n: byte = flag        # Implicit: bool → byte (1)
```

#### Explicit Conversions (Require Cast)

These conversions require explicit `type()` syntax because they may lose data:

| From     | To      | Notes                               |
| -------- | ------- | ----------------------------------- |
| `word`   | `byte`  | Truncates to low 8 bits             |
| `sword`  | `sbyte` | Truncates to low 8 bits             |
| `word`   | `fixed` | May overflow (-2048..2047 range)    |
| `sword`  | `fixed` | May overflow (-2048..2047 range)    |
| `word`   | `float` | May lose precision (>11 bits)       |
| `sword`  | `float` | May lose precision (>11 bits)       |
| `float`  | `fixed` | Range and precision loss            |
| `float`  | `byte`  | Truncates decimal, may overflow     |
| `float`  | `word`  | Truncates decimal, may overflow     |
| `float`  | `sbyte` | Truncates decimal, may overflow     |
| `float`  | `sword` | Truncates decimal, may overflow     |
| `fixed`  | `byte`  | Truncates to integer part           |
| `fixed`  | `word`  | Truncates to integer part           |
| `fixed`  | `sbyte` | Truncates to integer part           |
| `fixed`  | `sword` | Truncates to integer part           |
| `byte`   | `sbyte` | Reinterpret (>127 becomes negative) |
| `sbyte`  | `byte`  | Reinterpret (negative becomes >127) |
| `byte`   | `bool`  | true if != 0                        |
| `word`   | `bool`  | true if != 0                        |
| `string` | `byte`  | Parse string as number              |
| `string` | `word`  | Parse string as number              |

```python
def main():
    # Narrowing: requires explicit cast
    w: word = 1000
    b: byte = byte(w)     # 232 (1000 mod 256)

    # Decimal to integer: truncates
    f: fixed = 3.75
    i: byte = byte(f)     # 3 (integer part only)

    # Integer to bool: any non-zero is true
    x: byte = 42
    flag: bool = bool(x)  # true

    # String parsing
    s: string = "123"
    n: byte = byte(s)     # 123
```

#### String Conversions

The `str()` function converts any numeric type to its string representation:

```python
def main():
    # Numeric to string
    b: byte = 42
    s1: string = str(b)    # "42"

    w: word = 12345
    s2: string = str(w)    # "12345"

    neg: sbyte = -50
    s3: string = str(neg)  # "-50"

    f: fixed = 3.5
    s4: string = str(f)    # "3.5000"

    flag: bool = true
    s5: string = str(flag) # "1"

    # String to numeric (explicit cast)
    input: string = "255"
    val: byte = byte(input) # 255

    big: string = "54321"
    num: word = word(big)   # 54321
```

**String parsing notes:**

- Leading whitespace is skipped
- Parsing stops at first non-digit character
- Invalid strings return 0

#### Bool Conversions

Converting to `bool` tests for non-zero:

```python
def main():
    # Integer to bool
    x: byte = 0
    a: bool = bool(x)      # false

    y: byte = 1
    b: bool = bool(y)      # true

    z: word = 1000
    c: bool = bool(z)      # true

    # Bool to integer
    flag: bool = true
    n: byte = byte(flag)   # 1

    # Bool to word
    w: word = word(flag)   # 1
```

#### Forbidden Conversions

These conversions are not allowed:

| From     | To       | Reason                        |
| -------- | -------- | ----------------------------- |
| `array`  | `scalar` | Fundamentally different types |
| `scalar` | `array`  | Fundamentally different types |
| `byte[]` | `word[]` | Array element type mismatch   |
| `void`   | any      | No value to convert           |

#### Compile-Time Warnings

The compiler emits warnings for potentially dangerous casts:

```python
def main():
    # W001: Literal truncation
    b: byte = byte(300)    # Warning: Value 300 will be truncated to 44

    # W003: Fixed-point overflow
    f: fixed = fixed(5000) # Warning: Value overflows fixed-point range

    # W004: Negative to unsigned
    b2: byte = byte(-10)   # Warning: Negative value -10 will wrap to 246
```

Warnings do not stop compilation but indicate potential issues.

### Control Flow

#### If Statement

```python
def main():
    x: byte = 50

    if x > 100:
        println("LARGE")
    else:
        println("SMALL")
```

#### Elif Branches

```python
def main():
    x: byte = 50

    if x > 100:
        println("LARGE")
    elif x > 50:
        println("MEDIUM")
    elif x > 25:
        println("SMALL")
    else:
        println("TINY")
```

#### While Loop

```python
def main():
    i: byte = 0
    while i < 10:
        println(i)
        i = i + 1
```

#### For Loop

```python
def main():
    # Count up: for variable in start to end
    for i in 0 to 9:
        println(i)

    # Count down: for variable in start downto end
    for i in 10 downto 1:
        println(i)
```

#### Break Statement

```python
def main():
    i: byte = 0
    while true:
        println(i)
        i = i + 1
        if i >= 5:
            break
```

#### Continue Statement

```python
def main():
    for i in 0 to 9:
        if i % 2 == 0:
            continue  # Skip even numbers
        println(i)    # Only prints 1, 3, 5, 7, 9
```

#### Pass Statement

Use `pass` for empty blocks:

```python
def do_nothing():
    pass
```

### Functions

#### Function Definition

```python
def add(a: byte, b: byte) -> byte:
    return a + b

def greet(name: string):
    print("HELLO ")
    println(name)

def main():
    result: byte = add(10, 20)
    greet("PLAYER")
```

#### Return Types

- Functions that return a value must specify `-> type`
- Functions without a return type return nothing (void)

```python
def calculate(x: byte) -> byte:
    return x * 2

def print_result(val: byte):
    println(val)
```

### Comments

Comments start with `#` and continue to the end of the line:

```python
# Top-level comment
# Multiple lines work too

def main():
    # Comment as first line in block
    x: byte = 10  # Inline comment

    # Comments anywhere in the code
    println(x)
```

---

## Built-in Functions

### Screen Functions

#### `cls()`

Clears the screen.

```python
def main():
    cls()
```

#### `cursor(x: byte, y: byte)`

Moves the cursor to position (x, y).

- x: Column (0-39)
- y: Row (0-24)

```python
def main():
    cls()
    cursor(10, 12)
    println("CENTERED")
```

### Output Functions

#### `print(value)`

Prints a value without newline.

```python
def main():
    print("SCORE: ")
    print(100)
```

#### `println(value)`

Prints a value followed by a newline.

```python
def main():
    println("LINE 1")
    println("LINE 2")
```

Both `print` and `println` accept:

- Strings: `println("HELLO")`
- Unsigned numbers: `println(42)` (byte or word)
- Signed numbers: `println(-50)` (sbyte or sword, prints with minus sign)
- Fixed-point: `println(3.75)` (prints as "3.7500")
- Float: `println(3.14)` (prints integer part with ".0", or "INF"/"NAN" for special values)
- Booleans: `println(flag)` (prints "TRUE" or "FALSE")
- Variables: `println(score)`

### Input Functions

#### `get_key() -> byte`

Returns the current key being pressed, or 0 if no key.

```python
def main():
    k: byte = get_key()
    if k != 0:
        println(k)
```

#### `read() -> byte`

Waits for a key press and returns it.

```python
def main():
    println("PRESS ANY KEY")
    k: byte = read()
    println(k)
```

#### `readln() -> string`

Reads a line of text input from the user.

```python
def main():
    print("NAME: ")
    name: string = readln()
    print("HELLO ")
    println(name)
```

### Memory Functions

#### `poke(address: word, value: byte)`

Writes a byte to a memory address.

```python
def main():
    poke($D020, 0)  # Set border to black
    poke($D021, 0)  # Set background to black
```

#### `peek(address: word) -> byte`

Reads a byte from a memory address.

```python
def main():
    border: byte = peek($D020)
    println(border)
```

### Array and String Functions

#### `len(value) -> byte/word`

Returns the length of an array or string.

- For **strings**: returns `byte` (0-255)
- For **arrays**: returns `word` (0-65535)

```python
def main():
    # len() on arrays - returns word
    data: byte[] = [10, 20, 30, 40, 50]
    size: word = len(data)
    println(size)  # Prints: 5

    # len() on strings - returns byte
    name: string = "HELLO"
    str_size: byte = len(name)
    println(str_size)  # Prints: 5

    # Use in loop condition
    i: byte = 0
    while i < len(data):
        println(data[i])
        i = i + 1

    # Works with all array types
    buffer: byte[100]
    println(len(buffer))  # Prints: 100
```

#### `str_at(s: string, i: byte) -> byte`

Returns the character (byte value) at position i in the string.

```python
def main():
    name: string = "HELLO"

    # Get first character
    first: byte = str_at(name, 0)  # 72 ('H' in PETSCII)

    # Get last character
    last: byte = str_at(name, 4)   # 79 ('O' in PETSCII)

    # Print all characters
    i: byte = 0
    while i < len(name):
        println(str_at(name, i))
        i = i + 1
```

### String Concatenation

Strings can be concatenated using the `+` operator:

```python
def main():
    # Concatenate literals
    greeting: string = "HELLO" + " " + "WORLD"
    println(greeting)  # Prints: HELLO WORLD

    # Concatenate variables
    first: string = "HELLO"
    second: string = "WORLD"
    combined: string = first + " " + second
    println(combined)

    # Use in print
    name: string = "PLAYER"
    println("WELCOME, " + name + "!")
```

**Note:** String concatenation uses a temporary buffer. The result is valid until the next concatenation operation.

### Random Number Functions

The random number generator uses a 16-bit Galois LFSR (Linear Feedback Shift Register) with polynomial $0039, seeded at program start from the SID noise register and VIC raster line for unpredictable results.

#### `rand() -> fixed`

Returns a random fixed-point number between 0.0 and 0.9375 (15/16).
The resolution is 1/16 (0.0625), giving 16 possible values: 0.0, 0.0625, 0.125, ..., 0.9375.

```python
def main():
    r: fixed = rand()
    println(r)  # Prints something like 0.5000 or 0.8125
```

#### `rand_byte(from: byte, to: byte) -> byte`

Returns a random byte in the range [from, to] (inclusive).

```python
def main():
    # Simulate dice roll (1-6)
    dice: byte = rand_byte(1, 6)
    println(dice)
```

#### `rand_sbyte(from: sbyte, to: sbyte) -> sbyte`

Returns a random signed byte in the range [from, to] (inclusive).

```python
def main():
    # Random value from -10 to 10
    offset: sbyte = rand_sbyte(-10, 10)
    println(offset)
```

#### `rand_word(from: word, to: word) -> word`

Returns a random 16-bit word in the range [from, to] (inclusive).

```python
def main():
    # Random score between 100 and 1000
    score: word = rand_word(100, 1000)
    println(score)
```

#### `rand_sword(from: sword, to: sword) -> sword`

Returns a random signed 16-bit word in the range [from, to] (inclusive).

```python
def main():
    # Random value from -1000 to 1000
    value: sword = rand_sword(-1000, 1000)
    println(value)
```

#### `seed()`

Reseeds the random number generator from hardware entropy sources (SID noise, CIA timers, raster line). This is useful when:

- The emulator produces the same random sequence on each run
- You want to refresh the random state during program execution

```python
def main():
    # Generate some random numbers
    println(rand_byte(1, 100))
    println(rand_byte(1, 100))

    # Reseed from hardware entropy
    seed()

    # Generate more random numbers with fresh entropy
    println(rand_byte(1, 100))
    println(rand_byte(1, 100))
```

### Sprite Functions

The C64 VIC-II chip provides 8 hardware sprites. Cobra64 offers comprehensive sprite control through built-in functions.

#### Sprite Basics

Each sprite is 24×21 pixels (63 bytes of data + 1 padding = 64 bytes). Sprites can be positioned anywhere on the 320×200 screen, with X coordinates ranging from 0-511 (9-bit).

#### `sprite_enable(num: byte, enable: bool)`

Enables or disables a single sprite.

```python
def main():
    sprite_enable(0, true)   # Enable sprite 0
    sprite_enable(0, false)  # Disable sprite 0
```

#### `sprites_enable(mask: byte)`

Enables sprites using a bitmask (bit 0 = sprite 0, etc.).

```python
def main():
    sprites_enable(3)   # Enable sprites 0 and 1 (binary 00000011)
    sprites_enable(0)   # Disable all sprites
```

#### `sprite_pos(num: byte, x: word, y: byte)`

Sets both X and Y position for a sprite.

```python
def main():
    sprite_pos(0, 160, 100)  # Center of screen
    sprite_pos(0, 320, 200)  # X can be 0-511
```

#### `sprite_x(num: byte, x: word)` / `sprite_y(num: byte, y: byte)`

Sets X or Y position individually.

```python
def main():
    sprite_x(0, 256)  # Set X position (9-bit: 0-511)
    sprite_y(0, 100)  # Set Y position (8-bit: 0-255)
```

#### `sprite_get_x(num: byte) -> word` / `sprite_get_y(num: byte) -> byte`

Returns the current position of a sprite.

```python
def main():
    x: word = sprite_get_x(0)
    y: byte = sprite_get_y(0)
```

#### `sprite_data(num: byte, pointer: byte)`

Sets the sprite data pointer. The pointer value = memory_address / 64.

```python
def main():
    # Sprite data at $3000 = pointer 192 ($3000 / 64 = 192)
    sprite_data(0, 192)
```

#### `sprite_color(num: byte, color: byte)`

Sets the sprite's color (0-15).

```python
def main():
    sprite_color(0, 1)   # White
    sprite_color(1, 2)   # Red
```

#### `sprite_multicolor1(color: byte)` / `sprite_multicolor2(color: byte)`

Sets the shared multicolor palette colors.

```python
def main():
    sprite_multicolor1(5)  # Green - shared color 1
    sprite_multicolor2(6)  # Blue - shared color 2
```

#### `sprite_multicolor(num: byte, enable: bool)`

Enables multicolor mode for a sprite (12×21 pixels, 4 colors).

```python
def main():
    sprite_multicolor(0, true)   # Enable multicolor
    sprite_multicolor(0, false)  # Standard high-res mode
```

#### `sprite_expand_x(num: byte, expand: bool)` / `sprite_expand_y(num: byte, expand: bool)`

Doubles the sprite size in X or Y direction.

```python
def main():
    sprite_expand_x(0, true)  # Double width (48 pixels)
    sprite_expand_y(0, true)  # Double height (42 pixels)
```

#### `sprite_priority(num: byte, behind_bg: bool)`

Sets whether sprite appears behind background graphics.

```python
def main():
    sprite_priority(0, true)   # Sprite behind background
    sprite_priority(0, false)  # Sprite in front (default)
```

#### `sprite_collision_sprite() -> byte`

Reads the sprite-sprite collision register. Returns a bitmask of collided sprites. Reading clears the register.

```python
def main():
    coll: byte = sprite_collision_sprite()
    if coll & 3 == 3:  # Sprites 0 and 1 collided
        println("COLLISION!")
```

#### `sprite_collision_bg() -> byte`

Reads the sprite-background collision register.

```python
def main():
    coll: byte = sprite_collision_bg()
    if coll & 1 != 0:  # Sprite 0 hit background
        println("HIT WALL!")
```

#### `sprite_collides(mask: byte) -> bool`

Checks if any sprite in the mask has a collision.

```python
def main():
    if sprite_collides(1):  # Check sprite 0
        println("SPRITE 0 COLLISION")
```

#### Complete Sprite Example

```python
const SPRITE_PTR: byte = 192  # Data at $3000

def main():
    cls()

    # Setup sprite data pointer
    sprite_data(0, SPRITE_PTR)

    # Set color and position
    sprite_color(0, 1)
    sprite_pos(0, 160, 100)

    # Enable sprite
    sprite_enable(0, true)

    # Animation loop
    x: word = 160
    while get_key() == 0:
        x = x + 1
        if x > 320:
            x = 24
        sprite_x(0, x)

    sprite_enable(0, false)
```

#### Sprite Function Reference

| Function                            | Description                 |
| ----------------------------------- | --------------------------- |
| `sprite_enable(num, enable)`        | Enable/disable sprite       |
| `sprites_enable(mask)`              | Enable by bitmask           |
| `sprite_x(num, x)`                  | Set X position (0-511)      |
| `sprite_y(num, y)`                  | Set Y position (0-255)      |
| `sprite_pos(num, x, y)`             | Set both positions          |
| `sprite_get_x(num) -> word`         | Get X position              |
| `sprite_get_y(num) -> byte`         | Get Y position              |
| `sprite_data(num, ptr)`             | Set data pointer            |
| `sprite_get_data(num) -> byte`      | Get data pointer            |
| `sprite_color(num, color)`          | Set sprite color            |
| `sprite_get_color(num) -> byte`     | Get sprite color            |
| `sprite_multicolor1(color)`         | Set shared color 1          |
| `sprite_multicolor2(color)`         | Set shared color 2          |
| `sprite_get_multicolor1() -> byte`  | Get shared color 1          |
| `sprite_get_multicolor2() -> byte`  | Get shared color 2          |
| `sprite_multicolor(num, enable)`    | Enable multicolor mode      |
| `sprites_multicolor(mask)`          | Multicolor by mask          |
| `sprite_is_multicolor(num) -> bool` | Check multicolor            |
| `sprite_expand_x(num, expand)`      | Enable 2x width             |
| `sprite_expand_y(num, expand)`      | Enable 2x height            |
| `sprites_expand_x(mask)`            | X expand by mask            |
| `sprites_expand_y(mask)`            | Y expand by mask            |
| `sprite_is_expanded_x(num) -> bool` | Check X expansion           |
| `sprite_is_expanded_y(num) -> bool` | Check Y expansion           |
| `sprite_priority(num, behind)`      | Set priority                |
| `sprites_priority(mask)`            | Priority by mask            |
| `sprite_get_priority(num) -> bool`  | Get priority                |
| `sprite_collision_sprite() -> byte` | Sprite-sprite collision     |
| `sprite_collision_bg() -> byte`     | Sprite-background collision |
| `sprite_collides(mask) -> bool`     | Check collision             |

### Sound Functions

The C64 SID (Sound Interface Device) chip provides 3 independent voices with waveform generators, ADSR envelopes, and filters. Cobra64 offers comprehensive sound control through built-in functions.

#### SID Basics

Each voice can produce one of four waveforms (triangle, sawtooth, pulse, noise) at any frequency from ~0.06 Hz to ~4 kHz. The SID also includes a resonant multimode filter that can process any combination of voices.

#### Basic Sound Control

##### `sid_reset()`

Clears all 25 SID registers, silencing all sound.

```python
def main():
    sid_reset()  # Initialize SID to silence
```

##### `sid_volume(vol: byte)`

Sets the master volume (0-15).

```python
def main():
    sid_volume(15)  # Maximum volume
    sid_volume(0)   # Mute
```

##### `sid_frequency(voice: byte, freq: word)`

Sets the 16-bit frequency for a voice (0-2). The frequency value maps to actual Hz using the formula: Hz = freq × 0.0596 (PAL).

```python
def main():
    # Voice 0 at ~440 Hz (A4)
    sid_frequency(0, 7382)
```

##### `sid_waveform(voice: byte, wave: byte)`

Sets the waveform for a voice. Use waveform constants. Preserves the gate bit state.

```python
def main():
    sid_waveform(0, WAVE_PULSE)    # Pulse/square wave
    sid_waveform(1, WAVE_SAWTOOTH) # Sawtooth wave
    sid_waveform(2, WAVE_TRIANGLE) # Triangle wave
```

##### `sid_gate(voice: byte, on: byte)`

Controls the gate bit to start/stop the ADSR envelope.

```python
def main():
    sid_gate(0, 1)  # Start attack phase
    sid_gate(0, 0)  # Start release phase
```

#### ADSR Envelope

The ADSR (Attack, Decay, Sustain, Release) envelope shapes the volume of each note.

##### `sid_attack(voice: byte, val: byte)`

Sets attack time (0-15). Higher values = longer attack.

| Value | Time (ms) | Value | Time (ms) |
| ----- | --------- | ----- | --------- |
| 0     | 2         | 8     | 100       |
| 1     | 8         | 9     | 250       |
| 2     | 16        | 10    | 500       |
| 3     | 24        | 11    | 800       |
| 4     | 38        | 12    | 1000      |
| 5     | 56        | 13    | 3000      |
| 6     | 68        | 14    | 5000      |
| 7     | 80        | 15    | 8000      |

##### `sid_decay(voice: byte, val: byte)`

Sets decay time (0-15). Same timing table as attack.

##### `sid_sustain(voice: byte, val: byte)`

Sets sustain level (0-15). 15 = full volume, 0 = silent.

##### `sid_release(voice: byte, val: byte)`

Sets release time (0-15). Same timing table as attack.

##### `sid_envelope(voice: byte, a: byte, d: byte, s: byte, r: byte)`

Sets all ADSR values at once.

```python
def main():
    # Quick attack, medium decay, half sustain, long release
    sid_envelope(0, 0, 5, 8, 10)
```

#### Pulse Width

##### `sid_pulse_width(voice: byte, width: word)`

Sets the 12-bit pulse width (0-4095) for pulse waveform. 2048 = 50% duty cycle (square wave).

```python
def main():
    sid_waveform(0, WAVE_PULSE)
    sid_pulse_width(0, 2048)  # Square wave
    sid_pulse_width(0, 512)   # 12.5% duty cycle (thin sound)
```

#### Advanced Voice Control

##### `sid_ring_mod(voice: byte, enable: byte)`

Enables ring modulation. Voice N is modulated by voice N-1 (voice 0 by voice 2).

```python
def main():
    sid_ring_mod(0, 1)  # Enable ring mod on voice 0
```

##### `sid_sync(voice: byte, enable: byte)`

Enables hard oscillator sync. Voice N syncs to voice N-1.

```python
def main():
    sid_sync(1, 1)  # Sync voice 1 to voice 0
```

##### `sid_test(voice: byte, enable: byte)`

Controls the test bit (resets oscillator, used for special effects).

```python
def main():
    sid_test(0, 1)  # Hold oscillator at zero
    sid_test(0, 0)  # Release oscillator
```

#### Filter Control

The SID filter is a resonant multimode filter (low-pass, band-pass, high-pass).

##### `sid_filter_cutoff(freq: word)`

Sets the 11-bit filter cutoff frequency (0-2047).

```python
def main():
    sid_filter_cutoff(1024)  # Mid-range cutoff
```

##### `sid_filter_resonance(val: byte)`

Sets filter resonance (0-15). Higher values = more pronounced resonance peak.

```python
def main():
    sid_filter_resonance(8)  # Medium resonance
```

##### `sid_filter_route(voices: byte)`

Routes voices through the filter. Use a bitmask (bit 0 = voice 0, etc.).

```python
def main():
    sid_filter_route(1)  # Filter voice 0 only
    sid_filter_route(7)  # Filter all three voices
```

##### `sid_filter_mode(mode: byte)`

Sets the filter mode. Can combine modes with OR.

```python
def main():
    sid_filter_mode(FILTER_LOWPASS)   # Low-pass only
    sid_filter_mode(FILTER_BANDPASS)  # Band-pass only
    sid_filter_mode(FILTER_LOWPASS | FILTER_HIGHPASS)  # Notch filter
```

#### High-Level Music Functions

##### `play_note(voice: byte, note: byte, octave: byte)`

Plays a musical note using note constants (NOTE_C through NOTE_B) and octave (0-7).

```python
def main():
    sid_volume(15)
    sid_envelope(0, 0, 5, 10, 8)
    sid_waveform(0, WAVE_PULSE)
    sid_pulse_width(0, 2048)

    # Play C major chord
    play_note(0, NOTE_C, 4)  # Middle C
    play_note(1, NOTE_E, 4)  # E4
    play_note(2, NOTE_G, 4)  # G4
```

##### `play_tone(voice: byte, freq: word, wave: byte, duration: byte)`

Plays a tone with automatic gate control. Duration is in frames (~1/60 second).

```python
def main():
    sid_volume(15)
    sid_envelope(0, 0, 2, 8, 4)

    # Play a 440 Hz pulse tone for ~1 second
    play_tone(0, 7382, WAVE_PULSE, 60)
```

##### `sound_off()`

Silences all voices by clearing gate bits and waveforms.

```python
def main():
    sound_off()  # Stop all sound
```

##### `sound_off_voice(voice: byte)`

Silences a specific voice.

```python
def main():
    sound_off_voice(0)  # Silence voice 0 only
```

#### Sound Constants

##### Waveform Constants

| Constant        | Value | Description       |
| --------------- | ----- | ----------------- |
| `WAVE_TRIANGLE` | 16    | Triangle wave     |
| `WAVE_SAWTOOTH` | 32    | Sawtooth wave     |
| `WAVE_PULSE`    | 64    | Pulse/square wave |
| `WAVE_NOISE`    | 128   | White noise       |

##### Filter Mode Constants

| Constant          | Value | Description           |
| ----------------- | ----- | --------------------- |
| `FILTER_LOWPASS`  | 16    | Low-pass filter mode  |
| `FILTER_BANDPASS` | 32    | Band-pass filter mode |
| `FILTER_HIGHPASS` | 64    | High-pass filter mode |

##### Note Constants

| Constant  | Value | Note  |
| --------- | ----- | ----- |
| `NOTE_C`  | 0     | C     |
| `NOTE_CS` | 1     | C#/Db |
| `NOTE_D`  | 2     | D     |
| `NOTE_DS` | 3     | D#/Eb |
| `NOTE_E`  | 4     | E     |
| `NOTE_F`  | 5     | F     |
| `NOTE_FS` | 6     | F#/Gb |
| `NOTE_G`  | 7     | G     |
| `NOTE_GS` | 8     | G#/Ab |
| `NOTE_A`  | 9     | A     |
| `NOTE_AS` | 10    | A#/Bb |
| `NOTE_B`  | 11    | B     |

##### SID Register Address Constants

| Constant               | Address | Description                |
| ---------------------- | ------- | -------------------------- |
| `SID_BASE`             | $D400   | SID chip base address      |
| `SID_VOICE1_FREQ_LO`   | $D400   | Voice 1 frequency low      |
| `SID_VOICE1_FREQ_HI`   | $D401   | Voice 1 frequency high     |
| `SID_VOICE1_PW_LO`     | $D402   | Voice 1 pulse width low    |
| `SID_VOICE1_PW_HI`     | $D403   | Voice 1 pulse width high   |
| `SID_VOICE1_CTRL`      | $D404   | Voice 1 control register   |
| `SID_VOICE1_AD`        | $D405   | Voice 1 attack/decay       |
| `SID_VOICE1_SR`        | $D406   | Voice 1 sustain/release    |
| `SID_VOICE2_FREQ_LO`   | $D407   | Voice 2 frequency low      |
| `SID_VOICE2_FREQ_HI`   | $D408   | Voice 2 frequency high     |
| `SID_VOICE2_PW_LO`     | $D409   | Voice 2 pulse width low    |
| `SID_VOICE2_PW_HI`     | $D40A   | Voice 2 pulse width high   |
| `SID_VOICE2_CTRL`      | $D40B   | Voice 2 control register   |
| `SID_VOICE2_AD`        | $D40C   | Voice 2 attack/decay       |
| `SID_VOICE2_SR`        | $D40D   | Voice 2 sustain/release    |
| `SID_VOICE3_FREQ_LO`   | $D40E   | Voice 3 frequency low      |
| `SID_VOICE3_FREQ_HI`   | $D40F   | Voice 3 frequency high     |
| `SID_VOICE3_PW_LO`     | $D410   | Voice 3 pulse width low    |
| `SID_VOICE3_PW_HI`     | $D411   | Voice 3 pulse width high   |
| `SID_VOICE3_CTRL`      | $D412   | Voice 3 control register   |
| `SID_VOICE3_AD`        | $D413   | Voice 3 attack/decay       |
| `SID_VOICE3_SR`        | $D414   | Voice 3 sustain/release    |
| `SID_FILTER_CUTOFF_LO` | $D415   | Filter cutoff low (3 bits) |
| `SID_FILTER_CUTOFF_HI` | $D416   | Filter cutoff high         |
| `SID_FILTER_CTRL`      | $D417   | Filter resonance/routing   |
| `SID_VOLUME`           | $D418   | Volume and filter mode     |

#### Complete Sound Example

```python
# Sound demo - plays a simple melody

def main():
    cls()
    println("SOUND DEMO")

    # Initialize SID
    sid_reset()
    sid_volume(15)

    # Setup voice 0 for melody
    sid_envelope(0, 0, 5, 10, 8)
    sid_waveform(0, WAVE_PULSE)
    sid_pulse_width(0, 2048)

    # Play C major scale
    notes: byte[] = [NOTE_C, NOTE_D, NOTE_E, NOTE_F, NOTE_G, NOTE_A, NOTE_B]
    i: byte = 0
    while i < 7:
        play_note(0, notes[i], 4)
        delay(15)  # Wait ~250ms between notes
        i = i + 1

    # Play final high C
    play_note(0, NOTE_C, 5)
    delay(30)

    sound_off()
    println("DONE")

def delay(frames: byte):
    # Simple delay loop
    f: byte = 0
    while f < frames:
        # Wait for raster to reach bottom of screen
        while peek($D012) < 255:
            pass
        while peek($D012) == 255:
            pass
        f = f + 1
```

#### Sound Effects Example

```python
# Game sound effects

def laser_sound():
    sid_envelope(0, 0, 0, 15, 4)
    sid_waveform(0, WAVE_NOISE)
    sid_frequency(0, 8000)
    sid_gate(0, 1)
    # Pitch sweep down
    freq: word = 8000
    while freq > 500:
        sid_frequency(0, freq)
        freq = freq - 200
    sid_gate(0, 0)

def explosion_sound():
    sid_envelope(1, 0, 8, 0, 12)
    sid_waveform(1, WAVE_NOISE)
    sid_frequency(1, 500)
    sid_gate(1, 1)

def coin_sound():
    sid_envelope(2, 0, 2, 0, 4)
    sid_waveform(2, WAVE_TRIANGLE)
    play_note(2, NOTE_E, 6)
    delay(5)
    play_note(2, NOTE_A, 6)

def main():
    sid_reset()
    sid_volume(15)

    println("PRESS KEYS FOR SOUNDS:")
    println("L = LASER")
    println("E = EXPLOSION")
    println("C = COIN")

    while true:
        k: byte = get_key()
        if k == 'L':
            laser_sound()
        elif k == 'E':
            explosion_sound()
        elif k == 'C':
            coin_sound()
```

#### Sound Function Reference

| Function                            | Description                   |
| ----------------------------------- | ----------------------------- |
| `sid_reset()`                       | Clear all SID registers       |
| `sid_volume(vol)`                   | Set master volume (0-15)      |
| `sid_frequency(voice, freq)`        | Set 16-bit frequency          |
| `sid_waveform(voice, wave)`         | Set waveform (preserves gate) |
| `sid_gate(voice, on)`               | Control gate bit              |
| `sid_attack(voice, val)`            | Set attack time (0-15)        |
| `sid_decay(voice, val)`             | Set decay time (0-15)         |
| `sid_sustain(voice, val)`           | Set sustain level (0-15)      |
| `sid_release(voice, val)`           | Set release time (0-15)       |
| `sid_envelope(voice, a, d, s, r)`   | Set full ADSR envelope        |
| `sid_pulse_width(voice, width)`     | Set 12-bit pulse width        |
| `sid_ring_mod(voice, enable)`       | Enable ring modulation        |
| `sid_sync(voice, enable)`           | Enable oscillator sync        |
| `sid_test(voice, enable)`           | Control test bit              |
| `sid_filter_cutoff(freq)`           | Set 11-bit filter cutoff      |
| `sid_filter_resonance(val)`         | Set resonance (0-15)          |
| `sid_filter_route(voices)`          | Route voices through filter   |
| `sid_filter_mode(mode)`             | Set filter mode (LP/BP/HP)    |
| `play_note(voice, note, octave)`    | Play musical note             |
| `play_tone(voice, freq, wave, dur)` | Play tone with duration       |
| `sound_off()`                       | Silence all voices            |
| `sound_off_voice(voice)`            | Silence specific voice        |

---

### Graphics Mode Support

Cobra64 provides comprehensive VIC-II graphics mode support including bitmap graphics, color control, hardware scrolling, and raster functions.

#### VIC-II Register Constants

| Constant         | Value   | Description                    |
| ---------------- | ------- | ------------------------------ |
| `VIC_CONTROL1`   | $D011   | Control register 1             |
| `VIC_CONTROL2`   | $D016   | Control register 2             |
| `VIC_MEMORY`     | $D018   | Memory control register        |
| `VIC_RASTER`     | $D012   | Raster line register           |
| `VIC_BORDER`     | $D020   | Border color register          |
| `VIC_BACKGROUND` | $D021   | Background color register      |
| `VIC_BACKGROUND1`| $D022   | Extra background 1             |
| `VIC_BACKGROUND2`| $D023   | Extra background 2             |
| `VIC_BACKGROUND3`| $D024   | Extra background 3             |
| `COLOR_RAM`      | $D800   | Color RAM start address        |

#### Graphics Mode Constants

| Constant       | Value | Description                        |
| -------------- | ----- | ---------------------------------- |
| `GFX_TEXT`     | 0     | Standard character mode (40x25)    |
| `GFX_TEXT_MC`  | 1     | Multicolor character mode          |
| `GFX_BITMAP`   | 2     | Standard bitmap mode (320x200)     |
| `GFX_BITMAP_MC`| 3     | Multicolor bitmap mode (160x200)   |
| `GFX_TEXT_ECM` | 4     | Extended background color mode     |

#### VIC Bank Constants

| Constant    | Value | Address Range     |
| ----------- | ----- | ----------------- |
| `VIC_BANK0` | 0     | $0000-$3FFF       |
| `VIC_BANK1` | 1     | $4000-$7FFF       |
| `VIC_BANK2` | 2     | $8000-$BFFF       |
| `VIC_BANK3` | 3     | $C000-$FFFF       |

#### Raster Constants

| Constant          | Value | Description                   |
| ----------------- | ----- | ----------------------------- |
| `RASTER_TOP`      | 50    | First visible raster line     |
| `RASTER_BOTTOM`   | 250   | Last visible raster line      |
| `RASTER_MAX_PAL`  | 311   | Maximum raster line (PAL)     |
| `RASTER_MAX_NTSC` | 261   | Maximum raster line (NTSC)    |

#### Display Control Functions

```python
# Set border and background colors
border_color(5)       # Set border to green
background_color(0)   # Set background to black

# Read current colors
c: byte = get_border_color()
c: byte = get_background_color()
```

| Function                  | Description                     |
| ------------------------- | ------------------------------- |
| `border_color(color)`     | Set border color (0-15)         |
| `background_color(color)` | Set background color (0-15)     |
| `get_border_color()`      | Get current border color        |
| `get_background_color()`  | Get current background color    |

#### Mode Switching Functions

```python
# Switch to hires bitmap mode
gfx_mode(GFX_BITMAP)
# Or use shortcut
gfx_hires()

# Switch back to text mode
gfx_text()

# Set screen dimensions
screen_columns(38)    # 38-column mode (with border)
screen_rows(24)       # 24-row mode (with border)
```

| Function                  | Description                        |
| ------------------------- | ---------------------------------- |
| `gfx_mode(mode)`          | Switch graphics mode (0-4)         |
| `get_gfx_mode()`          | Get current graphics mode          |
| `gfx_text()`              | Switch to standard text mode       |
| `gfx_hires()`             | Switch to hires bitmap (320x200)   |
| `gfx_multicolor()`        | Switch to multicolor bitmap        |
| `screen_columns(cols)`    | Set 38 or 40 column mode           |
| `screen_rows(rows)`       | Set 24 or 25 row mode              |

#### Memory Configuration Functions

```python
# Set VIC bank 1 ($4000-$7FFF)
vic_bank(VIC_BANK1)

# Set screen RAM address (relative to bank)
screen_address($0400)

# Set bitmap address (relative to bank)
bitmap_address($2000)

# Set custom character set address
charset_address($0800)
```

| Function                  | Description                        |
| ------------------------- | ---------------------------------- |
| `vic_bank(bank)`          | Set VIC memory bank (0-3)          |
| `get_vic_bank()`          | Get current VIC bank               |
| `screen_address(addr)`    | Set screen RAM address             |
| `bitmap_address(addr)`    | Set bitmap address                 |
| `charset_address(addr)`   | Set character set address          |

#### Bitmap Pixel Operations

```python
# Initialize bitmap mode
gfx_hires()
bitmap_clear()

# Draw pixels in hires mode (320x200)
plot(160, 100)        # Set pixel at center
unplot(160, 100)      # Clear pixel

# Test if pixel is set
if point(160, 100):
    println("PIXEL SET")

# Multicolor mode (160x200, 4 colors per cell)
gfx_multicolor()
plot_mc(80, 100, 2)   # Set pixel with color 2
c: byte = point_mc(80, 100)  # Get pixel color (0-3)
```

| Function                      | Description                        |
| ----------------------------- | ---------------------------------- |
| `plot(x, y)`                  | Set pixel in hires mode            |
| `unplot(x, y)`                | Clear pixel in hires mode          |
| `point(x, y) -> bool`         | Test if pixel is set               |
| `plot_mc(x, y, color)`        | Set multicolor pixel (0-3)         |
| `point_mc(x, y) -> byte`      | Get multicolor pixel value         |
| `bitmap_clear()`              | Clear entire bitmap                |
| `bitmap_fill(pattern)`        | Fill bitmap with pattern           |

#### Drawing Primitives

```python
# Draw lines
line(0, 0, 319, 199)      # Diagonal line
hline(0, 100, 320)        # Fast horizontal line
vline(160, 0, 200)        # Fast vertical line

# Draw rectangles
rect(10, 10, 100, 80)     # Rectangle outline
rect_fill(120, 10, 100, 80)  # Filled rectangle
```

| Function                           | Description                     |
| ---------------------------------- | ------------------------------- |
| `line(x1, y1, x2, y2)`             | Draw line (Bresenham)           |
| `hline(x, y, length)`              | Fast horizontal line            |
| `vline(x, y, length)`              | Fast vertical line              |
| `rect(x, y, width, height)`        | Draw rectangle outline          |
| `rect_fill(x, y, width, height)`   | Draw filled rectangle           |

#### Cell Color Control

In bitmap modes, each 8x8 pixel cell has foreground and background colors:

```python
# Set colors for cell at column 20, row 12
cell_color(20, 12, 1, 0)    # White foreground, black background

# Get cell colors (fg in high nibble, bg in low)
colors: byte = get_cell_color(20, 12)

# Work with color RAM directly
color_ram(20, 12, 5)        # Set color RAM value
c: byte = get_color_ram(20, 12)

# Fill all cells with same colors
fill_colors(1, 0)           # All cells: white on black
fill_color_ram(1)           # Fill color RAM with 1
```

| Function                        | Description                        |
| ------------------------------- | ---------------------------------- |
| `cell_color(cx, cy, fg, bg)`    | Set cell foreground/background     |
| `get_cell_color(cx, cy)`        | Get cell colors (fg<<4 | bg)       |
| `color_ram(cx, cy, color)`      | Set color RAM at cell position     |
| `get_color_ram(cx, cy)`         | Get color RAM value                |
| `fill_colors(fg, bg)`           | Fill all cells with colors         |
| `fill_color_ram(color)`         | Fill color RAM with value          |

#### Hardware Scrolling

```python
# Set scroll offsets (0-7 pixels)
scroll_x(4)    # Horizontal scroll
scroll_y(2)    # Vertical scroll

# Get current scroll values
sx: byte = get_scroll_x()
sy: byte = get_scroll_y()

# Smooth scrolling example
while true:
    i: byte = 0
    while i < 8:
        scroll_x(i)
        wait_raster(250)
        i = i + 1
```

| Function          | Description                        |
| ----------------- | ---------------------------------- |
| `scroll_x(offset)`| Set horizontal scroll (0-7)        |
| `scroll_y(offset)`| Set vertical scroll (0-7)          |
| `get_scroll_x()`  | Get current horizontal scroll      |
| `get_scroll_y()`  | Get current vertical scroll        |

#### Raster Functions

```python
# Get current raster line (0-311 PAL)
r: word = raster()

# Wait for specific raster line (for timing)
wait_raster(250)   # Wait for line 250

# Raster bar effect
while true:
    wait_raster(100)
    border_color(2)
    wait_raster(150)
    border_color(0)
```

| Function              | Description                       |
| --------------------- | --------------------------------- |
| `raster() -> word`    | Get current raster line (0-311)   |
| `wait_raster(line)`   | Wait until raster reaches line    |

#### Extended Background Color Mode (ECM)

ECM mode allows 4 different background colors selected per character:

```python
# Set ECM backgrounds
ecm_background(0, 0)    # Background 0 = black
ecm_background(1, 2)    # Background 1 = red
ecm_background(2, 5)    # Background 2 = green
ecm_background(3, 6)    # Background 3 = blue

# Get ECM background colors
c: byte = get_ecm_background(1)  # Get background 1

# Switch to ECM mode
gfx_mode(GFX_TEXT_ECM)
```

Note: In ECM mode, only characters 0-63 are available. Bits 6-7 of the character code select which background color (0-3) to use.

| Function                         | Description                      |
| -------------------------------- | -------------------------------- |
| `ecm_background(index, color)`   | Set ECM background (0-3)         |
| `get_ecm_background(index)`      | Get ECM background color         |

#### Graphics Mode Bits Reference

| Mode | ECM | BMM | MCM | Name                    | Resolution  |
| ---- | --- | --- | --- | ----------------------- | ----------- |
| 0    | 0   | 0   | 0   | Standard Text           | 40x25 chars |
| 1    | 0   | 0   | 1   | Multicolor Text         | 40x25 chars |
| 2    | 0   | 1   | 0   | Standard Bitmap (Hires) | 320x200 px  |
| 3    | 0   | 1   | 1   | Multicolor Bitmap       | 160x200 px  |
| 4    | 1   | 0   | 0   | Extended Color Text     | 40x25 chars |

- **ECM** = Bit 6 of $D011 (Extended Color Mode)
- **BMM** = Bit 5 of $D011 (Bitmap Mode)
- **MCM** = Bit 4 of $D016 (Multicolor Mode)

#### Default Memory Layout

| Address Range | Size   | Content                    |
| ------------- | ------ | -------------------------- |
| $0400-$07FF   | 1 KB   | Screen RAM (text/colors)   |
| $2000-$3FFF   | 8 KB   | Bitmap data                |
| $D800-$DBFF   | 1 KB   | Color RAM                  |

---

## Example Programs

### Counter

```python
def main():
    cls()
    println("COUNTING:")

    i: byte = 1
    while i <= 10:
        println(i)
        i = i + 1

    println("DONE")
```

### Calculator

```python
def add(a: byte, b: byte) -> byte:
    return a + b

def multiply(a: byte, b: byte) -> byte:
    return a * b

def main():
    cls()
    x: byte = 5
    y: byte = 3

    print("5 + 3 = ")
    println(add(x, y))

    print("5 * 3 = ")
    println(multiply(x, y))
```

### Color Cycler

```python
# Constants use 'const' keyword
const BORDER: word = $D020
const BACKGROUND: word = $D021

def main():
    cls()
    println("PRESS ANY KEY")
    println("TO CYCLE COLORS")

    color: byte = 0
    while true:
        k: byte = read()
        color = color + 1
        if color > 15:
            color = 0
        poke(BORDER, color)
        poke(BACKGROUND, color)
```

### Number Guessing Game

```python
def main():
    cls()
    println("GUESS THE NUMBER")
    println("BETWEEN 1 AND 10")

    secret: byte = 7
    guesses: byte = 0

    while true:
        print("YOUR GUESS: ")
        k: byte = read()
        guess: byte = k - 48  # Convert ASCII to number
        println(guess)
        guesses = guesses + 1

        if guess == secret:
            print("CORRECT IN ")
            print(guesses)
            println(" TRIES!")
            break
        else:
            if guess < secret:
                println("TOO LOW")
            else:
                println("TOO HIGH")
```

### Temperature Converter (Signed Types)

```python
# Example using signed types for temperature values

def main():
    cls()
    println("TEMPERATURE VALUES")
    println("------------------")

    # Signed byte for temperatures that can be negative
    freezing: sbyte = 0
    cold: sbyte = -10
    very_cold: sbyte = -40

    print("FREEZING: ")
    println(freezing)

    print("COLD: ")
    println(cold)

    print("VERY COLD: ")
    println(very_cold)

    # Signed comparisons work correctly
    if cold < freezing:
        println("COLD IS BELOW FREEZING")

    # Counting from negative to positive
    temp: sbyte = -5
    println("COUNTING UP:")
    while temp <= 5:
        println(temp)
        temp = temp + 1
```

### Signed Arithmetic

```python
# Example demonstrating signed arithmetic operations

def main():
    cls()

    a: sbyte = -50
    b: sbyte = 30

    # Addition: -50 + 30 = -20
    print("-50 + 30 = ")
    println(a + b)

    # Subtraction: -50 - 30 = -80
    print("-50 - 30 = ")
    println(a - b)

    # Multiplication: -5 * 10 = -50
    c: sbyte = -5
    d: sbyte = 10
    print("-5 * 10 = ")
    println(c * d)

    # Division: -50 / 10 = -5
    print("-50 / 10 = ")
    println(a / d)

    # 16-bit signed operations
    big: sword = -1000
    small: sword = 500
    print("-1000 + 500 = ")
    println(big + small)
```

### Fixed-Point Smooth Movement

```python
# Example using fixed-point for smooth sprite movement

const SCREEN_WIDTH: word = 320

def main():
    cls()
    println("SMOOTH MOVEMENT DEMO")

    # Position with subpixel precision
    x: fixed = 0.0
    speed: fixed = 0.5

    # Move across screen smoothly
    while x < 100.0:
        # Get integer part for display
        screen_x: word = word(x)
        print("X = ")
        println(screen_x)

        # Smooth movement
        x = x + speed

    println("DONE")
```

### Float Calculations

```python
# Example using float for scientific calculations

def main():
    cls()
    println("FLOAT CALCULATIONS")
    println("------------------")

    # Basic float arithmetic
    a: float = 3.14159
    b: float = 2.71828
    println("PI + E:")
    println(a + b)

    # Large numbers
    big: float = 50000.0
    small: float = 0.001
    println("50000 * 0.001:")
    println(big * small)

    # Scientific notation
    sci: float = 1.5e3
    print("1.5e3 = ")
    println(sci)

    # Negative values
    neg: float = -42.5
    println("NEGATIVE:")
    println(neg)
```

### Mixed Type Operations

```python
# Example showing type promotions between int, fixed, and float

def main():
    cls()
    println("MIXED TYPES")

    # Integer + Fixed = Fixed
    i: byte = 10
    f: fixed = 2.5
    result1: fixed = i + f
    print("10 + 2.5 = ")
    println(result1)

    # Fixed + Float = Float
    fx: fixed = 5.25
    fl: float = 2.75
    result2: float = fx + fl
    print("5.25 + 2.75 = ")
    println(result2)

    # Type conversions
    big_float: float = 1234.5
    as_fixed: fixed = fixed(big_float)
    as_int: word = word(big_float)
    print("FLOAT TO FIXED: ")
    println(as_fixed)
    print("FLOAT TO INT: ")
    println(as_int)
```

### Array Operations

```python
# Example demonstrating array usage

def main():
    cls()
    println("ARRAY DEMO")

    # Byte array with values
    data: byte[] = [10, 20, 30, 40, 50]

    # Sum all elements
    total: word = 0
    i: byte = 0
    while i < 5:
        total = total + data[i]
        i = i + 1

    # Word array for high scores
    scores: word[5]
    scores[0] = 9500
    scores[1] = 7200
    scores[2] = 5800
    scores[3] = 4100
    scores[4] = 2500

    # Find if score qualifies
    new_score: word = 6000
    pos: byte = find_rank(scores, new_score)

def find_rank(scores: word[], new_score: word) -> byte:
    i: byte = 0
    while i < 5:
        if new_score > scores[i]:
            return i
        i = i + 1
    return 5
```

---

## Error Messages

### Lexer Errors

| Error                        | Description                              |
| ---------------------------- | ---------------------------------------- |
| `Invalid character`          | Source contains an unsupported character |
| `Unterminated string`        | String literal missing closing quote     |
| `Invalid escape sequence`    | Unknown escape like `\x`                 |
| `Number overflow`            | Number too large for type                |
| `Tabs not allowed`           | Use 4 spaces for indentation             |
| `Identifier only underscore` | Identifier cannot be just `_` or `__`    |

### Parser Errors

| Error              | Description                      |
| ------------------ | -------------------------------- |
| `Expected ':'`     | Missing colon after if/while/def |
| `Expected '('`     | Missing parenthesis in function  |
| `Unexpected token` | Syntax error in expression       |

### Semantic Errors

| Error                  | Description                                     |
| ---------------------- | ----------------------------------------------- |
| `Undefined variable`   | Variable used before declaration                |
| `Undefined function`   | Function called but not defined                 |
| `Type mismatch`        | Incompatible types in operation                 |
| `Duplicate definition` | Variable or function already exists             |
| `No main() function`   | Entry point missing                             |
| `Break outside loop`   | Break used outside while loop                   |
| `Value out of range`   | Value exceeds type bounds (e.g., 128 for sbyte) |

---

## Limitations

### Known Issues

1. ~~**elif has a code generation bug**~~ - **FIXED in v0.8.0**: The elif statement now works correctly. The bug was in label management during code generation.

2. ~~**Deep nesting limit**~~ - **FIXED in v0.7.0**: The compiler now automatically uses "branch trampolining" to handle deep nesting. When a branch target exceeds the 6510's 127-byte limit, the compiler inverts the condition and uses a JMP instruction instead.

**Note:** All major known issues have been resolved. Please report any new issues at the project repository.

### Platform Constraints

- **Memory:** Programs must fit in available RAM (~38KB typically)
- **Stack:** 256 bytes for the 6510 stack
- **Strings:** Limited by available memory
- **Numbers:**
  - Unsigned: byte (0-255), word (0-65535)
  - Signed: sbyte (-128 to 127), sword (-32768 to 32767)

### Not Supported

- Multi-dimensional arrays
- Structs/records
- Pointers
- Inline assembly

---

## File Format Reference

### PRG Files

PRG files are the standard C64 program format:

- First 2 bytes: Load address (little-endian, typically $0801)
- Remaining bytes: Program data

### D64 Disk Images

D64 files are virtual floppy disk images:

- 35 tracks with variable sectors
- 174,848 bytes total
- Compatible with VICE and other emulators

---

## Running in Emulators

### VICE

```bash
# Run PRG directly
x64sc program.prg

# Mount D64 and load
x64sc -attach8 program.d64
# Then type: LOAD "*",8,1 and RUN
```

### Other Emulators

The generated PRG and D64 files should work with any C64 emulator that supports standard formats.

---

## Version History

- **0.13.0** - Sound & Music Functions
  - Added comprehensive SID chip support with 20 new built-in functions
  - Basic sound: `sid_reset()`, `sid_volume()`, `sid_frequency()`, `sid_waveform()`, `sid_gate()`
  - ADSR envelope: `sid_attack()`, `sid_decay()`, `sid_sustain()`, `sid_release()`, `sid_envelope()`
  - Pulse width: `sid_pulse_width()`
  - Advanced: `sid_ring_mod()`, `sid_sync()`, `sid_test()`
  - Filter: `sid_filter_cutoff()`, `sid_filter_resonance()`, `sid_filter_route()`, `sid_filter_mode()`
  - High-level: `play_note()`, `play_tone()`, `sound_off()`, `sound_off_voice()`
  - Added 45+ sound constants (waveforms, filters, notes, SID registers)
  - Built-in note frequency lookup table with octave scaling (0-7)
  - New example programs: `sound_demo.cb64`, `sound_basic.cb64`, `sound_effects.cb64`

- **0.12.0** - LSP Documentation Examples
  - Added code examples (1-3 per item) for all built-in functions in VS Code extension
  - Added code examples for all built-in constants (COLOR*\*, VIC_SPRITE*\*)
  - Examples include beginner-friendly comments explaining C64-specific concepts
  - Examples shown in hover documentation and auto-completion
  - Built-in constants now appear in auto-completion suggestions
  - Improved documentation for VIC-II registers, sprite handling, and memory addresses

- **0.11.2** - Type Casting System Improvements
  - **Fixed `sbyte → fixed` conversion bug**: Negative sbyte values now correctly convert to fixed-point
  - **Removed dangerous implicit casts**: `byte ↔ sbyte` and `word/sword → fixed` now require explicit casts
  - **Added bool conversions**: `bool()` cast function and implicit `bool → integer`
  - **Added compile-time warnings**: W001-W005 for truncation, overflow, precision loss
  - **Added `fixed[]` and `float[]` array types**: Full array support for decimal types
  - **Added string conversions**: `str()` for numeric-to-string, `byte()`/`word()` for string-to-numeric
  - **Improved DecimalLiteral handling**: Context-aware type inference in binary operations
  - New conformance tests: `30_bool_cast.cb64`, `31_decimal_literal.cb64`, `32_fixed_array.cb64`, `33_float_array.cb64`, `34_string_cast.cb64`

- **0.10.0** - String operations
  - Extended `len()` to work on strings (returns byte for strings, word for arrays)
  - Added `str_at(s: string, i: byte) -> byte` to get character at index
  - Added `+` operator for string concatenation
  - String concatenation uses buffer at $C200
  - Maximum string length: 255 characters

- **0.9.0** - Compound assignment operators
  - Added all 10 compound assignment operators: `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
  - Compound assignment works on both variables and array elements
  - Supports all integer types: `byte`, `sbyte`, `word`, `sword`
  - Byte array element compound assignment with proper index preservation
  - Word array element compound assignment with 16-bit operations
  - New example program: `examples/compound_assignment.cb64`

- **0.8.0** - elif bug fix
  - **BUG FIX:** elif statements now work correctly
  - Fixed label management in `generate_if()` for elif chains
  - The bug caused "Undefined label 'elif_N'" errors when using multiple elif branches
  - Root cause: `next_label` was created but never defined for subsequent branches
  - Solution: Track `current_label` that gets updated after each elif branch

- **0.7.0** - Explicit `const` keyword and branch trampolining
  - **BREAKING CHANGE:** Constants are now declared with `const` keyword
  - Old syntax `MAX_VALUE: byte = 255` no longer creates a constant
  - New syntax `const MAX_VALUE: byte = 255` is required
  - Constants can now be declared inside functions (local constants)
  - Removed naming convention rules (UPPERCASE no longer means constant)
  - Mixed case identifiers like `MyValue` are now valid for variables
  - Removed error code E026 (InvalidIdentifierNaming)
  - **Branch trampolining**: Deep nesting no longer causes compile errors
  - Compiler automatically converts far branches to JMP instructions

- **0.6.0** - Explicit type annotations required
  - **BREAKING CHANGE:** Type inference for declarations has been removed
  - All variable declarations now require explicit type annotations
  - All constant declarations now require explicit type annotations
  - Old syntax `x = 10` must be updated to `x: byte = 10`
  - Old syntax `MAX = 255` must be updated to `MAX: byte = 255`
  - New error code E147 (MissingTypeAnnotation) for declarations without types

- **0.5.0** - Random number generation improvements
  - Added `seed()` function to reseed PRNG from hardware entropy
  - Changed `rand()` to return `fixed` type (was `float`)
  - `rand()` now returns values 0.0 to 0.9375 with 1/16 resolution
  - Fixed `__print_fixed` routine corruption bug
  - Improved fixed-point multiplication for fractional display

- **0.4.0** - Array support
  - Added `byte[]`, `word[]`, `bool[]` array types
  - Added `sbyte[]`, `sword[]` signed array types
  - Array literals with type inference: `[1, 2, 3]`, `[-10, 20, 30]`
  - Sized array declarations: `buffer: byte[100]`
  - Array element read/write: `arr[i]`, `arr[i] = value`
  - Built-in `len(array)` function returns array length
  - Arrays as function parameters
  - Zero-initialization optimization for arrays
  - Signed array type inference from negative values
  - Array-specific error messages

- **0.3.0** - Naming convention for constants _(superseded by 0.7.0)_
  - Removed `const` keyword (reintroduced in 0.7.0)
  - Constants were identified by UPPERCASE naming convention
  - First letter uppercase + all letters uppercase = constant
  - First letter lowercase = variable
  - Added compile-time validation for naming rules
  - New error codes: E026 (InvalidIdentifierNaming, removed in 0.7.0), E027 (IdentifierOnlyUnderscore)

- **0.2.0** - Decimal number types
  - Added `fixed` type (12.4 fixed-point, -2048.0 to +2047.9375)
  - Added `float` type (IEEE-754 binary16, ±65504)
  - Decimal literal support (3.14, 0.5, -2.25)
  - Scientific notation for floats (1.5e3, 2.0e-5)
  - Full arithmetic operations for both types
  - Type conversions between int, fixed, and float
  - Automatic type promotion in mixed operations
  - Print support for fixed and float values

- **0.1.0** - Signed integer types
  - Added `sbyte` type (-128 to 127)
  - Added `sword` type (-32768 to 32767)
  - Full signed arithmetic (add, sub, mul, div)
  - Full signed comparisons (<, >, <=, >=, ==, !=)
  - Negative number literals (-128, -$7F, -%01111111)
  - Compile-time range validation

- **0.0.1** - Initial release
  - Complete language implementation
  - PRG and D64 output
  - All basic built-in functions

---

## Compiler Architecture

The Cobra64 compiler is written in Rust and organized into focused modules following best practices for maintainability. The codebase uses the **Extension Trait Pattern** to split large modules into smaller, cohesive files while maintaining a unified API.

### Module Overview

```
src/
├── lib.rs              # Public API entry point
├── error.rs            # Error types and spans
├── ast/                # Abstract Syntax Tree
│   ├── mod.rs          # Re-exports
│   ├── expr.rs         # Expression nodes
│   ├── stmt.rs         # Statement nodes
│   └── types.rs        # Type definitions
├── lexer/              # Tokenization
│   ├── mod.rs          # Main lexer logic
│   ├── tokens.rs       # Token definitions
│   ├── helpers.rs      # Character navigation
│   ├── numbers.rs      # Number literal scanning
│   ├── strings.rs      # String/char scanning
│   ├── identifiers.rs  # Identifier/keyword scanning
│   ├── operators.rs    # Operator scanning
│   └── indentation.rs  # INDENT/DEDENT handling
├── parser/             # Syntax Analysis
│   ├── mod.rs          # Entry point, parse()
│   ├── blocks.rs       # Block/function parsing (BlockParser trait)
│   ├── control_flow.rs # If/while/for parsing (ControlFlowParser trait)
│   ├── expressions.rs  # Expression parsing (ExpressionParser trait)
│   ├── helpers.rs      # Token navigation (ParserHelpers trait)
│   ├── statements.rs   # Statement parsing (StatementParser trait)
│   └── types.rs        # Type parsing (TypeParser trait)
├── analyzer/           # Semantic Analysis
│   ├── mod.rs          # Entry point, analyze()
│   ├── builtins.rs     # Built-in functions (BuiltinRegistry trait)
│   ├── context.rs      # Analysis context state
│   ├── control_flow.rs # Control flow analysis (ControlFlowAnalyzer trait)
│   ├── expressions.rs  # Expression analysis (ExpressionAnalyzer trait)
│   ├── functions.rs    # Function analysis (FunctionAnalyzer trait)
│   ├── operators.rs    # Operator checking (OperatorChecker trait)
│   ├── scope.rs        # Scope management
│   ├── statements.rs   # Statement analysis (StatementAnalyzer trait)
│   ├── symbol.rs       # Symbol definitions
│   ├── symbol_table.rs # Symbol table with nested scopes
│   └── type_check.rs   # Type inference/checking (TypeChecker trait)
└── codegen/            # Code Generation
    ├── mod.rs          # Entry point, generate()
    ├── assignments.rs  # Assignment generation (AssignmentEmitter trait)
    ├── binary_ops.rs   # Binary operations (BinaryOpsEmitter trait)
    ├── comparisons.rs  # Comparison helpers
    ├── constants.rs    # Memory layout constants
    ├── control_flow.rs # Control flow generation (ControlFlowEmitter trait)
    ├── conversions.rs  # Type conversions (TypeConversions trait)
    ├── declarations.rs # Declaration generation (DeclarationEmitter trait)
    ├── emit.rs         # Byte emission helpers
    ├── expressions.rs  # Expression generation (ExpressionEmitter trait)
    ├── float_runtime.rs  # Float operations runtime
    ├── string_runtime.rs # String conversion routines (str(), parse)
    ├── functions.rs    # Function call generation (FunctionCallEmitter trait)
    ├── labels.rs       # Label management
    ├── mos6510.rs      # 6510 instruction encoding
    ├── runtime.rs      # Runtime library
    ├── strings.rs      # String handling
    ├── type_inference.rs # Type inference (TypeInference trait)
    ├── types.rs        # Type conversions
    ├── unary_ops.rs    # Unary operations (UnaryOpsEmitter trait)
    └── variables.rs    # Variable allocation
```

### Compilation Pipeline

```
Source Code (.cb64)
       │
       ▼
   ┌───────┐
   │ Lexer │  Tokenizes source into tokens
   └───┬───┘
       │
       ▼
   ┌────────┐
   │ Parser │  Builds Abstract Syntax Tree
   └───┬────┘
       │
       ▼
   ┌──────────┐
   │ Analyzer │  Type checking, semantic analysis
   └────┬─────┘
        │
        ▼
   ┌─────────┐
   │ Codegen │  Generates 6510 machine code
   └────┬────┘
        │
        ▼
    PRG/D64 File
```

### Key Design Patterns

- **Extension Trait Pattern**: Each submodule defines a trait (e.g., `ExpressionParser`, `TypeChecker`) implemented for the main struct (`Parser`, `Analyzer`, `CodeGenerator`). This enables:
  - Clean separation of concerns into focused files
  - Methods available via `self.method()` across all trait modules
  - Cross-module method calls through the shared receiver type
  - Easy maintenance with smaller, cohesive modules

  Example:

  ```rust
  // In expressions.rs
  pub trait ExpressionParser {
      fn parse_expression(&mut self) -> Result<Expr, CompileError>;
  }
  impl<'a> ExpressionParser for Parser<'a> { ... }

  // In mod.rs - import trait to make methods available
  use expressions::ExpressionParser;
  ```

- **Visitor Pattern**: AST traversal is separated from operations, making it easy to add new analysis or transformation passes.

- **Single Responsibility**: Each module has a clear, focused purpose with ~100-500 lines of code. Large modules are split when they exceed ~600 lines.

---

## License

Cobra64 - A concept for a modern Python-like compiler creating C64 binaries

Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
