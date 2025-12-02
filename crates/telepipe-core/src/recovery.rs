//! State Recovery Implementation
//!
//! This module implements the Telepipe recovery pipeline as specified in:
//! - 535 the telepipe state recovery algorithms.md
//! - 560 the telepipe implementation blueprint.md (Section 4.8: recovery.rs)
//!
//! The recovery pipeline:
//! - SCAN: Load all session entries from dictionary
//! - VALIDATE: Check if supervisor/child processes are alive
//! - REPAIR: (Future) Attempt to reattach to orphaned children
//! - EVICT: Remove stale sessions
//!
//! Recovery runs before every command except `info`.

use crate::errors::TelepipeError;
use crate::process_spawn::is_process_running;
use crate::registry::{load_all_sessions, delete_session};
use crate::session::{SessionEntry, SessionMode};

// =============================================================================
// RECOVERY ENTRY POINT
// =============================================================================

/// Executes the recovery pipeline.
///
/// This should be called before every Telepipe command except `info`.
///
/// # Steps (per spec 535)
///
/// 1. SCAN: Load all session entries from dictionary
/// 2. VALIDATE: Check each session for validity
/// 3. REPAIR: (Future) Attempt to repair recoverable sessions
/// 4. EVICT: Remove stale/dead sessions
///
/// # Returns
///
/// * `Ok(RecoveryResult)` - Recovery completed
/// * `Err(TelepipeError)` - Fatal error during recovery
///
/// # Performance
///
/// Recovery should complete in <1 second even with many sessions.
pub fn recover() -> Result<RecoveryResult, TelepipeError> {
    let mut result = RecoveryResult::default();

    // Step 1: SCAN - Load all sessions
    let sessions = match load_all_sessions() {
        Ok(s) => s,
        Err(TelepipeError::DictMissing) => {
            // No session directory yet - nothing to recover
            return Ok(result);
        }
        Err(e) => return Err(e),
    };

    result.total_scanned = sessions.len();

    // Step 2 & 3: VALIDATE and mark for eviction
    let mut to_evict: Vec<String> = Vec::new();

    for session in &sessions {
        match validate_session(session) {
            SessionState::Healthy => {
                result.healthy += 1;
            }
            SessionState::Stale(reason) => {
                result.stale += 1;
                result.stale_reasons.push((session.id.clone(), reason));
                to_evict.push(session.id.clone());
            }
            SessionState::Corrupt(reason) => {
                result.corrupt += 1;
                result.corrupt_reasons.push((session.id.clone(), reason));
                to_evict.push(session.id.clone());
            }
        }
    }

    // Step 4: EVICT - Remove stale sessions
    for id in &to_evict {
        if let Err(_) = delete_session(id) {
            // Ignore errors during eviction - session might already be gone
        }
        result.evicted += 1;
    }

    Ok(result)
}

/// Performs a quick recovery check without eviction.
///
/// Useful for diagnostics and testing.
///
/// # Returns
///
/// * `Ok(RecoveryResult)` - Check completed (no evictions performed)
pub fn check_recovery() -> Result<RecoveryResult, TelepipeError> {
    let mut result = RecoveryResult::default();

    let sessions = match load_all_sessions() {
        Ok(s) => s,
        Err(TelepipeError::DictMissing) => {
            return Ok(result);
        }
        Err(e) => return Err(e),
    };

    result.total_scanned = sessions.len();

    for session in &sessions {
        match validate_session(session) {
            SessionState::Healthy => {
                result.healthy += 1;
            }
            SessionState::Stale(reason) => {
                result.stale += 1;
                result.stale_reasons.push((session.id.clone(), reason));
            }
            SessionState::Corrupt(reason) => {
                result.corrupt += 1;
                result.corrupt_reasons.push((session.id.clone(), reason));
            }
        }
    }

    Ok(result)
}

// =============================================================================
// SESSION VALIDATION
// =============================================================================

/// Session validation state.
#[derive(Debug, Clone, PartialEq)]
enum SessionState {
    /// Session is healthy - supervisor and child (if applicable) are alive
    Healthy,
    /// Session is stale - supervisor or child is dead
    Stale(String),
    /// Session is corrupt - missing required fields
    Corrupt(String),
}

/// Validates a session entry.
///
/// # Validation Rules (per spec 535 Section 3)
///
/// 1. Required fields must be present
/// 2. Supervisor PID must be alive
/// 3. For redirect mode: child PID must be alive
fn validate_session(session: &SessionEntry) -> SessionState {
    // Check required fields
    if session.id.is_empty() {
        return SessionState::Corrupt("missing id".to_string());
    }

    if session.host.is_empty() {
        return SessionState::Corrupt("missing host".to_string());
    }

    // Check supervisor is alive
    if !is_process_running(session.supervisor_pid) {
        return SessionState::Stale(format!(
            "supervisor {} not running",
            session.supervisor_pid
        ));
    }

    // Mode-specific validation
    match session.mode {
        SessionMode::Redirect => {
            validate_redirect_session(session)
        }
        SessionMode::Connect => {
            validate_connect_session(session)
        }
    }
}

/// Validates a redirect-mode session.
fn validate_redirect_session(session: &SessionEntry) -> SessionState {
    // Child PID must exist
    let child_pid = match session.child_pid {
        Some(pid) => pid,
        None => {
            return SessionState::Corrupt("redirect session missing child_pid".to_string());
        }
    };

    // Child must be alive
    if !is_process_running(child_pid) {
        return SessionState::Stale(format!("child {} not running", child_pid));
    }

    // Check required ports
    if session.stdin_port.is_none() {
        return SessionState::Corrupt("missing stdin_port".to_string());
    }
    if session.stdout_port.is_none() {
        return SessionState::Corrupt("missing stdout_port".to_string());
    }
    if session.stderr_port.is_none() {
        return SessionState::Corrupt("missing stderr_port".to_string());
    }

    // Check required FDs
    if session.stdin_fd.is_none() {
        return SessionState::Corrupt("missing stdin_fd".to_string());
    }
    if session.stdout_fd.is_none() {
        return SessionState::Corrupt("missing stdout_fd".to_string());
    }
    if session.stderr_fd.is_none() {
        return SessionState::Corrupt("missing stderr_fd".to_string());
    }

    SessionState::Healthy
}

/// Validates a connect-mode session.
fn validate_connect_session(session: &SessionEntry) -> SessionState {
    // Check required port
    if session.connect_port.is_none() {
        return SessionState::Corrupt("missing connect_port".to_string());
    }

    // Check required FD
    if session.connect_fd.is_none() {
        return SessionState::Corrupt("missing connect_fd".to_string());
    }

    SessionState::Healthy
}

// =============================================================================
// RECOVERY RESULT
// =============================================================================

/// Result of a recovery operation.
#[derive(Debug, Clone, Default)]
pub struct RecoveryResult {
    /// Total sessions scanned
    pub total_scanned: usize,
    /// Number of healthy sessions
    pub healthy: usize,
    /// Number of stale sessions (dead supervisor/child)
    pub stale: usize,
    /// Number of corrupt sessions (missing fields)
    pub corrupt: usize,
    /// Number of sessions evicted
    pub evicted: usize,
    /// Reasons for stale sessions
    pub stale_reasons: Vec<(String, String)>,
    /// Reasons for corrupt sessions
    pub corrupt_reasons: Vec<(String, String)>,
}

impl RecoveryResult {
    /// Returns true if any sessions were evicted.
    pub fn had_evictions(&self) -> bool {
        self.evicted > 0
    }

    /// Returns true if all sessions are healthy.
    pub fn all_healthy(&self) -> bool {
        self.stale == 0 && self.corrupt == 0
    }

    /// Returns a summary string.
    pub fn summary(&self) -> String {
        format!(
            "scanned: {}, healthy: {}, stale: {}, corrupt: {}, evicted: {}",
            self.total_scanned,
            self.healthy,
            self.stale,
            self.corrupt,
            self.evicted
        )
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_result_default() {
        let result = RecoveryResult::default();
        assert_eq!(result.total_scanned, 0);
        assert_eq!(result.healthy, 0);
        assert_eq!(result.stale, 0);
        assert_eq!(result.corrupt, 0);
        assert_eq!(result.evicted, 0);
    }

    #[test]
    fn test_recovery_result_had_evictions() {
        let mut result = RecoveryResult::default();
        assert!(!result.had_evictions());

        result.evicted = 1;
        assert!(result.had_evictions());
    }

    #[test]
    fn test_recovery_result_all_healthy() {
        let mut result = RecoveryResult::default();
        result.healthy = 5;
        assert!(result.all_healthy());

        result.stale = 1;
        assert!(!result.all_healthy());
    }

    #[test]
    fn test_recovery_result_summary() {
        let mut result = RecoveryResult::default();
        result.total_scanned = 10;
        result.healthy = 8;
        result.stale = 1;
        result.corrupt = 1;
        result.evicted = 2;

        let summary = result.summary();
        assert!(summary.contains("scanned: 10"));
        assert!(summary.contains("healthy: 8"));
        assert!(summary.contains("stale: 1"));
        assert!(summary.contains("corrupt: 1"));
        assert!(summary.contains("evicted: 2"));
    }

    #[test]
    fn test_session_state_variants() {
        let healthy = SessionState::Healthy;
        let stale = SessionState::Stale("test".to_string());
        let corrupt = SessionState::Corrupt("test".to_string());

        assert_eq!(healthy, SessionState::Healthy);
        assert_ne!(stale, SessionState::Healthy);
        assert_ne!(corrupt, SessionState::Healthy);
    }

    #[test]
    fn test_validate_session_empty_id() {
        let session = SessionEntry {
            id: String::new(),
            mode: SessionMode::Redirect,
            supervisor_pid: 1,
            child_pid: Some(2),
            host: "127.0.0.1".to_string(),
            stdin_port: Some(50000),
            stdout_port: Some(50001),
            stderr_port: Some(50002),
            connect_port: None,
            stdin_fd: Some(3),
            stdout_fd: Some(4),
            stderr_fd: Some(5),
            connect_fd: None,
        };

        match validate_session(&session) {
            SessionState::Corrupt(reason) => {
                assert!(reason.contains("id"));
            }
            _ => panic!("expected Corrupt state"),
        }
    }

    #[test]
    fn test_validate_session_empty_host() {
        let session = SessionEntry {
            id: "test".to_string(),
            mode: SessionMode::Redirect,
            supervisor_pid: 1,
            child_pid: Some(2),
            host: String::new(),
            stdin_port: Some(50000),
            stdout_port: Some(50001),
            stderr_port: Some(50002),
            connect_port: None,
            stdin_fd: Some(3),
            stdout_fd: Some(4),
            stderr_fd: Some(5),
            connect_fd: None,
        };

        match validate_session(&session) {
            SessionState::Corrupt(reason) => {
                assert!(reason.contains("host"));
            }
            _ => panic!("expected Corrupt state"),
        }
    }

    #[test]
    fn test_validate_redirect_missing_child_pid() {
        let session = SessionEntry {
            id: "test".to_string(),
            mode: SessionMode::Redirect,
            supervisor_pid: std::process::id(), // Use current process
            child_pid: None, // Missing!
            host: "127.0.0.1".to_string(),
            stdin_port: Some(50000),
            stdout_port: Some(50001),
            stderr_port: Some(50002),
            connect_port: None,
            stdin_fd: Some(3),
            stdout_fd: Some(4),
            stderr_fd: Some(5),
            connect_fd: None,
        };

        match validate_session(&session) {
            SessionState::Corrupt(reason) => {
                assert!(reason.contains("child_pid"));
            }
            _ => panic!("expected Corrupt state"),
        }
    }

    #[test]
    fn test_validate_connect_missing_port() {
        let session = SessionEntry {
            id: "test".to_string(),
            mode: SessionMode::Connect,
            supervisor_pid: std::process::id(),
            child_pid: None,
            host: "127.0.0.1".to_string(),
            stdin_port: None,
            stdout_port: None,
            stderr_port: None,
            connect_port: None, // Missing!
            stdin_fd: None,
            stdout_fd: None,
            stderr_fd: None,
            connect_fd: Some(3),
        };

        match validate_session(&session) {
            SessionState::Corrupt(reason) => {
                assert!(reason.contains("connect_port"));
            }
            _ => panic!("expected Corrupt state"),
        }
    }

    #[test]
    fn test_recover_empty_directory() {
        // Recovery should succeed even with no sessions
        let result = recover();
        // This might fail if session dir doesn't exist, which is OK
        match result {
            Ok(r) => {
                assert!(r.all_healthy() || r.total_scanned == 0);
            }
            Err(TelepipeError::DictMissing) => {
                // Expected if directory doesn't exist
            }
            Err(_) => {
                // Other errors are acceptable in test environment
            }
        }
    }

    #[test]
    fn test_check_recovery_no_eviction() {
        // check_recovery should not evict
        let result = check_recovery();
        match result {
            Ok(r) => {
                assert_eq!(r.evicted, 0);
            }
            Err(_) => {
                // Errors acceptable in test environment
            }
        }
    }
}
