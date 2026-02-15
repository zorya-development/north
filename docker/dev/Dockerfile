ARG BASE_VERSION=1.0.0
FROM north-base:${BASE_VERSION}

# Dev-only tools
RUN cargo binstall -y just

# Tailwind CSS v4 standalone
RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 \
    && chmod +x tailwindcss-linux-x64 \
    && mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss

WORKDIR /app
