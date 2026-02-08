default:
    @just --list

dev:
    cargo leptos watch

build:
    cargo leptos build --release

test *args:
    cargo test {{ args }}

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

migrate:
    sqlx migrate run

migration name:
    sqlx migrate add {{ name }}

sqlx-prepare:
    cargo sqlx prepare --workspace

seed:
    cargo run --bin north-server --features ssr -- --seed

check: fmt-check lint test
