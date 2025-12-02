//! Redirect Mode Implementation
//!
//! This module implements the `telepipe redirect` command as specified in:
//! - 501 the telepipe core spec.md (Section 2.1: Operation A: redirect)
//! - 560 the telepipe implementation blueprint.md (Section 4.9: redirect_mode.rs)
//! - 575 the telepipe supervisor architecture.md (Section 7: Detailed I/O Flow)
//!
//! Redirect mode spawns a child process with piped stdio, then runs a supervisor
//! that accepts TCP connections and forwards data bidirectionally between
//! TCP sockets and the child's stdin/stdout/stderr.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, ChildStdin, ChildStdout, ChildStderr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::errors::TelepipeError;
use crate::fds::allocate_three_fds;
use crate::process_spawn::{current_pid, spawn_child};
use crate::registry::{session_exists, write_session, delete_session};
use crate::session::{SessionEntry, SessionMode};
use crate::tcp_alloc::allocate_three_ports_with_listeners;

// =============================================================================
// ONE-AT-A-TIME EXEC MODEL
// Per 550 Concurrency Model Section 7: Only ONE exec connection allowed at a time
// =============================================================================

/// Tracks the number of active exec connections per port.
/// Per spec: stdin must have 0 active connections to accept a new exec.
/// We use AtomicU32 to count active connections on the stdin port (the gating port).
/// When stdin has an active connection, other exec attempts are rejected.
#[derive(Debug)]
struct ExecConnectionTracker {
    /// Count of active stdin connections (0 or 1)
    stdin_active: AtomicBool,
}

impl ExecConnectionTracker {
    fn new() -> Self {
        Self {
            stdin_active: AtomicBool::new(false),
        }
    }

    /// Attempts to acquire the exec lock. Returns true if acquired, false if busy.
    fn try_acquire(&self) -> bool {
        // Compare-and-swap: only succeed if currently false
        self.stdin_active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Releases the exec lock.
    fn release(&self) {
        self.stdin_active.store(false, Ordering::SeqCst);
    }
}

// =============================================================================
// REDIRECT MODE ENTRY POINT
// =============================================================================

/// Executes the redirect operation.
///
/// This is the main entry point for `telepipe redirect --id <ID> -- <cmd> [args...]`.
///
/// # Steps (per spec 560 and 575)
///
/// 1. Check if session ID already exists (return DictConflict if so)
/// 2. Allocate 3 FDs (for session record)
/// 3. Allocate 3 ports and bind TCP listeners
/// 4. Spawn child process with piped stdio
/// 5. Write session dictionary
/// 6. Spawn supervisor thread that:
///    - Accepts TCP connections on all 3 ports
///    - Forwards data bidirectionally (per 575 Section 7.1)
/// 7. Return success (supervisor continues running)
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
/// * `command` - Command to execute
/// * `args` - Arguments for the command
///
/// # Returns
///
/// * `Ok(())` - Redirect started successfully
/// * `Err(TelepipeError)` - Various error conditions
pub fn redirect(id: &str, command: &str, args: &[String]) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Check for existing session with same ID
    if session_exists(id) {
        return Err(TelepipeError::DictConflict);
    }

    // Step 1: Allocate FDs (for session record, not actual redirection)
    let (stdin_fd, stdout_fd, stderr_fd) = allocate_three_fds()?;

    // Step 2: Allocate ports and bind TCP listeners
    let (stdin_listener, stdout_listener, stderr_listener) =
        allocate_three_ports_with_listeners()?;

    let stdin_port = listener_port(&stdin_listener)?;
    let stdout_port = listener_port(&stdout_listener)?;
    let stderr_port = listener_port(&stderr_listener)?;

    // Step 3: Spawn child process with PIPED stdio (not raw FDs)
    // Per 575 Section 7.1: supervisor reads/writes to child via pipes
    let child = spawn_child(command, args)?;
    let child_pid = child.id();

    // Step 4: Get supervisor PID (current process)
    let supervisor_pid = current_pid();

    // Step 5: Create and write session entry
    let session = SessionEntry::new_redirect(
        id.to_string(),
        supervisor_pid,
        child_pid,
        (stdin_port, stdout_port, stderr_port),
        (stdin_fd, stdout_fd, stderr_fd),
    );

    write_session(&session)?;

    // Step 6: Run supervisor in foreground
    // The supervisor accepts TCP connections and forwards data per 575 Section 7.1
    // Note: The redirect command blocks and runs as the supervisor process.
    // The parent (calling script) should run this in background with &.
    let session_id = id.to_string();
    run_supervisor(
        session_id,
        child,
        stdin_listener,
        stdout_listener,
        stderr_listener,
    );

    Ok(())
}

/// Executes redirect and waits for completion (blocking version).
///
/// Unlike `redirect()`, this function blocks until the child process exits.
/// Useful for testing and when the caller wants to wait for completion.
///
/// # Returns
///
/// * `Ok(exit_code)` - Child exit code
/// * `Err(TelepipeError)` - Error during redirect or waiting
pub fn redirect_and_wait(
    id: &str,
    command: &str,
    args: &[String],
) -> Result<i32, TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Check for existing session
    if session_exists(id) {
        return Err(TelepipeError::DictConflict);
    }

    // Allocate FDs (for session record)
    let (stdin_fd, stdout_fd, stderr_fd) = allocate_three_fds()?;

    // Allocate ports and bind listeners
    let (stdin_listener, stdout_listener, stderr_listener) =
        allocate_three_ports_with_listeners()?;

    let stdin_port = listener_port(&stdin_listener)?;
    let stdout_port = listener_port(&stdout_listener)?;
    let stderr_port = listener_port(&stderr_listener)?;

    // Spawn child with piped stdio
    let child = spawn_child(command, args)?;
    let child_pid = child.id();

    let supervisor_pid = current_pid();

    // Write session
    let session = SessionEntry::new_redirect(
        id.to_string(),
        supervisor_pid,
        child_pid,
        (stdin_port, stdout_port, stderr_port),
        (stdin_fd, stdout_fd, stderr_fd),
    );

    write_session(&session)?;

    // Run supervisor synchronously
    let exit_code = run_supervisor_sync(
        id.to_string(),
        child,
        stdin_listener,
        stdout_listener,
        stderr_listener,
    );

    Ok(exit_code)
}

// =============================================================================
// SUPERVISOR
// =============================================================================

/// Runs the supervisor loop (async version, in background thread).
///
/// Per 575 Supervisor Architecture Section 7.1, the supervisor:
/// - Accepts TCP connections on stdin/stdout/stderr ports
/// - Forwards data bidirectionally:
///   - read(tcp_stdin_fd) -> write(child_stdin_fd)
///   - read(child_stdout_fd) -> write(tcp_stdout_fd)
///   - read(child_stderr_fd) -> write(tcp_stderr_fd)
/// - Cleans up when child exits
///
/// Per 550 Section 7: Implements one-at-a-time exec model.
fn run_supervisor(
    session_id: String,
    mut child: Child,
    stdin_listener: TcpListener,
    stdout_listener: TcpListener,
    stderr_listener: TcpListener,
) {
    // Take ownership of child's stdio handles
    let child_stdin = child.stdin.take();
    let child_stdout = child.stdout.take();
    let child_stderr = child.stderr.take();

    // Flag to signal shutdown to all threads
    let shutdown = Arc::new(AtomicBool::new(false));

    // ONE-AT-A-TIME: Create shared exec connection tracker
    // Per 550 Section 7: Only one exec connection allowed at a time
    let exec_tracker = Arc::new(ExecConnectionTracker::new());

    // Set non-blocking on listeners so we can check shutdown flag
    let _ = stdin_listener.set_nonblocking(true);
    let _ = stdout_listener.set_nonblocking(true);
    let _ = stderr_listener.set_nonblocking(true);

    // Spawn forwarding threads
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // stdin: TCP -> child stdin (gatekeeper for one-at-a-time)
    if let Some(child_stdin) = child_stdin {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_tcp_to_child_stdin(stdin_listener, child_stdin, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    // stdout: child stdout -> TCP
    if let Some(child_stdout) = child_stdout {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_child_stdout_to_tcp(stdout_listener, child_stdout, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    // stderr: child stderr -> TCP
    if let Some(child_stderr) = child_stderr {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_child_stderr_to_tcp(stderr_listener, child_stderr, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    // Wait for child to exit
    let _ = child.wait();

    // Signal all threads to shutdown
    shutdown.store(true, Ordering::SeqCst);

    // Wait for all forwarding threads to finish (with timeout)
    for handle in handles {
        let _ = handle.join();
    }

    // Clean up session
    let _ = delete_session(&session_id);
}

/// Runs the supervisor synchronously (blocking).
///
/// Returns the child's exit code.
/// Per 550 Section 7: Implements one-at-a-time exec model.
fn run_supervisor_sync(
    session_id: String,
    mut child: Child,
    stdin_listener: TcpListener,
    stdout_listener: TcpListener,
    stderr_listener: TcpListener,
) -> i32 {
    // Take ownership of child's stdio handles
    let child_stdin = child.stdin.take();
    let child_stdout = child.stdout.take();
    let child_stderr = child.stderr.take();

    // Flag to signal shutdown
    let shutdown = Arc::new(AtomicBool::new(false));

    // ONE-AT-A-TIME: Create shared exec connection tracker
    // Per 550 Section 7: Only one exec connection allowed at a time
    let exec_tracker = Arc::new(ExecConnectionTracker::new());

    // Set non-blocking on listeners
    let _ = stdin_listener.set_nonblocking(true);
    let _ = stdout_listener.set_nonblocking(true);
    let _ = stderr_listener.set_nonblocking(true);

    // Spawn forwarding threads
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    if let Some(child_stdin) = child_stdin {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_tcp_to_child_stdin(stdin_listener, child_stdin, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    if let Some(child_stdout) = child_stdout {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_child_stdout_to_tcp(stdout_listener, child_stdout, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    if let Some(child_stderr) = child_stderr {
        let shutdown_clone = Arc::clone(&shutdown);
        let tracker_clone = Arc::clone(&exec_tracker);
        let handle = thread::spawn(move || {
            forward_child_stderr_to_tcp(stderr_listener, child_stderr, shutdown_clone, tracker_clone);
        });
        handles.push(handle);
    }

    // Wait for child to exit
    let exit_status = child.wait();
    let exit_code = exit_status
        .map(|s| s.code().unwrap_or(1))
        .unwrap_or(1);

    // Signal shutdown
    shutdown.store(true, Ordering::SeqCst);

    // Wait for threads
    for handle in handles {
        let _ = handle.join();
    }

    // Clean up session
    let _ = delete_session(&session_id);

    exit_code
}

// =============================================================================
// I/O FORWARDING FUNCTIONS
// Per 575 Supervisor Architecture Section 7.1
// =============================================================================

/// Forwards data from TCP connections to child's stdin.
/// Per 575: read(tcp_stdin_fd) -> write(child_stdin_fd)
/// Per 501/515: Supports multiple serial exec calls - accepts multiple sequential connections.
/// Per 550 Section 7: Only ONE exec connection allowed at a time (one-at-a-time model).
fn forward_tcp_to_child_stdin(
    listener: TcpListener,
    mut child_stdin: ChildStdin,
    shutdown: Arc<AtomicBool>,
    exec_tracker: Arc<ExecConnectionTracker>,
) {
    // OUTER LOOP: Accept multiple connections (per 501: "arbitrary number of commands")
    // Each exec call creates a new TCP connection; we accept them sequentially.
    // Per 550 Section 7: Only one exec at a time - concurrent attempts are rejected.
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        // Accept next connection (with polling for shutdown)
        let mut tcp_stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(_) => continue, // Retry on error
        };

        // Set stream to blocking for status byte and data transfer
        let _ = tcp_stream.set_nonblocking(false);

        // ONE-AT-A-TIME: Try to acquire the exec lock
        // Per 550 Section 7: Status byte protocol for reliable rejection detection
        if !exec_tracker.try_acquire() {
            // Another exec is active - send BUSY status and close
            // Per 550 Section 7: Status byte 0x01 = BUSY
            let _ = tcp_stream.write_all(&[0x01]);
            let _ = tcp_stream.flush();
            drop(tcp_stream);
            continue;
        }

        // Exec acquired successfully - send OK status
        // Per 550 Section 7: Status byte 0x00 = OK
        if tcp_stream.write_all(&[0x00]).is_err() || tcp_stream.flush().is_err() {
            // Failed to send status byte - release lock and try next connection
            exec_tracker.release();
            continue;
        }

        // INNER LOOP: Forward data from this connection
        // Use non-blocking I/O so we can also reject concurrent connection attempts
        let _ = tcp_stream.set_nonblocking(true);
        let mut buf = [0u8; 8192];

        loop {
            if shutdown.load(Ordering::SeqCst) {
                exec_tracker.release();
                return;
            }

            // Check for concurrent connection attempts and reject them with status byte 0x01
            // This allows us to immediately reject concurrent execs instead of making them wait
            loop {
                match listener.accept() {
                    Ok((mut concurrent_stream, _)) => {
                        // Reject this concurrent connection with BUSY status
                        let _ = concurrent_stream.set_nonblocking(false);
                        let _ = concurrent_stream.write_all(&[0x01]);
                        let _ = concurrent_stream.flush();
                        drop(concurrent_stream);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break; // No more pending connections
                    }
                    Err(_) => break,
                }
            }

            // Try to read data from the active exec connection
            match tcp_stream.read(&mut buf) {
                Ok(0) => break, // This connection closed, accept next
                Ok(n) => {
                    if child_stdin.write_all(&buf[..n]).is_err() {
                        exec_tracker.release();
                        return; // Child died, exit completely
                    }
                    let _ = child_stdin.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly and continue
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(_) => break, // Error on this connection, accept next
            }
        }
        // Connection closed - release the exec lock
        exec_tracker.release();
        // Loop back to accept next connection
        // child_stdin stays open - child keeps running
    }
}

/// Forwards data from child's stdout to TCP connections.
/// Per 575: read(child_stdout_fd) -> write(tcp_stdout_fd)
/// Per 501/515: Supports multiple exec calls - accepts multiple sequential connections.
/// Per 550 Section 7: Stdout connections follow the stdin exec tracker (one-at-a-time).
fn forward_child_stdout_to_tcp(
    listener: TcpListener,
    mut child_stdout: ChildStdout,
    shutdown: Arc<AtomicBool>,
    exec_tracker: Arc<ExecConnectionTracker>,
) {
    // Make child_stdout non-blocking for polling
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
        let fd = child_stdout.as_raw_fd();
        unsafe {
            let flags = fcntl(fd, F_GETFL);
            fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        }
    }

    let mut buf = [0u8; 8192];

    // OUTER LOOP: Accept multiple connections (per 501: "arbitrary number of commands")
    // Note: The one-at-a-time model is enforced by the stdin forwarder.
    // Stdout forwarder accepts all connections; rejected exec clients will
    // close their connections when they detect stdin rejection.
    let _ = exec_tracker; // Acknowledge parameter (used for API consistency)
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        // Accept next connection
        let tcp_stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(_) => continue,
        };

        // Set stream to blocking with read timeout for data transfer
        let _ = tcp_stream.set_nonblocking(false);
        let _ = tcp_stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
        let mut tcp_stream = tcp_stream;

        // Track consecutive WouldBlock reads to detect "no more data" state
        let mut consecutive_empty_reads = 0;

        // INNER LOOP: Forward data from child stdout to this connection
        loop {
            if shutdown.load(Ordering::SeqCst) {
                return;
            }

            // Try to read from child stdout (non-blocking)
            match child_stdout.read(&mut buf) {
                Ok(0) => return, // Child stdout closed, child exited
                Ok(n) => {
                    consecutive_empty_reads = 0;
                    // Got data, send to this connection
                    if tcp_stream.write_all(&buf[..n]).is_err() {
                        break; // This connection closed, accept next
                    }
                    let _ = tcp_stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    consecutive_empty_reads += 1;

                    // Check if connection still alive using peek
                    let mut peek_buf = [0u8; 1];
                    match tcp_stream.peek(&mut peek_buf) {
                        Ok(0) => {
                            // Connection closed by peer
                            // Wait a bit longer for any pending child output before accepting new connection
                            if consecutive_empty_reads >= 10 {
                                break; // Accept next connection
                            }
                        }
                        Err(ref e) if e.kind() != std::io::ErrorKind::WouldBlock
                            && e.kind() != std::io::ErrorKind::TimedOut => {
                            if consecutive_empty_reads >= 10 {
                                break; // Connection error, accept next
                            }
                        }
                        _ => {
                            // Connection still alive, reset counter
                            consecutive_empty_reads = 0;
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(_) => {} // Other error, try again
            }
        }
        // Connection closed, loop back to accept next connection
    }
}

/// Forwards data from child's stderr to TCP connections.
/// Per 575: read(child_stderr_fd) -> write(tcp_stderr_fd)
/// Per 501/515: Supports multiple exec calls - accepts multiple sequential connections.
/// Per 550 Section 7: Stderr connections follow the stdin exec tracker (one-at-a-time).
fn forward_child_stderr_to_tcp(
    listener: TcpListener,
    mut child_stderr: ChildStderr,
    shutdown: Arc<AtomicBool>,
    exec_tracker: Arc<ExecConnectionTracker>,
) {
    // Same pattern as stdout forwarder

    // Make child_stderr non-blocking for polling
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
        let fd = child_stderr.as_raw_fd();
        unsafe {
            let flags = fcntl(fd, F_GETFL);
            fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        }
    }

    let mut buf = [0u8; 8192];

    // OUTER LOOP: Accept multiple connections (per 501: "arbitrary number of commands")
    // Note: The one-at-a-time model is enforced by the stdin forwarder.
    // Stderr forwarder accepts all connections; rejected exec clients will
    // close their connections when they detect stdin rejection.
    let _ = exec_tracker; // Acknowledge parameter (used for API consistency)
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        // Accept next connection
        let tcp_stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(_) => continue,
        };

        // Set stream to blocking with read timeout for data transfer
        let _ = tcp_stream.set_nonblocking(false);
        let _ = tcp_stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
        let mut tcp_stream = tcp_stream;

        // Track consecutive WouldBlock reads
        let mut consecutive_empty_reads = 0;

        // INNER LOOP: Forward data from child stderr to this connection
        loop {
            if shutdown.load(Ordering::SeqCst) {
                return;
            }

            // Try to read from child stderr (non-blocking)
            match child_stderr.read(&mut buf) {
                Ok(0) => return, // Child stderr closed, child exited
                Ok(n) => {
                    consecutive_empty_reads = 0;
                    // Got data, send to this connection
                    if tcp_stream.write_all(&buf[..n]).is_err() {
                        break; // This connection closed, accept next
                    }
                    let _ = tcp_stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    consecutive_empty_reads += 1;

                    // Check if connection still alive
                    let mut peek_buf = [0u8; 1];
                    match tcp_stream.peek(&mut peek_buf) {
                        Ok(0) => {
                            if consecutive_empty_reads >= 10 {
                                break; // Accept next connection
                            }
                        }
                        Err(ref e) if e.kind() != std::io::ErrorKind::WouldBlock
                            && e.kind() != std::io::ErrorKind::TimedOut => {
                            if consecutive_empty_reads >= 10 {
                                break; // Connection error, accept next
                            }
                        }
                        _ => {
                            consecutive_empty_reads = 0;
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(_) => {} // Other error, try again
            }
        }
        // Connection closed, loop back to accept next connection
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Gets the local port from a bound TcpListener.
fn listener_port(listener: &TcpListener) -> Result<u16, TelepipeError> {
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|_| TelepipeError::AllocPort)
}

/// Returns session info for a redirect operation (for info mode).
pub fn get_redirect_info(session: &SessionEntry) -> Result<RedirectInfo, TelepipeError> {
    if session.mode != SessionMode::Redirect {
        return Err(TelepipeError::CliMode);
    }

    Ok(RedirectInfo {
        id: session.id.clone(),
        host: session.host.clone(),
        child_pid: session.child_pid.ok_or(TelepipeError::ProcNoChild)?,
        supervisor_pid: session.supervisor_pid,
        stdin_port: session.stdin_port.ok_or(TelepipeError::CliInvalidPort)?,
        stdout_port: session.stdout_port.ok_or(TelepipeError::CliInvalidPort)?,
        stderr_port: session.stderr_port.ok_or(TelepipeError::CliInvalidPort)?,
        stdin_fd: session.stdin_fd.ok_or(TelepipeError::CliInvalidFd)?,
        stdout_fd: session.stdout_fd.ok_or(TelepipeError::CliInvalidFd)?,
        stderr_fd: session.stderr_fd.ok_or(TelepipeError::CliInvalidFd)?,
    })
}

/// Structured redirect session info.
#[derive(Debug, Clone)]
pub struct RedirectInfo {
    pub id: String,
    pub host: String,
    pub child_pid: u32,
    pub supervisor_pid: u32,
    pub stdin_port: u16,
    pub stdout_port: u16,
    pub stderr_port: u16,
    pub stdin_fd: u32,
    pub stdout_fd: u32,
    pub stderr_fd: u32,
}

impl RedirectInfo {
    /// Formats info as tab-separated output per spec.
    pub fn to_tsv(&self) -> String {
        format!(
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
            self.host,
            self.child_pid,
            self.supervisor_pid,
            self.stdin_port,
            self.stdout_port,
            self.stderr_port,
            self.stdin_fd,
            self.stdout_fd,
            self.stderr_fd,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_info_to_tsv() {
        let info = RedirectInfo {
            id: "test".to_string(),
            host: "127.0.0.1".to_string(),
            child_pid: 1234,
            supervisor_pid: 5678,
            stdin_port: 49152,
            stdout_port: 49153,
            stderr_port: 49154,
            stdin_fd: 3,
            stdout_fd: 4,
            stderr_fd: 5,
        };

        let tsv = info.to_tsv();
        assert!(tsv.contains("mode\tredirect"));
        assert!(tsv.contains("host\t127.0.0.1"));
        assert!(tsv.contains("pid\t1234"));
        assert!(tsv.contains("supervisor\t5678"));
        assert!(tsv.contains("stdin\t49152"));
        assert!(tsv.contains("stdout\t49153"));
        assert!(tsv.contains("stderr\t49154"));
        assert!(tsv.contains("fdin\t3"));
        assert!(tsv.contains("fdout\t4"));
        assert!(tsv.contains("fderr\t5"));
    }

    #[test]
    fn test_redirect_empty_id() {
        let result = redirect("", "echo", &["hello".to_string()]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_get_redirect_info_wrong_mode() {
        let session = SessionEntry::new_connect(
            "test".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        let result = get_redirect_info(&session);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliMode);
    }

    #[test]
    fn test_get_redirect_info_success() {
        let session = SessionEntry::new_redirect(
            "test-redirect".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let result = get_redirect_info(&session);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.id, "test-redirect");
        assert_eq!(info.child_pid, 5678);
        assert_eq!(info.supervisor_pid, 1234);
        assert_eq!(info.stdin_port, 49152);
    }

    // Note: Full redirect tests require file system access and process spawning
    // These are integration tests that would be marked #[ignore] or run separately

    #[test]
    #[ignore] // Requires process spawning and file system
    fn test_redirect_and_wait_echo() {
        use std::env;
        use crate::registry::ensure_session_dir;

        // Setup test directory
        let test_dir = env::temp_dir().join(format!("telepipe-redirect-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(test_dir.join(".telepipe/sessions"));
        env::set_var("HOME", &test_dir);
        let _ = ensure_session_dir();

        let result = redirect_and_wait(
            "test-echo",
            "echo",
            &["hello".to_string()],
        );

        assert!(result.is_ok());
        let exit_code = result.unwrap();
        assert_eq!(exit_code, 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
