use anyhow::{anyhow, Result};
use garde::Validate;
use std::{net::SocketAddr, path::PathBuf};
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
        /// TLS cert location.
        /// If set, requires `tls-key` to be set as well.
        /// If both `tls-key` and `tls-cert` are unset, no TLS is used.
        #[clap(long, env, requires = "tls_key")]
        tls_cert: Option<PathBuf>,
        /// TLS key location.
        /// If set, requires `tls-cert` to be set as well.
        /// If both `tls-key` and `tls-cert` are unset, no TLS is used.
        #[clap(long, env, requires = "tls_cert")]
        tls_key: Option<PathBuf>,
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
    /// Format: `ip:port`. If omitted, try to obtain a port via the listenfd interface.
    #[clap(long, value_name = "SOCKET_ADDRESS")]
    pub listen: Option<SocketAddr>,
    /// Take a socket using the systemd socket passing protocol and listen on it.
    #[clap(long)]
    pub listenfd: bool,
}

pub async fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            listen: listen_address,
            admin_credentials,
            tls_cert,
            tls_key,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;

            db::migrate(&pool).await?;

            if let (Some(username), Some(password)) =
                (admin_credentials.username, admin_credentials.password)
            {
                let create = CreateUser { password, username };
                if let Err(e) = create.validate(&()) {
                    return Err(anyhow!("Invalid credentials for admin user provided:\n{e}"));
                }
                let mut tx = pool.begin().await?;
                db::users::create_user_if_not_exists(&mut tx, create)
                    .await
                    .unwrap();
                tx.commit().await?;
            }

            let app = server::app(pool).await?;
            server::start(listen_address, app, tls_cert, tls_key).await?;
        }
        Command::Db {
            command: DbCommand::Migrate,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;
            db::migrate(&pool).await?;
        }
    };

    Ok(())
}
