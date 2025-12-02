//! Process Spawning
//!
//! This module handles spawning child processes with redirected stdio as specified in:
//! - 501 the telepipe core spec.md (Section 2.1.4: Spawn process)
//! - 560 the telepipe implementation blueprint.md (Section 4.4: process_spawn.rs)
//!
//! The spawned child process has its stdin, stdout, and stderr redirected to
//! the provided file descriptors, which are typically connected to TCP sockets.

use std::process::{Child, Command, Stdio};

use crate::errors::TelepipeError;

// =============================================================================
// PROCESS SPAWNING (Unix)
// =============================================================================

/// Spawns a child process with redirected stdio.
///
/// The child's stdin, stdout, and stderr are redirected to the provided
/// file descriptors. These FDs should already be connected to TCP sockets.
///
/// # Arguments
///
/// * `command` - The command to execute
/// * `args` - Arguments to pass to the command
/// * `fds` - Tuple of (stdin_fd, stdout_fd, stderr_fd)
///
/// # Returns
///
/// * `Ok(pid)` - The PID of the spawned child process
/// * `Err(TelepipeError::ProcSpawn)` - Failed to spawn the process
///
/// # Platform Support
///
/// - Unix: Uses `Stdio::from_raw_fd()` for direct FD redirection
/// - Windows: Falls back to pipe-based redirection (less efficient)
#[cfg(unix)]
pub fn spawn_with_redirected_stdio(
    command: &str,
    args: &[String],
    fds: (u32, u32, u32),
) -> Result<u32, TelepipeError> {
    use std::os::unix::io::FromRawFd;

    let (stdin_fd, stdout_fd, stderr_fd) = fds;

    // SAFETY: We trust that the FDs are valid and open. The caller is responsible
    // for ensuring the FDs were properly allocated and are connected to valid
    // streams (typically TCP sockets).
    let stdin = unsafe { Stdio::from_raw_fd(stdin_fd as i32) };
    let stdout = unsafe { Stdio::from_raw_fd(stdout_fd as i32) };
    let stderr = unsafe { Stdio::from_raw_fd(stderr_fd as i32) };

    let child = Command::new(command)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .map_err(|_| TelepipeError::ProcSpawn)?;

    Ok(child.id())
}

/// Windows fallback: spawn with pipes (not direct FD redirection).
///
/// On Windows/Git Bash, we cannot directly redirect to arbitrary FDs.
/// Instead, we spawn with pipes and the caller must handle forwarding.
#[cfg(not(unix))]
pub fn spawn_with_redirected_stdio(
    command: &str,
    args: &[String],
    _fds: (u32, u32, u32),
) -> Result<u32, TelepipeError> {
    // On Windows, we spawn with piped stdio and return the child
    // The supervisor will need to forward between pipes and TCP
    let child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| TelepipeError::ProcSpawn)?;

    Ok(child.id())
}

/// Spawns a child process and returns the Child handle.
///
/// This is a lower-level function that returns the full Child struct,
/// allowing the caller to access stdin/stdout/stderr handles if needed.
///
/// # Arguments
///
/// * `command` - The command to execute
/// * `args` - Arguments to pass to the command
///
/// # Returns
///
/// * `Ok(Child)` - The spawned child process handle
/// * `Err(TelepipeError::ProcSpawn)` - Failed to spawn the process
pub fn spawn_child(command: &str, args: &[String]) -> Result<Child, TelepipeError> {
    Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| TelepipeError::ProcSpawn)
}

/// Spawns a child process with inherited stdio (for testing).
///
/// The child inherits the parent's stdin, stdout, and stderr.
pub fn spawn_inherited(command: &str, args: &[String]) -> Result<u32, TelepipeError> {
    let child = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| TelepipeError::ProcSpawn)?;

    Ok(child.id())
}

/// Spawns a child process with null stdio (silent).
///
/// The child's stdio is connected to /dev/null (or NUL on Windows).
pub fn spawn_silent(command: &str, args: &[String]) -> Result<u32, TelepipeError> {
    let child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| TelepipeError::ProcSpawn)?;

    Ok(child.id())
}

// =============================================================================
// PROCESS UTILITIES
// =============================================================================

/// Checks if a process with the given PID is still running.
///
/// # Arguments
///
/// * `pid` - The process ID to check
///
/// # Returns
///
/// `true` if the process exists and is running
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    // On Unix, we use kill(pid, 0) to check if process exists
    // SAFETY: kill with signal 0 is safe - it only checks if the process exists
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
pub fn is_process_running(_pid: u32) -> bool {
    // On non-Unix, we can't easily check without additional dependencies
    // Return true as a conservative default
    true
}

/// Sends SIGTERM to a process (Unix only).
///
/// # Arguments
///
/// * `pid` - The process ID to signal
///
/// # Returns
///
/// * `Ok(())` - Signal sent successfully
/// * `Err(TelepipeError::ProcNoChild)` - Process not found
#[cfg(unix)]
pub fn terminate_process(pid: u32) -> Result<(), TelepipeError> {
    let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
    if result == 0 {
        Ok(())
    } else {
        Err(TelepipeError::ProcNoChild)
    }
}

#[cfg(not(unix))]
pub fn terminate_process(_pid: u32) -> Result<(), TelepipeError> {
    // On non-Unix, process termination requires platform-specific APIs
    // This is a placeholder that should be implemented properly
    Err(TelepipeError::ProcNoChild)
}

/// Sends SIGKILL to a process (Unix only).
///
/// This is a forceful termination that cannot be caught or ignored.
///
/// # Arguments
///
/// * `pid` - The process ID to kill
///
/// # Returns
///
/// * `Ok(())` - Signal sent successfully
/// * `Err(TelepipeError::ProcNoChild)` - Process not found
#[cfg(unix)]
pub fn kill_process(pid: u32) -> Result<(), TelepipeError> {
    let result = unsafe { libc::kill(pid as i32, libc::SIGKILL) };
    if result == 0 {
        Ok(())
    } else {
        Err(TelepipeError::ProcNoChild)
    }
}

#[cfg(not(unix))]
pub fn kill_process(_pid: u32) -> Result<(), TelepipeError> {
    Err(TelepipeError::ProcNoChild)
}

/// Waits for a process to exit and returns its exit code.
///
/// # Arguments
///
/// * `pid` - The process ID to wait for
///
/// # Returns
///
/// * `Ok(exit_code)` - The process exit code
/// * `Err(TelepipeError::ProcNoChild)` - Process not found or wait failed
#[cfg(unix)]
pub fn wait_for_process(pid: u32) -> Result<i32, TelepipeError> {
    let mut status: libc::c_int = 0;
    let result = unsafe { libc::waitpid(pid as i32, &mut status, 0) };

    if result == -1 {
        return Err(TelepipeError::ProcNoChild);
    }

    // Extract exit code from status
    if libc::WIFEXITED(status) {
        Ok(libc::WEXITSTATUS(status))
    } else if libc::WIFSIGNALED(status) {
        // Process was killed by signal, return 128 + signal number
        Ok(128 + libc::WTERMSIG(status))
    } else {
        Err(TelepipeError::ProcExitCode)
    }
}

#[cfg(not(unix))]
pub fn wait_for_process(_pid: u32) -> Result<i32, TelepipeError> {
    Err(TelepipeError::ProcNoChild)
}

/// Gets the current process ID.
pub fn current_pid() -> u32 {
    std::process::id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_pid() {
        let pid = current_pid();
        assert!(pid > 0);
    }

    #[test]
    fn test_spawn_child_echo() {
        // Spawn a simple echo command
        let result = spawn_child("echo", &["hello".to_string()]);
        assert!(result.is_ok());

        let mut child = result.unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_spawn_child_invalid_command() {
        let result = spawn_child("nonexistent_command_xyz", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::ProcSpawn);
    }

    #[test]
    fn test_spawn_silent() {
        let result = spawn_silent("echo", &["silent".to_string()]);
        assert!(result.is_ok());

        let pid = result.unwrap();
        assert!(pid > 0);

        // Wait a bit for the process to complete
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running_self() {
        let pid = current_pid();
        assert!(is_process_running(pid));
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running_invalid() {
        // PID 99999999 is unlikely to exist
        assert!(!is_process_running(99999999));
    }

    #[cfg(unix)]
    #[test]
    fn test_spawn_and_terminate() {
        // Spawn a sleep process
        let result = spawn_silent("sleep", &["10".to_string()]);
        assert!(result.is_ok());

        let pid = result.unwrap();
        assert!(is_process_running(pid));

        // Kill it forcefully (SIGKILL, not SIGTERM)
        let kill_result = kill_process(pid);
        assert!(kill_result.is_ok());

        // Wait for the process to be reaped
        let _ = wait_for_process(pid);

        // Should no longer be running
        assert!(!is_process_running(pid));
    }

    #[cfg(unix)]
    #[test]
    fn test_terminate_nonexistent() {
        let result = terminate_process(99999999);
        assert!(result.is_err());
    }

    #[test]
    fn test_spawn_with_args() {
        // Test spawning with multiple arguments
        let result = spawn_child("echo", &["arg1".to_string(), "arg2".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spawn_empty_args() {
        // Test spawning with no arguments
        let result = spawn_child("true", &[]);
        assert!(result.is_ok());

        let mut child = result.unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }
}
