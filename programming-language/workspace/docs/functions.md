# System Functions Reference

Complete reference for all built-in functions.

---

## Screen Output

### cls

Clear the screen and move cursor to top-left.

```
cls()
```

---

### print

Print a value without newline.

```
print(value)
```

**Parameters:**

- `value` - byte, word, string, or bool

**Example:**

```
print("SCORE: ")
print(score)
```

---

### println

Print a value followed by newline.

```
println(value)
println()           # Just newline
```

---

### cursor

Move cursor to position.

```
cursor(x, y)
```

**Parameters:**

- `x` - Column (0-39)
- `y` - Row (0-24)

---

### home

Move cursor to top-left (0, 0).

```
home()
```

---

### char_at

Put a character at screen position.

```
char_at(x, y, char)
```

**Parameters:**

- `x` - Column (0-39)
- `y` - Row (0-24)
- `char` - Character code

---

### color_at

Set color at screen position.

```
color_at(x, y, color)
```

**Parameters:**

- `x` - Column (0-39)
- `y` - Row (0-24)
- `color` - Color (0-15)

---

### screen_color

Set background and border colors.

```
screen_color(background, border)
```

**Example:**

```
screen_color(BLUE, LIGHTBLUE)
```

---

### text_color

Set color for text output.

```
text_color(color)
```

**Example:**

```
text_color(YELLOW)
println("YELLOW TEXT")
```

---

## Input

### key

Get currently pressed key (non-blocking).

```
byte k = key()
```

**Returns:** PETSCII code of key, or 0 if no key pressed.

---

### read

Wait for a key press (blocking).

```
byte k = read()
```

**Returns:** PETSCII code of pressed key.

---

### key_pressed

Check if any key is pressed.

```
bool pressed = key_pressed()
```

---

### joystick

Read joystick state.

```
byte joy = joystick(port)
```

**Parameters:**

- `port` - 1 or 2

**Returns:** Bitmask of directions and fire.

**Constants:**

- `JOY_UP` (1)
- `JOY_DOWN` (2)
- `JOY_LEFT` (4)
- `JOY_RIGHT` (8)
- `JOY_FIRE` (16)

**Example:**

```
byte j = joystick(2)
if j & JOY_UP:
    y = y - 1
if j & JOY_FIRE:
    shoot()
```

---

## Sprites

### sprite_enable

Enable or disable a sprite.

```
sprite_enable(sprite, enabled)
```

**Parameters:**

- `sprite` - Sprite number (0-7)
- `enabled` - true to show, false to hide

---

### sprite_pos

Set sprite position.

```
sprite_pos(sprite, x, y)
```

**Parameters:**

- `sprite` - Sprite number (0-7)
- `x` - X position (0-511)
- `y` - Y position (0-255)

---

### sprite_color

Set sprite color.

```
sprite_color(sprite, color)
```

---

### sprite_data

Set sprite data pointer.

```
sprite_data(sprite, address)
```

**Parameters:**

- `sprite` - Sprite number (0-7)
- `address` - Memory address (must be divisible by 64)

---

### sprite_expand

Double sprite size.

```
sprite_expand(sprite, x_expand, y_expand)
```

---

### sprite_multicolor

Enable multicolor mode.

```
sprite_multicolor(sprite, enabled)
```

---

### sprite_priority

Set sprite behind or in front of background.

```
sprite_priority(sprite, behind)
```

---

### sprite_collision

Check sprite-sprite collisions.

```
byte mask = sprite_collision()
```

**Returns:** Bitmask of collided sprites.

---

### sprite_bg_collision

Check sprite-background collisions.

```
byte mask = sprite_bg_collision()
```

---

## Sound

### sound_init

Initialize SID chip.

```
sound_init()
```

---

### volume

Set master volume.

```
volume(level)
```

**Parameters:**

- `level` - 0 (silent) to 15 (max)

---

### sound_off

Turn off all sound.

```
sound_off()
```

---

### voice

Configure a voice.

```
voice(num, waveform, frequency, attack, decay, sustain, release)
```

**Parameters:**

- `num` - Voice (1-3)
- `waveform` - WAVE_TRIANGLE, WAVE_SAW, WAVE_PULSE, or WAVE_NOISE
- `frequency` - 0-65535
- `attack` - 0-15
- `decay` - 0-15
- `sustain` - 0-15
- `release` - 0-15

---

### voice_on

Start playing a voice.

```
voice_on(num)
```

---

### voice_off

Stop playing a voice.

```
voice_off(num)
```

---

### pulse_width

Set pulse width (for WAVE_PULSE).

```
pulse_width(num, width)
```

**Parameters:**

- `width` - 0-4095

---

## Timing

### wait

Wait for frames.

```
wait(frames)
```

**Parameters:**

- `frames` - Number of frames (1 frame ≈ 20ms on PAL)

**Example:**

```
wait(50)    # Wait ~1 second
```

---

### wait_ms

Wait for milliseconds (approximate).

```
wait_ms(ms)
```

---

### raster

Get current raster line.

```
word line = raster()
```

**Returns:** 0-311 (PAL) or 0-262 (NTSC)

---

## Memory

### peek

Read byte from memory.

```
byte value = peek(address)
```

---

### poke

Write byte to memory.

```
poke(address, value)
```

---

### peekw

Read word (16-bit) from memory.

```
word value = peekw(address)
```

---

### pokew

Write word to memory.

```
pokew(address, value)
```

---

### memset

Fill memory with value.

```
memset(address, value, count)
```

---

### memcpy

Copy memory.

```
memcpy(dest, src, count)
```

---

## Utility

### strlen

Get string length.

```
byte len = strlen(s)
```

---

### random

Generate random number.

```
byte r = random()
```

**Returns:** Random value 0-255.
