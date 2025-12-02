//! Telepipe Error Types and Exit Codes
//!
//! This module implements the complete Telepipe Error Model as defined in:
//! - 530 the telepipe error model + error codes.md
//!
//! All errors map 1:1 to specific exit codes and error messages.
//! The error model is deterministic, machine-parseable, and stable across versions.

use std::fmt;

/// Primary error type for all Telepipe operations.
///
/// Each variant corresponds to a specific error class and exit code
/// from the 530 error model specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelepipeError {
    // E-ALLOC: Allocation failures (exit codes 21-23)
    /// E-ALLOC-FD: No available numeric file descriptors (exit 21)
    AllocFd,
    /// E-ALLOC-PORT: Unable to allocate TCP port (exit 22)
    AllocPort,
    /// E-ALLOC-DICT: Unable to write session dictionary (exit 23)
    AllocDict,

    // E-FD: File descriptor failures (exit codes 31-32)
    /// E-FD-BIND: Unable to bind numeric FD to TCP socket (exit 31)
    FdBind,
    /// E-FD-REDIRECT: Failed to redirect child stdio (exit 32)
    FdRedirect,

    // E-TCP: TCP failures (exit codes 41-44)
    /// E-TCP-CONNECT: Unable to establish TCP connection (exit 41)
    TcpConnect,
    /// E-TCP-BROKEN: TCP connection closed unexpectedly (exit 42)
    TcpBroken,
    /// E-TCP-RESET: Remote endpoint reset connection (exit 43)
    TcpReset,
    /// E-TCP-HALF: Unexpected half-close of TCP stream (exit 44)
    TcpHalf,

    // E-PROC: Process lifecycle failures (exit codes 51-54)
    /// E-PROC-SPAWN: Failed to spawn child process (exit 51)
    ProcSpawn,
    /// E-PROC-SUPERVISOR: Supervisor internal failure (exit 52)
    ProcSupervisor,
    /// E-PROC-NOCHILD: Child process not found (exit 53)
    ProcNoChild,
    /// E-PROC-EXITCODE: Unexpected child exit code (exit 54)
    ProcExitCode,

    // E-DICT: Session dictionary failures (exit codes 61-64)
    /// E-DICT-MISSING: Session not found (exit 61)
    DictMissing,
    /// E-DICT-CORRUPT: Session dictionary corrupt (exit 62)
    DictCorrupt,
    /// E-DICT-STALE: Session dictionary entry is stale (exit 63)
    DictStale,
    /// E-DICT-CONFLICT: Session ID already exists (exit 64)
    DictConflict,

    // E-INFO: Info mode failures (exit code 68)
    /// E-INFO-READ: Unable to read session info (exit 68)
    InfoRead,

    // E-EXEC: Exec mode failures (exit code 81)
    /// E-EXEC-ALREADY-ACTIVE: Another exec is connected to session (exit 81)
    /// Per 550 Concurrency Model Section 7: Only one exec connection allowed at a time per session
    ExecAlreadyActive(String),

    // E-CLI: User input/CLI failures (exit codes 71-76)
    /// E-CLI-ARGS: Invalid or missing arguments (exit 71)
    CliArgs,
    /// E-CLI-MODE: Invalid mode or incompatible flags (exit 72)
    CliMode,
    /// E-CLI-ID: Invalid session identifier (exit 73)
    CliId,
    /// E-CLI-INVALID-HOST: Invalid host or IP address (exit 74)
    CliInvalidHost,
    /// E-CLI-INVALID-FD: Invalid file descriptor (exit 75)
    CliInvalidFd,
    /// E-CLI-INVALID-PORT: Invalid TCP port (exit 76)
    CliInvalidPort,
}

impl TelepipeError {
    /// Returns the exit code for this error as defined in the 530 error model.
    ///
    /// Exit codes are deterministic and stable across versions.
    pub fn exit_code(&self) -> i32 {
        match self {
            // E-ALLOC: 21-23
            TelepipeError::AllocFd => 21,
            TelepipeError::AllocPort => 22,
            TelepipeError::AllocDict => 23,

            // E-FD: 31-32
            TelepipeError::FdBind => 31,
            TelepipeError::FdRedirect => 32,

            // E-TCP: 41-44
            TelepipeError::TcpConnect => 41,
            TelepipeError::TcpBroken => 42,
            TelepipeError::TcpReset => 43,
            TelepipeError::TcpHalf => 44,

            // E-PROC: 51-54
            TelepipeError::ProcSpawn => 51,
            TelepipeError::ProcSupervisor => 52,
            TelepipeError::ProcNoChild => 53,
            TelepipeError::ProcExitCode => 54,

            // E-DICT: 61-64
            TelepipeError::DictMissing => 61,
            TelepipeError::DictCorrupt => 62,
            TelepipeError::DictStale => 63,
            TelepipeError::DictConflict => 64,

            // E-INFO: 68
            TelepipeError::InfoRead => 68,

            // E-EXEC: 81
            TelepipeError::ExecAlreadyActive(_) => 81,

            // E-CLI: 71-76
            TelepipeError::CliArgs => 71,
            TelepipeError::CliMode => 72,
            TelepipeError::CliId => 73,
            TelepipeError::CliInvalidHost => 74,
            TelepipeError::CliInvalidFd => 75,
            TelepipeError::CliInvalidPort => 76,
        }
    }

    /// Returns the error code string (e.g., "E-ALLOC-FD")
    pub fn error_code(&self) -> &'static str {
        match self {
            TelepipeError::AllocFd => "E-ALLOC-FD",
            TelepipeError::AllocPort => "E-ALLOC-PORT",
            TelepipeError::AllocDict => "E-ALLOC-DICT",
            TelepipeError::FdBind => "E-FD-BIND",
            TelepipeError::FdRedirect => "E-FD-REDIRECT",
            TelepipeError::TcpConnect => "E-TCP-CONNECT",
            TelepipeError::TcpBroken => "E-TCP-BROKEN",
            TelepipeError::TcpReset => "E-TCP-RESET",
            TelepipeError::TcpHalf => "E-TCP-HALF",
            TelepipeError::ProcSpawn => "E-PROC-SPAWN",
            TelepipeError::ProcSupervisor => "E-PROC-SUPERVISOR",
            TelepipeError::ProcNoChild => "E-PROC-NOCHILD",
            TelepipeError::ProcExitCode => "E-PROC-EXITCODE",
            TelepipeError::DictMissing => "E-DICT-MISSING",
            TelepipeError::DictCorrupt => "E-DICT-CORRUPT",
            TelepipeError::DictStale => "E-DICT-STALE",
            TelepipeError::DictConflict => "E-DICT-CONFLICT",
            TelepipeError::InfoRead => "E-INFO-READ",
            TelepipeError::ExecAlreadyActive(_) => "E-EXEC-ALREADY-ACTIVE",
            TelepipeError::CliArgs => "E-CLI-ARGS",
            TelepipeError::CliMode => "E-CLI-MODE",
            TelepipeError::CliId => "E-CLI-ID",
            TelepipeError::CliInvalidHost => "E-CLI-INVALID-HOST",
            TelepipeError::CliInvalidFd => "E-CLI-INVALID-FD",
            TelepipeError::CliInvalidPort => "E-CLI-INVALID-PORT",
        }
    }

    /// Returns the short error description
    pub fn description(&self) -> &'static str {
        match self {
            TelepipeError::AllocFd => "no available numeric file descriptors",
            TelepipeError::AllocPort => "unable to allocate TCP port",
            TelepipeError::AllocDict => "unable to write session dictionary",
            TelepipeError::FdBind => "unable to bind numeric FD to TCP socket",
            TelepipeError::FdRedirect => "failed to redirect child stdio",
            TelepipeError::TcpConnect => "unable to establish TCP connection",
            TelepipeError::TcpBroken => "tcp connection closed unexpectedly",
            TelepipeError::TcpReset => "remote endpoint reset connection",
            TelepipeError::TcpHalf => "unexpected half-close of tcp stream",
            TelepipeError::ProcSpawn => "failed to spawn child process",
            TelepipeError::ProcSupervisor => "supervisor internal failure",
            TelepipeError::ProcNoChild => "child process not found",
            TelepipeError::ProcExitCode => "unexpected child exit code",
            TelepipeError::DictMissing => "session not found",
            TelepipeError::DictCorrupt => "session dictionary corrupt",
            TelepipeError::DictStale => "session dictionary entry is stale",
            TelepipeError::DictConflict => "session id already exists",
            TelepipeError::InfoRead => "unable to read session info",
            TelepipeError::ExecAlreadyActive(_) => "another exec is connected to session",
            TelepipeError::CliArgs => "invalid or missing arguments",
            TelepipeError::CliMode => "invalid mode or incompatible flags",
            TelepipeError::CliId => "invalid session identifier",
            TelepipeError::CliInvalidHost => "invalid host or ip address",
            TelepipeError::CliInvalidFd => "invalid file descriptor",
            TelepipeError::CliInvalidPort => "invalid tcp port",
        }
    }
}

impl fmt::Display for TelepipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ExecAlreadyActive includes the session ID in the message
            TelepipeError::ExecAlreadyActive(id) => {
                write!(f, "ERROR E-EXEC-ALREADY-ACTIVE: another exec is connected to session '{}'", id)
            }
            // All other errors use the standard format
            _ => write!(f, "ERROR {}: {}", self.error_code(), self.description())
        }
    }
}

impl std::error::Error for TelepipeError {}

/// Result type alias for Telepipe operations
pub type Result<T> = std::result::Result<T, TelepipeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_exit_codes() {
        // E-ALLOC class (21-23)
        assert_eq!(TelepipeError::AllocFd.exit_code(), 21);
        assert_eq!(TelepipeError::AllocPort.exit_code(), 22);
        assert_eq!(TelepipeError::AllocDict.exit_code(), 23);

        // E-FD class (31-32)
        assert_eq!(TelepipeError::FdBind.exit_code(), 31);
        assert_eq!(TelepipeError::FdRedirect.exit_code(), 32);

        // E-TCP class (41-44)
        assert_eq!(TelepipeError::TcpConnect.exit_code(), 41);
        assert_eq!(TelepipeError::TcpBroken.exit_code(), 42);
        assert_eq!(TelepipeError::TcpReset.exit_code(), 43);
        assert_eq!(TelepipeError::TcpHalf.exit_code(), 44);

        // E-PROC class (51-54)
        assert_eq!(TelepipeError::ProcSpawn.exit_code(), 51);
        assert_eq!(TelepipeError::ProcSupervisor.exit_code(), 52);
        assert_eq!(TelepipeError::ProcNoChild.exit_code(), 53);
        assert_eq!(TelepipeError::ProcExitCode.exit_code(), 54);

        // E-DICT class (61-64)
        assert_eq!(TelepipeError::DictMissing.exit_code(), 61);
        assert_eq!(TelepipeError::DictCorrupt.exit_code(), 62);
        assert_eq!(TelepipeError::DictStale.exit_code(), 63);
        assert_eq!(TelepipeError::DictConflict.exit_code(), 64);

        // E-INFO class (68)
        assert_eq!(TelepipeError::InfoRead.exit_code(), 68);

        // E-EXEC class (81)
        assert_eq!(TelepipeError::ExecAlreadyActive("test".to_string()).exit_code(), 81);

        // E-CLI class (71-76)
        assert_eq!(TelepipeError::CliArgs.exit_code(), 71);
        assert_eq!(TelepipeError::CliMode.exit_code(), 72);
        assert_eq!(TelepipeError::CliId.exit_code(), 73);
        assert_eq!(TelepipeError::CliInvalidHost.exit_code(), 74);
        assert_eq!(TelepipeError::CliInvalidFd.exit_code(), 75);
        assert_eq!(TelepipeError::CliInvalidPort.exit_code(), 76);
    }

    #[test]
    fn test_error_codes_unique() {
        let errors = vec![
            TelepipeError::AllocFd,
            TelepipeError::AllocPort,
            TelepipeError::AllocDict,
            TelepipeError::FdBind,
            TelepipeError::FdRedirect,
            TelepipeError::TcpConnect,
            TelepipeError::TcpBroken,
            TelepipeError::TcpReset,
            TelepipeError::TcpHalf,
            TelepipeError::ProcSpawn,
            TelepipeError::ProcSupervisor,
            TelepipeError::ProcNoChild,
            TelepipeError::ProcExitCode,
            TelepipeError::DictMissing,
            TelepipeError::DictCorrupt,
            TelepipeError::DictStale,
            TelepipeError::DictConflict,
            TelepipeError::InfoRead,
            TelepipeError::ExecAlreadyActive("test".to_string()),
            TelepipeError::CliArgs,
            TelepipeError::CliMode,
            TelepipeError::CliId,
            TelepipeError::CliInvalidHost,
            TelepipeError::CliInvalidFd,
            TelepipeError::CliInvalidPort,
        ];

        let mut codes = std::collections::HashSet::new();
        for error in &errors {
            let code = error.exit_code();
            assert!(
                codes.insert(code),
                "Duplicate exit code: {}",
                code
            );
        }
    }

    #[test]
    fn test_error_display_format() {
        let error = TelepipeError::AllocFd;
        let formatted = format!("{}", error);
        assert_eq!(formatted, "ERROR E-ALLOC-FD: no available numeric file descriptors");

        let error = TelepipeError::DictMissing;
        let formatted = format!("{}", error);
        assert_eq!(formatted, "ERROR E-DICT-MISSING: session not found");
    }

    #[test]
    fn test_error_code_strings() {
        assert_eq!(TelepipeError::AllocFd.error_code(), "E-ALLOC-FD");
        assert_eq!(TelepipeError::TcpConnect.error_code(), "E-TCP-CONNECT");
        assert_eq!(TelepipeError::DictMissing.error_code(), "E-DICT-MISSING");
        assert_eq!(TelepipeError::CliArgs.error_code(), "E-CLI-ARGS");
    }

    #[test]
    fn test_all_errors_have_descriptions() {
        let errors = vec![
            TelepipeError::AllocFd,
            TelepipeError::AllocPort,
            TelepipeError::AllocDict,
            TelepipeError::FdBind,
            TelepipeError::FdRedirect,
            TelepipeError::TcpConnect,
            TelepipeError::TcpBroken,
            TelepipeError::TcpReset,
            TelepipeError::TcpHalf,
            TelepipeError::ProcSpawn,
            TelepipeError::ProcSupervisor,
            TelepipeError::ProcNoChild,
            TelepipeError::ProcExitCode,
            TelepipeError::DictMissing,
            TelepipeError::DictCorrupt,
            TelepipeError::DictStale,
            TelepipeError::DictConflict,
            TelepipeError::InfoRead,
            TelepipeError::ExecAlreadyActive("test".to_string()),
            TelepipeError::CliArgs,
            TelepipeError::CliMode,
            TelepipeError::CliId,
            TelepipeError::CliInvalidHost,
            TelepipeError::CliInvalidFd,
            TelepipeError::CliInvalidPort,
        ];

        for error in errors {
            let desc = error.description();
            assert!(!desc.is_empty(), "Error {:?} has empty description", error);
        }
    }

    #[test]
    fn test_exec_already_active_display() {
        // Per 530 error model: message must be exactly this format
        let error = TelepipeError::ExecAlreadyActive("my-session".to_string());
        let formatted = format!("{}", error);
        assert_eq!(
            formatted,
            "ERROR E-EXEC-ALREADY-ACTIVE: another exec is connected to session 'my-session'"
        );
        assert_eq!(error.exit_code(), 81);
        assert_eq!(error.error_code(), "E-EXEC-ALREADY-ACTIVE");
    }
}
