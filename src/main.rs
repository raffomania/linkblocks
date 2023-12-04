use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "linkblocks=debug,tower_http=debug,axum::rejection=trace",
        ))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let app = Router::new()
        .route("/", get(hello))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("localhost:4040")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, Web!"
}
