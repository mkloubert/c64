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
