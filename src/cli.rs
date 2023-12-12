use std::net::SocketAddr;

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
        /// Format: `ip:port`. If omitted, try to obtain a port via the listenfd interface.
        listen: Option<SocketAddr>,
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
