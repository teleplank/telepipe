//! Session Registry
//!
//! This module handles session dictionary I/O as specified in:
//! - 501 the telepipe core spec.md (Section 6: Session Dictionary)
//! - 560 the telepipe implementation blueprint.md (Section 4.6: session_dict)
//!
//! Sessions are stored as JSON files in `~/.telepipe/sessions/<id>.json`.
//! All writes are atomic (temp file + rename).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::config::{session_dir, session_file};
use crate::errors::TelepipeError;
use crate::session::SessionEntry;

// =============================================================================
// DIRECTORY MANAGEMENT
// =============================================================================

/// Ensures the session directory exists.
///
/// Creates `~/.telepipe/sessions/` if it doesn't exist.
/// Returns the path to the session directory.
pub fn ensure_session_dir() -> Result<PathBuf, TelepipeError> {
    let dir = session_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|_| TelepipeError::AllocDict)?;
    }
    Ok(dir)
}

// =============================================================================
// SESSION LOADING
// =============================================================================

/// Loads a session entry by ID.
///
/// Reads `~/.telepipe/sessions/<id>.json` and parses it.
///
/// # Arguments
///
/// * `id` - The session identifier
///
/// # Returns
///
/// * `Ok(SessionEntry)` - The loaded session
/// * `Err(TelepipeError::DictMissing)` - Session file not found
/// * `Err(TelepipeError::DictCorrupt)` - Invalid JSON or structure
pub fn load_session(id: &str) -> Result<SessionEntry, TelepipeError> {
    let path = session_file(id);

    if !path.exists() {
        return Err(TelepipeError::DictMissing);
    }

    let mut file = File::open(&path).map_err(|_| TelepipeError::DictMissing)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| TelepipeError::DictCorrupt)?;

    let entry = SessionEntry::from_json(&contents)?;

    // Validate that the ID matches
    if entry.id != id {
        return Err(TelepipeError::DictCorrupt);
    }

    Ok(entry)
}

/// Loads all session entries from the session directory.
///
/// Returns a vector of all valid sessions. Invalid sessions are skipped.
pub fn load_all_sessions() -> Result<Vec<SessionEntry>, TelepipeError> {
    let dir = session_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    let entries = fs::read_dir(&dir).map_err(|_| TelepipeError::DictCorrupt)?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Extract ID from filename
        if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
            // Try to load, skip invalid sessions
            if let Ok(session) = load_session(id) {
                sessions.push(session);
            }
        }
    }

    Ok(sessions)
}

// =============================================================================
// SESSION WRITING
// =============================================================================

/// Writes a session entry atomically.
///
/// Uses the atomic write pattern:
/// 1. Write to temp file
/// 2. Sync to disk (fsync)
/// 3. Rename to final location
///
/// # Arguments
///
/// * `entry` - The session entry to write
///
/// # Returns
///
/// * `Ok(())` - Session written successfully
/// * `Err(TelepipeError::AllocDict)` - Failed to write session
pub fn write_session(entry: &SessionEntry) -> Result<(), TelepipeError> {
    // Ensure directory exists
    ensure_session_dir()?;

    let final_path = session_file(&entry.id);
    let temp_path = session_dir().join(format!(".{}.tmp", entry.id));

    // Serialize to JSON
    let json = entry.to_json()?;

    // Write to temp file
    let mut file = File::create(&temp_path).map_err(|_| TelepipeError::AllocDict)?;

    file.write_all(json.as_bytes())
        .map_err(|_| TelepipeError::AllocDict)?;

    // Sync to disk
    file.sync_all().map_err(|_| TelepipeError::AllocDict)?;

    // Atomic rename
    fs::rename(&temp_path, &final_path).map_err(|_| TelepipeError::AllocDict)?;

    Ok(())
}

// =============================================================================
// SESSION DELETION
// =============================================================================

/// Deletes a session entry.
///
/// Removes `~/.telepipe/sessions/<id>.json`.
///
/// # Arguments
///
/// * `id` - The session identifier
///
/// # Returns
///
/// * `Ok(())` - Session deleted (or didn't exist)
/// * `Err(TelepipeError::DictMissing)` - Could not delete
pub fn delete_session(id: &str) -> Result<(), TelepipeError> {
    let path = session_file(id);

    if !path.exists() {
        // Already deleted, that's fine
        return Ok(());
    }

    fs::remove_file(&path).map_err(|_| TelepipeError::DictMissing)?;

    Ok(())
}

// =============================================================================
// SESSION LISTING
// =============================================================================

/// Lists all session IDs.
///
/// Returns the IDs of all session files in the session directory.
/// Does not validate the session contents.
pub fn list_sessions() -> Result<Vec<String>, TelepipeError> {
    let dir = session_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();

    let entries = fs::read_dir(&dir).map_err(|_| TelepipeError::DictCorrupt)?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Extract ID from filename (skip hidden temp files)
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if !stem.starts_with('.') {
                ids.push(stem.to_string());
            }
        }
    }

    Ok(ids)
}

/// Checks if a session exists.
///
/// # Arguments
///
/// * `id` - The session identifier
///
/// # Returns
///
/// `true` if the session file exists
pub fn session_exists(id: &str) -> bool {
    session_file(id).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Unique counter for test isolation
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Creates a unique temporary session directory for testing.
    /// Each test gets its own directory to avoid race conditions.
    fn setup_test_dir() -> PathBuf {
        let unique_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let thread_id = std::thread::current().id();
        let test_dir = env::temp_dir().join(format!(
            "telepipe-test-{}-{:?}-{}",
            std::process::id(),
            thread_id,
            unique_id
        ));
        let sessions_dir = test_dir.join(".telepipe/sessions");
        fs::create_dir_all(&sessions_dir).unwrap();

        // Set HOME for this test (note: not thread-safe, but tests use unique dirs)
        env::set_var("HOME", &test_dir);

        sessions_dir
    }

    /// Cleans up the test directory
    fn cleanup_test_dir(test_dir: &PathBuf) {
        if let Some(parent) = test_dir.parent() {
            if let Some(grandparent) = parent.parent() {
                let _ = fs::remove_dir_all(grandparent);
            }
        }
    }

    // Note: These tests modify the HOME environment variable which is process-global.
    // Run with `cargo test -- --test-threads=1` for reliable results.

    #[test]
    fn test_session_entry_json_roundtrip() {
        // Test that doesn't depend on file system
        let entry = SessionEntry::new_redirect(
            "json-test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        let json = entry.to_json().unwrap();
        let restored = SessionEntry::from_json(&json).unwrap();
        assert_eq!(entry, restored);
    }

    #[test]
    fn test_session_entry_connect_json() {
        // Test that doesn't depend on file system
        let entry = SessionEntry::new_connect(
            "connect-json".to_string(),
            1234,
            "127.0.0.1".to_string(),
            9222,
            3,
        );

        let json = entry.to_json().unwrap();
        let restored = SessionEntry::from_json(&json).unwrap();
        assert_eq!(entry.mode, restored.mode);
        assert_eq!(entry.connect_port, restored.connect_port);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_ensure_session_dir() {
        let test_dir = setup_test_dir();

        // Remove the sessions dir to test creation
        let _ = fs::remove_dir_all(&test_dir);

        let result = ensure_session_dir();
        assert!(result.is_ok());

        let dir = result.unwrap();
        assert!(dir.exists());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_write_and_load_session() {
        let test_dir = setup_test_dir();

        let entry = SessionEntry::new_redirect(
            "test-session".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        // Write session
        let write_result = write_session(&entry);
        assert!(write_result.is_ok(), "Failed to write session");

        // Load session
        let load_result = load_session("test-session");
        assert!(load_result.is_ok(), "Failed to load session");

        let loaded = load_result.unwrap();
        assert_eq!(loaded.id, "test-session");
        assert_eq!(loaded.supervisor_pid, 1234);
        assert_eq!(loaded.child_pid, Some(5678));

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_load_missing_session() {
        let test_dir = setup_test_dir();

        let result = load_session("nonexistent-xyz");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TelepipeError::DictMissing);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_delete_session() {
        let test_dir = setup_test_dir();

        let entry = SessionEntry::new_redirect(
            "to-delete".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );

        // Write session
        write_session(&entry).unwrap();
        assert!(session_exists("to-delete"));

        // Delete session
        let result = delete_session("to-delete");
        assert!(result.is_ok());
        assert!(!session_exists("to-delete"));

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_delete_nonexistent_session() {
        let test_dir = setup_test_dir();

        // Deleting nonexistent session should succeed
        let result = delete_session("nonexistent-del");
        assert!(result.is_ok());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_list_sessions() {
        let test_dir = setup_test_dir();

        // Create multiple sessions
        for i in 0..3 {
            let entry = SessionEntry::new_redirect(
                format!("list-session-{}", i),
                1234 + i,
                5678 + i,
                (49152 + i as u16, 49153 + i as u16, 49154 + i as u16),
                (3 + i, 4 + i, 5 + i),
            );
            write_session(&entry).unwrap();
        }

        let ids = list_sessions().unwrap();
        assert!(ids.len() >= 3);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_list_sessions_empty_dir() {
        let test_dir = setup_test_dir();

        // Clear any existing sessions
        for id in list_sessions().unwrap_or_default() {
            let _ = delete_session(&id);
        }

        let ids = list_sessions().unwrap();
        assert!(ids.is_empty());

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Run with --ignored flag or --test-threads=1
    fn test_session_exists() {
        let test_dir = setup_test_dir();

        assert!(!session_exists("not-here-exists"));

        let entry = SessionEntry::new_redirect(
            "exists-test".to_string(),
            1234,
            5678,
            (49152, 49153, 49154),
            (3, 4, 5),
        );
        write_session(&entry).unwrap();

        assert!(session_exists("exists-test"));

        cleanup_test_dir(&test_dir);
    }
}
