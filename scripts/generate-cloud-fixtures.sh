#!/usr/bin/env bash
#
# Capture Redis Cloud API responses locally, for INSPECTION when investigating
# real-vs-model drift. Output goes to tests/fixtures/cloud/captured/, which is
# .gitignored — these captures are NOT meant to be committed.
#
# The committed, CI-runnable fixtures live in tests/fixtures/cloud/samples/ and
# are HAND-AUTHORED with synthetic data (see tests/cloud_fixture_validation.rs).
# That is deliberate: the sanitizer below is a best-effort denylist, and a real
# run proved it leaks sensitive fields the API returns but the denylist doesn't
# know about (e.g. AWS access key ids, account ids under unexpected keys). Never
# trust it for data that gets committed to a public repo.
#
# Usage (read-only; never creates/modifies/deletes anything):
#   export REDIS_CLOUD_API_KEY=...      # or REDIS_CLOUD_API_ACCOUNT_KEY
#   export REDIS_CLOUD_API_SECRET=...   # or REDIS_CLOUD_API_USER_KEY
#   ./scripts/generate-cloud-fixtures.sh
#
# Requires: curl, jq.

set -euo pipefail

API_KEY="${REDIS_CLOUD_API_KEY:-${REDIS_CLOUD_API_ACCOUNT_KEY:-}}"
API_SECRET="${REDIS_CLOUD_API_SECRET:-${REDIS_CLOUD_API_USER_KEY:-}}"
BASE_URL="${REDIS_CLOUD_BASE_URL:-https://api.redislabs.com/v1}"

if [[ -z "$API_KEY" || -z "$API_SECRET" ]]; then
  echo "error: set REDIS_CLOUD_API_KEY and REDIS_CLOUD_API_SECRET" >&2
  exit 1
fi
command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../tests/fixtures/cloud/captured"
mkdir -p "$OUT_DIR"

# Recursive sanitizer. Scrubs identifying values by key name while preserving
# structural and enum-like values (status, provider, protocol, version, …) so
# the fixtures still exercise real serde paths.
#
# - numeric identifiers -> deterministic fakes
# - identifying strings -> placeholders
# - HATEOAS hrefs        -> a templated URL (they embed account/sub/db ids)
read -r -d '' SANITIZE <<'JQ' || true
def scrub:
  walk(
    if type == "object" then
      with_entries(
        (.key) as $k
        | if ($k | IN("id","account","accountId","subscriptionId","databaseId","paymentMethodId",
                      "resourceId","additionalResourceId","userAccountId","bdbId","planId"))
            and (.value | type == "number")
          then .value = 12345
        elif $k == "creditCardEndsWith"
          then .value = (if (.value | type == "number") then 1234 else "1234" end)
        elif $k | IN("name","nameOnCard","accountName","accountMarketplaceId",
                     "email","resourceName","serverCert","password","planName","role",
                     "accessKeyId","accessSecretKey","secretKey","awsUserArn",
                     "awsConsoleRoleArn","awsAccountId","gcpServiceAccountId")
          then .value = (if (.value | type == "string") then "example" else .value end)
        elif $k | IN("publicEndpoint","privateEndpoint","public","private")
          then .value = (if (.value | type == "string")
                         then "redis-12345.example.cloud.redislabs.com:12345" else .value end)
        elif $k | IN("httpSourceIp")
          then .value = "203.0.113.10"
        elif $k | IN("allowedSourceIps","sourceIps")
          then .value = (if (.value | type == "array") then ["0.0.0.0/0"] else .value end)
        elif $k == "href"
          then .value = "https://api.redislabs.com/v1/redacted"
        else .
        end
      )
    else .
    end
  );
scrub
JQ

fetch() {
  # fetch <relative-path> <fixture-name>
  local path="$1" name="$2"
  echo "  GET $path -> $name.json"
  curl -fsS "$BASE_URL$path" \
    -H "x-api-key: $API_KEY" -H "x-api-secret-key: $API_SECRET" \
    | jq "$SANITIZE" > "$OUT_DIR/$name.json"
}

echo "Capturing fixtures into $OUT_DIR"

fetch "/"                  account
fetch "/payment-methods"   payment_methods
fetch "/data-persistence"  data_persistence
fetch "/database-modules"  database_modules
fetch "/regions"           regions
fetch "/subscriptions"     subscriptions
fetch "/fixed/subscriptions" fixed_subscriptions
fetch "/acl/redisRules"    acl_redis_rules
fetch "/acl/roles"         acl_roles
fetch "/acl/users"         acl_users
fetch "/users"             users
fetch "/cloud-accounts"    cloud_accounts
fetch "/tasks"             tasks

# Drill into the first Essentials subscription for the rich database shape.
FIRST_SUB="$(curl -fsS "$BASE_URL/fixed/subscriptions" \
  -H "x-api-key: $API_KEY" -H "x-api-secret-key: $API_SECRET" \
  | jq -r '.subscriptions[0].id // empty')"

if [[ -n "$FIRST_SUB" ]]; then
  fetch "/fixed/subscriptions/$FIRST_SUB"            fixed_subscription
  fetch "/fixed/subscriptions/$FIRST_SUB/databases"  fixed_databases
else
  echo "  (no Essentials subscription found; skipping single-sub + databases fixtures)"
fi

echo "Done. Files in $OUT_DIR are .gitignored and for local inspection only."
echo "Do NOT commit them — the committed fixtures are hand-authored under"
echo "tests/fixtures/cloud/samples/."
