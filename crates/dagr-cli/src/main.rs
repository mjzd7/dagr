use clap::Parser;
use dagr_cli::{execute_cli, Cli};
use dagr_core::DagrError;
use std::io::ErrorKind;

#[cfg(unix)]
fn restore_sigpipe_default() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn restore_sigpipe_default() {}

#[tokio::main]
async fn main() {
    restore_sigpipe_default();
    let cli = Cli::parse();
    if let Err(e) = execute_cli(cli).await {
        // Belt-and-braces for writers that return Io errors instead of dying
        // on SIGPIPE (e.g. the MCP stdio loop).
        if let DagrError::Io(io) = &e {
            if io.kind() == ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
        }
        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
