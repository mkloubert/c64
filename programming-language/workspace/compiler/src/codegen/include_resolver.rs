// Cobra64 - A concept for a modern Python-like compiler creating C64 binaries
//
// Copyright (C) 2026 Marcel Joachim Kloubert <marcel@kloubert.dev>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Include file resolution for data blocks.
//!
//! This module handles resolving and reading binary files specified in
//! `include` directives within data blocks.

use crate::error::{CompileError, ErrorCode, Span};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Resolves and caches include file contents.
#[derive(Debug, Default)]
pub struct IncludeResolver {
    /// Base directory for resolving relative paths.
    base_dir: Option<PathBuf>,
    /// Cache of file contents to avoid re-reading.
    cache: HashMap<PathBuf, Vec<u8>>,
}

impl IncludeResolver {
    /// Create a new include resolver with no base directory.
    pub fn new() -> Self {
        Self {
            base_dir: None,
            cache: HashMap::new(),
        }
    }

    /// Create a new include resolver with a base directory.
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
            cache: HashMap::new(),
        }
    }

    /// Set the base directory for resolving relative paths.
    pub fn set_base_dir(&mut self, base_dir: impl Into<PathBuf>) {
        self.base_dir = Some(base_dir.into());
    }

    /// Resolve a path relative to the base directory.
    ///
    /// If the path is absolute, it is returned as-is.
    /// If the path is relative, it is resolved relative to the base directory.
    /// If no base directory is set, the path is resolved relative to the
    /// current working directory.
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);

        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref base) = self.base_dir {
            base.join(path)
        } else {
            path.to_path_buf()
        }
    }

    /// Read a file and return its contents.
    ///
    /// The file is cached for subsequent reads.
    pub fn read_file(&mut self, path: &str, span: Span) -> Result<Vec<u8>, CompileError> {
        let resolved = self.resolve_path(path);

        // Check cache first
        if let Some(data) = self.cache.get(&resolved) {
            return Ok(data.clone());
        }

        // Read the file
        let data = self.read_file_uncached(&resolved, path, span)?;

        // Cache it
        self.cache.insert(resolved, data.clone());

        Ok(data)
    }

    /// Read a file without caching.
    fn read_file_uncached(
        &self,
        resolved: &Path,
        original_path: &str,
        span: Span,
    ) -> Result<Vec<u8>, CompileError> {
        // Check if file exists
        if !resolved.exists() {
            let search_info = if let Some(ref base) = self.base_dir {
                format!(" (searched in: {})", base.display())
            } else {
                String::new()
            };

            return Err(CompileError::new(
                ErrorCode::FileNotFound,
                format!("File not found: \"{}\"{}", original_path, search_info),
                span,
            ));
        }

        // Try to read the file
        let mut file = match fs::File::open(resolved) {
            Ok(f) => f,
            Err(e) => {
                return Err(CompileError::new(
                    ErrorCode::FileReadError,
                    format!("Cannot read file \"{}\": {}", original_path, e),
                    span,
                ));
            }
        };

        let mut data = Vec::new();
        if let Err(e) = file.read_to_end(&mut data) {
            return Err(CompileError::new(
                ErrorCode::FileReadError,
                format!("Cannot read file \"{}\": {}", original_path, e),
                span,
            ));
        }

        Ok(data)
    }

    /// Read a file with optional offset and length.
    ///
    /// If offset is specified, reading starts from that byte position.
    /// If length is specified, only that many bytes are read.
    /// Both offset and length are validated against the file size.
    pub fn read_file_range(
        &mut self,
        path: &str,
        offset: Option<u32>,
        length: Option<u32>,
        span: Span,
    ) -> Result<Vec<u8>, CompileError> {
        let data = self.read_file(path, span)?;
        let file_size = data.len() as u32;

        let start = offset.unwrap_or(0) as usize;
        let end = if let Some(len) = length {
            start + len as usize
        } else {
            data.len()
        };

        // Validate offset
        if let Some(off) = offset {
            if off >= file_size {
                return Err(CompileError::new(
                    ErrorCode::IncludeOffsetOutOfBounds,
                    format!(
                        "Include offset ${:04X} exceeds file size ({} bytes)",
                        off, file_size
                    ),
                    span,
                ));
            }
        }

        // Validate length
        if end > data.len() {
            let actual_offset = offset.unwrap_or(0);
            let actual_length = length.unwrap_or(file_size - actual_offset);
            return Err(CompileError::new(
                ErrorCode::IncludeLengthOutOfBounds,
                format!(
                    "Include range exceeds file size: offset ${:04X} + length ${:04X} = ${:04X}, but file is only {} bytes",
                    actual_offset,
                    actual_length,
                    actual_offset + actual_length,
                    file_size
                ),
                span,
            ));
        }

        Ok(data[start..end].to_vec())
    }

    /// Clear the file cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_resolve_absolute_path() {
        let resolver = IncludeResolver::with_base_dir("/some/dir");
        let resolved = resolver.resolve_path("/absolute/path/file.bin");
        assert_eq!(resolved, PathBuf::from("/absolute/path/file.bin"));
    }

    #[test]
    fn test_resolve_relative_path() {
        let resolver = IncludeResolver::with_base_dir("/some/dir");
        let resolved = resolver.resolve_path("relative/file.bin");
        assert_eq!(resolved, PathBuf::from("/some/dir/relative/file.bin"));
    }

    #[test]
    fn test_resolve_relative_path_no_base() {
        let resolver = IncludeResolver::new();
        let resolved = resolver.resolve_path("relative/file.bin");
        assert_eq!(resolved, PathBuf::from("relative/file.bin"));
    }

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let content = vec![0xDE, 0xAD, 0xBE, 0xEF];
        create_test_file(temp_dir.path(), "test.bin", &content);

        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());
        let result = resolver.read_file("test.bin", Span::new(0, 10));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_read_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());
        let result = resolver.read_file("nonexistent.bin", Span::new(0, 10));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::FileNotFound);
    }

    #[test]
    fn test_read_file_range() {
        let temp_dir = TempDir::new().unwrap();
        let content: Vec<u8> = (0..16).collect();
        create_test_file(temp_dir.path(), "data.bin", &content);

        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());

        // Read with offset
        let result = resolver.read_file_range("data.bin", Some(4), None, Span::new(0, 10));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );

        // Read with offset and length
        let result = resolver.read_file_range("data.bin", Some(4), Some(4), Span::new(0, 10));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_read_file_range_offset_out_of_bounds() {
        let temp_dir = TempDir::new().unwrap();
        let content = vec![0x01, 0x02, 0x03, 0x04];
        create_test_file(temp_dir.path(), "small.bin", &content);

        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());
        let result = resolver.read_file_range("small.bin", Some(100), None, Span::new(0, 10));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IncludeOffsetOutOfBounds);
    }

    #[test]
    fn test_read_file_range_length_out_of_bounds() {
        let temp_dir = TempDir::new().unwrap();
        let content = vec![0x01, 0x02, 0x03, 0x04];
        create_test_file(temp_dir.path(), "small.bin", &content);

        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());
        let result = resolver.read_file_range("small.bin", Some(2), Some(10), Span::new(0, 10));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::IncludeLengthOutOfBounds);
    }

    #[test]
    fn test_file_caching() {
        let temp_dir = TempDir::new().unwrap();
        let content = vec![0xAA, 0xBB, 0xCC];
        create_test_file(temp_dir.path(), "cached.bin", &content);

        let mut resolver = IncludeResolver::with_base_dir(temp_dir.path());

        // First read
        let result1 = resolver.read_file("cached.bin", Span::new(0, 10));
        assert!(result1.is_ok());

        // Delete the file
        fs::remove_file(temp_dir.path().join("cached.bin")).unwrap();

        // Second read should still work (from cache)
        let result2 = resolver.read_file("cached.bin", Span::new(0, 10));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), content);
    }
}
