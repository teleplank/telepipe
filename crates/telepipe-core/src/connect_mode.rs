//! Connect Mode Implementation
//!
//! This module implements the `telepipe connect` command as specified in:
//! - 501 the telepipe core spec.md (Section 2.3: Operation C: connect)
//! - 560 the telepipe implementation blueprint.md (Section 4.11: connect_mode.rs)
//!
//! Connect mode attaches a supervisor to an existing external TCP service
//! (e.g., Chrome DevTools Protocol on port 9222).

use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::errors::TelepipeError;
use crate::fds::allocate_fd;
use crate::process_spawn::current_pid;
use crate::registry::{session_exists, write_session, delete_session};
use crate::session::{SessionEntry, SessionMode};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Connection timeout (5 seconds)
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Supervisor poll interval (100ms)
const POLL_INTERVAL: Duration = Duration::from_millis(100);

// =============================================================================
// CONNECT MODE ENTRY POINT
// =============================================================================

/// Executes the connect operation.
///
/// This is the main entry point for `telepipe connect --id <ID> --host <HOST> --port <PORT>`.
///
/// # Steps (per spec 560)
///
/// 1. Check if session ID already exists (return DictConflict if so)
/// 2. Allocate 1 FD
/// 3. Create TCP connection to external service
/// 4. Spawn supervisor
/// 5. Write session dictionary
/// 6. Return success (supervisor continues running)
///
/// # Arguments
///
/// * `id` - Mnemonic session identifier
/// * `host` - Host to connect to (e.g., "127.0.0.1")
/// * `port` - Port to connect to (e.g., 9222 for CDP)
///
/// # Returns
///
/// * `Ok(())` - Connect started successfully
/// * `Err(TelepipeError)` - Various error conditions
pub fn connect(id: &str, host: &str, port: u16) -> Result<(), TelepipeError> {
    // Validate inputs
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }
    if host.is_empty() {
        return Err(TelepipeError::CliInvalidHost);
    }
    if port == 0 {
        return Err(TelepipeError::CliInvalidPort);
    }

    // Check for existing session with same ID
    if session_exists(id) {
        return Err(TelepipeError::DictConflict);
    }

    // Step 1: Allocate FD (connect mode uses 1 FD)
    let connect_fd = allocate_fd()?;

    // Step 2: Create TCP connection to external service
    let stream = connect_to_external(host, port)?;

    // Step 3: Get supervisor PID (current process)
    let supervisor_pid = current_pid();

    // Step 4: Create and write session entry
    let session = SessionEntry::new_connect(
        id.to_string(),
        supervisor_pid,
        host.to_string(),
        port,
        connect_fd,
    );

    write_session(&session)?;

    // Step 5: Start supervisor in background thread
    let session_id = id.to_string();
    thread::spawn(move || {
        run_connect_supervisor(session_id, stream);
    });

    Ok(())
}

/// Executes connect and waits for completion (blocking version).
///
/// Unlike `connect()`, this function blocks until the connection closes.
/// Useful for testing and when the caller wants to wait for completion.
///
/// # Returns
///
/// * `Ok(())` - Connection completed normally
/// * `Err(TelepipeError)` - Error during connect
pub fn connect_and_wait(id: &str, host: &str, port: u16) -> Result<(), TelepipeError> {
    // Validate inputs
    if id.is_empty() {
        return Err(TelepipeError::CliId);
    }
    if host.is_empty() {
        return Err(TelepipeError::CliInvalidHost);
    }
    if port == 0 {
        return Err(TelepipeError::CliInvalidPort);
    }

    // Check for existing session
    if session_exists(id) {
        return Err(TelepipeError::DictConflict);
    }

    // Allocate FD
    let connect_fd = allocate_fd()?;

    // Create TCP connection
    let stream = connect_to_external(host, port)?;

    let supervisor_pid = current_pid();

    // Write session
    let session = SessionEntry::new_connect(
        id.to_string(),
        supervisor_pid,
        host.to_string(),
        port,
        connect_fd,
    );

    write_session(&session)?;

    // Run supervisor synchronously
    run_connect_supervisor(id.to_string(), stream);

    Ok(())
}

// =============================================================================
// TCP CONNECTION
// =============================================================================

/// Connects to an external TCP service.
///
/// # Arguments
///
/// * `host` - Host to connect to
/// * `port` - Port to connect to
///
/// # Returns
///
/// * `Ok(TcpStream)` - Connected stream
/// * `Err(TelepipeError::TcpConnect)` - Connection failed
fn connect_to_external(host: &str, port: u16) -> Result<TcpStream, TelepipeError> {
    let addr = format!("{}:{}", host, port);

    TcpStream::connect_timeout(
        &addr.parse().map_err(|_| TelepipeError::CliInvalidHost)?,
        CONNECT_TIMEOUT,
    )
    .map_err(|_| TelepipeError::TcpConnect)
}

// =============================================================================
// SUPERVISOR
// =============================================================================

/// Runs the connect-mode supervisor.
///
/// The supervisor maintains the TCP connection and cleans up when it closes.
///
/// # Arguments
///
/// * `session_id` - Session identifier for cleanup
/// * `stream` - TCP stream to the external service
fn run_connect_supervisor(session_id: String, stream: TcpStream) {
    // Set stream to non-blocking for polling
    let _ = stream.set_nonblocking(true);

    // Keep the connection alive until it closes
    // In a full implementation, this would forward data to/from the FD
    loop {
        // Check if connection is still alive by peeking
        let mut buf = [0u8; 1];
        match stream.peek(&mut buf) {
            Ok(0) => {
                // Connection closed by remote
                break;
            }
            Ok(_) => {
                // Data available, connection alive
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data, but connection still open
            }
            Err(_) => {
                // Connection error
                break;
            }
        }

        thread::sleep(POLL_INTERVAL);
    }

    // Clean up session
    let _ = delete_session(&session_id);
}

// =============================================================================
// CONNECT INFO
// =============================================================================

/// Returns session info for a connect operation (for info mode).
pub fn get_connect_info(session: &SessionEntry) -> Result<ConnectInfo, TelepipeError> {
    if session.mode != SessionMode::Connect {
        return Err(TelepipeError::CliMode);
    }

    Ok(ConnectInfo {
        id: session.id.clone(),
        host: session.host.clone(),
        port: session.connect_port.ok_or(TelepipeError::CliInvalidPort)?,
        supervisor_pid: session.supervisor_pid,
        connect_fd: session.connect_fd.ok_or(TelepipeError::CliInvalidFd)?,
    })
}

/// Structured connect session info.
#[derive(Debug, Clone)]
pub struct ConnectInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub supervisor_pid: u32,
    pub connect_fd: u32,
}

impl ConnectInfo {
    /// Formats info as tab-separated output per spec.
    pub fn to_tsv(&self) -> String {
        format!(
            "mode\tconnect\n\
             host\t{}\n\
             port\t{}\n\
             supervisor\t{}\n\
             fd\t{}",
            self.host,
            self.port,
            self.supervisor_pid,
            self.connect_fd,
        )
    }

    /// Returns the connection string.
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_empty_id() {
        let result = connect("", "127.0.0.1", 9222);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_connect_empty_host() {
        let result = connect("test", "", 9222);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliInvalidHost);
    }

    #[test]
    fn test_connect_zero_port() {
        let result = connect("test", "127.0.0.1", 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliInvalidPort);
    }

    #[test]
    fn test_get_connect_info_success() {
        let session = SessionEntry::new_connect(
            "test-connect".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        let result = get_connect_info(&session);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.id, "test-connect");
        assert_eq!(info.host, "127.0.0.1");
        assert_eq!(info.port, 9222);
        assert_eq!(info.supervisor_pid, 1234);
        assert_eq!(info.connect_fd, 3);
    }

    #[test]
    fn test_get_connect_info_wrong_mode() {
        let session = SessionEntry::new_redirect(
            "test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let result = get_connect_info(&session);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliMode);
    }

    #[test]
    fn test_connect_info_to_tsv() {
        let info = ConnectInfo {
            id: "test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 9222,
            supervisor_pid: 1234,
            connect_fd: 3,
        };

        let tsv = info.to_tsv();
        assert!(tsv.contains("mode\tconnect"));
        assert!(tsv.contains("host\t127.0.0.1"));
        assert!(tsv.contains("port\t9222"));
        assert!(tsv.contains("supervisor\t1234"));
        assert!(tsv.contains("fd\t3"));
    }

    #[test]
    fn test_connect_info_connection_string() {
        let info = ConnectInfo {
            id: "test".to_string(),
            host: "localhost".to_string(),
            port: 9222,
            supervisor_pid: 1234,
            connect_fd: 3,
        };

        assert_eq!(info.connection_string(), "localhost:9222");
    }

    #[test]
    fn test_connect_timeout_constant() {
        assert_eq!(CONNECT_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn test_poll_interval_constant() {
        assert_eq!(POLL_INTERVAL, Duration::from_millis(100));
    }

    // Integration tests require actual TCP servers
    #[test]
    #[ignore] // Requires external TCP service
    fn test_connect_to_cdp() {
        // This test would:
        // 1. Start Chrome with --remote-debugging-port=9222
        // 2. Call connect("cdp", "127.0.0.1", 9222)
        // 3. Verify session created
        // 4. Clean up
    }
}
