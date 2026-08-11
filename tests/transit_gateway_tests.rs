use redis_cloud::connectivity::transit_gateway::{
    Cidr, CidrStatus, TgwAttachmentRequest, TgwUpdateCidrsRequest, TransitGatewayAttachment,
    TransitGatewayInvitation,
};
use redis_cloud::types::TaskStateUpdate;
use redis_cloud::{CloudClient, CloudError, TransitGatewayHandler};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key")
        .api_secret("test-secret")
        .base_url(server.uri())
        .build()
        .expect("test client should build")
}

fn attachment_request() -> TgwAttachmentRequest {
    TgwAttachmentRequest {
        aws_account_id: Some("123456789012".to_string()),
        tgw_id: Some("tgw-123".to_string()),
        cidrs: Some(vec!["10.0.0.0/16".to_string(), "10.1.0.0/16".to_string()]),
    }
}

fn attachment_body() -> Value {
    json!({
        "awsAccountId": "123456789012",
        "tgwId": "tgw-123",
        "cidrs": ["10.0.0.0/16", "10.1.0.0/16"]
    })
}

fn task_body(task_id: &str) -> Value {
    json!({
        "taskId": task_id,
        "commandType": "TGW_OPERATION",
        "status": "processing-in-progress",
        "description": "Transit Gateway operation"
    })
}

async fn mount_task(
    server: &MockServer,
    verb: &str,
    request_path: &str,
    task_id: &str,
    body: Option<Value>,
) {
    let mock = Mock::given(method(verb))
        .and(path(request_path))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"));
    let mock = if let Some(body) = body {
        mock.and(body_json(body))
    } else {
        mock
    };
    mock.respond_with(ResponseTemplate::new(202).set_body_json(task_body(task_id)))
        .expect(1)
        .mount(server)
        .await;
}

fn assert_task(task: TaskStateUpdate, expected_id: &str) {
    assert_eq!(task.task_id.as_deref(), Some(expected_id));
    assert_eq!(task.command_type.as_deref(), Some("TGW_OPERATION"));
}

#[tokio::test]
async fn standard_tgw_routes_use_expected_methods_paths_and_bodies() {
    let server = MockServer::start().await;
    let gateways_path = "/subscriptions/123/transitGateways";
    let invitations_path = "/subscriptions/123/transitGateways/invitations";
    let attachment_path = "/subscriptions/123/transitGateways/tgw-123/attachment";

    mount_task(&server, "GET", gateways_path, "get-attachments", None).await;
    mount_task(&server, "GET", invitations_path, "get-invitations", None).await;
    mount_task(
        &server,
        "PUT",
        &format!("{invitations_path}/inv-1/accept"),
        "accept-invitation",
        Some(json!({})),
    )
    .await;
    mount_task(
        &server,
        "PUT",
        &format!("{invitations_path}/inv-2/reject"),
        "reject-invitation",
        Some(json!({})),
    )
    .await;
    mount_task(
        &server,
        "POST",
        attachment_path,
        "create-attachment",
        Some(json!({"tgwId": "tgw-123"})),
    )
    .await;
    mount_task(
        &server,
        "PUT",
        attachment_path,
        "update-attachment",
        Some(attachment_body()),
    )
    .await;
    mount_task(
        &server,
        "DELETE",
        attachment_path,
        "delete-attachment",
        None,
    )
    .await;

    let handler = client(&server).transit_gateway();
    assert_task(
        handler.get_attachments(123).await.unwrap(),
        "get-attachments",
    );
    assert_task(
        handler.get_shared_invitations(123).await.unwrap(),
        "get-invitations",
    );
    assert_task(
        handler
            .accept_resource_share(123, "inv-1".to_string())
            .await
            .unwrap(),
        "accept-invitation",
    );
    assert_task(
        handler
            .reject_resource_share(123, "inv-2".to_string())
            .await
            .unwrap(),
        "reject-invitation",
    );
    assert_task(
        handler
            .create_attachment_with_id(123, "tgw-123")
            .await
            .unwrap(),
        "create-attachment",
    );
    assert_task(
        handler
            .update_attachment_cidrs(123, "tgw-123".to_string(), &attachment_request())
            .await
            .unwrap(),
        "update-attachment",
    );
    assert_task(
        handler
            .delete_attachment(123, "tgw-123".to_string())
            .await
            .unwrap(),
        "delete-attachment",
    );
}

#[tokio::test]
async fn active_active_tgw_routes_include_the_region() {
    let server = MockServer::start().await;
    let gateways_path = "/subscriptions/123/regions/7/transitGateways";
    let invitations_path = "/subscriptions/123/regions/7/transitGateways/invitations";
    let attachment_path = "/subscriptions/123/regions/7/transitGateways/tgw-123/attachment";

    mount_task(&server, "GET", gateways_path, "aa-get-attachments", None).await;
    mount_task(&server, "GET", invitations_path, "aa-get-invitations", None).await;
    mount_task(
        &server,
        "PUT",
        &format!("{invitations_path}/inv-1/accept"),
        "aa-accept-invitation",
        Some(json!({})),
    )
    .await;
    mount_task(
        &server,
        "PUT",
        &format!("{invitations_path}/inv-2/reject"),
        "aa-reject-invitation",
        Some(json!({})),
    )
    .await;
    mount_task(
        &server,
        "POST",
        attachment_path,
        "aa-create-attachment",
        Some(attachment_body()),
    )
    .await;
    mount_task(
        &server,
        "PUT",
        attachment_path,
        "aa-update-attachment",
        Some(attachment_body()),
    )
    .await;
    mount_task(
        &server,
        "DELETE",
        attachment_path,
        "aa-delete-attachment",
        None,
    )
    .await;

    let handler = TransitGatewayHandler::new(client(&server));
    assert_task(
        handler.get_attachments_active_active(123, 7).await.unwrap(),
        "aa-get-attachments",
    );
    assert_task(
        handler
            .get_shared_invitations_active_active(123, 7)
            .await
            .unwrap(),
        "aa-get-invitations",
    );
    assert_task(
        handler
            .accept_resource_share_active_active(123, 7, "inv-1".to_string())
            .await
            .unwrap(),
        "aa-accept-invitation",
    );
    assert_task(
        handler
            .reject_resource_share_active_active(123, 7, "inv-2".to_string())
            .await
            .unwrap(),
        "aa-reject-invitation",
    );
    assert_task(
        handler
            .create_attachment_active_active(123, 7, "tgw-123", &attachment_request())
            .await
            .unwrap(),
        "aa-create-attachment",
    );
    assert_task(
        handler
            .update_attachment_cidrs_active_active(
                123,
                7,
                "tgw-123".to_string(),
                &attachment_request(),
            )
            .await
            .unwrap(),
        "aa-update-attachment",
    );
    assert_task(
        handler
            .delete_attachment_active_active(123, 7, "tgw-123".to_string())
            .await
            .unwrap(),
        "aa-delete-attachment",
    );
}

#[test]
fn tgw_request_and_response_models_round_trip_their_wire_fields() {
    let cidrs_raw = json!({
        "cidrs": [{"cidrAddress": "10.0.0.0/16"}],
        "commandType": "UPDATE_TGW_CIDRS"
    });
    let cidrs: TgwUpdateCidrsRequest = serde_json::from_value(cidrs_raw.clone()).unwrap();
    assert_eq!(
        cidrs
            .cidrs
            .as_ref()
            .and_then(|items| items.first())
            .and_then(|cidr| cidr.cidr_address.as_deref()),
        Some("10.0.0.0/16")
    );
    assert_eq!(serde_json::to_value(cidrs).unwrap(), cidrs_raw);

    let request_raw = attachment_body();
    let request: TgwAttachmentRequest = serde_json::from_value(request_raw.clone()).unwrap();
    assert_eq!(request.aws_account_id.as_deref(), Some("123456789012"));
    assert_eq!(serde_json::to_value(request).unwrap(), request_raw);

    let attachment_raw = json!({
        "id": 42,
        "awsTgwUid": "tgw-123",
        "attachmentUid": "tgw-attach-123",
        "status": "active",
        "attachmentStatus": "available",
        "awsAccountId": "123456789012",
        "cidrs": [{"cidrAddress": "10.0.0.0/16", "status": "active"}]
    });
    let attachment: TransitGatewayAttachment =
        serde_json::from_value(attachment_raw.clone()).unwrap();
    assert_eq!(
        attachment
            .cidrs
            .as_ref()
            .and_then(|items| items.first())
            .and_then(|cidr| cidr.status.as_deref()),
        Some("active")
    );
    assert_eq!(serde_json::to_value(attachment).unwrap(), attachment_raw);

    let invitation_raw = json!({
        "id": 9,
        "name": "redis-share",
        "resourceShareUid": "rs-123",
        "awsAccountId": "123456789012",
        "status": "pending",
        "sharedDate": "2026-08-11T12:00:00Z"
    });
    let invitation: TransitGatewayInvitation =
        serde_json::from_value(invitation_raw.clone()).unwrap();
    assert_eq!(invitation.resource_share_uid.as_deref(), Some("rs-123"));
    assert_eq!(serde_json::to_value(invitation).unwrap(), invitation_raw);

    let cidr = Cidr {
        cidr_address: Some("10.2.0.0/16".to_string()),
    };
    assert_eq!(
        serde_json::to_value(cidr).unwrap(),
        json!({"cidrAddress": "10.2.0.0/16"})
    );
    let cidr_status: CidrStatus =
        serde_json::from_value(json!({"cidrAddress": "10.3.0.0/16", "status": "pending"})).unwrap();
    assert_eq!(cidr_status.status.as_deref(), Some("pending"));
}

#[tokio::test]
async fn tgw_errors_propagate_from_the_client() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/subscriptions/123/transitGateways"))
        .respond_with(ResponseTemplate::new(503).set_body_string("TGW temporarily unavailable"))
        .mount(&server)
        .await;

    let error = client(&server)
        .transit_gateway()
        .get_attachments(123)
        .await
        .unwrap_err();
    match error {
        CloudError::ServiceUnavailable { message } => {
            assert_eq!(message, "TGW temporarily unavailable")
        }
        other => panic!("expected ServiceUnavailable, got {other:?}"),
    }
}
