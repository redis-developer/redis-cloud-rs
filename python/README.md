# redis-cloud (Python)

PyO3-based Python bindings for the [`redis-cloud`](https://crates.io/crates/redis-cloud)
Rust client for the Redis Cloud REST API.

## Parity scope

**Python is a deliberately smaller convenience layer.** It covers read-oriented
operations across all major API domains — the methods you reach for when
scripting, building dashboards, or doing field-engineering work. Write
operations (create, update, delete) are available via the raw HTTP helpers
(`post`, `delete`) or the full-featured Rust client.

Every domain method comes in two flavors:

- An **async** variant (e.g. `subscriptions()`), which returns an awaitable.
- A **sync** variant suffixed with `_sync` (e.g. `subscriptions_sync()`), which
  blocks and returns the result directly.

All methods return plain Python objects (dicts, lists, scalars) decoded from the
API's JSON responses.

## Supported API coverage

| Domain | Methods |
|--------|---------|
| Account | `account`, `account_sync` |
| Pro subscriptions | `subscriptions`, `subscriptions_sync`, `subscription`, `subscription_sync` |
| Pro databases | `databases`, `databases_sync`, `database`, `database_sync`, `all_databases`, `all_databases_sync` |
| Tasks | `tasks`, `tasks_sync`, `task`, `task_sync` |
| Users | `users`, `users_sync`, `user`, `user_sync` |
| ACL | `acl_redis_rules`, `acl_redis_rules_sync`, `acl_roles`, `acl_roles_sync`, `acl_users`, `acl_users_sync` |
| Cloud accounts | `cloud_accounts`, `cloud_accounts_sync`, `cloud_account`, `cloud_account_sync` |
| Essentials subscriptions | `fixed_subscriptions`, `fixed_subscriptions_sync`, `fixed_subscription`, `fixed_subscription_sync` |
| Essentials databases | `fixed_databases`, `fixed_databases_sync`, `fixed_database`, `fixed_database_sync` |
| Raw HTTP | `get`, `get_sync`, `post`, `post_sync`, `delete`, `delete_sync` |

## Out of scope

The following are intentionally deferred from the Python bindings for this
parity round. They are fully supported by the Rust client, and most can be
reached from Python via the raw HTTP helpers when needed:

- **Write operations** (create / update / delete of subscriptions, databases,
  users, etc.) — use `post` and `delete` with the relevant API path for simple
  operations, or the full-featured Rust client for typed `put`/`patch` request
  bodies and structured responses.
- **Connectivity handlers** — VPC peering, Transit Gateway, Private Service
  Connect (PSC), and Private Link. These are complex, write-heavy networking
  flows better served by the Rust client.
- **Cost reports** (`cost_report`) — FOCUS-format billing exports.
- **Pagination beyond `all_databases`** — the Pro `all_databases` helper is the
  one auto-paginating convenience exposed; other list endpoints return a single
  page (use the raw helpers with `offset`/`limit` for manual paging).

## Quick start

```python
from redis_cloud import CloudClient

# Construct from explicit credentials...
client = CloudClient(api_key="your-api-key", api_secret="your-api-secret")

# ...or from environment variables
# (REDIS_CLOUD_API_KEY / REDIS_CLOUD_API_SECRET).
client = CloudClient.from_env()

# Synchronous calls — block and return the decoded JSON.
account = client.account_sync()
subs = client.subscriptions_sync()

for sub in subs.get("subscriptions", []):
    print(sub["id"], sub["name"])
    for db in client.databases_sync(sub["id"]).get("subscription", []):
        print("  ", db)

# Async calls — await the awaitable.
import asyncio

async def main():
    tasks = await client.tasks()
    print(tasks)

asyncio.run(main())

# Raw HTTP for anything not covered by a typed method, including writes.
created = client.post_sync("/subscriptions", {"name": "example"})
```

## Installation

From PyPI:

```bash
pip install redis-cloud
```

### Building from source

The extension is built with [maturin](https://www.maturin.rs/). From the
`python/` directory, inside an activated virtualenv:

```bash
pip install maturin
maturin develop   # build and install into the current virtualenv
```

To build a release wheel:

```bash
maturin build --release
```
