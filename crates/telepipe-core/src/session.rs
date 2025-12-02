//! Session Data Structures
//!
//! This module defines the session data structures as specified in:
//! - 501 the telepipe core spec.md (Section 6: Session Dictionary)
//! - 560 the telepipe implementation blueprint.md (Section 2: Core Data Structures)
//!
//! Sessions are stored as JSON files in the session directory.

use serde::{Deserialize, Serialize};

use crate::config::{is_valid_fd, is_valid_port, DEFAULT_HOST};
use crate::errors::TelepipeError;

// =============================================================================
// SESSION MODE
// =============================================================================

/// The mode of a Telepipe session.
///
/// Per spec section 6: mode is either "redirect" or "connect".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    /// Redirect mode: spawned process with redirected stdio (3 FDs, 3 ports)
    Redirect,
    /// Connect mode: attached to external TCP service (1 FD, 1 port)
    Connect,
}

impl std::fmt::Display for SessionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionMode::Redirect => write!(f, "redirect"),
            SessionMode::Connect => write!(f, "connect"),
        }
    }
}

// =============================================================================
// SESSION ENTRY
// =============================================================================

/// A session record stored in the session dictionary.
///
/// Per spec section 6, a session record must contain:
/// - id: string (mnemonic identifier)
/// - mode: redirect|connect
/// - supervisor_pid: int
/// - child_pid: int|null (only for redirect mode)
/// - host: string
/// - stdin_port, stdout_port, stderr_port: int|null (redirect mode)
/// - stdin_fd, stdout_fd, stderr_fd: int|null (redirect mode)
/// - connect_port: int|null (connect mode)
/// - connect_fd: int|null (connect mode)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Mnemonic identifier for the session
    pub id: String,

    /// Session mode (redirect or connect)
    pub mode: SessionMode,

    /// PID of the supervisor process
    pub supervisor_pid: u32,

    /// PID of the child process (redirect mode only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_pid: Option<u32>,

    /// Host address (default: 127.0.0.1)
    pub host: String,

    // Redirect mode fields (stdin/stdout/stderr)
    /// TCP port for stdin (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_port: Option<u16>,

    /// TCP port for stdout (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_port: Option<u16>,

    /// TCP port for stderr (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_port: Option<u16>,

    /// File descriptor for stdin (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_fd: Option<u32>,

    /// File descriptor for stdout (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_fd: Option<u32>,

    /// File descriptor for stderr (redirect mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_fd: Option<u32>,

    // Connect mode fields
    /// TCP port for connect mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_port: Option<u16>,

    /// File descriptor for connect mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_fd: Option<u32>,
}

impl SessionEntry {
    /// Creates a new redirect-mode session entry.
    ///
    /// # Arguments
    ///
    /// * `id` - Mnemonic identifier
    /// * `supervisor_pid` - PID of the supervisor process
    /// * `child_pid` - PID of the child process
    /// * `ports` - Tuple of (stdin_port, stdout_port, stderr_port)
    /// * `fds` - Tuple of (stdin_fd, stdout_fd, stderr_fd)
    pub fn new_redirect(
        id: String,
        supervisor_pid: u32,
        child_pid: u32,
        ports: (u16, u16, u16),
        fds: (u32, u32, u32),
    ) -> Self {
        Self {
            id,
            mode: SessionMode::Redirect,
            supervisor_pid,
            child_pid: Some(child_pid),
            host: DEFAULT_HOST.to_string(),
            stdin_port: Some(ports.0),
            stdout_port: Some(ports.1),
            stderr_port: Some(ports.2),
            stdin_fd: Some(fds.0),
            stdout_fd: Some(fds.1),
            stderr_fd: Some(fds.2),
            connect_port: None,
            connect_fd: None,
        }
    }

    /// Creates a new connect-mode session entry.
    ///
    /// # Arguments
    ///
    /// * `id` - Mnemonic identifier
    /// * `supervisor_pid` - PID of the supervisor process
    /// * `host` - Host address of the external service
    /// * `port` - TCP port of the external service
    /// * `fd` - File descriptor for the connection
    pub fn new_connect(
        id: String,
        supervisor_pid: u32,
        host: String,
        port: u16,
        fd: u32,
    ) -> Self {
        Self {
            id,
            mode: SessionMode::Connect,
            supervisor_pid,
            child_pid: None,
            host,
            stdin_port: None,
            stdout_port: None,
            stderr_port: None,
            stdin_fd: None,
            stdout_fd: None,
            stderr_fd: None,
            connect_port: Some(port),
            connect_fd: Some(fd),
        }
    }

    /// Validates the session entry for internal consistency.
    ///
    /// Checks:
    /// - ID is not empty
    /// - Required fields present for the mode
    /// - FDs and ports are in valid ranges
    pub fn validate(&self) -> Result<(), TelepipeError> {
        // ID must not be empty
        if self.id.is_empty() {
            return Err(TelepipeError::CliId);
        }

        // Host must not be empty
        if self.host.is_empty() {
            return Err(TelepipeError::CliInvalidHost);
        }

        match self.mode {
            SessionMode::Redirect => self.validate_redirect(),
            SessionMode::Connect => self.validate_connect(),
        }
    }

    /// Validates redirect-mode specific fields.
    fn validate_redirect(&self) -> Result<(), TelepipeError> {
        // Redirect mode requires child_pid
        if self.child_pid.is_none() {
            return Err(TelepipeError::ProcNoChild);
        }

        // Redirect mode requires 3 ports
        let stdin_port = self.stdin_port.ok_or(TelepipeError::CliInvalidPort)?;
        let stdout_port = self.stdout_port.ok_or(TelepipeError::CliInvalidPort)?;
        let stderr_port = self.stderr_port.ok_or(TelepipeError::CliInvalidPort)?;

        // Validate port ranges
        if !is_valid_port(stdin_port) {
            return Err(TelepipeError::CliInvalidPort);
        }
        if !is_valid_port(stdout_port) {
            return Err(TelepipeError::CliInvalidPort);
        }
        if !is_valid_port(stderr_port) {
            return Err(TelepipeError::CliInvalidPort);
        }

        // Redirect mode requires 3 FDs
        let stdin_fd = self.stdin_fd.ok_or(TelepipeError::CliInvalidFd)?;
        let stdout_fd = self.stdout_fd.ok_or(TelepipeError::CliInvalidFd)?;
        let stderr_fd = self.stderr_fd.ok_or(TelepipeError::CliInvalidFd)?;

        // Validate FD ranges
        if !is_valid_fd(stdin_fd) {
            return Err(TelepipeError::CliInvalidFd);
        }
        if !is_valid_fd(stdout_fd) {
            return Err(TelepipeError::CliInvalidFd);
        }
        if !is_valid_fd(stderr_fd) {
            return Err(TelepipeError::CliInvalidFd);
        }

        Ok(())
    }

    /// Validates connect-mode specific fields.
    fn validate_connect(&self) -> Result<(), TelepipeError> {
        // Connect mode must not have child_pid
        if self.child_pid.is_some() {
            return Err(TelepipeError::CliMode);
        }

        // Connect mode requires connect_port
        let port = self.connect_port.ok_or(TelepipeError::CliInvalidPort)?;
        if !is_valid_port(port) {
            return Err(TelepipeError::CliInvalidPort);
        }

        // Connect mode requires connect_fd
        let fd = self.connect_fd.ok_or(TelepipeError::CliInvalidFd)?;
        if !is_valid_fd(fd) {
            return Err(TelepipeError::CliInvalidFd);
        }

        Ok(())
    }

    /// Serializes the session entry to JSON.
    pub fn to_json(&self) -> Result<String, TelepipeError> {
        serde_json::to_string_pretty(self).map_err(|_| TelepipeError::DictCorrupt)
    }

    /// Deserializes a session entry from JSON.
    pub fn from_json(json: &str) -> Result<Self, TelepipeError> {
        serde_json::from_str(json).map_err(|_| TelepipeError::DictCorrupt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mode_display() {
        assert_eq!(format!("{}", SessionMode::Redirect), "redirect");
        assert_eq!(format!("{}", SessionMode::Connect), "connect");
    }

    #[test]
    fn test_session_mode_serialize() {
        let json = serde_json::to_string(&SessionMode::Redirect).unwrap();
        assert_eq!(json, "\"redirect\"");

        let json = serde_json::to_string(&SessionMode::Connect).unwrap();
        assert_eq!(json, "\"connect\"");
    }

    #[test]
    fn test_session_mode_deserialize() {
        let mode: SessionMode = serde_json::from_str("\"redirect\"").unwrap();
        assert_eq!(mode, SessionMode::Redirect);

        let mode: SessionMode = serde_json::from_str("\"connect\"").unwrap();
        assert_eq!(mode, SessionMode::Connect);
    }

    #[test]
    fn test_new_redirect_session() {
        let entry = SessionEntry::new_redirect(
            "test-session".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        assert_eq!(entry.id, "test-session");
        assert_eq!(entry.mode, SessionMode::Redirect);
        assert_eq!(entry.supervisor_pid, 1234);
        assert_eq!(entry.child_pid, Some(5678));
        assert_eq!(entry.host, "127.0.0.1");
        assert_eq!(entry.stdin_port, Some(49152));
        assert_eq!(entry.stdout_port, Some(49153));
        assert_eq!(entry.stderr_port, Some(49154));
        assert_eq!(entry.stdin_fd, Some(3));
        assert_eq!(entry.stdout_fd, Some(4));
        assert_eq!(entry.stderr_fd, Some(5));
        assert_eq!(entry.connect_port, None);
        assert_eq!(entry.connect_fd, None);
    }

    #[test]
    fn test_new_connect_session() {
        let entry = SessionEntry::new_connect(
            "cdp-session".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        assert_eq!(entry.id, "cdp-session");
        assert_eq!(entry.mode, SessionMode::Connect);
        assert_eq!(entry.supervisor_pid, 1234);
        assert_eq!(entry.child_pid, None);
        assert_eq!(entry.host, "127.0.0.1");
        assert_eq!(entry.connect_port, Some(9222));
        assert_eq!(entry.connect_fd, Some(3));
        assert_eq!(entry.stdin_port, None);
        assert_eq!(entry.stdout_port, None);
        assert_eq!(entry.stderr_port, None);
    }

    #[test]
    fn test_redirect_session_validate_success() {
        let entry = SessionEntry::new_redirect(
            "valid".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_connect_session_validate_success() {
        let entry = SessionEntry::new_connect(
            "valid".to_string(),
            1234,
            "127.0.0.1".to_string(),
            49152,
            3,
        );
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let mut entry = SessionEntry::new_redirect(
            "".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        entry.id = "".to_string();
        assert_eq!(entry.validate().unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_validate_empty_host() {
        let mut entry = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        entry.host = "".to_string();
        assert_eq!(entry.validate().unwrap_err(), TelepipeError::CliInvalidHost);
    }

    #[test]
    fn test_validate_redirect_invalid_port() {
        let mut entry = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        // Port below ephemeral range
        entry.stdin_port = Some(80);
        assert_eq!(
            entry.validate().unwrap_err(),
            TelepipeError::CliInvalidPort
        );
    }

    #[test]
    fn test_validate_redirect_invalid_fd() {
        let mut entry = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        // FD 256 is reserved
        entry.stdin_fd = Some(256);
        assert_eq!(entry.validate().unwrap_err(), TelepipeError::CliInvalidFd);
    }

    #[test]
    fn test_validate_connect_invalid_port() {
        let mut entry = SessionEntry::new_connect(
            "test".to_string(),
            1234,
            "127.0.0.1".to_string(),
            49152,
            3,
        );
        // Port below ephemeral range
        entry.connect_port = Some(80);
        assert_eq!(
            entry.validate().unwrap_err(),
            TelepipeError::CliInvalidPort
        );
    }

    #[test]
    fn test_session_entry_json_roundtrip_redirect() {
        let original = SessionEntry::new_redirect(
            "test-roundtrip".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let json = original.to_json().unwrap();
        let restored = SessionEntry::from_json(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_session_entry_json_roundtrip_connect() {
        let original = SessionEntry::new_connect(
            "cdp-roundtrip".to_string(),
            1234,
            "localhost".to_string(),
            9222,
            3,
        );

        let json = original.to_json().unwrap();
        let restored = SessionEntry::from_json(&json).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_session_entry_json_format_redirect() {
        let entry = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let json = entry.to_json().unwrap();

        // Verify key fields are present
        assert!(json.contains("\"id\": \"test\""));
        assert!(json.contains("\"mode\": \"redirect\""));
        assert!(json.contains("\"supervisor_pid\": 1234"));
        assert!(json.contains("\"child_pid\": 5678"));
        assert!(json.contains("\"stdin_port\": 49152"));
    }

    #[test]
    fn test_session_entry_json_format_connect() {
        let entry = SessionEntry::new_connect(
            "cdp".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        let json = entry.to_json().unwrap();

        // Verify key fields are present
        assert!(json.contains("\"id\": \"cdp\""));
        assert!(json.contains("\"mode\": \"connect\""));
        assert!(json.contains("\"connect_port\": 9222"));
        assert!(json.contains("\"connect_fd\": 3"));

        // Redirect fields should NOT be present (skip_serializing_if)
        assert!(!json.contains("stdin_port"));
        assert!(!json.contains("stdout_port"));
        assert!(!json.contains("stderr_port"));
    }

    #[test]
    fn test_from_json_invalid() {
        let result = SessionEntry::from_json("not valid json");
        assert_eq!(result.unwrap_err(), TelepipeError::DictCorrupt);
    }

    #[test]
    fn test_from_json_missing_fields() {
        let json = r#"{"id": "test"}"#;
        let result = SessionEntry::from_json(json);
        assert_eq!(result.unwrap_err(), TelepipeError::DictCorrupt);
    }
}
