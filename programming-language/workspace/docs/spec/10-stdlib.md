# System Function Specification

This document defines all built-in system functions with their signatures and behavior.

---

## Function Signature Notation

```
function_name(param1: type, param2: type) -> return_type
```

- Parameters with `?` are optional
- `void` means no return value

---

## Screen Output Functions

### cls

Clear the screen and reset cursor to top-left.

```
cls() -> void
```

**Implementation:** Calls KERNAL SCINIT ($FF81) or outputs CHR$(147).

**Example:**

```
cls()
print("FRESH SCREEN")
```

---

### print

Print text or value at current cursor position.

```
print(value: byte) -> void
print(value: word) -> void
print(value: string) -> void
print(value: bool) -> void
```

**Behavior:**

- `byte/word`: Converts number to decimal string and prints
- `string`: Prints characters until null terminator
- `bool`: Prints "TRUE" or "FALSE"

**Implementation:** Uses KERNAL CHROUT ($FFD2) for each character.

**Example:**

```
print("SCORE: ")
print(score)
print(" LIVES: ")
print(lives)
```

---

### println

Print text or value followed by newline.

```
println(value: byte) -> void
println(value: word) -> void
println(value: string) -> void
println(value: bool) -> void
println() -> void
```

**Behavior:** Same as `print()` but appends carriage return (CHR$13).

**Example:**

```
println("LINE 1")
println("LINE 2")
println()           # Empty line
println("LINE 4")
```

---

### cursor

Set cursor position on screen.

```
cursor(x: byte, y: byte) -> void
```

**Parameters:**

- `x`: Column (0-39)
- `y`: Row (0-24)

**Implementation:** Uses KERNAL PLOT ($FFF0).

**Example:**

```
cursor(0, 0)        # Top-left
cursor(20, 12)      # Center of screen
cursor(39, 24)      # Bottom-right
```

---

### home

Move cursor to top-left corner (0, 0).

```
home() -> void
```

**Implementation:** Outputs CHR$(19).

---

### char_at

Put a character at specific screen position.

```
char_at(x: byte, y: byte, char: byte) -> void
```

**Parameters:**

- `x`: Column (0-39)
- `y`: Row (0-24)
- `char`: Screen code (not PETSCII!)

**Implementation:** Direct write to screen memory ($0400 + y\*40 + x).

**Example:**

```
char_at(0, 0, 1)        # 'A' at top-left (screen code 1)
char_at(10, 5, '*')     # Asterisk
```

---

### color_at

Set color of character at specific screen position.

```
color_at(x: byte, y: byte, color: byte) -> void
```

**Parameters:**

- `x`: Column (0-39)
- `y`: Row (0-24)
- `color`: Color value (0-15)

**Implementation:** Direct write to color memory ($D800 + y\*40 + x).

**Example:**

```
char_at(10, 10, 'X')
color_at(10, 10, RED)   # Make it red
```

---

### screen_color

Set screen background and border colors.

```
screen_color(background: byte, border: byte) -> void
```

**Parameters:**

- `background`: Background color (0-15)
- `border`: Border color (0-15)

**Implementation:** Writes to $D020 (border) and $D021 (background).

**Example:**

```
screen_color(BLACK, BLACK)      # All black
screen_color(BLUE, LIGHTBLUE)   # Blue theme
```

---

### text_color

Set color for subsequent text output.

```
text_color(color: byte) -> void
```

**Parameters:**

- `color`: Text color (0-15)

**Implementation:** Writes to $0286 (current text color).

**Example:**

```
text_color(WHITE)
println("NORMAL TEXT")
text_color(RED)
println("WARNING!")
text_color(WHITE)
```

---

## Input Functions

### key

Get currently pressed key (non-blocking).

```
key() -> byte
```

**Returns:**

- PETSCII code of pressed key
- 0 if no key pressed

**Implementation:** Uses KERNAL GETIN ($FFE4).

**Example:**

```
byte k = key()
if k == 'Q':
    quit_game()
elif k == ' ':
    pause_game()
```

---

### read

Wait for a key press (blocking).

```
read() -> byte
```

**Returns:** PETSCII code of pressed key.

**Implementation:** Loops calling GETIN until non-zero.

**Example:**

```
println("PRESS ANY KEY...")
byte k = read()
println("YOU PRESSED A KEY!")
```

---

### key_pressed

Check if any key is currently pressed.

```
key_pressed() -> bool
```

**Returns:** true if a key is held down.

**Implementation:** Checks keyboard matrix via SCNKEY ($FF9F).

---

## Joystick Functions

### joystick

Read joystick state.

```
joystick(port: byte) -> byte
```

**Parameters:**

- `port`: 1 or 2

**Returns:** Bitmask of joystick state:

- Bit 0 (1): Up
- Bit 1 (2): Down
- Bit 2 (4): Left
- Bit 3 (8): Right
- Bit 4 (16): Fire

**Implementation:**

- Port 2: Read from $DC00
- Port 1: Read from $DC01
- Values are inverted (active low)

**Example:**

```
byte joy = joystick(2)

if joy & JOY_UP:
    player_y = player_y - 1
if joy & JOY_DOWN:
    player_y = player_y + 1
if joy & JOY_LEFT:
    player_x = player_x - 1
if joy & JOY_RIGHT:
    player_x = player_x + 1
if joy & JOY_FIRE:
    shoot()
```

---

## Sprite Functions

### sprite_enable

Enable or disable a sprite.

```
sprite_enable(sprite: byte, enabled: bool) -> void
```

**Parameters:**

- `sprite`: Sprite number (0-7)
- `enabled`: true to show, false to hide

**Implementation:** Sets/clears bit in $D015.

**Example:**

```
sprite_enable(0, true)      # Show sprite 0
sprite_enable(1, false)     # Hide sprite 1
```

---

### sprite_pos

Set sprite position.

```
sprite_pos(sprite: byte, x: word, y: byte) -> void
```

**Parameters:**

- `sprite`: Sprite number (0-7)
- `x`: X position (0-511)
- `y`: Y position (0-255)

**Implementation:** Writes to $D000+sprite*2 (X), $D001+sprite*2 (Y), and sets MSB in $D010.

**Example:**

```
sprite_pos(0, 160, 100)     # Center of screen
sprite_pos(0, 320, 50)      # Right side (X > 255)
```

---

### sprite_color

Set sprite color.

```
sprite_color(sprite: byte, color: byte) -> void
```

**Parameters:**

- `sprite`: Sprite number (0-7)
- `color`: Color value (0-15)

**Implementation:** Writes to $D027+sprite.

---

### sprite_data

Set sprite data pointer.

```
sprite_data(sprite: byte, address: word) -> void
```

**Parameters:**

- `sprite`: Sprite number (0-7)
- `address`: Memory address of sprite data (must be divisible by 64)

**Implementation:** Calculates pointer value and writes to $07F8+sprite.

**Example:**

```
sprite_data(0, $2000)       # Sprite 0 uses data at $2000
```

---

### sprite_expand

Set sprite expansion (double size).

```
sprite_expand(sprite: byte, x_expand: bool, y_expand: bool) -> void
```

**Parameters:**

- `sprite`: Sprite number (0-7)
- `x_expand`: Double width
- `y_expand`: Double height

**Implementation:** Sets bits in $D01D (X) and $D017 (Y).

---

### sprite_multicolor

Enable multicolor mode for sprite.

```
sprite_multicolor(sprite: byte, enabled: bool) -> void
```

**Implementation:** Sets/clears bit in $D01C.

---

### sprite_priority

Set sprite priority (in front or behind background).

```
sprite_priority(sprite: byte, behind_background: bool) -> void
```

**Implementation:** Sets/clears bit in $D01B.

---

### sprite_collision

Check sprite-sprite collision.

```
sprite_collision() -> byte
```

**Returns:** Bitmask of sprites that collided.

**Implementation:** Reads $D01E (clears on read).

---

### sprite_bg_collision

Check sprite-background collision.

```
sprite_bg_collision() -> byte
```

**Returns:** Bitmask of sprites that hit background.

**Implementation:** Reads $D01F (clears on read).

---

## Sound Functions

### sound_init

Initialize SID chip.

```
sound_init() -> void
```

**Implementation:** Clears all SID registers ($D400-$D418).

---

### volume

Set master volume.

```
volume(level: byte) -> void
```

**Parameters:**

- `level`: Volume level (0-15)

**Implementation:** Writes to $D418 (low nibble).

---

### sound_off

Turn off all sound.

```
sound_off() -> void
```

**Implementation:** Sets volume to 0 and clears voice control registers.

---

### voice

Configure a voice.

```
voice(num: byte, waveform: byte, frequency: word,
      attack: byte, decay: byte, sustain: byte, release: byte) -> void
```

**Parameters:**

- `num`: Voice number (1-3)
- `waveform`: WAVE_TRIANGLE, WAVE_SAW, WAVE_PULSE, or WAVE_NOISE
- `frequency`: Frequency value (0-65535)
- `attack`: Attack time (0-15)
- `decay`: Decay time (0-15)
- `sustain`: Sustain level (0-15)
- `release`: Release time (0-15)

**Implementation:** Writes to voice registers at $D400+(num-1)\*7.

**Example:**

```
sound_init()
volume(15)
voice(1, WAVE_SAW, 5000, 0, 8, 12, 4)
voice_on(1)
wait(30)
voice_off(1)
```

---

### voice_on

Start playing a voice.

```
voice_on(num: byte) -> void
```

**Implementation:** Sets gate bit in voice control register.

---

### voice_off

Stop playing a voice.

```
voice_off(num: byte) -> void
```

**Implementation:** Clears gate bit in voice control register.

---

### pulse_width

Set pulse width for a voice (only affects WAVE_PULSE).

```
pulse_width(num: byte, width: word) -> void
```

**Parameters:**

- `num`: Voice number (1-3)
- `width`: Pulse width (0-4095)

---

## Timing Functions

### wait

Wait for specified number of frames.

```
wait(frames: word) -> void
```

**Parameters:**

- `frames`: Number of frames to wait (1 frame ≈ 20ms PAL, 16.7ms NTSC)

**Implementation:** Counts raster interrupts or busy-waits on raster line.

**Example:**

```
println("READY...")
wait(50)            # Wait ~1 second (PAL)
println("GO!")
```

---

### wait_ms

Wait for approximately specified milliseconds.

```
wait_ms(ms: word) -> void
```

**Note:** Timing is approximate due to C64 limitations.

---

### raster

Get current raster line.

```
raster() -> word
```

**Returns:** Current raster line (0-311 PAL, 0-262 NTSC).

**Implementation:** Reads $D011 (bit 7) and $D012.

**Example:**

```
# Wait for specific raster line
while raster() != 100:
    pass
# Now at line 100
```

---

## Memory Access Functions

### peek

Read byte from memory.

```
peek(address: word) -> byte
```

**Example:**

```
byte border = peek($D020)
byte char = peek($0400)     # First screen character
```

---

### poke

Write byte to memory.

```
poke(address: word, value: byte) -> void
```

**Example:**

```
poke($D020, 0)              # Black border
poke($0400, 1)              # 'A' at top-left
```

---

### peekw

Read word (16-bit) from memory.

```
peekw(address: word) -> word
```

**Implementation:** Reads two bytes, low byte first.

---

### pokew

Write word (16-bit) to memory.

```
pokew(address: word, value: word) -> void
```

**Implementation:** Writes two bytes, low byte first.

---

## Utility Functions

### strlen

Get string length.

```
strlen(s: string) -> byte
```

**Returns:** Number of characters (not including null terminator).

---

## Random Number Functions

The random number generator uses a 16-bit Galois LFSR (Linear Feedback Shift Register) with polynomial $0039. The seed is initialized at program start from the SID noise register and VIC raster line for unpredictable results.

### rand

Generate a random fixed-point number.

```
rand() -> fixed
```

**Returns:** Random value between 0.0 and 0.9375 (15/16).

**Implementation:** Takes upper 4 bits of a random byte as the fractional part, giving 16 possible values (0.0, 0.0625, 0.125, ..., 0.9375).

**Example:**

```
fixed r = rand()
if r < 0.5:
    println("HEADS")
else:
    println("TAILS")
```

---

### rand_byte

Generate a random byte in a range.

```
rand_byte(from: byte, to: byte) -> byte
```

**Parameters:**

- `from`: Minimum value (inclusive)
- `to`: Maximum value (inclusive)

**Returns:** Random byte in range [from, to].

**Implementation:** Uses rejection sampling for uniform distribution.

**Example:**

```
byte dice = rand_byte(1, 6)   # Dice roll
byte card = rand_byte(1, 52)  # Random card
```

---

### rand_sbyte

Generate a random signed byte in a range.

```
rand_sbyte(from: sbyte, to: sbyte) -> sbyte
```

**Parameters:**

- `from`: Minimum value (inclusive)
- `to`: Maximum value (inclusive)

**Returns:** Random signed byte in range [from, to].

**Example:**

```
sbyte offset = rand_sbyte(-10, 10)  # Random offset
```

---

### rand_word

Generate a random word in a range.

```
rand_word(from: word, to: word) -> word
```

**Parameters:**

- `from`: Minimum value (inclusive)
- `to`: Maximum value (inclusive)

**Returns:** Random word in range [from, to].

**Example:**

```
word score = rand_word(100, 1000)    # Random score
word addr = rand_word($0400, $07FF)  # Random screen address
```

---

### rand_sword

Generate a random signed word in a range.

```
rand_sword(from: sword, to: sword) -> sword
```

**Parameters:**

- `from`: Minimum value (inclusive)
- `to`: Maximum value (inclusive)

**Returns:** Random signed word in range [from, to].

**Example:**

```
sword velocity = rand_sword(-100, 100)  # Random velocity
```

---

### seed

Reseed the random number generator from hardware entropy.

```
seed() -> void
```

**Description:** Reseeds the PRNG from hardware entropy sources (SID noise register, CIA timers, raster line). This is useful when:
- The emulator produces the same random sequence on each run
- You want to refresh the random state during program execution

**Example:**

```
# Generate some random numbers
println(rand_byte(1, 100))

# Reseed from hardware entropy
seed()

# Generate more random numbers with fresh entropy
println(rand_byte(1, 100))
```

---

### memset

Fill memory region with value.

```
memset(address: word, value: byte, count: word) -> void
```

---

### memcpy

Copy memory region.

```
memcpy(dest: word, src: word, count: word) -> void
```

---

## Function Summary

| Category      | Functions                                                                                                                                      |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Screen Output | cls, print, println, cursor, home, char_at, color_at, screen_color, text_color                                                                 |
| Input         | key, read, key_pressed, joystick                                                                                                           |
| Sprites       | sprite_enable, sprite_pos, sprite_color, sprite_data, sprite_expand, sprite_multicolor, sprite_priority, sprite_collision, sprite_bg_collision |
| Sound         | sound_init, volume, sound_off, voice, voice_on, voice_off, pulse_width                                                                         |
| Timing        | wait, wait_ms, raster                                                                                                                          |
| Memory        | peek, poke, peekw, pokew, memset, memcpy                                                                                                       |
| Random        | rand, rand_byte, rand_sbyte, rand_word, rand_sword, seed                                                                                       |
| Utility       | strlen                                                                                                                                         |

**Total: 42 functions**
