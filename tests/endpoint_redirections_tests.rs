use redis_cloud::{
    CloudClient, CreateEndpointsRedirectionRequest, EndpointRedirectionStatus, EndpointTargetType,
};
use serde_json::json;
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

fn response(status: &str, is_revert: bool) -> serde_json::Value {
    json!({
        "redirectionId": "550e8400-e29b-41d4-a716-446655440000",
        "status": status,
        "sourceDatabaseId": 101,
        "targetDatabaseId": 202,
        "isRevert": is_revert,
        "duplicateACLs": true,
        "startedAt": "2026-08-11T12:00:00Z",
        "endpoints": [{
            "sourceEndpointName": "redis-source",
            "targetEndpointName": "redis-target",
            "sourceEndpointType": "public",
            "targetEndpointType": "public"
        }]
    })
}

#[test]
fn test_unknown_endpoint_redirection_enums_are_forward_compatible() {
    let response: redis_cloud::EndpointsRedirectionResponse = serde_json::from_value(json!({
        "status": "future-state",
        "endpoints": [{
            "sourceEndpointType": "future-endpoint",
            "targetEndpointType": "public"
        }]
    }))
    .unwrap();

    assert_eq!(response.status, Some(EndpointRedirectionStatus::Unknown));
    assert_eq!(
        response.endpoints.unwrap()[0].source_endpoint_type,
        Some(EndpointTargetType::Unknown)
    );
}

#[tokio::test]
async fn test_create_endpoint_redirection() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/endpoint-redirections"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .and(body_json(json!({
            "sourceDatabaseId": 101,
            "targetDatabaseId": 202,
            "endpointTargetType": "public",
            "duplicateACLs": true,
            "migrationProtection": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response("in-progress", false)))
        .mount(&server)
        .await;

    let request = CreateEndpointsRedirectionRequest::builder()
        .source_database_id(101)
        .target_database_id(202)
        .endpoint_target_type(EndpointTargetType::Public)
        .duplicate_acls(true)
        .migration_protection(true)
        .build();
    let result = client(&server)
        .endpoint_redirections()
        .create(&request)
        .await
        .unwrap();

    assert_eq!(result.status, Some(EndpointRedirectionStatus::InProgress));
    assert_eq!(result.endpoints.as_ref().map(Vec::len), Some(1));
}

#[tokio::test]
async fn test_get_endpoint_redirection() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/endpoint-redirections/550e8400-e29b-41d4-a716-446655440000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(response("completed", false)))
        .mount(&server)
        .await;

    let result = client(&server)
        .endpoint_redirections()
        .get("550e8400-e29b-41d4-a716-446655440000")
        .await
        .unwrap();
    assert_eq!(result.status, Some(EndpointRedirectionStatus::Completed));
}

#[tokio::test]
async fn test_revert_endpoint_redirection_without_request_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/endpoint-redirections/550e8400-e29b-41d4-a716-446655440000/revert",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(response("reverted", true)))
        .mount(&server)
        .await;

    let result = client(&server)
        .endpoint_redirections()
        .revert("550e8400-e29b-41d4-a716-446655440000")
        .await
        .unwrap();
    assert_eq!(result.status, Some(EndpointRedirectionStatus::Reverted));
    assert_eq!(result.is_revert, Some(true));
}
