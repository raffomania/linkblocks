watch: 
    systemfd --no-pid -s http::4040 -- cargo watch -- cargo run start --listenfd

run *args:
    cargo run -- {{args}}

ci-dev: 
    cargo build
    cargo test
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
