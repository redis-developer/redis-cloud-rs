"""Tests for the Redis Cloud Python client."""

import os
import pytest
from redis_cloud import CloudClient, RedisCloudError


class TestClientCreation:
    """Tests for client creation."""

    def test_client_creation_with_credentials(self):
        """Test creating a client with explicit credentials."""
        client = CloudClient(api_key="test-key", api_secret="test-secret")
        assert client is not None

    def test_client_creation_with_base_url(self):
        """Test creating a client with a custom base URL."""
        client = CloudClient(
            api_key="test-key",
            api_secret="test-secret",
            base_url="https://custom.api.example.com",
        )
        assert client is not None

    def test_client_creation_with_timeout(self):
        """Test creating a client with a custom timeout."""
        client = CloudClient(
            api_key="test-key", api_secret="test-secret", timeout_secs=60
        )
        assert client is not None
        assert client.timeout == 60.0

    def test_client_timeout_default(self):
        """Test that timeout has a default value when not specified."""
        client = CloudClient(api_key="test-key", api_secret="test-secret")
        # Default timeout is set by the underlying Rust client
        assert client.timeout > 0

    def test_from_env_missing_api_key(self):
        """Test that from_env raises error when API key is missing."""
        # Clear any existing env vars
        for var in [
            "REDIS_CLOUD_API_KEY",
            "REDIS_CLOUD_ACCOUNT_KEY",
            "REDIS_CLOUD_API_SECRET",
            "REDIS_CLOUD_SECRET_KEY",
            "REDIS_CLOUD_USER_KEY",
        ]:
            os.environ.pop(var, None)

        with pytest.raises(ValueError, match="API key not found"):
            CloudClient.from_env()

    def test_from_env_missing_api_secret(self):
        """Test that from_env raises error when API secret is missing."""
        os.environ["REDIS_CLOUD_API_KEY"] = "test-key"
        # Clear secret vars
        for var in [
            "REDIS_CLOUD_API_SECRET",
            "REDIS_CLOUD_SECRET_KEY",
            "REDIS_CLOUD_USER_KEY",
        ]:
            os.environ.pop(var, None)

        try:
            with pytest.raises(ValueError, match="API secret not found"):
                CloudClient.from_env()
        finally:
            os.environ.pop("REDIS_CLOUD_API_KEY", None)

    def test_from_env_with_valid_credentials(self):
        """Test that from_env works with valid environment variables."""
        os.environ["REDIS_CLOUD_API_KEY"] = "test-key"
        os.environ["REDIS_CLOUD_API_SECRET"] = "test-secret"

        try:
            client = CloudClient.from_env()
            assert client is not None
        finally:
            os.environ.pop("REDIS_CLOUD_API_KEY", None)
            os.environ.pop("REDIS_CLOUD_API_SECRET", None)

    def test_from_env_with_alternate_key_names(self):
        """Test that from_env works with alternate environment variable names."""
        os.environ["REDIS_CLOUD_ACCOUNT_KEY"] = "test-key"
        os.environ["REDIS_CLOUD_SECRET_KEY"] = "test-secret"

        try:
            client = CloudClient.from_env()
            assert client is not None
        finally:
            os.environ.pop("REDIS_CLOUD_ACCOUNT_KEY", None)
            os.environ.pop("REDIS_CLOUD_SECRET_KEY", None)


class TestClientMethods:
    """Tests for client methods (without actual API calls)."""

    @pytest.fixture
    def client(self):
        """Create a client for testing."""
        return CloudClient(api_key="test-key", api_secret="test-secret")

    def test_client_has_subscriptions_method(self, client):
        """Test that client has subscriptions method."""
        assert hasattr(client, "subscriptions")
        assert hasattr(client, "subscriptions_sync")

    def test_client_has_subscription_method(self, client):
        """Test that client has subscription method."""
        assert hasattr(client, "subscription")
        assert hasattr(client, "subscription_sync")

    def test_client_has_databases_method(self, client):
        """Test that client has databases method."""
        assert hasattr(client, "databases")
        assert hasattr(client, "databases_sync")

    def test_client_has_database_method(self, client):
        """Test that client has database method."""
        assert hasattr(client, "database")
        assert hasattr(client, "database_sync")

    def test_client_has_all_databases_method(self, client):
        """Test that client has all_databases pagination helper."""
        assert hasattr(client, "all_databases")
        assert hasattr(client, "all_databases_sync")

    def test_client_has_account_method(self, client):
        """Test that client has account method."""
        assert hasattr(client, "account")
        assert hasattr(client, "account_sync")

    def test_client_has_raw_methods(self, client):
        """Test that client has raw HTTP methods."""
        assert hasattr(client, "get")
        assert hasattr(client, "get_sync")
        assert hasattr(client, "post")
        assert hasattr(client, "post_sync")
        assert hasattr(client, "delete")
        assert hasattr(client, "delete_sync")

    def test_client_has_timeout_property(self, client):
        """Test that client has timeout property."""
        assert hasattr(client, "timeout")


class TestErrorHandling:
    """Tests for error handling."""

    def test_redis_cloud_error_exists(self):
        """Test that RedisCloudError is exported."""
        assert RedisCloudError is not None

    def test_redis_cloud_error_is_exception(self):
        """Test that RedisCloudError is an Exception subclass."""
        assert issubclass(RedisCloudError, Exception)


class TestModuleExports:
    """Tests for module exports."""

    def test_cloud_client_exported(self):
        """Test that CloudClient is exported."""
        from redis_cloud import CloudClient

        assert CloudClient is not None

    def test_redis_cloud_error_exported(self):
        """Test that RedisCloudError is exported."""
        from redis_cloud import RedisCloudError

        assert RedisCloudError is not None

    def test_version_exported(self):
        """Test that __version__ is exported."""
        import redis_cloud

        assert hasattr(redis_cloud, "__version__")
        assert isinstance(redis_cloud.__version__, str)
