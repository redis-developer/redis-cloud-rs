# redis-cloud

[![Crates.io](https://img.shields.io/crates/v/redis-cloud.svg)](https://crates.io/crates/redis-cloud)
[![Documentation](https://docs.rs/redis-cloud/badge.svg)](https://docs.rs/redis-cloud)
[![CI](https://github.com/redis-developer/redis-cloud-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/redis-developer/redis-cloud-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/redis-cloud.svg)](https://github.com/redis-developer/redis-cloud-rs)
[![PyPI](https://img.shields.io/pypi/v/redis-cloud.svg)](https://pypi.org/project/redis-cloud/)

A comprehensive Rust client library for the Redis Cloud REST API, with Python bindings.

## Features

- Complete coverage of Redis Cloud REST API endpoints
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
redis-cloud = "0.8"

# Optional: Enable Tower service integration
redis-cloud = { version = "0.8", features = ["tower-integration"] }
```

## Quick Start

```rust
use redis_cloud::CloudClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client using builder pattern
    let client = CloudClient::builder()
        .api_key("your-api-key")
        .api_secret("your-api-secret")
        .build()?;

    // Get account information using fluent API
    let account = client.account().get_current_account().await?;
    println!("Account: {:?}", account);

    // List all subscriptions
    let subscriptions = client.subscriptions().get_all_subscriptions().await?;
    println!("Subscriptions: {:?}", subscriptions);

    // List databases in a subscription
    let databases = client.databases().get_subscription_databases(123, None, None).await?;
    println!("Databases: {:?}", databases);

    Ok(())
}
```

You can also use explicit handler creation if preferred:

```rust
use redis_cloud::{CloudClient, SubscriptionHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CloudClient::builder()
        .api_key("your-api-key")
        .api_secret("your-api-secret")
        .build()?;

    // Explicit handler creation
    let handler = SubscriptionHandler::new(client.clone());
    let subscriptions = handler.get_all_subscriptions().await?;

    Ok(())
}
```

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

See the `examples/` directory for runnable examples:

```bash
# Basic usage - connect and list subscriptions
REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example basic

# Database operations
REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example databases

# Streaming with pagination
REDIS_CLOUD_API_KEY=xxx REDIS_CLOUD_API_SECRET=yyy cargo run --example stream_databases -- SUBSCRIPTION_ID
```

## Python Bindings

This library also provides Python bindings via PyO3:

```bash
pip install redis-cloud
```

```python
from redis_cloud import CloudClient

# Create client
client = CloudClient(
    api_key="your-api-key",
    api_secret="your-api-secret"
)

# Or from environment variables
client = CloudClient.from_env()

# Async usage
async def main():
    subs = await client.subscriptions()
    for sub in subs:
        print(sub["name"], sub["id"])

# Sync usage
subs = client.subscriptions_sync()
```

### Python API

- `CloudClient(api_key, api_secret, base_url=None, timeout_secs=None)`
- `CloudClient.from_env()` - Create from environment variables
- `client.timeout` - Get configured timeout in seconds (property)

#### Account
- `account()` / `account_sync()` - Get current account information

#### Subscriptions
- `subscriptions()` / `subscriptions_sync()` - List all subscriptions
- `subscription(id)` / `subscription_sync(id)` - Get subscription by ID

#### Databases
- `databases(subscription_id, offset=None, limit=None)` / `databases_sync(...)` - List databases (paginated)
- `database(subscription_id, database_id)` / `database_sync(...)` - Get database
- `all_databases(subscription_id)` / `all_databases_sync(...)` - Get all databases (auto-pagination)

#### Raw API
- `get(path)` / `get_sync(path)` - Raw GET request
- `post(path, body)` / `post_sync(path, body)` - Raw POST request
- `delete(path)` / `delete_sync(path)` - Raw DELETE request

### Environment Variables

- `REDIS_CLOUD_API_KEY` - API key
- `REDIS_CLOUD_API_SECRET` - API secret
- `REDIS_CLOUD_BASE_URL` - Base URL (optional)

## API Coverage

This library provides comprehensive coverage of the Redis Cloud REST API:

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

## Documentation

- [API Documentation](https://docs.rs/redis-cloud)
- [Redis Cloud API Reference](https://api.redislabs.com/v1/swagger-ui/index.html)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
