//! Info Mode Implementation
//!
//! This module implements the `telepipe info` command as specified in:
//! - 501 the telepipe core spec.md (Section 3.3: Operation F: info)
//! - 560 the telepipe implementation blueprint.md (Section 4.14: info.rs)
//!
//! Info mode prints session state in tab-separated format and exits immediately.
//! It is read-only and must complete in <100ms.

use std::io::{self, Write};

use crate::errors::TelepipeError;
use crate::registry::load_session;
use crate::session::{SessionEntry, SessionMode};

// =============================================================================
// INFO MODE ENTRY POINT
// =============================================================================

/// Executes the info operation.
///
/// This is the main entry point for `telepipe info --id <ID>`.
///
/// # Steps (per spec 560)
///
/// 1. Load session entry (read-only, no global lock needed)
/// 2. Format fields as tab-separated output
/// 3. Print to stdout
/// 4. Exit immediately with code 0
///
/// **Performance requirement:** Must complete in <100ms.
///
/// **Critical guarantee:** Info mode exits immediately after printing.
/// Never blocks on supervisor, child process, or network I/O.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(())` - Info printed successfully
/// * `Err(TelepipeError)` - Session not found or other error
pub fn info(id: &str) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Step 1: Load session (read-only)
    let session = load_session(id)?;

    // Step 2-3: Format and print based on mode
    let output = format_session_info(&session)?;

    // Print to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(output.as_bytes())
        .map_err(|_| TelepipeError::InfoRead)?;
    handle.write_all(b"\n").map_err(|_| TelepipeError::InfoRead)?;
    handle.flush().map_err(|_| TelepipeError::InfoRead)?;

    // Step 4: Exit with code 0 (implicit via Ok)
    Ok(())
}

/// Executes info and returns the output as a string (for testing).
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(String)` - Formatted info output
/// * `Err(TelepipeError)` - Session not found or other error
pub fn info_to_string(id: &str) -> Result<String, TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Load session
    let session = load_session(id)?;

    // Format and return
    format_session_info(&session)
}

// =============================================================================
// OUTPUT FORMATTING
// =============================================================================

/// Formats session info as tab-separated output.
///
/// Output format depends on session mode:
/// - REDIRECT mode: 10 fields (mode, host, pid, supervisor, stdin, stdout, stderr, fdin, fdout, fderr)
/// - CONNECT mode: 6 fields (mode, host, port, supervisor, connected, fd)
///
/// # Arguments
///
/// * `session` - Session entry to format
///
/// # Returns
///
/// * `Ok(String)` - Tab-separated output
/// * `Err(TelepipeError)` - Missing required fields
fn format_session_info(session: &SessionEntry) -> Result<String, TelepipeError> {
    match session.mode {
        SessionMode::Redirect => format_redirect_info(session),
        SessionMode::Connect => format_connect_info(session),
    }
}

/// Formats redirect-mode session info.
///
/// Output format (per spec):
/// ```text
/// mode        redirect
/// host        127.0.0.1
/// pid         <child-pid>
/// supervisor  <supervisor-pid>
/// stdin       <port>
/// stdout      <port>
/// stderr      <port>
/// fdin        <fd-number>
/// fdout       <fd-number>
/// fderr       <fd-number>
/// ```
fn format_redirect_info(session: &SessionEntry) -> Result<String, TelepipeError> {
    let child_pid = session.child_pid.ok_or(TelepipeError::ProcNoChild)?;
    let stdin_port = session.stdin_port.ok_or(TelepipeError::CliInvalidPort)?;
    let stdout_port = session.stdout_port.ok_or(TelepipeError::CliInvalidPort)?;
    let stderr_port = session.stderr_port.ok_or(TelepipeError::CliInvalidPort)?;
    let stdin_fd = session.stdin_fd.ok_or(TelepipeError::CliInvalidFd)?;
    let stdout_fd = session.stdout_fd.ok_or(TelepipeError::CliInvalidFd)?;
    let stderr_fd = session.stderr_fd.ok_or(TelepipeError::CliInvalidFd)?;

    Ok(format!(
        "mode\tredirect\n\
         host\t{}\n\
         pid\t{}\n\
         supervisor\t{}\n\
         stdin\t{}\n\
         stdout\t{}\n\
         stderr\t{}\n\
         fdin\t{}\n\
         fdout\t{}\n\
         fderr\t{}",
        session.host,
        child_pid,
        session.supervisor_pid,
        stdin_port,
        stdout_port,
        stderr_port,
        stdin_fd,
        stdout_fd,
        stderr_fd,
    ))
}

/// Formats connect-mode session info.
///
/// Output format (per spec):
/// ```text
/// mode        connect
/// host        <host>
/// port        <protocol-port>
/// supervisor  <supervisor-pid>
/// connected   yes|no
/// fd          <fd-number>
/// ```
///
/// Note: "connected" and "reconnect" fields require runtime state checking.
/// For simplicity, we assume "yes" and "enabled" as defaults.
fn format_connect_info(session: &SessionEntry) -> Result<String, TelepipeError> {
    let connect_port = session.connect_port.ok_or(TelepipeError::CliInvalidPort)?;
    let connect_fd = session.connect_fd.ok_or(TelepipeError::CliInvalidFd)?;

    // Note: "connected" would require checking actual TCP state
    // For now, we assume connected if session exists
    let connected = "yes";

    Ok(format!(
        "mode\tconnect\n\
         host\t{}\n\
         port\t{}\n\
         supervisor\t{}\n\
         connected\t{}\n\
         fd\t{}",
        session.host,
        connect_port,
        session.supervisor_pid,
        connected,
        connect_fd,
    ))
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Parses info output to extract a specific field value.
///
/// Useful for scripts that need to extract values from info output.
///
/// # Arguments
///
/// * `output` - Tab-separated info output
/// * `field` - Field name to extract
///
/// # Returns
///
/// * `Some(value)` - Field value if found
/// * `None` - Field not found
pub fn parse_field(output: &str, field: &str) -> Option<String> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 && parts[0] == field {
            return Some(parts[1].to_string());
        }
    }
    None
}

/// Returns all field names for a given mode.
pub fn field_names(mode: SessionMode) -> Vec<&'static str> {
    match mode {
        SessionMode::Redirect => vec![
            "mode", "host", "pid", "supervisor", "stdin", "stdout", "stderr", "fdin", "fdout",
            "fderr",
        ],
        SessionMode::Connect => vec!["mode", "host", "port", "supervisor", "connected", "fd"],
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_empty_id() {
        let result = info("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_info_missing_session() {
        let result = info("nonexistent-session-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_format_redirect_info() {
        let session = SessionEntry::new_redirect(
            "test-redirect".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let result = format_session_info(&session);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("mode\tredirect"));
        assert!(output.contains("host\t127.0.0.1"));
        assert!(output.contains("pid\t5678"));
        assert!(output.contains("supervisor\t1234"));
        assert!(output.contains("stdin\t49152"));
        assert!(output.contains("stdout\t49153"));
        assert!(output.contains("stderr\t49154"));
        assert!(output.contains("fdin\t3"));
        assert!(output.contains("fdout\t4"));
        assert!(output.contains("fderr\t5"));
    }

    #[test]
    fn test_format_connect_info() {
        let session = SessionEntry::new_connect(
            "test-connect".to_string(),
            1234,
            "localhost".to_string(),
            9222,
            3,
        );

        let result = format_session_info(&session);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("mode\tconnect"));
        assert!(output.contains("host\tlocalhost"));
        assert!(output.contains("port\t9222"));
        assert!(output.contains("supervisor\t1234"));
        assert!(output.contains("connected\tyes"));
        assert!(output.contains("fd\t3"));
    }

    #[test]
    fn test_parse_field_found() {
        let output = "mode\tredirect\nhost\t127.0.0.1\npid\t1234";

        assert_eq!(parse_field(output, "mode"), Some("redirect".to_string()));
        assert_eq!(parse_field(output, "host"), Some("127.0.0.1".to_string()));
        assert_eq!(parse_field(output, "pid"), Some("1234".to_string()));
    }

    #[test]
    fn test_parse_field_not_found() {
        let output = "mode\tredirect\nhost\t127.0.0.1";

        assert_eq!(parse_field(output, "nonexistent"), None);
    }

    #[test]
    fn test_field_names_redirect() {
        let names = field_names(SessionMode::Redirect);
        assert_eq!(names.len(), 10);
        assert!(names.contains(&"mode"));
        assert!(names.contains(&"pid"));
        assert!(names.contains(&"stdin"));
        assert!(names.contains(&"fdin"));
    }

    #[test]
    fn test_field_names_connect() {
        let names = field_names(SessionMode::Connect);
        assert_eq!(names.len(), 6);
        assert!(names.contains(&"mode"));
        assert!(names.contains(&"port"));
        assert!(names.contains(&"connected"));
        assert!(names.contains(&"fd"));
    }

    #[test]
    fn test_redirect_output_has_tabs() {
        let session = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let output = format_session_info(&session).unwrap();

        // Every line should have exactly one tab
        for line in output.lines() {
            let tab_count = line.matches('\t').count();
            assert_eq!(tab_count, 1, "Line '{}' should have exactly 1 tab", line);
        }
    }

    #[test]
    fn test_connect_output_has_tabs() {
        let session = SessionEntry::new_connect(
            "test".to_string(),
            1234,
            "localhost".to_string(),
            9222,
            3,
        );

        let output = format_session_info(&session).unwrap();

        // Every line should have exactly one tab
        for line in output.lines() {
            let tab_count = line.matches('\t').count();
            assert_eq!(tab_count, 1, "Line '{}' should have exactly 1 tab", line);
        }
    }

    #[test]
    fn test_redirect_output_line_count() {
        let session = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let output = format_session_info(&session).unwrap();
        let line_count = output.lines().count();

        // Redirect mode should have 10 lines
        assert_eq!(line_count, 10);
    }

    #[test]
    fn test_connect_output_line_count() {
        let session = SessionEntry::new_connect(
            "test".to_string(),
            1234,
            "localhost".to_string(),
            9222,
            3,
        );

        let output = format_session_info(&session).unwrap();
        let line_count = output.lines().count();

        // Connect mode should have 6 lines
        assert_eq!(line_count, 6);
    }
}
