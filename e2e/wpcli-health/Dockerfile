# Build context: repo root. Driven by e2e/wpcli-health.sh.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends mariadb-server mariadb-client && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wp-agent /usr/local/bin/wp-agent
COPY e2e/wpcli-health/wp /usr/local/bin/wp
COPY e2e/wpcli-health/scenario.sh /e2e/
RUN mkdir -p /var/www/html && printf "%s\n" "<?php" \
  "define( 'DB_NAME', 'wp' );" "define( 'DB_USER', 'wp' );" "define( 'DB_PASSWORD', 'S3cr3t!pw' );" "define( 'DB_HOST', 'localhost' );" \
  "\$table_prefix = 'wpx_';" > /var/www/html/wp-config.php
