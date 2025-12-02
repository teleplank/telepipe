//! Telepipe Configuration Constants
//!
//! This module defines the configuration constants for Telepipe as specified in:
//! - 501 the telepipe core spec.md (Section 5: Deterministic Allocation Rules)
//! - 560 the telepipe implementation blueprint.md
//!
//! These constants are used throughout the codebase and must not be changed
//! without updating the specification.

use std::path::PathBuf;

// =============================================================================
// FD ALLOCATION CONSTANTS
// =============================================================================

/// Minimum file descriptor number for allocation.
/// FDs 0, 1, 2 are reserved for stdin, stdout, stderr.
/// Per spec section 5.2: "Scan from 3-255, sequentially."
pub const MIN_FD: u32 = 3;

/// Maximum file descriptor number for allocation.
/// Per spec section 5.2: "Scan from 3-255, sequentially."
pub const MAX_FD: u32 = 255;

/// Reserved file descriptor that must never be allocated.
/// Per spec section 5.2: "FD 256 is reserved for future use."
pub const RESERVED_FD: u32 = 256;

// =============================================================================
// PORT ALLOCATION CONSTANTS
// =============================================================================

/// Minimum TCP port number for allocation.
/// Per spec section 5.1: "Scan from 49152-65535, sequentially."
/// This is the IANA-defined ephemeral port range start.
pub const MIN_PORT: u16 = 49152;

/// Maximum TCP port number for allocation.
/// Per spec section 5.1: "Scan from 49152-65535, sequentially."
/// This is the IANA-defined ephemeral port range end.
pub const MAX_PORT: u16 = 65535;

// =============================================================================
// SESSION DIRECTORY
// =============================================================================

/// Session directory name within user's home directory.
const SESSION_DIR_NAME: &str = ".telepipe/sessions";

/// Returns the path to the Telepipe session directory.
///
/// Default location: `$HOME/.telepipe/sessions/`
///
/// This directory stores session dictionary files as JSON.
/// Each session is stored as `<id>.json`.
///
/// # Panics
///
/// Panics if the home directory cannot be determined. This is a
/// fatal condition as Telepipe cannot function without session storage.
pub fn session_dir() -> PathBuf {
    match home_dir() {
        Some(home) => home.join(SESSION_DIR_NAME),
        None => panic!("unable to determine home directory"),
    }
}

/// Returns the path to a specific session file.
///
/// # Arguments
///
/// * `id` - The session identifier
///
/// # Returns
///
/// Path to `$HOME/.telepipe/sessions/<id>.json`
pub fn session_file(id: &str) -> PathBuf {
    session_dir().join(format!("{}.json", id))
}

// =============================================================================
// DEFAULT HOST
// =============================================================================

/// Default host for TCP connections.
/// Per spec section 7: "By default, host = 127.0.0.1"
pub const DEFAULT_HOST: &str = "127.0.0.1";

// =============================================================================
// INTERNAL HELPERS
// =============================================================================

/// Cross-platform home directory detection.
/// Returns None if home directory cannot be determined.
fn home_dir() -> Option<PathBuf> {
    // Try $HOME first (works on Unix and most shells)
    if let Some(home) = std::env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }

    // On Windows, try USERPROFILE
    #[cfg(target_os = "windows")]
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return Some(PathBuf::from(profile));
    }

    None
}

// =============================================================================
// VALIDATION HELPERS
// =============================================================================

/// Checks if a file descriptor number is valid for allocation.
///
/// Returns true if the FD is within the valid range [MIN_FD, MAX_FD]
/// and is not the reserved FD.
pub fn is_valid_fd(fd: u32) -> bool {
    fd >= MIN_FD && fd <= MAX_FD && fd != RESERVED_FD
}

/// Checks if a port number is valid for allocation.
///
/// Returns true if the port is within the ephemeral range [MIN_PORT, MAX_PORT].
pub fn is_valid_port(port: u16) -> bool {
    port >= MIN_PORT && port <= MAX_PORT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fd_constants() {
        assert_eq!(MIN_FD, 3);
        assert_eq!(MAX_FD, 255);
        assert_eq!(RESERVED_FD, 256);
    }

    #[test]
    fn test_port_constants() {
        assert_eq!(MIN_PORT, 49152);
        assert_eq!(MAX_PORT, 65535);
    }

    #[test]
    fn test_fd_range_excludes_stdio() {
        // FDs 0, 1, 2 are stdin, stdout, stderr
        assert!(MIN_FD > 2, "MIN_FD must be greater than stdio FDs");
    }

    #[test]
    fn test_reserved_fd_outside_range() {
        // RESERVED_FD should be just above MAX_FD
        assert!(RESERVED_FD > MAX_FD);
    }

    #[test]
    fn test_is_valid_fd() {
        // Valid FDs
        assert!(is_valid_fd(3));
        assert!(is_valid_fd(100));
        assert!(is_valid_fd(255));

        // Invalid FDs
        assert!(!is_valid_fd(0)); // stdin
        assert!(!is_valid_fd(1)); // stdout
        assert!(!is_valid_fd(2)); // stderr
        assert!(!is_valid_fd(256)); // reserved
        assert!(!is_valid_fd(1000)); // out of range
    }

    #[test]
    fn test_is_valid_port() {
        // Valid ports
        assert!(is_valid_port(49152));
        assert!(is_valid_port(55000));
        assert!(is_valid_port(65535));

        // Invalid ports
        assert!(!is_valid_port(0));
        assert!(!is_valid_port(80)); // well-known port
        assert!(!is_valid_port(8080)); // common application port
        assert!(!is_valid_port(49151)); // just below range
    }

    #[test]
    fn test_session_dir_under_home() {
        let dir = session_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains(".telepipe"));
        assert!(dir_str.contains("sessions"));
    }

    #[test]
    fn test_session_file_format() {
        let path = session_file("my-session");
        let filename = path.file_name().unwrap().to_string_lossy();
        assert_eq!(filename, "my-session.json");
    }

    #[test]
    fn test_default_host() {
        assert_eq!(DEFAULT_HOST, "127.0.0.1");
    }

    #[test]
    fn test_ephemeral_port_range_size() {
        // IANA ephemeral range should have plenty of ports
        let range_size = (MAX_PORT - MIN_PORT + 1) as u32;
        assert!(range_size > 16000, "Ephemeral range should be substantial");
    }

    #[test]
    fn test_fd_range_size() {
        // Should have enough FDs for practical use
        let range_size = MAX_FD - MIN_FD + 1;
        assert!(range_size >= 253, "FD range should be at least 253");
    }
}
