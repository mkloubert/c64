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

Variables can be declared with an explicit type or use type inference:

```python
# Top-level variables with explicit type
counter: byte = 0
score: word = 1000

# Top-level variables with type inference
x = 10          # Inferred as byte (0-255)
y = 1000        # Inferred as word (256-65535)
z = -50         # Inferred as sbyte (-128 to -1)
big = -1000     # Inferred as sword (-32768 to -129)
pi = 3.14       # Inferred as float
flag = true     # Inferred as bool

def main():
    # Inside functions, explicit type is required
    a: byte = 1
    b: byte = 2
    c: byte = a + b
```

**Note:** Type inference for variables is only available at top-level. Inside functions, explicit type annotation is required.

### Constants

Constants are identified by their **naming convention**: names that start with an uppercase letter and have ALL letters in uppercase are constants.

Constants support both type inference and explicit type annotation:

```python
# Constants with type inference (type determined from value)
MAX_SCORE = 255         # Inferred as byte
SCREEN_WIDTH = 40       # Inferred as byte
LARGE_VALUE = 1000      # Inferred as word
NEGATIVE = -100         # Inferred as sbyte
E = 2.718               # Inferred as float

# Constants with explicit type (can override inference)
MAX: word = 255         # Force word type for small value
PI: fixed = 3.14159     # Use fixed instead of float
GOLDEN: float = 1.618   # Explicitly float

def main():
    x: byte = MAX_SCORE
    y: word = MAX + 1
```

#### Naming Rules

| Type         | Rule                                                   | Examples                     |
| ------------ | ------------------------------------------------------ | ---------------------------- |
| **Constant** | First letter uppercase → ALL letters must be uppercase | `MAX`, `SCREEN_WIDTH`, `_4K` |
| **Variable** | First letter lowercase                                 | `myVar`, `counter`, `_temp`  |
| **Invalid**  | First letter uppercase but mixed case                  | `MyConst`, `MaxValue`        |

```python
# Valid constants
MAX_VALUE = 255        # All uppercase letters
VIC_BASE = $D000       # All uppercase letters
_SPRITE_COUNT = 8      # Underscore prefix, all letters uppercase
_4K = 4096             # First letter is 'K' (uppercase)

# Valid variables
counter: byte = 0      # First letter 'c' is lowercase
myScore: word = 1000   # First letter 'm' is lowercase
_temp: byte = 5        # First letter 't' is lowercase

# INVALID - will cause compile error
# MaxValue = 100       # Error: Mixed case (first uppercase, but not all)
# _MyConst = 50        # Error: Mixed case
```

**Important:**

- Constants are declared at **top-level only** (not inside functions)
- Constants are **immutable** - they cannot be reassigned
- Constants can have any type (byte, word, sbyte, sword, fixed, float, bool, string)

### Type Inference

When declaring variables or constants without an explicit type, the compiler infers the type from the value:

| Value                 | Inferred Type |
| --------------------- | ------------- |
| `0` to `255`          | `byte`        |
| `256` to `65535`      | `word`        |
| `-128` to `-1`        | `sbyte`       |
| `-32768` to `-129`    | `sword`       |
| Decimal (e.g. `3.14`) | `float`       |
| `true` / `false`      | `bool`        |
| `"text"`              | `string`      |

```python
# Type inference examples
small = 10          # byte (value fits in 0-255)
medium = 1000       # word (value fits in 256-65535)
negative = -50      # sbyte (value fits in -128 to -1)
big_neg = -1000     # sword (value fits in -32768 to -129)
decimal = 3.14      # float (decimal values default to float)
flag = true         # bool
text = "hello"      # string

# Use explicit type to override inference
MAX: word = 255     # Force word instead of byte
PI: fixed = 3.14    # Force fixed instead of float
```

**When to use explicit types:**

- When you need a larger type than inferred (e.g., `word` for value `100`)
- When you want `fixed` instead of the default `float` for decimals
- Inside functions (type inference only works at top-level)

### Arrays

Arrays allow storing multiple values of the same type in contiguous memory.

#### Array Types

| Type      | Element Size | Element Range   | Description           |
| --------- | ------------ | --------------- | --------------------- |
| `byte[]`  | 1 byte       | 0-255           | Unsigned 8-bit array  |
| `word[]`  | 2 bytes      | 0-65535         | Unsigned 16-bit array |
| `bool[]`  | 1 byte       | true/false      | Boolean array         |
| `sbyte[]` | 1 byte       | -128 to 127     | Signed 8-bit array    |
| `sword[]` | 2 bytes      | -32768 to 32767 | Signed 16-bit array   |

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

Convert values between types using the type name as a function:

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

    # Array element conversion
    scores: word[] = [1000, 2000]
    low_byte: byte = byte(scores[0])
```

**Available casts:**

| From    | To      | Notes                      |
| ------- | ------- | -------------------------- |
| `word`  | `byte`  | Truncates to low 8 bits    |
| `sbyte` | `word`  | Sign-extends to 16 bits    |
| `sbyte` | `sword` | Sign-extends to 16 bits    |
| `float` | `fixed` | May lose precision         |
| `float` | `word`  | Truncates decimal part     |
| `fixed` | `word`  | Truncates fractional part  |
| `fixed` | `float` | Converts to floating-point |
| `byte`  | `word`  | Zero-extends to 16 bits    |

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

#### `wait_for_key() -> byte`

Waits for a key press and returns it.

```python
def main():
    println("PRESS ANY KEY")
    k: byte = wait_for_key()
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

### Array Functions

#### `len(array) -> word`

Returns the length (number of elements) of an array.

```python
def main():
    data: byte[] = [10, 20, 30, 40, 50]
    size: word = len(data)
    println(size)  # Prints: 5

    # Use in loop condition
    i: byte = 0
    while i < len(data):
        println(data[i])
        i = i + 1

    # Works with all array types
    buffer: byte[100]
    println(len(buffer))  # Prints: 100
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
# Constants use UPPERCASE names (no 'const' keyword needed)
BORDER = $D020
BACKGROUND = $D021

def main():
    cls()
    println("PRESS ANY KEY")
    println("TO CYCLE COLORS")

    color: byte = 0
    while true:
        k: byte = wait_for_key()
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
        k: byte = wait_for_key()
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
# Constants use UPPERCASE names

SCREEN_WIDTH = 320

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

| Error                        | Description                                                          |
| ---------------------------- | -------------------------------------------------------------------- |
| `Invalid character`          | Source contains an unsupported character                             |
| `Unterminated string`        | String literal missing closing quote                                 |
| `Invalid escape sequence`    | Unknown escape like `\x`                                             |
| `Number overflow`            | Number too large for type                                            |
| `Tabs not allowed`           | Use 4 spaces for indentation                                         |
| `Invalid identifier naming`  | Mixed case like `MyConst` (must be all uppercase or start lowercase) |
| `Identifier only underscore` | Identifier cannot be just `_` or `__`                                |

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

1. **elif has a code generation bug** - Use nested if-else instead:

   ```python
   # Instead of elif, use:
   if condition1:
       # ...
   else:
       if condition2:
           # ...
   ```

2. **Deep nesting limit** - The 6510 CPU has a ~127 byte branch range. Very deep nesting (more than 4-5 levels) may fail to compile.

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

- **1.4.0** - Array support
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

- **1.3.0** - Naming convention for constants
  - Removed `const` keyword
  - Constants are now identified by UPPERCASE naming convention
  - First letter uppercase + all letters uppercase = constant
  - First letter lowercase = variable
  - Added compile-time validation for naming rules
  - New error codes: E026 (InvalidIdentifierNaming), E027 (IdentifierOnlyUnderscore)

- **1.2.0** - Decimal number types
  - Added `fixed` type (12.4 fixed-point, -2048.0 to +2047.9375)
  - Added `float` type (IEEE-754 binary16, ±65504)
  - Decimal literal support (3.14, 0.5, -2.25)
  - Scientific notation for floats (1.5e3, 2.0e-5)
  - Full arithmetic operations for both types
  - Type conversions between int, fixed, and float
  - Automatic type promotion in mixed operations
  - Print support for fixed and float values

- **1.1.0** - Signed integer types
  - Added `sbyte` type (-128 to 127)
  - Added `sword` type (-32768 to 32767)
  - Full signed arithmetic (add, sub, mul, div)
  - Full signed comparisons (<, >, <=, >=, ==, !=)
  - Negative number literals (-128, -$7F, -%01111111)
  - Compile-time range validation

- **1.0.0** - Initial release
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
    ├── float_runtime.rs # Float operations runtime
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
