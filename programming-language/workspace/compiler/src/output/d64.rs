// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
// Copyright (C) 2026  Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! D64 disk image writer for the Cobra64 compiler.
//!
//! D64 is the standard disk image format for the Commodore 64.
//! It represents a 1541 floppy disk with:
//! - 35 tracks
//! - Variable sectors per track (17-21)
//! - 256 bytes per sector
//! - Total: 683 sectors = 174,848 bytes

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Total size of a D64 image in bytes.
pub const D64_SIZE: usize = 174_848;

/// Number of tracks on a 1541 disk.
pub const NUM_TRACKS: u8 = 35;

/// Directory track number.
#[allow(dead_code)]
pub const DIRECTORY_TRACK: u8 = 18;

/// Sectors per track for each track (1-indexed).
const SECTORS_PER_TRACK: [u8; 36] = [
    0, // Track 0 doesn't exist
    21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, // Tracks 1-17
    19, 19, 19, 19, 19, 19, 19, // Tracks 18-24
    18, 18, 18, 18, 18, 18, // Tracks 25-30
    17, 17, 17, 17, 17, // Tracks 31-35
];

/// Get the number of sectors for a given track.
pub fn sectors_in_track(track: u8) -> u8 {
    if track == 0 || track > NUM_TRACKS {
        0
    } else {
        SECTORS_PER_TRACK[track as usize]
    }
}

/// Get the byte offset for a track/sector in the D64 image.
pub fn sector_offset(track: u8, sector: u8) -> Option<usize> {
    if track == 0 || track > NUM_TRACKS || sector >= sectors_in_track(track) {
        return None;
    }

    let mut offset = 0usize;
    for t in 1..track {
        offset += sectors_in_track(t) as usize * 256;
    }
    offset += sector as usize * 256;

    Some(offset)
}

/// A D64 disk image builder.
pub struct D64Builder {
    /// The disk image data.
    data: Vec<u8>,
    /// Current track for file allocation.
    current_track: u8,
    /// Current sector for file allocation.
    current_sector: u8,
}

impl D64Builder {
    /// Create a new empty D64 image.
    pub fn new() -> Self {
        let mut builder = Self {
            data: vec![0; D64_SIZE],
            current_track: 1,
            current_sector: 0,
        };
        builder.initialize_bam();
        builder
    }

    /// Initialize the Block Availability Map (BAM) at track 18, sector 0.
    fn initialize_bam(&mut self) {
        let bam_offset = sector_offset(18, 0).unwrap();

        // Track/sector of first directory sector
        self.data[bam_offset] = 18; // Track
        self.data[bam_offset + 1] = 1; // Sector

        // DOS version type
        self.data[bam_offset + 2] = 0x41; // 'A' - DOS 2.6

        // Unused
        self.data[bam_offset + 3] = 0x00;

        // BAM entries for tracks 1-35 (4 bytes each)
        for track in 1..=NUM_TRACKS {
            let entry_offset = bam_offset + 4 + ((track - 1) as usize * 4);
            let sectors = sectors_in_track(track);

            // Free sector count
            self.data[entry_offset] = sectors;

            // Bitmap of free sectors (1 = free, 0 = used)
            // Initially all sectors are free except directory track
            if track == 18 {
                // Directory track: mark BAM and first directory sector as used
                self.data[entry_offset] = sectors - 2;
                self.data[entry_offset + 1] = 0xFC; // Sectors 0,1 used
                self.data[entry_offset + 2] = 0xFF;
                self.data[entry_offset + 3] = 0x07; // Only 19 sectors
            } else {
                // All sectors free
                let full_bytes = sectors / 8;
                let remaining = sectors % 8;

                for i in 0..full_bytes {
                    self.data[entry_offset + 1 + i as usize] = 0xFF;
                }
                if remaining > 0 {
                    self.data[entry_offset + 1 + full_bytes as usize] = (1 << remaining) - 1;
                }
            }
        }

        // Disk name (16 chars, padded with $A0)
        let disk_name = b"COBRA64         ";
        for (i, &c) in disk_name.iter().enumerate() {
            self.data[bam_offset + 0x90 + i] = c;
        }

        // $A0 padding
        self.data[bam_offset + 0xA0] = 0xA0;
        self.data[bam_offset + 0xA1] = 0xA0;

        // Disk ID
        self.data[bam_offset + 0xA2] = b'0';
        self.data[bam_offset + 0xA3] = b'1';

        // $A0
        self.data[bam_offset + 0xA4] = 0xA0;

        // DOS type
        self.data[bam_offset + 0xA5] = b'2';
        self.data[bam_offset + 0xA6] = b'A';

        // Padding with $A0
        for i in 0xA7..=0xAA {
            self.data[bam_offset + i] = 0xA0;
        }

        // Initialize first directory sector (track 18, sector 1)
        let dir_offset = sector_offset(18, 1).unwrap();
        self.data[dir_offset] = 0x00; // No next directory sector
        self.data[dir_offset + 1] = 0xFF; // End marker
    }

    /// Mark a sector as used in the BAM.
    fn mark_sector_used(&mut self, track: u8, sector: u8) {
        if track == 0 || track > NUM_TRACKS {
            return;
        }

        let bam_offset = sector_offset(18, 0).unwrap();
        let entry_offset = bam_offset + 4 + ((track - 1) as usize * 4);

        // Decrement free count
        if self.data[entry_offset] > 0 {
            self.data[entry_offset] -= 1;
        }

        // Clear bit in bitmap
        let byte_index = (sector / 8) as usize;
        let bit_index = sector % 8;
        self.data[entry_offset + 1 + byte_index] &= !(1 << bit_index);
    }

    /// Find the next free sector, avoiding the directory track.
    fn find_free_sector(&mut self) -> Option<(u8, u8)> {
        // Start from current position
        let mut track = self.current_track;
        let mut sector = self.current_sector;

        // Try to find a free sector
        for _ in 0..683 {
            // Skip directory track
            if track == 18 {
                track = 19;
                sector = 0;
            }

            if track > NUM_TRACKS {
                track = 1;
                sector = 0;
            }

            let bam_offset = sector_offset(18, 0).unwrap();
            let entry_offset = bam_offset + 4 + ((track - 1) as usize * 4);

            // Check if this track has free sectors
            if self.data[entry_offset] > 0 {
                // Check if this specific sector is free
                let byte_index = (sector / 8) as usize;
                let bit_index = sector % 8;

                if self.data[entry_offset + 1 + byte_index] & (1 << bit_index) != 0 {
                    self.current_track = track;
                    self.current_sector = sector + 1;
                    if self.current_sector >= sectors_in_track(track) {
                        self.current_track += 1;
                        self.current_sector = 0;
                    }
                    return Some((track, sector));
                }
            }

            // Try next sector
            sector += 1;
            if sector >= sectors_in_track(track) {
                track += 1;
                sector = 0;
            }
        }

        None
    }

    /// Add a file to the disk image.
    pub fn add_file(&mut self, name: &str, data: &[u8]) -> io::Result<()> {
        // Find space for directory entry
        let dir_offset = sector_offset(18, 1).unwrap();

        // Find first empty directory entry (8 entries per sector, 32 bytes each)
        let mut entry_offset = None;
        for i in 0..8 {
            let offset = dir_offset + i * 32;
            if self.data[offset + 2] == 0 {
                // Empty entry (file type = 0)
                entry_offset = Some(offset);
                break;
            }
        }

        let entry_offset =
            entry_offset.ok_or_else(|| io::Error::other("Directory full"))?;

        // Allocate sectors for file data
        let mut file_sectors: Vec<(u8, u8)> = Vec::new();
        let mut remaining = data;

        while !remaining.is_empty() {
            let (track, sector) = self
                .find_free_sector()
                .ok_or_else(|| io::Error::other("Disk full"))?;

            self.mark_sector_used(track, sector);
            file_sectors.push((track, sector));

            let chunk_size = remaining.len().min(254);
            remaining = &remaining[chunk_size..];
        }

        // Write file data to sectors
        for (i, &(track, sector)) in file_sectors.iter().enumerate() {
            let sector_offset = sector_offset(track, sector).unwrap();
            let data_start = i * 254;
            let data_end = (data_start + 254).min(data.len());
            let chunk = &data[data_start..data_end];

            if i < file_sectors.len() - 1 {
                // Not the last sector
                let (next_track, next_sector) = file_sectors[i + 1];
                self.data[sector_offset] = next_track;
                self.data[sector_offset + 1] = next_sector;
            } else {
                // Last sector
                self.data[sector_offset] = 0x00;
                self.data[sector_offset + 1] = (chunk.len() + 1) as u8;
            }

            // Copy data
            self.data[sector_offset + 2..sector_offset + 2 + chunk.len()].copy_from_slice(chunk);
        }

        // Write directory entry
        let (first_track, first_sector) = file_sectors.first().copied().unwrap_or((0, 0));

        // File type: PRG ($82 = PRG + "closed" bit)
        self.data[entry_offset + 2] = 0x82;

        // First track/sector
        self.data[entry_offset + 3] = first_track;
        self.data[entry_offset + 4] = first_sector;

        // Filename (16 chars, padded with $A0)
        let name_bytes = name.to_uppercase().as_bytes().to_vec();
        for i in 0..16 {
            self.data[entry_offset + 5 + i] = if i < name_bytes.len() {
                name_bytes[i]
            } else {
                0xA0
            };
        }

        // File size in sectors (2 bytes, little-endian)
        let sectors = file_sectors.len() as u16;
        self.data[entry_offset + 0x1E] = (sectors & 0xFF) as u8;
        self.data[entry_offset + 0x1F] = (sectors >> 8) as u8;

        Ok(())
    }

    /// Write the D64 image to a file.
    pub fn write(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(&self.data)?;
        Ok(())
    }

    /// Get the raw image data.
    #[allow(dead_code)]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for D64Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a PRG file to a D64 disk image.
pub fn write_d64(prg_data: &[u8], path: &Path, program_name: &str) -> io::Result<()> {
    let mut builder = D64Builder::new();

    // Add load address to create complete PRG
    let mut full_prg = Vec::with_capacity(prg_data.len() + 2);
    full_prg.extend_from_slice(&0x0801u16.to_le_bytes());
    full_prg.extend_from_slice(prg_data);

    builder.add_file(program_name, &full_prg)?;
    builder.write(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sectors_per_track() {
        assert_eq!(sectors_in_track(0), 0);
        assert_eq!(sectors_in_track(1), 21);
        assert_eq!(sectors_in_track(17), 21);
        assert_eq!(sectors_in_track(18), 19);
        assert_eq!(sectors_in_track(35), 17);
        assert_eq!(sectors_in_track(36), 0);
    }

    #[test]
    fn test_sector_offset() {
        assert_eq!(sector_offset(1, 0), Some(0));
        assert_eq!(sector_offset(1, 1), Some(256));
        assert_eq!(sector_offset(2, 0), Some(21 * 256));
        assert!(sector_offset(0, 0).is_none());
        assert!(sector_offset(1, 21).is_none());
    }

    #[test]
    fn test_d64_creation() {
        let builder = D64Builder::new();
        assert_eq!(builder.data().len(), D64_SIZE);
    }

    #[test]
    fn test_add_file() {
        let mut builder = D64Builder::new();
        let data = vec![0x00, 0x08, 0x60]; // Load address + RTS

        builder.add_file("TEST", &data).unwrap();

        // Check directory entry exists
        let dir_offset = sector_offset(18, 1).unwrap();
        assert_eq!(builder.data()[dir_offset + 2], 0x82); // PRG file type
    }
}
