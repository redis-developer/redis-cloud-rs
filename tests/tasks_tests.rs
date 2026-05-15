use redis_cloud::{CloudClient, tasks::TasksHandler};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper: build a Cloud client wired to the given mock server URI.
fn test_client(uri: String) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(uri)
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_get_all_tasks_canonical_wrapper() {
    // OpenAPI spec response shape: TasksStateUpdate { tasks: [TaskStateUpdate] }.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tasks": [
                {
                    "taskId": "task-1",
                    "commandType": "CREATE_DATABASE",
                    "status": "processing-completed",
                    "description": "Created database successfully",
                    "timestamp": "2024-01-01T10:00:00Z",
                    "response": {
                        "resourceId": 456
                    }
                },
                {
                    "taskId": "task-2",
                    "commandType": "UPDATE_SUBSCRIPTION",
                    "status": "processing-in-progress",
                    "description": "Updating subscription",
                    "timestamp": "2024-01-01T11:00:00Z"
                },
                {
                    "taskId": "task-3",
                    "commandType": "DELETE_DATABASE",
                    "status": "processing-error",
                    "description": "Failed to delete database",
                    "timestamp": "2024-01-01T12:00:00Z",
                    "response": {
                        "error": "Database in use"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let handler = TasksHandler::new(test_client(mock_server.uri()));
    let tasks = handler.get_all_tasks().await.unwrap();

    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].task_id, Some("task-1".to_string()));
    assert_eq!(tasks[0].command_type, Some("CREATE_DATABASE".to_string()));
    assert_eq!(tasks[1].status, Some("processing-in-progress".to_string()));
    assert_eq!(tasks[2].task_id, Some("task-3".to_string()));
}

#[tokio::test]
async fn test_get_all_tasks_empty_object() {
    // When the account has no tasks, the API returns `{}` rather than the wrapper.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let handler = TasksHandler::new(test_client(mock_server.uri()));
    let tasks = handler.get_all_tasks().await.unwrap();

    assert!(tasks.is_empty());
}

#[tokio::test]
async fn test_get_all_tasks_legacy_bare_array() {
    // Older responses use a bare array. Still tolerated for backward compatibility.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "taskId": "legacy-1", "status": "processing-completed" }
        ])))
        .mount(&mock_server)
        .await;

    let handler = TasksHandler::new(test_client(mock_server.uri()));
    let tasks = handler.get_all_tasks().await.unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, Some("legacy-1".to_string()));
}

#[tokio::test]
async fn test_get_all_tasks_unknown_shape_errors() {
    // Anything outside the three known shapes should surface a JsonError —
    // never an empty Vec that silently masks a schema regression.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_string("\"unexpected string\""))
        .mount(&mock_server)
        .await;

    let handler = TasksHandler::new(test_client(mock_server.uri()));
    let err = handler.get_all_tasks().await.unwrap_err();

    match err {
        redis_cloud::CloudError::JsonError(msg) => {
            assert!(
                msg.contains("GET /tasks"),
                "JsonError message should mention the endpoint: {msg}"
            );
        }
        other => panic!("expected CloudError::JsonError, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_get_task_by_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks/task-123"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "taskId": "task-123",
            "commandType": "CREATE_DATABASE",
            "status": "completed",
            "description": "Database created successfully",
            "timestamp": "2024-01-01T00:00:00Z",
            "response": {
                "resourceId": 789,
                "databaseName": "production-db",
                "endpoint": "redis-123.c1.us-east-1-2.ec2.cloud.redislabs.com:12345"
            },
            "links": [
                {
                    "href": "https://api.redislabs.com/v1/subscriptions/123/databases/789",
                    "rel": "database",
                    "type": "GET"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler
        .get_task_by_id("task-123".to_string())
        .await
        .unwrap();

    assert_eq!(result.task_id, Some("task-123".to_string()));
    assert_eq!(result.command_type, Some("CREATE_DATABASE".to_string()));
    assert_eq!(result.status, Some("completed".to_string()));
    assert!(result.response.is_some());
}

#[tokio::test]
async fn test_get_task_by_id_processing() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks/task-456"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "taskId": "task-456",
            "commandType": "UPDATE_SUBSCRIPTION",
            "status": "processing",
            "description": "Updating subscription configuration",
            "timestamp": "2024-01-01T00:00:00Z",
            "progress": 65
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler
        .get_task_by_id("task-456".to_string())
        .await
        .unwrap();

    assert_eq!(result.task_id, Some("task-456".to_string()));
    assert_eq!(result.status, Some("processing".to_string()));
}

#[tokio::test]
async fn test_get_task_by_id_failed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks/task-789"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "taskId": "task-789",
            "commandType": "DELETE_DATABASE",
            "status": "failed",
            "description": "Failed to delete database",
            "timestamp": "2024-01-01T00:00:00Z",
            "response": {
                "error": "Database not found or already deleted",
                "errorCode": "DATABASE_NOT_FOUND"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler
        .get_task_by_id("task-789".to_string())
        .await
        .unwrap();

    assert_eq!(result.task_id, Some("task-789".to_string()));
    assert_eq!(result.status, Some("failed".to_string()));
    assert!(result.response.is_some());
}

#[tokio::test]
async fn test_error_handling_401() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API credentials"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("wrong-key".to_string())
        .api_secret("wrong-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler.get_all_tasks().await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::AuthenticationFailed { .. }) => {}
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[tokio::test]
async fn test_error_handling_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks/task-nonexistent"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Task not found"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler.get_task_by_id("task-nonexistent".to_string()).await;

    assert!(result.is_err());
    if let Err(redis_cloud::CloudError::NotFound { message }) = result {
        assert!(message.contains("not found") || message.contains("404"));
    } else {
        panic!("Expected NotFound error");
    }
}

#[tokio::test]
async fn test_error_handling_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/tasks/task-500"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = TasksHandler::new(client);
    let result = handler.get_task_by_id("task-500".to_string()).await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::InternalServerError { .. }) => {}
        _ => panic!("Expected InternalServerError error"),
    }
}
