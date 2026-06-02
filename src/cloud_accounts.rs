//! Cloud provider account management operations and models
//!
//! This module handles the integration between Redis Cloud and your cloud provider
//! accounts (AWS, GCP, Azure). It manages cloud account credentials, access keys,
//! and provider-specific configurations.
//!
//! # Overview
//!
//! Cloud accounts are the bridge between Redis Cloud and your infrastructure provider.
//! They store the credentials and permissions needed for Redis Cloud to provision
//! resources in your cloud environment.
//!
//! # Supported Providers
//!
//! - **AWS**: Amazon Web Services accounts with IAM roles or access keys
//! - **GCP**: Google Cloud Platform projects with service accounts
//! - **Azure**: Microsoft Azure subscriptions with service principals
//!
//! # Key Features
//!
//! - **Account Registration**: Register cloud provider accounts with Redis Cloud
//! - **Credential Management**: Securely store and manage cloud credentials
//! - **Access Key Operations**: Create, update, and delete cloud access keys
//! - **Provider Details**: Retrieve provider-specific account information
//! - **Multi-cloud Support**: Manage accounts across different cloud providers
//!
//! # API Reference
//!
//! All operations in this module map to the Redis Cloud REST API's Cloud Accounts endpoints.
//! For detailed API documentation, see the [Redis Cloud OpenAPI Specification].
//!
//! [Redis Cloud OpenAPI Specification]: https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, CloudAccountHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = CloudAccountHandler::new(client);
//!
//! // List all cloud accounts
//! let accounts = handler.get_cloud_accounts().await?;
//!
//! // Get specific account details (account ID 123)
//! let account = handler.get_cloud_account_by_id(123).await?;
//! # Ok(())
//! # }
//! ```

use crate::types::Link;
pub use crate::types::TaskStateUpdate;
use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

// ============================================================================
// Models
// ============================================================================

/// Cloud account update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAccountUpdateRequest {
    /// Cloud account display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Cloud account ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_account_id: Option<i32>,

    /// Cloud provider access key.
    pub access_key_id: String,

    /// Cloud provider secret key.
    pub access_secret_key: String,

    /// Cloud provider management console username.
    pub console_username: String,

    /// Cloud provider management console password.
    pub console_password: String,

    /// Optional. Cloud provider management console login URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign_in_login_url: Option<String>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_CLOUD_ACCOUNT"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Cloud provider account information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAccount {
    /// Cloud account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Cloud account display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Account status (e.g., "active", "error")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Cloud provider (e.g., "AWS", "GCP", "Azure")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Cloud provider access key ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,

    /// Cloud provider secret key (typically masked in responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_secret_key: Option<String>,

    /// AWS Console Role ARN (AWS-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_console_role_arn: Option<String>,

    /// AWS User ARN (AWS-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_user_arn: Option<String>,

    /// Cloud provider management console username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_username: Option<String>,

    /// Cloud provider management console password (typically masked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_password: Option<String>,

    /// Cloud provider management console login URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign_in_login_url: Option<String>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Cloud account create request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAccountCreateRequest {
    /// Cloud account display name.
    pub name: String,

    /// Optional. Cloud provider. Default: 'AWS'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Cloud provider access key.
    pub access_key_id: String,

    /// Cloud provider secret key.
    pub access_secret_key: String,

    /// Cloud provider management console username.
    pub console_username: String,

    /// Cloud provider management console password.
    pub console_password: String,

    /// Cloud provider management console login URL.
    pub sign_in_login_url: String,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"CREATE_CLOUD_ACCOUNT"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Cloud accounts response
///
/// Response from GET /cloud-accounts containing list of cloud provider integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudAccounts {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of cloud provider accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_accounts: Option<Vec<CloudAccount>>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

// ============================================================================
// Handler
// ============================================================================

/// Handler for cloud provider account operations
///
/// Manages integration with AWS, GCP, and Azure accounts, including
/// credential management and provider-specific configurations.
pub struct CloudAccountsHandler {
    client: CloudClient,
}

impl CloudAccountsHandler {
    /// Create a new handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get cloud accounts
    ///
    /// Gets a list of all configured cloud accounts.
    ///
    /// # API Endpoint
    ///
    /// `GET /cloud-accounts`
    ///
    /// See [OpenAPI Spec](https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json) - `getCloudAccounts`
    pub async fn get_cloud_accounts(&self) -> Result<CloudAccounts> {
        self.client.get("/cloud-accounts").await
    }

    /// Create cloud account
    ///
    /// Creates a cloud account.
    ///
    /// # API Endpoint
    ///
    /// `POST /cloud-accounts`
    ///
    /// See [OpenAPI Spec](https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json) - `createCloudAccount`
    pub async fn create_cloud_account(
        &self,
        request: &CloudAccountCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client.post("/cloud-accounts", request).await
    }

    /// Delete cloud account
    ///
    /// Deletes a cloud account.
    ///
    /// # API Endpoint
    ///
    /// `DELETE /cloud-accounts/{cloudAccountId}`
    ///
    /// See [OpenAPI Spec](https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json) - `deleteCloudAccount`
    pub async fn delete_cloud_account(&self, cloud_account_id: i32) -> Result<TaskStateUpdate> {
        let response = self
            .client
            .delete_raw(&format!("/cloud-accounts/{cloud_account_id}"))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Get a single cloud account
    ///
    /// Gets details on a single cloud account.
    ///
    /// # API Endpoint
    ///
    /// `GET /cloud-accounts/{cloudAccountId}`
    ///
    /// See [OpenAPI Spec](https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json) - `getCloudAccountById`
    pub async fn get_cloud_account_by_id(&self, cloud_account_id: i32) -> Result<CloudAccount> {
        self.client
            .get(&format!("/cloud-accounts/{cloud_account_id}"))
            .await
    }

    /// Update cloud account
    ///
    /// Updates cloud account details.
    ///
    /// # API Endpoint
    ///
    /// `PUT /cloud-accounts/{cloudAccountId}`
    ///
    /// See [OpenAPI Spec](https://redis.io/docs/latest/operate/rc/api/api-reference/openapi.json) - `updateCloudAccount`
    pub async fn update_cloud_account(
        &self,
        cloud_account_id: i32,
        request: &CloudAccountUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/cloud-accounts/{cloud_account_id}"), request)
            .await
    }

    // ============================================================================
    // Simplified aliases
    // ============================================================================

    /// List cloud accounts (simplified)
    ///
    /// Alias for [`get_cloud_accounts`](Self::get_cloud_accounts).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let accounts = client.cloud_accounts().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<CloudAccounts> {
        self.get_cloud_accounts().await
    }

    /// Create a cloud account (simplified)
    ///
    /// Alias for [`create_cloud_account`](Self::create_cloud_account).
    ///
    /// # Arguments
    ///
    /// * `request` - The cloud account creation request
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    /// use redis_cloud::cloud_accounts::CloudAccountCreateRequest;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let request = CloudAccountCreateRequest {
    ///     name: "my-aws-account".to_string(),
    ///     provider: Some("AWS".to_string()),
    ///     access_key_id: "key".to_string(),
    ///     access_secret_key: "secret".to_string(),
    ///     console_username: "user".to_string(),
    ///     console_password: "pass".to_string(),
    ///     sign_in_login_url: "https://console.aws.amazon.com".to_string(),
    ///     command_type: None,
    /// };
    ///
    /// let task = client.cloud_accounts().create(&request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: &CloudAccountCreateRequest) -> Result<TaskStateUpdate> {
        self.create_cloud_account(request).await
    }

    /// Delete a cloud account (simplified)
    ///
    /// Alias for [`delete_cloud_account`](Self::delete_cloud_account).
    ///
    /// # Arguments
    ///
    /// * `cloud_account_id` - The cloud account ID
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let task = client.cloud_accounts().delete(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, cloud_account_id: i32) -> Result<TaskStateUpdate> {
        self.delete_cloud_account(cloud_account_id).await
    }

    /// Get a cloud account by ID (simplified)
    ///
    /// Alias for [`get_cloud_account_by_id`](Self::get_cloud_account_by_id).
    ///
    /// # Arguments
    ///
    /// * `cloud_account_id` - The cloud account ID
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let account = client.cloud_accounts().get(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, cloud_account_id: i32) -> Result<CloudAccount> {
        self.get_cloud_account_by_id(cloud_account_id).await
    }

    /// Update a cloud account (simplified)
    ///
    /// Alias for [`update_cloud_account`](Self::update_cloud_account).
    ///
    /// # Arguments
    ///
    /// * `cloud_account_id` - The cloud account ID
    /// * `request` - The cloud account update request
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    /// use redis_cloud::cloud_accounts::CloudAccountUpdateRequest;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let request = CloudAccountUpdateRequest {
    ///     name: Some("renamed-account".to_string()),
    ///     cloud_account_id: None,
    ///     access_key_id: "key".to_string(),
    ///     access_secret_key: "secret".to_string(),
    ///     console_username: "user".to_string(),
    ///     console_password: "pass".to_string(),
    ///     sign_in_login_url: None,
    ///     command_type: None,
    /// };
    ///
    /// let task = client.cloud_accounts().update(123, &request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        cloud_account_id: i32,
        request: &CloudAccountUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.update_cloud_account(cloud_account_id, request).await
    }
}
