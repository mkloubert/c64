# Language Reference

Complete reference for all language features.

---

## Program Structure

### File Extension

Source files use the `.c64` extension.

### Entry Point

Every program must have a `main` function:

```
def main():
    # Program starts here
```

### Comments

```
# Single line comment
```

---

## Data Types

### Numeric Types

| Type    | Size    | Range           |
| ------- | ------- | --------------- |
| `byte`  | 1 byte  | 0 to 255        |
| `word`  | 2 bytes | 0 to 65535      |
| `sbyte` | 1 byte  | -128 to 127     |
| `sword` | 2 bytes | -32768 to 32767 |

### Other Types

| Type     | Description              |
| -------- | ------------------------ |
| `bool`   | `true` or `false`        |
| `string` | Text, max 255 characters |

### Arrays

```
byte data[10]           # 10 bytes
word scores[5]          # 5 words (10 bytes)
byte init[] = [1,2,3]   # Initialized
```

---

## Literals

### Numbers

```
42          # Decimal
$FF         # Hexadecimal
%10101010   # Binary
```

### Characters and Strings

```
'A'         # Character
"HELLO"     # String
```

### Escape Sequences

| Sequence | Meaning         |
| -------- | --------------- |
| `\n`     | Newline         |
| `\r`     | Carriage return |
| `\\`     | Backslash       |
| `\"`     | Double quote    |
| `\'`     | Single quote    |
| `\0`     | Null            |

---

## Variables

### Declaration

```
byte x                  # Uninitialized (default 0)
byte y = 10             # Initialized
word score = 0
bool done = false
string name = "PLAYER"
```

### Constants

```
const MAX_LIVES = 3
const SCREEN_WIDTH = 40
```

---

## Operators

### Arithmetic

| Operator | Description    |
| -------- | -------------- |
| `+`      | Addition       |
| `-`      | Subtraction    |
| `*`      | Multiplication |
| `/`      | Division       |
| `%`      | Modulo         |

### Comparison

| Operator | Description      |
| -------- | ---------------- |
| `==`     | Equal            |
| `!=`     | Not equal        |
| `<`      | Less than        |
| `>`      | Greater than     |
| `<=`     | Less or equal    |
| `>=`     | Greater or equal |

### Logical

| Operator | Description |
| -------- | ----------- |
| `and`    | Logical AND |
| `or`     | Logical OR  |
| `not`    | Logical NOT |

### Bitwise

| Operator | Description |
| -------- | ----------- | --- |
| `&`      | AND         |
| `        | `           | OR  |
| `^`      | XOR         |
| `~`      | NOT         |
| `<<`     | Left shift  |
| `>>`     | Right shift |

### Assignment

| Operator | Equivalent  |
| -------- | ----------- | ------ | --- |
| `=`      | Assignment  |
| `+=`     | `a = a + b` |
| `-=`     | `a = a - b` |
| `*=`     | `a = a * b` |
| `/=`     | `a = a / b` |
| `&=`     | `a = a & b` |
| `        | =`          | `a = a | b`  |

---

## Control Flow

### If Statement

```
if condition:
    statements

if condition:
    statements
else:
    statements

if condition1:
    statements
elif condition2:
    statements
else:
    statements
```

### While Loop

```
while condition:
    statements
```

### For Loop

```
for i in start to end:
    statements

for i in start downto end:
    statements
```

### Break and Continue

```
break       # Exit loop
continue    # Skip to next iteration
```

### Pass

```
if condition:
    pass    # Do nothing placeholder
```

---

## Functions

### Definition

```
def function_name():
    statements

def function_name(type param1, type param2):
    statements

def function_name() -> return_type:
    return value

def function_name(type param) -> return_type:
    return expression
```

### Examples

```
def greet():
    println("HELLO")

def add(byte a, byte b) -> byte:
    return a + b

def print_score(word score):
    print("SCORE: ")
    println(score)
```

---

## Built-in Functions

### Screen

| Function                   | Description         |
| -------------------------- | ------------------- |
| `cls()`                    | Clear screen        |
| `print(value)`             | Print value         |
| `println(value)`           | Print with newline  |
| `cursor(x, y)`             | Set cursor position |
| `home()`                   | Cursor to top-left  |
| `char_at(x, y, char)`      | Put character       |
| `color_at(x, y, color)`    | Set character color |
| `screen_color(bg, border)` | Set screen colors   |
| `text_color(color)`        | Set text color      |

### Input

| Function         | Description                 |
| ---------------- | --------------------------- |
| `key()`          | Get pressed key (0 if none) |
| `read()`     | Wait for keypress           |
| `joystick(port)` | Read joystick (1 or 2)      |

### Sprites

| Function                 | Description           |
| ------------------------ | --------------------- |
| `sprite_enable(n, on)`   | Enable/disable sprite |
| `sprite_pos(n, x, y)`    | Set position          |
| `sprite_color(n, color)` | Set color             |
| `sprite_data(n, addr)`   | Set data pointer      |

### Sound

| Function                           | Description       |
| ---------------------------------- | ----------------- |
| `sound_init()`                     | Initialize SID    |
| `volume(level)`                    | Set volume (0-15) |
| `sound_off()`                      | Turn off sound    |
| `voice(n, wave, freq, a, d, s, r)` | Configure voice   |
| `voice_on(n)`                      | Start voice       |
| `voice_off(n)`                     | Stop voice        |

### Timing

| Function       | Description     |
| -------------- | --------------- |
| `wait(frames)` | Wait N frames   |
| `raster()`     | Get raster line |

### Memory

| Function            | Description |
| ------------------- | ----------- |
| `peek(addr)`        | Read byte   |
| `poke(addr, value)` | Write byte  |

---

## Built-in Constants

### Colors

```
BLACK, WHITE, RED, CYAN, PURPLE, GREEN, BLUE, YELLOW,
ORANGE, BROWN, LIGHTRED, DARKGREY, GREY,
LIGHTGREEN, LIGHTBLUE, LIGHTGREY
```

### Joystick

```
JOY_UP, JOY_DOWN, JOY_LEFT, JOY_RIGHT, JOY_FIRE
```

### Waveforms

```
WAVE_TRIANGLE, WAVE_SAW, WAVE_PULSE, WAVE_NOISE
```

---

## Keywords

```
and         bool        break       byte        const
continue    def         downto      elif        else
false       for         if          in          not
or          pass        return      sbyte       string
sword       to          true        while       word
```

---

## Indentation

- Use 4 spaces for each level
- Tabs are not allowed
- Blocks start after `:`
- Blocks end when indentation decreases

```
def example():
    if condition:
        statement1      # 8 spaces
        statement2
    else:
        statement3
    statement4          # 4 spaces, outside if
```
