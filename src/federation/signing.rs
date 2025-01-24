use activitypub_federation::http_signatures::{Keypair, generate_actor_keypair};
use anyhow::Result;

/// Use a single static keypair during testing which is signficantly faster than
/// generating dozens of keys from scratch.
#[cfg(debug_assertions)]
#[allow(clippy::unnecessary_wraps)]
pub fn generate_keypair() -> Result<Keypair> {
    use std::sync::LazyLock;

    #[allow(clippy::expect_used)]
    static KEYPAIR: LazyLock<Keypair> =
        LazyLock::new(|| generate_actor_keypair().expect("generate keypair"));

    Ok(KEYPAIR.clone())
}

#[cfg(not(debug_assertions))]
pub fn generate_keypair() -> Result<Keypair> {
    Ok(generate_actor_keypair()?)
}
