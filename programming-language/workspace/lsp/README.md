# Cobra64 Language Support for VS Code

<p align="center">
  <img src="icons/cobra64.png" alt="Cobra64 Logo" width="256">
</p>

A full-featured VS Code extension providing language support for Cobra64 - a modern Python-like programming language that compiles to Commodore 64 binaries.

## Features

### Syntax Highlighting

- Full syntax highlighting for all Cobra64 language constructs
- Support for keywords, types, operators, and built-in functions
- Hex ($FF) and binary (%1010) literal highlighting
- Comment highlighting

### Diagnostics (Real-time Error Checking)

- Lexer errors (invalid characters, unterminated strings)
- Parser errors (syntax errors, unexpected tokens)
- Semantic errors (undefined variables, type mismatches)
- Errors and warnings displayed inline and in Problems panel

### IntelliSense

- **Auto-completion** for:
  - Keywords (def, if, while, for, etc.)
  - Types (byte, word, sbyte, sword, fixed, float, bool, string)
  - Built-in functions (cls, print, println, peek, poke, etc.)
  - User-defined functions and variables
  - Context-aware suggestions (types after `:`, etc.)

- **Signature Help** for function calls:
  - Parameter information while typing
  - Active parameter highlighting
  - Documentation for built-in functions

### Navigation

- **Go to Definition** (Ctrl+Click or F12)
  - Jump to variable/constant declarations
  - Jump to function definitions

- **Find All References** (Shift+F12)
  - Find all usages of a symbol

- **Document Symbols** (Ctrl+Shift+O)
  - Outline view of functions, variables, constants

- **Workspace Symbols** (Ctrl+T)
  - Search symbols across all .cb64 files

### Hover Information

- Type information for variables and constants
- Function signatures with parameter types
- Documentation for built-in functions
- Keyword descriptions with examples

### Code Editing

- **Rename Symbol** (F2)
  - Rename variables, constants, and functions
  - Updates all references automatically

- **Document Highlighting**
  - Highlights all occurrences of a symbol
  - Distinguishes between read and write access

- **Folding**
  - Fold functions, loops, and conditionals
  - Fold comment blocks

- **Inlay Hints**
  - Type hints for inferred variables
  - Parameter name hints in function calls

### Quick Fixes

- Suggest similar names for typos
- Create variable/function stubs
- Add type annotations

## Supported Language Features

### Data Types

| Type     | Description                | Range                 |
| -------- | -------------------------- | --------------------- |
| `byte`   | Unsigned 8-bit integer     | 0 to 255              |
| `word`   | Unsigned 16-bit integer    | 0 to 65535            |
| `sbyte`  | Signed 8-bit integer       | -128 to 127           |
| `sword`  | Signed 16-bit integer      | -32768 to 32767       |
| `fixed`  | Fixed-point decimal (12.4) | -2048.0 to +2047.9375 |
| `float`  | IEEE-754 binary16          | +/-65504              |
| `bool`   | Boolean                    | true/false            |
| `string` | Text string                | Variable length       |
| `T[]`    | Arrays                     | Any base type         |

### Keywords

```
def, if, elif, else, while, for, in, to, downto,
break, continue, return, pass, and, or, not, true, false
```

### Built-in Functions

**Screen:**

- `cls()` - Clear screen
- `cursor(x, y)` - Move cursor

**I/O:**

- `print(value)` - Print without newline
- `println(value)` - Print with newline
- `get_key()` - Get current key
- `read()` - Wait for key
- `readln()` - Read line of text

**Memory:**

- `poke(addr, val)` - Write to memory
- `peek(addr)` - Read from memory

**Arrays:**

- `len(array)` - Get array length

**Random:**

- `rand()` - Random fixed-point 0.0-0.9375
- `rand_byte(from, to)` - Random byte in range
- `rand_sbyte(from, to)` - Random signed byte
- `rand_word(from, to)` - Random word
- `rand_sword(from, to)` - Random signed word
- `seed()` - Reseed RNG

## Example Code

```python
# Simple Cobra64 program
MAX_SCORE: byte = 100

def add(a: byte, b: byte) -> byte:
    return a + b

def main():
    cls()
    score: byte = 0

    for i in 1 to 10:
        score = add(score, i)
        println(score)

    if score >= MAX_SCORE:
        println("High score!")
    else:
        println("Keep trying!")
```

## Installation

### From VSIX

1. Download the `.vsix` file
2. Open VS Code
3. Go to Extensions (Ctrl+Shift+X)
4. Click "..." menu then "Install from VSIX..."
5. Select the downloaded file

### For Development

1. Clone the repository
2. Run `npm install`
3. Run `npm run compile`
4. Press F5 to launch Extension Development Host

## Configuration

| Setting                       | Description                         | Default |
| ----------------------------- | ----------------------------------- | ------- |
| `cobra64.maxNumberOfProblems` | Maximum problems to report per file | 100     |
| `cobra64.trace.server`        | Trace communication with server     | "off"   |

## Requirements

- VS Code 1.85.0 or higher

## Development

### Building

```bash
cd lsp
npm install
npm run compile
```

### Running Tests

```bash
npm run test:unit
```

### Packaging

```bash
npm run package
```

## Known Issues

- Multi-file project support is limited
- Some complex expressions may not parse correctly

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).

Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>
