//! Telepipe CLI Main Entry Point
//!
//! This is the main entry point for the Telepipe CLI application.
//! It simply calls cli::run() and exits with the returned code.

mod cli;

fn main() {
    let exit_code = cli::run();
    std::process::exit(exit_code);
}
