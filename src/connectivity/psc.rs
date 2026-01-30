//! Private Service Connect (PSC) operations
//!
//! Manages Google Cloud Private Service Connect endpoints for secure connectivity
//! to Redis Cloud databases without traversing the public internet.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

/// Private Service Connect endpoint update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PscEndpointUpdateRequest {
    pub subscription_id: i32,
    pub psc_service_id: i32,
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

    /// PowerShell script for endpoint creation
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

    /// PowerShell script for endpoint deletion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powershell: Option<String>,
}

/// Private Service Connect handler
pub struct PscHandler {
    client: CloudClient,
}

impl PscHandler {
    /// Create a new PSC handler
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    // ========================================================================
    // Standard PSC Operations
    // ========================================================================

    /// Delete Private Service Connect service
    pub async fn delete_service(&self, subscription_id: i32) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{}/private-service-connect",
                subscription_id
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Get Private Service Connect service
    pub async fn get_service(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{}/private-service-connect",
                subscription_id
            ))
            .await
    }

    /// Create Private Service Connect service
    pub async fn create_service(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{}/private-service-connect", subscription_id),
                &serde_json::json!({}),
            )
            .await
    }

    /// Get Private Service Connect endpoints
    pub async fn get_endpoints(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{}/private-service-connect/endpoints",
                subscription_id
            ))
            .await
    }

    /// Create Private Service Connect endpoint
    pub async fn create_endpoint(
        &self,
        subscription_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{}/private-service-connect/endpoints",
                    subscription_id
                ),
                request,
            )
            .await
    }

    /// Delete Private Service Connect endpoint
    pub async fn delete_endpoint(
        &self,
        subscription_id: i32,
        endpoint_id: i32,
    ) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{}/private-service-connect/endpoints/{}",
                subscription_id, endpoint_id
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Update Private Service Connect endpoint
    pub async fn update_endpoint(
        &self,
        subscription_id: i32,
        endpoint_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        // Use psc_service_id from request
        let psc_service_id = request.psc_service_id;
        self.client
            .put(
                &format!(
                    "/subscriptions/{}/private-service-connect/{}/endpoints/{}",
                    subscription_id, psc_service_id, endpoint_id
                ),
                request,
            )
            .await
    }

    /// Get PSC endpoint creation script
    pub async fn get_endpoint_creation_script(
        &self,
        subscription_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{}/private-service-connect/endpoints/{}/creationScripts",
                subscription_id, endpoint_id
            ))
            .await
    }

    /// Get PSC endpoint deletion script
    pub async fn get_endpoint_deletion_script(
        &self,
        subscription_id: i32,
        endpoint_id: i32,
    ) -> Result<String> {
        self.client
            .get(&format!(
                "/subscriptions/{}/private-service-connect/endpoints/{}/deletionScripts",
                subscription_id, endpoint_id
            ))
            .await
    }

    // ========================================================================
    // Active-Active PSC Operations
    // ========================================================================

    /// Delete Active-Active PSC service
    pub async fn delete_service_active_active(
        &self,
        subscription_id: i32,
    ) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{}/regions/private-service-connect",
                subscription_id
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Get Active-Active PSC service
    pub async fn get_service_active_active(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{}/regions/private-service-connect",
                subscription_id
            ))
            .await
    }

    /// Create Active-Active PSC service
    pub async fn create_service_active_active(
        &self,
        subscription_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{}/regions/private-service-connect",
                    subscription_id
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Get Active-Active PSC endpoints
    pub async fn get_endpoints_active_active(
        &self,
        subscription_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{}/regions/private-service-connect/endpoints",
                subscription_id
            ))
            .await
    }

    /// Create Active-Active PSC endpoint
    pub async fn create_endpoint_active_active(
        &self,
        subscription_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{}/regions/private-service-connect/endpoints",
                    subscription_id
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
        endpoint_id: i32,
    ) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{}/regions/{}/private-service-connect/endpoints/{}",
                subscription_id, region_id, endpoint_id
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Update Active-Active PSC endpoint
    pub async fn update_endpoint_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        endpoint_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!(
                    "/subscriptions/{}/regions/{}/private-service-connect/{}/endpoints/{}",
                    subscription_id, region_id, subscription_id, endpoint_id
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
                "/subscriptions/{}/regions/{}/private-service-connect/{}/endpoints/{}/creationScripts",
                subscription_id, region_id, psc_service_id, endpoint_id
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
                "/subscriptions/{}/regions/{}/private-service-connect/{}/endpoints/{}/deletionScripts",
                subscription_id, region_id, psc_service_id, endpoint_id
            ))
            .await
    }
}
