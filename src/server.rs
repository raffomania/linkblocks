use axum::{debug_handler, routing::get, Router};
use listenfd::ListenFd;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn start(host: String, port: u64) {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "linkblocks=debug,tower_http=debug,axum::rejection=trace",
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let app = Router::new()
        .route("/", get(hello))
        .layer(TraceLayer::new_for_http());

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => tokio::net::TcpListener::from_std(listener).unwrap(),
        None => tokio::net::TcpListener::bind(format!("{host}:{port}"))
            .await
            .unwrap(),
    };

    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn hello() -> &'static str {
    "Hello, Web!"
}
