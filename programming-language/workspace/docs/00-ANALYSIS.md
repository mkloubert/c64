# Feasibility Analysis: Modern C64 Compiler

This document analyzes whether it is possible to create a cross-compiler that runs on modern systems (Linux, Windows, macOS) and produces D64 disk images with programs for the Commodore 64.

## Executive Summary

**Result: Fully feasible.**

Creating a cross-compiler for the C64 is not only possible but has been done multiple times by others. The necessary documentation, tools, and knowledge are available. A working compiler can be built with moderate effort using Rust, resulting in a fast native binary.

---

## Key Questions and Answers

### 1. Can we generate valid 6510 machine code?

**Yes.**

The 6510 instruction set is well-documented:
- 56 instructions with 151 valid opcodes
- Simple encoding (1-3 bytes per instruction)
- Well-defined addressing modes

**Evidence:**
- cc65, oscar64, and others generate valid 6510 code
- Detailed opcode documentation exists (see [6502.org](http://www.6502.org/tutorials/6502opcodes.html))
- Emulators (VICE) can verify generated code

**Implementation complexity:** Low to medium.

### 2. Can we create valid PRG files?

**Yes.**

PRG format is trivial:
- 2-byte load address (little-endian)
- Followed by raw machine code

**Implementation complexity:** Very low.

### 3. Can we create valid D64 disk images?

**Yes.**

D64 format is well-documented:
- Fixed structure (683 sectors × 256 bytes = 174,848 bytes)
- BAM (Block Availability Map) format is known
- Directory structure is documented

**Options:**
1. Use existing tool (c1541 from VICE)
2. Implement D64 writer ourselves (medium complexity)

**Implementation complexity:** Medium (if self-implemented).

### 4. Can we design and parse a new language?

**Yes.**

Standard compiler techniques work:
- Lexical analysis (tokenization)
- Parsing (recursive descent is sufficient)
- AST generation and analysis

**Rust tools available:**
- **logos** - Fast lexer generator via derive macros
- **lalrpop** - LR(1) parser generator
- **pest** - PEG-based parser
- **nom** - Parser combinator library
- Hand-written recursive descent parser (full control)

**Implementation complexity:** Medium.

### 5. Can the generated programs actually run on C64/emulator?

**Yes.**

Programs can be tested:
- VICE emulator runs on all major platforms
- VICE can be automated (command line, remote monitor)
- Other emulators also available

**Verification process:**
1. Generate PRG
2. Create D64 (or test PRG directly)
3. Load in VICE
4. Verify execution

---

## Technical Requirements

### Host System Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Linux, Windows, or macOS |
| Rust | 1.70+ (for building the compiler) |
| Disk space | Minimal (~50 MB for compiler binary) |
| Testing | VICE emulator (optional but recommended) |

### Compiler Binary

The compiler is distributed as a single native binary:
- No runtime dependencies (statically linked)
- Fast execution (compiled Rust)
- Cross-platform (builds for all major OS)

### Output Requirements

| Output | Format | Complexity |
|--------|--------|------------|
| PRG file | 2-byte header + code | Very simple |
| D64 file | 174,848 bytes structured | Medium |

### Language Requirements (proposed)

The new language should be:
- Simpler than BASIC V2
- More readable than assembly
- Provide easy access to C64 hardware (VIC-II, SID)

---

## Proposed Compiler Architecture

```
Source File (.c64)
       ↓
   ┌───────┐
   │ Lexer │ → Tokens
   └───────┘
       ↓
   ┌────────┐
   │ Parser │ → AST
   └────────┘
       ↓
   ┌──────────┐
   │ Analyzer │ → Typed AST
   └──────────┘
       ↓
   ┌──────────┐
   │ CodeGen  │ → 6510 Assembly
   └──────────┘
       ↓
   ┌───────────┐
   │ Assembler │ → Machine Code
   └───────────┘
       ↓
   ┌────────────┐
   │ PRG Writer │ → .prg file
   └────────────┘
       ↓
   ┌────────────┐
   │ D64 Writer │ → .d64 file
   └────────────┘
```

### Components to Implement (Rust)

| Component | Lines of Code (est.) | Complexity |
|-----------|---------------------|------------|
| Lexer (logos) | 150-250 | Low |
| Parser (lalrpop/hand) | 400-600 | Medium |
| AST types | 150-250 | Low |
| Analyzer | 300-500 | Medium |
| IR layer | 200-300 | Medium |
| Code generator (6510) | 600-900 | Medium-High |
| PRG writer | 50-100 | Very low |
| D64 writer | 300-500 | Medium |
| CLI (clap) | 100-150 | Low |
| **Total** | **2,250-3,550** | **Medium** |

---

## Risk Assessment

### Low Risk

| Risk | Mitigation |
|------|------------|
| Unknown PRG format | Format is trivial and documented |
| Unknown D64 format | Format is documented; c1541 available as fallback |
| 6510 instruction encoding | Many references available |

### Medium Risk

| Risk | Mitigation |
|------|------------|
| Code generation quality | Start simple, optimize later |
| Language design flaws | Iterate based on testing |
| Complex C64 hardware access | Use KERNAL routines initially |

### Low/Manageable Risk

| Risk | Mitigation |
|------|------------|
| Testing effort | Automate with VICE |
| Documentation | Write as we go |

---

## Comparison with Existing Solutions

| Feature | cc65 | oscar64 | Our Compiler |
|---------|------|---------|--------------|
| Source language | C | C/C++ | New (simple) |
| Output quality | Good | Excellent | TBD |
| Hardware access | Via library | Via library | Built-in syntax |
| Learning curve | Medium | Medium | Low (goal) |
| Implementation | Complex | Complex | Moderate |

### Our Advantages

1. **Simple syntax** - Not constrained by C compatibility
2. **Built-in hardware access** - `screen.color = red` instead of POKE
3. **Educational value** - Understanding the full stack
4. **Customizable** - We control everything

---

## Implementation Strategy

### Phase 1: Proof of Concept

1. Generate "Hello World" PRG manually
2. Test in VICE
3. Implement minimal assembler
4. Generate simple program from assembly

### Phase 2: Minimal Compiler

1. Design basic language syntax
2. Implement lexer
3. Implement parser
4. Implement code generator (direct to assembly)
5. Generate working PRG from source

### Phase 3: D64 Support

1. Implement D64 writer (or integrate c1541)
2. Add autostart support (BASIC stub)
3. Test full workflow

### Phase 4: Language Features

1. Variables and expressions
2. Control flow (if, while, for)
3. Functions/procedures
4. Hardware abstractions (screen, sound)

### Phase 5: Optimization

1. Constant folding
2. Dead code elimination
3. Zero-page allocation optimization
4. Peephole optimization

---

## Conclusion

### Feasibility: CONFIRMED

Building a cross-compiler for the C64 is:
- **Technically possible** - All required information is available
- **Practically achievable** - Moderate complexity, well-defined scope
- **Testable** - Emulators provide immediate feedback
- **Educational** - Valuable learning experience

### Recommended Approach

1. Use Rust for implementation (native binary, fast, safe)
2. Use logos for lexing, lalrpop or hand-written parser
3. Start with minimal viable compiler
4. Use VICE for testing
5. Implement D64 writer natively (no external dependencies)

### Success Criteria

The project will be successful when:
1. A source file in the new language compiles without errors
2. The resulting PRG runs correctly in VICE
3. The PRG can be packaged in a D64 file
4. The D64 boots and runs the program in VICE

---

## References

### Hardware Documentation
- [C64 Memory Map](https://sta.c64.org/cbm64mem.html)
- [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html)
- [VIC-II Documentation](https://www.c64-wiki.com/wiki/VIC)
- [SID Documentation](https://www.c64-wiki.com/wiki/SID)

### File Formats
- [D64 Format Specification](https://ist.uwaterloo.ca/~schepers/formats/D64.TXT)
- [PRG File Format](http://justsolve.archiveteam.org/wiki/Commodore_64_binary_executable)

### Existing Tools
- [cc65](https://cc65.github.io/)
- [oscar64](https://github.com/drmortalwombat/oscar64)
- [VICE Emulator](https://vice-emu.sourceforge.io/)

### Compiler Design
- [Compilers 101](https://dev.to/lefebvre/compilers-101---overview-and-lexer-3i0m)
- [LALRPOP Parser Generator](https://github.com/lalrpop/lalrpop)
- [Logos Lexer Generator](https://docs.rs/logos/latest/logos/)
- [Parsing Strategies in Rust](https://willcrichton.net/notes/parsing-strategies-in-rust/)
