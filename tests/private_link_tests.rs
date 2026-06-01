use redis_cloud::connectivity::{
    PrincipalType, PrivateLinkAddPrincipalRequest, PrivateLinkCreateRequest,
    PrivateLinkRemovePrincipalRequest,
};
use redis_cloud::types::TaskStatus;
use redis_cloud::{CloudClient, PrivateLinkHandler};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A `TaskStateUpdate`-shaped body. Per the OpenAPI spec every private-link
/// operation is asynchronous and returns a task the caller can poll.
fn task_body(task_id: &str, command_type: &str, resource_id: i64) -> serde_json::Value {
    json!({
        "taskId": task_id,
        "commandType": command_type,
        "status": "processing-completed",
        "response": { "resourceId": resource_id }
    })
}

fn test_client(uri: String) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key")
        .api_secret("test-secret")
        .base_url(uri)
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_get_private_link() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/private-link"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-get",
            "PRIVATE_LINK_GET",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let result = handler.get(123).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-get"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingCompleted));
    assert_eq!(result.response.and_then(|r| r.resource_id), Some(123456));
}

#[tokio::test]
async fn test_create_private_link() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/private-link"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-789",
            "PRIVATE_LINK_CREATE",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkCreateRequest {
        share_name: "my-redis-share".to_string(),
        principal: "123456789012".to_string(),
        principal_type: PrincipalType::AwsAccount,
        alias: Some("Production Account".to_string()),
    };

    let result = handler.create(123, &request).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-789"));
    assert_eq!(result.command_type.as_deref(), Some("PRIVATE_LINK_CREATE"));
}

#[tokio::test]
async fn test_add_principals() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/private-link/principals"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-add",
            "PRIVATE_LINK_ADD_PRINCIPAL",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkAddPrincipalRequest {
        principal: "987654321098".to_string(),
        principal_type: Some(PrincipalType::IamRole),
        alias: Some("Dev Role".to_string()),
    };

    let result = handler.add_principals(123, &request).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-add"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingCompleted));
}

#[tokio::test]
async fn test_remove_principals() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/subscriptions/123/private-link/principals"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-remove",
            "PRIVATE_LINK_REMOVE_PRINCIPAL",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkRemovePrincipalRequest {
        principal: "987654321098".to_string(),
        principal_type: Some(PrincipalType::IamRole),
        alias: Some("Dev Role".to_string()),
    };

    let result = handler.remove_principals(123, &request).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-remove"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingCompleted));
}

#[tokio::test]
async fn test_get_endpoint_script() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/private-link/endpoint-script"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-script",
            "PRIVATE_LINK_ENDPOINT_SCRIPT",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let result = handler.get_endpoint_script(123).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-script"));
}

#[tokio::test]
async fn test_get_active_active_private_link() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/regions/1/private-link"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-aa-get",
            "PRIVATE_LINK_GET",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let result = handler.get_active_active(123, 1).await.unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-aa-get"));
    assert_eq!(result.response.and_then(|r| r.resource_id), Some(123456));
}

#[tokio::test]
async fn test_create_active_active_private_link() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/regions/1/private-link"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-999",
            "PRIVATE_LINK_CREATE",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkCreateRequest {
        share_name: "my-crdb-share".to_string(),
        principal: "111222333444".to_string(),
        principal_type: PrincipalType::AwsAccount,
        alias: None,
    };

    let result = handler
        .create_active_active(123, 1, &request)
        .await
        .unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-999"));
    assert_eq!(result.command_type.as_deref(), Some("PRIVATE_LINK_CREATE"));
}

#[tokio::test]
async fn test_add_principals_active_active() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/regions/1/private-link/principals"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-aa-add",
            "PRIVATE_LINK_ADD_PRINCIPAL",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkAddPrincipalRequest {
        principal: "555666777888".to_string(),
        principal_type: Some(PrincipalType::AwsAccount),
        alias: None,
    };

    let result = handler
        .add_principals_active_active(123, 1, &request)
        .await
        .unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-aa-add"));
}

#[tokio::test]
async fn test_remove_principals_active_active() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/subscriptions/123/regions/1/private-link/principals"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-aa-remove",
            "PRIVATE_LINK_REMOVE_PRINCIPAL",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));

    let request = PrivateLinkRemovePrincipalRequest {
        principal: "555666777888".to_string(),
        principal_type: Some(PrincipalType::AwsAccount),
        alias: None,
    };

    let result = handler
        .remove_principals_active_active(123, 1, &request)
        .await
        .unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-aa-remove"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingCompleted));
}

#[tokio::test]
async fn test_get_endpoint_script_active_active() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/subscriptions/123/regions/1/private-link/endpoint-script",
        ))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(task_body(
            "task-aa-script",
            "PRIVATE_LINK_ENDPOINT_SCRIPT",
            123456,
        )))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let result = handler
        .get_endpoint_script_active_active(123, 1)
        .await
        .unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-aa-script"));
}

#[tokio::test]
async fn test_error_handling_401() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/private-link"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("invalid-key")
        .api_secret("invalid-secret")
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = PrivateLinkHandler::new(client);
    let result = handler.get(123).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/999/private-link"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let result = handler.get(999).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/private-link"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let handler = PrivateLinkHandler::new(test_client(mock_server.uri()));
    let request = PrivateLinkCreateRequest {
        share_name: "test".to_string(),
        principal: "123".to_string(),
        principal_type: PrincipalType::AwsAccount,
        alias: None,
    };
    let result = handler.create(123, &request).await;

    assert!(result.is_err());
}
