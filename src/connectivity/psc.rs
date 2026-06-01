//! Google Cloud Private Service Connect (PSC) operations.
//!
//! Manages Private Service Connect services and endpoints so Redis Cloud
//! databases can be reached from a GCP VPC without traversing the public
//! internet.
//!
//! # When to use this module
//!
//! - The subscription is on GCP and you want connectivity that does not
//!   require a VPC peering connection or a public endpoint.
//! - You manage multiple client projects and want each to attach via its
//!   own consumer endpoint.
//!
//! For AWS connectivity see [`crate::connectivity::vpc_peering`] (general
//! VPC peering) or [`crate::connectivity::private_link`] (AWS PrivateLink).
//! For AWS hub-and-spoke topologies see
//! [`crate::connectivity::transit_gateway`].
//!
//! # Endpoint surface
//!
//! Service-level (one per subscription / region):
//!
//! - `GET    /subscriptions/{subscriptionId}/private-service-connect`
//! - `POST   /subscriptions/{subscriptionId}/private-service-connect`
//! - `DELETE /subscriptions/{subscriptionId}/private-service-connect`
//!
//! Endpoint-level (consumer endpoints under the service):
//!
//! - `POST /subscriptions/{subscriptionId}/private-service-connect/.../endpoints`
//! - `PUT  /subscriptions/{subscriptionId}/private-service-connect/.../endpoints/{endpointId}`
//!
//! Active-Active subscriptions expose the same surface scoped to a region
//! id via `/subscriptions/{subscriptionId}/regions/{regionId}/...`.
//!
//! # Errors
//!
//! All operations return [`crate::Result`]; transport, auth, and 4xx/5xx
//! responses surface as the corresponding [`crate::CloudError`] variant.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

/// Private Service Connect endpoint update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PscEndpointUpdateRequest {
    /// Subscription that owns the PSC service. Server-populated; clients
    /// pass the value via the path parameter and may leave the default.
    pub subscription_id: i32,
    /// PSC service ID under the subscription. Server-populated.
    pub psc_service_id: i32,
    /// PSC endpoint ID being updated. Server-populated.
    pub endpoint_id: i32,

    /// Google Cloud project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_project_id: Option<String>,

    /// Name of the Google Cloud VPC that hosts your application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_vpc_name: Option<String>,

    /// Name of your VPC's subnet of IP address ranges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_vpc_subnet_name: Option<String>,

    /// Prefix used to create PSC endpoints in the consumer application VPC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_connection_name: Option<String>,
}

/// Task state update response
pub use crate::types::TaskStateUpdate;

/// Private Service Connect service information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateServiceConnectService {
    /// PSC service ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Connection host name for the PSC service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_host_name: Option<String>,

    /// GCP service attachment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_attachment_name: Option<String>,

    /// PSC service status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Private Service Connect endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateServiceConnectEndpoint {
    /// Endpoint ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// GCP project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_project_id: Option<String>,

    /// GCP VPC name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_vpc_name: Option<String>,

    /// GCP VPC subnet name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_vpc_subnet_name: Option<String>,

    /// Endpoint connection name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_connection_name: Option<String>,

    /// Endpoint status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Private Service Connect endpoints response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateServiceConnectEndpoints {
    /// PSC service ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psc_service_id: Option<i32>,

    /// List of PSC endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<PrivateServiceConnectEndpoint>>,
}

/// GCP creation script for PSC endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcpCreationScript {
    /// Bash script for endpoint creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash: Option<String>,

    /// `PowerShell` script for endpoint creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powershell: Option<String>,

    /// Terraform GCP configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terraform_gcp: Option<TerraformGcp>,
}

/// Terraform GCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerraformGcp {
    /// Service attachment configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_attachments: Option<Vec<TerraformGcpServiceAttachment>>,
}

/// Terraform GCP service attachment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerraformGcpServiceAttachment {
    /// Service attachment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// DNS record
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_record: Option<String>,

    /// IP address name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address_name: Option<String>,

    /// Forwarding rule name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarding_rule_name: Option<String>,
}

/// GCP deletion script for PSC endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcpDeletionScript {
    /// Bash script for endpoint deletion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bash: Option<String>,

    /// `PowerShell` script for endpoint deletion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powershell: Option<String>,
}

/// Private Service Connect handler
pub struct PscHandler {
    client: CloudClient,
}

impl PscHandler {
    /// Create a new PSC handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    // ========================================================================
    // Standard PSC Operations
    // ========================================================================

    /// Delete Private Service Connect service
    pub async fn delete_service(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .delete_typed(&format!(
                "/subscriptions/{subscription_id}/private-service-connect"
            ))
            .await
    }

    /// Get Private Service Connect service
    pub async fn get_service(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/private-service-connect"
            ))
            .await
    }

    /// Create Private Service Connect service
    pub async fn create_service(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/private-service-connect"),
                &serde_json::json!({}),
            )
            .await
    }

    /// Create a Private Service Connect endpoint under the given service.
    pub async fn create_endpoint(
        &self,
        subscription_id: i32,
        psc_service_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/private-service-connect/{psc_service_id}"
                ),
                request,
            )
            .await
    }

    /// Delete Private Service Connect endpoint
    pub async fn delete_endpoint(
        &self,
        subscription_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .delete_typed(&format!(
                "/subscriptions/{subscription_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}"
            ))
            .await
    }

    /// Update Private Service Connect endpoint
    pub async fn update_endpoint(
        &self,
        subscription_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!(
                    "/subscriptions/{subscription_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}"
                ),
                request,
            )
            .await
    }

    /// Get PSC endpoint creation script
    pub async fn get_endpoint_creation_script(
        &self,
        subscription_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}/creationScripts"
            ))
            .await
    }

    /// Get PSC endpoint deletion script
    pub async fn get_endpoint_deletion_script(
        &self,
        subscription_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}/deletionScripts"
            ))
            .await
    }

    // ========================================================================
    // Active-Active PSC Operations
    // ========================================================================

    /// Delete Active-Active PSC service for a region
    pub async fn delete_service_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .delete_typed(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect"
            ))
            .await
    }

    /// Get Active-Active PSC service for a region
    pub async fn get_service_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect"
            ))
            .await
    }

    /// Create Active-Active PSC service for a region
    pub async fn create_service_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect"
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Create an Active-Active Private Service Connect endpoint under the
    /// given service for a region.
    pub async fn create_endpoint_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        psc_service_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect/{psc_service_id}"
                ),
                request,
            )
            .await
    }

    /// Delete Active-Active PSC endpoint
    pub async fn delete_endpoint_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client.delete_typed(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}"
            )).await
    }

    /// Update Active-Active PSC endpoint
    pub async fn update_endpoint_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}"
                ),
                request,
            )
            .await
    }

    /// Get Active-Active PSC endpoint creation script
    pub async fn get_endpoint_creation_script_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}/creationScripts"
            ))
            .await
    }

    /// Get Active-Active PSC endpoint deletion script
    pub async fn get_endpoint_deletion_script_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        psc_service_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/private-service-connect/{psc_service_id}/endpoints/{endpoint_id}/deletionScripts"
            ))
            .await
    }
}
