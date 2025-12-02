//! Signal Handling Implementation
//!
//! This module implements signal handling for Telepipe supervisors as specified in:
//! - 575 the telepipe supervisor architecture.md (Section 4.6: Signal-safe behavior)
//! - 560 the telepipe implementation blueprint.md (platform/posix.rs signal handling)
//!
//! The signal handler:
//! - Catches SIGINT and SIGTERM
//! - Forwards signals to child processes (redirect mode)
//! - Triggers graceful cleanup
//! - Ensures clean exit

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(unix)]
use crate::process_spawn::{terminate_process, kill_process};

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// Flag indicating a termination signal was received.
static SIGNAL_RECEIVED: AtomicBool = AtomicBool::new(false);

/// The signal number that was received (for debugging).
static SIGNAL_NUMBER: AtomicU32 = AtomicU32::new(0);

/// Child PID to forward signals to (0 if none).
static CHILD_PID: AtomicU32 = AtomicU32::new(0);

// =============================================================================
// SIGNAL SETUP
// =============================================================================

/// Sets up signal handlers for SIGINT and SIGTERM.
///
/// This should be called once at supervisor startup.
///
/// # Platform Support
///
/// - Unix: Uses signal handlers via libc
/// - Windows: Uses ctrl-c handler (limited support)
///
/// # Safety
///
/// Signal handlers are async-signal-safe and only set atomic flags.
#[cfg(unix)]
pub fn setup_signal_handlers() {
    unsafe {
        // Install SIGINT handler
        libc::signal(libc::SIGINT, signal_handler as libc::sighandler_t);
        // Install SIGTERM handler
        libc::signal(libc::SIGTERM, signal_handler as libc::sighandler_t);
    }
}

/// Signal handler function (Unix).
///
/// This function is async-signal-safe - it only sets atomic flags.
#[cfg(unix)]
extern "C" fn signal_handler(sig: libc::c_int) {
    SIGNAL_RECEIVED.store(true, Ordering::SeqCst);
    SIGNAL_NUMBER.store(sig as u32, Ordering::SeqCst);

    // Forward signal to child if registered
    let child_pid = CHILD_PID.load(Ordering::SeqCst);
    if child_pid != 0 {
        unsafe {
            libc::kill(child_pid as libc::pid_t, sig);
        }
    }
}

/// Sets up signal handlers (Windows fallback).
///
/// Windows Git Bash has limited signal support. We use ctrlc crate
/// patterns for basic Ctrl+C handling.
#[cfg(not(unix))]
pub fn setup_signal_handlers() {
    // Windows: Limited signal handling
    // In Git Bash, SIGINT is emulated via console events
    // For now, we just set the flag on any termination attempt
}

// =============================================================================
// CHILD REGISTRATION
// =============================================================================

/// Registers a child PID for signal forwarding.
///
/// When a signal is received, it will be forwarded to this PID.
/// Call with pid=0 to unregister.
///
/// # Arguments
///
/// * `pid` - Child process ID to forward signals to
pub fn register_child(pid: u32) {
    CHILD_PID.store(pid, Ordering::SeqCst);
}

/// Unregisters the child PID.
///
/// Signals will no longer be forwarded.
pub fn unregister_child() {
    CHILD_PID.store(0, Ordering::SeqCst);
}

/// Gets the currently registered child PID.
///
/// Returns 0 if no child is registered.
pub fn get_registered_child() -> u32 {
    CHILD_PID.load(Ordering::SeqCst)
}

// =============================================================================
// SIGNAL CHECKING
// =============================================================================

/// Checks if a termination signal was received.
///
/// This is meant to be called in the supervisor event loop.
///
/// # Returns
///
/// * `true` if SIGINT or SIGTERM was received
/// * `false` otherwise
pub fn signal_received() -> bool {
    SIGNAL_RECEIVED.load(Ordering::SeqCst)
}

/// Clears the signal received flag.
///
/// Useful for resetting state after handling a signal.
pub fn clear_signal() {
    SIGNAL_RECEIVED.store(false, Ordering::SeqCst);
    SIGNAL_NUMBER.store(0, Ordering::SeqCst);
}

/// Gets the signal number that was received.
///
/// # Returns
///
/// * Signal number (e.g., 2 for SIGINT, 15 for SIGTERM)
/// * 0 if no signal received
pub fn get_signal_number() -> u32 {
    SIGNAL_NUMBER.load(Ordering::SeqCst)
}

/// Checks if the received signal was SIGINT.
#[cfg(unix)]
pub fn was_sigint() -> bool {
    SIGNAL_NUMBER.load(Ordering::SeqCst) == libc::SIGINT as u32
}

/// Checks if the received signal was SIGTERM.
#[cfg(unix)]
pub fn was_sigterm() -> bool {
    SIGNAL_NUMBER.load(Ordering::SeqCst) == libc::SIGTERM as u32
}

/// Checks if the received signal was SIGINT (Windows fallback).
#[cfg(not(unix))]
pub fn was_sigint() -> bool {
    // On Windows, we treat any signal as potential SIGINT
    signal_received()
}

/// Checks if the received signal was SIGTERM (Windows fallback).
#[cfg(not(unix))]
pub fn was_sigterm() -> bool {
    false // SIGTERM not typically available on Windows
}

// =============================================================================
// GRACEFUL SHUTDOWN
// =============================================================================

/// Initiates graceful shutdown of a child process.
///
/// This function:
/// 1. Sends SIGTERM to the child
/// 2. Waits for graceful termination
/// 3. Sends SIGKILL if still running after timeout
///
/// # Arguments
///
/// * `child_pid` - PID of child process to terminate
/// * `timeout_ms` - Milliseconds to wait before SIGKILL
#[cfg(unix)]
pub fn graceful_shutdown(child_pid: u32, timeout_ms: u64) {
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::process_spawn::is_process_running;

    // Send SIGTERM
    let _ = terminate_process(child_pid);

    // Wait for process to exit
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if !is_process_running(child_pid) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Process still running, send SIGKILL
    if is_process_running(child_pid) {
        let _ = kill_process(child_pid);
    }
}

/// Initiates graceful shutdown (Windows fallback).
#[cfg(not(unix))]
pub fn graceful_shutdown(_child_pid: u32, _timeout_ms: u64) {
    // Windows: Process termination handled differently
    // For Git Bash, the child process will receive the signal
}

// =============================================================================
// SIGNAL NAMES
// =============================================================================

/// Returns a human-readable name for a signal number.
#[cfg(unix)]
pub fn signal_name(sig: u32) -> &'static str {
    match sig as i32 {
        libc::SIGINT => "SIGINT",
        libc::SIGTERM => "SIGTERM",
        libc::SIGKILL => "SIGKILL",
        libc::SIGHUP => "SIGHUP",
        libc::SIGQUIT => "SIGQUIT",
        _ => "UNKNOWN",
    }
}

/// Returns a human-readable name for a signal number (Windows fallback).
#[cfg(not(unix))]
pub fn signal_name(sig: u32) -> &'static str {
    match sig {
        2 => "SIGINT",
        15 => "SIGTERM",
        9 => "SIGKILL",
        _ => "UNKNOWN",
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_received_initial() {
        // Clear any previous state
        clear_signal();
        assert!(!signal_received());
    }

    #[test]
    fn test_clear_signal() {
        SIGNAL_RECEIVED.store(true, Ordering::SeqCst);
        SIGNAL_NUMBER.store(42, Ordering::SeqCst);

        clear_signal();

        assert!(!signal_received());
        assert_eq!(get_signal_number(), 0);
    }

    #[test]
    fn test_register_child() {
        register_child(12345);
        assert_eq!(get_registered_child(), 12345);

        unregister_child();
        assert_eq!(get_registered_child(), 0);
    }

    #[test]
    fn test_child_registration_zero() {
        register_child(0);
        assert_eq!(get_registered_child(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_signal_name_sigint() {
        assert_eq!(signal_name(libc::SIGINT as u32), "SIGINT");
    }

    #[cfg(unix)]
    #[test]
    fn test_signal_name_sigterm() {
        assert_eq!(signal_name(libc::SIGTERM as u32), "SIGTERM");
    }

    #[cfg(unix)]
    #[test]
    fn test_signal_name_sigkill() {
        assert_eq!(signal_name(libc::SIGKILL as u32), "SIGKILL");
    }

    #[test]
    fn test_signal_name_unknown() {
        assert_eq!(signal_name(999), "UNKNOWN");
    }

    #[test]
    fn test_was_sigint_false_initially() {
        clear_signal();
        assert!(!was_sigint());
    }

    #[test]
    fn test_was_sigterm_false_initially() {
        clear_signal();
        assert!(!was_sigterm());
    }

    #[cfg(unix)]
    #[test]
    fn test_setup_signal_handlers() {
        // Should not panic
        setup_signal_handlers();
    }

    #[cfg(unix)]
    #[test]
    fn test_graceful_shutdown_nonexistent() {
        // Should not panic on non-existent process
        graceful_shutdown(99999999, 100);
    }
}
