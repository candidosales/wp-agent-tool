# Build context: repo root. Driven by e2e/exposure-mail.sh.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN echo "postfix postfix/main_mailer_type select Internet Site" | debconf-set-selections \
 && echo "postfix postfix/mailname string example.com" | debconf-set-selections \
 && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends postfix && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wp-agent /usr/local/bin/wp-agent
COPY e2e/exposure-mail/scenario.sh /e2e/
# Stub wp-cli: active plugins from $PLUGINS (comma-separated), users from $USERS (JSON);
# everything else fails fast.
RUN printf '#!/bin/sh\ncase "$*" in\n  *"plugin list --status=active"*) echo "$PLUGINS" | tr , "\\n";;\n  *"user list"*) echo "$USERS";;\n  *) exit 1;;\nesac\n' > /usr/local/bin/wp \
 && chmod +x /usr/local/bin/wp \
 && mkdir -p /var/www/html/wp-content/uploads/2024/01 && echo '<?php' > /var/www/html/wp-config.php \
 && touch /var/www/html/wp-content/uploads/2024/01/photo.jpg
