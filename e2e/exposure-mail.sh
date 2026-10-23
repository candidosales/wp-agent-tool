#!/bin/sh
# E2E: Exposure and Mail modules, --json output and the exit code.
# Needs Docker. Artifacts: e2e/artifacts/exposure-mail-<scenario>.json (stdout of
# `wp-agent --json`), exposure-mail-<scenario>.exit, exposure-mail-exposed.txt (text report).
#
# Ways it can fail, each covered below:
#  - a .sql dump in uploads isn't an ERROR, or a plain image is flagged
#  - archives/exports in uploads (incl. wpallexport CSVs) aren't flagged; .htaccess is
#  - a recent admin is missed, or an old one is flagged
#  - no MTA / direct delivery / missing SASL map / stuck queue go unreported
#  - an SMTP plugin still triggers mail warnings
#  - progress output leaks into stdout, so --json isn't valid JSON (or the text report is noisy)
#  - exit code isn't 2 when any module reports ERROR
set -eu
cd "$(dirname "$0")/.."
out=e2e/artifacts
mkdir -p "$out"
docker build -q -f e2e/exposure-mail/Dockerfile -t wp-agent-e2e-exposure-mail . >/dev/null

for s in exposed direct stuck clean; do
  docker run --rm wp-agent-e2e-exposure-mail /e2e/scenario.sh "$s" --json \
    > "$out/exposure-mail-$s.json" 2> "$out/exposure-mail-$s.stderr" || true
  grep -o 'exit=[0-9]*' "$out/exposure-mail-$s.stderr" | cut -d= -f2 > "$out/exposure-mail-$s.exit"
done
docker run --rm wp-agent-e2e-exposure-mail /e2e/scenario.sh exposed 2>/dev/null \
  | sed 's/\x1b\[[0-9;]*m//g' > "$out/exposure-mail-exposed.txt"

fail=0
mod() { jq -r --arg m "$2" '.[] | select(.module == $m) | .status, .message, .details[]' "$out/exposure-mail-$1.json"; }
expect() { mod "$1" "$2" | grep -qF -- "$3" || { echo "FAIL [$1/$2] missing: $3"; fail=1; }; }
reject() { if mod "$1" "$2" | grep -qF -- "$3"; then echo "FAIL [$1/$2] unexpected: $3"; fail=1; fi; }

for s in exposed direct stuck clean; do
  jq -e 'type == "array" and length > 10' "$out/exposure-mail-$s.json" >/dev/null || { echo "FAIL [$s] stdout is not the JSON report"; fail=1; }
  has_error=$(jq '[.[] | select(.status == "Error")] | length > 0' "$out/exposure-mail-$s.json")
  code=$(cat "$out/exposure-mail-$s.exit")
  { [ "$has_error" = true ] && [ "$code" = 2 ]; } || { [ "$has_error" = false ] && [ "$code" = 0 ]; } \
    || { echo "FAIL [$s] exit $code with has_error=$has_error"; fail=1; }
done

expect exposed Exposure "Error"
expect exposed Exposure "ERROR: 1 database dump(s) in uploads"
expect exposed Exposure "wp-content/uploads/2024/01/site-backup.sql"
expect exposed Exposure "Warning: 2 archive/export file(s)"
expect exposed Exposure "wp-content/uploads/2024/01/fotos.zip"
expect exposed Exposure "wp-content/uploads/wpallexport/exports/907d/Posts-Export.csv"
reject exposed Exposure "907d/.htaccess"
reject exposed Exposure "photo.jpg"
expect exposed Exposure "1 administrator/editor account(s) registered in the last 90 days"
expect exposed Exposure "intruder <x@evil.test> administrator"
reject exposed Exposure "owner@example.com"
expect exposed Mail "no SMTP plugin and no sendmail on this server"
grep -qF "Exposure             | ERROR" "$out/exposure-mail-exposed.txt" || { echo "FAIL [exposed text] summary row"; fail=1; }
if grep -qF "Running " "$out/exposure-mail-exposed.txt"; then echo "FAIL [exposed text] progress on stdout"; fail=1; fi

expect direct Mail "Postfix delivers directly from this server's IP"

expect stuck Mail "Postfix relays mail through [127.0.0.1]:2525"
expect stuck Mail "ERROR: SMTP authentication is on but the password map /etc/postfix/sasl_passwd.db doesn't exist"
expect stuck Mail "Warning: 1 message(s) stuck in the Postfix queue. Delivery error:"

expect clean Exposure "Ok"
expect clean Exposure "No administrators or editors registered in the last 90 days."
expect clean Mail "Ok"
expect clean Mail "WordPress sends mail through the wp-mail-smtp plugin."

[ "$fail" = 0 ] && echo "PASS: exposure-mail E2E (reports in $out/)"
exit "$fail"
