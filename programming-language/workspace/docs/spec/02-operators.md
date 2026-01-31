# Operators and Precedence

This document defines all operators and their precedence levels.

## Operator Precedence Table

Operators are listed from highest to lowest precedence.

| Level | Operators                   | Description                           | Associativity |
| ----- | --------------------------- | ------------------------------------- | ------------- | ---- |
| 1     | `()` `[]`                   | Grouping, Array index                 | Left          |
| 2     | `f()`                       | Function call                         | Left          |
| 3     | `-` `~` `not`               | Unary minus, bitwise NOT, logical NOT | Right         |
| 4     | `*` `/` `%`                 | Multiplication, Division, Modulo      | Left          |
| 5     | `+` `-`                     | Addition, Subtraction                 | Left          |
| 6     | `<<` `>>`                   | Bit shift left, right                 | Left          |
| 7     | `&`                         | Bitwise AND                           | Left          |
| 8     | `^`                         | Bitwise XOR                           | Left          |
| 9     | `                           | `                                     | Bitwise OR    | Left |
| 10    | `==` `!=` `<` `>` `<=` `>=` | Comparison                            | Left          |
| 11    | `and`                       | Logical AND                           | Left          |
| 12    | `or`                        | Logical OR                            | Left          |
| 13    | `=` `+=` `-=` etc.          | Assignment                            | Right         |

---

## Arithmetic Operators

| Operator    | Name           | Example | Result             |
| ----------- | -------------- | ------- | ------------------ |
| `+`         | Addition       | `5 + 3` | `8`                |
| `-`         | Subtraction    | `5 - 3` | `2`                |
| `*`         | Multiplication | `5 * 3` | `15`               |
| `/`         | Division       | `7 / 2` | `3` (integer)      |
| `%`         | Modulo         | `7 % 3` | `1`                |
| `-` (unary) | Negation       | `-5`    | `-5` (signed only) |

### Notes

- Division is integer division (truncates toward zero)
- Multiplication and division are slow on 6502 (no hardware support)
- Division by zero causes undefined behavior
- Modulo only works with positive divisor

---

## Comparison Operators

| Operator | Name             | Example  | Result |
| -------- | ---------------- | -------- | ------ |
| `==`     | Equal            | `5 == 5` | `true` |
| `!=`     | Not equal        | `5 != 3` | `true` |
| `<`      | Less than        | `3 < 5`  | `true` |
| `>`      | Greater than     | `5 > 3`  | `true` |
| `<=`     | Less or equal    | `3 <= 3` | `true` |
| `>=`     | Greater or equal | `5 >= 3` | `true` |

### Notes

- Comparisons return `bool` type
- Comparing different types: smaller type is promoted
- Signed vs unsigned comparison may produce unexpected results

---

## Logical Operators

| Operator | Name        | Example          | Result  |
| -------- | ----------- | ---------------- | ------- |
| `and`    | Logical AND | `true and false` | `false` |
| `or`     | Logical OR  | `true or false`  | `true`  |
| `not`    | Logical NOT | `not true`       | `false` |

### Short-Circuit Evaluation

- `and`: If left operand is false, right operand is not evaluated
- `or`: If left operand is true, right operand is not evaluated

```
# Safe: array bounds check before access
if i < 10 and data[i] > 0:
    process(data[i])
```

---

## Bitwise Operators

| Operator | Name        | Example     | Result |
| -------- | ----------- | ----------- | ------ | ---- | ----- |
| `&`      | Bitwise AND | `$0F & $F0` | `$00`  |
| `        | `           | Bitwise OR  | `$0F   | $F0` | `$FF` |
| `^`      | Bitwise XOR | `$FF ^ $0F` | `$F0`  |
| `~`      | Bitwise NOT | `~$0F`      | `$F0`  |
| `<<`     | Left shift  | `$01 << 4`  | `$10`  |
| `>>`     | Right shift | `$80 >> 4`  | `$08`  |

### Notes

- Shift amounts should be 0-7 for byte, 0-15 for word
- Shifting by >= type size is undefined
- Right shift is logical (zero-fill), not arithmetic

### Common Bit Patterns

```
# Set bit n
value = value | (1 << n)

# Clear bit n
value = value & ~(1 << n)

# Toggle bit n
value = value ^ (1 << n)

# Test bit n
if value & (1 << n):
    # bit is set

# Extract low nibble
low = value & $0F

# Extract high nibble
high = (value >> 4) & $0F
```

---

## Assignment Operators

| Operator | Name               | Equivalent   |
| -------- | ------------------ | ------------ | ------ | --- |
| `=`      | Assignment         | -            |
| `+=`     | Add assign         | `a = a + b`  |
| `-=`     | Subtract assign    | `a = a - b`  |
| `*=`     | Multiply assign    | `a = a * b`  |
| `/=`     | Divide assign      | `a = a / b`  |
| `%=`     | Modulo assign      | `a = a % b`  |
| `&=`     | AND assign         | `a = a & b`  |
| `        | =`                 | OR assign    | `a = a | b`  |
| `^=`     | XOR assign         | `a = a ^ b`  |
| `<<=`    | Left shift assign  | `a = a << b` |
| `>>=`    | Right shift assign | `a = a >> b` |

### Notes

- Assignment is a statement, not an expression
- Cannot chain assignments: `a = b = c` is not allowed
- Compound assignments may be more efficient

---

## Type Promotion Rules

When operators have operands of different types:

| Left Type | Right Type | Result Type |
| --------- | ---------- | ----------- |
| `byte`    | `byte`     | `byte`      |
| `byte`    | `word`     | `word`      |
| `word`    | `byte`     | `word`      |
| `word`    | `word`     | `word`      |
| `sbyte`   | `sbyte`    | `sbyte`     |
| `sbyte`   | `sword`    | `sword`     |
| `bool`    | `bool`     | `bool`      |

### Overflow Behavior

- `byte` operations wrap at 256 (0-255)
- `word` operations wrap at 65536 (0-65535)
- No automatic overflow detection

---

## Operator Examples

### Precedence Examples

```
# Multiplication before addition
a + b * c       # Parsed as: a + (b * c)

# Comparison before logical
a < b and c > d # Parsed as: (a < b) and (c > d)

# Bitwise before comparison
a & b == 0      # Parsed as: (a & b) == 0

# Use parentheses for clarity
(a + b) * c     # Force addition first
```

### Common Patterns

```
# Check if value is in range
if value >= min and value <= max:
    in_range()

# Check multiple flags
if flags & (FLAG_A | FLAG_B):
    has_either_flag()

# Swap nibbles
swapped = (value << 4) | (value >> 4)

# Check if power of 2
if value != 0 and (value & (value - 1)) == 0:
    is_power_of_2()
```

---

## Performance Notes

### Fast Operations (1-2 cycles)

- Addition, Subtraction
- Bitwise AND, OR, XOR
- Left/Right shift by 1
- Comparison

### Slow Operations (many cycles)

- Multiplication (software routine)
- Division (software routine)
- Shift by variable amount

### Optimization Tips

```
# Instead of multiplication by 2
x = x * 2       # Slow
x = x << 1      # Fast

# Instead of division by 2
x = x / 2       # Slow
x = x >> 1      # Fast

# Instead of modulo 256
x = x % 256     # Slow
x = x & $FF     # Fast (for byte, automatic)

# Instead of multiplication by 10
x = x * 10              # Slow
x = (x << 3) + (x << 1) # Fast: 8x + 2x = 10x
```
