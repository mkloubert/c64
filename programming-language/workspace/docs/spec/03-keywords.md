# Keywords and Reserved Words

This document lists all keywords and reserved words in the language.

## Keywords

Keywords are reserved and cannot be used as identifiers.

### Type Keywords

| Keyword  | Description                             |
| -------- | --------------------------------------- |
| `byte`   | Unsigned 8-bit integer (0-255)          |
| `word`   | Unsigned 16-bit integer (0-65535)       |
| `sbyte`  | Signed 8-bit integer (-128 to 127)      |
| `sword`  | Signed 16-bit integer (-32768 to 32767) |
| `bool`   | Boolean value (true or false)           |
| `string` | Text string                             |

### Definition Keywords

| Keyword | Description          |
| ------- | -------------------- |
| `const` | Constant declaration |
| `def`   | Function definition  |

### Control Flow Keywords

| Keyword    | Description              |
| ---------- | ------------------------ |
| `if`       | Conditional statement    |
| `elif`     | Else-if branch           |
| `else`     | Else branch              |
| `while`    | While loop               |
| `for`      | For loop                 |
| `in`       | Range iteration          |
| `to`       | Ascending range          |
| `downto`   | Descending range         |
| `break`    | Exit loop                |
| `continue` | Skip to next iteration   |
| `return`   | Return from function     |
| `pass`     | No-operation placeholder |

### Logical Keywords

| Keyword | Description |
| ------- | ----------- |
| `and`   | Logical AND |
| `or`    | Logical OR  |
| `not`   | Logical NOT |

### Literal Keywords

| Keyword | Description         |
| ------- | ------------------- |
| `true`  | Boolean true value  |
| `false` | Boolean false value |

---

## Complete Keyword List (Alphabetical)

```
and         bool        break       byte        const
continue    def         downto      elif        else
false       for         if          in          not
or          pass        return      sbyte       string
sword       to          true        while       word
```

**Total: 25 keywords**

---

## Reserved for Future Use

These words are reserved but not yet implemented:

| Word       | Potential Use           |
| ---------- | ----------------------- |
| `struct`   | User-defined types      |
| `enum`     | Enumeration types       |
| `import`   | Module import           |
| `export`   | Module export           |
| `inline`   | Inline functions        |
| `asm`      | Inline assembly         |
| `var`      | Type-inferred variables |
| `let`      | Immutable binding       |
| `match`    | Pattern matching        |
| `case`     | Match case              |
| `try`      | Error handling          |
| `except`   | Exception handler       |
| `finally`  | Cleanup block           |
| `raise`    | Raise exception         |
| `as`       | Type alias              |
| `is`       | Type check              |
| `from`     | Import from             |
| `global`   | Global scope            |
| `static`   | Static allocation       |
| `volatile` | No optimization         |

---

## Identifier Rules

### Valid Identifiers

- Start with a letter (a-z, A-Z) or underscore (\_)
- Followed by letters, digits (0-9), or underscores
- Case-sensitive (`score` and `Score` are different)
- No length limit (but keep reasonable)

### Examples

```
# Valid identifiers
x
score
player_x
_private
MAX_VALUE
camelCase
PascalCase
snake_case
value1
data2process
```

```
# Invalid identifiers
1st_place       # Cannot start with digit
my-var          # Hyphen not allowed
my var          # Space not allowed
if              # Reserved keyword
const           # Reserved keyword
```

### Naming Conventions

| Type      | Convention      | Example         |
| --------- | --------------- | --------------- |
| Variables | snake_case      | `player_score`  |
| Constants | UPPER_SNAKE     | `MAX_LIVES`     |
| Functions | snake_case      | `update_player` |
| Types     | (built-in only) | `byte`, `word`  |

---

## Scope Rules

### Global Scope

- Constants declared outside functions
- Variables declared outside functions
- Function definitions

### Local Scope

- Variables declared inside functions
- Function parameters
- Loop variables (for loops)

### Name Resolution

1. Look in current local scope
2. Look in enclosing scopes (nested functions)
3. Look in global scope
4. If not found: error

### Shadowing

Local variables can shadow global variables:

```
byte score = 0          # Global

def update():
    byte score = 100    # Local, shadows global
    score = score + 1   # Modifies local
    # Global score is unchanged
```

**Warning:** Shadowing is allowed but not recommended.

---

## Built-in Names

These names are pre-defined but can be shadowed (not recommended):

### Color Constants

```
BLACK, WHITE, RED, CYAN, PURPLE, GREEN, BLUE, YELLOW,
ORANGE, BROWN, LIGHTRED, DARKGREY, GREY,
LIGHTGREEN, LIGHTBLUE, LIGHTGREY
```

### Joystick Constants

```
JOY_UP, JOY_DOWN, JOY_LEFT, JOY_RIGHT, JOY_FIRE
```

### Wave Constants (for sound)

```
WAVE_TRIANGLE, WAVE_SAW, WAVE_PULSE, WAVE_NOISE
```

### System Functions

```
# Screen
cls, print, println, cursor, char_at, color_at,
screen_color, text_color

# Input
key, wait_key, joystick

# Sound
sound_init, volume, sound_off, voice, voice_on, voice_off

# Sprites
sprite_enable, sprite_pos, sprite_color, sprite_data

# Timing
wait, raster

# Memory
poke, peek
```

---

## Error Messages for Keywords

| Error               | Message                                                                      |
| ------------------- | ---------------------------------------------------------------------------- |
| Keyword as variable | `Error: 'if' is a reserved keyword and cannot be used as a variable name`    |
| Keyword as function | `Error: 'while' is a reserved keyword and cannot be used as a function name` |
| Reserved word       | `Error: 'struct' is reserved for future use`                                 |
| Misspelled keyword  | `Error: Unknown identifier 'iff'. Did you mean 'if'?`                        |
