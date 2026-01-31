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

//! Output module for the Cobra64 compiler.
//!
//! This module handles writing compiled programs to disk in various formats:
//! - PRG files (raw C64 program files)
//! - D64 disk images

pub mod d64;
mod prg;

pub use d64::write_d64;
pub use prg::write_prg;

use std::path::Path;

/// Determine the output format from a file extension.
pub fn format_from_extension(path: &Path) -> Option<OutputFormat> {
    match path.extension()?.to_str()?.to_lowercase().as_str() {
        "prg" => Some(OutputFormat::Prg),
        "d64" => Some(OutputFormat::D64),
        _ => None,
    }
}

/// The output format for compiled programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PRG file (raw program with load address).
    Prg,
    /// D64 disk image.
    D64,
}

/// Write compiled code to a file in the specified format.
pub fn write_output(
    code: &[u8],
    path: &Path,
    format: OutputFormat,
    program_name: &str,
) -> std::io::Result<()> {
    match format {
        OutputFormat::Prg => write_prg(code, path),
        OutputFormat::D64 => write_d64(code, path, program_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            format_from_extension(Path::new("test.prg")),
            Some(OutputFormat::Prg)
        );
        assert_eq!(
            format_from_extension(Path::new("test.d64")),
            Some(OutputFormat::D64)
        );
        assert_eq!(
            format_from_extension(Path::new("test.PRG")),
            Some(OutputFormat::Prg)
        );
        assert_eq!(format_from_extension(Path::new("test.txt")), None);
    }
}
