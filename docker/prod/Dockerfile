# Stage 1: Get diesel_cli from base image
ARG BASE_VERSION=1.0.0
FROM ghcr.io/zorya-development/north/base:${BASE_VERSION} AS tools

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1001 north && \
    useradd -u 1001 -g north -m north

RUN mkdir -p /app/uploads && chown -R north:north /app

COPY --from=tools /usr/local/cargo/bin/diesel /usr/local/bin/diesel

COPY --chown=north:north --chmod=755 target/release/north-server /app/north-server
COPY --chown=north:north target/site /app/site
COPY --chown=north:north Cargo.toml /app/
COPY --chown=north:north migrations /app/migrations
COPY --chown=north:north diesel.toml /app/

WORKDIR /app
USER north

ENV RUST_LOG="info" \
    LEPTOS_SITE_ADDR="0.0.0.0:5000" \
    LEPTOS_SITE_ROOT="site"

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/ || exit 1

CMD ["/app/north-server"]
