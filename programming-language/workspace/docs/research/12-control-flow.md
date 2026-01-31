# Control Flow Implementation

This document describes how control flow structures (if, while, for) map to 6510 assembly.

## If-Then-Else

### Source Code

```
if score > 100:
    lives = lives + 1
else:
    lives = lives - 1
```

### Generated Assembly Pattern

```asm
        ; Load score
        LDA score
        ; Compare with 100
        CMP #100
        ; Branch if not greater (<=)
        BCC else_label      ; Branch if Carry Clear (less than)
        BEQ else_label      ; Branch if Equal

then_label:
        ; lives = lives + 1
        INC lives
        JMP endif_label

else_label:
        ; lives = lives - 1
        DEC lives

endif_label:
        ; Continue...
```

### 6502 Branch Instructions

| Instruction | Meaning               | Condition                           |
| ----------- | --------------------- | ----------------------------------- |
| BEQ         | Branch if Equal       | Z=1                                 |
| BNE         | Branch if Not Equal   | Z=0                                 |
| BCC         | Branch if Carry Clear | C=0 (less than for unsigned)        |
| BCS         | Branch if Carry Set   | C=1 (greater or equal for unsigned) |
| BMI         | Branch if Minus       | N=1 (negative)                      |
| BPL         | Branch if Plus        | N=0 (positive or zero)              |

### Comparison Patterns

For **unsigned** comparisons (byte, word):

| Condition | Assembly                             |
| --------- | ------------------------------------ |
| `a == b`  | `CMP` then `BEQ`                     |
| `a != b`  | `CMP` then `BNE`                     |
| `a < b`   | `CMP` then `BCC`                     |
| `a >= b`  | `CMP` then `BCS`                     |
| `a > b`   | `CMP` then `BEQ` to skip, then `BCS` |
| `a <= b`  | `CMP` then `BCC` or `BEQ`            |

For **signed** comparisons, use overflow flag (more complex).

---

## While Loop

### Source Code

```
while count > 0:
    print(count)
    count = count - 1
```

### Generated Assembly Pattern

```asm
while_start:
        ; Check condition: count > 0
        LDA count
        BEQ while_end       ; If count == 0, exit
        ; (for > 0, we only need to check not zero for unsigned)

        ; Loop body
        LDA count
        JSR print_byte      ; Print the count

        DEC count           ; count = count - 1

        JMP while_start     ; Repeat

while_end:
        ; Continue after loop
```

### Optimization: Compare at End

More efficient pattern for simple loops:

```asm
        ; Initialize
        JMP while_check

while_body:
        ; Loop body here
        LDA count
        JSR print_byte
        DEC count

while_check:
        LDA count
        BNE while_body      ; Continue if count != 0

while_end:
```

---

## For Loop

### Source Code

```
for i in 0 to 9:
    print(i)
```

### Generated Assembly Pattern

```asm
        ; Initialize i = 0
        LDA #0
        STA i

for_loop:
        ; Loop body
        LDA i
        JSR print_byte

        ; Increment and check
        INC i
        LDA i
        CMP #10             ; Compare with end+1
        BNE for_loop        ; Continue if not equal

for_end:
```

### Countdown Loop (More Efficient)

```
for i in 9 downto 0:
    print(i)
```

```asm
        ; Initialize i = 9
        LDA #9
        STA i

for_loop:
        ; Loop body
        LDA i
        JSR print_byte

        ; Decrement and check
        DEC i
        BPL for_loop        ; Continue if >= 0

for_end:
```

The countdown pattern is more efficient because:

- DEC sets the Zero and Negative flags
- BPL (Branch if Plus) is a single instruction check

---

## Break and Continue

### Break

```
while true:
    if key_pressed():
        break
    update()
```

```asm
while_start:
        ; Check key_pressed()
        JSR key_pressed
        BNE while_end       ; If non-zero, break

        ; update()
        JSR update

        JMP while_start

while_end:
```

### Continue

```
for i in 0 to 9:
    if i == 5:
        continue
    print(i)
```

```asm
        LDA #0
        STA i

for_loop:
        ; Check if i == 5
        LDA i
        CMP #5
        BEQ for_continue    ; Skip to increment

        ; print(i)
        LDA i
        JSR print_byte

for_continue:
        ; Increment
        INC i
        LDA i
        CMP #10
        BNE for_loop

for_end:
```

---

## Nested Loops

```
for y in 0 to 24:
    for x in 0 to 39:
        char_at(x, y, '*')
```

```asm
        LDA #0
        STA y

outer_loop:
        LDA #0
        STA x

inner_loop:
        ; char_at(x, y, '*')
        LDA x
        LDX y
        LDY #'*'
        JSR char_at

        ; Inner loop increment
        INC x
        LDA x
        CMP #40
        BNE inner_loop

        ; Outer loop increment
        INC y
        LDA y
        CMP #25
        BNE outer_loop

loops_end:
```

---

## Boolean Short-Circuit Evaluation

### AND (short-circuit)

```
if a > 0 and b > 0:
    do_something()
```

```asm
        ; Check first condition
        LDA a
        BEQ skip            ; If a == 0, skip (short-circuit)

        ; Check second condition
        LDA b
        BEQ skip            ; If b == 0, skip

        ; Both conditions true
        JSR do_something

skip:
```

### OR (short-circuit)

```
if a > 0 or b > 0:
    do_something()
```

```asm
        ; Check first condition
        LDA a
        BNE do_it           ; If a != 0, do it (short-circuit)

        ; Check second condition
        LDA b
        BEQ skip            ; If b == 0, skip

do_it:
        JSR do_something

skip:
```

---

## Implementation Considerations

### Branch Range Limitation

6502 branch instructions have a limited range: -128 to +127 bytes.

For distant jumps, use:

```asm
        BEQ local_skip
        JMP far_target
local_skip:
```

### Flag Preservation

Be careful: operations between CMP and branch can modify flags.

```asm
        CMP value
        ; DON'T do anything that changes flags here!
        BEQ target
```

### Loop Variable Scope

After a loop, the loop variable may have an undefined value:

- Compiler can optimize away final store
- Don't rely on loop variable value after loop

## References

- [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html)
- [6502 Branch Instructions](https://www.c64-wiki.com/wiki/BNE)
- [prog8 Control Flow](https://prog8.readthedocs.io/en/stable/programming.html)
