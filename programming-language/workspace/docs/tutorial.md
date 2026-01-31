# Getting Started Tutorial

This tutorial teaches the basics of programming for the C64 with our new language.

---

## Your First Program

### Hello World

Create a file called `hello.c64`:

```
def main():
    print("HELLO WORLD!")
```

Every program needs a `main` function - this is where execution begins.

### Compile and Run

```bash
c64c hello.c64 -o hello.d64
```

Load `hello.d64` in VICE emulator and type `RUN`.

---

## Variables

### Declaring Variables

```
byte lives = 3          # 0-255
word score = 0          # 0-65535
bool gameover = false   # true or false
string name = "PLAYER"  # Text
```

### Using Variables

```
def main():
    byte x = 10
    byte y = 20
    byte sum = x + y

    print("SUM IS: ")
    println(sum)
```

---

## Screen Output

### Basic Printing

```
def main():
    cls()                   # Clear screen
    print("HELLO ")         # Print without newline
    println("WORLD!")       # Print with newline
    println()               # Empty line
    println("LINE 2")
```

### Cursor Position

```
def main():
    cls()
    cursor(0, 0)            # Top-left corner
    print("TOP LEFT")

    cursor(15, 12)          # Center of screen
    print("CENTER")

    cursor(30, 24)          # Near bottom-right
    print("BOTTOM")
```

### Colors

```
def main():
    cls()
    screen_color(BLUE, LIGHTBLUE)   # Background, border

    text_color(WHITE)
    println("WHITE TEXT")

    text_color(YELLOW)
    println("YELLOW TEXT")

    text_color(RED)
    println("RED WARNING!")
```

---

## Control Flow

### If Statements

```
def main():
    byte score = 75

    if score >= 90:
        println("GRADE: A")
    elif score >= 80:
        println("GRADE: B")
    elif score >= 70:
        println("GRADE: C")
    else:
        println("GRADE: F")
```

### While Loops

```
def main():
    byte count = 5

    println("COUNTDOWN:")
    while count > 0:
        println(count)
        count = count - 1

    println("BLAST OFF!")
```

### For Loops

```
def main():
    println("COUNTING UP:")
    for i in 1 to 10:
        print(i)
        print(" ")
    println()

    println("COUNTING DOWN:")
    for i in 10 downto 1:
        print(i)
        print(" ")
    println()
```

---

## Functions

### Simple Function

```
def greet():
    println("HELLO PLAYER!")

def main():
    greet()
    greet()
```

### Function with Parameters

```
def greet_player(byte player_num):
    print("HELLO PLAYER ")
    println(player_num)

def main():
    greet_player(1)
    greet_player(2)
```

### Function with Return Value

```
def add(byte a, byte b) -> byte:
    return a + b

def double(byte x) -> byte:
    return x * 2

def main():
    byte result = add(10, 20)
    println(result)             # Prints 30

    println(double(5))          # Prints 10
```

---

## Arrays

### Creating Arrays

```
def main():
    byte scores[5]              # Array of 5 bytes

    # Set values
    scores[0] = 100
    scores[1] = 85
    scores[2] = 92
    scores[3] = 78
    scores[4] = 88

    # Print values
    for i in 0 to 4:
        print("SCORE ")
        print(i)
        print(": ")
        println(scores[i])
```

### Initialized Array

```
def main():
    byte primes[] = [2, 3, 5, 7, 11, 13]

    println("PRIME NUMBERS:")
    for i in 0 to 5:
        println(primes[i])
```

---

## Input

### Keyboard

```
def main():
    cls()
    println("PRESS Q TO QUIT")
    println("PRESS SPACE TO BEEP")

    while true:
        byte k = key()

        if k == 'Q':
            break

        if k == ' ':
            println("BEEP!")
```

### Joystick

```
def main():
    byte x = 20
    byte y = 12

    cls()
    println("MOVE WITH JOYSTICK 2")

    while true:
        byte joy = joystick(2)

        if joy & JOY_UP and y > 0:
            y = y - 1
        if joy & JOY_DOWN and y < 24:
            y = y + 1
        if joy & JOY_LEFT and x > 0:
            x = x - 1
        if joy & JOY_RIGHT and x < 39:
            x = x + 1

        cursor(x, y)
        print("@")

        wait(2)

        if joy & JOY_FIRE:
            break
```

---

## Sound

### Simple Beep

```
def beep():
    sound_init()
    volume(15)
    voice(1, WAVE_PULSE, 5000, 0, 0, 15, 4)
    voice_on(1)
    wait(5)
    voice_off(1)

def main():
    println("PRESS SPACE FOR BEEP")

    while true:
        if key() == ' ':
            beep()
```

### Simple Melody

```
def play_note(word freq, byte duration):
    voice(1, WAVE_SAW, freq, 0, 8, 10, 4)
    voice_on(1)
    wait(duration)
    voice_off(1)
    wait(2)

def main():
    sound_init()
    volume(15)

    # Simple scale
    play_note(4186, 10)     # C
    play_note(4699, 10)     # D
    play_note(5274, 10)     # E
    play_note(5588, 10)     # F
    play_note(6272, 10)     # G

    println("MELODY COMPLETE!")
```

---

## Sprites

### Display a Sprite

```
def main():
    cls()

    # Enable sprite 0
    sprite_enable(0, true)
    sprite_color(0, YELLOW)
    sprite_pos(0, 160, 100)

    # Note: Sprite data must be defined at address divisible by 64
    # For this example, assume sprite data is at $2000

    println("SPRITE DISPLAYED!")
    println("MOVE WITH JOYSTICK")

    word x = 160
    byte y = 100

    while true:
        byte joy = joystick(2)

        if joy & JOY_LEFT and x > 24:
            x = x - 2
        if joy & JOY_RIGHT and x < 320:
            x = x + 2
        if joy & JOY_UP and y > 50:
            y = y - 2
        if joy & JOY_DOWN and y < 229:
            y = y + 2

        sprite_pos(0, x, y)

        wait(1)

        if joy & JOY_FIRE:
            break

    sprite_enable(0, false)
```

---

## Complete Game Example

```
# DODGE GAME
# Avoid the falling enemy!

byte player_x = 20
byte enemy_x = 20
byte enemy_y = 0
word score = 0
bool running = true

def init():
    cls()
    screen_color(BLACK, BLACK)
    text_color(WHITE)

def update():
    # Player input
    byte joy = joystick(2)
    if joy & JOY_LEFT and player_x > 1:
        player_x = player_x - 1
    if joy & JOY_RIGHT and player_x < 38:
        player_x = player_x + 1

    # Move enemy
    enemy_y = enemy_y + 1
    if enemy_y > 23:
        enemy_y = 0
        enemy_x = (enemy_x * 7 + 13) % 38 + 1
        score = score + 10

    # Collision?
    if enemy_y == 22 and enemy_x == player_x:
        running = false

def draw():
    # Clear play area
    for y in 1 to 23:
        cursor(0, y)
        print("                                        ")

    # Draw score
    cursor(2, 0)
    print("SCORE: ")
    print(score)

    # Draw enemy
    text_color(RED)
    cursor(enemy_x, enemy_y)
    print("V")

    # Draw player
    text_color(GREEN)
    cursor(player_x, 22)
    print("A")

    text_color(WHITE)

def main():
    init()

    cursor(15, 10)
    println("DODGE GAME")
    cursor(12, 12)
    println("PRESS FIRE TO START")

    while not (joystick(2) & JOY_FIRE):
        wait(1)

    cls()

    while running:
        update()
        draw()
        wait(3)

    cursor(15, 12)
    text_color(RED)
    println("GAME OVER!")
    cursor(13, 14)
    print("FINAL SCORE: ")
    println(score)
```

---

## Next Steps

1. Read the **Language Reference** for all features
2. Check **System Functions** for available commands
3. Look at **Example Programs** for more ideas
4. Try modifying the examples to learn!

Happy coding for the C64!
