use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    linkblocks::cli::run().await
}
