use anyhow::anyhow;

use crate::{app_error::Result, cli::ListenArgs};
use askama::Template;
use axum::{debug_handler, routing::get, Router};
use listenfd::ListenFd;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn start(listen: ListenArgs) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "linkblocks=debug,tower_http=debug,axum::rejection=trace",
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let app = Router::new()
        .route("/", get(hello))
        .route("/htmx-fragment", get(htmx_fragment))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let listener = if let Some(listen_address) = listen.listen {
        tokio::net::TcpListener::bind(format!("{listen_address}")).await?
    } else {
        let mut listenfd = ListenFd::from_env();
        let listener = listenfd
            .take_tcp_listener(0)?
            .ok_or(anyhow!("No systemfd TCP socket found"))?;
        tokio::net::TcpListener::from_std(listener)?
    };

    axum::serve(listener, app).await?;

    Ok(())
}

#[debug_handler]
async fn hello() -> Result<HelloTemplate> {
    Ok(HelloTemplate {})
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {}

#[debug_handler]
async fn htmx_fragment() -> &'static str {
    "Here's some dynamically loaded content!"
}
