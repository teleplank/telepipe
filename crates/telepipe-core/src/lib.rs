//! Telepipe Core Library
//!
//! This crate implements the core Telepipe functionality as defined in
//! the Telepipe 0.3.0 specification.

pub mod config;
pub mod connect_mode;
pub mod disconnect;
pub mod errors;
pub mod exec_mode;
pub mod fds;
pub mod info;
pub mod locking;
pub mod process_spawn;
pub mod recovery;
pub mod redirect_mode;
pub mod registry;
pub mod router;
pub mod session;
pub mod signals;
pub mod stop;
pub mod tcp_alloc;

// Re-export commonly used types
pub use errors::{TelepipeError, Result};
pub use config::{
    MIN_FD, MAX_FD, RESERVED_FD,
    MIN_PORT, MAX_PORT,
    DEFAULT_HOST,
    session_dir, session_file,
    is_valid_fd, is_valid_port,
};
pub use fds::{allocate_fd, allocate_three_fds, allocate_fds};
pub use tcp_alloc::{allocate_port, allocate_three_ports, allocate_ports};
pub use registry::{
    load_session, write_session, delete_session,
    list_sessions, load_all_sessions, session_exists,
    ensure_session_dir,
};
pub use process_spawn::{
    spawn_with_redirected_stdio, spawn_child, spawn_silent,
    is_process_running, terminate_process, kill_process,
    wait_for_process, current_pid,
};
pub use session::{SessionEntry, SessionMode};
pub use redirect_mode::{redirect, redirect_and_wait, get_redirect_info, RedirectInfo};
pub use router::{
    run_redirect_router, run_connect_router, run_simple_router,
    forward_data, spawn_forwarder,
    ConnectionState, RedirectConnections,
};
pub use exec_mode::{exec, exec_with_exit_code, get_exec_info, ExecInfo};
pub use connect_mode::{connect, connect_and_wait, get_connect_info, ConnectInfo};
pub use info::{info, info_to_string, parse_field, field_names};
pub use stop::{stop, stop_with_exit_code, kill_child, force_kill, check_status};
pub use disconnect::{disconnect, disconnect_with_exit_code, check_supervisor_status};
pub use signals::{
    setup_signal_handlers, register_child, unregister_child,
    signal_received, clear_signal, get_signal_number,
    was_sigint, was_sigterm, graceful_shutdown, signal_name,
};
pub use recovery::{recover, check_recovery, RecoveryResult};
pub use locking::{
    acquire_global_lock, acquire_session_lock, acquire_portalloc_lock,
    try_acquire_global_lock, try_acquire_session_lock, try_acquire_portalloc_lock,
    with_global_lock, with_session_lock, with_portalloc_lock,
    cleanup_stale_locks, GlobalLock, SessionLock, PortAllocLock,
};
