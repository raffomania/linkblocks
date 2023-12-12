watch: 
    systemfd --no-pid -s http::4040 -- cargo watch -- cargo run start

ci-dev: 
    cargo build
    cargo test
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
