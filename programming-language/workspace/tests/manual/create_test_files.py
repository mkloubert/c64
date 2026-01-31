#!/usr/bin/env python3
"""
Create test PRG and D64 files to validate the feasibility analysis.

This script creates:
1. A minimal PRG that changes the border color
2. A PRG with BASIC stub for autostart
3. A D64 disk image containing the program
"""

import struct
from pathlib import Path

# Output directory
OUTPUT_DIR = Path(__file__).parent


def create_minimal_prg():
    """
    Create a minimal PRG that changes border color to black.

    Load address: $C000
    Usage: LOAD "MINIMAL.PRG",8,1 then SYS 49152

    Assembly:
        * = $C000
        LDA #$00        ; Load 0 (black) into accumulator
        STA $D020       ; Store to border color register
        STA $D021       ; Store to background color register
        RTS             ; Return
    """
    load_address = 0xC000

    # Machine code
    code = bytes([
        0xA9, 0x00,       # LDA #$00
        0x8D, 0x20, 0xD0, # STA $D020
        0x8D, 0x21, 0xD0, # STA $D021
        0x60,             # RTS
    ])

    # PRG = load address (little-endian) + code
    prg = struct.pack('<H', load_address) + code

    output_path = OUTPUT_DIR / "minimal.prg"
    output_path.write_bytes(prg)
    print(f"Created: {output_path} ({len(prg)} bytes)")
    print(f"  Load address: ${load_address:04X}")
    print(f"  Usage: LOAD \"MINIMAL\",8,1 then SYS {load_address}")

    return output_path


def create_autostart_prg():
    """
    Create a PRG with BASIC stub that auto-runs when you type RUN.

    The BASIC stub is: 10 SYS 2064

    Load address: $0801 (BASIC program area)
    Usage: LOAD "HELLO.PRG",8 then RUN
    """
    load_address = 0x0801

    # BASIC stub: 10 SYS 2064
    # Format: [next_line_ptr][line_number][tokens][null][end_marker]

    basic_stub = bytearray()

    # Pointer to next BASIC line (will calculate after)
    # Line starts at $0801, we need pointer to end
    next_line_ptr = 0x080D  # $0801 + 12 bytes
    basic_stub.extend(struct.pack('<H', next_line_ptr))  # $0801-$0802

    # Line number: 10
    basic_stub.extend(struct.pack('<H', 10))  # $0803-$0804

    # SYS token
    basic_stub.append(0x9E)  # $0805

    # Space
    basic_stub.append(0x20)  # $0806

    # Address as ASCII: "2064" (points to $0810)
    basic_stub.extend(b'2064')  # $0807-$080A

    # End of line
    basic_stub.append(0x00)  # $080B

    # End of BASIC program (null pointer)
    basic_stub.extend([0x00, 0x00])  # $080C-$080D

    # Machine code starts at $080E = 2062... let me recalculate
    # Actually: $0801 + len(stub) = $0801 + 14 = $080F = 2063
    # Let's use 2064 = $0810, so we need padding

    # Recalculate for SYS 2064 ($0810)
    basic_stub = bytearray()

    # Next line pointer: end of this line
    basic_stub.extend([0x0C, 0x08])  # -> $080C

    # Line number 10
    basic_stub.extend([0x0A, 0x00])

    # SYS token + space + "2064" + null
    basic_stub.append(0x9E)  # SYS
    basic_stub.extend(b'2064')  # ASCII
    basic_stub.append(0x00)  # End of line

    # End of program
    basic_stub.extend([0x00, 0x00])

    # Pad to reach $0810
    while len(basic_stub) < (0x0810 - 0x0801):
        basic_stub.append(0x00)

    # Machine code at $0810
    # This program cycles border colors
    machine_code = bytes([
        # Initialize
        0xA9, 0x00,       # LDA #$00       ; Start with black

        # Main loop
        0x8D, 0x20, 0xD0, # STA $D020      ; Set border color
        0x8D, 0x21, 0xD0, # STA $D021      ; Set background color

        # Delay loop
        0xA2, 0x00,       # LDX #$00
        0xA0, 0x00,       # LDY #$00
        # delay_inner:
        0x88,             # DEY
        0xD0, 0xFD,       # BNE delay_inner (-3)
        0xCA,             # DEX
        0xD0, 0xFA,       # BNE delay_inner (-6, to DEY)

        # Next color
        0x18,             # CLC
        0x69, 0x01,       # ADC #$01
        0x29, 0x0F,       # AND #$0F       ; Keep in range 0-15

        # Loop forever
        0x4C, 0x12, 0x08, # JMP $0812      ; Jump to STA $D020
    ])

    # Combine
    prg_data = bytes(basic_stub) + machine_code

    # PRG file
    prg = struct.pack('<H', load_address) + prg_data

    output_path = OUTPUT_DIR / "hello.prg"
    output_path.write_bytes(prg)
    print(f"Created: {output_path} ({len(prg)} bytes)")
    print(f"  Load address: ${load_address:04X}")
    print(f"  Usage: LOAD \"HELLO\",8 then RUN")
    print(f"  Effect: Cycles through all 16 border colors")

    return output_path


def create_d64(prg_path: Path, disk_name: str = "TEST DISK", disk_id: str = "01"):
    """
    Create a D64 disk image containing the PRG file.

    D64 structure:
    - 35 tracks, 683 sectors total
    - Each sector is 256 bytes
    - Track 18 contains BAM (sector 0) and directory (sectors 1-18)
    """

    # Constants
    TRACKS = 35
    SECTOR_SIZE = 256
    TOTAL_SECTORS = 683
    D64_SIZE = TOTAL_SECTORS * SECTOR_SIZE  # 174848 bytes

    # Sectors per track (Zone Bit Recording)
    SECTORS_PER_TRACK = [
        21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,  # 1-17
        19, 19, 19, 19, 19, 19, 19,  # 18-24
        18, 18, 18, 18, 18, 18,      # 25-30
        17, 17, 17, 17, 17,          # 31-35
    ]

    def track_sector_to_offset(track: int, sector: int) -> int:
        """Convert track/sector to byte offset in D64."""
        offset = 0
        for t in range(1, track):
            offset += SECTORS_PER_TRACK[t - 1] * SECTOR_SIZE
        offset += sector * SECTOR_SIZE
        return offset

    # Initialize disk image with zeros
    disk = bytearray(D64_SIZE)

    # === BAM (Block Availability Map) at Track 18, Sector 0 ===
    bam_offset = track_sector_to_offset(18, 0)

    # Track/sector of first directory sector
    disk[bam_offset + 0] = 18  # Track
    disk[bam_offset + 1] = 1   # Sector

    # DOS version type
    disk[bam_offset + 2] = 0x41  # 'A'
    disk[bam_offset + 3] = 0x00  # Unused

    # BAM entries for tracks 1-35 (4 bytes each)
    bam_ptr = bam_offset + 4
    for track in range(1, TRACKS + 1):
        num_sectors = SECTORS_PER_TRACK[track - 1]

        if track == 18:
            # Track 18 is partially used (BAM + directory)
            # Mark sectors 0-1 as used, rest free
            free_sectors = num_sectors - 2
            disk[bam_ptr] = free_sectors
            disk[bam_ptr + 1] = 0xFC  # 11111100 - sectors 0,1 used
            disk[bam_ptr + 2] = 0xFF
            disk[bam_ptr + 3] = 0x07 if num_sectors > 16 else 0xFF
        else:
            # All sectors free
            disk[bam_ptr] = num_sectors
            # Set bits for available sectors
            if num_sectors >= 8:
                disk[bam_ptr + 1] = 0xFF
            else:
                disk[bam_ptr + 1] = (1 << num_sectors) - 1
            if num_sectors >= 16:
                disk[bam_ptr + 2] = 0xFF
            else:
                disk[bam_ptr + 2] = (1 << (num_sectors - 8)) - 1 if num_sectors > 8 else 0x00
            if num_sectors > 16:
                disk[bam_ptr + 3] = (1 << (num_sectors - 16)) - 1
            else:
                disk[bam_ptr + 3] = 0x00

        bam_ptr += 4

    # Disk name (16 bytes, padded with 0xA0)
    disk_name_bytes = disk_name.upper().encode('ascii')[:16]
    disk_name_bytes = disk_name_bytes.ljust(16, b'\xA0')
    disk[bam_offset + 0x90:bam_offset + 0xA0] = disk_name_bytes

    # Padding
    disk[bam_offset + 0xA0] = 0xA0
    disk[bam_offset + 0xA1] = 0xA0

    # Disk ID (2 bytes)
    disk_id_bytes = disk_id.encode('ascii')[:2].ljust(2, b'0')
    disk[bam_offset + 0xA2:bam_offset + 0xA4] = disk_id_bytes

    # Padding
    disk[bam_offset + 0xA4] = 0xA0

    # DOS type "2A"
    disk[bam_offset + 0xA5] = 0x32  # '2'
    disk[bam_offset + 0xA6] = 0x41  # 'A'

    # More padding
    for i in range(0xA7, 0xAB):
        disk[bam_offset + i] = 0xA0

    # === Directory at Track 18, Sector 1 ===
    dir_offset = track_sector_to_offset(18, 1)

    # First two bytes: track/sector of next directory block (0 = none)
    disk[dir_offset + 0] = 0x00
    disk[dir_offset + 1] = 0xFF  # Indicates last directory sector

    # === Add PRG file to directory ===
    prg_data = prg_path.read_bytes()
    prg_name = prg_path.stem.upper()[:16]

    # Directory entry starts at offset 2 in directory sector
    entry_offset = dir_offset + 2

    # File type: PRG (0x82 = PRG + closed)
    disk[entry_offset + 0] = 0x82

    # Track/sector of first data block
    data_track = 1
    data_sector = 0
    disk[entry_offset + 1] = data_track
    disk[entry_offset + 2] = data_sector

    # Filename (16 bytes, padded with 0xA0)
    filename = prg_name.encode('ascii').ljust(16, b'\xA0')
    disk[entry_offset + 3:entry_offset + 19] = filename

    # Unused bytes
    for i in range(19, 28):
        disk[entry_offset + i] = 0x00

    # File size in sectors
    file_size = len(prg_data)
    sectors_needed = (file_size + 253) // 254  # 254 bytes of data per sector
    disk[entry_offset + 28] = sectors_needed & 0xFF
    disk[entry_offset + 29] = (sectors_needed >> 8) & 0xFF

    # === Write file data ===
    # Mark sectors as used in BAM
    bam_ptr = bam_offset + 4  # Track 1 BAM entry

    data_offset = track_sector_to_offset(data_track, data_sector)
    remaining_data = prg_data
    current_track = data_track
    current_sector = data_sector

    while remaining_data:
        sector_offset = track_sector_to_offset(current_track, current_sector)

        # Mark sector as used in BAM
        bam_entry_offset = bam_offset + 4 + (current_track - 1) * 4
        disk[bam_entry_offset] -= 1  # Decrease free count
        byte_index = current_sector // 8
        bit_index = current_sector % 8
        disk[bam_entry_offset + 1 + byte_index] &= ~(1 << bit_index)

        if len(remaining_data) <= 254:
            # Last sector
            disk[sector_offset + 0] = 0x00  # No next track
            disk[sector_offset + 1] = len(remaining_data) + 1  # Bytes used
            disk[sector_offset + 2:sector_offset + 2 + len(remaining_data)] = remaining_data
            remaining_data = b''
        else:
            # More data follows
            next_sector = current_sector + 1
            if next_sector >= SECTORS_PER_TRACK[current_track - 1]:
                current_track += 1
                if current_track == 18:
                    current_track = 19  # Skip directory track
                next_sector = 0

            disk[sector_offset + 0] = current_track
            disk[sector_offset + 1] = next_sector
            disk[sector_offset + 2:sector_offset + 256] = remaining_data[:254]
            remaining_data = remaining_data[254:]
            current_sector = next_sector

    # Write D64 file
    output_path = OUTPUT_DIR / "test.d64"
    output_path.write_bytes(disk)
    print(f"Created: {output_path} ({len(disk)} bytes)")
    print(f"  Disk name: {disk_name}")
    print(f"  Contains: {prg_name}.PRG")

    return output_path


def main():
    print("=" * 60)
    print("C64 Test File Generator")
    print("=" * 60)
    print()

    # Create minimal PRG
    print("1. Creating minimal PRG...")
    minimal_prg = create_minimal_prg()
    print()

    # Create autostart PRG
    print("2. Creating autostart PRG...")
    hello_prg = create_autostart_prg()
    print()

    # Create D64 with the hello program
    print("3. Creating D64 disk image...")
    d64_path = create_d64(hello_prg, "HELLO DISK", "C6")
    print()

    print("=" * 60)
    print("Testing Instructions:")
    print("=" * 60)
    print()
    print("To test these files in VICE emulator:")
    print()
    print("  Option A: Load PRG directly")
    print("    x64sc -autostart hello.prg")
    print()
    print("  Option B: Load from D64")
    print("    x64sc test.d64")
    print("    Then type: LOAD \"*\",8,1")
    print("    Then type: RUN")
    print()
    print("Expected result: Border color cycles through all 16 colors")
    print()


if __name__ == "__main__":
    main()
