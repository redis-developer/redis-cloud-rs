//! AWS `PrivateLink` connectivity operations
//!
//! This module provides AWS `PrivateLink` connectivity functionality for Redis Cloud,
//! enabling secure, private connections from AWS VPCs to Redis Cloud databases.
//!
//! # Overview
//!
//! AWS `PrivateLink` allows you to connect to Redis Cloud from your AWS VPC without
//! traversing the public internet. This provides enhanced security and potentially
//! lower latency.
//!
//! # Features
//!
//! - **`PrivateLink` Management**: Create and retrieve `PrivateLink` configurations
//! - **Principal Management**: Control which AWS principals can access the service
//! - **Endpoint Scripts**: Get scripts to create endpoints in your AWS account
//! - **Active-Active Support**: `PrivateLink` for CRDB (Active-Active) databases
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, PrivateLinkHandler, PrivateLinkCreateRequest, PrincipalType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = PrivateLinkHandler::new(client);
//!
//! // Create a PrivateLink
//! let request = PrivateLinkCreateRequest {
//!     share_name: "my-redis-share".to_string(),
//!     principal: "123456789012".to_string(),
//!     principal_type: PrincipalType::AwsAccount,
//!     alias: Some("Production Account".to_string()),
//! };
//! let result = handler.create(123, &request).await?;
//!
//! // Get PrivateLink configuration
//! let config = handler.get(123).await?;
//! # Ok(())
//! # }
//! ```

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
// Note: Value is still needed for return types that use raw JSON responses

// ============================================================================
// Request/Response Types
// ============================================================================

/// Principal type for `PrivateLink` access control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalType {
    /// AWS account ID
    AwsAccount,
    /// AWS Organization
    Organization,
    /// AWS Organization Unit
    OrganizationUnit,
    /// AWS IAM Role
    IamRole,
    /// AWS IAM User
    IamUser,
    /// Service Principal
    ServicePrincipal,
}

/// Request to create a `PrivateLink` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkCreateRequest {
    /// Share name for the `PrivateLink` service (max 64 characters)
    pub share_name: String,

    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,

    /// Type of principal
    #[serde(rename = "type")]
    pub principal_type: PrincipalType,

    /// Optional alias for the `PrivateLink`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// Request to add a principal to `PrivateLink` access list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkAddPrincipalRequest {
    /// AWS principal (account ID, role ARN, etc.)
    pub principal: String,

    /// Type of principal
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<PrincipalType>,

    /// Optional alias for the principal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// Request to remove a principal from `PrivateLink` access list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkRemovePrincipalRequest {
    /// AWS principal to remove
    pub principal: String,

    /// Type of principal
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<PrincipalType>,

    /// Alias of the principal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

/// `PrivateLink` configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLink {
    /// `PrivateLink` status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// List of principals with access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principals: Option<Vec<PrivateLinkPrincipal>>,

    /// AWS Resource Configuration ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_configuration_id: Option<String>,

    /// AWS Resource Configuration ARN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_configuration_arn: Option<String>,

    /// RAM share ARN
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_arn: Option<String>,

    /// Share name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_name: Option<String>,

    /// List of `PrivateLink` connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<PrivateLinkConnection>>,

    /// List of databases accessible via `PrivateLink`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases: Option<Vec<PrivateLinkDatabase>>,

    /// Subscription ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Region ID (for Active-Active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<i32>,

    /// Error message if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// `PrivateLink` principal information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkPrincipal {
    /// AWS principal (account ID, role ARN, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,

    /// Type of principal
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub principal_type: Option<String>,

    /// Alias for the principal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    /// Principal status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// `PrivateLink` connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkConnection {
    /// Association ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub association_id: Option<String>,

    /// Connection ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,

    /// Connection type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<String>,

    /// Owner ID (AWS account)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,

    /// Association date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub association_date: Option<String>,
}

/// Database accessible via `PrivateLink`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkDatabase {
    /// Database ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<i32>,

    /// Database port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,

    /// Resource link endpoint URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_link_endpoint: Option<String>,
}

/// `PrivateLink` endpoint script response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateLinkEndpointScript {
    /// AWS CLI/CloudFormation script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_endpoint_script: Option<String>,

    /// Terraform AWS script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terraform_aws_script: Option<String>,
}

/// AWS `PrivateLink` handler
///
/// Manages AWS `PrivateLink` connectivity for Redis Cloud subscriptions.
pub struct PrivateLinkHandler {
    client: CloudClient,
}

impl PrivateLinkHandler {
    /// Create a new `PrivateLink` handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get `PrivateLink` configuration
    ///
    /// Gets the AWS `PrivateLink` configuration for a subscription.
    ///
    /// GET /subscriptions/{subscriptionId}/private-link
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    ///
    /// # Returns
    ///
    /// Returns the `PrivateLink` configuration as JSON
    pub async fn get(&self, subscription_id: i32) -> Result<Value> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/private-link"))
            .await
    }

    /// Create a `PrivateLink`
    ///
    /// Creates a new AWS `PrivateLink` configuration for a subscription.
    ///
    /// POST /subscriptions/{subscriptionId}/private-link
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - `PrivateLink` creation request
    ///
    /// # Returns
    ///
    /// Returns a task response that can be tracked for completion
    pub async fn create(
        &self,
        subscription_id: i32,
        request: &PrivateLinkCreateRequest,
    ) -> Result<Value> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/private-link"),
                request,
            )
            .await
    }

    /// Add principals to `PrivateLink`
    ///
    /// Adds AWS principals (accounts, IAM roles, etc.) that can access the `PrivateLink`.
    ///
    /// POST /subscriptions/{subscriptionId}/private-link/principals
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - Principal to add
    ///
    /// # Returns
    ///
    /// Returns the updated principal configuration
    pub async fn add_principals(
        &self,
        subscription_id: i32,
        request: &PrivateLinkAddPrincipalRequest,
    ) -> Result<Value> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/private-link/principals"),
                request,
            )
            .await
    }

    /// Remove principals from `PrivateLink`
    ///
    /// Removes AWS principals from the `PrivateLink` access list.
    ///
    /// DELETE /subscriptions/{subscriptionId}/private-link/principals
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - Principal to remove
    ///
    /// # Returns
    ///
    /// Returns confirmation of deletion
    pub async fn remove_principals(
        &self,
        subscription_id: i32,
        request: &PrivateLinkRemovePrincipalRequest,
    ) -> Result<Value> {
        self.client
            .delete_with_body(
                &format!("/subscriptions/{subscription_id}/private-link/principals"),
                serde_json::to_value(request).unwrap_or_default(),
            )
            .await
    }

    /// Get endpoint creation script
    ///
    /// Gets a script to create the VPC endpoint in your AWS account.
    ///
    /// GET /subscriptions/{subscriptionId}/private-link/endpoint-script
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    ///
    /// # Returns
    ///
    /// Returns the endpoint creation script
    pub async fn get_endpoint_script(&self, subscription_id: i32) -> Result<Value> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/private-link/endpoint-script"
            ))
            .await
    }

    /// Delete `PrivateLink`
    ///
    /// Deletes the AWS `PrivateLink` configuration for a subscription.
    ///
    /// DELETE /subscriptions/{subscriptionId}/private-link
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    ///
    /// # Returns
    ///
    /// Returns task information for tracking the deletion
    pub async fn delete(&self, subscription_id: i32) -> Result<Value> {
        self.client
            .delete_raw(&format!("/subscriptions/{subscription_id}/private-link"))
            .await
    }

    /// Get Active-Active `PrivateLink` configuration
    ///
    /// Gets the AWS `PrivateLink` configuration for an Active-Active (CRDB) subscription region.
    ///
    /// GET /subscriptions/{subscriptionId}/regions/{regionId}/private-link
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `region_id` - The region ID
    ///
    /// # Returns
    ///
    /// Returns the `PrivateLink` configuration for the region
    pub async fn get_active_active(&self, subscription_id: i32, region_id: i32) -> Result<Value> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-link"
            ))
            .await
    }

    /// Create Active-Active `PrivateLink`
    ///
    /// Creates a new AWS `PrivateLink` for an Active-Active (CRDB) subscription region.
    ///
    /// POST /subscriptions/{subscriptionId}/regions/{regionId}/private-link
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `region_id` - The region ID
    /// * `request` - `PrivateLink` creation request
    ///
    /// # Returns
    ///
    /// Returns a task response
    pub async fn create_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        request: &PrivateLinkCreateRequest,
    ) -> Result<Value> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/regions/{region_id}/private-link"),
                request,
            )
            .await
    }

    /// Add principals to Active-Active `PrivateLink`
    ///
    /// Adds AWS principals to an Active-Active `PrivateLink`.
    ///
    /// POST /subscriptions/{subscriptionId}/regions/{regionId}/private-link/principals
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `region_id` - The region ID
    /// * `request` - Principal to add
    ///
    /// # Returns
    ///
    /// Returns the updated configuration
    pub async fn add_principals_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        request: &PrivateLinkAddPrincipalRequest,
    ) -> Result<Value> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/private-link/principals"
                ),
                request,
            )
            .await
    }

    /// Remove principals from Active-Active `PrivateLink`
    ///
    /// Removes AWS principals from an Active-Active `PrivateLink`.
    ///
    /// DELETE /subscriptions/{subscriptionId}/regions/{regionId}/private-link/principals
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `region_id` - The region ID
    /// * `request` - Principal to remove
    ///
    /// # Returns
    ///
    /// Returns confirmation of deletion
    pub async fn remove_principals_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        request: &PrivateLinkRemovePrincipalRequest,
    ) -> Result<Value> {
        self.client
            .delete_with_body(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/private-link/principals"
                ),
                serde_json::to_value(request).unwrap_or_default(),
            )
            .await
    }

    /// Get Active-Active endpoint creation script
    ///
    /// Gets a script to create the VPC endpoint for an Active-Active region.
    ///
    /// GET /subscriptions/{subscriptionId}/regions/{regionId}/private-link/endpoint-script
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `region_id` - The region ID
    ///
    /// # Returns
    ///
    /// Returns the endpoint creation script
    pub async fn get_endpoint_script_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
    ) -> Result<Value> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-link/endpoint-script"
            ))
            .await
    }
}
