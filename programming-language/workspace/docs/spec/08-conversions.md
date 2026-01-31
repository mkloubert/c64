# Type Conversions

This document defines all type conversion rules.

---

## Conversion Categories

### Implicit Conversions (Automatic)

Conversions that happen automatically without explicit syntax.

### Explicit Conversions (Casts)

Conversions that require explicit type syntax.

---

## Implicit Conversion Rules

### Safe Promotions (Always Allowed)

Smaller types automatically promote to larger types when needed:

| From    | To      | Example               |
| ------- | ------- | --------------------- |
| `byte`  | `word`  | `word w = byte_var`   |
| `sbyte` | `sword` | `sword s = sbyte_var` |
| `bool`  | `byte`  | `byte b = true` → 1   |
| `bool`  | `word`  | `word w = false` → 0  |

### In Expressions

When operators have mixed types, the smaller type is promoted:

```
byte a = 100
word b = 1000
word c = a + b      # 'a' promoted to word, result is word
```

### Promotion Table

| Left    | Right   | Result  |
| ------- | ------- | ------- |
| `byte`  | `byte`  | `byte`  |
| `byte`  | `word`  | `word`  |
| `word`  | `byte`  | `word`  |
| `word`  | `word`  | `word`  |
| `sbyte` | `sbyte` | `sbyte` |
| `sbyte` | `sword` | `sword` |
| `sword` | `sbyte` | `sword` |
| `sword` | `sword` | `sword` |

### Signed/Unsigned Mixing

**Warning:** Mixing signed and unsigned types produces a warning:

```
byte a = 200
sbyte b = -10
# Warning W011: Comparison between signed and unsigned types
if a > b:       # May produce unexpected results!
```

**Recommendation:** Avoid mixing signed and unsigned types.

---

## Explicit Conversions (Casts)

### Syntax

```
target_type(expression)
```

### Examples

```
word score = 1000
byte low_byte = byte(score)         # Truncates to low byte

sbyte delta = -5
byte positive = byte(delta)         # Reinterprets bits

byte x = 200
sbyte signed_x = sbyte(x)           # 200 becomes -56
```

### Conversion Functions

| Function      | Description                          |
| ------------- | ------------------------------------ |
| `byte(expr)`  | Convert to byte (truncate if needed) |
| `word(expr)`  | Convert to word (zero-extend)        |
| `sbyte(expr)` | Convert to signed byte               |
| `sword(expr)` | Convert to signed word               |
| `bool(expr)`  | Convert to bool (0=false, else true) |

---

## Narrowing Conversions

Converting from larger to smaller type may lose data:

### word → byte

Only the low byte is kept:

```
word w = $1234
byte b = byte(w)    # b = $34 (low byte only)
```

Assembly equivalent:

```asm
LDA w           ; Load low byte
STA b
```

### sword → sbyte

Only the low byte is kept, sign may change:

```
sword s = -1000     # $FC18
sbyte b = sbyte(s)  # b = $18 (24, not negative!)
```

### Warning on Implicit Narrowing

Implicit narrowing produces a warning:

```
word score = 1000
byte display = score    # Warning W010: may lose data
```

Use explicit cast to suppress:

```
byte display = byte(score)  # OK, explicit
```

---

## Boolean Conversions

### To Bool

Any numeric type can convert to bool:

- `0` → `false`
- Non-zero → `true`

```
byte x = 0
if x:               # false

byte y = 42
if y:               # true

word z = 0
bool flag = bool(z) # false
```

### From Bool

Bool converts to numeric types:

- `false` → `0`
- `true` → `1`

```
bool active = true
byte count = active     # count = 1
word value = active     # value = 1
```

---

## Integer Literal Conversions

Integer literals adapt to the context:

### In Declarations

```
byte a = 42         # Literal fits in byte: OK
byte b = 300        # Error: too large for byte

word c = 42         # Literal fits in word: OK
word d = 70000      # Error: too large for word
```

### In Expressions

Literals take the type needed by context:

```
word result = 100 + 200     # Literals treated as word
byte small = 10 + 20        # Literals treated as byte
```

### Hex and Binary Literals

```
byte a = $FF        # 255, fits in byte
byte b = $100       # Error: 256 too large

word c = $1234      # OK for word
byte d = %11111111  # 255, OK
byte e = %100000000 # Error: 256 too large
```

---

## String Conversions

### No Automatic String Conversion

Numbers do not automatically convert to strings:

```
byte score = 42
print(score)        # OK: print() accepts byte
string s = score    # Error: cannot convert byte to string
```

### String to Number

Not supported. Use parsing functions if needed (future feature).

---

## Array Conversions

### No Array Conversions

Arrays cannot be converted between types:

```
byte data[10]
word big_data = data    # Error: cannot convert array
```

### Element Access

Array elements follow normal conversion rules:

```
byte values[10]
values[0] = 100

word total = values[0]      # OK: byte promoted to word
values[1] = word(total)     # OK: explicit narrowing
```

---

## Constant Expression Evaluation

### Compile-Time Arithmetic

Constants are evaluated at compile time with full precision:

```
const A = 1000
const B = 2000
const C = A + B     # C = 3000 (evaluated at compile time)

const D = 255 + 1   # D = 256 (word required)
const E = 128 * 3   # E = 384 (word required)
```

### Type of Constants

Constants get the smallest type that fits:

| Value Range    | Type    |
| -------------- | ------- |
| 0-255          | `byte`  |
| 256-65535      | `word`  |
| -128 to -1     | `sbyte` |
| -32768 to -129 | `sword` |

### Constant in Context

Constants adapt when used:

```
const MAX = 100     # Type: byte

word score = MAX    # MAX treated as word here
byte limit = MAX    # MAX treated as byte here
```

---

## Conversion Errors

### Error: Cannot Convert

```
string name = "PLAYER"
byte x = name               # Error E211: Cannot convert 'string' to 'byte'
```

### Error: Value Out of Range

```
byte x = 300                # Error E020: Integer literal too large for byte
```

### Warning: May Lose Data

```
word big = 1000
byte small = big            # Warning W010: may lose data
```

### Warning: Signed/Unsigned Mix

```
byte a = 200
sbyte b = -50
if a > b:                   # Warning W011: signed/unsigned comparison
```

---

## Conversion Summary Table

| From     | To       | Implicit | Explicit | Notes         |
| -------- | -------- | -------- | -------- | ------------- |
| `byte`   | `word`   | Yes      | Yes      | Zero-extend   |
| `byte`   | `sbyte`  | No       | Yes      | Reinterpret   |
| `byte`   | `sword`  | Yes      | Yes      | Zero-extend   |
| `byte`   | `bool`   | Yes      | Yes      | 0=false       |
| `word`   | `byte`   | Warning  | Yes      | Truncate      |
| `word`   | `sbyte`  | No       | Yes      | Truncate      |
| `word`   | `sword`  | No       | Yes      | Reinterpret   |
| `word`   | `bool`   | Yes      | Yes      | 0=false       |
| `sbyte`  | `byte`   | No       | Yes      | Reinterpret   |
| `sbyte`  | `sword`  | Yes      | Yes      | Sign-extend   |
| `sbyte`  | `word`   | No       | Yes      | Sign-extend   |
| `sword`  | `sbyte`  | Warning  | Yes      | Truncate      |
| `sword`  | `word`   | No       | Yes      | Reinterpret   |
| `sword`  | `byte`   | No       | Yes      | Truncate      |
| `bool`   | `byte`   | Yes      | Yes      | 0 or 1        |
| `bool`   | `word`   | Yes      | Yes      | 0 or 1        |
| `string` | (any)    | No       | No       | Not supported |
| (any)    | `string` | No       | No       | Not supported |

---

## Implementation Notes

### Zero Extension (byte → word)

```asm
; byte_var → word_var
LDA byte_var
STA word_var        ; Low byte
LDA #0
STA word_var+1      ; High byte = 0
```

### Sign Extension (sbyte → sword)

```asm
; sbyte_var → sword_var
LDA sbyte_var
STA sword_var       ; Low byte
ORA #$7F            ; Extend sign
BMI .negative
LDA #$00            ; Positive: high = 0
JMP .done
.negative:
LDA #$FF            ; Negative: high = $FF
.done:
STA sword_var+1
```

### Truncation (word → byte)

```asm
; word_var → byte_var
LDA word_var        ; Just take low byte
STA byte_var
```
