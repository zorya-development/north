ARG BASE_VERSION=1.0.0
ARG REGISTRY=ghcr.io/zorya-development/north/base

# === Build stage ===
FROM ${REGISTRY}:${BASE_VERSION} AS builder

WORKDIR /build
COPY . .

RUN cargo leptos build --release

# === Runtime stage ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1001 north && \
    useradd -u 1001 -g north -m north

RUN mkdir -p /app/uploads && chown -R north:north /app

COPY --from=builder --chown=north:north /build/target/release/north-server /app/north-server
COPY --from=builder --chown=north:north /build/target/site /app/site
COPY --from=builder --chown=north:north /build/Cargo.toml /app/

WORKDIR /app
USER north

ENV RUST_LOG="info" \
    LEPTOS_SITE_ADDR="0.0.0.0:5000" \
    LEPTOS_SITE_ROOT="site"

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/ || exit 1

CMD ["/app/north-server"]
