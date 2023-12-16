use anyhow::Result;
use std::net::SocketAddr;

use clap::{Args, Parser, Subcommand};

use crate::server;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start {
        #[clap(flatten)]
        listen: ListenArgs,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ListenArgs {
    #[clap(long, value_name = "SOCKET_ADDRESS")]
    /// Format: `ip:port`. If omitted, try to obtain a port via the listenfd interface.
    pub listen: Option<SocketAddr>,
    #[clap(long)]
    /// Take a socket using the systemd socket passing protocol and listen on it.
    pub listenfd: bool,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            listen: listen_address,
        } => server::start(listen_address).await,
    }
}
