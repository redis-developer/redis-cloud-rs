//! VPC Peering connectivity operations
//!
//! Manages VPC peering connections between Redis Cloud VPCs and customer VPCs
//! for both standard and Active-Active subscriptions.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

/// VPC peering creation request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcPeeringCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    /// AWS VPC ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// AWS region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_region: Option<String>,

    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// VPC CIDR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// List of VPC CIDRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidrs: Option<Vec<String>>,

    /// GCP project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_project_id: Option<String>,

    /// GCP network name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,
}

/// Base VPC peering creation request (for backward compatibility)
pub type VpcPeeringCreateBaseRequest = VpcPeeringCreateRequest;

/// VPC peering update request for AWS
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcPeeringUpdateAwsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// VPC Peering ID to update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_peering_id: Option<i32>,

    /// Optional. VPC CIDR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// Optional. List of VPC CIDRs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidrs: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// VPC peering update request (generic)
pub type VpcPeeringUpdateRequest = VpcPeeringUpdateAwsRequest;

/// Task state update response
pub use crate::types::TaskStateUpdate;

/// VPC CIDR with status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcCidr {
    /// VPC CIDR block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// CIDR status (active/inactive)
    #[serde(rename = "active", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// VPC Peering information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcPeering {
    /// VPC Peering ID
    #[serde(rename = "vpcPeeringId", skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Peering status (e.g., "active", "pending-acceptance")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// AWS VPC peering connection ID
    #[serde(rename = "awsPeeringUid", skip_serializing_if = "Option::is_none")]
    pub aws_peering_id: Option<String>,

    /// VPC ID
    #[serde(rename = "vpcUid", skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// VPC CIDR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// List of VPC CIDRs with status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidrs: Option<Vec<VpcCidr>>,

    /// GCP project UID
    #[serde(rename = "projectUid", skip_serializing_if = "Option::is_none")]
    pub gcp_project_uid: Option<String>,

    /// GCP network name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,

    /// Redis GCP project UID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_project_uid: Option<String>,

    /// Redis GCP network name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_network_name: Option<String>,

    /// Cloud peering ID (GCP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_peering_id: Option<String>,

    /// Cloud provider region
    #[serde(rename = "regionName", skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Cloud provider (AWS, GCP, Azure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Active-Active VPC Peering information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveVpcPeering {
    /// VPC Peering ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Peering status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Region ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<i32>,

    /// Region name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_name: Option<String>,

    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// AWS VPC peering UID
    #[serde(rename = "awsPeeringUid", skip_serializing_if = "Option::is_none")]
    pub aws_peering_id: Option<String>,

    /// VPC UID
    #[serde(rename = "vpcUid", skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// VPC CIDR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// List of VPC CIDRs with status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidrs: Option<Vec<VpcCidr>>,

    /// GCP project UID
    #[serde(rename = "vpcProjectUid", skip_serializing_if = "Option::is_none")]
    pub gcp_project_uid: Option<String>,

    /// GCP network name
    #[serde(rename = "vpcNetworkName", skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,

    /// Redis GCP project UID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_project_uid: Option<String>,

    /// Redis GCP network name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_network_name: Option<String>,

    /// Cloud peering ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_peering_id: Option<String>,

    /// Source region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_region: Option<String>,

    /// Destination region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_region: Option<String>,
}

/// Active-Active VPC Peering region
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveVpcRegion {
    /// Region ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Source region name
    #[serde(rename = "region", skip_serializing_if = "Option::is_none")]
    pub source_region: Option<String>,

    /// VPC Peerings in this region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_peerings: Option<Vec<ActiveActiveVpcPeering>>,
}

/// Active-Active VPC Peering list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveVpcPeeringList {
    /// Subscription ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Regions with VPC peerings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<ActiveActiveVpcRegion>>,
}

/// VPC Peering handler
pub struct VpcPeeringHandler {
    client: CloudClient,
}

impl VpcPeeringHandler {
    /// Create a new VPC peering handler
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    // ========================================================================
    // Standard VPC Peering
    // ========================================================================

    /// Get VPC peering for subscription
    pub async fn get(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!("/subscriptions/{}/peerings", subscription_id))
            .await
    }

    /// Create VPC peering
    pub async fn create(
        &self,
        subscription_id: i32,
        request: &VpcPeeringCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{}/peerings", subscription_id),
                request,
            )
            .await
    }

    /// Delete VPC peering
    pub async fn delete(&self, subscription_id: i32, peering_id: i32) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{}/peerings/{}",
                subscription_id, peering_id
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Update VPC peering
    pub async fn update(
        &self,
        subscription_id: i32,
        peering_id: i32,
        request: &VpcPeeringCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!("/subscriptions/{}/peerings/{}", subscription_id, peering_id),
                request,
            )
            .await
    }

    // ========================================================================
    // Active-Active VPC Peering
    // ========================================================================
    //
    // Note: Active-Active VPC peering uses the same API endpoints as standard
    // VPC peering. These methods are provided for API consistency and to match
    // the naming convention used by other connectivity handlers.

    /// Get Active-Active VPC peerings
    ///
    /// Note: Uses the same endpoint as standard VPC peering.
    pub async fn get_active_active(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.get(subscription_id).await
    }

    /// Create Active-Active VPC peering
    ///
    /// Note: Uses the same endpoint as standard VPC peering.
    pub async fn create_active_active(
        &self,
        subscription_id: i32,
        request: &VpcPeeringCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.create(subscription_id, request).await
    }

    /// Delete Active-Active VPC peering
    ///
    /// Note: Uses the same endpoint as standard VPC peering.
    pub async fn delete_active_active(
        &self,
        subscription_id: i32,
        peering_id: i32,
    ) -> Result<serde_json::Value> {
        self.delete(subscription_id, peering_id).await
    }

    /// Update Active-Active VPC peering
    ///
    /// Note: Uses the same endpoint as standard VPC peering.
    pub async fn update_active_active(
        &self,
        subscription_id: i32,
        peering_id: i32,
        request: &VpcPeeringCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.update(subscription_id, peering_id, request).await
    }
}
