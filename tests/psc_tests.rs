use redis_cloud::connectivity::psc::{
    GcpCreationScript, GcpDeletionScript, PrivateServiceConnectEndpoints,
    PrivateServiceConnectService, PscEndpointUpdateRequest,
};
use redis_cloud::types::TaskStateUpdate;
use redis_cloud::{CloudClient, CloudError, PscHandler};
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

fn endpoint_request() -> PscEndpointUpdateRequest {
    PscEndpointUpdateRequest {
        subscription_id: 123,
        psc_service_id: 456,
        endpoint_id: 789,
        gcp_project_id: Some("project-id".to_string()),
        gcp_vpc_name: Some("application-vpc".to_string()),
        gcp_vpc_subnet_name: Some("application-subnet".to_string()),
        endpoint_connection_name: Some("redis-endpoint".to_string()),
    }
}

fn endpoint_body() -> Value {
    json!({
        "subscriptionId": 123,
        "pscServiceId": 456,
        "endpointId": 789,
        "gcpProjectId": "project-id",
        "gcpVpcName": "application-vpc",
        "gcpVpcSubnetName": "application-subnet",
        "endpointConnectionName": "redis-endpoint"
    })
}

fn task_body(task_id: &str) -> Value {
    json!({
        "taskId": task_id,
        "commandType": "PSC_OPERATION",
        "status": "processing-in-progress",
        "description": "PSC operation"
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

async fn mount_script(server: &MockServer, request_path: &str, script: &str) {
    Mock::given(method("GET"))
        .and(path(request_path))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(script))
        .expect(1)
        .mount(server)
        .await;
}

fn assert_task(task: TaskStateUpdate, expected_id: &str) {
    assert_eq!(task.task_id.as_deref(), Some(expected_id));
    assert_eq!(task.command_type.as_deref(), Some("PSC_OPERATION"));
}

#[tokio::test]
async fn standard_psc_routes_use_expected_methods_paths_and_bodies() {
    let server = MockServer::start().await;
    let service_path = "/subscriptions/123/private-service-connect";
    let endpoints_path = "/subscriptions/123/private-service-connect/456";
    let endpoint_path = "/subscriptions/123/private-service-connect/456/endpoints/789";

    mount_task(&server, "GET", service_path, "get-service", None).await;
    mount_task(
        &server,
        "POST",
        service_path,
        "create-service",
        Some(json!({})),
    )
    .await;
    mount_task(&server, "DELETE", service_path, "delete-service", None).await;
    mount_task(
        &server,
        "POST",
        endpoints_path,
        "create-endpoint",
        Some(endpoint_body()),
    )
    .await;
    mount_task(&server, "GET", endpoints_path, "get-endpoints", None).await;
    mount_task(
        &server,
        "PUT",
        endpoint_path,
        "update-endpoint",
        Some(endpoint_body()),
    )
    .await;
    mount_task(&server, "DELETE", endpoint_path, "delete-endpoint", None).await;
    mount_script(
        &server,
        &format!("{endpoint_path}/creationScripts"),
        "create script",
    )
    .await;
    mount_script(
        &server,
        &format!("{endpoint_path}/deletionScripts"),
        "delete script",
    )
    .await;

    let handler = client(&server).psc();
    assert_task(handler.get_service(123).await.unwrap(), "get-service");
    assert_task(handler.create_service(123).await.unwrap(), "create-service");
    assert_task(handler.delete_service(123).await.unwrap(), "delete-service");
    assert_task(
        handler
            .create_endpoint(123, 456, &endpoint_request())
            .await
            .unwrap(),
        "create-endpoint",
    );
    assert_task(
        handler.get_endpoints(123, 456).await.unwrap(),
        "get-endpoints",
    );
    assert_task(
        handler
            .update_endpoint(123, 456, 789, &endpoint_request())
            .await
            .unwrap(),
        "update-endpoint",
    );
    assert_task(
        handler.delete_endpoint(123, 456, 789).await.unwrap(),
        "delete-endpoint",
    );
    assert_eq!(
        handler
            .get_endpoint_creation_script(123, 456, 789)
            .await
            .unwrap(),
        "create script"
    );
    assert_eq!(
        handler
            .get_endpoint_deletion_script(123, 456, 789)
            .await
            .unwrap(),
        "delete script"
    );
}

#[tokio::test]
async fn active_active_psc_routes_include_the_region() {
    let server = MockServer::start().await;
    let service_path = "/subscriptions/123/regions/7/private-service-connect";
    let endpoints_path = "/subscriptions/123/regions/7/private-service-connect/456";
    let endpoint_path = "/subscriptions/123/regions/7/private-service-connect/456/endpoints/789";

    mount_task(&server, "GET", service_path, "aa-get-service", None).await;
    mount_task(
        &server,
        "POST",
        service_path,
        "aa-create-service",
        Some(json!({})),
    )
    .await;
    mount_task(&server, "DELETE", service_path, "aa-delete-service", None).await;
    mount_task(
        &server,
        "POST",
        endpoints_path,
        "aa-create-endpoint",
        Some(endpoint_body()),
    )
    .await;
    mount_task(&server, "GET", endpoints_path, "aa-get-endpoints", None).await;
    mount_task(
        &server,
        "PUT",
        endpoint_path,
        "aa-update-endpoint",
        Some(endpoint_body()),
    )
    .await;
    mount_task(&server, "DELETE", endpoint_path, "aa-delete-endpoint", None).await;
    mount_script(
        &server,
        &format!("{endpoint_path}/creationScripts"),
        "aa create script",
    )
    .await;
    mount_script(
        &server,
        &format!("{endpoint_path}/deletionScripts"),
        "aa delete script",
    )
    .await;

    let handler = PscHandler::new(client(&server));
    assert_task(
        handler.get_service_active_active(123, 7).await.unwrap(),
        "aa-get-service",
    );
    assert_task(
        handler.create_service_active_active(123, 7).await.unwrap(),
        "aa-create-service",
    );
    assert_task(
        handler.delete_service_active_active(123, 7).await.unwrap(),
        "aa-delete-service",
    );
    assert_task(
        handler
            .create_endpoint_active_active(123, 7, 456, &endpoint_request())
            .await
            .unwrap(),
        "aa-create-endpoint",
    );
    assert_task(
        handler
            .get_endpoints_active_active(123, 7, 456)
            .await
            .unwrap(),
        "aa-get-endpoints",
    );
    assert_task(
        handler
            .update_endpoint_active_active(123, 7, 456, 789, &endpoint_request())
            .await
            .unwrap(),
        "aa-update-endpoint",
    );
    assert_task(
        handler
            .delete_endpoint_active_active(123, 7, 456, 789)
            .await
            .unwrap(),
        "aa-delete-endpoint",
    );
    assert_eq!(
        handler
            .get_endpoint_creation_script_active_active(123, 7, 456, 789)
            .await
            .unwrap(),
        "aa create script"
    );
    assert_eq!(
        handler
            .get_endpoint_deletion_script_active_active(123, 7, 456, 789)
            .await
            .unwrap(),
        "aa delete script"
    );
}

#[test]
fn psc_response_models_round_trip_their_wire_fields() {
    let service_raw = json!({
        "id": 456,
        "connectionHostName": "psc.example.com",
        "serviceAttachmentName": "projects/p/regions/r/serviceAttachments/a",
        "status": "active"
    });
    let service: PrivateServiceConnectService =
        serde_json::from_value(service_raw.clone()).unwrap();
    assert_eq!(service.id, Some(456));
    assert_eq!(serde_json::to_value(service).unwrap(), service_raw);

    let endpoints_raw = json!({
        "pscServiceId": 456,
        "endpoints": [{
            "id": 789,
            "gcpProjectId": "project-id",
            "gcpVpcName": "application-vpc",
            "gcpVpcSubnetName": "application-subnet",
            "endpointConnectionName": "redis-endpoint",
            "status": "active"
        }]
    });
    let endpoints: PrivateServiceConnectEndpoints =
        serde_json::from_value(endpoints_raw.clone()).unwrap();
    assert_eq!(endpoints.endpoints.as_ref().map(Vec::len), Some(1));
    assert_eq!(serde_json::to_value(endpoints).unwrap(), endpoints_raw);

    let creation_raw = json!({
        "bash": "gcloud create",
        "powershell": "gcloud create",
        "terraformGcp": {
            "serviceAttachments": [{
                "name": "attachment",
                "dnsRecord": "redis.example.com",
                "ipAddressName": "redis-ip",
                "forwardingRuleName": "redis-forwarding-rule"
            }]
        }
    });
    let creation: GcpCreationScript = serde_json::from_value(creation_raw.clone()).unwrap();
    assert_eq!(
        creation
            .terraform_gcp
            .as_ref()
            .and_then(|terraform| terraform.service_attachments.as_ref())
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(serde_json::to_value(creation).unwrap(), creation_raw);

    let deletion_raw = json!({"bash": "gcloud delete", "powershell": "gcloud delete"});
    let deletion: GcpDeletionScript = serde_json::from_value(deletion_raw.clone()).unwrap();
    assert_eq!(deletion.bash.as_deref(), Some("gcloud delete"));
    assert_eq!(serde_json::to_value(deletion).unwrap(), deletion_raw);
}

#[tokio::test]
async fn psc_errors_propagate_from_the_client() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/subscriptions/123/private-service-connect"))
        .respond_with(ResponseTemplate::new(404).set_body_string("PSC service missing"))
        .mount(&server)
        .await;

    let error = client(&server).psc().get_service(123).await.unwrap_err();
    match error {
        CloudError::NotFound { message } => assert_eq!(message, "PSC service missing"),
        other => panic!("expected NotFound, got {other:?}"),
    }
}
