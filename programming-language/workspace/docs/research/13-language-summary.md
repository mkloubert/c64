# Language Design Summary

This document summarizes all research and proposes the final language design.

## Language Name Ideas

- **C64Script** - Clear reference to target platform
- **SixtyFour** - Plays on C64
- **RetroLang** - Generic retro theme
- **Commodore Script** - Full name reference
- **SixFive** - Reference to 6502/6510 CPU

(Name to be decided later)

---

## Design Goals

1. **Simple** - Easier than BASIC, much easier than assembly
2. **Readable** - Python-like syntax, self-documenting
3. **Efficient** - Compiles to fast native 6510 code
4. **Accessible** - Built-in hardware access with clear function names
5. **Modern** - Structured programming (no GOTO/line numbers)

---

## Core Language Features

### Data Types

| Type     | Size | Range      | Usage                             |
| -------- | ---- | ---------- | --------------------------------- |
| `byte`   | 1    | 0-255      | Default integer, counters, colors |
| `word`   | 2    | 0-65535    | Addresses, large values, scores   |
| `bool`   | 1    | true/false | Flags, conditions                 |
| `string` | var  | -          | Text (null-terminated)            |
| `byte[]` | var  | -          | Arrays (max 256 elements)         |

### Variable Declaration

```
byte counter = 0
word score = 0
bool gameover = false
string name = "PLAYER"
byte highscores[10]
```

### Constants

```
const SCREEN_WIDTH = 40
const MAX_LIVES = 3
const PLAYER_COLOR = RED
```

### Operators

| Category   | Operators                   |
| ---------- | --------------------------- | ------------------- |
| Arithmetic | `+` `-` `*` `/` `%`         |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical    | `and` `or` `not`            |
| Bitwise    | `&` `                       | ` `^` `~` `<<` `>>` |
| Assignment | `=` `+=` `-=` etc.          |

### Control Flow

```
# If statement
if condition:
    statements
elif other:
    statements
else:
    statements

# While loop
while condition:
    statements

# For loop
for i in start to end:
    statements

for i in start downto end:
    statements

# Loop control
break
continue
```

### Functions

```
def function_name(byte param1, byte param2) -> byte:
    statements
    return value

def procedure_name(word address):
    statements
```

### Program Structure

```
# file: game.c64

# Constants at top
const MAX_ENEMIES = 8

# Global variables
byte player_x = 160
byte player_y = 100

# Main entry point
def main():
    init_game()
    game_loop()

def init_game():
    cls()
    screen_color(BLACK, BLACK)

def game_loop():
    while not gameover:
        update()
        draw()
```

---

## System Functions

### Screen Output

| Function                   | Description         |
| -------------------------- | ------------------- |
| `cls()`                    | Clear screen        |
| `print(text)`              | Print text          |
| `println(text)`            | Print with newline  |
| `cursor(x, y)`             | Set cursor position |
| `char_at(x, y, c)`         | Put character       |
| `color_at(x, y, c)`        | Set character color |
| `screen_color(bg, border)` | Set screen colors   |
| `text_color(c)`            | Set text color      |

### Input

| Function         | Description                 |
| ---------------- | --------------------------- |
| `key()`          | Get pressed key (0 if none) |
| `read()`     | Wait for keypress           |
| `joystick(port)` | Read joystick (1 or 2)      |

### Sound

| Function                           | Description       |
| ---------------------------------- | ----------------- |
| `sound_init()`                     | Initialize SID    |
| `volume(v)`                        | Set volume (0-15) |
| `sound_off()`                      | Turn off sound    |
| `voice(n, wave, freq, a, d, s, r)` | Configure voice   |
| `voice_on(n)`                      | Start voice       |
| `voice_off(n)`                     | Stop voice        |

### Sprites

| Function               | Description             |
| ---------------------- | ----------------------- |
| `sprite_enable(n, on)` | Enable/disable sprite   |
| `sprite_pos(n, x, y)`  | Set sprite position     |
| `sprite_color(n, c)`   | Set sprite color        |
| `sprite_data(n, addr)` | Set sprite data pointer |

### Timing

| Function       | Description             |
| -------------- | ----------------------- |
| `wait(frames)` | Wait N frames           |
| `raster()`     | Get current raster line |

### Low-Level

| Function          | Description           |
| ----------------- | --------------------- |
| `poke(addr, val)` | Write byte to memory  |
| `peek(addr)`      | Read byte from memory |

---

## Color Constants

```
BLACK, WHITE, RED, CYAN, PURPLE, GREEN, BLUE, YELLOW,
ORANGE, BROWN, LIGHTRED, DARKGREY, GREY,
LIGHTGREEN, LIGHTBLUE, LIGHTGREY
```

---

## Joystick Constants

```
JOY_UP, JOY_DOWN, JOY_LEFT, JOY_RIGHT, JOY_FIRE
```

---

## Complete Example Program

```
# Space Shooter Demo
# A simple game demonstrating language features

const MAX_STARS = 20

byte player_x = 160
byte player_y = 200
word score = 0
byte stars_x[MAX_STARS]
byte stars_y[MAX_STARS]
bool running = true

def main():
    init()

    while running:
        input()
        update()
        draw()
        wait(1)

def init():
    cls()
    screen_color(BLACK, BLACK)
    sound_init()
    volume(8)

    # Initialize stars
    for i in 0 to MAX_STARS - 1:
        stars_x[i] = random() % 40
        stars_y[i] = random() % 25

def input():
    byte joy = joystick(2)

    if joy & JOY_LEFT and player_x > 0:
        player_x = player_x - 2

    if joy & JOY_RIGHT and player_x < 255:
        player_x = player_x + 2

    if joy & JOY_FIRE:
        shoot()

    if key() == 'Q':
        running = false

def update():
    # Move stars down
    for i in 0 to MAX_STARS - 1:
        stars_y[i] = stars_y[i] + 1
        if stars_y[i] > 24:
            stars_y[i] = 0
            stars_x[i] = random() % 40

def draw():
    cls()

    # Draw stars
    text_color(GREY)
    for i in 0 to MAX_STARS - 1:
        char_at(stars_x[i], stars_y[i], '.')

    # Draw player (simple character)
    text_color(GREEN)
    char_at(player_x / 8, 24, '^')

    # Draw score
    cursor(0, 0)
    text_color(WHITE)
    print("SCORE: ")
    print(score)

def shoot():
    # Play shoot sound
    voice(1, WAVE_NOISE, 2000, 0, 0, 8, 4)
    voice_on(1)
    score = score + 10
```

---

## Implementation Priority

### Phase 1: Minimal Viable Language

1. Variables (byte only)
2. Constants
3. Arithmetic operators
4. Print function
5. If-else
6. While loop

### Phase 2: Basic Features

7. Word type
8. For loop
9. Functions
10. More system functions
11. Keyboard input

### Phase 3: Full Features

12. Arrays
13. Strings
14. All system functions
15. Sprites
16. Sound

---

## References

All research documents:

- `08-data-types.md` - Data type design
- `09-syntax-design.md` - Syntax specification
- `10-system-functions.md` - System functions
- `11-existing-compilers.md` - Competitor analysis
- `12-control-flow.md` - Control flow implementation
