use std::{path::PathBuf, time::Duration};

use anyhow::anyhow;
use axum_server::tls_rustls::RustlsConfig;
use tower_sessions::ExpiredDeletion;

use crate::{
    cli::ListenArgs,
    routes::{self},
};

use axum::Router;
use listenfd::ListenFd;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub base_url: String,
}

pub async fn app(state: AppState) -> anyhow::Result<Router> {
    let session_store = tower_sessions::PostgresStore::new(state.pool.clone());
    session_store.migrate().await?;
    tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(6 * 60 * 60)),
    );

    let session_service = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(tower_sessions::Expiry::OnInactivity(
            tower_sessions::cookie::time::Duration::weeks(2),
        ));

    Ok(Router::new()
        .merge(routes::users::router())
        .merge(routes::index::router())
        .merge(routes::notes::router())
        .merge(routes::bookmarks::router())
        .merge(routes::links::router())
        .merge(routes::assets::router().with_state(()))
        .layer(TraceLayer::new_for_http())
        .layer(session_service)
        .with_state(state))
}

pub async fn start(
    listen: ListenArgs,
    app: Router,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
) -> anyhow::Result<()> {
    let handle = axum_server::Handle::new();

    let listener = if let Some(listen_address) = listen.listen {
        tokio::spawn(shutdown_signal(handle.clone(), true));
        tokio::net::TcpListener::bind(format!("{listen_address}")).await?
    } else {
        // Graceful shutdown is somehow broken with listenfd at the moment
        tokio::spawn(shutdown_signal(handle.clone(), false));
        let mut listenfd = ListenFd::from_env();
        let listener = listenfd
            .take_tcp_listener(0)?
            .ok_or(anyhow!("No systemfd TCP socket found"))?;
        tokio::net::TcpListener::from_std(listener)?
    };

    let listening_on = listener.local_addr()?;

    match (tls_cert, tls_key) {
        (Some(cert), Some(key)) => {
            tracing::info!("Using TLS files at: {cert:?}, {key:?}");
            let config = RustlsConfig::from_pem_file(cert, key).await?;
            tracing::info!("Listening on https://{listening_on}");
            axum_server::from_tcp_rustls(listener.into_std()?, config)
                .handle(handle)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            tracing::info!("No TLS certificate specified, not using TLS");
            tracing::info!("Listening on http://{listening_on}");
            axum_server::from_tcp(listener.into_std()?)
                .handle(handle)
                .serve(app.into_make_service())
                .await?;
        }
    };

    Ok(())
}

async fn shutdown_signal(handle: axum_server::Handle, graceful: bool) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!(
        "Received termination signal - waiting 10 seconds to close existing connections"
    );
    if graceful {
        handle.graceful_shutdown(Some(Duration::from_secs(10)));
    } else {
        handle.shutdown();
    }
}
