#!/bin/sh
# Runs inside the container. Env: UBUNTU (VERSION_ID), LSPHP (version, empty = no
# worker running), WP_PHP (version wp-cli reports).
set -e
printf 'ID=ubuntu\nVERSION_ID="%s"\n' "$UBUNTU" > /etc/os-release
if [ -n "$LSPHP" ]; then
  mkdir -p /usr/local/lsws/lsphp83/bin
  gcc -DVER="\"$LSPHP\"" -o /usr/local/lsws/lsphp83/bin/lsphp /e2e/fakephp.c
  /usr/local/lsws/lsphp83/bin/lsphp &
fi
if [ -n "${PLANT:-}" ]; then
  # Unprivileged user runs a fake php-fpm from /tmp; wp-agent (root) must not exec it (if it did, 8.3.1 would be flagged ERROR).
  gcc -DVER='"8.3.1"' -o /tmp/php-fpm /e2e/fakephp.c
  setpriv --reuid=65534 --regid=65534 --clear-groups /tmp/php-fpm &
  sleep 0.5
fi
cd /var/www/html && wp-agent
