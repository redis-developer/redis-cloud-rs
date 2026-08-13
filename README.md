# redis-cloud

[![Crates.io](https://img.shields.io/crates/v/redis-cloud.svg)](https://crates.io/crates/redis-cloud)
[![Documentation](https://docs.rs/redis-cloud/badge.svg)](https://docs.rs/redis-cloud)
[![CI](https://github.com/redis-developer/redis-cloud-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/redis-developer/redis-cloud-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/redis-cloud.svg)](https://github.com/redis-developer/redis-cloud-rs)
[![PyPI](https://img.shields.io/pypi/v/redis-cloud.svg)](https://pypi.org/project/redis-cloud/)

A comprehensive Rust client library for the Redis Cloud REST API, with Python bindings.

## Features

- Complete coverage of all Redis Cloud REST API endpoints (169/169 routes verified against the bundled OpenAPI spec in CI)
- Async/await support with tokio
- Strong typing for API requests and responses
- Comprehensive error handling
- Optional Tower service integration for middleware composition
- Support for all Redis Cloud features including:
  - Pro and Essentials subscriptions and databases
  - User and ACL management
  - Backup, restore, and import operations
  - VPC peering, Transit Gateway, Private Service Connect, PrivateLink
  - Cloud account integration (AWS, GCP, Azure)
  - Task tracking for async operations
  - Cost reports (FOCUS format)

## Installation

```toml
[dependencies]
redis-cloud = "0.12"

# Optional: Enable Tower service integration
redis-cloud = { version = "0.12", features = ["tower-integration"] }
```

## Quick Start

```rust
use redis_cloud::CloudClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CloudClient::builder()
        .api_key(std::env::var("REDIS_CLOUD_API_KEY")?)
        .api_secret(std::env::var("REDIS_CLOUD_API_SECRET")?)
        .build()?;

    // Account info
    let account = client.account().get_current_account().await?;
    if let Some(acc) = account.account {
        println!("Account: {:?} ({:?})", acc.name, acc.id);
    }

    // List Pro subscriptions
    let subs = client.subscriptions().list().await?;
    for sub in subs.subscriptions.unwrap_or_default() {
        println!("  - subscription {:?}: {:?}", sub.id, sub.name);
    }

    // List databases for a subscription (unwrapped helper)
    let dbs = client.databases().list(123).await?;
    for db in &dbs {
        println!("  - db {:?}: {:?}", db.database_id, db.name);
    }

    Ok(())
}
```

## Environment Variables

- `REDIS_CLOUD_API_KEY` — API key
- `REDIS_CLOUD_API_SECRET` — API secret
- `REDIS_CLOUD_BASE_URL` — Override the API base URL (optional; defaults to the production Redis Cloud endpoint)

## Tower Integration

Enable the `tower-integration` feature to use the client with Tower middleware:

```rust
use redis_cloud::CloudClient;
use redis_cloud::tower_support::ApiRequest;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CloudClient::builder()
        .api_key("your-api-key")
        .api_secret("your-api-secret")
        .build()?;

    // Convert to a Tower service
    let mut service = client.into_service();

    // Use the service
    let response = service
        .oneshot(ApiRequest::get("/subscriptions"))
        .await?;

    println!("Response: {:?}", response.body);
    Ok(())
}
```

This enables composition with Tower middleware like circuit breakers, retry, rate limiting, and more.

## Examples

See the [`examples/`](examples) directory for runnable end-to-end programs:

```bash
# Basic — connect, fetch account, list Pro + Essentials subscriptions
cargo run --example basic

# Database operations — list databases for a subscription, paginated
cargo run --example databases [SUBSCRIPTION_ID]

# Streaming — process databases one at a time via the paginated stream
cargo run --example stream_databases -- SUBSCRIPTION_ID

# Tasks — poll an async task by ID
cargo run --example tasks -- TASK_ID

# Cost reports — fetch a cost report in FOCUS-format JSON
cargo run --example cost_report -- START_DATE END_DATE
```

All examples read credentials from `REDIS_CLOUD_API_KEY` / `REDIS_CLOUD_API_SECRET`.

## Python Bindings

A PyO3 binding providing a convenience layer covering read operations across
all major API domains is published at
[redis-cloud on PyPI](https://pypi.org/project/redis-cloud/). See
[`python/README.md`](python/README.md) for the full supported API, the parity
scope decision, and what is intentionally out of scope (connectivity handlers,
cost reports, and write operations — all reachable via the raw HTTP helpers or
the Rust client).

The PyPI publish workflow has been failing since the `reqwest 0.13` upgrade
([#48](https://github.com/redis-developer/redis-cloud-rs/issues/48)).

## API Coverage

The crate covers **100% of the documented Redis Cloud REST API surface** (169/169
routes). Route coverage is enforced by an executable check
([`tests/openapi_route_coverage.rs`](tests/openapi_route_coverage.rs)) that
diffs the typed handlers against the bundled OpenAPI spec; intentional gaps
(currently none) and known off-spec routes (currently none) are tracked in
[`tests/fixtures/openapi_unsupported_routes.txt`](tests/fixtures/openapi_unsupported_routes.txt)
and
[`tests/fixtures/openapi_non_spec_routes.txt`](tests/fixtures/openapi_non_spec_routes.txt).

Handler organization:

| Handler | Description |
|---------|-------------|
| `account()` | Account info, payment methods, regions, logs |
| `subscriptions()` | Pro subscription CRUD, pricing, CIDR, maintenance windows |
| `databases()` | Pro database lifecycle, backups, imports, flush |
| `fixed_subscriptions()` | Essentials subscription management |
| `fixed_databases()` | Essentials database management |
| `acl()` | ACL users, roles, Redis rules |
| `users()` | Account user management |
| `cloud_accounts()` | Cloud provider integration (AWS, GCP, Azure) |
| `vpc_peering()` | VPC peering (standard and Active-Active) |
| `transit_gateway()` | AWS Transit Gateway attachments |
| `psc()` | GCP Private Service Connect |
| `private_link()` | AWS PrivateLink |
| `tasks()` | Async operation tracking |
| `cost_reports()` | Cost reports in FOCUS format |
| `data_integration()` | Data Integration workspaces and opaque workspace proxy operations |
| `endpoint_redirections()` | Dynamic database endpoint redirection lifecycle |

For the authoritative per-endpoint mapping see the bundled OpenAPI spec at
[`tests/fixtures/cloud_openapi.json`](tests/fixtures/cloud_openapi.json).

## Test Coverage

CI enforces a repository-wide line-coverage floor of 76%, based on the reviewed
76.366% all-features baseline. Reproduce the gate locally with:

```bash
cargo llvm-cov --all-features --lcov --output-path lcov.info --fail-under-lines 76
```

The floor is intentionally stored in `.github/workflows/ci.yml`, so lowering it
requires an explicit reviewed change. Raise it as coverage improves. Patch
coverage is not initially enforced: the deterministic repository-wide gate is
the source-of-truth check, while Codecov remains an external reporting service.
An upload outage is reported as a CI warning without being confused with a
source coverage regression.

## Documentation

- [API Documentation](https://docs.rs/redis-cloud)
- [Redis Cloud API Reference](https://api.redislabs.com/v1/swagger-ui/index.html)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
