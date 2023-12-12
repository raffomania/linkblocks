use std::net::SocketAddr;

use askama::Template;
use axum::{debug_handler, routing::get, Router};
use listenfd::ListenFd;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn start(listen_address: Option<SocketAddr>) {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "linkblocks=debug,tower_http=debug,axum::rejection=trace",
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let app = Router::new()
        .route("/", get(hello))
        .layer(TraceLayer::new_for_http());

    let listener = if let Some(listen_address) = listen_address {
        tokio::net::TcpListener::bind(format!("{listen_address}"))
            .await
            .unwrap()
    } else {
        let mut listenfd = ListenFd::from_env();
        let listener = listenfd.take_tcp_listener(0).unwrap().unwrap();
        tokio::net::TcpListener::from_std(listener).unwrap()
    };

    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn hello() -> HelloTemplate {
    HelloTemplate {}
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {}
