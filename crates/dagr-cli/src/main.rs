use clap::Parser;
use dagr_cli::{execute_cli, Cli};

fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute_cli(cli) {
        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
