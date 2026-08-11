#!/usr/bin/env bash
#
# Compare the bundled Redis Cloud OpenAPI spec against the upstream published
# spec and report drift: added/removed operations (method + path) and schemas.
#
# Why: the bundled spec (tests/fixtures/cloud_openapi.json) is what
# tests/openapi_route_coverage.rs checks our typed handlers against. When Redis
# changes the upstream spec, the bundled copy goes stale until refreshed. This
# script makes that drift visible (and refreshable).
#
# Note: this detects drift between the bundled spec and the *published spec*.
# It does NOT detect where the spec disagrees with the real API's behavior —
# that class (e.g. #119/#121/#128/#130) is only caught by the live tests.
#
# Usage:
#   ./scripts/check-spec-drift.sh           # report drift; exit 1 if any drift
#   ./scripts/check-spec-drift.sh --update  # overwrite the bundled spec with upstream
#
# Override the source with REDIS_CLOUD_SPEC_URL. Requires: curl, jq.
# The upstream endpoint needs no authentication.

set -euo pipefail

UPSTREAM_URL="${REDIS_CLOUD_SPEC_URL:-https://api.redislabs.com/v1/cloud-api-docs}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLED="$SCRIPT_DIR/../tests/fixtures/cloud_openapi.json"
UPDATE=0
[[ "${1:-}" == "--update" ]] && UPDATE=1

command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 2; }

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
echo "Fetching upstream spec: $UPSTREAM_URL"
curl -fsS "$UPSTREAM_URL" -o "$TMP" || { echo "error: failed to fetch upstream spec" >&2; exit 2; }
jq -e '.openapi and .paths' "$TMP" >/dev/null 2>&1 \
  || { echo "error: upstream response is not a valid OpenAPI document" >&2; exit 2; }

# method + path for every operation (path items also carry non-method keys).
ops() {
  jq -r '
    def normalize_path:
      if . == "/v1" then "/"
      elif startswith("/v1/") then .[3:]
      else .
      end;
    .paths | to_entries[] | .key as $p
    | (.value | to_entries[]
       | select(.key | IN("get","put","post","delete","patch","head","options","trace"))
       | "\(.key | ascii_upcase) \($p | normalize_path)")
  ' "$1" | sort -u
}
schemas() { jq -r '.components.schemas | keys[]' "$1" | sort -u; }

added_ops="$(comm -13 <(ops "$BUNDLED") <(ops "$TMP") || true)"
removed_ops="$(comm -23 <(ops "$BUNDLED") <(ops "$TMP") || true)"
added_schemas="$(comm -13 <(schemas "$BUNDLED") <(schemas "$TMP") || true)"
removed_schemas="$(comm -23 <(schemas "$BUNDLED") <(schemas "$TMP") || true)"

section() { # title, body
  [[ -n "$2" ]] || return 0
  echo ""
  echo "$1"
  echo "$2" | awk '{ print "  " $0 }'
}

bundled_ver="$(jq -r '.info.version // "?"' "$BUNDLED")"
upstream_ver="$(jq -r '.info.version // "?"' "$TMP")"
echo "bundled:  $(ops "$BUNDLED" | wc -l | tr -d ' ') ops, $(schemas "$BUNDLED" | wc -l | tr -d ' ') schemas (info.version: $bundled_ver)"
echo "upstream: $(ops "$TMP" | wc -l | tr -d ' ') ops, $(schemas "$TMP" | wc -l | tr -d ' ') schemas (info.version: $upstream_ver)"

drift=0
[[ -n "$added_ops$removed_ops$added_schemas$removed_schemas" ]] && drift=1

if [[ $drift -eq 0 ]]; then
  echo ""
  echo "No drift: bundled spec matches upstream."
  exit 0
fi

echo ""
echo "DRIFT DETECTED between bundled spec and upstream:"
section "Operations added upstream (missing from bundled):" "$added_ops"
section "Operations removed upstream (still in bundled):" "$removed_ops"
section "Schemas added upstream (missing from bundled):" "$added_schemas"
section "Schemas removed upstream (still in bundled):" "$removed_schemas"

if [[ $UPDATE -eq 1 ]]; then
  jq . "$TMP" > "$BUNDLED"
  echo ""
  echo "Updated $BUNDLED from upstream. Review the diff, then update"
  echo "tests/openapi_route_coverage.rs allowlists if new routes need handlers."
  exit 0
fi

echo ""
echo "Re-run with --update to refresh the bundled spec, then reconcile handlers"
echo "(tests/openapi_route_coverage.rs) and CHANGELOG."
exit 1
