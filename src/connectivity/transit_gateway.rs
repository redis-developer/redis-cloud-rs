//! AWS Transit Gateway operations
//!
//! Manages AWS Transit Gateway attachments for hub-and-spoke network topologies,
//! enabling centralized connectivity management for Redis Cloud subscriptions.

use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

/// CIDR block definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cidr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_address: Option<String>,
}

/// Transit Gateway CIDRs update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TgwUpdateCidrsRequest {
    /// Optional. List of transit gateway attachment CIDRs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidrs: Option<Vec<Cidr>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Transit Gateway attachment request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TgwAttachmentRequest {
    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// Transit Gateway ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tgw_id: Option<String>,

    /// CIDR blocks to route through the TGW
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidrs: Option<Vec<String>>,
}

/// Task state update response
pub use crate::types::TaskStateUpdate;

/// Transit Gateway attachment information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitGatewayAttachment {
    /// Attachment ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// AWS Transit Gateway UID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_tgw_uid: Option<String>,

    /// AWS attachment UID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_uid: Option<String>,

    /// Attachment status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// AWS attachment status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_status: Option<String>,

    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// CIDR blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidrs: Option<Vec<CidrStatus>>,
}

/// CIDR block with status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CidrStatus {
    /// CIDR address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_address: Option<String>,

    /// CIDR status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Transit Gateway resource share invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitGatewayInvitation {
    /// Invitation ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Invitation name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// AWS Resource share UID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_share_uid: Option<String>,

    /// AWS account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// Invitation status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Date the resource was shared
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_date: Option<String>,
}

/// Transit Gateway handler
pub struct TransitGatewayHandler {
    client: CloudClient,
}

impl TransitGatewayHandler {
    /// Create a new Transit Gateway handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    // ========================================================================
    // Standard Transit Gateway Operations
    // ========================================================================

    /// Get Transit Gateway attachments
    pub async fn get_attachments(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/transitGateways"))
            .await
    }

    /// Get Transit Gateway shared invitations
    pub async fn get_shared_invitations(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/tgw/shared-invitations"
            ))
            .await
    }

    /// Accept Transit Gateway resource share
    pub async fn accept_resource_share(
        &self,
        subscription_id: i32,
        invitation_id: String,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/tgw/shared-invitations/{invitation_id}/accept"
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Reject Transit Gateway resource share
    pub async fn reject_resource_share(
        &self,
        subscription_id: i32,
        invitation_id: String,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/tgw/shared-invitations/{invitation_id}/reject"
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Delete Transit Gateway attachment
    pub async fn delete_attachment(
        &self,
        subscription_id: i32,
        attachment_id: String,
    ) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{subscription_id}/transitGateways/{attachment_id}/attachment"
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Create Transit Gateway attachment with `tgw_id` in path
    pub async fn create_attachment_with_id(
        &self,
        subscription_id: i32,
        tgw_id: &str,
    ) -> Result<TaskStateUpdate> {
        let request = TgwAttachmentRequest {
            tgw_id: Some(tgw_id.to_string()),
            aws_account_id: None,
            cidrs: None,
        };

        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/transitGateways/{tgw_id}/attachment"),
                &request,
            )
            .await
    }

    /// Create Transit Gateway attachment
    pub async fn create_attachment(
        &self,
        subscription_id: i32,
        request: &TgwAttachmentRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/transitGateways/attachments"),
                request,
            )
            .await
    }

    /// Update Transit Gateway attachment CIDRs
    pub async fn update_attachment_cidrs(
        &self,
        subscription_id: i32,
        attachment_id: String,
        request: &TgwAttachmentRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!(
                    "/subscriptions/{subscription_id}/transitGateways/{attachment_id}/attachment"
                ),
                request,
            )
            .await
    }

    // ========================================================================
    // Active-Active Transit Gateway Operations
    // ========================================================================

    /// Get Active-Active Transit Gateway attachments
    pub async fn get_attachments_active_active(
        &self,
        subscription_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/transitGateways"
            ))
            .await
    }

    /// Get Active-Active Transit Gateway shared invitations
    pub async fn get_shared_invitations_active_active(
        &self,
        subscription_id: i32,
    ) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/regions/tgw/shared-invitations"
            ))
            .await
    }

    /// Accept Active-Active Transit Gateway resource share
    pub async fn accept_resource_share_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        invitation_id: String,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/tgw/shared-invitations/{invitation_id}/accept"
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Reject Active-Active Transit Gateway resource share
    pub async fn reject_resource_share_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        invitation_id: String,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/tgw/shared-invitations/{invitation_id}/reject"
                ),
                &serde_json::json!({}),
            )
            .await
    }

    /// Delete Active-Active Transit Gateway attachment
    pub async fn delete_attachment_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        attachment_id: String,
    ) -> Result<serde_json::Value> {
        self.client
            .delete(&format!(
                "/subscriptions/{subscription_id}/regions/{region_id}/tgw/attachments/{attachment_id}"
            ))
            .await?;
        Ok(serde_json::Value::Null)
    }

    /// Create Active-Active Transit Gateway attachment
    pub async fn create_attachment_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        request: &TgwAttachmentRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/regions/{region_id}/tgw/attachments"),
                request,
            )
            .await
    }

    /// Update Active-Active Transit Gateway attachment CIDRs
    pub async fn update_attachment_cidrs_active_active(
        &self,
        subscription_id: i32,
        region_id: i32,
        attachment_id: String,
        request: &TgwAttachmentRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!(
                    "/subscriptions/{subscription_id}/regions/{region_id}/tgw/attachments/{attachment_id}/cidrs"
                ),
                request,
            )
            .await
    }
}
