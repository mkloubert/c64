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

//! File watching for hot-reload functionality.
//!
//! This module provides the `SourceWatcher` struct for monitoring source files
//! and triggering recompilation on changes.
//!
//! # Editor Compatibility
//!
//! Different editors save files differently:
//! - **Direct write**: Truncate and write (vim with `set nobackup nowritebackup`)
//! - **Atomic save**: Write to temp file, then rename (VS Code, most modern editors)
//! - **Backup save**: Rename original to backup, write new file
//!
//! The watcher handles all these patterns by watching the parent directory
//! and filtering events for the target files.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::RunnerError;

/// Debounce window for file change events.
/// Multiple rapid changes within this window are collapsed into one.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Watches source files for changes.
///
/// Uses the `notify` crate to monitor file system events and trigger
/// recompilation when source files are modified.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use cobra64::runner::SourceWatcher;
///
/// let paths = vec![PathBuf::from("main.cb64"), PathBuf::from("utils.cb64")];
/// let watcher = SourceWatcher::new(&paths).expect("Failed to create watcher");
///
/// println!("Watching for changes...");
/// watcher.wait_for_change().expect("Watch error");
/// println!("File changed!");
/// ```
pub struct SourceWatcher {
    /// The underlying file system watcher.
    _watcher: RecommendedWatcher,
    /// Receiver for file system events.
    rx: Receiver<Result<Event, notify::Error>>,
    /// Paths being watched.
    paths: Vec<PathBuf>,
}

impl SourceWatcher {
    /// Create a new SourceWatcher for the given paths.
    ///
    /// Watches the specified files for modifications. The watcher also monitors
    /// parent directories to catch atomic save operations (write to temp + rename).
    ///
    /// # Arguments
    ///
    /// * `paths` - Source file paths to watch
    ///
    /// # Errors
    ///
    /// Returns `RunnerError::WatchError` if the watcher cannot be created.
    pub fn new(paths: &[PathBuf]) -> Result<Self, RunnerError> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| RunnerError::WatchError(format!("Failed to create watcher: {}", e)))?;

        // Canonicalize paths and collect unique parent directories
        let mut canonical_paths = Vec::new();
        let mut watched_dirs = std::collections::HashSet::new();

        for path in paths {
            let canonical = path.canonicalize().map_err(|e| {
                RunnerError::WatchError(format!("Cannot resolve path {}: {}", path.display(), e))
            })?;

            // Watch the parent directory to catch atomic saves
            if let Some(parent) = canonical.parent() {
                if watched_dirs.insert(parent.to_path_buf()) {
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .map_err(|e| {
                            RunnerError::WatchError(format!(
                                "Failed to watch {}: {}",
                                parent.display(),
                                e
                            ))
                        })?;
                }
            }

            canonical_paths.push(canonical);
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            paths: canonical_paths,
        })
    }

    /// Get the watched paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Wait for a file change event.
    ///
    /// Blocks until a watched file is modified. Implements debouncing to
    /// collapse multiple rapid changes into a single event.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when a change is detected, or an error if watching fails.
    pub fn wait_for_change(&self) -> Result<(), RunnerError> {
        loop {
            // Block waiting for an event
            let event = self
                .rx
                .recv()
                .map_err(|e| RunnerError::WatchError(format!("Watch channel closed: {}", e)))?
                .map_err(|e| RunnerError::WatchError(format!("Watch error: {}", e)))?;

            // Check if this event is relevant to our watched files
            if !self.is_relevant_event(&event) {
                continue;
            }

            // Wait for debounce period to collapse rapid changes
            std::thread::sleep(DEBOUNCE_DURATION);

            // Drain any events that came during the debounce window
            self.drain_pending_events();

            return Ok(());
        }
    }

    /// Check if an event is relevant to our watched files.
    fn is_relevant_event(&self, event: &Event) -> bool {
        // Only care about modifications and creates (for atomic saves)
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {}
            _ => return false,
        }

        // Check if any event path matches our watched files
        for event_path in &event.paths {
            // Try to canonicalize for comparison
            let canonical = event_path
                .canonicalize()
                .unwrap_or_else(|_| event_path.clone());

            for watched_path in &self.paths {
                if canonical == *watched_path {
                    return true;
                }
                // Also check filename match (for atomic saves where path might differ briefly)
                if let (Some(event_name), Some(watched_name)) =
                    (canonical.file_name(), watched_path.file_name())
                {
                    if event_name == watched_name {
                        if let (Some(event_parent), Some(watched_parent)) =
                            (canonical.parent(), watched_path.parent())
                        {
                            if event_parent == watched_parent {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Drain any pending events from the channel.
    fn drain_pending_events(&self) {
        while self.rx.try_recv().is_ok() {
            // Discard event
        }
    }
}

/// Check if a path matches any of the watched paths.
#[allow(dead_code)]
pub fn path_matches(path: &Path, watched_paths: &[PathBuf]) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for watched in watched_paths {
        if canonical == *watched {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_source_watcher_new() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.cb64");
        File::create(&file_path).unwrap();

        let watcher = SourceWatcher::new(&[file_path.clone()]).unwrap();
        assert_eq!(watcher.paths().len(), 1);
    }

    #[test]
    fn test_source_watcher_nonexistent_file() {
        let result = SourceWatcher::new(&[PathBuf::from("/nonexistent/path/file.cb64")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_source_watcher_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("main.cb64");
        let file2 = temp_dir.path().join("utils.cb64");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let watcher = SourceWatcher::new(&[file1, file2]).unwrap();
        assert_eq!(watcher.paths().len(), 2);
    }

    #[test]
    fn test_path_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.cb64");
        File::create(&file_path).unwrap();

        let canonical = file_path.canonicalize().unwrap();
        let watched = vec![canonical.clone()];

        assert!(path_matches(&file_path, &watched));
        assert!(!path_matches(&PathBuf::from("/other/path.cb64"), &watched));
    }

    #[test]
    fn test_file_change_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("watch_test.cb64");

        // Create initial file
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "def main():").unwrap();
            writeln!(file, "    pass").unwrap();
        }

        let watcher = SourceWatcher::new(&[file_path.clone()]).unwrap();

        // Spawn thread to modify file after a short delay
        let file_path_clone = file_path.clone();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&file_path_clone)
                .unwrap();
            writeln!(file, "def main():").unwrap();
            writeln!(file, "    println(\"CHANGED\")").unwrap();
        });

        // This should return when the file is modified
        let result = watcher.wait_for_change();
        handle.join().unwrap();

        assert!(result.is_ok(), "Should detect file change");
    }

    #[test]
    fn test_debounce_duration() {
        // Just verify the constant is reasonable
        assert!(DEBOUNCE_DURATION.as_millis() >= 50);
        assert!(DEBOUNCE_DURATION.as_millis() <= 500);
    }
}
