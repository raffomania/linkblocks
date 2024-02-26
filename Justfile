set dotenv-load := true
set export := true

watch: development-cert start-database
    cargo bin systemfd --no-pid -s http::4040 -- cargo bin cargo-watch -- cargo run start --listenfd

run *args: development-cert
    cargo run -- {{args}}

insert-demo-data: migrate-database
    cargo run -- insert-demo-data

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

stop-database:
    podman stop linkblocks_postgres

wipe-database: stop-database
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
    DATABASE_URL=${DATABASE_URL_TEST} SQLX_OFFLINE=true cargo test {{args}}

development-cert:
    mkdir -p development_cert
    test -f development_cert/localhost.crt || mkcert -cert-file development_cert/localhost.crt -key-file development_cert/localhost.key localhost 127.0.0.1 ::1

ci-dev: start-database start-test-database
    #!/usr/bin/env bash

    cargo build --release
    just test
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
