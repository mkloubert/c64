# D64 Disk Image Format

This document describes the D64 disk image format used by Commodore 1541 floppy drives.

## Overview

The D64 format is a sector-by-sector copy of a physical 1541 floppy disk. It is the standard format for distributing C64 software in emulators.

### Key Properties

- **File Extension**: `.d64`
- **Disk Type**: 5.25" single-sided floppy
- **Tracks**: 35 (standard) or 40 (extended)
- **Total Sectors**: 683 (standard) or 768 (extended)
- **Sector Size**: 256 bytes
- **Standard File Size**: 174,848 bytes (683 × 256)
- **Capacity**: ~170KB usable

---

## Track and Sector Layout

The 1541 uses **Zone Bit Recording**, meaning outer tracks have more sectors than inner tracks.

### Sectors Per Track

| Tracks    | Sectors per Track | Total Sectors |
| --------- | ----------------- | ------------- |
| 1-17      | 21                | 357           |
| 18-24     | 19                | 133           |
| 25-30     | 18                | 108           |
| 31-35     | 17                | 85            |
| **Total** |                   | **683**       |

### Extended 40-Track Format

| Tracks                | Sectors per Track | Total Sectors |
| --------------------- | ----------------- | ------------- |
| 36-40                 | 17                | 85            |
| **Total (40 tracks)** |                   | **768**       |

---

## D64 File Variants

| Type              | Tracks | Error Bytes | Size (bytes) |
| ----------------- | ------ | ----------- | ------------ |
| Standard          | 35     | No          | 174,848      |
| Standard + errors | 35     | Yes         | 175,531      |
| Extended          | 40     | No          | 196,608      |
| Extended + errors | 40     | Yes         | 197,376      |

### Error Bytes

Optional error bytes (one per sector) follow the sector data. They indicate read errors from the original disk:

| Error Code | Meaning                        |
| ---------- | ------------------------------ |
| 0 or 1     | No error                       |
| 2          | Header block not found         |
| 3          | No sync sequence               |
| 4          | Data block not found           |
| 5          | Checksum error in data         |
| 6          | Checksum error in header       |
| 7          | Verify error                   |
| 8          | Write protected                |
| 9          | Header block checksum mismatch |
| 10         | Write error                    |
| 11         | Disk ID mismatch               |
| 15         | No drive present               |

---

## Sector Offset Calculation

To find the byte offset for track T, sector S:

```
offset = (sum of sectors before track T) × 256 + (S × 256)
```

### Sector Offset Table

| Track | First Sector Offset |
| ----- | ------------------- |
| 1     | 0                   |
| 2     | 5,376 (21 × 256)    |
| 3     | 10,752              |
| ...   | ...                 |
| 18    | 91,392              |
| 19    | 96,256              |
| ...   | ...                 |

---

## Directory Structure

The directory is located on **Track 18**. This track is reserved for disk management.

### Track 18 Layout

| Sector | Content                      |
| ------ | ---------------------------- |
| 0      | BAM (Block Availability Map) |
| 1-18   | Directory entries            |

### BAM Sector (Track 18, Sector 0)

| Offset  | Size | Description                          |
| ------- | ---- | ------------------------------------ |
| $00     | 1    | Track of first directory sector (18) |
| $01     | 1    | Sector of first directory sector (1) |
| $02     | 1    | DOS version ('A' = $41)              |
| $03     | 1    | Reserved (0)                         |
| $04-$8F | 140  | BAM entries (4 bytes per track)      |
| $90-$9F | 16   | Disk name (padded with $A0)          |
| $A0-$A1 | 2    | $A0 padding                          |
| $A2-$A3 | 2    | Disk ID                              |
| $A4     | 1    | $A0 padding                          |
| $A5-$A6 | 2    | DOS type ("2A")                      |
| $A7-$AA | 4    | $A0 padding                          |
| $AB-$FF | 85   | Unused                               |

### BAM Entry Format (4 bytes per track)

```
Byte 0: Free sectors on this track
Byte 1: Bitmap for sectors 0-7 (1=free, 0=used)
Byte 2: Bitmap for sectors 8-15
Byte 3: Bitmap for sectors 16-20/23 (depends on track)
```

### Directory Entry Format (32 bytes each)

Each directory sector holds 8 entries.

| Offset  | Size | Description                                       |
| ------- | ---- | ------------------------------------------------- |
| $00-$01 | 2    | Track/Sector of next dir block (first entry only) |
| $02     | 1    | File type                                         |
| $03-$04 | 2    | Track/Sector of first data block                  |
| $05-$14 | 16   | Filename (padded with $A0)                        |
| $15-$16 | 2    | REL file: Side-sector track/sector                |
| $17     | 1    | REL file: Record length                           |
| $18-$1D | 6    | Unused                                            |
| $1E-$1F | 2    | File size in sectors (little-endian)              |

### File Types

| Value | Type | Description              |
| ----- | ---- | ------------------------ |
| $00   | DEL  | Deleted/Scratched        |
| $80   | DEL  | Deleted                  |
| $81   | SEQ  | Sequential file          |
| $82   | PRG  | Program file             |
| $83   | USR  | User file                |
| $84   | REL  | Relative file            |
| +$40  |      | Locked (write protected) |
| +$80  |      | Properly closed          |

Example: `$82` = PRG, properly closed

---

## File Data Chain

Files are stored as a linked list of sectors.

### Sector Data Format

| Offset  | Size | Description                                   |
| ------- | ---- | --------------------------------------------- |
| $00     | 1    | Track of next sector (0 = last sector)        |
| $01     | 1    | Sector of next sector (or bytes used if last) |
| $02-$FF | 254  | Data bytes                                    |

### Chain Rules

- First sector is pointed to by directory entry
- Each sector points to the next
- Last sector has track = 0
- In last sector, sector byte = number of used bytes (2-255)
- Maximum usable data per sector: 254 bytes

---

## Creating a D64 File

### Minimal Steps

1. Create 683 sectors (174,848 bytes)
2. Format BAM at track 18, sector 0:
   - Set directory pointer
   - Set disk name and ID
   - Initialize free sector counts
3. Create empty directory at track 18, sector 1
4. Write file data starting from available sectors
5. Update BAM to mark used sectors
6. Add directory entry

### Example BAM Initialization

```
Offset $00-$01: $12 $01 (track 18, sector 1 = directory)
Offset $02-$03: $41 $00 (DOS version 'A', unused)
Offset $04-$07: $15 $FF $FF $1F (track 1: 21 free, all sectors available)
...
Offset $90-$9F: Disk name in PETSCII
Offset $A2-$A3: Disk ID (2 characters)
Offset $A5-$A6: $32 $41 ("2A" = DOS type)
```

---

## Tools for D64 Manipulation

### c1541 (VICE)

```bash
# Create new D64
c1541 -format "diskname,id" d64 disk.d64

# Add file to D64
c1541 -attach disk.d64 -write program.prg

# List directory
c1541 -attach disk.d64 -list

# Extract file
c1541 -attach disk.d64 -read program.prg
```

### cc1541

Alternative tool with more control over sector placement:

```bash
# Create D64 with file
cc1541 -n "diskname" -i "id" -w program.prg disk.d64
```

---

## Sources

- [D64 Format - University of Waterloo](https://ist.uwaterloo.ca/~schepers/formats/D64.TXT)
- [D64 - C64-Wiki](https://www.c64-wiki.com/wiki/D64)
- [Understanding D64 File Structure](https://theoasisbbs.com/understanding-d64-file-structure/)
- [VICE Manual - c1541](https://vice-emu.sourceforge.io/vice_14.html)
