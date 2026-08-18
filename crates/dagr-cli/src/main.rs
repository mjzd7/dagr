use clap::Parser;
use dagr_cli::{execute_cli, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = execute_cli(cli).await {
        eprintln!("\x1b[1;31mError:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
