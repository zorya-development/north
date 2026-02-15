FROM rust:1.93-bookworm

LABEL org.opencontainers.image.source="https://github.com/zorya-development/north"

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-binstall for fast binary installs
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install tools
RUN cargo binstall -y cargo-leptos
RUN cargo install diesel_cli --no-default-features --features postgres

# WASM target for Leptos client-side
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add clippy rustfmt
