set dotenv-load := true
set export := true

watch *args: development-cert start-database
    cargo bin systemfd --no-pid -s http::4040 -- cargo bin cargo-watch -- cargo run start --listenfd {{args}}

run *args: development-cert
    cargo run -- {{args}}

insert-demo-data: migrate-database
    RUST_LOG=error cargo run -- insert-demo-data

start-database:
    #!/usr/bin/env bash
    set -euxo pipefail

    if podman ps --format "{{{{.Names}}" | grep -wq linkblocks_postgres; then
        echo "Database is running."
        exit
    fi

    if ! podman inspect linkblocks_postgres &> /dev/null; then
        podman create \
            --name linkblocks_postgres \
            -e POSTGRES_HOST_AUTH_METHOD=trust -e POSTGRES_DB=${DATABASE_NAME} \
            -p ${DATABASE_PORT}:5432 docker.io/postgres:15 \
            postgres
    fi

    podman start linkblocks_postgres

    for i in {1..20}; do
        pg_isready -h localhost -p $DATABASE_PORT && break
        sleep 2
    done

start-rauthy:
    #!/usr/bin/env bash
    set -euxo pipefail

    # TODO: extract helpers for repetitive podman tasks.
    if podman ps --format "{{{{.Names}}" | grep -wq linkblocks_rauthy; then
        echo "Rauthy is running."
        exit
    fi

    if ! podman inspect linkblocks_rauthy &> /dev/null; then
        podman create \
            --replace --name linkblocks_rauthy \
            -e COOKIE_MODE=danger-insecure \
            -e PUB_URL=localhost:${RAUTHY_PORT} \
            -e LOG_LEVEL=info \
            -e BOOTSTRAP_ADMIN_PASSWORD_PLAIN="test" \
            -e DATABASE_URL=sqlite:data/rauthy.db \
            -p ${RAUTHY_PORT}:8080 \
            ghcr.io/sebadob/rauthy:0.25.0-lite
    fi

    podman start linkblocks_rauthy

stop-rauthy:
    podman stop linkblocks_rauthy

wipe-rauthy: stop-rauthy
    podman rm linkblocks_rauthy

stop-database:
    podman stop linkblocks_postgres

wipe-database: stop-database && migrate-database
    podman rm linkblocks_postgres

migrate-database: start-database
    cargo bin sqlx-cli migrate run

generate-database-info: start-database migrate-database
    cargo bin sqlx-cli prepare

start-test-database:
    #!/usr/bin/env bash
    set -euxo pipefail

    if podman ps --format "{{{{.Names}}" | grep -wq linkblocks_postgres_test; then
        echo "Test database is running."
        exit
    fi

    if ! podman inspect linkblocks_postgres_test &> /dev/null; then
        podman create \
            --replace --name linkblocks_postgres_test --image-volume tmpfs \
            --health-cmd pg_isready --health-interval 10s \
            -e POSTGRES_HOST_AUTH_METHOD=trust -e POSTGRES_DB=${DATABASE_NAME_TEST} \
            -p ${DATABASE_PORT_TEST}:5432 --rm docker.io/postgres:16 \
            postgres \
            -c fsync=off \
            -c synchronous_commit=off \
            -c full_page_writes=off \
            -c autovacuum=off
    fi

    podman start linkblocks_postgres_test

    for i in {1..20}; do
        pg_isready -h localhost -p $DATABASE_PORT_TEST && break
        sleep 2
    done

test *args: start-test-database
    RUST_BACKTRACE=1 DATABASE_URL=${DATABASE_URL_TEST} SQLX_OFFLINE=true cargo test {{args}}

development-cert:
    mkdir -p development_cert
    test -f development_cert/localhost.crt || mkcert -cert-file development_cert/localhost.crt -key-file development_cert/localhost.key localhost 127.0.0.1 ::1

ci-dev : migrate-database start-test-database && generate-sbom
    #!/usr/bin/env bash
    export RUSTFLAGS="-D warnings"
    # Prevent full recompilations in the normal dev setup which has different rustflags
    export CARGO_TARGET_DIR="target_ci"

    cargo build --release

    just lint
    just format
    just test

lint *args: reuse-lint
    cargo clippy {{args}} -- -D warnings

lint-fix *args: reuse-lint
    cargo clippy --fix {{args}}

reuse-lint:
    reuse --root . lint

format: format-templates
    cargo +nightly fmt --all

format-templates:
    npx prettier --write '**/*.html'

generate-sbom:
    cargo bin cargo-cyclonedx --format json --describe binaries
    # Remove some fields that make the sbom non-reproducible.
    # https://github.com/CycloneDX/cyclonedx-rust-cargo/issues/556
    # https://github.com/CycloneDX/cyclonedx-rust-cargo/issues/514
    jq --sort-keys '.components |= sort_by(.purl) | del(.serialNumber) | del(.metadata.timestamp) | del(..|select(type == "string" and test("^path\\+file")))' linkblocks_bin.cdx.json > linkblocks.cdx.json
    rm linkblocks_bin.cdx.json

install-git-hooks:
    ln -srf pre-commit.sh .git/hooks/pre-commit
