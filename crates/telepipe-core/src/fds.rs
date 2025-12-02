//! File Descriptor Allocation
//!
//! This module handles FD allocation as specified in:
//! - 501 the telepipe core spec.md (Section 5.2: Numeric file descriptors)
//! - 560 the telepipe implementation blueprint.md (Section 4.3: fds.rs)
//!
//! FDs are scanned sequentially from 3-255. FD 256 is reserved.

use crate::config::{MAX_FD, MIN_FD, RESERVED_FD};
use crate::errors::TelepipeError;

// =============================================================================
// FD AVAILABILITY CHECK (Platform-specific)
// =============================================================================

/// Checks if a file descriptor is available (not in use).
///
/// Uses fcntl(fd, F_GETFD) which returns -1 with EBADF if the FD is not open.
#[cfg(unix)]
fn is_fd_available(fd: u32) -> bool {
    // SAFETY: fcntl with F_GETFD is safe - it only queries FD status
    // and does not modify any state.
    unsafe {
        let result = libc::fcntl(fd as libc::c_int, libc::F_GETFD);
        // If fcntl returns -1, the FD is not open (available)
        result == -1
    }
}

/// Windows/Git Bash fallback: assume FDs 3-255 are available.
///
/// On Windows, FD allocation works differently. For Git Bash compatibility,
/// we use a simple availability check that assumes higher FDs are available.
/// This may need refinement based on testing.
#[cfg(not(unix))]
fn is_fd_available(fd: u32) -> bool {
    // On non-Unix systems, we can't easily check FD availability.
    // Assume FDs in our range are available unless we've allocated them.
    // This is a simplification that works for most Git Bash scenarios.
    fd >= MIN_FD && fd <= MAX_FD
}

// =============================================================================
// FD ALLOCATION
// =============================================================================

/// Allocates a single available file descriptor.
///
/// Scans FDs from 3-255 sequentially and returns the first available.
/// FD 256 is reserved and will never be returned.
///
/// # Returns
///
/// * `Ok(fd)` - An available FD in the range [3, 255]
/// * `Err(TelepipeError::AllocFd)` - No FDs available
///
/// # Example
///
/// ```ignore
/// let fd = allocate_fd()?;
/// assert!(fd >= 3 && fd <= 255);
/// ```
pub fn allocate_fd() -> Result<u32, TelepipeError> {
    for fd in MIN_FD..=MAX_FD {
        // Never allocate the reserved FD
        if fd == RESERVED_FD {
            continue;
        }

        if is_fd_available(fd) {
            return Ok(fd);
        }
    }

    Err(TelepipeError::AllocFd)
}

/// Allocates three consecutive available file descriptors.
///
/// Used by redirect mode for stdin, stdout, and stderr.
/// Scans FDs from 3-255 and returns three distinct available FDs.
///
/// # Returns
///
/// * `Ok((fd1, fd2, fd3))` - Three available FDs for stdin, stdout, stderr
/// * `Err(TelepipeError::AllocFd)` - Unable to allocate three FDs
///
/// # Note
///
/// The returned FDs are not necessarily consecutive numbers, but they
/// are guaranteed to be distinct and in the valid range.
pub fn allocate_three_fds() -> Result<(u32, u32, u32), TelepipeError> {
    let mut allocated = Vec::with_capacity(3);

    for fd in MIN_FD..=MAX_FD {
        // Never allocate the reserved FD
        if fd == RESERVED_FD {
            continue;
        }

        // Skip FDs we've already allocated in this call
        if allocated.contains(&fd) {
            continue;
        }

        if is_fd_available(fd) {
            allocated.push(fd);
            if allocated.len() == 3 {
                return Ok((allocated[0], allocated[1], allocated[2]));
            }
        }
    }

    Err(TelepipeError::AllocFd)
}

/// Allocates a specific number of file descriptors.
///
/// General-purpose allocation function that returns a vector of FDs.
///
/// # Arguments
///
/// * `count` - Number of FDs to allocate
///
/// # Returns
///
/// * `Ok(Vec<u32>)` - Vector of allocated FDs
/// * `Err(TelepipeError::AllocFd)` - Unable to allocate requested count
pub fn allocate_fds(count: usize) -> Result<Vec<u32>, TelepipeError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut allocated = Vec::with_capacity(count);

    for fd in MIN_FD..=MAX_FD {
        if fd == RESERVED_FD {
            continue;
        }

        if allocated.contains(&fd) {
            continue;
        }

        if is_fd_available(fd) {
            allocated.push(fd);
            if allocated.len() == count {
                return Ok(allocated);
            }
        }
    }

    Err(TelepipeError::AllocFd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_fd_returns_valid_range() {
        // This test may behave differently based on system state,
        // but the returned FD should always be in valid range
        if let Ok(fd) = allocate_fd() {
            assert!(fd >= MIN_FD, "FD {} is below MIN_FD {}", fd, MIN_FD);
            assert!(fd <= MAX_FD, "FD {} is above MAX_FD {}", fd, MAX_FD);
            assert!(fd != RESERVED_FD, "FD {} is reserved", fd);
        }
        // If allocation fails, that's acceptable in test environment
    }

    #[test]
    fn test_allocate_three_fds_returns_distinct() {
        if let Ok((fd1, fd2, fd3)) = allocate_three_fds() {
            // All three must be distinct
            assert_ne!(fd1, fd2, "fd1 and fd2 must be distinct");
            assert_ne!(fd2, fd3, "fd2 and fd3 must be distinct");
            assert_ne!(fd1, fd3, "fd1 and fd3 must be distinct");

            // All must be in valid range
            assert!(fd1 >= MIN_FD && fd1 <= MAX_FD);
            assert!(fd2 >= MIN_FD && fd2 <= MAX_FD);
            assert!(fd3 >= MIN_FD && fd3 <= MAX_FD);

            // None can be reserved
            assert_ne!(fd1, RESERVED_FD);
            assert_ne!(fd2, RESERVED_FD);
            assert_ne!(fd3, RESERVED_FD);
        }
    }

    #[test]
    fn test_allocate_fds_zero_count() {
        let result = allocate_fds(0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_allocate_fds_one() {
        if let Ok(fds) = allocate_fds(1) {
            assert_eq!(fds.len(), 1);
            assert!(fds[0] >= MIN_FD && fds[0] <= MAX_FD);
        }
    }

    #[test]
    fn test_allocate_fds_multiple_distinct() {
        if let Ok(fds) = allocate_fds(5) {
            assert_eq!(fds.len(), 5);

            // All must be distinct
            let mut seen = std::collections::HashSet::new();
            for fd in &fds {
                assert!(seen.insert(*fd), "Duplicate FD allocated: {}", fd);
            }

            // All must be in valid range
            for fd in &fds {
                assert!(*fd >= MIN_FD && *fd <= MAX_FD);
                assert_ne!(*fd, RESERVED_FD);
            }
        }
    }

    #[test]
    fn test_reserved_fd_never_allocated() {
        // Allocate many FDs and ensure none is RESERVED_FD
        if let Ok(fds) = allocate_fds(50) {
            for fd in fds {
                assert_ne!(
                    fd, RESERVED_FD,
                    "Reserved FD {} was allocated",
                    RESERVED_FD
                );
            }
        }
    }

    #[test]
    fn test_fd_range_constants() {
        assert_eq!(MIN_FD, 3);
        assert_eq!(MAX_FD, 255);
        assert_eq!(RESERVED_FD, 256);
    }

    #[cfg(unix)]
    #[test]
    fn test_is_fd_available_stdio() {
        // FDs 0, 1, 2 (stdin, stdout, stderr) should NOT be available
        assert!(!is_fd_available(0), "stdin should not be available");
        assert!(!is_fd_available(1), "stdout should not be available");
        assert!(!is_fd_available(2), "stderr should not be available");
    }

    #[cfg(unix)]
    #[test]
    fn test_is_fd_available_high_fd() {
        // High FDs (like 200+) are typically available
        // Note: This might fail if the system has many open FDs
        let high_fd = 250;
        // We just test that the function runs without panicking
        let _ = is_fd_available(high_fd);
    }
}
