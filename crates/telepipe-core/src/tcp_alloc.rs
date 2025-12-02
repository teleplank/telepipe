//! TCP Port Allocation
//!
//! This module handles port allocation as specified in:
//! - 501 the telepipe core spec.md (Section 5.1: TCP ports)
//! - 560 the telepipe implementation blueprint.md (Section 4.2: tcp_alloc.rs)
//!
//! Ports are scanned sequentially from 49152-65535 (IANA ephemeral range).
//! Availability is verified by attempting to bind.

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use crate::config::{MAX_PORT, MIN_PORT};
use crate::errors::TelepipeError;

// =============================================================================
// PORT AVAILABILITY CHECK
// =============================================================================

/// Checks if a port is available by attempting to bind to it.
///
/// The binding is immediately released after the check.
/// This is the most reliable way to verify port availability.
fn is_port_available(port: u16) -> bool {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(addr).is_ok()
}

/// Checks if a port is available and returns the bound listener.
///
/// This version keeps the listener bound to prevent race conditions
/// between checking and actual use.
fn try_bind_port(port: u16) -> Option<TcpListener> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    TcpListener::bind(addr).ok()
}

// =============================================================================
// PORT ALLOCATION
// =============================================================================

/// Allocates a single available TCP port.
///
/// Scans ports from 49152-65535 sequentially and returns the first available.
///
/// # Returns
///
/// * `Ok(port)` - An available port in the ephemeral range
/// * `Err(TelepipeError::AllocPort)` - No ports available
///
/// # Example
///
/// ```ignore
/// let port = allocate_port()?;
/// assert!(port >= 49152 && port <= 65535);
/// ```
pub fn allocate_port() -> Result<u16, TelepipeError> {
    for port in MIN_PORT..=MAX_PORT {
        if is_port_available(port) {
            return Ok(port);
        }
    }

    Err(TelepipeError::AllocPort)
}

/// Allocates three available TCP ports.
///
/// Used by redirect mode for stdin, stdout, and stderr.
/// Returns three distinct ports that were verified available.
///
/// # Returns
///
/// * `Ok((port1, port2, port3))` - Three available ports for stdin, stdout, stderr
/// * `Err(TelepipeError::AllocPort)` - Unable to allocate three ports
///
/// # Note
///
/// The returned ports are not necessarily consecutive, but they are
/// guaranteed to be distinct and in the valid ephemeral range.
pub fn allocate_three_ports() -> Result<(u16, u16, u16), TelepipeError> {
    let mut allocated = Vec::with_capacity(3);

    for port in MIN_PORT..=MAX_PORT {
        if allocated.contains(&port) {
            continue;
        }

        if is_port_available(port) {
            allocated.push(port);
            if allocated.len() == 3 {
                return Ok((allocated[0], allocated[1], allocated[2]));
            }
        }
    }

    Err(TelepipeError::AllocPort)
}

/// Allocates three TCP ports and returns bound listeners.
///
/// This version returns the TcpListeners to prevent race conditions.
/// The caller takes ownership and should keep them bound until ready to use.
///
/// # Returns
///
/// * `Ok((listener1, listener2, listener3))` - Three bound listeners
/// * `Err(TelepipeError::AllocPort)` - Unable to allocate three ports
pub fn allocate_three_ports_with_listeners(
) -> Result<(TcpListener, TcpListener, TcpListener), TelepipeError> {
    let mut listeners = Vec::with_capacity(3);
    let mut ports_used = Vec::with_capacity(3);

    for port in MIN_PORT..=MAX_PORT {
        if ports_used.contains(&port) {
            continue;
        }

        if let Some(listener) = try_bind_port(port) {
            ports_used.push(port);
            listeners.push(listener);
            if listeners.len() == 3 {
                let mut iter = listeners.into_iter();
                return Ok((
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                ));
            }
        }
    }

    Err(TelepipeError::AllocPort)
}

/// Allocates a specific number of TCP ports.
///
/// General-purpose allocation function that returns a vector of ports.
///
/// # Arguments
///
/// * `count` - Number of ports to allocate
///
/// # Returns
///
/// * `Ok(Vec<u16>)` - Vector of allocated ports
/// * `Err(TelepipeError::AllocPort)` - Unable to allocate requested count
pub fn allocate_ports(count: usize) -> Result<Vec<u16>, TelepipeError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut allocated = Vec::with_capacity(count);

    for port in MIN_PORT..=MAX_PORT {
        if allocated.contains(&port) {
            continue;
        }

        if is_port_available(port) {
            allocated.push(port);
            if allocated.len() == count {
                return Ok(allocated);
            }
        }
    }

    Err(TelepipeError::AllocPort)
}

/// Gets the local port from a bound TcpListener.
///
/// Utility function to extract the port number from a listener.
pub fn listener_port(listener: &TcpListener) -> Result<u16, TelepipeError> {
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|_| TelepipeError::AllocPort)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_port_returns_valid_range() {
        let port = allocate_port().expect("Should allocate a port");
        assert!(
            port >= MIN_PORT,
            "Port {} is below MIN_PORT {}",
            port,
            MIN_PORT
        );
        assert!(
            port <= MAX_PORT,
            "Port {} is above MAX_PORT {}",
            port,
            MAX_PORT
        );
    }

    #[test]
    fn test_allocate_three_ports_returns_distinct() {
        let (port1, port2, port3) = allocate_three_ports().expect("Should allocate three ports");

        // All three must be distinct
        assert_ne!(port1, port2, "port1 and port2 must be distinct");
        assert_ne!(port2, port3, "port2 and port3 must be distinct");
        assert_ne!(port1, port3, "port1 and port3 must be distinct");

        // All must be in valid range
        assert!(port1 >= MIN_PORT && port1 <= MAX_PORT);
        assert!(port2 >= MIN_PORT && port2 <= MAX_PORT);
        assert!(port3 >= MIN_PORT && port3 <= MAX_PORT);
    }

    #[test]
    fn test_allocate_ports_zero_count() {
        let result = allocate_ports(0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_allocate_ports_one() {
        let ports = allocate_ports(1).expect("Should allocate one port");
        assert_eq!(ports.len(), 1);
        assert!(ports[0] >= MIN_PORT && ports[0] <= MAX_PORT);
    }

    #[test]
    fn test_allocate_ports_multiple_distinct() {
        let ports = allocate_ports(5).expect("Should allocate five ports");
        assert_eq!(ports.len(), 5);

        // All must be distinct
        let mut seen = std::collections::HashSet::new();
        for port in &ports {
            assert!(seen.insert(*port), "Duplicate port allocated: {}", port);
        }

        // All must be in valid range
        for port in &ports {
            assert!(*port >= MIN_PORT && *port <= MAX_PORT);
        }
    }

    #[test]
    fn test_port_range_constants() {
        assert_eq!(MIN_PORT, 49152);
        assert_eq!(MAX_PORT, 65535);
    }

    #[test]
    fn test_is_port_available_low_port() {
        // Port 80 (HTTP) is likely in use or requires privileges
        // We just verify the function doesn't panic
        let _ = is_port_available(80);
    }

    #[test]
    fn test_allocate_three_ports_with_listeners() {
        let result = allocate_three_ports_with_listeners();
        assert!(result.is_ok(), "Should allocate three listeners");

        let (l1, l2, l3) = result.unwrap();

        // Get ports and verify they're distinct
        let p1 = listener_port(&l1).unwrap();
        let p2 = listener_port(&l2).unwrap();
        let p3 = listener_port(&l3).unwrap();

        assert_ne!(p1, p2);
        assert_ne!(p2, p3);
        assert_ne!(p1, p3);

        // All in valid range
        assert!(p1 >= MIN_PORT && p1 <= MAX_PORT);
        assert!(p2 >= MIN_PORT && p2 <= MAX_PORT);
        assert!(p3 >= MIN_PORT && p3 <= MAX_PORT);
    }

    #[test]
    fn test_listener_port() {
        let listener =
            TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("Should bind");
        let port = listener_port(&listener).expect("Should get port");
        // Port 0 means OS assigns, so it should be > 0
        assert!(port > 0);
    }

    #[test]
    fn test_ephemeral_range_size() {
        // IANA ephemeral range should have plenty of ports
        let range_size = (MAX_PORT - MIN_PORT + 1) as u32;
        assert!(
            range_size > 16000,
            "Ephemeral range should have at least 16000 ports"
        );
    }
}
