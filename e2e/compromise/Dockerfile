# Build context: repo root. Driven by e2e/compromise.sh.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends procps gcc libc6-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wp-agent /usr/local/bin/wp-agent
COPY e2e/compromise/rk_hook.c e2e/compromise/scenario.sh /e2e/
# Stub wp-cli (every call fails fast) and a minimal WordPress tree; only the
# Compromise module's result matters here.
RUN printf '#!/bin/sh\nexit 1\n' > /usr/local/bin/wp && chmod +x /usr/local/bin/wp \
 && mkdir -p /var/www/html/wp-content/uploads/2024/01 \
 && echo '<?php' > /var/www/html/wp-config.php \
 && echo '<?php // Silence is golden.' > /var/www/html/wp-content/uploads/index.php
