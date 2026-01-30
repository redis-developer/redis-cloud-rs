"""Redis Cloud Python client.

This module provides Python bindings for the Redis Cloud REST API.

Example:
    from redis_cloud import CloudClient

    # Create client
    client = CloudClient(api_key="...", api_secret="...")

    # Async usage
    async def main():
        subs = await client.subscriptions()
        print(subs)

    # Sync usage
    subs = client.subscriptions_sync()
"""

from .redis_cloud import CloudClient, RedisCloudError, __version__

__all__ = ["CloudClient", "RedisCloudError", "__version__"]
