//! Mock server wrapper for Redis Cloud API testing

use serde_json::{Value, json};
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::CloudClient;

/// A mock server configured for Redis Cloud API testing
///
/// This wrapper provides convenience methods for common API mocking scenarios
/// while still allowing access to the underlying wiremock server for custom needs.
pub struct MockCloudServer {
    server: MockServer,
}

impl MockCloudServer {
    /// Start a new mock server
    ///
    /// The server listens on a random available port.
    pub async fn start() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }

    /// Get the base URI of the mock server
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Create a CloudClient configured to use this mock server
    ///
    /// The client is pre-configured with test credentials and the mock server's URI.
    pub fn client(&self) -> CloudClient {
        CloudClient::builder()
            .api_key("test-key")
            .api_secret("test-secret")
            .base_url(self.server.uri())
            .build()
            .expect("Failed to build test client")
    }

    /// Create a CloudClient with custom credentials
    pub fn client_with_credentials(&self, api_key: &str, api_secret: &str) -> CloudClient {
        CloudClient::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .base_url(self.server.uri())
            .build()
            .expect("Failed to build test client")
    }

    /// Get the underlying wiremock MockServer for custom mocking
    pub fn inner(&self) -> &MockServer {
        &self.server
    }

    /// Mount a custom Mock on the server
    pub async fn mount(&self, mock: Mock) {
        mock.mount(&self.server).await;
    }

    /// Mount a mock for a specific path with a given response
    pub async fn mock_path(&self, http_method: &str, path_str: &str, response: ResponseTemplate) {
        Mock::given(method(http_method))
            .and(path(path_str))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(response)
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // Account Mocks
    // =========================================================================

    /// Mock the root account endpoint (GET /)
    pub async fn mock_account(&self, account: Value) {
        Mock::given(method("GET"))
            .and(path("/"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(account))
            .mount(&self.server)
            .await;
    }

    /// Mock the regions endpoint (GET /regions)
    pub async fn mock_regions(&self, regions: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/regions"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "regions": regions
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock the database modules endpoint (GET /database-modules)
    pub async fn mock_database_modules(&self, modules: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/database-modules"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "modules": modules
            })))
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // Subscription Mocks
    // =========================================================================

    /// Mock the subscriptions list endpoint (GET /subscriptions)
    pub async fn mock_subscriptions_list(&self, subscriptions: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/subscriptions"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "accountId": 12345,
                "subscriptions": subscriptions
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock a specific subscription get endpoint (GET /subscriptions/{id})
    pub async fn mock_subscription_get(&self, subscription_id: i32, subscription: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/subscriptions/{subscription_id}")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(subscription))
            .mount(&self.server)
            .await;
    }

    /// Mock subscription creation endpoint (POST /subscriptions)
    pub async fn mock_subscription_create(&self, task_id: &str, resource_id: i32) {
        Mock::given(method("POST"))
            .and(path("/subscriptions"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "taskId": task_id,
                "commandType": "subscriptionCreateRequest",
                "status": "received",
                "response": {
                    "resourceId": resource_id
                }
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock subscription deletion endpoint (DELETE /subscriptions/{id})
    pub async fn mock_subscription_delete(&self, subscription_id: i32, task_id: &str) {
        Mock::given(method("DELETE"))
            .and(path(format!("/subscriptions/{subscription_id}")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "taskId": task_id,
                "commandType": "subscriptionDeleteRequest",
                "status": "received"
            })))
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // Database Mocks
    // =========================================================================

    /// Mock the databases list endpoint (GET /subscriptions/{id}/databases)
    ///
    /// Returns the correct nested structure expected by `get_subscription_databases()`:
    /// ```json
    /// {
    ///   "accountId": 12345,
    ///   "subscription": [{
    ///     "subscriptionId": 123,
    ///     "numberOfDatabases": 2,
    ///     "databases": [...]
    ///   }]
    /// }
    /// ```
    pub async fn mock_databases_list(&self, subscription_id: i32, databases: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path(format!("/subscriptions/{subscription_id}/databases")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "accountId": 12345,
                "subscription": [{
                    "subscriptionId": subscription_id,
                    "numberOfDatabases": databases.len(),
                    "databases": databases
                }]
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock a specific database get endpoint (GET /subscriptions/{sub_id}/databases/{db_id})
    pub async fn mock_database_get(&self, subscription_id: i32, database_id: i32, database: Value) {
        Mock::given(method("GET"))
            .and(path(format!(
                "/subscriptions/{subscription_id}/databases/{database_id}"
            )))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(database))
            .mount(&self.server)
            .await;
    }

    /// Mock database creation endpoint (POST /subscriptions/{id}/databases)
    pub async fn mock_database_create(
        &self,
        subscription_id: i32,
        task_id: &str,
        resource_id: i32,
    ) {
        Mock::given(method("POST"))
            .and(path(format!("/subscriptions/{subscription_id}/databases")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "taskId": task_id,
                "commandType": "databaseCreateRequest",
                "status": "received",
                "response": {
                    "resourceId": resource_id
                }
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock database deletion endpoint (DELETE /subscriptions/{sub_id}/databases/{db_id})
    pub async fn mock_database_delete(
        &self,
        subscription_id: i32,
        database_id: i32,
        task_id: &str,
    ) {
        Mock::given(method("DELETE"))
            .and(path(format!(
                "/subscriptions/{subscription_id}/databases/{database_id}"
            )))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "taskId": task_id,
                "commandType": "databaseDeleteRequest",
                "status": "received"
            })))
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // Task Mocks
    // =========================================================================

    /// Mock the tasks list endpoint (GET /tasks)
    ///
    /// Returns a direct array since `get_all_tasks()` returns `Result<Vec<TaskStateUpdate>>`.
    pub async fn mock_tasks_list(&self, tasks: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/tasks"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(tasks))
            .mount(&self.server)
            .await;
    }

    /// Mock a specific task get endpoint (GET /tasks/{id})
    pub async fn mock_task_get(&self, task_id: &str, task: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/tasks/{task_id}")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(task))
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // User Mocks
    // =========================================================================

    /// Mock the users list endpoint (GET /users)
    pub async fn mock_users_list(&self, users: Vec<Value>) {
        Mock::given(method("GET"))
            .and(path("/users"))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "users": users
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock a specific user get endpoint (GET /users/{id})
    pub async fn mock_user_get(&self, user_id: i32, user: Value) {
        Mock::given(method("GET"))
            .and(path(format!("/users/{user_id}")))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user))
            .mount(&self.server)
            .await;
    }

    // =========================================================================
    // Error Mocks
    // =========================================================================

    /// Mock an endpoint to return a 401 Unauthorized error
    pub async fn mock_unauthorized(&self, path_pattern: &str) {
        Mock::given(method("GET"))
            .and(path_regex(path_pattern))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized",
                "message": "Invalid API credentials"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock an endpoint to return a 404 Not Found error
    pub async fn mock_not_found(&self, path_str: &str) {
        Mock::given(method("GET"))
            .and(path(path_str))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "Not found",
                "message": "Resource not found"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock an endpoint to return a 500 Internal Server Error
    pub async fn mock_server_error(&self, path_str: &str) {
        Mock::given(method("GET"))
            .and(path(path_str))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "Internal server error",
                "message": "An unexpected error occurred"
            })))
            .mount(&self.server)
            .await;
    }

    /// Mock an endpoint to return a 429 Rate Limited error
    pub async fn mock_rate_limited(&self, path_str: &str, retry_after: u32) {
        Mock::given(method("GET"))
            .and(path(path_str))
            .and(header("x-api-key", "test-key"))
            .and(header("x-api-secret-key", "test-secret"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", retry_after.to_string())
                    .set_body_json(json!({
                        "error": "Rate limited",
                        "message": "Too many requests"
                    })),
            )
            .mount(&self.server)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::fixtures::{DatabaseFixture, SubscriptionFixture, TaskFixture};
    use crate::{AccountHandler, DatabaseHandler, SubscriptionHandler, TaskHandler};

    #[tokio::test]
    async fn test_mock_server_start() {
        let server = MockCloudServer::start().await;
        assert!(!server.uri().is_empty());
    }

    #[tokio::test]
    async fn test_mock_server_client() {
        let server = MockCloudServer::start().await;
        let _client = server.client();
        // Client created successfully
    }

    #[tokio::test]
    async fn test_mock_account() {
        let server = MockCloudServer::start().await;

        server
            .mock_account(json!({
                "account": {
                    "id": 12345,
                    "name": "Test Account"
                },
                "links": []
            }))
            .await;

        let client = server.client();
        let handler = AccountHandler::new(client);
        let result = handler.get_current_account().await.unwrap();

        assert!(result.account.is_some());
        assert_eq!(result.account.unwrap().id, Some(12345));
    }

    #[tokio::test]
    async fn test_mock_not_found() {
        let server = MockCloudServer::start().await;
        server.mock_not_found("/subscriptions/999").await;

        let client = server.client();
        let result = client.get::<serde_json::Value>("/subscriptions/999").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_subscriptions_list_with_handler() {
        let server = MockCloudServer::start().await;

        server
            .mock_subscriptions_list(vec![
                SubscriptionFixture::new(123, "Production").build(),
                SubscriptionFixture::new(456, "Staging").build(),
            ])
            .await;

        let client = server.client();
        let handler = SubscriptionHandler::new(client);
        let result = handler.get_all_subscriptions().await.unwrap();

        assert!(result.subscriptions.is_some());
        let subs = result.subscriptions.unwrap();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].id, Some(123));
        assert_eq!(subs[1].id, Some(456));
    }

    #[tokio::test]
    async fn test_mock_databases_list_with_handler() {
        let server = MockCloudServer::start().await;

        server
            .mock_databases_list(
                123,
                vec![
                    DatabaseFixture::new(1, "cache-db").build(),
                    DatabaseFixture::new(2, "session-db").build(),
                ],
            )
            .await;

        let client = server.client();
        let handler = DatabaseHandler::new(client);
        let result = handler
            .get_subscription_databases(123, None, None)
            .await
            .unwrap();

        assert!(!result.subscription.is_empty());
        let sub_info = &result.subscription[0];
        assert_eq!(sub_info.subscription_id, 123);
        assert_eq!(sub_info.databases.len(), 2);
        assert_eq!(sub_info.databases[0].name, Some("cache-db".to_string()));
    }

    #[tokio::test]
    async fn test_mock_tasks_list_with_handler() {
        let server = MockCloudServer::start().await;

        server
            .mock_tasks_list(vec![
                TaskFixture::new("task-1")
                    .status("processing-completed")
                    .build(),
                TaskFixture::new("task-2")
                    .status("processing-in-progress")
                    .build(),
            ])
            .await;

        let client = server.client();
        let handler = TaskHandler::new(client);
        let result = handler.get_all_tasks().await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].task_id, Some("task-1".to_string()));
        assert_eq!(result[1].task_id, Some("task-2".to_string()));
    }
}
