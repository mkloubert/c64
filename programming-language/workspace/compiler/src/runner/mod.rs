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

//! Runner module for VICE emulator integration and file watching.
//!
//! This module provides functionality to:
//! - Detect and launch the VICE emulator
//! - Watch source files for changes
//! - Hot-reload programs in a running VICE instance

mod vice;
mod watcher;

pub use vice::{check_vice_version, find_vice, ViceRunner};
pub use watcher::SourceWatcher;

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during runner operations.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// VICE emulator was not found on the system.
    #[error("VICE emulator not found. Install VICE (x64sc) or specify path with --vice-path")]
    ViceNotFound,

    /// VICE emulator failed to start.
    #[error("Failed to start VICE: {0}")]
    ViceStartFailed(#[from] io::Error),

    /// Failed to connect to VICE remote monitor.
    #[error("Failed to connect to VICE monitor on port {port}: {source}")]
    ConnectionFailed {
        port: u16,
        #[source]
        source: io::Error,
    },

    /// Error communicating with VICE monitor.
    #[error("Monitor command failed: {0}")]
    MonitorError(String),

    /// Error watching files.
    #[error("File watch error: {0}")]
    WatchError(String),

    /// VICE version is too old or could not be determined.
    #[error("VICE version check failed: {0}")]
    VersionError(String),

    /// The specified VICE path does not exist.
    #[error("VICE path does not exist: {0}")]
    InvalidVicePath(PathBuf),
}
