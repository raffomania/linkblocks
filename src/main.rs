use anyhow::Result;

mod app_error;
mod cli;
mod routes;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}
