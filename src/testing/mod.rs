//! Test utilities for Redis Cloud API consumers
//!
//! This module provides mock server utilities and fixtures for testing code
//! that uses the `redis-cloud` client library. It is only available when the
//! `test-support` feature is enabled.
//!
//! # Features
//!
//! - **MockCloudServer**: A pre-configured mock server that mimics the Redis Cloud API
//! - **Fixtures**: Builder-pattern fixtures for common response types
//! - **Response helpers**: Convenience functions for creating HTTP responses
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use redis_cloud::testing::{MockCloudServer, SubscriptionFixture, responses};
//!
//! #[tokio::test]
//! async fn test_list_subscriptions() {
//!     // Start a mock server
//!     let server = MockCloudServer::start().await;
//!
//!     // Create a fixture for subscription response
//!     let subscription = SubscriptionFixture::new(123, "Production")
//!         .status("active")
//!         .build();
//!
//!     // Mock the subscriptions list endpoint
//!     server.mock_subscriptions_list(vec![subscription]).await;
//!
//!     // Get a pre-configured client pointing to the mock
//!     let client = server.client();
//!
//!     // Your test code here...
//! }
//! ```
//!
//! # Custom Mocking
//!
//! For scenarios not covered by the convenience methods, you can access
//! the underlying wiremock server directly:
//!
//! ```rust,ignore
//! use redis_cloud::testing::{MockCloudServer, Mock, method, path, ResponseTemplate};
//!
//! #[tokio::test]
//! async fn test_custom_endpoint() {
//!     let server = MockCloudServer::start().await;
//!
//!     // Mount a custom mock
//!     Mock::given(method("GET"))
//!         .and(path("/custom-endpoint"))
//!         .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
//!         .mount(server.inner())
//!         .await;
//!
//!     let client = server.client();
//!     // ...
//! }
//! ```

mod fixtures;
mod responses;
mod server;

// Re-export main types
pub use fixtures::{
    AccountFixture, DatabaseFixture, SubscriptionFixture, TaskFixture, UserFixture,
};
pub use responses::{
    accepted, accepted_with_resource, bad_request, conflict, created, delayed, error, forbidden,
    no_content, not_found, rate_limited, server_error, service_unavailable, success, unauthorized,
};
pub use server::MockCloudServer;

// Re-export wiremock types for custom mocking
pub use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path, path_regex, query_param},
};
