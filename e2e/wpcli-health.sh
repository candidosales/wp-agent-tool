#!/bin/sh
# E2E: WpCliHealth module and the Performance fallback's error attribution.
# Needs Docker. Artifacts: e2e/artifacts/wpcli-health-<scenario>.json (`wp-agent --json`).
#
# Ways it can fail, each covered below:
#  - wp-cli that can't boot WordPress isn't an ERROR, or the fatal text is lost
#  - the "must be compatible with" fatal isn't recognised as PHP-version-specific
#  - a wp-cli older than 2.5 isn't flagged, or a current one is
#  - Performance's raw-DB fallback hides wp-cli's original error
#  - with wp-cli dead, table_prefix isn't read from wp-config.php, the raw-DB fallback
#    doesn't authenticate (password passed via MYSQL_PWD), or Performance/Mail don't use it
set -eu
cd "$(dirname "$0")/.."
out=e2e/artifacts
mkdir -p "$out"
docker build -q -f e2e/wpcli-health/Dockerfile -t wp-agent-e2e-wpcli-health . >/dev/null

run() { # name boot version db
  docker run --rm -e BOOT="$2" -e WPCLI_VERSION="$3" -e DB="$4" wp-agent-e2e-wpcli-health \
    /e2e/scenario.sh --json 2>/dev/null > "$out/wpcli-health-$1.json" || true
}
run broken fatal 2.4.0 0
run healthy ok 2.12.0 0
run dead-db dead 2.12.0 1

fail=0
mod() { jq -r --arg m "$2" '.[] | select(.module == $m) | .status, .message, .details[]' "$out/wpcli-health-$1.json"; }
expect() { mod "$1" "$2" | grep -qF -- "$3" || { echo "FAIL [$1/$2] missing: $3"; fail=1; }; }
reject() { if mod "$1" "$2" | grep -qF -- "$3"; then echo "FAIL [$1/$2] unexpected: $3"; fail=1; fi; }

expect broken WpCliHealth "wp-cli cannot boot WordPress"
expect broken WpCliHealth "ERROR: wp-cli failed to boot WordPress: WP-CLI failed: PHP Fatal error:  Declaration of Foo::fromArray"
expect broken WpCliHealth "is often a PHP-version-specific engine bug"
expect broken WpCliHealth "Warning: wp-cli 2.4.0 is older than 2.5"
expect broken Performance "Could not determine autoloaded options size: WP-CLI failed: PHP Fatal error"
expect broken Performance "raw DB fallback also failed"
expect broken Mail "Could not list active plugins (wp-cli and raw DB both failed)"

expect healthy WpCliHealth "Ok"
expect healthy WpCliHealth "wp-cli booted WordPress successfully."
expect healthy WpCliHealth "wp-cli version: 2.12.0"
reject healthy WpCliHealth "Warning:"

expect dead-db WpCliHealth "wp-cli cannot boot WordPress"
expect dead-db Performance "Autoloaded options size: 2.00 MB"
expect dead-db Mail "WordPress sends mail through the wp-mail-smtp plugin."

[ "$fail" = 0 ] && echo "PASS: wpcli-health E2E (reports in $out/)"
exit "$fail"
