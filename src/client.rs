//! Redis Cloud API client core implementation
//!
//! This module contains the core HTTP client for interacting with the Redis Cloud REST API.
//! It provides authentication handling, request/response processing, and error management.
//!
//! The client is designed around a builder pattern for flexible configuration and supports
//! both typed and untyped API interactions.

use crate::{CloudError as RestError, Result};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, instrument, trace};

/// Default user agent for the Redis Cloud client
const DEFAULT_USER_AGENT: &str = concat!("redis-cloud/", env!("CARGO_PKG_VERSION"));

/// Builder for constructing a `CloudClient` with custom configuration
///
/// Provides a fluent interface for configuring API credentials, base URL, timeouts,
/// and other client settings before creating the final `CloudClient` instance.
///
/// # Examples
///
/// ```rust,no_run
/// use redis_cloud::CloudClient;
///
/// // Basic configuration
/// let client = CloudClient::builder()
///     .api_key("your-api-key")
///     .api_secret("your-api-secret")
///     .build()?;
///
/// // Advanced configuration
/// let client = CloudClient::builder()
///     .api_key("your-api-key")
///     .api_secret("your-api-secret")
///     .base_url("https://api.redislabs.com/v1".to_string())
///     .timeout(std::time::Duration::from_secs(120))
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct CloudClientBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
    base_url: String,
    timeout: std::time::Duration,
    user_agent: String,
}

impl Default for CloudClientBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            base_url: "https://api.redislabs.com/v1".to_string(),
            timeout: std::time::Duration::from_secs(30),
            user_agent: DEFAULT_USER_AGENT.to_string(),
        }
    }
}

impl CloudClientBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API key
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the API secret
    #[must_use]
    pub fn api_secret(mut self, secret: impl Into<String>) -> Self {
        self.api_secret = Some(secret.into());
        self
    }

    /// Set the base URL
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the timeout
    #[must_use]
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the user agent string for HTTP requests
    ///
    /// The default user agent is `redis-cloud/{version}`.
    /// This can be overridden to identify specific clients, for example:
    /// `redisctl/1.2.3` or `my-app/1.0.0`.
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Build the client
    pub fn build(self) -> Result<CloudClient> {
        let api_key = self
            .api_key
            .ok_or_else(|| RestError::ConnectionError("API key is required".to_string()))?;
        let api_secret = self
            .api_secret
            .ok_or_else(|| RestError::ConnectionError("API secret is required".to_string()))?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent)
                .map_err(|e| RestError::ConnectionError(format!("Invalid user agent: {e}")))?,
        );

        let client = Client::builder()
            .timeout(self.timeout)
            .default_headers(default_headers)
            .build()
            .map_err(|e| RestError::ConnectionError(e.to_string()))?;

        Ok(CloudClient {
            api_key,
            api_secret,
            base_url: self.base_url,
            timeout: self.timeout,
            client: Arc::new(client),
        })
    }
}

/// Redis Cloud API client
#[derive(Clone)]
pub struct CloudClient {
    pub(crate) api_key: String,
    pub(crate) api_secret: String,
    pub(crate) base_url: String,
    pub(crate) timeout: std::time::Duration,
    pub(crate) client: Arc<Client>,
}

impl CloudClient {
    /// Create a new builder for the client
    #[must_use]
    pub fn builder() -> CloudClientBuilder {
        CloudClientBuilder::new()
    }

    /// Get the configured request timeout
    ///
    /// Returns the timeout duration that was set when building the client.
    /// This timeout is applied to all HTTP requests made by this client.
    #[must_use]
    pub fn timeout(&self) -> std::time::Duration {
        self.timeout
    }

    // ========================================================================
    // Fluent API - Handler accessors
    // ========================================================================

    /// Get an account handler for account management operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let account = client.account().get_current_account().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn account(&self) -> crate::AccountHandler {
        crate::AccountHandler::new(self.clone())
    }

    /// Get a subscription handler for Pro subscription operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let subscriptions = client.subscriptions().get_all_subscriptions().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn subscriptions(&self) -> crate::SubscriptionHandler {
        crate::SubscriptionHandler::new(self.clone())
    }

    /// Get a database handler for Pro database operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let databases = client.databases().get_subscription_databases(123, None, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn databases(&self) -> crate::DatabaseHandler {
        crate::DatabaseHandler::new(self.clone())
    }

    /// Get a fixed subscription handler for Essentials subscription operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let subscriptions = client.fixed_subscriptions().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn fixed_subscriptions(&self) -> crate::FixedSubscriptionHandler {
        crate::FixedSubscriptionHandler::new(self.clone())
    }

    /// Get a fixed database handler for Essentials database operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let databases = client.fixed_databases().list(123, None, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn fixed_databases(&self) -> crate::FixedDatabaseHandler {
        crate::FixedDatabaseHandler::new(self.clone())
    }

    /// Get an ACL handler for access control operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let users = client.acl().get_all_acl_users().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn acl(&self) -> crate::AclHandler {
        crate::AclHandler::new(self.clone())
    }

    /// Get a users handler for user management operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let users = client.users().get_all_users().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn users(&self) -> crate::UserHandler {
        crate::UserHandler::new(self.clone())
    }

    /// Get a tasks handler for async operation tracking
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let tasks = client.tasks().get_all_tasks().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn tasks(&self) -> crate::TaskHandler {
        crate::TaskHandler::new(self.clone())
    }

    /// Get a cloud accounts handler for cloud provider integration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let accounts = client.cloud_accounts().get_cloud_accounts().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn cloud_accounts(&self) -> crate::CloudAccountHandler {
        crate::CloudAccountHandler::new(self.clone())
    }

    /// Get a VPC peering handler for VPC peering operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let peering = client.vpc_peering().get(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn vpc_peering(&self) -> crate::VpcPeeringHandler {
        crate::VpcPeeringHandler::new(self.clone())
    }

    /// Get a transit gateway handler for AWS Transit Gateway operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let attachments = client.transit_gateway().get_attachments(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn transit_gateway(&self) -> crate::TransitGatewayHandler {
        crate::TransitGatewayHandler::new(self.clone())
    }

    /// Get a Private Service Connect handler for GCP PSC operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let service = client.psc().get_service(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn psc(&self) -> crate::PscHandler {
        crate::PscHandler::new(self.clone())
    }

    /// Get a `PrivateLink` handler for AWS `PrivateLink` operations
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let config = client.private_link().get(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn private_link(&self) -> crate::PrivateLinkHandler {
        crate::PrivateLinkHandler::new(self.clone())
    }

    /// Get a cost report handler for generating cost reports
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use redis_cloud::CloudClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CloudClient::builder()
    ///     .api_key("key")
    ///     .api_secret("secret")
    ///     .build()?;
    ///
    /// let handler = client.cost_reports();
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn cost_reports(&self) -> crate::CostReportHandler {
        crate::CostReportHandler::new(self.clone())
    }

    /// Normalize URL path concatenation to avoid double slashes
    fn normalize_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }

    /// Convert HTTP status code and response text to appropriate error
    ///
    /// This is a helper to avoid duplicating the error handling pattern
    /// across multiple methods.
    fn status_to_error(status: reqwest::StatusCode, text: String) -> RestError {
        match status.as_u16() {
            400 => RestError::BadRequest { message: text },
            401 => RestError::AuthenticationFailed { message: text },
            403 => RestError::Forbidden { message: text },
            404 => RestError::NotFound { message: text },
            412 => RestError::PreconditionFailed,
            429 => RestError::RateLimited { message: text },
            500 => RestError::InternalServerError { message: text },
            503 => RestError::ServiceUnavailable { message: text },
            _ => RestError::ApiError {
                code: status.as_u16(),
                message: text,
            },
        }
    }

    /// Make a GET request with API key authentication
    #[instrument(skip(self), fields(method = "GET"))]
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.normalize_url(path);
        debug!("GET {}", url);

        // Redis Cloud API uses these headers for authentication
        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    /// Make a POST request
    #[instrument(skip(self, body), fields(method = "POST"))]
    pub async fn post<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.normalize_url(path);
        debug!("POST {}", url);
        trace!("Request body: {:?}", serde_json::to_value(body).ok());

        // Same backwards header naming as GET
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .json(body)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    /// Make a PUT request
    #[instrument(skip(self, body), fields(method = "PUT"))]
    pub async fn put<B: Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.normalize_url(path);
        debug!("PUT {}", url);
        trace!("Request body: {:?}", serde_json::to_value(body).ok());

        // Same backwards header naming as GET
        let response = self
            .client
            .put(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .json(body)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    /// Make a DELETE request
    #[instrument(skip(self), fields(method = "DELETE"))]
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.normalize_url(path);
        debug!("DELETE {}", url);

        // Same backwards header naming as GET
        let response = self
            .client
            .delete(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            Err(Self::status_to_error(status, text))
        }
    }

    /// Execute raw GET request returning JSON Value
    #[instrument(skip(self), fields(method = "GET"))]
    pub async fn get_raw(&self, path: &str) -> Result<serde_json::Value> {
        self.get(path).await
    }

    /// Execute GET request returning raw bytes
    ///
    /// Useful for downloading binary content like cost reports or other files.
    #[instrument(skip(self), fields(method = "GET"))]
    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let url = self.normalize_url(path);
        debug!("GET {} (bytes)", url);

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        let status = response.status();

        if status.is_success() {
            response
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| RestError::ConnectionError(format!("Failed to read response: {e}")))
        } else {
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            Err(Self::status_to_error(status, text))
        }
    }

    /// Execute raw POST request with JSON body
    #[instrument(skip(self, body), fields(method = "POST"))]
    pub async fn post_raw(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.post(path, &body).await
    }

    /// Execute raw PUT request with JSON body
    #[instrument(skip(self, body), fields(method = "PUT"))]
    pub async fn put_raw(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.put(path, &body).await
    }

    /// Execute raw PATCH request with JSON body
    #[instrument(skip(self, body), fields(method = "PATCH"))]
    pub async fn patch_raw(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = self.normalize_url(path);
        debug!("PATCH {}", url);
        trace!("Request body: {:?}", body);

        // Use backwards header names for compatibility
        let response = self
            .client
            .patch(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .json(&body)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    /// Execute raw DELETE request returning any response body
    #[instrument(skip(self), fields(method = "DELETE"))]
    pub async fn delete_raw(&self, path: &str) -> Result<serde_json::Value> {
        let url = self.normalize_url(path);
        debug!("DELETE {}", url);

        // Use backwards header names for compatibility
        let response = self
            .client
            .delete(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        if response.status().is_success() {
            if response.content_length() == Some(0) {
                Ok(serde_json::json!({"status": "deleted"}))
            } else {
                response.json().await.map_err(Into::into)
            }
        } else {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            Err(Self::status_to_error(status, text))
        }
    }

    /// Execute DELETE request with JSON body (used by some endpoints like `PrivateLink` principals)
    #[instrument(skip(self, body), fields(method = "DELETE"))]
    pub async fn delete_with_body<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        let url = self.normalize_url(path);
        debug!("DELETE {} (with body)", url);
        trace!("Request body: {:?}", body);

        let response = self
            .client
            .delete(&url)
            .header("x-api-key", &self.api_key)
            .header("x-api-secret-key", &self.api_secret)
            .json(&body)
            .send()
            .await?;

        trace!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    /// Handle HTTP response and return both status code and body as JSON
    ///
    /// This is used internally by the Tower service implementation to preserve
    /// the actual HTTP status code in responses.
    #[cfg(feature = "tower-integration")]
    async fn handle_response_with_status(
        &self,
        response: reqwest::Response,
    ) -> Result<(u16, serde_json::Value)> {
        let status = response.status();
        let status_code = status.as_u16();

        if status.is_success() {
            let bytes = response
                .bytes()
                .await
                .map_err(|e| RestError::ConnectionError(format!("Failed to read response: {e}")))?;

            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
                RestError::ConnectionError(format!("Failed to parse JSON response: {e}"))
            })?;

            Ok((status_code, value))
        } else {
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            Err(Self::status_to_error(status, text))
        }
    }

    /// Handle HTTP response
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            // Get the response bytes for better error reporting
            let bytes = response
                .bytes()
                .await
                .map_err(|e| RestError::ConnectionError(format!("Failed to read response: {e}")))?;

            // Use serde_path_to_error for better deserialization error messages
            let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);
            serde_path_to_error::deserialize(deserializer).map_err(|err| {
                let path = err.path().to_string();
                // Use ConnectionError to provide detailed error message with field path
                RestError::ConnectionError(format!(
                    "Failed to deserialize field '{}': {}",
                    path,
                    err.inner()
                ))
            })
        } else {
            let text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(failed to read response body: {e})"));
            Err(Self::status_to_error(status, text))
        }
    }
}

/// Tower Service integration for `CloudClient`
///
/// This module provides Tower Service implementations for `CloudClient`, enabling
/// middleware composition with patterns like circuit breakers, retry, and rate limiting.
///
/// # Feature Flag
///
/// This module is only available when the `tower-integration` feature is enabled.
///
/// # Examples
///
/// ```rust,ignore
/// use redis_cloud::CloudClient;
/// use redis_cloud::tower_support::ApiRequest;
/// use tower::ServiceExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = CloudClient::builder()
///     .api_key("your-key")
///     .api_secret("your-secret")
///     .build()?;
///
/// // Convert to a Tower service
/// let mut service = client.into_service();
///
/// // Use the service
/// let response = service.oneshot(ApiRequest::get("/subscriptions")).await?;
/// println!("Status: {}", response.status);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tower-integration")]
pub mod tower_support {
    use super::{CloudClient, RestError, Result};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::Service;

    /// HTTP method for API requests
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Method {
        /// GET request
        Get,
        /// POST request
        Post,
        /// PUT request
        Put,
        /// PATCH request
        Patch,
        /// DELETE request
        Delete,
    }

    /// Tower-compatible request type for Redis Cloud API
    ///
    /// This wraps the essential components of an API request in a format
    /// suitable for Tower middleware composition.
    #[derive(Debug, Clone)]
    pub struct ApiRequest {
        /// HTTP method
        pub method: Method,
        /// API endpoint path (e.g., "/subscriptions")
        pub path: String,
        /// Optional JSON body for POST/PUT/PATCH requests
        pub body: Option<serde_json::Value>,
    }

    impl ApiRequest {
        /// Create a GET request
        pub fn get(path: impl Into<String>) -> Self {
            Self {
                method: Method::Get,
                path: path.into(),
                body: None,
            }
        }

        /// Create a POST request with a JSON body
        pub fn post(path: impl Into<String>, body: serde_json::Value) -> Self {
            Self {
                method: Method::Post,
                path: path.into(),
                body: Some(body),
            }
        }

        /// Create a PUT request with a JSON body
        pub fn put(path: impl Into<String>, body: serde_json::Value) -> Self {
            Self {
                method: Method::Put,
                path: path.into(),
                body: Some(body),
            }
        }

        /// Create a PATCH request with a JSON body
        pub fn patch(path: impl Into<String>, body: serde_json::Value) -> Self {
            Self {
                method: Method::Patch,
                path: path.into(),
                body: Some(body),
            }
        }

        /// Create a DELETE request
        pub fn delete(path: impl Into<String>) -> Self {
            Self {
                method: Method::Delete,
                path: path.into(),
                body: None,
            }
        }
    }

    /// Tower-compatible response type
    ///
    /// Contains the HTTP status code and response body as JSON.
    #[derive(Debug, Clone)]
    pub struct ApiResponse {
        /// HTTP status code
        pub status: u16,
        /// Response body as JSON
        pub body: serde_json::Value,
    }

    impl CloudClient {
        /// Convert this client into a Tower service
        ///
        /// This consumes the client and returns it wrapped in a Tower service
        /// implementation, enabling middleware composition.
        ///
        /// # Examples
        ///
        /// ```rust,ignore
        /// use redis_cloud::CloudClient;
        /// use tower::ServiceExt;
        /// use redis_cloud::tower_support::ApiRequest;
        ///
        /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
        /// let client = CloudClient::builder()
        ///     .api_key("key")
        ///     .api_secret("secret")
        ///     .build()?;
        ///
        /// let mut service = client.into_service();
        /// let response = service.oneshot(ApiRequest::get("/subscriptions")).await?;
        /// # Ok(())
        /// # }
        /// ```
        #[must_use]
        pub fn into_service(self) -> Self {
            self
        }
    }

    impl Service<ApiRequest> for CloudClient {
        type Response = ApiResponse;
        type Error = RestError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            // CloudClient is always ready since it uses an internal connection pool
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: ApiRequest) -> Self::Future {
            let client = self.clone();
            Box::pin(async move {
                let url = client.normalize_url(&req.path);

                let request_builder = match req.method {
                    Method::Get => client.client.get(&url),
                    Method::Post => {
                        let body = req.body.ok_or_else(|| RestError::BadRequest {
                            message: "POST request requires a body".to_string(),
                        })?;
                        client.client.post(&url).json(&body)
                    }
                    Method::Put => {
                        let body = req.body.ok_or_else(|| RestError::BadRequest {
                            message: "PUT request requires a body".to_string(),
                        })?;
                        client.client.put(&url).json(&body)
                    }
                    Method::Patch => {
                        let body = req.body.ok_or_else(|| RestError::BadRequest {
                            message: "PATCH request requires a body".to_string(),
                        })?;
                        client.client.patch(&url).json(&body)
                    }
                    Method::Delete => client.client.delete(&url),
                };

                let response = request_builder
                    .header("x-api-key", &client.api_key)
                    .header("x-api-secret-key", &client.api_secret)
                    .send()
                    .await?;

                let (status, body) = client.handle_response_with_status(response).await?;

                Ok(ApiResponse { status, body })
            })
        }
    }
}
