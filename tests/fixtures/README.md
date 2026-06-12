# Redis Cloud API Fixtures

## Contents

- `cloud_openapi.json` — bundled Redis Cloud OpenAPI specification.
- `openapi_unsupported_routes.txt` — spec routes with no typed client handler
  yet (intentionally deferred). Enforced by `tests/openapi_route_coverage.rs`.
- `openapi_non_spec_routes.txt` — typed client routes that don't match any spec
  path (known drift). Also enforced by `tests/openapi_route_coverage.rs`.
- `cloud/samples/*.json` — hand-authored response fixtures that reproduce real
  API shapes with synthetic data. Validated by
  `tests/cloud_fixture_validation.rs`.
- `cloud/captured/` — *gitignored* output of `scripts/generate-cloud-fixtures.sh`
  (local inspection only; never committed).

## Keeping the bundled spec current

`cloud_openapi.json` is the authoritative reference for
`tests/openapi_route_coverage.rs` (it checks our typed handlers against it). It
goes stale when Redis changes the upstream spec. `scripts/check-spec-drift.sh`
compares the bundled copy against the published upstream
(`https://api.redislabs.com/v1/cloud-api-docs`, no auth) and reports
added/removed operations and schemas:

```bash
./scripts/check-spec-drift.sh           # report drift; exit 1 if any
./scripts/check-spec-drift.sh --update  # refresh the bundled spec from upstream
```

After a refresh, reconcile any new routes in `openapi_route_coverage.rs` and
note the change in the CHANGELOG. (Must run outside a sandbox that blocks
outbound network or `/dev/fd` process substitution.)

This catches drift between the bundled spec and the *published spec*. It does
**not** catch where the spec disagrees with the real API's behavior — that
class (#119/#121/#128/#130) is only caught by the live tests.

## Testing layers

Cloud API responses are exercised at three levels:

1. **Inline wiremock tests** (`tests/*_tests.rs`) — fast, no network, no creds.
   They catch logic/routing regressions but not real-vs-model drift, because
   the mocks are written to match our models.
2. **Hand-authored fixtures** (`cloud/samples/`, validated by
   `tests/cloud_fixture_validation.rs`) — encode the shapes the live API
   actually returns (e.g. module `parameters` as an array, numeric
   `creditCardEndsWith`, the `response.resource.costReportId` task envelope),
   with synthetic values. Run in normal CI, no credentials. These guard the
   type-fidelity regressions found via live testing (#118, #119, #120).
3. **Live integration tests** (`tests/live_integration.rs`) — `#[ignore]`d,
   read-only, hit a real account. They catch new drift the fixtures can't
   anticipate.

## Running the live integration tests

```bash
export REDIS_CLOUD_API_KEY=...      # or REDIS_CLOUD_API_ACCOUNT_KEY
export REDIS_CLOUD_API_SECRET=...   # or REDIS_CLOUD_API_USER_KEY
cargo test --test live_integration -- --ignored
```

Read-only (no resource is created, modified, or deleted), so they are safe
against a shared or billable account. Run them outside any sandbox that blocks
outbound TLS.

## Capturing fixtures for inspection

`scripts/generate-cloud-fixtures.sh` captures live responses into the gitignored
`cloud/captured/` directory for local comparison when investigating drift.

**Do not commit captured fixtures.** Its sanitizer is a best-effort denylist; a
real run leaked fields it didn't know about (AWS access key ids, account ids
under unexpected keys). Committed fixtures are hand-authored under
`cloud/samples/` precisely so repo safety never depends on that denylist.
