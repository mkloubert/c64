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

//! PRG file writer for the Cobra64 compiler.
//!
//! PRG format is very simple:
//! - 2-byte load address (little-endian)
//! - Program data

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// The default load address for C64 programs.
pub const DEFAULT_LOAD_ADDRESS: u16 = 0x0801;

/// Write a PRG file.
///
/// The PRG file format consists of:
/// 1. A 2-byte load address in little-endian format
/// 2. The program data
pub fn write_prg(code: &[u8], path: &Path) -> io::Result<()> {
    write_prg_with_address(code, path, DEFAULT_LOAD_ADDRESS)
}

/// Write a PRG file with a custom load address.
pub fn write_prg_with_address(code: &[u8], path: &Path, load_address: u16) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Write load address (little-endian)
    file.write_all(&load_address.to_le_bytes())?;

    // Write program data
    file.write_all(code)?;

    Ok(())
}

/// Read a PRG file and return the load address and code.
#[allow(dead_code)]
pub fn read_prg(path: &Path) -> io::Result<(u16, Vec<u8>)> {
    let data = std::fs::read(path)?;

    if data.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PRG file too short",
        ));
    }

    let load_address = u16::from_le_bytes([data[0], data[1]]);
    let code = data[2..].to_vec();

    Ok((load_address, code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_and_read_prg() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_cobra64.prg");

        let code = vec![0xA9, 0x00, 0x60]; // LDA #$00, RTS

        write_prg(&code, &path).unwrap();

        let (load_addr, read_code) = read_prg(&path).unwrap();

        assert_eq!(load_addr, DEFAULT_LOAD_ADDRESS);
        assert_eq!(read_code, code);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_custom_load_address() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_cobra64_custom.prg");

        let code = vec![0x60]; // RTS
        let custom_address = 0xC000;

        write_prg_with_address(&code, &path, custom_address).unwrap();

        let (load_addr, _) = read_prg(&path).unwrap();

        assert_eq!(load_addr, custom_address);

        fs::remove_file(&path).ok();
    }
}
