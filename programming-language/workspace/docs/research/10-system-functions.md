# System Functions for the C64 Language

This document describes the system functions that provide easy access to C64 hardware.

## Design Philosophy

Instead of using POKE/PEEK like BASIC, our language provides named functions that:

1. Are self-documenting (clear names)
2. Hide the complexity of memory addresses
3. Provide a consistent interface

## Screen Functions

### Text Output

| Function        | Description                   | KERNAL Equivalent  |
| --------------- | ----------------------------- | ------------------ |
| `print(text)`   | Print text at cursor position | CHROUT ($FFD2)     |
| `println(text)` | Print text with newline       | CHROUT + chr(13)   |
| `cls()`         | Clear screen                  | SCINIT or chr(147) |
| `home()`        | Move cursor to top-left       | chr(19)            |
| `cursor(x, y)`  | Set cursor position           | PLOT ($FFF0)       |

### Screen Control

| Function                   | Description                      |
| -------------------------- | -------------------------------- |
| `screen_color(bg, border)` | Set background and border colors |
| `text_color(color)`        | Set text color (0-15)            |
| `char_at(x, y, char)`      | Put character at position        |
| `color_at(x, y, color)`    | Set color at position            |

### Screen Memory Access

```
# Screen memory starts at $0400 (1024)
# Color memory starts at $D800 (55296)

char_at(0, 0, 1)        # Put 'A' at top-left
color_at(0, 0, RED)     # Make it red
```

### Color Constants

```
const BLACK = 0
const WHITE = 1
const RED = 2
const CYAN = 3
const PURPLE = 4
const GREEN = 5
const BLUE = 6
const YELLOW = 7
const ORANGE = 8
const BROWN = 9
const LIGHTRED = 10
const DARKGREY = 11
const GREY = 12
const LIGHTGREEN = 13
const LIGHTBLUE = 14
const LIGHTGREY = 15
```

## Input Functions

### Keyboard

| Function        | Description                 | KERNAL Equivalent |
| --------------- | --------------------------- | ----------------- |
| `key()`         | Get current key (0 if none) | GETIN ($FFE4)     |
| `read()`    | Wait for keypress           | Loop with GETIN   |
| `key_pressed()` | Check if any key pressed    | SCNKEY ($FF9F)    |

### Joystick

```
# Joystick port 1 at $DC01, port 2 at $DC00
joy = joystick(1)       # Read joystick 1

if joy & JOY_UP:
    move_up()
if joy & JOY_FIRE:
    shoot()
```

| Constant    | Value | Description    |
| ----------- | ----- | -------------- |
| `JOY_UP`    | 1     | Joystick up    |
| `JOY_DOWN`  | 2     | Joystick down  |
| `JOY_LEFT`  | 4     | Joystick left  |
| `JOY_RIGHT` | 8     | Joystick right |
| `JOY_FIRE`  | 16    | Fire button    |

## Sound Functions (SID)

### Basic Sound

| Function       | Description              |
| -------------- | ------------------------ |
| `sound_init()` | Initialize SID chip      |
| `volume(v)`    | Set master volume (0-15) |
| `sound_off()`  | Turn off all sound       |

### Voice Control

```
# voice: 1-3
# waveform: WAVE_TRIANGLE, WAVE_SAW, WAVE_PULSE, WAVE_NOISE
# frequency: 0-65535
# attack, decay, sustain, release: 0-15

voice(1, WAVE_SAW, 5000, 0, 8, 12, 4)
voice_on(1)
wait(500)
voice_off(1)
```

### SID Registers (for advanced use)

The SID chip is at $D400 (54272):

| Register  | Offset | Description                 |
| --------- | ------ | --------------------------- |
| FREQ_LO_1 | 0      | Voice 1 frequency low       |
| FREQ_HI_1 | 1      | Voice 1 frequency high      |
| PW_LO_1   | 2      | Voice 1 pulse width low     |
| PW_HI_1   | 3      | Voice 1 pulse width high    |
| CTRL_1    | 4      | Voice 1 control register    |
| AD_1      | 5      | Voice 1 attack/decay        |
| SR_1      | 6      | Voice 1 sustain/release     |
| ...       | ...    | (repeat for voice 2, 3)     |
| VOLUME    | 24     | Master volume & filter mode |

## Graphics Functions (VIC-II)

### Sprite Control

```
sprite_enable(0, true)          # Enable sprite 0
sprite_pos(0, 100, 100)         # Set position
sprite_color(0, RED)            # Set color
sprite_data(0, $0800)           # Set data pointer
sprite_expand(0, true, false)   # Double width, normal height
sprite_priority(0, false)       # Sprite in front of background
sprite_multicolor(0, true)      # Enable multicolor mode
```

### Sprite Collision Detection

```
if sprite_sprite_collision(0, 1):
    # Sprite 0 and 1 collided

if sprite_background_collision(0):
    # Sprite 0 hit background
```

### VIC-II Registers

The VIC-II chip is at $D000 (53248):

| Register          | Address     | Description              |
| ----------------- | ----------- | ------------------------ |
| SPRITE_X          | $D000-$D00F | Sprite X positions       |
| SPRITE_Y          | $D001-$D00F | Sprite Y positions       |
| SPRITE_X_MSB      | $D010       | X position bit 8         |
| SPRITE_ENABLE     | $D015       | Sprite enable bits       |
| SPRITE_EXPAND_Y   | $D017       | Vertical expansion       |
| SPRITE_PRIORITY   | $D01B       | Priority over background |
| SPRITE_MULTICOLOR | $D01C       | Multicolor mode          |
| SPRITE_EXPAND_X   | $D01D       | Horizontal expansion     |
| SPRITE_COLLISION  | $D01E       | Sprite-sprite collision  |
| BORDER_COLOR      | $D020       | Border color             |
| BG_COLOR          | $D021       | Background color         |

## Timing Functions

| Function       | Description                       |
| -------------- | --------------------------------- |
| `wait(frames)` | Wait for N frames (1/50s PAL)     |
| `wait_ms(ms)`  | Wait approximately N milliseconds |
| `raster()`     | Get current raster line (0-311)   |

## Memory Access (Low-Level)

For advanced users who need direct memory access:

```
poke(address, value)    # Write byte to memory
value = peek(address)   # Read byte from memory
pokew(address, value)   # Write word (16-bit)
value = peekw(address)  # Read word (16-bit)
```

## File I/O (Future)

```
# Disk operations (optional, complex)
file_open(8, "FILENAME", "R")
data = file_read()
file_close()
```

## Implementation Notes

### KERNAL Routine Addresses

| Name   | Address | Description             |
| ------ | ------- | ----------------------- |
| SCINIT | $FF81   | Initialize screen       |
| CHROUT | $FFD2   | Output character        |
| CHRIN  | $FFCF   | Input character         |
| GETIN  | $FFE4   | Get key from buffer     |
| PLOT   | $FFF0   | Set/get cursor position |
| SCREEN | $FFED   | Get screen size         |

### Usage Example

```
# Clear screen and print colored text
cls()
text_color(YELLOW)
println("WELCOME TO THE GAME")
text_color(WHITE)
cursor(10, 12)
print("PRESS FIRE TO START")

while not (joystick(1) & JOY_FIRE):
    pass

sound_init()
volume(15)
voice(1, WAVE_SAW, 8000, 0, 0, 15, 0)
voice_on(1)
wait(10)
voice_off(1)
```

## References

- [C64 KERNAL Functions](https://sta.c64.org/cbm64krnfunc.html)
- [VIC-II Documentation](https://www.c64-wiki.com/wiki/VIC)
- [SID Documentation](https://www.c64-wiki.com/wiki/SID)
- [C64 Memory Map](https://sta.c64.org/cbm64mem.html)
