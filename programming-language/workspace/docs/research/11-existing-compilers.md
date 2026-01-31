# Existing C64 Cross-Compilers

This document summarizes existing cross-compilers for the C64 to learn from their approaches.

## prog8

**Repository:** https://github.com/irmen/prog8
**Documentation:** https://prog8.readthedocs.io/

### Overview

prog8 is a high-level programming language and cross-compiler specifically designed for 6502-based systems like the Commodore 64.

### Language Features

- **Data Types:** byte, word, float, bool, string
- **Loops:** for, while, do-until, repeat
- **Conditionals:** if-else, when (switch)
- **Functions:** sub, asmsub (assembly), extsub (external)
- **Special:** CPU flag conditionals (if_cs, if_z, etc.)

### Syntax Example

```
%import textio

main {
    sub start() {
        txt.print("hello world\n")
    }
}
```

### Architecture

1. **Parser:** ANTLR4 generates AST
2. **Intermediate Code:** Stack-based virtual machine opcodes
3. **Assembly Generation:** Pattern matching to 6502 assembly
4. **Assembler:** Uses 64tass external assembler

### Key Lessons

- "Dealing with anything but bytes on the 6502 quickly turns into a mess"
- Evaluation stack uses split LSB/MSB indexed by X register
- Parameters passed via fixed memory locations
- Code size is a major concern (must fit in ~40KB)

### Limitations

- Non-reentrant subroutines (no recursion)
- No advanced optimizations (no dataflow analysis)
- Significant code generation overhead for word/float operations

---

## oscar64

**Repository:** https://github.com/drmortalwombat/oscar64

### Overview

oscar64 is a C/C++ cross-compiler for the 6502 family, focused on the Commodore 64.

### Language Features

- Full C99 support
- Many C++ features (templates, lambdas)
- Extensions for C64-specific features
- Disk overlays and banked cartridges

### Compilation Strategy

- Direct 6502 compilation (faster and smaller than bytecode)
- Aggressive optimization
- Source-level debugging support

### Performance

- 418 Dhrystone iterations per second on C64
- Generated code can match hand-crafted assembly
- Developer writes games entirely in C/C++

### Key Lessons

- Native code generation is better than bytecode interpretation
- Library code included via pragma for full optimization
- Memory region pragmas for flexible memory layout

---

## cc65

**Website:** https://cc65.github.io/

### Overview

The classic C cross-compiler for 6502 systems, supporting many platforms including C64.

### Features

- Standard C compiler
- Macro assembler (ca65)
- Linker (ld65)
- Large library of platform-specific functions

### Architecture

- Traditional compiler toolchain
- Separate compilation and linking
- Library-based hardware access

---

## Comparison

| Feature         | prog8    | oscar64   | cc65      | Our Goal    |
| --------------- | -------- | --------- | --------- | ----------- |
| Language        | Custom   | C/C++     | C         | Python-like |
| Complexity      | Medium   | High      | Medium    | Low         |
| Syntax          | Unique   | C         | C         | Simple      |
| Learning curve  | Medium   | Medium    | Medium    | Low         |
| Hardware access | Built-in | Libraries | Libraries | Built-in    |
| Optimization    | Basic    | Advanced  | Good      | Basic       |

## What We Can Learn

### From prog8

1. **Block-based structure** works well for 6502
2. **Split LSB/MSB stacks** for 16-bit values
3. **Fixed memory for parameters** is simpler than stack-based
4. **ANTLR4** is viable for parsing
5. Keep **code size** as primary concern

### From oscar64

1. **Native code generation** is preferred over bytecode
2. **Include library source** for better optimization
3. **Memory region control** is useful

### From cc65

1. **Separate assembler step** allows debugging
2. **Platform-specific libraries** are maintainable
3. **Linker scripts** control memory layout

## Our Approach

Based on this analysis, our compiler should:

1. **Simple syntax** - Python-like, not C-like
2. **Direct to assembly** - Use external assembler (64tass)
3. **Built-in hardware functions** - No separate libraries
4. **Focus on bytes** - Minimize word/float usage
5. **Fixed parameters** - Use memory locations, not stack
6. **Small code** - Prioritize size over speed

## References

- [prog8 Wiki](https://github.com/irmen/prog8/wiki)
- [prog8 Documentation](https://prog8.readthedocs.io/)
- [oscar64 GitHub](https://github.com/drmortalwombat/oscar64)
- [cc65 GitHub](https://cc65.github.io/)
