use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use clap::{Args, Parser, Subcommand};

use crate::{db, schemas::users::CreateUser, server};

#[derive(Parser)]
#[clap(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[clap(flatten)]
    config: SharedConfig,
}

#[derive(Args)]
struct SharedConfig {
    #[clap(env, long, hide_env_values = true)]
    database_url: String,
}

#[derive(Parser)]
enum Command {
    /// Migrate the database, then start the server
    Start {
        #[clap(flatten)]
        listen: ListenArgs,
        #[clap(flatten)]
        admin_credentials: AdminCredentials,
    },
    Db {
        #[clap(subcommand)]
        command: DbCommand,
    },
}

#[derive(Args)]
#[group(multiple = true, requires_all = ["username", "password"])]
struct AdminCredentials {
    #[clap(env = "ADMIN_USERNAME", long = "admin_username")]
    /// Create an admin user if it does not exist yet.
    username: Option<String>,
    #[clap(
        env = "ADMIN_PASSWORD",
        long = "admin_password",
        hide_env_values = true
    )]
    /// Password for the admin user.
    password: Option<String>,
}

#[derive(Subcommand)]
enum DbCommand {
    Migrate,
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
            admin_credentials,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;

            db::migrate(&pool).await?;

            if let (Some(username), Some(password)) =
                (admin_credentials.username, admin_credentials.password)
            {
                let mut tx = pool.begin().await?;
                db::users::create_user_if_not_exists(&mut tx, CreateUser { password, username })
                    .await
                    .unwrap();
                tx.commit().await?;
            }

            let app = server::app(pool).await?;
            server::start(listen_address, app).await
        }
        Command::Db {
            command: DbCommand::Migrate,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;
            db::migrate(&pool).await
        }
    }
}
