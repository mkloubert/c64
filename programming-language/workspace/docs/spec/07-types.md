# Data Type Specification

This document defines all data types, their behavior, and memory representation.

---

## Type Overview

| Type     | Size     | Range            | Default Value |
| -------- | -------- | ---------------- | ------------- |
| `byte`   | 1 byte   | 0 to 255         | 0             |
| `word`   | 2 bytes  | 0 to 65535       | 0             |
| `sbyte`  | 1 byte   | -128 to +127     | 0             |
| `sword`  | 2 bytes  | -32768 to +32767 | 0             |
| `bool`   | 1 byte   | true, false      | false         |
| `string` | variable | 0-255 chars      | ""            |
| `byte[]` | variable | 1-256 elements   | all zeros     |
| `word[]` | variable | 1-128 elements   | all zeros     |

---

## Byte Type

### Definition

```
byte variable_name
byte variable_name = initial_value
```

### Properties

| Property | Value           |
| -------- | --------------- |
| Size     | 1 byte (8 bits) |
| Minimum  | 0               |
| Maximum  | 255             |
| Signed   | No (unsigned)   |
| Default  | 0               |

### Memory Representation

Single byte, stored directly:

```
byte x = 42
# Memory: $2A (42 decimal)
```

### Overflow Behavior

Overflow wraps around (modulo 256):

```
byte x = 255
x = x + 1       # x becomes 0
x = x + 10      # x becomes 10

byte y = 0
y = y - 1       # y becomes 255
```

### Operations

All arithmetic, bitwise, and comparison operators are supported.

```
byte a = 100
byte b = 50
byte c = a + b      # 150
byte d = a * 3      # 44 (300 mod 256)
byte e = a >> 2     # 25
bool f = a > b      # true
```

### Best Practices

- Use `byte` as the default integer type
- Prefer `byte` for loop counters, indices, colors
- Check for overflow in critical calculations

---

## Word Type

### Definition

```
word variable_name
word variable_name = initial_value
```

### Properties

| Property | Value             |
| -------- | ----------------- |
| Size     | 2 bytes (16 bits) |
| Minimum  | 0                 |
| Maximum  | 65535             |
| Signed   | No (unsigned)     |
| Default  | 0                 |

### Memory Representation

Little-endian (low byte first):

```
word x = $1234
# Memory at address $1000:
#   $1000: $34 (low byte)
#   $1001: $12 (high byte)
```

### Overflow Behavior

Overflow wraps around (modulo 65536):

```
word x = 65535
x = x + 1       # x becomes 0
x = x + 100     # x becomes 100
```

### Operations

All arithmetic, bitwise, and comparison operators are supported.

**Performance Note:** Word operations are slower than byte operations because the 6502 is an 8-bit CPU. Each word operation requires multiple instructions.

```
word score = 1000
score = score + 100     # Requires 4+ instructions
```

### Accessing High/Low Bytes

```
word addr = $1234
byte lo = addr & $FF        # $34
byte hi = (addr >> 8) & $FF # $12
```

### Best Practices

- Use `word` only when byte range is insufficient
- Good for: scores, addresses, large counters
- Avoid in tight loops if possible

---

## Signed Byte Type (sbyte)

### Definition

```
sbyte variable_name
sbyte variable_name = initial_value
```

### Properties

| Property | Value                  |
| -------- | ---------------------- |
| Size     | 1 byte (8 bits)        |
| Minimum  | -128                   |
| Maximum  | +127                   |
| Signed   | Yes (two's complement) |
| Default  | 0                      |

### Memory Representation

Two's complement encoding:

| Value | Binary   | Hex |
| ----- | -------- | --- |
| 0     | 00000000 | $00 |
| 1     | 00000001 | $01 |
| 127   | 01111111 | $7F |
| -1    | 11111111 | $FF |
| -128  | 10000000 | $80 |

### Overflow Behavior

```
sbyte x = 127
x = x + 1       # x becomes -128 (overflow)

sbyte y = -128
y = y - 1       # y becomes 127 (underflow)
```

### Use Cases

```
sbyte velocity_x = -5   # Moving left
sbyte velocity_y = 3    # Moving down
sbyte delta = -10       # Negative change
```

---

## Signed Word Type (sword)

### Definition

```
sword variable_name
sword variable_name = initial_value
```

### Properties

| Property | Value                  |
| -------- | ---------------------- |
| Size     | 2 bytes (16 bits)      |
| Minimum  | -32768                 |
| Maximum  | +32767                 |
| Signed   | Yes (two's complement) |
| Default  | 0                      |

### Memory Representation

Little-endian, two's complement:

```
sword x = -1
# Memory: $FF $FF

sword y = -256
# Memory: $00 $FF
```

---

## Bool Type

### Definition

```
bool variable_name
bool variable_name = initial_value
```

### Properties

| Property | Value               |
| -------- | ------------------- |
| Size     | 1 byte              |
| Values   | true (1), false (0) |
| Default  | false               |

### Memory Representation

- `false` = $00 (0)
- `true` = $01 (1)

Note: Any non-zero value is truthy when converting to bool.

### Operations

| Operation        | Result |
| ---------------- | ------ |
| `not true`       | false  |
| `not false`      | true   |
| `true and true`  | true   |
| `true and false` | false  |
| `true or false`  | true   |
| `false or false` | false  |

### Conversion from Integer

```
byte x = 0
if x:           # false (0 is falsy)

byte y = 42
if y:           # true (non-zero is truthy)
```

### Best Practices

```
# Good: explicit bool
bool game_over = false
if game_over:
    end_game()

# Avoid: using integers as bools (unclear)
byte flag = 1
if flag:        # Works but less clear
```

---

## String Type

### Definition

```
string variable_name
string variable_name = "initial value"
string[max_length] variable_name
```

### Properties

| Property   | Value                       |
| ---------- | --------------------------- |
| Encoding   | PETSCII (C64 character set) |
| Terminator | Null byte ($00)             |
| Max length | 255 characters              |
| Default    | "" (empty string)           |

### Memory Representation

Null-terminated byte sequence:

```
string name = "HI"
# Memory:
#   $48 'H'
#   $49 'I'
#   $00 null terminator
```

### Storage Allocation

| Declaration       | Storage                |
| ----------------- | ---------------------- |
| `string s`        | 256 bytes (255 + null) |
| `string[32] s`    | 33 bytes (32 + null)   |
| `string s = "AB"` | 256 bytes, initialized |

### String Operations

```
string name = "PLAYER"

# Length
byte len = strlen(name)     # 6

# Character access (read-only)
byte ch = name[0]           # 'P'

# Comparison
if name == "PLAYER":
    # Equal

# Printing
print(name)
println(name)
```

### String Limitations

1. **No concatenation operator** - use multiple print() calls
2. **No substring** - access individual characters only
3. **Fixed size** - cannot grow beyond declared size
4. **No modification** - strings are effectively immutable

### PETSCII Encoding

Characters are stored in PETSCII format:

| ASCII | PETSCII | Notes                 |
| ----- | ------- | --------------------- |
| A-Z   | $41-$5A | Same as ASCII         |
| a-z   | $C1-$DA | Different from ASCII! |
| 0-9   | $30-$39 | Same as ASCII         |
| Space | $20     | Same as ASCII         |

---

## Array Types

### Definition

```
byte array_name[size]
byte array_name[] = [value1, value2, ...]
word array_name[size]
```

### Properties

| Property     | Byte Array  | Word Array  |
| ------------ | ----------- | ----------- |
| Element size | 1 byte      | 2 bytes     |
| Max elements | 256         | 128         |
| Index range  | 0 to size-1 | 0 to size-1 |
| Index type   | byte        | byte        |

### Memory Layout

**Byte Array:**

```
byte data[4] = [10, 20, 30, 40]
# Memory:
#   [0]: $0A (10)
#   [1]: $14 (20)
#   [2]: $1E (30)
#   [3]: $28 (40)
```

**Word Array:**

```
word scores[3] = [1000, 2000, 3000]
# Memory (little-endian):
#   [0]: $E8 $03 (1000)
#   [1]: $D0 $07 (2000)
#   [2]: $B8 $0B (3000)
```

### Array Operations

```
byte values[10]

# Assignment
values[0] = 100
values[5] = 50

# Reading
byte x = values[0]

# In expressions
byte sum = values[0] + values[1]

# Loop iteration
for i in 0 to 9:
    print(values[i])
```

### Array Initialization

```
# Uninitialized (all zeros)
byte empty[10]

# Initialized with values
byte primes[] = [2, 3, 5, 7, 11]

# Partially initialized (rest are zero)
byte data[10] = [1, 2, 3]   # [1,2,3,0,0,0,0,0,0,0]
```

### Bounds Checking

**No runtime bounds checking!** Accessing out-of-bounds is undefined behavior:

```
byte data[5]
data[10] = 99   # UNDEFINED! Corrupts memory
byte x = data[100]  # UNDEFINED! Reads garbage
```

### Multi-dimensional Arrays

Not directly supported. Use calculated indices:

```
# Simulate 8x8 grid
byte grid[64]

def get_cell(byte x, byte y) -> byte:
    return grid[y * 8 + x]

def set_cell(byte x, byte y, byte value):
    grid[y * 8 + x] = value
```

---

## Type Summary Table

| Type     | Size | Min    | Max       | Signed | Default |
| -------- | ---- | ------ | --------- | ------ | ------- |
| `byte`   | 1    | 0      | 255       | No     | 0       |
| `word`   | 2    | 0      | 65535     | No     | 0       |
| `sbyte`  | 1    | -128   | 127       | Yes    | 0       |
| `sword`  | 2    | -32768 | 32767     | Yes    | 0       |
| `bool`   | 1    | false  | true      | -      | false   |
| `string` | var  | -      | 255 chars | -      | ""      |
| `byte[]` | var  | -      | 256 elem  | No     | zeros   |
| `word[]` | var  | -      | 128 elem  | No     | zeros   |
