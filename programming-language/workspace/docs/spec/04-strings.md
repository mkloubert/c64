# Strings and Escape Sequences

This document defines string literals, character literals, and escape sequences.

## String Literals

Strings are enclosed in double quotes:

```
"Hello, World!"
"PLAYER 1"
"Score: 0"
""              # Empty string
```

### String Encoding

- Strings are stored as null-terminated byte arrays
- Characters are converted to PETSCII (C64 character set)
- Maximum string length: 255 characters (plus null terminator)

### String Memory Layout

```
# String "HI" in memory:
# Address:  $1000  $1001  $1002
# Value:    $48    $49    $00
#           'H'    'I'    null
```

---

## Character Literals

Single characters are enclosed in single quotes:

```
'A'
'0'
' '     # Space
'\n'    # Newline (escape sequence)
```

### Character Values

Characters are stored as single bytes in PETSCII encoding:

| Character | PETSCII Value               |
| --------- | --------------------------- |
| 'A'-'Z'   | $41-$5A (65-90)             |
| 'a'-'z'   | $61-$7A (97-122) or $C1-$DA |
| '0'-'9'   | $30-$39 (48-57)             |
| ' '       | $20 (32)                    |

---

## Escape Sequences

Escape sequences start with backslash (`\`):

| Sequence | Meaning                                 | PETSCII Value |
| -------- | --------------------------------------- | ------------- |
| `\n`     | Newline (cursor down + carriage return) | $0D           |
| `\r`     | Carriage return                         | $0D           |
| `\t`     | Tab (move cursor right)                 | -             |
| `\\`     | Backslash                               | $5C           |
| `\"`     | Double quote                            | $22           |
| `\'`     | Single quote                            | $27           |
| `\0`     | Null character                          | $00           |
| `\{`     | Left brace                              | $7B           |
| `\}`     | Right brace                             | $7D           |

### PETSCII Control Characters

| Sequence    | Meaning      | PETSCII |
| ----------- | ------------ | ------- |
| `\[CLR]`    | Clear screen | $93     |
| `\[HOME]`   | Cursor home  | $13     |
| `\[UP]`     | Cursor up    | $91     |
| `\[DOWN]`   | Cursor down  | $11     |
| `\[LEFT]`   | Cursor left  | $9D     |
| `\[RIGHT]`  | Cursor right | $1D     |
| `\[RVS]`    | Reverse on   | $12     |
| `\[RVSOFF]` | Reverse off  | $92     |

### Color Escape Sequences

| Sequence | Color  |
| -------- | ------ |
| `\[BLK]` | Black  |
| `\[WHT]` | White  |
| `\[RED]` | Red    |
| `\[CYN]` | Cyan   |
| `\[PUR]` | Purple |
| `\[GRN]` | Green  |
| `\[BLU]` | Blue   |
| `\[YEL]` | Yellow |

### Hex Escape Sequence

For any byte value:

```
"\x00"      # Null byte
"\x0D"      # Carriage return
"\xFF"      # Byte value 255
```

---

## String Examples

### Basic Strings

```
string greeting = "HELLO WORLD"
string empty = ""
string quoted = "He said \"Hi\""
```

### Multi-line Strings

Multi-line strings are not directly supported. Use concatenation:

```
string msg = "LINE 1\n"
msg = msg + "LINE 2\n"
msg = msg + "LINE 3"
```

Or define multiple strings:

```
string line1 = "WELCOME TO"
string line2 = "THE GAME!"

println(line1)
println(line2)
```

### PETSCII Control Examples

```
# Clear screen and print centered text
print("\[CLR]")
print("          GAME TITLE")

# Colored text
print("\[RED]ERROR: \[WHT]File not found")

# Reverse video
print("\[RVS]HIGHLIGHTED\[RVSOFF] Normal")
```

---

## String Operations

### Printing

```
print("Hello")          # Print without newline
println("Hello")        # Print with newline
print("A")              # Single character
```

### Comparison

```
if name == "PLAYER":
    # Strings are compared byte-by-byte
```

### Length

```
byte len = strlen(name)     # Get string length
```

### Character Access

```
string text = "HELLO"
byte ch = text[0]           # 'H' (first character)
byte last = text[4]         # 'O' (fifth character)
```

**Note:** No bounds checking. Accessing beyond string length is undefined.

---

## String Limitations

1. **Fixed allocation**: String variables have fixed maximum size
2. **No dynamic strings**: Cannot create strings at runtime
3. **Limited operations**: No built-in substring, concat operators
4. **PETSCII only**: No Unicode support

### Memory Considerations

Each string variable reserves 256 bytes by default:

- 255 bytes for characters
- 1 byte for null terminator

For memory-constrained programs, use explicit sizing:

```
string[16] short_name       # Only 16 bytes reserved
string[64] message          # 64 bytes reserved
```

---

## PETSCII vs ASCII

The C64 uses PETSCII, which differs from ASCII:

| Difference          | ASCII   | PETSCII              |
| ------------------- | ------- | -------------------- |
| Uppercase letters   | $41-$5A | $41-$5A (same)       |
| Lowercase letters   | $61-$7A | $C1-$DA (different!) |
| Graphics characters | None    | $A0-$BF, $E0-$FE     |

### Automatic Conversion

The compiler converts string literals to PETSCII:

- Lowercase letters are mapped to PETSCII lowercase
- Special characters are preserved where possible

### Screen Codes vs PETSCII

For direct screen memory access, use screen codes (different from PETSCII):

```
# PETSCII 'A' = $41, but screen code 'A' = $01
poke($0400, 1)      # Put 'A' at top-left (screen code)
```

---

## Error Messages

| Error               | Message                                                  |
| ------------------- | -------------------------------------------------------- |
| Unterminated string | `Error: Unterminated string literal`                     |
| Invalid escape      | `Error: Invalid escape sequence '\q'`                    |
| String too long     | `Error: String exceeds maximum length of 255 characters` |
| Invalid character   | `Error: Invalid character in string literal`             |
