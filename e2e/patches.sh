#!/bin/sh
# E2E: wp-agent's Patches module (PHP patch level, served vs wp-cli PHP, Ubuntu EOL).
# Needs Docker and network. Artifacts: e2e/artifacts/patches-<scenario>.txt
set -eu
cd "$(dirname "$0")/.."
out=e2e/artifacts
mkdir -p "$out"
# Today's latest 8.3, so the "patched" scenario stays patched after future releases.
latest83=$(curl -fsS https://www.php.net/releases/active.php | grep -o 'php-8\.3\.[0-9]*\.tar\.gz' | head -1 | sed 's/php-//; s/\.tar\.gz//')
docker build -q -f e2e/patches/Dockerfile -t wp-agent-e2e-patches . >/dev/null

run() { # name ubuntu lsphp wp_php [docker args]
  name=$1 ubuntu=$2 lsphp=$3 wp_php=$4; shift 4
  docker run --rm "$@" -e UBUNTU="$ubuntu" -e LSPHP="$lsphp" -e WP_PHP="$wp_php" wp-agent-e2e-patches /e2e/scenario.sh \
    | sed 's/\x1b\[[0-9;]*m//g' > "$out/patches-$name.txt"
}
run eol      20.04 8.3.21        7.4.33
run offline  20.04 8.3.21        7.4.33 --network none
run distro   22.04 ""            8.1.2-1ubuntu2.20
run patched  26.04 "$latest83"   "$latest83"
# SYS_PTRACE: lets container root read other users' /proc/<pid>/exe, as host root can.
run planted  24.04 ""            "$latest83" --cap-add SYS_PTRACE -e PLANT=1

fail=0
# Only the Patches summary row and its details section count.
section() { awk '/^Patches +\|/{print} /STATUS for Patches/{f=1} f{print} f&&/^$/{f=0}' "$out/patches-$1.txt"; }
expect() { section "$1" | grep -qF -- "$2" || { echo "FAIL [$1] missing: $2"; fail=1; }; }
reject() { if section "$1" | grep -qF -- "$2"; then echo "FAIL [$1] unexpected: $2"; fail=1; fi; }

expect eol "Unpatched or end-of-life PHP in use"
expect eol "ERROR: Web server PHP (/usr/local/lsws/lsphp83/bin/lsphp) is 8.3.21, missing security fixes. Latest 8.3 release is $latest83."
expect eol "ERROR: wp-cli PHP is 7.4.33: branch 7.4 is end of life"
expect eol "wp-cli runs a different PHP branch than the web server"
expect eol "Warning: Ubuntu 20.04 standard support ended 2025-05"
reject eol "php.net unreachable"

expect offline "php.net unreachable"
expect offline "ERROR: Web server PHP (/usr/local/lsws/lsphp83/bin/lsphp) is 8.3.21, missing security fixes."

expect distro "Patch status needs attention"
expect distro "Warning: wp-cli PHP is 8.1.2-1ubuntu2.20: branch 8.1 is end of life upstream"
expect distro "No PHP worker running right now"
expect distro "Ubuntu 22.04 is supported until 2027-06"
reject distro "ERROR:"

expect patched "| OK         | PHP and OS are on supported, patched releases"

expect planted "| OK         | PHP and OS are on supported, patched releases"
reject planted "/tmp/php-fpm"

[ "$fail" = 0 ] && echo "PASS: patches E2E (latest 8.3 = $latest83, reports in $out/)"
exit "$fail"
