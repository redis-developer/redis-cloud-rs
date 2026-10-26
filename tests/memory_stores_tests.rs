use redis_cloud::memory_stores::{
    MemoryCustomFieldType, MemoryDetectorAction, MemoryExtractionStrategy, MemoryProblemDetails,
    MemoryStore, MemoryStoreApiKeyCreateRequest, MemoryStoreCreateRequest,
    MemoryStoreCustomDetector, MemoryStoreCustomDetectorsConfig,
    MemoryStoreLongTermMemoryExclusions, MemoryStoreMatcherSpec, MemoryStoreModelConfig,
    MemoryStoreModelCredentials, MemoryStoreShortMemoryConfig, MemoryStoreStatus,
    MemoryStoreUpdateRequest,
};
use redis_cloud::types::TaskStateUpdate;
use redis_cloud::{CloudClient, CloudError, MemoryStoreHandler};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_KEY_ID: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn client(server: &MockServer) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key")
        .api_secret("test-secret")
        .base_url(server.uri())
        .build()
        .expect("test client should build")
}

fn task_body(task_id: &str) -> Value {
    json!({
        "taskId": task_id,
        "commandType": "MEMORY_STORE_OPERATION",
        "status": "received",
        "description": "Memory store operation"
    })
}

fn store_body() -> Value {
    json!({
        "storeId": "store-123",
        "name": "my-memory-store",
        "createdAt": "2026-10-01T12:00:00Z",
        "databaseId": "12345",
        "databaseName": "memory-db",
        "databaseProvider": "AWS",
        "databaseRegion": "us-east-1",
        "databaseAccessCidrs": ["10.0.0.0/16"],
        "subscriptionId": "678",
        "shortMemory": {"ttlSeconds": 86400},
        "longTermMemory": {
            "ttlSeconds": 31536000,
            "embedding": {"provider": "openai", "model": "text-embedding-3-small"}
        },
        "llm": {"provider": "openai", "model": "gpt-4o-mini"},
        "endpoint": "https://memory.example.com",
        "endpoints": [{
            "url": "https://memory.example.com",
            "provider": "AWS",
            "region": "us-east-1",
            "egressIps": ["203.0.113.10"],
            "isAccessible": true
        }],
        "extractionStrategy": "INSTRUCT",
        "summarization": {
            "enabled": true,
            "triggerStrategy": "event_count",
            "eventCount": {"threshold": 50, "retainCount": 10}
        },
        "longTermMemoryExclusions": {
            "enabled": true,
            "semantic": {"enabled": true, "prompt": "Do not remember card numbers"},
            "builtInDetectors": {
                "enabled": true,
                "detectors": [{"id": "credit-card", "enabled": true, "action": "drop"}]
            },
            "customDetectors": {
                "enabled": true,
                "detectors": [{
                    "name": "internal-ticket-id",
                    "enabled": true,
                    "action": "redact",
                    "matcher": {"kind": "regex", "regex": {"pattern": "TICKET-\\d+"}}
                }]
            }
        },
        "customMemoryTypes": [{
            "name": "travel_preference",
            "description": "Preferences the user expressed about travel",
            "fields": [
                {"name": "preferred_airline", "description": "Airline", "type": "str"},
                {"name": "seats", "description": "Seat choices", "type": "list[str]"}
            ],
            "extractionStrategy": {"prompt": "Extract travel preferences", "enabled": true}
        }],
        "extractionCadence": {"activeIntervalSeconds": 120},
        "status": "READY"
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
    assert_eq!(task.command_type.as_deref(), Some("MEMORY_STORE_OPERATION"));
}

#[tokio::test]
async fn async_memory_store_routes_return_tasks() {
    let server = MockServer::start().await;
    let store_path = "/memory-stores/store-123";

    let create = MemoryStoreCreateRequest::builder()
        .name("my-memory-store")
        .database_id(12345)
        .short_memory(MemoryStoreShortMemoryConfig { ttl_seconds: 86400 })
        .llm(MemoryStoreModelConfig {
            provider: Some("openai".to_string()),
            model: Some("gpt-4o-mini".to_string()),
            credentials: Some(MemoryStoreModelCredentials::api_key("sk-test")),
        })
        .extraction_strategy(MemoryExtractionStrategy::Instruct)
        .build();
    mount_task(
        &server,
        "POST",
        "/memory-stores",
        "create",
        Some(json!({
            "name": "my-memory-store",
            "databaseId": 12345,
            "shortMemory": {"ttlSeconds": 86400},
            "llm": {
                "provider": "openai",
                "model": "gpt-4o-mini",
                "credentials": {"type": "apiKey", "apiKey": "sk-test"}
            },
            "extractionStrategy": "INSTRUCT"
        })),
    )
    .await;

    let update = MemoryStoreUpdateRequest::builder()
        .name("renamed-store")
        .long_term_memory_exclusions(MemoryStoreLongTermMemoryExclusions {
            enabled: true,
            semantic: None,
            built_in_detectors: None,
            custom_detectors: Some(MemoryStoreCustomDetectorsConfig {
                enabled: true,
                detectors: Some(vec![MemoryStoreCustomDetector {
                    name: "internal-ticket-id".to_string(),
                    enabled: true,
                    action: Some(MemoryDetectorAction::Drop),
                    matcher: MemoryStoreMatcherSpec::regex("TICKET-\\d+"),
                }]),
            }),
        })
        .build();
    mount_task(
        &server,
        "PATCH",
        store_path,
        "update",
        Some(json!({
            "name": "renamed-store",
            "longTermMemoryExclusions": {
                "enabled": true,
                "customDetectors": {
                    "enabled": true,
                    "detectors": [{
                        "name": "internal-ticket-id",
                        "enabled": true,
                        "action": "drop",
                        "matcher": {"kind": "regex", "regex": {"pattern": "TICKET-\\d+"}}
                    }]
                }
            }
        })),
    )
    .await;
    mount_task(
        &server,
        "DELETE",
        &format!("{store_path}/entries"),
        "flush",
        None,
    )
    .await;
    mount_task(
        &server,
        "DELETE",
        &format!("{store_path}/session-memory"),
        "empty",
        None,
    )
    .await;

    let handler = client(&server).memory_stores();
    assert_task(handler.create(&create).await.unwrap(), "create");
    assert_task(
        handler.update("store-123", &update).await.unwrap(),
        "update",
    );
    assert_task(handler.flush_entries("store-123").await.unwrap(), "flush");
    assert_task(
        handler.empty_session_memory("store-123").await.unwrap(),
        "empty",
    );
}

#[tokio::test]
async fn delete_passes_the_optional_flush_query() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/memory-stores/store-123"))
        .and(query_param("flush", "true"))
        .respond_with(ResponseTemplate::new(202).set_body_json(task_body("delete-flush")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/memory-stores/store-456"))
        .respond_with(ResponseTemplate::new(202).set_body_json(task_body("delete")))
        .expect(1)
        .mount(&server)
        .await;

    let handler = MemoryStoreHandler::new(client(&server));
    assert_task(
        handler.delete("store-123", Some(true)).await.unwrap(),
        "delete-flush",
    );
    assert_task(handler.delete("store-456", None).await.unwrap(), "delete");

    let requests = server.received_requests().await.unwrap();
    let unflushed = requests
        .iter()
        .find(|r| r.url.path() == "/memory-stores/store-456")
        .unwrap();
    assert_eq!(unflushed.url.query(), None);
}

#[tokio::test]
async fn list_and_get_deserialize_store_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/memory-stores"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "storeId": "store-123",
                "name": "my-memory-store",
                "databaseId": "12345",
                "subscriptionId": "678",
                "extractionStrategy": "INSTRUCT",
                "status": "PROVISIONING"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/memory-stores/store-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(store_body()))
        .expect(1)
        .mount(&server)
        .await;

    let handler = client(&server).memory_stores();
    let list = handler.list().await.unwrap();
    let stores = list.data.unwrap();
    assert_eq!(stores.len(), 1);
    assert_eq!(stores[0].store_id.as_deref(), Some("store-123"));
    assert_eq!(stores[0].status, Some(MemoryStoreStatus::Provisioning));

    let store = handler.get("store-123").await.unwrap();
    assert_eq!(store.status, Some(MemoryStoreStatus::Ready));
    assert_eq!(
        store.short_memory.as_ref().map(|s| s.ttl_seconds),
        Some(86400)
    );
    let custom = store.custom_memory_types.as_ref().unwrap();
    assert_eq!(
        custom[0].fields.as_ref().unwrap()[1].field_type,
        MemoryCustomFieldType::ListStr
    );
}

#[tokio::test]
async fn api_key_routes_use_expected_methods_paths_and_bodies() {
    let server = MockServer::start().await;
    let keys_path = "/memory-stores/store-123/api-keys";

    Mock::given(method("GET"))
        .and(path(keys_path))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([{"keyName": "my-agent-key"}])),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(keys_path))
        .and(body_json(json!({"keyName": "my-agent-key"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"apiKey": "secret"})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(format!("{keys_path}/{API_KEY_ID}")))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let handler = client(&server).memory_stores();
    let keys = handler.list_api_keys("store-123").await.unwrap();
    assert_eq!(keys[0]["keyName"], "my-agent-key");

    let created = handler
        .create_api_key(
            "store-123",
            &MemoryStoreApiKeyCreateRequest::new("my-agent-key"),
        )
        .await
        .unwrap();
    assert_eq!(created.api_key.as_deref(), Some("secret"));

    handler
        .delete_api_key("store-123", API_KEY_ID)
        .await
        .unwrap();
}

#[test]
fn memory_store_model_round_trips_its_wire_fields() {
    let raw = store_body();
    let store: MemoryStore = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(store).unwrap(), raw);
}

#[test]
fn memory_store_models_tolerate_unknown_fields_and_values() {
    let store: MemoryStore = serde_json::from_value(json!({
        "storeId": "store-123",
        "status": "MIGRATING",
        "extractionStrategy": "SOMETHING_NEW",
        "brandNewField": {"nested": true}
    }))
    .unwrap();
    assert_eq!(store.status, Some(MemoryStoreStatus::Unknown));
    assert_eq!(
        store.extraction_strategy,
        Some(MemoryExtractionStrategy::Unknown)
    );
}

#[tokio::test]
async fn problem_details_are_recoverable_from_the_error() {
    let server = MockServer::start().await;
    let problem = json!({
        "type": "/errors/invalid-data",
        "title": "Invalid data",
        "status": 400,
        "detail": "request validation failed",
        "fields": [{
            "field": "keyName",
            "rule": "pattern",
            "message": "must not contain whitespace"
        }]
    });
    Mock::given(method("POST"))
        .and(path("/memory-stores/store-123/api-keys"))
        .respond_with(
            ResponseTemplate::new(424)
                .insert_header("content-type", "application/problem+json")
                .set_body_json(problem),
        )
        .mount(&server)
        .await;

    let error = client(&server)
        .memory_stores()
        .create_api_key("store-123", &MemoryStoreApiKeyCreateRequest::new("bad key"))
        .await
        .unwrap_err();
    assert!(matches!(error, CloudError::ApiError { code: 424, .. }));

    let details = MemoryProblemDetails::from_error(&error).expect("problem details");
    assert_eq!(
        details.problem_type.as_deref(),
        Some("/errors/invalid-data")
    );
    assert_eq!(details.status, Some(400));
    let fields = details.fields.unwrap();
    assert_eq!(fields[0].field.as_deref(), Some("keyName"));
    assert_eq!(fields[0].rule.as_deref(), Some("pattern"));
}

#[test]
fn problem_details_ignore_non_problem_errors() {
    let plain = CloudError::NotFound {
        message: "store missing".to_string(),
    };
    assert!(MemoryProblemDetails::from_error(&plain).is_none());

    let unrelated_json = CloudError::BadRequest {
        message: r#"{"error":{"code":"X"}}"#.to_string(),
    };
    assert!(MemoryProblemDetails::from_error(&unrelated_json).is_none());

    assert!(MemoryProblemDetails::from_error(&CloudError::PreconditionFailed).is_none());
}
