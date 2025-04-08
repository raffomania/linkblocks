use anyhow::{anyhow, Result};
use garde::Validate;
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use clap::{Args, Parser, Subcommand};

#[cfg(debug_assertions)]
use crate::insert_demo_data::insert_demo_data;
use crate::{
    db,
    forms::users::CreateUser,
    oidc,
    server::{self, AppState},
};

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

#[derive(Args, Debug)]
#[group(requires_all = ["oidc_client_id", "oidc_client_secret", "oidc_issuer_url", "oidc_issuer_name"])]
pub struct OidcArgs {
    /// To use OIDC, all options beginning with `oidc` must be set.
    /// We support RS*, PS*, or HS* signature algorithms.
    /// Configure your redirect URL to be `{base_url}/login_oidc_redirect`.
    #[clap(long, env, required = false)]
    pub oidc_client_id: String,
    #[clap(hide_env_values = true, long, env, required = false)]
    pub oidc_client_secret: String,
    #[clap(long, env, required = false)]
    pub oidc_issuer_url: String,
    /// This will be displayed on the login page.
    #[clap(long, env, required = false)]
    pub oidc_issuer_name: String,
}

// Since this enum is only ever constructed once,
// we only waste very little memory due to large enum variants.
#[expect(clippy::large_enum_variant)]
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
        #[clap(long, env)]
        base_url: String,
        #[clap(long, env, default_value = "false")]
        demo_mode: bool,
        #[clap(flatten)]
        oidc_args: Option<OidcArgs>,
    },
    Db {
        #[clap(subcommand)]
        command: DbCommand,
    },
    #[cfg(debug_assertions)]
    /// Put some demo data into the database
    InsertDemoData {
        #[clap(flatten)]
        dev_user_credentials: AdminCredentials,
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

impl From<AdminCredentials> for Option<CreateUser> {
    fn from(value: AdminCredentials) -> Self {
        if let (Some(username), Some(password)) = (value.username, value.password) {
            Some(CreateUser { username, password })
        } else {
            None
        }
    }
}

#[derive(Subcommand)]
enum DbCommand {
    Migrate,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct ListenArgs {
    /// Format: `ip:port`. If omitted, try to obtain a port via the listenfd
    /// interface.
    #[clap(long, env, value_name = "SOCKET_ADDRESS")]
    pub listen: Option<SocketAddr>,
    /// Take a socket using the systemd socket passing protocol and listen on
    /// it.
    #[clap(long, env)]
    pub listenfd: bool,
}

pub async fn run() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("Failed to set default crypto provider"))?;

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            listen: listen_address,
            admin_credentials,
            tls_cert,
            tls_key,
            base_url,
            demo_mode,
            oidc_args,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;

            db::migrate(&pool).await?;

            if let Some(create) = Option::<CreateUser>::from(admin_credentials) {
                if let Err(e) = create.validate() {
                    return Err(anyhow!("Invalid credentials for admin user provided:\n{e}"));
                }
                let mut tx = pool.begin().await?;
                db::users::create_user_if_not_exists(&mut tx, create).await?;
                tx.commit().await?;
            }

            let oidc_state = oidc::State::initialize(base_url.clone(), oidc_args).await;

            let app = server::app(AppState {
                pool,
                base_url: base_url.clone(),
                demo_mode,
                oidc_state,
            })
            .await?;
            server::start(listen_address, app, tls_cert, tls_key).await?;
        }
        Command::Db {
            command: DbCommand::Migrate,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;
            db::migrate(&pool).await?;
        }
        #[cfg(debug_assertions)]
        Command::InsertDemoData {
            dev_user_credentials,
        } => {
            let pool = db::pool(&cli.config.database_url).await?;
            insert_demo_data(&pool, Option::<CreateUser>::from(dev_user_credentials)).await?;
        }
    };

    Ok(())
}
