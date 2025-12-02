//! Stop Mode Implementation
//!
//! This module implements the `telepipe stop` command as specified in:
//! - 501 the telepipe core spec.md (Section 3.1: Operation D: stop)
//! - 560 the telepipe implementation blueprint.md (Section 4.12: stop.rs)
//!
//! Stop mode terminates a redirect session by killing the child process
//! and supervisor, then removing the session record.

use std::thread;
use std::time::Duration;

use crate::errors::TelepipeError;
use crate::process_spawn::{is_process_running, terminate_process, kill_process};
use crate::registry::{load_session, delete_session};
use crate::session::SessionMode;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Time to wait after SIGTERM before sending SIGKILL (2 seconds)
const SIGTERM_WAIT: Duration = Duration::from_secs(2);

/// Poll interval when waiting for process to exit (100ms)
const POLL_INTERVAL: Duration = Duration::from_millis(100);

// =============================================================================
// STOP MODE ENTRY POINT
// =============================================================================

/// Executes the stop operation.
///
/// This is the main entry point for `telepipe stop --id <ID>`.
///
/// # Steps (per spec 560)
///
/// 1. Load session entry
/// 2. Kill child process (SIGTERM, wait 2 seconds, then SIGKILL if needed)
/// 3. Kill supervisor process
/// 4. Delete session dictionary entry
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(())` - Stop completed successfully
/// * `Err(TelepipeError)` - Various error conditions
pub fn stop(id: &str) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Step 1: Load session
    let session = load_session(id)?;

    // Validate session is redirect mode (stop is for redirect sessions)
    if session.mode != SessionMode::Redirect {
        return Err(TelepipeError::CliMode);
    }

    // Step 2: Kill child process
    if let Some(child_pid) = session.child_pid {
        graceful_kill(child_pid);
    }

    // Step 3: Kill supervisor process
    let supervisor_pid = session.supervisor_pid;
    graceful_kill(supervisor_pid);

    // Step 4: Delete session
    delete_session(id)?;

    Ok(())
}

/// Executes stop and returns the child's exit code if available.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(exit_code)` - Exit code (0 for success, child's exit code if available)
/// * `Err(TelepipeError)` - Various error conditions
pub fn stop_with_exit_code(id: &str) -> Result<i32, TelepipeError> {
    stop(id)?;
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

/// Forcefully kills a process immediately (SIGKILL only).
///
/// # Arguments
///
/// * `pid` - Process ID to kill
pub fn force_kill(pid: u32) {
    if is_process_running(pid) {
        let _ = kill_process(pid);
    }
}

/// Kills only the child process without affecting the supervisor.
///
/// Useful when you want to stop the child but keep the session.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(())` - Child killed successfully
/// * `Err(TelepipeError)` - Session not found or not redirect mode
pub fn kill_child(id: &str) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Load session
    let session = load_session(id)?;

    // Validate session is redirect mode
    if session.mode != SessionMode::Redirect {
        return Err(TelepipeError::CliMode);
    }

    // Kill child
    if let Some(child_pid) = session.child_pid {
        graceful_kill(child_pid);
    }

    Ok(())
}

// =============================================================================
// STATUS CHECKING
// =============================================================================

/// Checks if a session's processes are still running.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok((child_running, supervisor_running))` - Process states
/// * `Err(TelepipeError)` - Session not found
pub fn check_status(id: &str) -> Result<(bool, bool), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Load session
    let session = load_session(id)?;

    let child_running = session
        .child_pid
        .map(is_process_running)
        .unwrap_or(false);

    let supervisor_running = is_process_running(session.supervisor_pid);

    Ok((child_running, supervisor_running))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_empty_id() {
        let result = stop("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_stop_missing_session() {
        let result = stop("nonexistent-session-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_kill_child_empty_id() {
        let result = kill_child("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_kill_child_missing_session() {
        let result = kill_child("nonexistent-session-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_check_status_empty_id() {
        let result = check_status("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_check_status_missing_session() {
        let result = check_status("nonexistent-session-xyz");
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

    #[cfg(unix)]
    #[test]
    fn test_force_kill_nonexistent() {
        // Force killing a non-existent process should not panic
        force_kill(99999999);
    }

    // Integration tests require actual sessions
    #[test]
    #[ignore] // Requires running redirect session
    fn test_stop_redirect_session() {
        // This test would:
        // 1. Create a redirect session
        // 2. Call stop()
        // 3. Verify session is deleted
        // 4. Verify processes are terminated
    }
}
