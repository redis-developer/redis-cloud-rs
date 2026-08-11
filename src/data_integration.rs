//! Redis Data Integration workspace operations.
//!
//! The Redis Cloud API exposes Data Integration as an opaque JSON workspace
//! service. The published schema intentionally models request and response
//! bodies as arbitrary JSON, so this handler provides structured routing while
//! preserving payloads as [`serde_json::Value`].

use crate::{CloudClient, Result};
use serde_json::Value;

/// Handler for Data Integration workspace and proxy operations.
pub struct DataIntegrationHandler {
    client: CloudClient,
}

impl DataIntegrationHandler {
    /// Create a Data Integration handler.
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// List Data Integration workspaces visible to the account.
    pub async fn list_workspaces(&self) -> Result<Value> {
        self.client.get_raw("/data-integration-workspaces").await
    }

    /// Get the root Data Integration workspace document for a subscription.
    pub async fn get_workspace(&self, subscription_id: i32) -> Result<Value> {
        self.client
            .get_raw(&format!(
                "/subscriptions/{subscription_id}/data-integration-workspace"
            ))
            .await
    }

    /// POST an opaque JSON document to a subscription's workspace root.
    pub async fn post_workspace(&self, subscription_id: i32, body: Value) -> Result<Value> {
        self.client
            .post_raw(
                &format!("/subscriptions/{subscription_id}/data-integration-workspace"),
                body,
            )
            .await
    }

    /// Replace a subscription's workspace root with an opaque JSON document.
    pub async fn put_workspace(&self, subscription_id: i32, body: Value) -> Result<Value> {
        self.client
            .put_raw(
                &format!("/subscriptions/{subscription_id}/data-integration-workspace"),
                body,
            )
            .await
    }

    /// Patch a subscription's workspace root with an opaque JSON document.
    pub async fn patch_workspace(&self, subscription_id: i32, body: Value) -> Result<Value> {
        self.client
            .patch_raw(
                &format!("/subscriptions/{subscription_id}/data-integration-workspace"),
                body,
            )
            .await
    }

    /// DELETE a subscription's workspace root with an opaque JSON document.
    pub async fn delete_workspace(&self, subscription_id: i32, body: Value) -> Result<Value> {
        self.client
            .delete_with_body(
                &format!("/subscriptions/{subscription_id}/data-integration-workspace"),
                body,
            )
            .await
    }

    /// GET an arbitrary path below a subscription's Data Integration workspace.
    pub async fn get_workspace_path(
        &self,
        subscription_id: i32,
        workspace_path: &str,
    ) -> Result<Value> {
        let workspace_path = workspace_path.trim_matches('/');
        self.client
            .get_raw(&format!(
                "/subscriptions/{subscription_id}/data-integration-workspace/{workspace_path}"
            ))
            .await
    }

    /// POST an opaque JSON document to an arbitrary workspace path.
    pub async fn post_workspace_path(
        &self,
        subscription_id: i32,
        workspace_path: &str,
        body: Value,
    ) -> Result<Value> {
        let workspace_path = workspace_path.trim_matches('/');
        self.client
            .post_raw(
                &format!(
                    "/subscriptions/{subscription_id}/data-integration-workspace/{workspace_path}"
                ),
                body,
            )
            .await
    }

    /// PUT an opaque JSON document at an arbitrary workspace path.
    pub async fn put_workspace_path(
        &self,
        subscription_id: i32,
        workspace_path: &str,
        body: Value,
    ) -> Result<Value> {
        let workspace_path = workspace_path.trim_matches('/');
        self.client
            .put_raw(
                &format!(
                    "/subscriptions/{subscription_id}/data-integration-workspace/{workspace_path}"
                ),
                body,
            )
            .await
    }

    /// PATCH an arbitrary workspace path with an opaque JSON document.
    pub async fn patch_workspace_path(
        &self,
        subscription_id: i32,
        workspace_path: &str,
        body: Value,
    ) -> Result<Value> {
        let workspace_path = workspace_path.trim_matches('/');
        self.client
            .patch_raw(
                &format!(
                    "/subscriptions/{subscription_id}/data-integration-workspace/{workspace_path}"
                ),
                body,
            )
            .await
    }

    /// DELETE an arbitrary workspace path with an opaque JSON document.
    pub async fn delete_workspace_path(
        &self,
        subscription_id: i32,
        workspace_path: &str,
        body: Value,
    ) -> Result<Value> {
        let workspace_path = workspace_path.trim_matches('/');
        self.client
            .delete_with_body(
                &format!(
                    "/subscriptions/{subscription_id}/data-integration-workspace/{workspace_path}"
                ),
                body,
            )
            .await
    }
}
