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
   - [Operators](#operators)
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

Variables must be declared with a type:

```python
def main():
    # Variable declaration with initial value
    x: byte = 10
    name: string = "PLAYER"

    # Multiple variables
    a: byte = 1
    b: byte = 2
    c: byte = a + b
```

### Constants

Constants are declared at the top level with `const`:

```python
const MAX_SCORE = 255
const SCREEN_WIDTH = 40
const SCREEN_HEIGHT = 25
const VIC_BORDER = $D020

def main():
    x: byte = MAX_SCORE
```

Constants must be numeric values (byte or word).

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

#### Nested If (instead of elif)

```python
def main():
    x: byte = 50

    if x > 100:
        println("LARGE")
    else:
        if x > 25:
            println("MEDIUM")
        else:
            println("SMALL")
```

#### While Loop

```python
def main():
    i: byte = 0
    while i < 10:
        println(i)
        i = i + 1
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
const BORDER = $D020
const BACKGROUND = $D021

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

const SCREEN_WIDTH = 320

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

---

## Error Messages

### Lexer Errors

| Error                     | Description                              |
| ------------------------- | ---------------------------------------- |
| `Invalid character`       | Source contains an unsupported character |
| `Unterminated string`     | String literal missing closing quote     |
| `Invalid escape sequence` | Unknown escape like `\x`                 |
| `Number overflow`         | Number too large for type                |
| `Tabs not allowed`        | Use 4 spaces for indentation             |

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

- Arrays (planned for future)
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
