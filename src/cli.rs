use clap::{Parser, Subcommand};

use crate::server;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start {
        #[clap(long)]
        listen: Option<String>,
    },
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            listen: listen_address,
        } => server::start(listen_address).await,
    };
}
