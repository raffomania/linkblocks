use anyhow::anyhow;

use crate::{
    cli::ListenArgs,
    routes::{self, index},
};

use axum::{routing::get, Router};
use listenfd::ListenFd;
use tower_http::trace::TraceLayer;

pub fn app(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route("/", get(index::index))
        .route(
            "/assets/railwind.css",
            get(routes::assets::railwind_generated_css),
        )
        .route("/assets/*path", get(routes::assets::assets))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
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

    axum::serve(listener, app);

    Ok(())
}
