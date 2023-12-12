mod cli;
mod server;

#[tokio::main]
async fn main() {
    cli::run().await
}
