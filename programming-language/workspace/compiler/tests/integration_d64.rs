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

//! Integration tests for D64 disk image generation.

use cobra64::output::d64::{sector_offset, sectors_in_track, D64Builder, D64_SIZE, NUM_TRACKS};

/// Test that D64 images have correct size.
#[test]
fn test_d64_size() {
    let builder = D64Builder::new();
    assert_eq!(builder.data().len(), D64_SIZE);
    assert_eq!(D64_SIZE, 174_848);
}

/// Test track/sector layout.
#[test]
fn test_track_sector_layout() {
    // Tracks 1-17: 21 sectors each
    for track in 1..=17 {
        assert_eq!(
            sectors_in_track(track),
            21,
            "Track {} should have 21 sectors",
            track
        );
    }

    // Tracks 18-24: 19 sectors each
    for track in 18..=24 {
        assert_eq!(
            sectors_in_track(track),
            19,
            "Track {} should have 19 sectors",
            track
        );
    }

    // Tracks 25-30: 18 sectors each
    for track in 25..=30 {
        assert_eq!(
            sectors_in_track(track),
            18,
            "Track {} should have 18 sectors",
            track
        );
    }

    // Tracks 31-35: 17 sectors each
    for track in 31..=35 {
        assert_eq!(
            sectors_in_track(track),
            17,
            "Track {} should have 17 sectors",
            track
        );
    }

    // Invalid tracks
    assert_eq!(sectors_in_track(0), 0);
    assert_eq!(sectors_in_track(36), 0);
}

/// Test sector offset calculation.
#[test]
fn test_sector_offsets() {
    // Track 1, sector 0 starts at offset 0
    assert_eq!(sector_offset(1, 0), Some(0));

    // Track 1, sector 1 starts at offset 256
    assert_eq!(sector_offset(1, 1), Some(256));

    // Track 2, sector 0 starts after all of track 1 (21 * 256 = 5376)
    assert_eq!(sector_offset(2, 0), Some(21 * 256));

    // Track 18 (directory track) offset
    let expected_track_18_offset: usize = (0..18)
        .map(|t| {
            if t == 0 {
                0
            } else {
                sectors_in_track(t) as usize * 256
            }
        })
        .sum();
    assert_eq!(sector_offset(18, 0), Some(expected_track_18_offset));

    // Invalid sector offsets
    assert!(sector_offset(0, 0).is_none());
    assert!(sector_offset(1, 21).is_none()); // Track 1 only has sectors 0-20
    assert!(sector_offset(36, 0).is_none());
}

/// Test BAM initialization.
#[test]
fn test_bam_initialization() {
    let builder = D64Builder::new();
    let data = builder.data();

    // BAM is at track 18, sector 0
    let bam_offset = sector_offset(18, 0).unwrap();

    // First two bytes point to first directory sector (track 18, sector 1)
    assert_eq!(data[bam_offset], 18, "BAM should point to directory track");
    assert_eq!(
        data[bam_offset + 1],
        1,
        "BAM should point to directory sector 1"
    );

    // DOS version
    assert_eq!(data[bam_offset + 2], 0x41, "DOS version should be 'A'");

    // Disk name at offset 0x90 should be "COBRA64"
    let disk_name = &data[bam_offset + 0x90..bam_offset + 0x90 + 7];
    assert_eq!(disk_name, b"COBRA64", "Disk name should be COBRA64");
}

/// Test adding a file to the disk.
#[test]
fn test_add_file() {
    let mut builder = D64Builder::new();

    // Add a simple test file
    let test_data = vec![0x01, 0x08, 0xA9, 0x00, 0x60]; // Load addr + LDA #0 + RTS
    builder.add_file("TESTPROG", &test_data).unwrap();

    let data = builder.data();

    // Check directory entry at track 18, sector 1
    let dir_offset = sector_offset(18, 1).unwrap();

    // File type should be PRG ($82)
    assert_eq!(data[dir_offset + 2], 0x82, "File type should be PRG");

    // Filename should be "TESTPROG" padded with $A0
    let filename = &data[dir_offset + 5..dir_offset + 5 + 8];
    assert_eq!(filename, b"TESTPROG", "Filename should be TESTPROG");
}

/// Test compiling a program to D64.
#[test]
fn test_compile_to_d64() {
    let source = r#"
def main():
    println("TEST")
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    let mut builder = D64Builder::new();

    // Add load address to create complete PRG
    let mut full_prg = Vec::with_capacity(code.len() + 2);
    full_prg.extend_from_slice(&0x0801u16.to_le_bytes());
    full_prg.extend_from_slice(&code);

    builder
        .add_file("MYPROGRAM", &full_prg)
        .expect("Adding file should succeed");

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_compile.d64");

    builder.write(&path).expect("Writing D64 should succeed");

    // Verify file size
    let metadata = std::fs::metadata(&path).expect("File should exist");
    assert_eq!(metadata.len() as usize, D64_SIZE);

    // Clean up
    std::fs::remove_file(&path).ok();
}

/// Test that the directory can hold multiple files.
#[test]
fn test_multiple_files() {
    let mut builder = D64Builder::new();

    // Add multiple files
    for i in 0..5 {
        let name = format!("FILE{}", i);
        let data = vec![0x01, 0x08, 0x60]; // Minimal PRG
        builder
            .add_file(&name, &data)
            .expect("Adding file should succeed");
    }

    let data = builder.data();
    let dir_offset = sector_offset(18, 1).unwrap();

    // Check that all 5 files exist in directory
    for i in 0..5 {
        let entry_offset = dir_offset + i * 32;
        assert_eq!(
            data[entry_offset + 2],
            0x82,
            "File {} should be PRG type",
            i
        );
    }
}

/// Test sector allocation avoids directory track.
#[test]
fn test_sector_allocation_avoids_directory() {
    let mut builder = D64Builder::new();

    // Add a large file that would span multiple sectors
    let large_data = vec![0u8; 5000]; // About 20 sectors
    builder
        .add_file("BIGFILE", &large_data)
        .expect("Adding file should succeed");

    let data = builder.data();
    let dir_offset = sector_offset(18, 1).unwrap();

    // Get first track/sector of file
    let first_track = data[dir_offset + 3];
    let first_sector = data[dir_offset + 4];

    // File should not start on directory track
    assert_ne!(first_track, 18, "File should not be on directory track");

    // First sector should be track 1
    assert_eq!(first_track, 1);
    assert_eq!(first_sector, 0);
}

/// Test write_d64 convenience function.
#[test]
fn test_write_d64_function() {
    let source = r#"
def main():
    pass
"#;

    let code = cobra64::compile(source).expect("Compilation should succeed");

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("test_write_d64.d64");

    cobra64::output::write_d64(&code, &path, "HELLO").expect("write_d64 should succeed");

    // Verify file was created with correct size
    let metadata = std::fs::metadata(&path).expect("File should exist");
    assert_eq!(metadata.len() as usize, D64_SIZE);

    // Read back and verify BAM
    let data = std::fs::read(&path).expect("Should be able to read D64");
    let bam_offset = sector_offset(18, 0).unwrap();

    // Check disk name
    let disk_name = &data[bam_offset + 0x90..bam_offset + 0x90 + 7];
    assert_eq!(disk_name, b"COBRA64");

    // Clean up
    std::fs::remove_file(&path).ok();
}

/// Test total sector count.
#[test]
fn test_total_sectors() {
    let mut total = 0u32;
    for track in 1..=NUM_TRACKS {
        total += sectors_in_track(track) as u32;
    }
    assert_eq!(total, 683, "Total sectors should be 683");
}

/// Test that D64 size matches sector count.
#[test]
fn test_d64_size_calculation() {
    let mut calculated_size = 0usize;
    for track in 1..=NUM_TRACKS {
        calculated_size += sectors_in_track(track) as usize * 256;
    }
    assert_eq!(calculated_size, D64_SIZE);
}
