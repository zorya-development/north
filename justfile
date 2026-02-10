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
    diesel migration run

migration name:
    diesel migration generate {{ name }}

migration-diff name:
    diesel migration generate --diff-schema {{ name }}

migrate-revert:
    diesel migration revert

migrate-redo:
    diesel migration redo

seed:
    cargo run --bin north-server --features ssr -- --seed

check: fmt-check lint test
