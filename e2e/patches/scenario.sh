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
cd /var/www/html && wp-agent
