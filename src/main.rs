mod cli;
mod start;

#[tokio::main]
async fn main() {
    cli::run().await
}
