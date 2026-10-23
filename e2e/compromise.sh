#!/bin/sh
# E2E: wp-agent's Compromise module against a simulated LD_PRELOAD rootkit.
# Needs Docker. Artifacts: e2e/artifacts/compromise-{infected,clean}.txt
set -eu
cd "$(dirname "$0")/.."
out=e2e/artifacts
mkdir -p "$out"
docker build -q -f e2e/compromise/Dockerfile -t wp-agent-e2e-compromise . >/dev/null
for s in infected clean; do
  docker run --rm wp-agent-e2e-compromise /e2e/scenario.sh "$s" | sed 's/\x1b\[[0-9;]*m//g' > "$out/compromise-$s.txt"
done

fail=0
expect() { grep -qF -- "$2" "$out/compromise-$1.txt" || { echo "FAIL [$1] missing: $2"; fail=1; }; }
reject() { if grep -qF -- "$2" "$out/compromise-$1.txt"; then echo "FAIL [$1] unexpected: $2"; fail=1; fi; }

expect infected "indicator(s) of compromise"
expect infected "ERROR: /etc/ld.so.preload preloads into every process: /usr/lib/rk_hook.so"
expect infected "hidden from ps:"
expect infected "rk_miner, /tmp/rk_miner"
expect infected "hidden from ls:"
expect infected "   /usr/lib/rk_hook.so"
expect infected "/var/lib/gone (deleted)"
expect infected "ERROR: 2 dockerd processes"
expect infected "wp-content/uploads/2024/01/thumb.php"
reject infected "wp-content/uploads/index.php"
reject infected "/usr/local/bin/upgraded"

expect clean "No indicators of compromise found"
reject clean "indicator(s) of compromise"

[ "$fail" = 0 ] && echo "PASS: compromise E2E (reports in $out/)"
exit "$fail"
