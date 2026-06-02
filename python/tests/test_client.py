"""Tests for the Redis Cloud Python client."""

import json
import os
import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

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


class TestNewDomainMethods:
    """Tests that new domain methods are present on CloudClient."""

    @pytest.fixture
    def client(self):
        """Create a client for testing."""
        return CloudClient(api_key="test-key", api_secret="test-secret")

    def test_client_has_tasks_methods(self, client):
        """Test that client has tasks list method."""
        assert hasattr(client, "tasks")
        assert hasattr(client, "tasks_sync")

    def test_client_has_task_method(self, client):
        """Test that client has task get method."""
        assert hasattr(client, "task")
        assert hasattr(client, "task_sync")

    def test_client_has_users_methods(self, client):
        """Test that client has users list method."""
        assert hasattr(client, "users")
        assert hasattr(client, "users_sync")

    def test_client_has_user_method(self, client):
        """Test that client has user get method."""
        assert hasattr(client, "user")
        assert hasattr(client, "user_sync")

    def test_client_has_acl_redis_rules_methods(self, client):
        """Test that client has acl_redis_rules method."""
        assert hasattr(client, "acl_redis_rules")
        assert hasattr(client, "acl_redis_rules_sync")

    def test_client_has_acl_roles_methods(self, client):
        """Test that client has acl_roles method."""
        assert hasattr(client, "acl_roles")
        assert hasattr(client, "acl_roles_sync")

    def test_client_has_acl_users_methods(self, client):
        """Test that client has acl_users method."""
        assert hasattr(client, "acl_users")
        assert hasattr(client, "acl_users_sync")

    def test_client_has_cloud_accounts_methods(self, client):
        """Test that client has cloud_accounts list method."""
        assert hasattr(client, "cloud_accounts")
        assert hasattr(client, "cloud_accounts_sync")

    def test_client_has_cloud_account_method(self, client):
        """Test that client has cloud_account get method."""
        assert hasattr(client, "cloud_account")
        assert hasattr(client, "cloud_account_sync")

    def test_client_has_fixed_subscriptions_methods(self, client):
        """Test that client has fixed_subscriptions list method."""
        assert hasattr(client, "fixed_subscriptions")
        assert hasattr(client, "fixed_subscriptions_sync")

    def test_client_has_fixed_subscription_method(self, client):
        """Test that client has fixed_subscription get method."""
        assert hasattr(client, "fixed_subscription")
        assert hasattr(client, "fixed_subscription_sync")

    def test_client_has_fixed_databases_methods(self, client):
        """Test that client has fixed_databases list method."""
        assert hasattr(client, "fixed_databases")
        assert hasattr(client, "fixed_databases_sync")

    def test_client_has_fixed_database_method(self, client):
        """Test that client has fixed_database get method."""
        assert hasattr(client, "fixed_database")
        assert hasattr(client, "fixed_database_sync")


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


class _JsonMockHandler(BaseHTTPRequestHandler):
    """Dispatch GET requests to pre-registered JSON routes."""

    routes: dict = {}

    def do_GET(self):
        body = self.routes.get(self.path)
        if body is None:
            self.send_response(404)
            self.end_headers()
            return
        data = json.dumps(body).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *_args):
        pass  # silence


@pytest.fixture(scope="class")
def mock_server():
    # NOTE: list endpoints (/fixed/subscriptions, .../databases) deserialize
    # into wrapper *objects* on the Rust side, not bare arrays, so their
    # fixtures are `{}` rather than `[]`.
    routes = {
        "/tasks": {"tasks": []},
        "/tasks/task-1": {"taskId": "task-1", "status": "processing-completed"},
        "/users": {"account": 1},
        "/users/1": {"id": 1, "name": "Test User", "email": "t@example.com"},
        "/acl/redisRules": {},
        "/acl/roles": {},
        "/acl/users": {},
        "/acl/users/1": {"id": 1},
        "/cloud-accounts": {"accountId": 1},
        "/cloud-accounts/1": {
            "id": 1,
            "name": "test",
            "accessKeyId": "AKID",
            "status": "active",
            "provider": "AWS",
        },
        "/fixed/subscriptions": {},
        "/fixed/subscriptions/1": {"id": 1, "name": "test-fixed"},
        "/fixed/subscriptions/1/databases": {},
        "/fixed/subscriptions/1/databases/1": {"id": 1, "name": "test-db"},
    }

    class _Handler(_JsonMockHandler):
        pass

    _Handler.routes = routes

    server = HTTPServer(("127.0.0.1", 0), _Handler)
    port = server.server_address[1]
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    yield f"http://127.0.0.1:{port}"
    server.shutdown()


class TestDomainCallsSync:
    """Smoke: each new domain binding makes a real HTTP call through the Rust layer."""

    @pytest.fixture
    def client(self, mock_server):
        return CloudClient(
            api_key="test-key",
            api_secret="test-secret",
            base_url=mock_server,
        )

    def test_tasks_list_sync(self, client):
        result = client.tasks_sync()
        assert isinstance(result, list)

    def test_task_get_sync(self, client):
        result = client.task_sync("task-1")
        assert result is not None

    def test_users_list_sync(self, client):
        result = client.users_sync()
        assert result is not None

    def test_user_get_sync(self, client):
        result = client.user_sync(1)
        assert result is not None

    def test_acl_redis_rules_sync(self, client):
        result = client.acl_redis_rules_sync()
        assert result is not None

    def test_acl_roles_sync(self, client):
        result = client.acl_roles_sync()
        assert result is not None

    def test_acl_users_sync(self, client):
        result = client.acl_users_sync()
        assert result is not None

    def test_cloud_accounts_sync(self, client):
        result = client.cloud_accounts_sync()
        assert result is not None

    def test_cloud_account_get_sync(self, client):
        result = client.cloud_account_sync(1)
        assert result is not None

    def test_fixed_subscriptions_sync(self, client):
        result = client.fixed_subscriptions_sync()
        assert result is not None

    def test_fixed_subscription_get_sync(self, client):
        result = client.fixed_subscription_sync(1)
        assert result is not None

    def test_fixed_databases_sync(self, client):
        result = client.fixed_databases_sync(1)
        assert result is not None

    def test_fixed_database_get_sync(self, client):
        result = client.fixed_database_sync(1, 1)
        assert result is not None
