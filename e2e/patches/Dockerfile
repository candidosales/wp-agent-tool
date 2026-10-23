# Build context: repo root. Driven by e2e/patches.sh.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates gcc libc6-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wp-agent /usr/local/bin/wp-agent
COPY e2e/patches/fakephp.c e2e/patches/scenario.sh /e2e/
# Stub wp-cli: `cli info` reports $WP_PHP, everything else fails fast.
RUN printf '#!/bin/sh\ncase "$*" in *"cli info"*) echo "{\\"php_version\\":\\"$WP_PHP\\"}";; *) exit 1;; esac\n' > /usr/local/bin/wp \
 && chmod +x /usr/local/bin/wp \
 && mkdir -p /var/www/html && echo '<?php' > /var/www/html/wp-config.php
