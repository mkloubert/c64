# Example Programs

This document contains example programs for testing the compiler.

## 1. Hello World (Minimal)

The simplest possible program.

```
# hello.c64
# The classic first program

def main():
    print("HELLO WORLD!")
```

**Tests:** Basic program structure, print function, string literals.

---

## 2. Hello World (Extended)

More features of text output.

```
# hello2.c64
# Extended hello world with colors

def main():
    cls()
    screen_color(BLUE, LIGHTBLUE)
    text_color(WHITE)

    cursor(15, 12)
    println("HELLO WORLD!")

    cursor(10, 14)
    text_color(YELLOW)
    print("PRESS ANY KEY...")

    wait_key()
```

**Tests:** cls, screen_color, text_color, cursor, println, wait_key.

---

## 3. Variables and Arithmetic

Testing variable declarations and basic math.

```
# math.c64
# Variable and arithmetic tests

def main():
    cls()

    # Variable declarations
    byte a = 10
    byte b = 5
    byte result = 0

    # Arithmetic operations
    result = a + b
    print("10 + 5 = ")
    println(result)

    result = a - b
    print("10 - 5 = ")
    println(result)

    result = a * b
    print("10 * 5 = ")
    println(result)

    result = a / b
    print("10 / 5 = ")
    println(result)

    result = a % 3
    print("10 % 3 = ")
    println(result)
```

**Tests:** byte variables, initialization, arithmetic operators, print with numbers.

---

## 4. Word Variables

Testing 16-bit integers.

```
# words.c64
# Word (16-bit) variable tests

def main():
    cls()

    word score = 0
    word high_score = 10000

    score = 1234
    print("SCORE: ")
    println(score)

    score = score + 100
    print("AFTER +100: ")
    println(score)

    if score < high_score:
        println("KEEP TRYING!")
    else:
        println("NEW HIGH SCORE!")
```

**Tests:** word variables, large numbers, word arithmetic, word comparison.

---

## 5. If-Else Conditions

Testing conditional statements.

```
# conditions.c64
# If-else tests

def main():
    cls()

    byte x = 50

    # Simple if
    if x > 25:
        println("X IS GREATER THAN 25")

    # If-else
    if x == 100:
        println("X IS 100")
    else:
        println("X IS NOT 100")

    # If-elif-else
    if x < 25:
        println("SMALL")
    elif x < 75:
        println("MEDIUM")
    else:
        println("LARGE")

    # Logical operators
    byte y = 30
    if x > 40 and y > 20:
        println("BOTH CONDITIONS TRUE")

    if x > 100 or y > 20:
        println("AT LEAST ONE TRUE")

    if not (x == 0):
        println("X IS NOT ZERO")
```

**Tests:** if, else, elif, comparison operators, logical and/or/not.

---

## 6. While Loops

Testing while loops.

```
# while.c64
# While loop tests

def main():
    cls()

    # Simple countdown
    byte count = 10
    println("COUNTDOWN:")

    while count > 0:
        print(count)
        print(" ")
        count = count - 1

    println("")
    println("BLASTOFF!")

    # While with break
    println("")
    println("BREAK TEST:")
    count = 0

    while true:
        print(count)
        print(" ")
        count = count + 1
        if count >= 5:
            break

    println("")
    println("STOPPED AT 5")
```

**Tests:** while, condition, decrement, break, infinite loop exit.

---

## 7. For Loops

Testing for loops.

```
# for.c64
# For loop tests

def main():
    cls()

    # Counting up
    println("COUNTING UP:")
    for i in 0 to 9:
        print(i)
        print(" ")
    println("")

    # Counting down
    println("COUNTING DOWN:")
    for i in 9 downto 0:
        print(i)
        print(" ")
    println("")

    # Nested loops
    println("MULTIPLICATION TABLE:")
    for y in 1 to 5:
        for x in 1 to 5:
            byte result = x * y
            print(result)
            print(" ")
        println("")

    # For with continue
    println("SKIP EVENS:")
    for i in 0 to 9:
        if i % 2 == 0:
            continue
        print(i)
        print(" ")
    println("")
```

**Tests:** for-to, for-downto, nested loops, continue.

---

## 8. Arrays

Testing array functionality.

```
# arrays.c64
# Array tests

def main():
    cls()

    # Array declaration and initialization
    byte numbers[5]
    numbers[0] = 10
    numbers[1] = 20
    numbers[2] = 30
    numbers[3] = 40
    numbers[4] = 50

    # Print array elements
    println("ARRAY CONTENTS:")
    for i in 0 to 4:
        print("NUMBERS[")
        print(i)
        print("] = ")
        println(numbers[i])

    # Sum of array
    word sum = 0
    for i in 0 to 4:
        sum = sum + numbers[i]
    print("SUM = ")
    println(sum)

    # Initialized array
    byte primes[] = [2, 3, 5, 7, 11]
    println("FIRST 5 PRIMES:")
    for i in 0 to 4:
        print(primes[i])
        print(" ")
    println("")
```

**Tests:** Array declaration, indexing, initialization, array in expressions.

---

## 9. Functions

Testing function definitions and calls.

```
# functions.c64
# Function tests

# Function with return value
def add(byte a, byte b) -> byte:
    return a + b

# Function without return value
def greet(string name):
    print("HELLO, ")
    print(name)
    println("!")

# Function with multiple statements
def print_box(byte width, byte height):
    for y in 0 to height - 1:
        for x in 0 to width - 1:
            if y == 0 or y == height - 1:
                print("*")
            elif x == 0 or x == width - 1:
                print("*")
            else:
                print(" ")
        println("")

def main():
    cls()

    # Call void function
    greet("PLAYER 1")
    println("")

    # Call function with return
    byte result = add(15, 27)
    print("15 + 27 = ")
    println(result)
    println("")

    # Call with literals
    print("8 + 4 = ")
    println(add(8, 4))
    println("")

    # Draw a box
    println("BOX 10x5:")
    print_box(10, 5)
```

**Tests:** Function definition, parameters, return values, function calls.

---

## 10. Joystick Input

Testing joystick reading.

```
# joystick.c64
# Joystick input test

byte player_x = 20
byte player_y = 12

def main():
    cls()
    println("JOYSTICK TEST")
    println("MOVE WITH JOYSTICK 2")
    println("PRESS FIRE TO EXIT")
    println("")

    while true:
        byte joy = joystick(2)

        if joy & JOY_UP and player_y > 0:
            player_y = player_y - 1

        if joy & JOY_DOWN and player_y < 24:
            player_y = player_y + 1

        if joy & JOY_LEFT and player_x > 0:
            player_x = player_x - 1

        if joy & JOY_RIGHT and player_x < 39:
            player_x = player_x + 1

        if joy & JOY_FIRE:
            break

        # Draw player
        cursor(player_x, player_y)
        print("@")

        wait(2)

        # Clear player
        cursor(player_x, player_y)
        print(" ")

    cls()
    println("GOODBYE!")
```

**Tests:** Joystick reading, bitwise AND, constants, game loop.

---

## 11. Sound Test

Testing SID sound.

```
# sound.c64
# Sound test

def play_note(word freq, byte duration):
    voice(1, WAVE_SAW, freq, 0, 8, 10, 4)
    voice_on(1)
    wait(duration)
    voice_off(1)

def main():
    cls()
    println("SOUND TEST")
    println("")

    sound_init()
    volume(15)

    # Simple scale
    println("PLAYING SCALE...")

    word notes[] = [4186, 4699, 5274, 5588, 6272, 7040, 7902, 8372]

    for i in 0 to 7:
        play_note(notes[i], 10)
        wait(5)

    println("DONE!")

    sound_off()
```

**Tests:** Sound initialization, voice control, frequency, word array.

---

## 12. Sprite Test

Testing sprite display.

```
# sprites.c64
# Sprite test

# Sprite data (ball shape) - would be at fixed address
const SPRITE_ADDR = $2000

byte sprite_x = 100
byte sprite_y = 100

def main():
    cls()
    println("SPRITE TEST")
    println("MOVE WITH JOYSTICK")

    # Initialize sprite
    sprite_enable(0, true)
    sprite_color(0, YELLOW)
    sprite_data(0, SPRITE_ADDR)

    while true:
        byte joy = joystick(2)

        if joy & JOY_UP and sprite_y > 50:
            sprite_y = sprite_y - 2
        if joy & JOY_DOWN and sprite_y < 229:
            sprite_y = sprite_y + 2
        if joy & JOY_LEFT and sprite_x > 24:
            sprite_x = sprite_x - 2
        if joy & JOY_RIGHT and sprite_x < 255:
            sprite_x = sprite_x + 2

        sprite_pos(0, sprite_x, sprite_y)

        if joy & JOY_FIRE:
            break

        wait(1)

    sprite_enable(0, false)
    cls()
    println("GOODBYE!")
```

**Tests:** Sprite enable, position, color, data pointer.

---

## 13. Bitwise Operations

Testing bitwise operators.

```
# bitwise.c64
# Bitwise operation tests

def print_binary(byte value):
    for i in 7 downto 0:
        if value & (1 << i):
            print("1")
        else:
            print("0")

def main():
    cls()

    byte a = %10101010
    byte b = %11001100

    print("A =        ")
    print_binary(a)
    println("")

    print("B =        ")
    print_binary(b)
    println("")
    println("")

    print("A AND B =  ")
    print_binary(a & b)
    println("")

    print("A OR B =   ")
    print_binary(a | b)
    println("")

    print("A XOR B =  ")
    print_binary(a ^ b)
    println("")

    print("NOT A =    ")
    print_binary(~a)
    println("")

    print("A << 2 =   ")
    print_binary(a << 2)
    println("")

    print("A >> 2 =   ")
    print_binary(a >> 2)
    println("")
```

**Tests:** Bitwise AND, OR, XOR, NOT, shifts, binary literals.

---

## 14. Constants

Testing constant definitions.

```
# constants.c64
# Constant tests

const SCREEN_WIDTH = 40
const SCREEN_HEIGHT = 25
const CENTER_X = SCREEN_WIDTH / 2
const CENTER_Y = SCREEN_HEIGHT / 2
const MAX_SCORE = 9999
const LIVES_START = 3

def main():
    cls()

    println("CONSTANTS:")
    print("SCREEN: ")
    print(SCREEN_WIDTH)
    print("x")
    println(SCREEN_HEIGHT)

    print("CENTER: ")
    print(CENTER_X)
    print(",")
    println(CENTER_Y)

    print("MAX SCORE: ")
    println(MAX_SCORE)

    # Using constants in code
    cursor(CENTER_X - 5, CENTER_Y)
    println("CENTERED!")

    byte lives = LIVES_START
    print("LIVES: ")
    println(lives)
```

**Tests:** Constant definition, constant expressions, usage in code.

---

## 15. Complete Mini-Game

A simple complete game demonstrating multiple features.

```
# dodge.c64
# Simple dodge game

const PLAYER_Y = 22
const MIN_X = 1
const MAX_X = 38

byte player_x = 20
word score = 0
byte enemy_x = 20
byte enemy_y = 0
bool game_over = false

def init():
    cls()
    screen_color(BLACK, BLACK)
    text_color(WHITE)

def draw_border():
    for x in 0 to 39:
        char_at(x, 0, '-')
        char_at(x, 24, '-')
    for y in 0 to 24:
        char_at(0, y, '|')
        char_at(39, y, '|')

def update():
    # Read input
    byte joy = joystick(2)

    if joy & JOY_LEFT and player_x > MIN_X:
        player_x = player_x - 1
    if joy & JOY_RIGHT and player_x < MAX_X:
        player_x = player_x + 1

    # Move enemy
    enemy_y = enemy_y + 1
    if enemy_y > 23:
        enemy_y = 1
        enemy_x = (enemy_x * 7 + 13) % 38 + 1  # Pseudo-random
        score = score + 10

    # Collision check
    if enemy_y == PLAYER_Y and enemy_x == player_x:
        game_over = true

def draw():
    # Clear previous positions
    for y in 1 to 23:
        for x in 1 to 38:
            char_at(x, y, ' ')

    # Draw score
    cursor(2, 0)
    print("SCORE:")
    print(score)

    # Draw enemy
    text_color(RED)
    char_at(enemy_x, enemy_y, 'V')

    # Draw player
    text_color(GREEN)
    char_at(player_x, PLAYER_Y, 'A')

    text_color(WHITE)

def main():
    init()
    draw_border()

    cursor(15, 12)
    println("DODGE GAME")
    cursor(12, 14)
    println("PRESS FIRE TO START")

    while not (joystick(2) & JOY_FIRE):
        wait(1)

    cls()
    draw_border()

    while not game_over:
        update()
        draw()
        wait(3)

    cursor(15, 12)
    text_color(RED)
    println("GAME OVER!")
    cursor(14, 14)
    print("SCORE: ")
    println(score)

    text_color(WHITE)
    cursor(12, 16)
    println("PRESS ANY KEY...")
    wait_key()
```

**Tests:** Complete program, multiple functions, game loop, all features combined.

---

## Test Coverage Summary

| Feature           | Covered In |
| ----------------- | ---------- |
| print/println     | 1, 2, all  |
| cls               | 2+         |
| Variables (byte)  | 3+         |
| Variables (word)  | 4, 10, 11  |
| Arithmetic        | 3, 4       |
| Comparison        | 4, 5       |
| if/elif/else      | 5          |
| Logical operators | 5          |
| while loop        | 6          |
| break             | 6, 10      |
| for loop          | 7          |
| continue          | 7          |
| Arrays            | 8          |
| Functions         | 9          |
| Return values     | 9          |
| Joystick          | 10, 15     |
| Sound             | 11         |
| Sprites           | 12         |
| Bitwise ops       | 13         |
| Constants         | 14         |
| Colors            | 2, 15      |
| cursor            | 2+         |
