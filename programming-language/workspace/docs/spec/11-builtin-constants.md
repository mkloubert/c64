# Built-in Constants

This document defines all built-in constants available without declaration.

---

## Color Constants

Standard C64 colors for use with screen and sprite functions.

| Constant     | Value | Color       |
| ------------ | ----- | ----------- |
| `BLACK`      | 0     | Black       |
| `WHITE`      | 1     | White       |
| `RED`        | 2     | Red         |
| `CYAN`       | 3     | Cyan        |
| `PURPLE`     | 4     | Purple      |
| `GREEN`      | 5     | Green       |
| `BLUE`       | 6     | Blue        |
| `YELLOW`     | 7     | Yellow      |
| `ORANGE`     | 8     | Orange      |
| `BROWN`      | 9     | Brown       |
| `LIGHTRED`   | 10    | Light Red   |
| `DARKGREY`   | 11    | Dark Grey   |
| `GREY`       | 12    | Grey        |
| `LIGHTGREEN` | 13    | Light Green |
| `LIGHTBLUE`  | 14    | Light Blue  |
| `LIGHTGREY`  | 15    | Light Grey  |

**Usage:**

```
screen_color(BLUE, LIGHTBLUE)
text_color(YELLOW)
sprite_color(0, RED)
```

---

## Joystick Constants

Bit flags for joystick state.

| Constant    | Value | Meaning               |
| ----------- | ----- | --------------------- |
| `JOY_UP`    | 1     | Joystick pushed up    |
| `JOY_DOWN`  | 2     | Joystick pushed down  |
| `JOY_LEFT`  | 4     | Joystick pushed left  |
| `JOY_RIGHT` | 8     | Joystick pushed right |
| `JOY_FIRE`  | 16    | Fire button pressed   |

**Usage:**

```
byte joy = joystick(2)

if joy & JOY_UP:
    move_up()

if joy & JOY_FIRE:
    shoot()

# Check diagonal
if joy & JOY_UP and joy & JOY_RIGHT:
    move_diagonal()
```

---

## Waveform Constants

SID waveform selection for voice() function.

| Constant        | Value | Waveform          |
| --------------- | ----- | ----------------- |
| `WAVE_TRIANGLE` | 16    | Triangle wave     |
| `WAVE_SAW`      | 32    | Sawtooth wave     |
| `WAVE_PULSE`    | 64    | Pulse/square wave |
| `WAVE_NOISE`    | 128   | White noise       |

**Usage:**

```
voice(1, WAVE_SAW, 5000, 0, 8, 12, 4)
voice(2, WAVE_PULSE, 3000, 2, 4, 8, 6)
voice(3, WAVE_NOISE, 1000, 0, 0, 15, 0)
```

**Combining Waveforms:**
Waveforms can be combined (though results are unusual):

```
voice(1, WAVE_SAW | WAVE_TRIANGLE, 5000, 0, 8, 12, 4)
```

---

## Key Constants

Common key codes for input handling.

| Constant     | Value | Key          |
| ------------ | ----- | ------------ |
| `KEY_RETURN` | 13    | Return/Enter |
| `KEY_SPACE`  | 32    | Space bar    |
| `KEY_UP`     | 145   | Cursor up    |
| `KEY_DOWN`   | 17    | Cursor down  |
| `KEY_LEFT`   | 157   | Cursor left  |
| `KEY_RIGHT`  | 29    | Cursor right |
| `KEY_HOME`   | 19    | Home         |
| `KEY_DEL`    | 20    | Delete       |
| `KEY_F1`     | 133   | F1           |
| `KEY_F3`     | 134   | F3           |
| `KEY_F5`     | 135   | F5           |
| `KEY_F7`     | 136   | F7           |
| `KEY_STOP`   | 3     | Run/Stop     |

**Usage:**

```
byte k = key()
if k == KEY_RETURN:
    confirm()
elif k == KEY_SPACE:
    pause()
elif k == 'Q':
    quit()
```

---

## Memory Address Constants

Important C64 memory locations.

### Screen Memory

| Constant      | Value | Description           |
| ------------- | ----- | --------------------- |
| `SCREEN`      | $0400 | Default screen memory |
| `SCREEN_SIZE` | 1000  | Screen size (40×25)   |
| `COLOR_RAM`   | $D800 | Color memory          |

### VIC-II Registers

| Constant        | Value | Description               |
| --------------- | ----- | ------------------------- |
| `VIC`           | $D000 | VIC-II base address       |
| `BORDER`        | $D020 | Border color register     |
| `BACKGROUND`    | $D021 | Background color register |
| `SPRITE_ENABLE` | $D015 | Sprite enable register    |

### SID Registers

| Constant     | Value | Description      |
| ------------ | ----- | ---------------- |
| `SID`        | $D400 | SID base address |
| `SID_VOLUME` | $D418 | Volume register  |

### CIA Registers

| Constant | Value | Description                   |
| -------- | ----- | ----------------------------- |
| `CIA1`   | $DC00 | CIA1 base (keyboard/joystick) |
| `CIA2`   | $DD00 | CIA2 base (serial/VIC bank)   |

**Usage:**

```
# Direct hardware access (advanced)
poke(BORDER, RED)
poke(BACKGROUND, BLACK)

# Read joystick directly
byte raw = peek(CIA1)
```

---

## Screen Dimension Constants

| Constant          | Value | Description                 |
| ----------------- | ----- | --------------------------- |
| `SCREEN_WIDTH`    | 40    | Screen width in characters  |
| `SCREEN_HEIGHT`   | 25    | Screen height in characters |
| `SCREEN_CENTER_X` | 20    | Horizontal center           |
| `SCREEN_CENTER_Y` | 12    | Vertical center             |

**Usage:**

```
cursor(SCREEN_CENTER_X - 5, SCREEN_CENTER_Y)
print("CENTERED!")

for x in 0 to SCREEN_WIDTH - 1:
    char_at(x, 0, '-')
```

---

## Sprite Constants

| Constant          | Value | Description                     |
| ----------------- | ----- | ------------------------------- |
| `SPRITE_WIDTH`    | 24    | Sprite width in pixels          |
| `SPRITE_HEIGHT`   | 21    | Sprite height in pixels         |
| `SPRITE_BYTES`    | 63    | Bytes per sprite definition     |
| `SPRITE_POINTERS` | $07F8 | Default sprite pointer location |

---

## Timing Constants

| Constant      | Value | Description              |
| ------------- | ----- | ------------------------ |
| `FRAMES_PAL`  | 50    | Frames per second (PAL)  |
| `FRAMES_NTSC` | 60    | Frames per second (NTSC) |

**Usage:**

```
# Wait approximately 1 second on PAL
wait(FRAMES_PAL)

# Wait approximately 2 seconds
wait(FRAMES_PAL * 2)
```

---

## Boolean Constants

| Constant | Value | Description   |
| -------- | ----- | ------------- |
| `true`   | 1     | Boolean true  |
| `false`  | 0     | Boolean false |

---

## Summary

| Category  | Count | Constants                        |
| --------- | ----- | -------------------------------- |
| Colors    | 16    | BLACK, WHITE, RED, ...           |
| Joystick  | 5     | JOY_UP, JOY_DOWN, ...            |
| Waveforms | 4     | WAVE_TRIANGLE, WAVE_SAW, ...     |
| Keys      | 13    | KEY_RETURN, KEY_SPACE, ...       |
| Memory    | 10    | SCREEN, VIC, SID, ...            |
| Screen    | 4     | SCREEN_WIDTH, SCREEN_HEIGHT, ... |
| Sprite    | 4     | SPRITE_WIDTH, SPRITE_HEIGHT, ... |
| Timing    | 2     | FRAMES_PAL, FRAMES_NTSC          |
| Boolean   | 2     | true, false                      |

**Total: 60 built-in constants**
