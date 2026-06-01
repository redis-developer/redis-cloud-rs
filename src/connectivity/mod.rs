//! Network connectivity and peering operations for Pro subscriptions
//!
//! This module manages advanced networking features for Redis Cloud Pro subscriptions,
//! including VPC peering, AWS Transit Gateway attachments, GCP Private Service Connect,
//! AWS `PrivateLink`, and other cloud-native networking integrations.
//!
//! # Supported Connectivity Types
//!
//! - **VPC Peering**: Direct peering between Redis Cloud VPC and your VPC
//! - **Transit Gateway**: AWS Transit Gateway attachments for hub-and-spoke topologies
//! - **Private Service Connect**: GCP Private Service Connect for private endpoints
//! - **`PrivateLink`**: AWS `PrivateLink` for secure private connectivity
//!
//! # Module Organization
//!
//! The connectivity features are split into four specialized modules:
//! - `vpc_peering` - VPC peering operations for AWS, GCP, and Azure
//! - `psc` - Google Cloud Private Service Connect endpoints
//! - `transit_gateway` - AWS Transit Gateway attachments
//! - `private_link` - AWS `PrivateLink` connectivity

pub mod private_link;
pub mod psc;
pub mod transit_gateway;
pub mod vpc_peering;

// Re-export handlers for convenience
pub use private_link::PrivateLinkHandler;

// Re-export PrivateLink types
pub use private_link::{
    PrincipalType, PrivateLinkAddPrincipalRequest, PrivateLinkCreateRequest,
    PrivateLinkRemovePrincipalRequest,
};
pub use psc::PscHandler;
pub use transit_gateway::TransitGatewayHandler;
pub use vpc_peering::VpcPeeringHandler;

// Re-export types used by handlers
pub use psc::PscEndpointUpdateRequest;
pub use transit_gateway::{Cidr, TgwAttachmentRequest, TgwUpdateCidrsRequest};
pub use vpc_peering::{
    ActiveActiveVpcPeering, ActiveActiveVpcPeeringList, ActiveActiveVpcRegion, VpcCidr, VpcPeering,
    VpcPeeringCreateBaseRequest, VpcPeeringCreateRequest, VpcPeeringUpdateAwsRequest,
    VpcPeeringUpdateRequest,
};

// For backward compatibility, provide a unified handler
use crate::CloudClient;

/// Unified connectivity handler combining VPC Peering, PSC, and Transit Gateway operations.
///
/// `ConnectivityHandler` is a backward-compatibility facade: each method
/// delegates to the underlying specialized handler. New code should prefer
/// the specialized handlers directly — they expose the full surface for their
/// respective connectivity domains:
///
/// - [`VpcPeeringHandler`] — VPC peering for AWS, GCP, and Azure
/// - [`PscHandler`] — Google Cloud Private Service Connect
/// - [`TransitGatewayHandler`] — AWS Transit Gateway attachments
///
/// Errors returned from delegated calls propagate unchanged; see each backing
/// handler's documentation for the full error surface.
///
/// # Example
///
/// ```rust,no_run
/// use redis_cloud::{CloudClient, ConnectivityHandler};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = CloudClient::builder()
///     .api_key("key")
///     .api_secret("secret")
///     .build()?;
///
/// let connectivity = ConnectivityHandler::new(client);
/// let _vpc_task = connectivity.get_vpc_peering(123).await?;
/// # Ok(())
/// # }
/// ```
pub struct ConnectivityHandler {
    /// VPC peering sub-handler. See [`VpcPeeringHandler`].
    pub vpc_peering: VpcPeeringHandler,
    /// Private Service Connect sub-handler. See [`PscHandler`].
    pub psc: PscHandler,
    /// Transit Gateway sub-handler. See [`TransitGatewayHandler`].
    pub transit_gateway: TransitGatewayHandler,
}

impl ConnectivityHandler {
    /// Construct a `ConnectivityHandler` wired to the supplied [`CloudClient`].
    ///
    /// Internally clones the client into each of the three sub-handlers, so
    /// `client` can be dropped after construction if not needed elsewhere.
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self {
            vpc_peering: VpcPeeringHandler::new(client.clone()),
            psc: PscHandler::new(client.clone()),
            transit_gateway: TransitGatewayHandler::new(client),
        }
    }

    // ========================================================================
    // VPC Peering delegation methods
    // ========================================================================

    /// Get the VPC peering configuration for a Pro subscription.
    ///
    /// `GET /subscriptions/{subscriptionId}/peerings`. Delegates to
    /// [`VpcPeeringHandler::get`].
    ///
    /// # Errors
    ///
    /// Propagates any [`crate::CloudError`] returned by the underlying call —
    /// typically `NotFound` if the subscription has no peering configured,
    /// `AuthenticationFailed` for bad credentials, or `InternalServerError`.
    pub async fn get_vpc_peering(
        &self,
        subscription_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.vpc_peering.get(subscription_id).await
    }

    /// Create a VPC peering on a Pro subscription.
    ///
    /// `POST /subscriptions/{subscriptionId}/peerings`. Delegates to
    /// [`VpcPeeringHandler::create`]. Use
    /// [`VpcPeeringCreateRequest::for_aws`] or
    /// [`VpcPeeringCreateRequest::for_gcp`] to build a provider-targeted body
    /// with the spec's required fields.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses. `BadRequest` is common when the body is missing
    /// provider-required fields.
    pub async fn create_vpc_peering(
        &self,
        subscription_id: i32,
        request: &VpcPeeringCreateRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.vpc_peering.create(subscription_id, request).await
    }

    /// Delete a VPC peering by its peering ID.
    ///
    /// `DELETE /subscriptions/{subscriptionId}/peerings/{peeringId}`.
    /// Delegates to [`VpcPeeringHandler::delete`]. The deletion is asynchronous;
    /// the returned [`TaskStateUpdate`](crate::types::TaskStateUpdate) carries a
    /// `task_id` that can be polled to completion.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for non-2xx responses; `NotFound` if the
    /// peering does not exist.
    pub async fn delete_vpc_peering(
        &self,
        subscription_id: i32,
        peering_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.vpc_peering.delete(subscription_id, peering_id).await
    }

    /// Update a VPC peering's CIDR list.
    ///
    /// `PUT /subscriptions/{subscriptionId}/peerings/{peeringId}`. Translates
    /// the AWS-only [`VpcPeeringUpdateAwsRequest`] into the underlying
    /// [`VpcPeeringCreateRequest`] shape and delegates to
    /// [`VpcPeeringHandler::update`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses.
    pub async fn update_vpc_peering(
        &self,
        subscription_id: i32,
        peering_id: i32,
        request: &VpcPeeringUpdateAwsRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        let create_request = VpcPeeringCreateRequest {
            provider: None,
            command_type: request.command_type.clone(),
            vpc_cidr: request.vpc_cidr.clone(),
            vpc_cidrs: request.vpc_cidrs.clone(),
            ..Default::default()
        };
        self.vpc_peering
            .update(subscription_id, peering_id, &create_request)
            .await
    }

    // ========================================================================
    // PSC (Private Service Connect) delegation methods
    // ========================================================================

    /// Get the Private Service Connect service for a Pro subscription.
    ///
    /// `GET /subscriptions/{subscriptionId}/private-service-connect`.
    /// Delegates to [`PscHandler::get_service`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses. `NotFound` is returned when no PSC service is configured.
    pub async fn get_psc_service(
        &self,
        subscription_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.psc.get_service(subscription_id).await
    }

    /// Create a Private Service Connect service on a Pro subscription.
    ///
    /// `POST /subscriptions/{subscriptionId}/private-service-connect`.
    /// Delegates to [`PscHandler::create_service`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses.
    pub async fn create_psc_service(
        &self,
        subscription_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.psc.create_service(subscription_id).await
    }

    /// Delete the Private Service Connect service for a Pro subscription.
    ///
    /// `DELETE /subscriptions/{subscriptionId}/private-service-connect`.
    /// Delegates to [`PscHandler::delete_service`]. The deletion is
    /// asynchronous; the returned
    /// [`TaskStateUpdate`](crate::types::TaskStateUpdate) carries a `task_id`
    /// that can be polled to completion.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for non-2xx responses.
    pub async fn delete_psc_service(
        &self,
        subscription_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.psc.delete_service(subscription_id).await
    }

    /// Create a PSC endpoint under the subscription's PSC service.
    ///
    /// `POST /subscriptions/{subscriptionId}/private-service-connect/.../endpoints`.
    /// Delegates to [`PscHandler::create_endpoint`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses; `BadRequest` if the endpoint payload is malformed.
    pub async fn create_psc_endpoint(
        &self,
        subscription_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.psc.create_endpoint(subscription_id, request).await
    }

    // ========================================================================
    // Transit Gateway delegation methods
    // ========================================================================

    /// List Transit Gateway attachments for a Pro subscription.
    ///
    /// `GET /subscriptions/{subscriptionId}/transitGateways`. Delegates to
    /// [`TransitGatewayHandler::get_attachments`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses.
    pub async fn get_tgws(
        &self,
        subscription_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.transit_gateway.get_attachments(subscription_id).await
    }

    /// Create a Transit Gateway attachment for the given TGW ID.
    ///
    /// `POST /subscriptions/{subscriptionId}/transitGateways/{tgwId}`.
    /// Delegates to [`TransitGatewayHandler::create_attachment_with_id`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses; `BadRequest` if `tgw_id` is not in a valid AWS format.
    pub async fn create_tgw_attachment(
        &self,
        subscription_id: i32,
        tgw_id: &str,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.transit_gateway
            .create_attachment_with_id(subscription_id, tgw_id)
            .await
    }

    /// Delete a Transit Gateway attachment by attachment ID.
    ///
    /// `DELETE /subscriptions/{subscriptionId}/transitGateways/{tgwId}`.
    /// Delegates to [`TransitGatewayHandler::delete_attachment`]. The deletion
    /// is asynchronous; the returned
    /// [`TaskStateUpdate`](crate::types::TaskStateUpdate) carries a `task_id`
    /// that can be polled to completion.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for non-2xx responses; `NotFound` if
    /// the attachment does not exist.
    pub async fn delete_tgw_attachment(
        &self,
        subscription_id: i32,
        attachment_id: i32,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.transit_gateway
            .delete_attachment(subscription_id, attachment_id.to_string())
            .await
    }

    /// Update the CIDR list on a Transit Gateway attachment.
    ///
    /// `PUT /subscriptions/{subscriptionId}/transitGateways/{tgwId}/attachment`.
    /// Translates [`TgwUpdateCidrsRequest`] (which carries CIDR-with-status
    /// entries) into the [`TgwAttachmentRequest`] the underlying handler
    /// expects and delegates to
    /// [`TransitGatewayHandler::update_attachment_cidrs`]. Only the
    /// `cidr_address` portion of each CIDR entry is forwarded.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses; `BadRequest` if any CIDR string is not in valid form.
    pub async fn update_tgw_cidrs(
        &self,
        subscription_id: i32,
        attachment_id: &str,
        request: &TgwUpdateCidrsRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        let attachment_request = TgwAttachmentRequest {
            aws_account_id: None,
            tgw_id: None,
            cidrs: request.cidrs.as_ref().map(|cidrs| {
                cidrs
                    .iter()
                    .filter_map(|c| c.cidr_address.clone())
                    .collect()
            }),
        };
        self.transit_gateway
            .update_attachment_cidrs(
                subscription_id,
                attachment_id.to_string(),
                &attachment_request,
            )
            .await
    }

    // ========================================================================
    // Backward-compatibility shims
    // ========================================================================

    /// Update a PSC endpoint by endpoint ID. Backward-compatibility wrapper
    /// over [`PscHandler::update_endpoint`].
    ///
    /// `PUT /subscriptions/{subscriptionId}/private-service-connect/.../endpoints/{endpointId}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CloudError`] for transport, auth, or 4xx/5xx
    /// responses.
    pub async fn update_psc_service_endpoint(
        &self,
        subscription_id: i32,
        endpoint_id: i32,
        request: &PscEndpointUpdateRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.psc
            .update_endpoint(subscription_id, endpoint_id, request)
            .await
    }

    /// Alias for [`Self::update_tgw_cidrs`] retained for backward
    /// compatibility. New code should call `update_tgw_cidrs` directly.
    ///
    /// # Errors
    ///
    /// Forwards any [`crate::CloudError`] from the wrapped call.
    pub async fn update_tgw_attachment_cidrs(
        &self,
        subscription_id: i32,
        attachment_id: &str,
        request: &TgwUpdateCidrsRequest,
    ) -> crate::Result<crate::types::TaskStateUpdate> {
        self.update_tgw_cidrs(subscription_id, attachment_id, request)
            .await
    }
}
