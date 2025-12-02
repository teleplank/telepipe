//! Exec Mode Implementation
//!
//! This module implements the `telepipe exec` command as specified in:
//! - 501 the telepipe core spec.md (Section 2.2: Operation B: exec)
//! - 560 the telepipe implementation blueprint.md (Section 4.10: exec_mode.rs)
//!
//! Exec mode connects to an existing redirect session and streams data
//! between the client's stdio and the session's TCP ports.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::errors::TelepipeError;
use crate::registry::load_session;
use crate::session::{SessionEntry, SessionMode};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Buffer size for data forwarding (64KB)
const BUFFER_SIZE: usize = 64 * 1024;

/// Connection timeout (5 seconds)
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

// =============================================================================
// EXEC MODE ENTRY POINT
// =============================================================================

/// Executes the exec operation.
///
/// This is the main entry point for `telepipe exec --id <ID>`.
///
/// # Steps (per spec 560)
///
/// 1. Load session (no global lock needed)
/// 2. Attempt TCP connections to stdin/stdout/stderr ports
/// 3. Stream stdin → remote stdin
/// 4. Stream stdout/stderr → local stdout/stderr
/// 5. Exit when EOF
///
/// Zero dictionary writes.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(())` - Exec completed successfully
/// * `Err(TelepipeError)` - Various error conditions
pub fn exec(id: &str) -> Result<(), TelepipeError> {
    // Validate ID
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    // Step 1: Load session (no global lock per spec)
    let session = load_session(id)?;

    // Validate session is redirect mode
    if session.mode != SessionMode::Redirect {
        return Err(TelepipeError::CliMode);
    }

    // Get ports from session
    let stdin_port = session.stdin_port.ok_or(TelepipeError::CliInvalidPort)?;
    let stdout_port = session.stdout_port.ok_or(TelepipeError::CliInvalidPort)?;
    let stderr_port = session.stderr_port.ok_or(TelepipeError::CliInvalidPort)?;

    let host = &session.host;

    // Step 2: Establish TCP connections
    let stdin_stream = connect_to_port(host, stdin_port)?;
    let stdout_stream = connect_to_port(host, stdout_port)?;
    let stderr_stream = connect_to_port(host, stderr_port)?;

    // Step 3-5: Stream data bidirectionally
    stream_exec(stdin_stream, stdout_stream, stderr_stream, id)?;

    Ok(())
}

/// Executes exec and returns when complete.
///
/// This is the same as `exec()` but with explicit exit handling.
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
///
/// # Returns
///
/// * `Ok(exit_code)` - Exit code (0 for success)
/// * `Err(TelepipeError)` - Various error conditions
pub fn exec_with_exit_code(id: &str) -> Result<i32, TelepipeError> {
    exec(id)?;
    Ok(0)
}

// =============================================================================
// TCP CONNECTION
// =============================================================================

/// Connects to a TCP port on the specified host.
///
/// # Arguments
///
/// * `host` - Host to connect to (typically 127.0.0.1)
/// * `port` - Port number
///
/// # Returns
///
/// * `Ok(TcpStream)` - Connected stream
/// * `Err(TelepipeError::TcpConnect)` - Connection failed
fn connect_to_port(host: &str, port: u16) -> Result<TcpStream, TelepipeError> {
    let addr = format!("{}:{}", host, port);

    TcpStream::connect_timeout(
        &addr.parse().map_err(|_| TelepipeError::CliInvalidHost)?,
        CONNECT_TIMEOUT,
    )
    .map_err(|_| TelepipeError::TcpConnect)
}

// =============================================================================
// STREAM FORWARDING
// =============================================================================

/// Streams data between local stdio and remote TCP connections.
///
/// Creates three forwarding threads:
/// - Local stdin → Remote stdin (TCP)
/// - Remote stdout (TCP) → Local stdout
/// - Remote stderr (TCP) → Local stderr
///
/// Exits when:
/// - Local stdin closes (EOF), AND
/// - Remote stdout/stderr reach EOF (or timeout)
///
/// Per 550 Section 7: If connections close immediately, another exec is active.
///
/// # Arguments
///
/// * `stdin_stream` - TCP stream for stdin
/// * `stdout_stream` - TCP stream for stdout
/// * `stderr_stream` - TCP stream for stderr
/// * `session_id` - Session ID for error messages
fn stream_exec(
    mut stdin_stream: TcpStream,
    stdout_stream: TcpStream,
    stderr_stream: TcpStream,
    session_id: &str,
) -> Result<(), TelepipeError> {
    // ONE-AT-A-TIME: Read status byte from supervisor
    // Per 550 Section 7: Status byte protocol
    //   0x00 = OK (exec acquired, proceed)
    //   0x01 = BUSY (another exec is active)
    {
        let mut status = [0u8; 1];

        // Set timeout for status byte read (should be immediate, 2 seconds is generous)
        stdin_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| TelepipeError::TcpConnect)?;

        match stdin_stream.read_exact(&mut status) {
            Ok(_) => {
                match status[0] {
                    0x00 => {
                        // OK - exec acquired successfully, proceed
                    }
                    0x01 => {
                        // BUSY - another exec is active
                        return Err(TelepipeError::ExecAlreadyActive(session_id.to_string()));
                    }
                    other => {
                        // Unknown status byte - protocol error, but be defensive
                        eprintln!(
                            "[telepipe] warning: unknown status byte: 0x{:02x}",
                            other
                        );
                        // Proceed anyway (forward compatibility)
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                // Timeout reading status byte - session might be dead or old version
                return Err(TelepipeError::TcpConnect);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Timeout on some platforms
                return Err(TelepipeError::TcpConnect);
            }
            Err(_) => {
                // Connection closed or other error before status byte
                return Err(TelepipeError::TcpConnect);
            }
        }

        // Reset timeout for normal data streaming
        let _ = stdin_stream.set_read_timeout(None);
    }

    // Shared shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));

    // Clone streams for reading/writing
    let stdin_writer = stdin_stream;
    let stdout_reader = stdout_stream;
    let stderr_reader = stderr_stream;

    // Spawn stdin forwarder: local stdin → remote stdin
    let shutdown_stdin = Arc::clone(&shutdown);
    let stdin_handle = thread::spawn(move || {
        forward_stdin_to_tcp(stdin_writer, &shutdown_stdin)
    });

    // Spawn stdout forwarder: remote stdout → local stdout
    let shutdown_stdout = Arc::clone(&shutdown);
    let stdout_handle = thread::spawn(move || {
        forward_tcp_to_stdout(stdout_reader, &shutdown_stdout)
    });

    // Spawn stderr forwarder: remote stderr → local stderr
    let shutdown_stderr = Arc::clone(&shutdown);
    let stderr_handle = thread::spawn(move || {
        forward_tcp_to_stderr(stderr_reader, &shutdown_stderr)
    });

    // Wait for stdin to close first (user finished input)
    let _ = stdin_handle.join();

    // Give stdout/stderr time to receive any pending data from the remote
    // This is important for interactive programs like cat where output follows input
    thread::sleep(Duration::from_millis(100));

    // Signal shutdown to output threads
    shutdown.store(true, Ordering::Relaxed);

    // Wait for output threads to finish
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    Ok(())
}

/// Forwards data from local stdin to a TCP stream.
fn forward_stdin_to_tcp(mut writer: TcpStream, shutdown: &AtomicBool) -> io::Result<u64> {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut total_bytes = 0u64;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match stdin_lock.read(&mut buffer) {
            Ok(0) => {
                // EOF on stdin
                break;
            }
            Ok(n) => {
                if writer.write_all(&buffer[..n]).is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
                total_bytes += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok(total_bytes)
}

/// Forwards data from a TCP stream to local stdout.
fn forward_tcp_to_stdout(mut reader: TcpStream, shutdown: &AtomicBool) -> io::Result<u64> {
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut total_bytes = 0u64;

    // Set read timeout for polling
    let _ = reader.set_read_timeout(Some(Duration::from_millis(100)));

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match reader.read(&mut buffer) {
            Ok(0) => {
                // EOF on remote
                break;
            }
            Ok(n) => {
                if stdout_lock.write_all(&buffer[..n]).is_err() {
                    break;
                }
                if stdout_lock.flush().is_err() {
                    break;
                }
                total_bytes += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok(total_bytes)
}

/// Forwards data from a TCP stream to local stderr.
fn forward_tcp_to_stderr(mut reader: TcpStream, shutdown: &AtomicBool) -> io::Result<u64> {
    let stderr = io::stderr();
    let mut stderr_lock = stderr.lock();
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut total_bytes = 0u64;

    // Set read timeout for polling
    let _ = reader.set_read_timeout(Some(Duration::from_millis(100)));

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match reader.read(&mut buffer) {
            Ok(0) => {
                // EOF on remote
                break;
            }
            Ok(n) => {
                if stderr_lock.write_all(&buffer[..n]).is_err() {
                    break;
                }
                if stderr_lock.flush().is_err() {
                    break;
                }
                total_bytes += n as u64;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok(total_bytes)
}

// =============================================================================
// EXEC INFO
// =============================================================================

/// Returns session info for an exec operation (for display).
pub fn get_exec_info(session: &SessionEntry) -> Result<ExecInfo, TelepipeError> {
    if session.mode != SessionMode::Redirect {
        return Err(TelepipeError::CliMode);
    }

    Ok(ExecInfo {
        id: session.id.clone(),
        host: session.host.clone(),
        stdin_port: session.stdin_port.ok_or(TelepipeError::CliInvalidPort)?,
        stdout_port: session.stdout_port.ok_or(TelepipeError::CliInvalidPort)?,
        stderr_port: session.stderr_port.ok_or(TelepipeError::CliInvalidPort)?,
    })
}

/// Structured exec session info.
#[derive(Debug, Clone)]
pub struct ExecInfo {
    pub id: String,
    pub host: String,
    pub stdin_port: u16,
    pub stdout_port: u16,
    pub stderr_port: u16,
}

impl ExecInfo {
    /// Formats the connection strings for display.
    pub fn connection_strings(&self) -> (String, String, String) {
        (
            format!("{}:{}", self.host, self.stdin_port),
            format!("{}:{}", self.host, self.stdout_port),
            format!("{}:{}", self.host, self.stderr_port),
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
    fn test_exec_empty_id() {
        let result = exec("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_exec_missing_session() {
        // Session "nonexistent-session-xyz" should not exist
        let result = exec("nonexistent-session-xyz");
        assert!(result.is_err());
        // Should be DictMissing error
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);
    }

    #[test]
    fn test_get_exec_info_success() {
        let session = SessionEntry::new_redirect(
            "test-exec".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let result = get_exec_info(&session);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.id, "test-exec");
        assert_eq!(info.stdin_port, 49152);
        assert_eq!(info.stdout_port, 49153);
        assert_eq!(info.stderr_port, 49154);
    }

    #[test]
    fn test_get_exec_info_wrong_mode() {
        let session = SessionEntry::new_connect(
            "test".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        let result = get_exec_info(&session);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliMode);
    }

    #[test]
    fn test_exec_info_connection_strings() {
        let info = ExecInfo {
            id: "test".to_string(),
            host: "127.0.0.1".to_string(),
            stdin_port: 49152,
            stdout_port: 49153,
            stderr_port: 49154,
        };

        let (stdin, stdout, stderr) = info.connection_strings();
        assert_eq!(stdin, "127.0.0.1:49152");
        assert_eq!(stdout, "127.0.0.1:49153");
        assert_eq!(stderr, "127.0.0.1:49154");
    }

    #[test]
    fn test_connect_timeout_constant() {
        assert_eq!(CONNECT_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn test_buffer_size_constant() {
        assert_eq!(BUFFER_SIZE, 64 * 1024);
    }

    // Integration tests would require actual TCP servers
    // These are marked as ignored and run separately

    #[test]
    #[ignore] // Requires running redirect session
    fn test_exec_full_cycle() {
        // This test would:
        // 1. Start a redirect session with `echo hello`
        // 2. Call exec() to connect
        // 3. Verify data flows correctly
        // 4. Clean up
    }
}
