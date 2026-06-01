# Redis Cloud API Fixtures

## Contents

- `cloud_openapi.json` — bundled Redis Cloud OpenAPI specification.
- `openapi_unsupported_routes.txt` — spec routes with no typed client handler
  yet (intentionally deferred). Enforced by `tests/openapi_route_coverage.rs`.
- `openapi_non_spec_routes.txt` — typed client routes that don't match any spec
  path (known drift). Also enforced by `tests/openapi_route_coverage.rs`.

Both allowlists are intentional-exception lists: the coverage test fails if a
new gap appears without an entry, **and** if an entry goes stale (the route got
covered or the path was fixed). Shrinking them is tracked in #72.

## Current Status

Beyond the OpenAPI spec and route-coverage allowlists above, there are no
captured response fixtures yet.

## Why No Real Fixtures Yet?

Unlike Enterprise API (which uses Docker for testing), Cloud API fixtures require:
1. A real Cloud account with active resources
2. Billable subscriptions and databases
3. Careful sanitization of account data before committing

## Generating Cloud Fixtures

When you have a Cloud account with test resources, you can generate fixtures:

```bash
export REDIS_CLOUD_API_KEY="your-key"
export REDIS_CLOUD_SECRET_KEY="your-secret"
./scripts/generate-cloud-fixtures.sh
```

**Important**: Review all generated fixtures for sensitive data before committing!

## Current Testing Approach

Cloud API tests currently use wiremock with inline JSON mocks. This approach:
- ✅ Works well for testing
- ✅ No infrastructure required
- ✅ No costs
- ⚠️  Doesn't catch type mismatches from real API responses

## Future Work

To get the full benefits of fixture-based testing for Cloud:
1. Use a test Cloud account with minimal resources
2. Generate fixtures from real API responses
3. Sanitize account/subscription IDs
4. Add validation tests like Enterprise has
