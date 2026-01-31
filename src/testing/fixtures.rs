//! Builder-pattern fixtures for Redis Cloud API responses

use serde_json::{Value, json};

/// Fixture for building subscription responses
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::SubscriptionFixture;
///
/// let subscription = SubscriptionFixture::new(123, "Production")
///     .status("active")
///     .payment_method_type("credit-card")
///     .build();
/// ```
pub struct SubscriptionFixture {
    id: i32,
    name: String,
    status: String,
    payment_method_type: Option<String>,
    memory_storage: Option<String>,
    cloud_provider: Option<String>,
    region: Option<String>,
}

impl SubscriptionFixture {
    /// Create a new subscription fixture with required fields
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            status: "active".to_string(),
            payment_method_type: None,
            memory_storage: None,
            cloud_provider: None,
            region: None,
        }
    }

    /// Set the subscription status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set the payment method type
    pub fn payment_method_type(mut self, payment_method_type: impl Into<String>) -> Self {
        self.payment_method_type = Some(payment_method_type.into());
        self
    }

    /// Set the memory storage type
    pub fn memory_storage(mut self, memory_storage: impl Into<String>) -> Self {
        self.memory_storage = Some(memory_storage.into());
        self
    }

    /// Set the cloud provider
    pub fn cloud_provider(mut self, provider: impl Into<String>) -> Self {
        self.cloud_provider = Some(provider.into());
        self
    }

    /// Set the region
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Build the subscription as a JSON Value
    pub fn build(self) -> Value {
        let mut sub = json!({
            "id": self.id,
            "name": self.name,
            "status": self.status
        });

        if let Some(pmt) = self.payment_method_type {
            sub["paymentMethodType"] = json!(pmt);
        }
        if let Some(ms) = self.memory_storage {
            sub["memoryStorage"] = json!(ms);
        }
        if let Some(provider) = self.cloud_provider {
            sub["cloudProviders"] = json!([{
                "provider": provider,
                "regions": self.region.map(|r| json!([{"region": r}])).unwrap_or(json!([]))
            }]);
        }

        sub
    }
}

/// Fixture for building database responses
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::DatabaseFixture;
///
/// let database = DatabaseFixture::new(456, "my-redis-db")
///     .memory_limit_in_gb(1.0)
///     .protocol("redis")
///     .build();
/// ```
pub struct DatabaseFixture {
    database_id: i32,
    name: String,
    status: String,
    memory_limit_in_gb: f64,
    protocol: Option<String>,
    data_persistence: Option<String>,
    replication: Option<bool>,
    throughput_measurement: Option<Value>,
    public_endpoint: Option<String>,
    private_endpoint: Option<String>,
}

impl DatabaseFixture {
    /// Create a new database fixture with required fields
    pub fn new(database_id: i32, name: impl Into<String>) -> Self {
        Self {
            database_id,
            name: name.into(),
            status: "active".to_string(),
            memory_limit_in_gb: 1.0,
            protocol: None,
            data_persistence: None,
            replication: None,
            throughput_measurement: None,
            public_endpoint: None,
            private_endpoint: None,
        }
    }

    /// Set the database status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set the memory limit in GB
    pub fn memory_limit_in_gb(mut self, limit: f64) -> Self {
        self.memory_limit_in_gb = limit;
        self
    }

    /// Set the database protocol
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Set the data persistence option
    pub fn data_persistence(mut self, persistence: impl Into<String>) -> Self {
        self.data_persistence = Some(persistence.into());
        self
    }

    /// Set whether replication is enabled
    pub fn replication(mut self, enabled: bool) -> Self {
        self.replication = Some(enabled);
        self
    }

    /// Set the throughput measurement
    pub fn throughput(mut self, by: &str, value: i32) -> Self {
        self.throughput_measurement = Some(json!({
            "by": by,
            "value": value
        }));
        self
    }

    /// Set the public endpoint
    pub fn public_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.public_endpoint = Some(endpoint.into());
        self
    }

    /// Set the private endpoint
    pub fn private_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.private_endpoint = Some(endpoint.into());
        self
    }

    /// Build the database as a JSON Value
    pub fn build(self) -> Value {
        let mut db = json!({
            "databaseId": self.database_id,
            "name": self.name,
            "status": self.status,
            "memoryLimitInGb": self.memory_limit_in_gb
        });

        if let Some(protocol) = self.protocol {
            db["protocol"] = json!(protocol);
        }
        if let Some(persistence) = self.data_persistence {
            db["dataPersistence"] = json!(persistence);
        }
        if let Some(replication) = self.replication {
            db["replication"] = json!(replication);
        }
        if let Some(throughput) = self.throughput_measurement {
            db["throughputMeasurement"] = throughput;
        }
        if let Some(endpoint) = self.public_endpoint {
            db["publicEndpoint"] = json!(endpoint);
        }
        if let Some(endpoint) = self.private_endpoint {
            db["privateEndpoint"] = json!(endpoint);
        }

        db
    }
}

/// Fixture for building task responses
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::TaskFixture;
///
/// let task = TaskFixture::new("task-123")
///     .command_type("subscriptionCreateRequest")
///     .status("processing-completed")
///     .resource_id(456)
///     .build();
/// ```
pub struct TaskFixture {
    task_id: String,
    command_type: Option<String>,
    status: String,
    description: Option<String>,
    resource_id: Option<i32>,
    error: Option<String>,
}

impl TaskFixture {
    /// Create a new task fixture with required fields
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            command_type: None,
            status: "processing-completed".to_string(),
            description: None,
            resource_id: None,
            error: None,
        }
    }

    /// Create a completed task fixture
    pub fn completed(task_id: impl Into<String>, resource_id: i32) -> Self {
        Self {
            task_id: task_id.into(),
            command_type: None,
            status: "processing-completed".to_string(),
            description: Some("Task completed successfully".to_string()),
            resource_id: Some(resource_id),
            error: None,
        }
    }

    /// Create a failed task fixture
    pub fn failed(task_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            command_type: None,
            status: "processing-error".to_string(),
            description: Some("Task failed".to_string()),
            resource_id: None,
            error: Some(error.into()),
        }
    }

    /// Set the command type
    pub fn command_type(mut self, command_type: impl Into<String>) -> Self {
        self.command_type = Some(command_type.into());
        self
    }

    /// Set the task status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Set the task description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the resource ID in the response
    pub fn resource_id(mut self, resource_id: i32) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    /// Set an error message
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Build the task as a JSON Value
    pub fn build(self) -> Value {
        let mut task = json!({
            "taskId": self.task_id,
            "status": self.status
        });

        if let Some(ct) = self.command_type {
            task["commandType"] = json!(ct);
        }
        if let Some(desc) = self.description {
            task["description"] = json!(desc);
        }

        let mut response = json!({});
        if let Some(rid) = self.resource_id {
            response["resourceId"] = json!(rid);
        }
        if let Some(err) = self.error {
            response["error"] = json!(err);
        }
        if !response.as_object().unwrap().is_empty() {
            task["response"] = response;
        }

        task
    }
}

/// Fixture for building account responses
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::AccountFixture;
///
/// let account = AccountFixture::new(12345, "My Account")
///     .marketplace_status("active")
///     .build();
/// ```
pub struct AccountFixture {
    id: i32,
    name: String,
    marketplace_status: Option<String>,
    created_timestamp: Option<String>,
    updated_timestamp: Option<String>,
}

impl AccountFixture {
    /// Create a new account fixture with required fields
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            marketplace_status: None,
            created_timestamp: None,
            updated_timestamp: None,
        }
    }

    /// Set the marketplace status
    pub fn marketplace_status(mut self, status: impl Into<String>) -> Self {
        self.marketplace_status = Some(status.into());
        self
    }

    /// Set the created timestamp
    pub fn created_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.created_timestamp = Some(timestamp.into());
        self
    }

    /// Set the updated timestamp
    pub fn updated_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.updated_timestamp = Some(timestamp.into());
        self
    }

    /// Build the account as a JSON Value (wrapped in RootAccount format)
    pub fn build(self) -> Value {
        let mut account = json!({
            "id": self.id,
            "name": self.name
        });

        if let Some(status) = self.marketplace_status {
            account["marketplaceStatus"] = json!(status);
        }
        if let Some(ts) = self.created_timestamp {
            account["createdTimestamp"] = json!(ts);
        }
        if let Some(ts) = self.updated_timestamp {
            account["updatedTimestamp"] = json!(ts);
        }

        json!({
            "account": account,
            "links": []
        })
    }

    /// Build just the account object (not wrapped)
    pub fn build_account_only(self) -> Value {
        let mut account = json!({
            "id": self.id,
            "name": self.name
        });

        if let Some(status) = self.marketplace_status {
            account["marketplaceStatus"] = json!(status);
        }
        if let Some(ts) = self.created_timestamp {
            account["createdTimestamp"] = json!(ts);
        }
        if let Some(ts) = self.updated_timestamp {
            account["updatedTimestamp"] = json!(ts);
        }

        account
    }
}

/// Fixture for building user responses
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::UserFixture;
///
/// let user = UserFixture::new(1, "user@example.com")
///     .name("Test User")
///     .role("owner")
///     .build();
/// ```
pub struct UserFixture {
    id: i32,
    email: String,
    name: Option<String>,
    role: String,
    status: String,
}

impl UserFixture {
    /// Create a new user fixture with required fields
    pub fn new(id: i32, email: impl Into<String>) -> Self {
        Self {
            id,
            email: email.into(),
            name: None,
            role: "member".to_string(),
            status: "active".to_string(),
        }
    }

    /// Set the user's name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the user's role
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    /// Set the user's status
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// Build the user as a JSON Value
    pub fn build(self) -> Value {
        let mut user = json!({
            "id": self.id,
            "email": self.email,
            "role": self.role,
            "status": self.status
        });

        if let Some(name) = self.name {
            user["name"] = json!(name);
        }

        user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_fixture_defaults() {
        let sub = SubscriptionFixture::new(123, "Test").build();
        assert_eq!(sub["id"], 123);
        assert_eq!(sub["name"], "Test");
        assert_eq!(sub["status"], "active");
    }

    #[test]
    fn test_subscription_fixture_customized() {
        let sub = SubscriptionFixture::new(123, "Production")
            .status("pending")
            .payment_method_type("credit-card")
            .cloud_provider("AWS")
            .region("us-east-1")
            .build();

        assert_eq!(sub["id"], 123);
        assert_eq!(sub["status"], "pending");
        assert_eq!(sub["paymentMethodType"], "credit-card");
        assert_eq!(sub["cloudProviders"][0]["provider"], "AWS");
    }

    #[test]
    fn test_database_fixture_defaults() {
        let db = DatabaseFixture::new(456, "my-db").build();
        assert_eq!(db["databaseId"], 456);
        assert_eq!(db["name"], "my-db");
        assert_eq!(db["status"], "active");
        assert_eq!(db["memoryLimitInGb"], 1.0);
    }

    #[test]
    fn test_database_fixture_customized() {
        let db = DatabaseFixture::new(456, "my-db")
            .memory_limit_in_gb(2.5)
            .protocol("redis")
            .replication(true)
            .throughput("operations-per-second", 25000)
            .public_endpoint("redis-12345.c1.us-east-1.ec2.cloud.redislabs.com:12345")
            .build();

        assert_eq!(db["memoryLimitInGb"], 2.5);
        assert_eq!(db["protocol"], "redis");
        assert_eq!(db["replication"], true);
        assert_eq!(db["throughputMeasurement"]["by"], "operations-per-second");
        assert_eq!(db["throughputMeasurement"]["value"], 25000);
    }

    #[test]
    fn test_task_fixture_completed() {
        let task = TaskFixture::completed("task-123", 789).build();
        assert_eq!(task["taskId"], "task-123");
        assert_eq!(task["status"], "processing-completed");
        assert_eq!(task["response"]["resourceId"], 789);
    }

    #[test]
    fn test_task_fixture_failed() {
        let task = TaskFixture::failed("task-456", "Something went wrong").build();
        assert_eq!(task["taskId"], "task-456");
        assert_eq!(task["status"], "processing-error");
        assert_eq!(task["response"]["error"], "Something went wrong");
    }

    #[test]
    fn test_account_fixture() {
        let account = AccountFixture::new(12345, "My Account")
            .marketplace_status("active")
            .build();

        assert_eq!(account["account"]["id"], 12345);
        assert_eq!(account["account"]["name"], "My Account");
        assert_eq!(account["account"]["marketplaceStatus"], "active");
    }

    #[test]
    fn test_user_fixture() {
        let user = UserFixture::new(1, "user@example.com")
            .name("Test User")
            .role("owner")
            .build();

        assert_eq!(user["id"], 1);
        assert_eq!(user["email"], "user@example.com");
        assert_eq!(user["name"], "Test User");
        assert_eq!(user["role"], "owner");
    }
}
