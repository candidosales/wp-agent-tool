# Build context: repo root. Driven by e2e/wpcli-health.sh.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /src/target/release/wp-agent /usr/local/bin/wp-agent
COPY e2e/wpcli-health/wp /usr/local/bin/wp
RUN mkdir -p /var/www/html && echo '<?php' > /var/www/html/wp-config.php
