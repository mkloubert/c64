# Quick Reference Card

## Data Types

```
byte x = 0          # 0-255
word y = 0          # 0-65535
bool z = false      # true/false
string s = "TEXT"   # Max 255 chars
byte arr[10]        # Array
const N = 100       # Constant
```

## Operators

```
+ - * / %           # Arithmetic
== != < > <= >=     # Comparison
and or not          # Logical
& | ^ ~ << >>       # Bitwise
= += -= *= /=       # Assignment
```

## Control Flow

```
if cond:            while cond:         for i in 0 to 9:
    ...                 ...                 ...
elif cond:
    ...             break               for i in 9 downto 0:
else:               continue                ...
    ...
```

## Functions

```
def name():                     def add(byte a, byte b) -> byte:
    ...                             return a + b
```

## Screen Output

```
cls()                           # Clear screen
print("TEXT")                   # Print
println("TEXT")                 # Print + newline
cursor(x, y)                    # Position (0-39, 0-24)
char_at(x, y, ch)              # Put character
color_at(x, y, col)            # Set color
screen_color(bg, border)       # Screen colors
text_color(col)                # Text color
```

## Colors

```
BLACK=0  WHITE=1  RED=2     CYAN=3    PURPLE=4
GREEN=5  BLUE=6   YELLOW=7  ORANGE=8  BROWN=9
LIGHTRED=10  DARKGREY=11  GREY=12
LIGHTGREEN=13  LIGHTBLUE=14  LIGHTGREY=15
```

## Input

```
byte k = key()                  # Get key (0=none)
byte k = read()             # Wait for key
byte j = joystick(2)            # Read joy (1 or 2)

JOY_UP=1  JOY_DOWN=2  JOY_LEFT=4  JOY_RIGHT=8  JOY_FIRE=16

if j & JOY_FIRE: ...            # Check fire button
```

## Sound

```
sound_init()                    # Initialize
volume(15)                      # Volume 0-15
voice(1, WAVE_SAW, 5000, 0, 8, 12, 4)  # Configure
voice_on(1)                     # Start playing
voice_off(1)                    # Stop
sound_off()                     # All off

WAVE_TRIANGLE=16  WAVE_SAW=32  WAVE_PULSE=64  WAVE_NOISE=128
```

## Sprites

```
sprite_enable(0, true)          # Enable sprite 0
sprite_pos(0, 160, 100)         # Position (x: 0-511, y: 0-255)
sprite_color(0, YELLOW)         # Color
sprite_data(0, $2000)           # Data pointer
```

## Timing

```
wait(50)                        # Wait ~1 second (PAL)
word r = raster()               # Get raster line
```

## Memory

```
byte v = peek($D020)            # Read byte
poke($D020, 0)                  # Write byte
```

## Minimal Program

```
def main():
    cls()
    println("HELLO C64!")
```

## Game Loop Template

```
def main():
    init()
    while not gameover:
        input()
        update()
        draw()
        wait(2)
```
