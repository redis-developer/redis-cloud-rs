//! Dynamic database endpoint redirection operations.
//!
//! Endpoint redirection moves a source database endpoint to a target database.
//! The operation is asynchronous and exposes a redirection identifier that can
//! be polled or reverted.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Endpoint type selected for a dynamic redirection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum EndpointTargetType {
    /// Redirect the public endpoint.
    Public,
    /// Redirect the private endpoint.
    Private,
    /// An endpoint type added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// Request to create a dynamic endpoint redirection.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct CreateEndpointsRedirectionRequest {
    /// Source database whose endpoint will be redirected.
    pub source_database_id: i32,

    /// Target database that will receive the endpoint.
    pub target_database_id: i32,

    /// Public or private endpoint to redirect.
    pub endpoint_target_type: EndpointTargetType,

    /// Whether to duplicate source database ACLs on the target database.
    #[serde(rename = "duplicateACLs", skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub duplicate_acls: Option<bool>,

    /// Explicit protection flag required by the API to start the migration.
    pub migration_protection: bool,
}

/// Current status of a dynamic endpoint redirection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum EndpointRedirectionStatus {
    /// The request was accepted.
    Initiated,
    /// The request is waiting to run.
    Pending,
    /// Endpoint migration is in progress.
    InProgress,
    /// Endpoint migration completed successfully.
    Completed,
    /// Endpoint migration failed.
    Failed,
    /// A completed redirection was reverted.
    Reverted,
    /// A status added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// Details for one endpoint moved by a redirection operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointRedirection {
    /// Source endpoint name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_endpoint_name: Option<String>,

    /// Target endpoint name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_endpoint_name: Option<String>,

    /// Source endpoint type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_endpoint_type: Option<EndpointTargetType>,

    /// Target endpoint type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_endpoint_type: Option<EndpointTargetType>,

    /// Endpoint-specific failure detail, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Dynamic endpoint redirection status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointsRedirectionResponse {
    /// Redirection identifier used for polling and reversion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirection_id: Option<String>,

    /// Current redirection state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EndpointRedirectionStatus>,

    /// Source database ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_database_id: Option<i32>,

    /// Target database ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_database_id: Option<i32>,

    /// Whether this operation is reverting a prior redirection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_revert: Option<bool>,

    /// Whether source ACLs were duplicated to the target.
    #[serde(rename = "duplicateACLs", skip_serializing_if = "Option::is_none")]
    pub duplicate_acls: Option<bool>,

    /// Timestamp when the operation started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,

    /// Timestamp when the operation completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    /// Timestamp when the operation was reverted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverted_at: Option<String>,

    /// Operation-level failure detail, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Individual endpoint movements performed by the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<EndpointRedirection>>,
}

/// Handler for dynamic endpoint redirection operations.
pub struct EndpointRedirectionsHandler {
    client: CloudClient,
}

impl EndpointRedirectionsHandler {
    /// Create an endpoint redirections handler.
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Start a dynamic endpoint redirection.
    pub async fn create(
        &self,
        request: &CreateEndpointsRedirectionRequest,
    ) -> Result<EndpointsRedirectionResponse> {
        self.client.post("/endpoint-redirections", request).await
    }

    /// Get the current state of a dynamic endpoint redirection.
    pub async fn get(&self, redirection_id: &str) -> Result<EndpointsRedirectionResponse> {
        self.client
            .get(&format!("/endpoint-redirections/{redirection_id}"))
            .await
    }

    /// Revert a completed dynamic endpoint redirection.
    pub async fn revert(&self, redirection_id: &str) -> Result<EndpointsRedirectionResponse> {
        self.client
            .post_empty(&format!("/endpoint-redirections/{redirection_id}/revert"))
            .await
    }
}
