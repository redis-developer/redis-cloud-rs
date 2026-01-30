//! Error types for Redis Cloud API client
//!
//! This module provides error handling for all API operations, including
//! typed errors for common HTTP status codes and network failures.
//!
//! # Error Types
//!
//! - `CloudError::BadRequest` - HTTP 400 errors
//! - `CloudError::AuthenticationFailed` - HTTP 401 errors
//! - `CloudError::Forbidden` - HTTP 403 errors
//! - `CloudError::NotFound` - HTTP 404 errors
//! - `CloudError::RateLimited` - HTTP 429 errors
//! - `CloudError::InternalServerError` - HTTP 500 errors
//! - `CloudError::ServiceUnavailable` - HTTP 503 errors
//!
//! # Retryable Errors
//!
//! Some errors are considered retryable (transient failures that may succeed on retry):
//! - Rate limited (429)
//! - Service unavailable (503)
//! - Connection/request errors (network issues)
//!
//! Use `CloudError::is_retryable()` to check if an error should be retried.

use thiserror::Error;

/// Errors that can occur when interacting with the Redis Cloud API
#[derive(Error, Debug, Clone)]
pub enum CloudError {
    /// HTTP request failed (network error, timeout, etc.)
    #[error("HTTP request failed: {0}")]
    Request(String),

    /// Bad Request (400) - Invalid request parameters
    #[error("Bad Request (400): {message}")]
    BadRequest {
        /// Error message from the API
        message: String,
    },

    /// Authentication failed (401) - Invalid or missing credentials
    #[error("Authentication failed (401): {message}")]
    AuthenticationFailed {
        /// Error message from the API
        message: String,
    },

    /// Forbidden (403) - Insufficient permissions
    #[error("Forbidden (403): {message}")]
    Forbidden {
        /// Error message from the API
        message: String,
    },

    /// Not Found (404) - Resource does not exist
    #[error("Not Found (404): {message}")]
    NotFound {
        /// Error message from the API
        message: String,
    },

    /// Precondition Failed (412) - Feature flag is disabled
    #[error("Precondition Failed (412): Feature flag for this flow is off")]
    PreconditionFailed,

    /// Rate Limited (429) - Too many requests
    #[error("Rate Limited (429): {message}")]
    RateLimited {
        /// Error message from the API
        message: String,
    },

    /// Internal Server Error (500) - Server-side error
    #[error("Internal Server Error (500): {message}")]
    InternalServerError {
        /// Error message from the API
        message: String,
    },

    /// Service Unavailable (503) - Server temporarily unavailable
    #[error("Service Unavailable (503): {message}")]
    ServiceUnavailable {
        /// Error message from the API
        message: String,
    },

    /// Generic API error for other HTTP status codes
    #[error("API error ({code}): {message}")]
    ApiError {
        /// HTTP status code
        code: u16,
        /// Error message from the API
        message: String,
    },

    /// Connection error (failed to establish connection)
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),
}

impl CloudError {
    /// Returns true if this error is retryable.
    ///
    /// Retryable errors include:
    /// - Rate limited (429)
    /// - Service unavailable (503)
    /// - Connection/request errors (may be transient network issues)
    ///
    /// # Examples
    ///
    /// ```
    /// use redis_cloud::CloudError;
    ///
    /// let error = CloudError::RateLimited { message: "Too many requests".to_string() };
    /// assert!(error.is_retryable());
    ///
    /// let error = CloudError::NotFound { message: "Resource not found".to_string() };
    /// assert!(!error.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CloudError::RateLimited { .. }
                | CloudError::ServiceUnavailable { .. }
                | CloudError::Request(_)
                | CloudError::ConnectionError(_)
        )
    }
}

impl From<reqwest::Error> for CloudError {
    fn from(err: reqwest::Error) -> Self {
        CloudError::Request(err.to_string())
    }
}

impl From<serde_json::Error> for CloudError {
    fn from(err: serde_json::Error) -> Self {
        CloudError::JsonError(err.to_string())
    }
}

/// Result type alias for Redis Cloud operations
pub type Result<T> = std::result::Result<T, CloudError>;
