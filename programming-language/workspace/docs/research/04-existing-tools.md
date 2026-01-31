# Existing C64 Development Tools

This document surveys existing compilers, assemblers, and tools for C64 development.

## C Compilers

### cc65

The most established C cross-compiler for 6502 systems.

| Property      | Value                    |
| ------------- | ------------------------ |
| **Language**  | C (mostly C99 compliant) |
| **License**   | zlib license             |
| **Platforms** | Linux, Windows, macOS    |
| **Website**   | https://cc65.github.io/  |

**Features:**

- Full C compiler (Small-C origin)
- Macro assembler (ca65)
- Linker (ld65)
- Librarian (ar65)
- Runtime library for C64 and many other systems

**Pros:**

- Mature and well-documented
- Large community
- Supports many 6502 platforms

**Cons:**

- Generates slower code than hand-written assembly
- C calling conventions have overhead on 6502

### Oscar64

Modern optimizing C/C++ compiler specifically for C64.

| Property       | Value                                     |
| -------------- | ----------------------------------------- |
| **Language**   | C99, partial C++ (up to C++17 features)   |
| **License**    | MIT                                       |
| **Platforms**  | Linux, Windows, macOS                     |
| **Repository** | https://github.com/drmortalwombat/oscar64 |

**Features:**

- Advanced optimizer
- C++ support (templates, lambdas)
- Direct 6502 code generation
- Very compact output

**Pros:**

- Produces faster and smaller code than cc65
- Modern C++ features
- Active development

**Cons:**

- Newer, less documentation
- Smaller community

### KickC

C compiler with Kick Assembler integration.

| Property     | Value                |
| ------------ | -------------------- |
| **Language** | C subset             |
| **License**  | Open source          |
| **Requires** | Java, Kick Assembler |

**Pros:**

- Generates efficient code
- Good Kick Assembler integration

**Cons:**

- Not fully C compliant
- Slow compilation for large projects
- Missing "fragments" can cause failures

### LLVM-MOS

Full LLVM/Clang port to 6502.

| Property       | Value                       |
| -------------- | --------------------------- |
| **Language**   | C, C++11                    |
| **License**    | Apache 2.0 / LLVM           |
| **Repository** | https://github.com/llvm-mos |

**Pros:**

- Full Clang/LLVM toolchain
- Whole-program optimization
- Modern standards support

**Cons:**

- Large toolchain
- Still maturing

---

## BASIC Cross-Compilers

### XC=BASIC

Optimizing cross-compiler for an enhanced BASIC dialect.

| Property     | Value                 |
| ------------ | --------------------- |
| **Language** | Extended BASIC        |
| **Author**   | Csaba Fekete          |
| **Website**  | https://xc-basic.net/ |

**Features:**

- Modern BASIC syntax
- Compiled to machine code
- Structs, arrays, procedures

### MOSpeed

BASIC V2 optimizer and cross-compiler.

- Compiles BASIC V2 programs
- Optimizes for speed
- Maintains compatibility with C64 BASIC

---

## Assemblers

### KickAssembler

Java-based powerful macro assembler.

| Property     | Value                               |
| ------------ | ----------------------------------- |
| **Requires** | Java                                |
| **Features** | Macros, scripting, vice integration |

**Pros:**

- Very powerful macro system
- Built-in scripting
- Excellent for demo coding

### ACME

Simple and fast cross-assembler.

| Property      | Value                          |
| ------------- | ------------------------------ |
| **Platforms** | Windows, Linux, macOS, AmigaOS |
| **Features**  | Multiple CPU support           |

### 64tass

Turbo Assembler compatible cross-assembler.

| Property          | Value                  |
| ----------------- | ---------------------- |
| **Compatibility** | Turbo Assembler syntax |
| **Platforms**     | Cross-platform         |

### ca65 (part of cc65)

Macro assembler from the cc65 suite.

- 6502, 65C02, 65816 support
- Powerful macro system
- Integrates with cc65 C compiler

---

## IDEs and Editors

### C64 Studio

Full-featured Windows IDE.

| Property     | Value               |
| ------------ | ------------------- |
| **Author**   | Georg Rottensteiner |
| **Platform** | Windows             |

**Features:**

- BASIC and assembly editing
- Integrated charpad/spritepad
- VICE integration
- Syntax highlighting

### CBM prg Studio

Another Windows IDE for C64 development.

- BASIC V2 support
- Assembly support
- Project management

### Relaunch64

Cross-platform Java-based editor.

- Assembly code editor
- Multiple assembler support
- VICE integration

### VS64

Visual Studio Code extension.

| Property       | Value                                  |
| -------------- | -------------------------------------- |
| **Repository** | https://github.com/rolandshacks/vs64   |
| **Supports**   | acme, kick, llvm, cc65, oscar64, basic |

**Features:**

- Syntax highlighting
- Build integration
- VICE debugging
- 6502 emulator for code analysis

---

## Disk Image Tools

### c1541 (VICE)

Official VICE disk image tool.

```bash
# Create D64
c1541 -format "name,id" d64 disk.d64

# Write file
c1541 -attach disk.d64 -write program.prg

# List contents
c1541 -attach disk.d64 -list
```

### cc1541

Alternative D64 creation tool.

- More control over sector placement
- Supports fast loaders
- Better for advanced disk layouts

### DirMaster

GUI-based disk image editor.

- Supports D64, D71, D81, G64, T64
- Drag-and-drop interface
- Search functionality

---

## Emulators

### VICE

The standard C64 emulator for development.

| Property      | Value                            |
| ------------- | -------------------------------- |
| **Website**   | https://vice-emu.sourceforge.io/ |
| **Platforms** | Cross-platform                   |

**Features:**

- Accurate emulation
- Debugger / monitor
- Supports all Commodore systems
- Snapshot and recording

### Hoxs64

Windows-only cycle-exact emulator.

- Very accurate
- Good debugging

### CCS64

Another accurate emulator.

- Windows-based
- REU support

---

## Graphics Tools

### CharPad

Character and tile editor.

### SpritePad

Sprite editor for C64.

### Pixcen

Bitmap graphics editor for C64 formats.

---

## Music Tools

### GoatTracker

SID music tracker.

| Property     | Value                   |
| ------------ | ----------------------- |
| **Platform** | Cross-platform          |
| **Output**   | SID player + music data |

### SID Wizard

Another SID music creation tool.

---

## Compression Tools

### Exomizer

Efficient compressor for C64 programs.

```bash
exomizer sfx $0810 program.prg -o compressed.prg
```

Creates self-extracting compressed executables.

### Pucrunch

Alternative compressor.

- Good compression ratio
- Fast decompression

---

## Build Systems

Many developers use Makefiles or custom scripts:

```makefile
# Example Makefile for cc65
CC = cl65
CFLAGS = -t c64 -O

%.prg: %.c
    $(CC) $(CFLAGS) -o $@ $<

disk.d64: program.prg
    c1541 -format "game,01" d64 $@
    c1541 -attach $@ -write program.prg
```

---

## Key Takeaways for New Compiler Design

1. **PRG generation is straightforward** - Just 2-byte header + code
2. **D64 creation has existing tools** - Can use c1541 or implement directly
3. **Many choose assembly for performance** - New language should generate efficient code
4. **cc65 is the baseline** - Should aim to match or exceed its output quality
5. **oscar64 shows modern approach works** - C++ features are implementable

---

## Sources

- [Cross-Development - C64-Wiki](https://www.c64-wiki.com/wiki/Cross-Development)
- [cc65 Homepage](https://cc65.github.io/)
- [oscar64 GitHub](https://github.com/drmortalwombat/oscar64)
- [KickAssembler](http://theweb.dk/KickAssembler/)
- [VICE Emulator](https://vice-emu.sourceforge.io/)
