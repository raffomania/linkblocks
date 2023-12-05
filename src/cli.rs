use clap::{Parser, Subcommand};

use crate::start;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start { host: String, port: u64 },
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Start { host, port } => start::start(host, port).await,
    };
}
