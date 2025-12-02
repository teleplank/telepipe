//! Disconnect Mode Implementation
//!
//! This module implements the `telepipe disconnect` command as specified in:
//! - 501 the telepipe core spec.md (Section 3.2: Operation E: disconnect)
//! - 560 the telepipe implementation blueprint.md (Section 4.13: disconnect.rs)
//!
//! Disconnect mode terminates a connect session's supervisor and removes the
//! session record. Unlike stop mode, disconnect does NOT kill a child process
//! (connect sessions have no child - they attach to external services).

use crate::errors::TelepipeError;
use crate::process_spawn::{is_process_running, terminate_process, kill_process};
use crate::registry::{load_session, delete_session};
use crate::session::SessionMode;

use std::thread;
use std::time::Duration;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Time to wait after SIGTERM before sending SIGKILL (2 seconds)
const SIGTERM_WAIT: Duration = Duration::from_secs(2);

/// Poll interval when waiting for process to exit (100ms)
const POLL_INTERVAL: Duration = Duration::from_millis(100);

// =============================================================================
// DISCONNECT MODE ENTRY POINT
// =============================================================================

/// Executes the disconnect operation.
///
/// This is the main entry point for `telepipe disconnect --id <ID>`.
///
/// # Steps (per spec 560)
///
/// 1. Load session entry
/// 2. Validate session is connect mode
/// 3. Kill supervisor process
/// 4. Delete session dictionary entry
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(())` - Disconnect completed successfully
/// * `Err(TelepipeError)` - Various error conditions
pub fn disconnect(id: &str) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Step 1: Load session
    let session = load_session(id)?;

    // Validate session is connect mode (disconnect is for connect sessions)
    if session.mode != SessionMode::Connect {
        return Err(TelepipeError::CliMode);
    }

    // Step 2: Kill supervisor process
    // Connect mode has no child process, only supervisor
    let supervisor_pid = session.supervisor_pid;
    graceful_kill(supervisor_pid);

    // Step 3: Delete session
    delete_session(id)?;

    Ok(())
}

/// Executes disconnect and returns an exit code.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(exit_code)` - Exit code (0 for success)
/// * `Err(TelepipeError)` - Various error conditions
pub fn disconnect_with_exit_code(id: &str) -> Result<i32, TelepipeError> {
    disconnect(id)?;
    Ok(0)
}

// =============================================================================
// PROCESS TERMINATION
// =============================================================================

/// Gracefully kills a process (SIGTERM, wait, then SIGKILL).
///
/// # Arguments
///
/// * `pid` - Process ID to kill
///
/// This function:
/// 1. Sends SIGTERM to the process
/// 2. Waits up to 2 seconds for the process to exit
/// 3. If still running, sends SIGKILL
fn graceful_kill(pid: u32) {
    // Check if process is running
    if !is_process_running(pid) {
        return;
    }

    // Send SIGTERM
    let _ = terminate_process(pid);

    // Wait for process to exit (up to SIGTERM_WAIT)
    let start = std::time::Instant::now();
    while start.elapsed() < SIGTERM_WAIT {
        if !is_process_running(pid) {
            return;
        }
        thread::sleep(POLL_INTERVAL);
    }

    // Process still running, send SIGKILL
    if is_process_running(pid) {
        let _ = kill_process(pid);
    }
}

// =============================================================================
// STATUS CHECKING
// =============================================================================

/// Checks if a connect session's supervisor is still running.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(supervisor_running)` - Whether supervisor is alive
/// * `Err(TelepipeError)` - Session not found or not connect mode
pub fn check_supervisor_status(id: &str) -> Result<bool, TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Load session
    let session = load_session(id)?;

    // Validate session is connect mode
    if session.mode != SessionMode::Connect {
        return Err(TelepipeError::CliMode);
    }

    Ok(is_process_running(session.supervisor_pid))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disconnect_empty_id() {
        let result = disconnect("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_disconnect_missing_session() {
        let result = disconnect("nonexistent-session-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_disconnect_with_exit_code_empty_id() {
        let result = disconnect_with_exit_code("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_check_supervisor_status_empty_id() {
        let result = check_supervisor_status("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_check_supervisor_status_missing_session() {
        let result = check_supervisor_status("nonexistent-session-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_sigterm_wait_constant() {
        assert_eq!(SIGTERM_WAIT, Duration::from_secs(2));
    }

    #[test]
    fn test_poll_interval_constant() {
        assert_eq!(POLL_INTERVAL, Duration::from_millis(100));
    }

    #[cfg(unix)]
    #[test]
    fn test_graceful_kill_nonexistent() {
        // Killing a non-existent process should not panic
        graceful_kill(99999999);
    }

    // Integration tests require actual sessions
    #[test]
    #[ignore] // Requires running connect session
    fn test_disconnect_connect_session() {
        // This test would:
        // 1. Create a connect session
        // 2. Call disconnect()
        // 3. Verify session is deleted
        // 4. Verify supervisor is terminated
    }
}
