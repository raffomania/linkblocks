set dotenv-load := true
set export := true

watch: development-cert start-database
    systemfd --no-pid -s http::4040 -- cargo watch -- cargo run start --listenfd

run *args: development-cert
    cargo run -- {{args}}

generate-database-info: start-database migrate-database
    cargo sqlx prepare

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
            --health-cmd pg_isready --health-interval 10s \
            -e POSTGRES_HOST_AUTH_METHOD=trust -e POSTGRES_DB=linkblocks \
            -p ${DATABASE_PORT}:5432 docker.io/postgres:16 \
            postgres
    fi

    podman start linkblocks_postgres

    for i in {1..20}; do 
        pg_isready -h localhost -p $DATABASE_PORT && break
        sleep 1
    done

stop-database:
    podman stop linkblocks_postgres

wipe-database: stop-database
    podman rm linkblocks_postgres

migrate-database:
    cargo sqlx migrate run

test:
    cargo test

development-cert:
    mkdir -p development_cert
    test -f development_cert/localhost.crt || mkcert -cert-file development_cert/localhost.crt -key-file development_cert/localhost.key localhost 127.0.0.1 ::1

ci-dev: 
    cargo build
    cargo test
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
