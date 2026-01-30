# redis-cloud (Python)

Python bindings for the Redis Cloud REST API client.

## Installation

```bash
pip install redis-cloud
```

## Quick Start

```python
from redis_cloud import CloudClient

# Create client
client = CloudClient(api_key="your-key", api_secret="your-secret")

# Or from environment variables
client = CloudClient.from_env()

# Async usage
async def main():
    subs = await client.subscriptions()
    for sub in subs:
        print(sub["id"], sub["name"])

# Sync usage
subs = client.subscriptions_sync()
```

## API

### CloudClient

- `CloudClient(api_key, api_secret, base_url=None, timeout_secs=None)` - Create client
- `CloudClient.from_env()` - Create from environment variables

#### Subscriptions
- `subscriptions()` / `subscriptions_sync()` - List all subscriptions
- `subscription(id)` / `subscription_sync(id)` - Get subscription by ID

#### Databases
- `databases(subscription_id)` / `databases_sync(subscription_id)` - List databases
- `database(subscription_id, database_id)` / `database_sync(...)` - Get database

#### Raw API
- `get(path)` / `get_sync(path)` - Raw GET request
- `post(path, body)` / `post_sync(path, body)` - Raw POST request
- `delete(path)` / `delete_sync(path)` - Raw DELETE request

## Environment Variables

- `REDIS_CLOUD_API_KEY` - API key
- `REDIS_CLOUD_API_SECRET` - API secret
- `REDIS_CLOUD_BASE_URL` - Optional base URL

## License

MIT OR Apache-2.0
