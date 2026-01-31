# Syntax Design for the C64 Language

This document describes the syntax design for our Python-like C64 language.

## Design Principles

1. **Python-like readability** - Indentation-based blocks, minimal punctuation
2. **Simple and clear** - No complex features, easy to learn
3. **Low overhead** - Syntax should map efficiently to 6510 code
4. **Modern convenience** - Named functions instead of POKE/PEEK

## Inspiration from Existing Languages

### Python

- Indentation for blocks
- No semicolons
- Clear, readable keywords
- Simple function definitions

### Nim

- Python-like syntax that compiles to native code
- Static typing with type inference
- Clean and elegant

### prog8

- Designed specifically for 6502 systems
- Block-based structure
- Built-in hardware access

## Proposed Syntax

### Comments

```
# This is a single-line comment
```

### Variables

```
# Explicit type
byte counter = 0
word score = 1000
bool gameover = false

# Type inference (optional)
x = 42          # Inferred as byte
name = "HELLO"  # Inferred as string
```

### Constants

```
const MAX_LIVES = 3
const SCREEN_WIDTH = 40
const SCREEN_HEIGHT = 25
```

### Arithmetic Operators

```
x = a + b       # Addition
x = a - b       # Subtraction
x = a * b       # Multiplication (expensive on 6502!)
x = a / b       # Division (expensive on 6502!)
x = a % b       # Modulo/remainder
```

### Comparison Operators

```
a == b          # Equal
a != b          # Not equal
a < b           # Less than
a > b           # Greater than
a <= b          # Less or equal
a >= b          # Greater or equal
```

### Logical Operators

```
a and b         # Logical AND
a or b          # Logical OR
not a           # Logical NOT
```

### Bitwise Operators

```
a & b           # Bitwise AND
a | b           # Bitwise OR
a ^ b           # Bitwise XOR
~a              # Bitwise NOT
a << n          # Left shift
a >> n          # Right shift
```

### Control Flow

#### If-Then-Else

```
if score > 1000:
    print("HIGH SCORE!")
elif score > 500:
    print("GOOD SCORE")
else:
    print("TRY AGAIN")
```

#### While Loop

```
while lives > 0:
    play_round()
    lives = lives - 1
```

#### For Loop

```
for i in 0 to 9:
    print(i)

for i in 10 downto 0:
    print(i)
```

#### Loop Control

```
while true:
    if key_pressed():
        break       # Exit loop
    if paused:
        continue    # Skip to next iteration
```

### Functions

```
def add(byte a, byte b) -> byte:
    return a + b

def greet(string name):
    print("HELLO ")
    print(name)
```

### Arrays

```
byte scores[10]             # Array of 10 bytes
scores[0] = 100
scores[1] = 50

byte data[] = [1, 2, 3, 4]  # Initialized array
```

### Strings

```
name = "PLAYER 1"
print(name)

# String with special characters
msg = "SCORE: 0\n"   # Newline
```

## Program Structure

### Minimal Program

```
# hello.c64 - A simple hello world program

def main():
    cls()
    print("HELLO WORLD!")
```

### Complete Example

```
# game.c64 - A simple game structure

const MAX_LIVES = 3

byte score = 0
byte lives = MAX_LIVES
bool gameover = false

def main():
    cls()
    print("WELCOME TO THE GAME!")

    while not gameover:
        update()
        draw()

    print("GAME OVER!")
    print("SCORE: ")
    print(score)

def update():
    if lives == 0:
        gameover = true

def draw():
    cursor(0, 0)
    print("LIVES: ")
    print(lives)
```

## Keywords (Reserved Words)

```
# Types
byte, word, sbyte, sword, bool, string

# Values
true, false

# Control flow
if, elif, else, while, for, in, to, downto, break, continue, return

# Definitions
def, const

# Operators
and, or, not
```

## Differences from Python

1. **Static typing** - Types are known at compile time
2. **No classes/objects** - Too complex for 8-bit
3. **No dynamic memory** - Fixed memory allocation
4. **Limited recursion** - Stack is only 256 bytes
5. **No exceptions** - Error handling via return values

## References

- [Python Syntax](https://docs.python.org/3/reference/grammar.html)
- [Nim Language](https://nim-lang.org/documentation.html)
- [prog8 Programming](https://prog8.readthedocs.io/en/stable/programming.html)
