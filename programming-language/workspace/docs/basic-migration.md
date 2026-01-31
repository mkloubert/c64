# Migration Guide: From BASIC to C64 Language

This guide helps BASIC programmers transition to our new language.

---

## Key Differences

| BASIC | New Language |
|-------|--------------|
| Line numbers | No line numbers |
| GOTO/GOSUB | Functions (def) |
| POKE/PEEK | Named functions |
| Untyped | Typed variables |
| Uppercase only | Case sensitive |

---

## Hello World

### BASIC

```basic
10 PRINT "HELLO WORLD"
```

### New Language

```
def main():
    println("HELLO WORLD")
```

---

## Variables

### BASIC

```basic
10 X = 10
20 X$ = "HELLO"
30 X% = 1000
```

### New Language

```
byte x = 10
string s = "HELLO"
word n = 1000
```

**Note:** Variables must be declared with a type.

---

## Printing

### BASIC

```basic
10 PRINT "SCORE:"; SC
20 PRINT "LINE 1"
30 PRINT "LINE 2"
```

### New Language

```
print("SCORE: ")
println(score)
println("LINE 1")
println("LINE 2")
```

**Note:** `print` doesn't add newline, `println` does.

---

## Screen Control

### BASIC

```basic
10 PRINT CHR$(147)
20 POKE 53280, 0
30 POKE 53281, 0
```

### New Language

```
cls()
screen_color(BLACK, BLACK)
```

---

## Cursor Position

### BASIC

```basic
10 POKE 211, 10
20 POKE 214, 5
30 SYS 58732
40 PRINT "HELLO"
```

### New Language

```
cursor(10, 5)
print("HELLO")
```

---

## IF Statements

### BASIC

```basic
10 IF X > 10 THEN PRINT "BIG"
20 IF X > 10 THEN 100
```

### New Language

```
if x > 10:
    println("BIG")

if x > 10:
    do_something()
```

**Note:** Use indentation for blocks, no THEN keyword.

---

## IF-ELSE

### BASIC

```basic
10 IF X > 10 THEN PRINT "BIG": GOTO 30
20 PRINT "SMALL"
30 REM CONTINUE
```

### New Language

```
if x > 10:
    println("BIG")
else:
    println("SMALL")
```

---

## FOR Loops

### BASIC

```basic
10 FOR I = 1 TO 10
20 PRINT I
30 NEXT I
```

### New Language

```
for i in 1 to 10:
    println(i)
```

**Note:** No NEXT needed, use indentation.

---

## FOR Loops (Countdown)

### BASIC

```basic
10 FOR I = 10 TO 1 STEP -1
20 PRINT I
30 NEXT I
```

### New Language

```
for i in 10 downto 1:
    println(i)
```

---

## GOTO

### BASIC

```basic
10 PRINT "START"
20 GOTO 10
```

### New Language

```
while true:
    println("START")
```

**Note:** Use `while` loops instead of GOTO.

---

## GOSUB/RETURN

### BASIC

```basic
10 GOSUB 100
20 END
100 PRINT "HELLO"
110 RETURN
```

### New Language

```
def greet():
    println("HELLO")

def main():
    greet()
```

---

## POKE/PEEK

### BASIC

```basic
10 POKE 53280, 0
20 X = PEEK(53280)
```

### New Language

```
poke($D020, 0)
byte x = peek($D020)
```

**Better:** Use named functions:

```
screen_color(BLACK, BLACK)
```

---

## Arrays

### BASIC

```basic
10 DIM A(10)
20 A(0) = 100
30 A(1) = 200
40 PRINT A(0)
```

### New Language

```
byte a[10]
a[0] = 100
a[1] = 200
println(a[0])
```

**Note:** Arrays are 0-indexed in both, but use `[]` not `()`.

---

## Joystick

### BASIC

```basic
10 J = PEEK(56320)
20 IF J AND 1 THEN PRINT "UP"
30 IF J AND 16 THEN PRINT "FIRE"
```

### New Language

```
byte j = joystick(2)
if j & JOY_UP:
    println("UP")
if j & JOY_FIRE:
    println("FIRE")
```

---

## Sound

### BASIC

```basic
10 POKE 54296, 15
20 POKE 54277, 9
30 POKE 54278, 0
40 POKE 54273, 28
50 POKE 54272, 49
60 POKE 54276, 33
```

### New Language

```
sound_init()
volume(15)
voice(1, WAVE_SAW, 7000, 0, 9, 0, 0)
voice_on(1)
```

---

## Sprites

### BASIC

```basic
10 POKE 53248, 100: REM X POS
20 POKE 53249, 100: REM Y POS
30 POKE 53287, 1: REM COLOR
40 POKE 53269, 1: REM ENABLE
```

### New Language

```
sprite_pos(0, 100, 100)
sprite_color(0, WHITE)
sprite_enable(0, true)
```

---

## Complete Example Comparison

### BASIC

```basic
10 PRINT CHR$(147)
20 POKE 53280, 6: POKE 53281, 6
30 X = 20
40 Y = 12
50 J = PEEK(56320)
60 IF J AND 1 THEN Y = Y - 1
70 IF J AND 2 THEN Y = Y + 1
80 IF J AND 4 THEN X = X - 1
90 IF J AND 8 THEN X = X + 1
100 POKE 211, X: POKE 214, Y
110 SYS 58732: PRINT "@"
120 IF J AND 16 THEN END
130 GOTO 50
```

### New Language

```
def main():
    cls()
    screen_color(BLUE, BLUE)

    byte x = 20
    byte y = 12

    while true:
        byte j = joystick(2)

        if j & JOY_UP and y > 0:
            y = y - 1
        if j & JOY_DOWN and y < 24:
            y = y + 1
        if j & JOY_LEFT and x > 0:
            x = x - 1
        if j & JOY_RIGHT and x < 39:
            x = x + 1

        cursor(x, y)
        print("@")

        if j & JOY_FIRE:
            break

        wait(2)
```

---

## Tips for BASIC Programmers

1. **No line numbers** - Code flows top to bottom
2. **Indentation matters** - 4 spaces per level
3. **Declare variables** - Always specify type
4. **Use functions** - Replace GOSUB with `def`
5. **Named constants** - Use `const` instead of magic numbers
6. **Use built-in functions** - Clearer than POKE/PEEK
7. **No GOTO** - Use loops and functions instead
