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

//! VICE emulator detection, launching, and remote monitor communication.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::RunnerError;

/// Connection timeout for remote monitor.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Read/write timeout for remote monitor commands.
const IO_TIMEOUT: Duration = Duration::from_secs(5);

// Binary monitor protocol constants
const STX: u8 = 0x02;
const API_VERSION: u8 = 0x02;
const CMD_AUTOSTART: u8 = 0xdd;
const CMD_EXIT: u8 = 0xaa;

/// List of VICE binary names to search for, in order of preference.
/// x64sc is the more accurate emulator, x64 is the faster legacy version.
const VICE_BINARIES: &[&str] = &["x64sc", "x64"];

/// Find VICE emulator on the system.
///
/// Searches for VICE binaries (x64sc, x64) in the system PATH.
/// Returns the full path to the first found binary, or `None` if not found.
///
/// # Example
///
/// ```no_run
/// use cobra64::runner::find_vice;
///
/// if let Some(vice_path) = find_vice() {
///     println!("Found VICE: {}", vice_path.display());
/// } else {
///     println!("VICE not found");
/// }
/// ```
pub fn find_vice() -> Option<PathBuf> {
    for binary in VICE_BINARIES {
        if let Ok(path) = which::which(binary) {
            return Some(path);
        }
    }
    None
}

/// Check VICE version and verify it supports remote monitor.
///
/// Runs `<vice_path> --version` and parses the output.
/// Returns the version string on success.
///
/// # Arguments
///
/// * `vice_path` - Path to the VICE binary
///
/// # Errors
///
/// Returns `RunnerError::VersionError` if version cannot be determined.
pub fn check_vice_version(vice_path: &Path) -> Result<String, RunnerError> {
    // VICE uses single-dash options: -version, not --version
    let output = Command::new(vice_path)
        .arg("-version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| RunnerError::VersionError(format!("Failed to run VICE: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // VICE outputs version info to stdout or stderr depending on version
    let version_output = if !stdout.is_empty() {
        stdout.to_string()
    } else {
        stderr.to_string()
    };

    // Parse version - look for patterns like "VICE x.y" or "x64sc x.y"
    // Example output: "x64sc (VICE 3.8)"
    if let Some(version) = parse_vice_version(&version_output) {
        Ok(version)
    } else if !version_output.is_empty() {
        // Return raw output if we can't parse it but something was returned
        Ok(version_output
            .lines()
            .next()
            .unwrap_or("unknown")
            .to_string())
    } else {
        Err(RunnerError::VersionError(
            "Could not determine VICE version".to_string(),
        ))
    }
}

/// Parse VICE version from command output.
fn parse_vice_version(output: &str) -> Option<String> {
    // Try to find "VICE x.y" or "VICE version x.y" pattern
    if let Some(pos) = output.find("VICE") {
        let after_vice = &output[pos..];
        // Skip "VICE" and any non-digit characters (like " " or " version ")
        let version: String = after_vice
            .chars()
            .skip(4) // Skip "VICE"
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if !version.is_empty() {
            return Some(format!("VICE {}", version));
        }
    }

    // Try to find version in format "x64sc x.y"
    for binary in VICE_BINARIES {
        if let Some(pos) = output.find(binary) {
            let after_binary = &output[pos + binary.len()..];
            let version: String = after_binary
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if !version.is_empty() {
                return Some(format!("{} {}", binary, version));
            }
        }
    }

    None
}

/// VICE emulator runner for launching and controlling VICE.
pub struct ViceRunner {
    /// Path to the VICE binary.
    vice_path: PathBuf,
    /// Remote monitor port.
    port: u16,
    /// Child process handle (if VICE was launched by us).
    child: Option<Child>,
}

impl ViceRunner {
    /// Create a new ViceRunner.
    ///
    /// # Arguments
    ///
    /// * `vice_path` - Path to the VICE binary
    /// * `port` - Remote monitor port (default: 6510)
    pub fn new(vice_path: PathBuf, port: u16) -> Self {
        Self {
            vice_path,
            port,
            child: None,
        }
    }

    /// Get the configured remote monitor port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the VICE binary path.
    pub fn vice_path(&self) -> &Path {
        &self.vice_path
    }

    /// Check if VICE process is still running.
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,     // Still running
                Ok(Some(_)) => false, // Exited
                Err(_) => false,      // Error checking, assume not running
            }
        } else {
            false
        }
    }

    /// Launch VICE with autostart for the given program.
    ///
    /// # Arguments
    ///
    /// * `prg_path` - Path to the PRG or D64 file to run
    ///
    /// # Errors
    ///
    /// Returns `RunnerError::ViceStartFailed` if VICE cannot be started.
    pub fn launch(&mut self, prg_path: &Path) -> Result<(), RunnerError> {
        // Kill existing instance if any
        self.kill()?;

        let child = Command::new(&self.vice_path)
            .arg("-autostart")
            .arg(prg_path)
            .arg("-autostartprgmode")
            .arg("1") // Inject to RAM for faster loading
            // Enable binary monitor for hot-reload (more reliable than text monitor)
            .arg("-binarymonitor")
            .arg("-binarymonitoraddress")
            .arg(format!("ip4://127.0.0.1:{}", self.port))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        self.child = Some(child);
        Ok(())
    }

    /// Kill the VICE process if it was started by us.
    pub fn kill(&mut self) -> Result<(), RunnerError> {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
        Ok(())
    }

    /// Wait for VICE to exit.
    ///
    /// # Returns
    ///
    /// Returns the exit status if VICE was running, or `None` if not.
    pub fn wait(&mut self) -> Option<std::process::ExitStatus> {
        if let Some(ref mut child) = self.child {
            child.wait().ok()
        } else {
            None
        }
    }

    /// Connect to the VICE binary monitor.
    ///
    /// Attempts to establish a TCP connection to the binary monitor port.
    /// Retries several times if the connection is refused (VICE may still be starting).
    ///
    /// # Returns
    ///
    /// Returns a `TcpStream` on success, or an error if connection fails.
    fn connect_binary_monitor(&self) -> Result<TcpStream, RunnerError> {
        let addr = format!("127.0.0.1:{}", self.port);

        // Retry connection a few times (VICE may need time to start the monitor)
        let max_retries = 10;
        let retry_delay = Duration::from_millis(500);

        for attempt in 0..max_retries {
            match TcpStream::connect_timeout(
                &addr.parse().expect("Valid socket address"),
                CONNECT_TIMEOUT,
            ) {
                Ok(stream) => {
                    // Ensure socket is in blocking mode
                    stream.set_nonblocking(false).ok();
                    // Set timeouts for read/write operations
                    stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
                    stream.set_write_timeout(Some(IO_TIMEOUT)).ok();
                    return Ok(stream);
                }
                Err(_) if attempt < max_retries - 1 => {
                    // Connection refused, wait and retry
                    std::thread::sleep(retry_delay);
                }
                Err(e) => {
                    return Err(RunnerError::ConnectionFailed {
                        port: self.port,
                        source: e,
                    });
                }
            }
        }

        Err(RunnerError::ConnectionFailed {
            port: self.port,
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection timed out after retries",
            ),
        })
    }

    /// Build a binary monitor command packet.
    ///
    /// Format: STX | API_VERSION | body_length(4) | request_id(4) | command | body
    fn build_binary_command(command: u8, body: &[u8], request_id: u32) -> Vec<u8> {
        let body_len = body.len() as u32;
        let mut packet = Vec::with_capacity(11 + body.len());

        packet.push(STX);
        packet.push(API_VERSION);
        packet.extend_from_slice(&body_len.to_le_bytes());
        packet.extend_from_slice(&request_id.to_le_bytes());
        packet.push(command);
        packet.extend_from_slice(body);

        packet
    }

    /// Send a binary monitor command and read the response.
    ///
    /// This handles the VICE binary monitor protocol which can send
    /// asynchronous events. We skip events and wait for the actual
    /// command response matching our request ID.
    fn send_binary_command(
        &self,
        stream: &mut TcpStream,
        command: u8,
        body: &[u8],
        request_id: u32,
    ) -> Result<Vec<u8>, RunnerError> {
        let packet = Self::build_binary_command(command, body, request_id);

        stream.write_all(&packet).map_err(|e| {
            RunnerError::MonitorError(format!("Failed to send binary command: {}", e))
        })?;
        stream.flush().map_err(|e| {
            RunnerError::MonitorError(format!("Failed to flush binary command: {}", e))
        })?;

        // Loop to handle async events until we get our response.
        // Response types: the command byte echoed back (e.g., 0xdd for autostart)
        // means it's a response to that command. 0x62 = async event (skip these).
        const RESPONSE_TYPE_EVENT: u8 = 0x62;
        const MAX_EVENTS: usize = 20;

        for _ in 0..MAX_EVENTS {
            // Read response header (12 bytes)
            // Format: STX(1) | API_VERSION(1) | body_length(4) | request_id(4) | response_type(1) | error_code(1)
            let mut header = [0u8; 12];
            stream.read_exact(&mut header).map_err(|e| {
                RunnerError::MonitorError(format!("Failed to read response header: {}", e))
            })?;

            // Parse header fields
            let response_len =
                u32::from_le_bytes([header[2], header[3], header[4], header[5]]) as usize;
            let response_request_id =
                u32::from_le_bytes([header[6], header[7], header[8], header[9]]);
            let response_type = header[10];
            let error_code = header[11];

            // Read response body if any
            let mut response_body = vec![0u8; response_len];
            if response_len > 0 {
                stream.read_exact(&mut response_body).map_err(|e| {
                    RunnerError::MonitorError(format!("Failed to read response body: {}", e))
                })?;
            }

            // Skip async events and responses for other requests.
            // A valid response has: response_type == command we sent, and matching request_id
            if response_type == RESPONSE_TYPE_EVENT || response_request_id != request_id {
                continue;
            }

            // Also accept if response_type matches the command we sent
            if response_type != command {
                continue;
            }

            // Check error code
            if error_code != 0 {
                return Err(RunnerError::MonitorError(format!(
                    "Binary monitor error code: 0x{:02x}",
                    error_code
                )));
            }

            return Ok(response_body);
        }

        Err(RunnerError::MonitorError(
            "Too many events without command response".to_string(),
        ))
    }

    /// Reload a program into running VICE via the binary monitor.
    ///
    /// Uses the autostart command (0xdd) which handles loading and running
    /// the program automatically.
    ///
    /// Based on: https://vice-emu.sourceforge.io/vice_13.html
    ///
    /// # Arguments
    ///
    /// * `prg_path` - Path to the PRG file to load
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if reload fails.
    pub fn reload(&self, prg_path: &Path) -> Result<(), RunnerError> {
        let mut stream = self.connect_binary_monitor()?;

        // Get absolute path for the PRG file
        let abs_path = prg_path
            .canonicalize()
            .unwrap_or_else(|_| prg_path.to_path_buf());
        let path_bytes = abs_path.to_string_lossy().as_bytes().to_vec();

        // Build autostart command body:
        // RL: 1 byte - Run after loading (1 = yes)
        // FI: 2 bytes - File index (0 for PRG files)
        // FL: 1 byte - Filename length
        // FN: FL bytes - Filename
        let mut body = Vec::new();
        body.push(0x01); // Run after loading = true
        body.extend_from_slice(&0u16.to_le_bytes()); // File index = 0
        body.push(path_bytes.len() as u8); // Filename length
        body.extend_from_slice(&path_bytes); // Filename

        // Send autostart command (request_id = 1)
        self.send_binary_command(&mut stream, CMD_AUTOSTART, &body, 1)?;

        // Send exit command to resume emulation (request_id = 2)
        self.send_binary_command(&mut stream, CMD_EXIT, &[], 2)?;

        Ok(())
    }

    /// Wait for VICE to be ready by attempting to connect to the monitor.
    ///
    /// This is useful after launching VICE to ensure it's ready for commands.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when VICE is ready, or an error on timeout.
    pub fn wait_until_ready(&self) -> Result<(), RunnerError> {
        // Try to connect - this already includes retry logic
        let _stream = self.connect_binary_monitor()?;
        Ok(())
    }
}

impl Drop for ViceRunner {
    fn drop(&mut self) {
        // Don't kill VICE on drop - let it keep running
        // User can close it manually
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vice_version_standard() {
        let output = "x64sc (VICE 3.8)";
        assert_eq!(parse_vice_version(output), Some("VICE 3.8".to_string()));
    }

    #[test]
    fn test_parse_vice_version_full_output() {
        let output = "x64sc - The Versatile Commodore Emulator, VICE version 3.8\nCopyright...";
        assert_eq!(parse_vice_version(output), Some("VICE 3.8".to_string()));
    }

    #[test]
    fn test_parse_vice_version_binary_format() {
        let output = "x64sc 3.8.0 (rev 12345)";
        assert_eq!(parse_vice_version(output), Some("x64sc 3.8.0".to_string()));
    }

    #[test]
    fn test_parse_vice_version_no_version() {
        let output = "Some random output without version";
        assert_eq!(parse_vice_version(output), None);
    }

    #[test]
    fn test_vice_runner_new() {
        let runner = ViceRunner::new(PathBuf::from("/usr/bin/x64sc"), 6510);
        assert_eq!(runner.port(), 6510);
        assert_eq!(runner.vice_path(), Path::new("/usr/bin/x64sc"));
    }

    #[test]
    fn test_vice_runner_not_running_initially() {
        let mut runner = ViceRunner::new(PathBuf::from("/usr/bin/x64sc"), 6510);
        assert!(!runner.is_running());
    }
}
