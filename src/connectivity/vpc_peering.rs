//! VPC peering operations for AWS and GCP Pro subscriptions.
//!
//! Manages VPC peering connections between a Redis Cloud subscription's
//! VPC and a customer-owned VPC, covering both standard subscriptions and
//! Active-Active (CRDB) subscriptions where each region is peered
//! independently.
//!
//! # When to use this module
//!
//! - You want direct VPC-to-VPC private connectivity (no public
//!   endpoint, no shared TGW).
//! - The subscription is on **AWS** or **GCP**. Azure connectivity is
//!   handled separately by the Redis Cloud console; the SDK does not
//!   yet expose Azure-specific endpoints here.
//!
//! For AWS hub-and-spoke topologies see
//! [`crate::connectivity::transit_gateway`]; for AWS endpoint-style
//! private connectivity see [`crate::connectivity::private_link`]; for
//! GCP endpoint-style private connectivity see
//! [`crate::connectivity::psc`].
//!
//! # Endpoint surface
//!
//! - `GET    /subscriptions/{subscriptionId}/peerings`
//! - `POST   /subscriptions/{subscriptionId}/peerings`
//! - `PUT    /subscriptions/{subscriptionId}/peerings/{peeringId}`
//! - `DELETE /subscriptions/{subscriptionId}/peerings/{peeringId}`
//!
//! Active-Active subscriptions expose the same surface scoped to a
//! region under `/subscriptions/{subscriptionId}/regions/{regionId}/...`.
//!
//! # Example
//!
//! Construct a provider-targeted body and create a peering:
//!
//! ```rust,no_run
//! use redis_cloud::{CloudClient, VpcPeeringHandler};
//! use redis_cloud::connectivity::VpcPeeringCreateRequest;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("k").api_secret("s").build()?;
//! let handler = VpcPeeringHandler::new(client);
//!
//! let mut request = VpcPeeringCreateRequest::for_aws(
//!     "us-east-1", "123456789012", "vpc-12345678",
//! );
//! request.vpc_cidr = Some("10.0.0.0/16".to_string());
//! let task = handler.create(123, &request).await?;
//! # let _ = task;
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! All operations return [`crate::Result`]; transport, auth, and 4xx/5xx
//! responses surface as the corresponding [`crate::CloudError`] variant.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

/// VPC peering creation request.
///
/// The Redis Cloud API documents this as a `oneOf` between an AWS-shaped
/// body (requiring `region`, `awsAccountId`, `vpcId`) and a GCP-shaped body
/// (requiring `vpcProjectUid`, `vpcNetworkName`). This struct keeps both
/// providers in one type for caller flexibility, but uses
/// `#[serde(rename = ...)]` so the AWS and GCP fields serialize to the
/// **exact wire names the spec requires**. Use [`Self::for_aws`] or
/// [`Self::for_gcp`] to construct provider-targeted bodies that avoid
/// mixing fields.
///
/// A type-safe enum split that prevents AWS+GCP field mixing at compile
/// time is tracked as a follow-on under #65.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcPeeringCreateRequest {
    /// Cloud provider discriminator (e.g. "AWS", "GCP").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Read-only on the response; populated by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    // ------- AWS body -------
    /// AWS region. Wire name: `region` (spec required for AWS).
    #[serde(rename = "region", skip_serializing_if = "Option::is_none")]
    pub aws_region: Option<String>,

    /// AWS account ID (spec required for AWS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// AWS VPC ID (spec required for AWS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// VPC CIDR. AWS only; optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidr: Option<String>,

    /// List of VPC CIDRs. AWS only; optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_cidrs: Option<Vec<String>>,

    // ------- GCP body -------
    /// GCP project UID. Wire name: `vpcProjectUid` (spec required for GCP).
    #[serde(rename = "vpcProjectUid", skip_serializing_if = "Option::is_none")]
    pub gcp_project_id: Option<String>,

    /// GCP network name. Wire name: `vpcNetworkName` (spec required for GCP).
    #[serde(rename = "vpcNetworkName", skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,
}

impl VpcPeeringCreateRequest {
    /// Construct an AWS-targeted VPC peering creation body.
    ///
    /// Pre-populates `provider = "AWS"` and the three required AWS fields
    /// (`region`, `awsAccountId`, `vpcId`). Optional CIDR fields can be set
    /// directly on the returned struct.
    #[must_use]
    pub fn for_aws(
        region: impl Into<String>,
        aws_account_id: impl Into<String>,
        vpc_id: impl Into<String>,
    ) -> Self {
        Self {
            provider: Some("AWS".to_string()),
            aws_region: Some(region.into()),
            aws_account_id: Some(aws_account_id.into()),
            vpc_id: Some(vpc_id.into()),
            ..Self::default()
        }
    }

    /// Construct a GCP-targeted VPC peering creation body.
    ///
    /// Pre-populates `provider = "GCP"` and the two required GCP fields
    /// (`vpcProjectUid`, `vpcNetworkName`).
    #[must_use]
    pub fn for_gcp(project_uid: impl Into<String>, network_name: impl Into<String>) -> Self {
        Self {
            provider: Some("GCP".to_string()),
            gcp_project_id: Some(project_uid.into()),
            network_name: Some(network_name.into()),
            ..Self::default()
        }
    }
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
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    // ========================================================================
    // Standard VPC Peering
    // ========================================================================

    /// Get VPC peering for subscription
    pub async fn get(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/peerings"))
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
                &format!("/subscriptions/{subscription_id}/peerings"),
                request,
            )
            .await
    }

    /// Delete VPC peering
    pub async fn delete(&self, subscription_id: i32, peering_id: i32) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{subscription_id}/peerings/{peering_id}"
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
                &format!("/subscriptions/{subscription_id}/peerings/{peering_id}"),
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
