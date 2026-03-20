ARG BASE_VERSION=1.2.0
FROM north-base:${BASE_VERSION}

# Dev-only tools
RUN cargo binstall -y just
RUN cargo binstall -y wasm-bindgen-cli@0.2.108 \
    && mkdir -p /root/.cache/cargo-leptos/wasm-bindgen-0.2.108 \
    && ln -s /usr/local/cargo/bin/wasm-bindgen /root/.cache/cargo-leptos/wasm-bindgen-0.2.108/wasm-bindgen

WORKDIR /app
