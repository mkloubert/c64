# Error Messages Catalog

This document lists all compiler error and warning messages.

## Error Message Format

```
filename.c64:line:column: error: message
    source line with error
    ^^^^^ indicator
```

Example:

```
game.c64:15:10: error: Undefined variable 'scroe'
    print(scroe)
          ^^^^^
    Hint: Did you mean 'score'?
```

---

## Lexical Errors

### Invalid Characters

| Code | Message                           |
| ---- | --------------------------------- |
| E001 | `Invalid character '{char}'`      |
| E002 | `Invalid character in identifier` |
| E003 | `Invalid digit in number literal` |

### String/Character Errors

| Code | Message                                                  |
| ---- | -------------------------------------------------------- |
| E010 | `Unterminated string literal`                            |
| E011 | `Unterminated character literal`                         |
| E012 | `Invalid escape sequence '\\{char}'`                     |
| E013 | `Empty character literal`                                |
| E014 | `Character literal too long (expected single character)` |
| E015 | `String exceeds maximum length of 255 characters`        |
| E016 | `Invalid hex escape sequence`                            |

### Number Errors

| Code | Message                                          |
| ---- | ------------------------------------------------ |
| E020 | `Integer literal too large for byte (max 255)`   |
| E021 | `Integer literal too large for word (max 65535)` |
| E022 | `Invalid binary digit (expected 0 or 1)`         |
| E023 | `Invalid hexadecimal digit`                      |
| E024 | `Number literal cannot be empty`                 |

### Comment Errors

| Code | Message                      |
| ---- | ---------------------------- |
| E030 | `Unterminated block comment` |

---

## Syntax Errors

### General Syntax

| Code | Message                                  |
| ---- | ---------------------------------------- |
| E100 | `Unexpected token '{token}'`             |
| E101 | `Expected '{expected}', found '{found}'` |
| E102 | `Expected expression`                    |
| E103 | `Expected statement`                     |
| E104 | `Invalid assignment target`              |

### Indentation Errors

| Code | Message                                                     |
| ---- | ----------------------------------------------------------- |
| E110 | `Expected indented block after ':'`                         |
| E111 | `Unexpected indentation`                                    |
| E112 | `Inconsistent indentation (expected {n} spaces, found {m})` |
| E113 | `Tab character not allowed (use 4 spaces)`                  |
| E114 | `Mixed tabs and spaces in indentation`                      |

### Declaration Errors

| Code | Message                                 |
| ---- | --------------------------------------- |
| E120 | `Expected type name`                    |
| E121 | `Expected variable name`                |
| E122 | `Expected constant value`               |
| E123 | `Array size must be a positive integer` |
| E124 | `Array size exceeds maximum (256)`      |

### Function Errors

| Code | Message                                 |
| ---- | --------------------------------------- |
| E130 | `Expected function name after 'def'`    |
| E131 | `Expected '(' after function name`      |
| E132 | `Expected ')' after parameters`         |
| E133 | `Expected ':' after function signature` |
| E134 | `Expected '->' before return type`      |
| E135 | `Duplicate parameter name '{name}'`     |

### Control Flow Errors

| Code | Message                                 |
| ---- | --------------------------------------- |
| E140 | `Expected ':' after condition`          |
| E141 | `'elif' without matching 'if'`          |
| E142 | `'else' without matching 'if'`          |
| E143 | `Expected 'to' or 'downto' in for loop` |
| E144 | `'break' outside of loop`               |
| E145 | `'continue' outside of loop`            |
| E146 | `'return' outside of function`          |

---

## Semantic Errors

### Variable Errors

| Code | Message                                        |
| ---- | ---------------------------------------------- |
| E200 | `Undefined variable '{name}'`                  |
| E201 | `Variable '{name}' already defined`            |
| E202 | `Cannot assign to constant '{name}'`           |
| E203 | `Variable '{name}' used before initialization` |

### Type Errors

| Code | Message                                                 |
| ---- | ------------------------------------------------------- |
| E210 | `Type mismatch: expected '{expected}', found '{found}'` |
| E211 | `Cannot convert '{from}' to '{to}'`                     |
| E212 | `Cannot apply operator '{op}' to type '{type}'`         |
| E213 | `Cannot compare '{type1}' with '{type2}'`               |
| E214 | `Array index must be integer type`                      |
| E215 | `Cannot index non-array type '{type}'`                  |

### Function Errors

| Code | Message                                                   |
| ---- | --------------------------------------------------------- |
| E220 | `Undefined function '{name}'`                             |
| E221 | `Function '{name}' already defined`                       |
| E222 | `Wrong number of arguments (expected {n}, got {m})`       |
| E223 | `Argument type mismatch for parameter '{param}'`          |
| E224 | `Missing return statement in function returning '{type}'` |
| E225 | `Cannot return value from void function`                  |
| E226 | `Missing return value (expected '{type}')`                |

### Constant Errors

| Code | Message                        |
| ---- | ------------------------------ |
| E230 | `Constant expression required` |
| E231 | `Array size must be constant`  |
| E232 | `Constant value out of range`  |

### Array Errors

| Code | Message                                   |
| ---- | ----------------------------------------- |
| E240 | `Array index out of bounds`               |
| E241 | `Array initializer has too many elements` |
| E242 | `Array initializer has too few elements`  |

---

## Warnings

### General Warnings

| Code | Message                                     |
| ---- | ------------------------------------------- |
| W001 | `Variable '{name}' declared but never used` |
| W002 | `Variable '{name}' assigned but never read` |
| W003 | `Unreachable code after 'return'`           |
| W004 | `Unreachable code after 'break'`            |

### Type Warnings

| Code | Message                                                     |
| ---- | ----------------------------------------------------------- |
| W010 | `Implicit conversion from '{from}' to '{to}' may lose data` |
| W011 | `Comparison between signed and unsigned types`              |
| W012 | `Integer overflow in constant expression`                   |

### Style Warnings

| Code | Message                                       |
| ---- | --------------------------------------------- |
| W020 | `Variable '{name}' shadows global variable`   |
| W021 | `Function '{name}' shadows built-in function` |
| W022 | `Empty statement (use 'pass' if intentional)` |

### Performance Warnings

| Code | Message                                                 |
| ---- | ------------------------------------------------------- |
| W030 | `Multiplication is slow on 6502, consider using shifts` |
| W031 | `Division is slow on 6502, consider using shifts`       |
| W032 | `Word operation where byte would suffice`               |

---

## Hints and Suggestions

The compiler may provide hints to fix errors:

### Spelling Suggestions

```
game.c64:10:5: error: Undefined variable 'scroe'
    Hint: Did you mean 'score'?
```

### Missing Import Suggestion

```
game.c64:5:1: error: Undefined function 'sprite_enable'
    Hint: This is a system function. Make sure you're targeting C64.
```

### Type Conversion Suggestion

```
game.c64:20:10: error: Cannot assign 'word' to 'byte'
    Hint: Use explicit conversion: 'score = byte(total)'
```

---

## Exit Codes

| Code | Meaning                      |
| ---- | ---------------------------- |
| 0    | Success                      |
| 1    | Compilation errors           |
| 2    | Command-line argument errors |
| 3    | File not found               |
| 4    | Internal compiler error      |

---

## Disabling Warnings

Specific warnings can be disabled:

```
# At top of file
#pragma warning(disable: W001)

# Or for specific line
x = 10  # pragma: ignore W002
```
