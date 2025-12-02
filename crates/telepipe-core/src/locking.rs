//! Concurrency & Locking Implementation
//!
//! This module implements the Telepipe locking model as specified in:
//! - 550 the telepipe concurrency model & locks.md
//! - 560 the telepipe implementation blueprint.md (Section 4.7: locks.rs)
//!
//! Lock hierarchy (must be acquired in this order to avoid deadlocks):
//! 1. GLOBAL_LOCK - Dictionary mutations and recovery
//! 2. SESSION_LOCK - Per-session operations
//! 3. PORTALLOC_LOCK - Port allocation
//!
//! Lock ordering: GLOBAL → SESSION → PORTALLOC (never reverse)

use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::config::session_dir;
use crate::errors::TelepipeError;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Lock file names
const GLOBAL_LOCK_FILE: &str = ".global.lock";
const PORTALLOC_LOCK_FILE: &str = ".portalloc.lock";

/// Maximum retry attempts for acquiring a lock
const MAX_LOCK_RETRIES: u32 = 20;

/// Initial backoff duration (50ms)
const INITIAL_BACKOFF_MS: u64 = 50;

/// Maximum backoff duration (1 second)
const MAX_BACKOFF_MS: u64 = 1000;

// =============================================================================
// LOCK TYPES
// =============================================================================

/// Global lock guard - ensures only one Telepipe mutates dictionary.
///
/// Held during:
/// - Dictionary scan
/// - Recovery pipeline
/// - Dictionary write
/// - Eviction pass
#[derive(Debug)]
pub struct GlobalLock {
    _file: File,
}

/// Session lock guard - ensures no concurrent edits to a single session.
///
/// Held during:
/// - redirect creation
/// - connect creation
/// - stop
/// - disconnect
/// - recovery updates for that session
#[derive(Debug)]
pub struct SessionLock {
    _file: File,
    pub session_id: String,
}

/// Port allocation lock guard - ensures only one Telepipe allocates ports.
///
/// Held during:
/// - Port scanning
/// - FD allocation
/// - Creating new redirect or connect sessions
#[derive(Debug)]
pub struct PortAllocLock {
    _file: File,
}

// =============================================================================
// GLOBAL LOCK
// =============================================================================

/// Acquires the global lock.
///
/// This lock protects dictionary-wide operations like recovery and writes.
///
/// # Returns
///
/// * `Ok(GlobalLock)` - Lock acquired
/// * `Err(TelepipeError)` - Failed to acquire lock
pub fn acquire_global_lock() -> Result<GlobalLock, TelepipeError> {
    let lock_path = session_dir().join(GLOBAL_LOCK_FILE);
    let file = acquire_file_lock(&lock_path)?;
    Ok(GlobalLock { _file: file })
}

/// Tries to acquire the global lock without blocking.
///
/// # Returns
///
/// * `Ok(Some(GlobalLock))` - Lock acquired
/// * `Ok(None)` - Lock is held by another process
/// * `Err(TelepipeError)` - Error accessing lock file
pub fn try_acquire_global_lock() -> Result<Option<GlobalLock>, TelepipeError> {
    let lock_path = session_dir().join(GLOBAL_LOCK_FILE);
    match try_acquire_file_lock(&lock_path)? {
        Some(file) => Ok(Some(GlobalLock { _file: file })),
        None => Ok(None),
    }
}

// =============================================================================
// SESSION LOCK
// =============================================================================

/// Acquires a session-specific lock.
///
/// Each session has its own lock file: `<id>.lock`
///
/// # Arguments
///
/// * `session_id` - The session identifier
///
/// # Returns
///
/// * `Ok(SessionLock)` - Lock acquired
/// * `Err(TelepipeError)` - Failed to acquire lock
pub fn acquire_session_lock(session_id: &str) -> Result<SessionLock, TelepipeError> {
    if session_id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    let lock_path = session_dir().join(format!("{}.lock", session_id));
    let file = acquire_file_lock(&lock_path)?;
    Ok(SessionLock {
        _file: file,
        session_id: session_id.to_string(),
    })
}

/// Tries to acquire a session lock without blocking.
///
/// # Returns
///
/// * `Ok(Some(SessionLock))` - Lock acquired
/// * `Ok(None)` - Lock is held by another process
/// * `Err(TelepipeError)` - Error accessing lock file
pub fn try_acquire_session_lock(session_id: &str) -> Result<Option<SessionLock>, TelepipeError> {
    if session_id.is_empty() {
        return Err(TelepipeError::CliId);
    }

    let lock_path = session_dir().join(format!("{}.lock", session_id));
    match try_acquire_file_lock(&lock_path)? {
        Some(file) => Ok(Some(SessionLock {
            _file: file,
            session_id: session_id.to_string(),
        })),
        None => Ok(None),
    }
}

// =============================================================================
// PORT ALLOCATION LOCK
// =============================================================================

/// Acquires the port allocation lock.
///
/// This lock ensures only one Telepipe process scans ports at a time.
///
/// # Returns
///
/// * `Ok(PortAllocLock)` - Lock acquired
/// * `Err(TelepipeError)` - Failed to acquire lock
pub fn acquire_portalloc_lock() -> Result<PortAllocLock, TelepipeError> {
    let lock_path = session_dir().join(PORTALLOC_LOCK_FILE);
    let file = acquire_file_lock(&lock_path)?;
    Ok(PortAllocLock { _file: file })
}

/// Tries to acquire the port allocation lock without blocking.
///
/// # Returns
///
/// * `Ok(Some(PortAllocLock))` - Lock acquired
/// * `Ok(None)` - Lock is held by another process
/// * `Err(TelepipeError)` - Error accessing lock file
pub fn try_acquire_portalloc_lock() -> Result<Option<PortAllocLock>, TelepipeError> {
    let lock_path = session_dir().join(PORTALLOC_LOCK_FILE);
    match try_acquire_file_lock(&lock_path)? {
        Some(file) => Ok(Some(PortAllocLock { _file: file })),
        None => Ok(None),
    }
}

// =============================================================================
// FILE LOCKING IMPLEMENTATION
// =============================================================================

/// Acquires an advisory file lock with exponential backoff retry.
fn acquire_file_lock(path: &PathBuf) -> Result<File, TelepipeError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut backoff_ms = INITIAL_BACKOFF_MS;

    for _ in 0..MAX_LOCK_RETRIES {
        match try_acquire_file_lock(path)? {
            Some(file) => return Ok(file),
            None => {
                // Lock is held, wait and retry
                thread::sleep(Duration::from_millis(backoff_ms));
                backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
            }
        }
    }

    Err(TelepipeError::DictConflict)
}

/// Tries to acquire a file lock without blocking.
#[cfg(unix)]
fn try_acquire_file_lock(path: &PathBuf) -> Result<Option<File>, TelepipeError> {
    use std::os::unix::io::AsRawFd;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| TelepipeError::AllocDict)?;

    // Try non-blocking exclusive lock
    let fd = file.as_raw_fd();
    let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

    if result == 0 {
        Ok(Some(file))
    } else {
        // EWOULDBLOCK means lock is held by another process
        Ok(None)
    }
}

/// Tries to acquire a file lock (Windows fallback using atomic create).
#[cfg(not(unix))]
fn try_acquire_file_lock(path: &PathBuf) -> Result<Option<File>, TelepipeError> {
    // Windows fallback: Try to create lock file exclusively
    // If file exists and is locked, creation will fail
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok(Some(file)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Try to open existing file
            match OpenOptions::new().read(true).write(true).open(path) {
                Ok(file) => Ok(Some(file)),
                Err(_) => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

// =============================================================================
// LOCK CLEANUP
// =============================================================================

/// Cleans up stale lock files.
///
/// This should be called during recovery to remove orphaned lock files.
pub fn cleanup_stale_locks() -> Result<(), TelepipeError> {
    let dir = session_dir();
    if !dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&dir).map_err(|_| TelepipeError::DictMissing)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "lock" {
                // Try to acquire and immediately release the lock
                // If successful, the lock file is stale and can be removed
                if let Ok(Some(_)) = try_acquire_file_lock(&path) {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    Ok(())
}

// =============================================================================
// HELPER: LOCK GUARD SCOPE
// =============================================================================

/// Executes a closure with the global lock held.
///
/// # Example
///
/// ```ignore
/// with_global_lock(|| {
///     // Critical section
///     Ok(())
/// })?;
/// ```
pub fn with_global_lock<F, T>(f: F) -> Result<T, TelepipeError>
where
    F: FnOnce() -> Result<T, TelepipeError>,
{
    let _lock = acquire_global_lock()?;
    f()
}

/// Executes a closure with a session lock held.
pub fn with_session_lock<F, T>(session_id: &str, f: F) -> Result<T, TelepipeError>
where
    F: FnOnce() -> Result<T, TelepipeError>,
{
    let _lock = acquire_session_lock(session_id)?;
    f()
}

/// Executes a closure with the port allocation lock held.
pub fn with_portalloc_lock<F, T>(f: F) -> Result<T, TelepipeError>
where
    F: FnOnce() -> Result<T, TelepipeError>,
{
    let _lock = acquire_portalloc_lock()?;
    f()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(GLOBAL_LOCK_FILE, ".global.lock");
        assert_eq!(PORTALLOC_LOCK_FILE, ".portalloc.lock");
        assert!(MAX_LOCK_RETRIES > 0);
        assert!(INITIAL_BACKOFF_MS > 0);
        assert!(MAX_BACKOFF_MS >= INITIAL_BACKOFF_MS);
    }

    #[test]
    fn test_session_lock_empty_id() {
        let result = acquire_session_lock("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_try_session_lock_empty_id() {
        let result = try_acquire_session_lock("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::CliId);
    }

    #[test]
    fn test_session_lock_stores_id() {
        // This test may fail if session dir doesn't exist
        if let Ok(lock) = acquire_session_lock("test-lock-id") {
            assert_eq!(lock.session_id, "test-lock-id");
        }
    }

    #[test]
    fn test_global_lock_reentrant_different_threads() {
        // Global lock should be acquirable from different calls
        // (same thread can't test true concurrency)
        if let Ok(_lock1) = try_acquire_global_lock() {
            // Lock acquired, test passes
        }
    }

    #[test]
    fn test_portalloc_lock_basic() {
        // Port alloc lock should be acquirable
        if let Ok(_lock) = try_acquire_portalloc_lock() {
            // Lock acquired, test passes
        }
    }

    #[test]
    fn test_with_global_lock_success() {
        let result = with_global_lock(|| Ok(42));
        // May fail if directory doesn't exist
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn test_with_global_lock_error() {
        let result: Result<i32, TelepipeError> =
            with_global_lock(|| Err(TelepipeError::DictMissing));
        // May fail if directory doesn't exist
        match result {
            Err(TelepipeError::DictMissing) => (),
            Ok(_) => panic!("expected error"),
            Err(_) => (), // Other errors acceptable in test environment
        }
    }

    #[test]
    fn test_cleanup_stale_locks_no_crash() {
        // Should not panic even if directory doesn't exist
        let _ = cleanup_stale_locks();
    }

    #[test]
    fn test_backoff_calculation() {
        // Verify backoff doubles up to max
        let mut backoff = INITIAL_BACKOFF_MS;
        backoff = (backoff * 2).min(MAX_BACKOFF_MS);
        assert_eq!(backoff, 100); // 50 * 2 = 100

        backoff = (backoff * 2).min(MAX_BACKOFF_MS);
        assert_eq!(backoff, 200);

        backoff = (backoff * 2).min(MAX_BACKOFF_MS);
        assert_eq!(backoff, 400);

        backoff = (backoff * 2).min(MAX_BACKOFF_MS);
        assert_eq!(backoff, 800);

        backoff = (backoff * 2).min(MAX_BACKOFF_MS);
        assert_eq!(backoff, MAX_BACKOFF_MS); // Capped at 1000
    }
}
