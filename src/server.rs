use anyhow::anyhow;
use tower_sessions::ExpiredDeletion;

use crate::{
    cli::ListenArgs,
    routes::{self, index},
};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use listenfd::ListenFd;
use tower_http::trace::TraceLayer;

pub async fn app(pool: sqlx::PgPool) -> anyhow::Result<Router> {
    let session_store = tower_sessions::PostgresStore::new(pool.clone());
    session_store.migrate().await?;
    tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(3600)),
    );

    let session_service = tower::ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: axum::BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            tower_sessions::SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(tower_sessions::Expiry::OnInactivity(
                    tower_sessions::cookie::time::Duration::weeks(2),
                )),
        );

    Ok(Router::new()
        .route("/", get(index::index))
        .route(
            "/assets/railwind.css",
            get(routes::assets::railwind_generated_css),
        )
        .route("/assets/*path", get(routes::assets::assets))
        .route("/login", post(routes::users::post_login))
        .route("/login", get(routes::users::get_login))
        .layer(TraceLayer::new_for_http())
        .layer(session_service)
        .with_state(pool))
}

pub async fn start(listen: ListenArgs, app: Router) -> anyhow::Result<()> {
    let listener = if let Some(listen_address) = listen.listen {
        tokio::net::TcpListener::bind(format!("{listen_address}")).await?
    } else {
        let mut listenfd = ListenFd::from_env();
        let listener = listenfd
            .take_tcp_listener(0)?
            .ok_or(anyhow!("No systemfd TCP socket found"))?;
        tokio::net::TcpListener::from_std(listener)?
    };

    let listening_on = listener.local_addr()?;
    tracing::info!("Listening on http://{listening_on}");

    axum::serve(listener, app).await?;

    Ok(())
}
