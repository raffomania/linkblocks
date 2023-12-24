use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use clap::{Args, Parser, Subcommand};

use crate::{db, server};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Migrate the database, then start the server
    Start {
        #[clap(flatten)]
        listen: ListenArgs,
        #[clap(env, long, hide_env_values = true)]
        database_url: String,
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
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "linkblocks=debug,tower_http=debug,axum::rejection=trace",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            listen: listen_address,
            database_url,
        } => {
            let pool = db::pool(&database_url).await?;
            db::migrate(&pool).await?;
            server::start(listen_address, pool).await
        }
    }
}
