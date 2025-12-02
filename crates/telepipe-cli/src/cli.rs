//! CLI Entry Point Implementation
//!
//! This module implements the Telepipe CLI as specified in:
//! - 515 the telepipe cli reference.md
//! - 560 the telepipe implementation blueprint.md (Section 4.1: cli.rs)
//!
//! The CLI:
//! - Parses command-line arguments
//! - Dispatches to appropriate module
//! - Handles errors and exit codes
//! - Never performs logic directly

use std::env;
use std::io::{self, Write};

use telepipe_core::{
    redirect_mode, exec_mode, connect_mode,
    info, stop, disconnect,
    setup_signal_handlers,
};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Program name for help and error messages
const PROGRAM_NAME: &str = "telepipe";

/// Version string
const VERSION: &str = "0.3.0";

// =============================================================================
// CLI ENTRY POINT
// =============================================================================

/// Main CLI entry point.
///
/// Parses arguments and dispatches to appropriate command handler.
///
/// # Returns
///
/// Exit code (0 for success, non-zero for errors)
pub fn run() -> i32 {
    // Set up signal handlers for graceful shutdown
    setup_signal_handlers();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return 2; // Invalid arguments
    }

    let command = &args[1];

    match command.as_str() {
        "redirect" => handle_redirect(&args[2..]),
        "exec" => handle_exec(&args[2..]),
        "connect" => handle_connect(&args[2..]),
        "stop" => handle_stop(&args[2..]),
        "disconnect" => handle_disconnect(&args[2..]),
        "info" => handle_info(&args[2..]),
        "--help" | "-h" | "help" => {
            print_usage();
            0
        }
        "--version" | "-V" | "version" => {
            println!("{} {}", PROGRAM_NAME, VERSION);
            0
        }
        _ => {
            print_error(&format!("unknown command: {}", command));
            print_usage();
            2
        }
    }
}

// =============================================================================
// COMMAND HANDLERS
// =============================================================================

/// Handles the `redirect` command.
fn handle_redirect(args: &[String]) -> i32 {
    let parsed = match parse_redirect_args(args) {
        Ok(p) => p,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match redirect_mode::redirect(&parsed.id, &parsed.command, &parsed.args) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

/// Handles the `exec` command.
fn handle_exec(args: &[String]) -> i32 {
    let id = match parse_id_arg(args) {
        Ok(id) => id,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match exec_mode::exec(&id) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

/// Handles the `connect` command.
fn handle_connect(args: &[String]) -> i32 {
    let parsed = match parse_connect_args(args) {
        Ok(p) => p,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match connect_mode::connect(&parsed.id, &parsed.host, parsed.port) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

/// Handles the `stop` command.
fn handle_stop(args: &[String]) -> i32 {
    let id = match parse_id_arg(args) {
        Ok(id) => id,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match stop::stop(&id) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

/// Handles the `disconnect` command.
fn handle_disconnect(args: &[String]) -> i32 {
    let id = match parse_id_arg(args) {
        Ok(id) => id,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match disconnect::disconnect(&id) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

/// Handles the `info` command.
fn handle_info(args: &[String]) -> i32 {
    let id = match parse_id_arg(args) {
        Ok(id) => id,
        Err(e) => {
            print_error(&e);
            return 71; // E-CLI-ARGS
        }
    };

    match info::info(&id) {
        Ok(()) => 0,
        Err(e) => {
            print_error(&e.to_string());
            e.exit_code()
        }
    }
}

// =============================================================================
// ARGUMENT PARSING
// =============================================================================

/// Parsed redirect arguments.
#[derive(Debug)]
struct RedirectArgs {
    id: String,
    command: String,
    args: Vec<String>,
}

/// Parses redirect command arguments.
fn parse_redirect_args(args: &[String]) -> Result<RedirectArgs, String> {
    let mut id: Option<String> = None;
    let mut command: Option<String> = None;
    let mut cmd_args: Vec<String> = Vec::new();
    let mut in_command = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if in_command {
            // Everything after -- is part of the command
            if command.is_none() {
                command = Some(arg.clone());
            } else {
                cmd_args.push(arg.clone());
            }
            i += 1;
            continue;
        }

        match arg.as_str() {
            "--id" => {
                i += 1;
                if i >= args.len() {
                    return Err("--id requires a value".to_string());
                }
                id = Some(args[i].clone());
            }
            "--" => {
                in_command = true;
            }
            _ if arg.starts_with("--id=") => {
                id = Some(arg[5..].to_string());
            }
            _ => {
                return Err(format!("unexpected argument: {}", arg));
            }
        }
        i += 1;
    }

    let id = id.ok_or("--id is required")?;
    let command = command.ok_or("command is required after --")?;

    Ok(RedirectArgs { id, command, args: cmd_args })
}

/// Parsed connect arguments.
#[derive(Debug)]
struct ConnectArgs {
    id: String,
    host: String,
    port: u16,
}

/// Parses connect command arguments.
fn parse_connect_args(args: &[String]) -> Result<ConnectArgs, String> {
    let mut id: Option<String> = None;
    let mut host: String = "127.0.0.1".to_string();
    let mut port: Option<u16> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--id" => {
                i += 1;
                if i >= args.len() {
                    return Err("--id requires a value".to_string());
                }
                id = Some(args[i].clone());
            }
            "--port" => {
                i += 1;
                if i >= args.len() {
                    return Err("--port requires a value".to_string());
                }
                port = Some(args[i].parse().map_err(|_| "invalid port number")?);
            }
            "--host" => {
                i += 1;
                if i >= args.len() {
                    return Err("--host requires a value".to_string());
                }
                host = args[i].clone();
            }
            "--no-reconnect" => {
                // TODO: Handle reconnect flag when supported
            }
            _ if arg.starts_with("--id=") => {
                id = Some(arg[5..].to_string());
            }
            _ if arg.starts_with("--port=") => {
                port = Some(arg[7..].parse().map_err(|_| "invalid port number")?);
            }
            _ if arg.starts_with("--host=") => {
                host = arg[7..].to_string();
            }
            _ => {
                return Err(format!("unexpected argument: {}", arg));
            }
        }
        i += 1;
    }

    let id = id.ok_or("--id is required")?;
    let port = port.ok_or("--port is required")?;

    Ok(ConnectArgs { id, host, port })
}

/// Parses arguments that only need --id.
fn parse_id_arg(args: &[String]) -> Result<String, String> {
    let mut id: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--id" => {
                i += 1;
                if i >= args.len() {
                    return Err("--id requires a value".to_string());
                }
                id = Some(args[i].clone());
            }
            _ if arg.starts_with("--id=") => {
                id = Some(arg[5..].to_string());
            }
            _ => {
                return Err(format!("unexpected argument: {}", arg));
            }
        }
        i += 1;
    }

    id.ok_or("--id is required".to_string())
}

// =============================================================================
// OUTPUT HELPERS
// =============================================================================

/// Prints an error message to stderr.
///
/// Format: `[telepipe] error: <message>`
fn print_error(msg: &str) {
    let _ = writeln!(io::stderr(), "[{}] error: {}", PROGRAM_NAME, msg);
}

/// Prints usage information.
fn print_usage() {
    println!(
        r#"Usage: {} <command> [options]

Core Operations:
  redirect  Spawn a process with redirected stdio to TCP sockets
  exec      Pipe input to and stream output from a session
  connect   Attach to an existing external TCP service

Lifecycle Operations:
  stop        Terminate a redirect-mode child process
  disconnect  Terminate a connect-mode supervisor
  info        Print session state and exit immediately

Options:
  -h, --help     Print this help message
  -V, --version  Print version information

Examples:
  {} redirect --id psql -- psql
  {} exec --id psql
  {} connect --id chrome --port 9222
  {} info --id psql
  {} stop --id psql
  {} disconnect --id chrome

For more information, see: https://github.com/teleplank/telepipe"#,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
        PROGRAM_NAME,
    );
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id_arg_valid() {
        let args = vec!["--id".to_string(), "test-session".to_string()];
        let result = parse_id_arg(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-session");
    }

    #[test]
    fn test_parse_id_arg_equals() {
        let args = vec!["--id=my-session".to_string()];
        let result = parse_id_arg(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my-session");
    }

    #[test]
    fn test_parse_id_arg_missing() {
        let args: Vec<String> = vec![];
        let result = parse_id_arg(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--id is required"));
    }

    #[test]
    fn test_parse_id_arg_no_value() {
        let args = vec!["--id".to_string()];
        let result = parse_id_arg(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a value"));
    }

    #[test]
    fn test_parse_redirect_args_valid() {
        let args = vec![
            "--id".to_string(),
            "psql".to_string(),
            "--".to_string(),
            "psql".to_string(),
            "-h".to_string(),
            "localhost".to_string(),
        ];
        let result = parse_redirect_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "psql");
        assert_eq!(parsed.command, "psql");
        assert_eq!(parsed.args, vec!["-h", "localhost"]);
    }

    #[test]
    fn test_parse_redirect_args_missing_id() {
        let args = vec!["--".to_string(), "echo".to_string()];
        let result = parse_redirect_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--id is required"));
    }

    #[test]
    fn test_parse_redirect_args_missing_command() {
        let args = vec!["--id".to_string(), "test".to_string()];
        let result = parse_redirect_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("command is required"));
    }

    #[test]
    fn test_parse_connect_args_valid() {
        let args = vec![
            "--id".to_string(),
            "chrome".to_string(),
            "--port".to_string(),
            "9222".to_string(),
        ];
        let result = parse_connect_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.id, "chrome");
        assert_eq!(parsed.port, 9222);
        assert_eq!(parsed.host, "127.0.0.1");
    }

    #[test]
    fn test_parse_connect_args_with_host() {
        let args = vec![
            "--id".to_string(),
            "chrome".to_string(),
            "--port".to_string(),
            "9222".to_string(),
            "--host".to_string(),
            "192.168.1.1".to_string(),
        ];
        let result = parse_connect_args(&args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.host, "192.168.1.1");
    }

    #[test]
    fn test_parse_connect_args_missing_port() {
        let args = vec!["--id".to_string(), "chrome".to_string()];
        let result = parse_connect_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--port is required"));
    }

    #[test]
    fn test_parse_connect_args_invalid_port() {
        let args = vec![
            "--id".to_string(),
            "chrome".to_string(),
            "--port".to_string(),
            "invalid".to_string(),
        ];
        let result = parse_connect_args(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid port"));
    }
}
