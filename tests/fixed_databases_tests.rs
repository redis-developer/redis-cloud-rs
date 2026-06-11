use redis_cloud::{CloudClient, FixedDatabaseHandler};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_fixed_subscription_databases() {
    let mock_server = MockServer::start().await;

    // Shape derived from the AccountFixedSubscriptionDatabases.example field in the
    // bundled cloud_openapi.json: subscription is a single object containing
    // subscriptionId, numberOfDatabases, databases[], and links.
    Mock::given(method("GET"))
        .and(path("/fixed/subscriptions/123/databases"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "accountId": 456,
            "subscription": {
                "subscriptionId": 123,
                "numberOfDatabases": 2,
                "databases": [
                    {
                        "databaseId": 51324587,
                        "name": "bdb",
                        "protocol": "stack",
                        "provider": "AWS",
                        "region": "us-east-1",
                        "redisVersion": "7.4",
                        "status": "active",
                        "planMemoryLimit": 250,
                        "memoryLimitMeasurementUnit": "MB",
                        "memoryUsedInMb": 1,
                        "memoryStorage": "ram",
                        "dataPersistence": "none",
                        "replication": true,
                        "dataEvictionPolicy": "noeviction",
                    },
                    {
                        "databaseId": 51324586,
                        "name": "firstDB",
                        "protocol": "stack",
                        "provider": "AWS",
                        "region": "us-east-1",
                        "status": "active",
                        "planMemoryLimit": 250,
                        "memoryLimitMeasurementUnit": "MB",
                        "memoryStorage": "ram",
                    },
                ],
                "links": []
            },
            "links": [
                {
                    "href": "https://api.redislabs.com/v1/fixed/subscriptions/123/databases/1",
                    "rel": "database",
                    "type": "GET"
                }
            ],
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.list(123, None, None).await.unwrap();

    assert_eq!(result.account_id, Some(456));
    assert!(result.links.is_some());

    let subscription = result
        .subscription
        .expect("response should include the subscription object");
    assert_eq!(subscription.subscription_id, Some(123));
    assert_eq!(subscription.number_of_databases, Some(2));
    assert_eq!(subscription.databases.len(), 2);
    assert_eq!(subscription.databases[0].database_id, Some(51324587));
    assert_eq!(subscription.databases[0].name.as_deref(), Some("bdb"));
    assert_eq!(subscription.databases[1].database_id, Some(51324586));
    assert_eq!(subscription.databases[1].name.as_deref(), Some("firstDB"));
}

#[tokio::test]
async fn test_create_fixed_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fixed/subscriptions/123/databases"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-create-fixed-db",
            "commandType": "CREATE_FIXED_DATABASE",
            "status": "processing",
            "description": "Creating fixed database",
            "timestamp": "2024-01-01T00:00:00Z",
            "response": {
                "resourceId": 789
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

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::FixedDatabaseCreateRequest {
        name: "fixed-test-database".to_string(),
        subscription_id: None,
        protocol: None,
        memory_limit_in_gb: Some(1.0),
        dataset_size_in_gb: None,
        support_oss_cluster_api: None,
        redis_version: None,
        resp_version: None,
        use_external_endpoint_for_oss_cluster_api: None,
        enable_database_clustering: None,
        number_of_shards: None,
        data_persistence: None,
        data_eviction_policy: Some("noeviction".to_string()),
        replication: Some(true),
        periodic_backup_path: None,
        source_ips: None,
        regex_rules: None,
        replica_of: None,
        replica: None,
        client_ssl_certificate: None,
        client_tls_certificates: None,
        enable_tls: None,
        password: None,
        alerts: None,
        modules: None,
        command_type: None,
    };

    let result = handler.create(123, &request).await.unwrap();
    assert_eq!(result.task_id, Some("task-create-fixed-db".to_string()));
    assert_eq!(
        result.command_type,
        Some("CREATE_FIXED_DATABASE".to_string())
    );
}

#[tokio::test]
async fn test_fixed_database_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/fixed/subscriptions/123/databases/456"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-update-fixed-db",
            "commandType": "UPDATE_FIXED_DATABASE",
            "status": "processing",
            "description": "Updating fixed database configuration"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::FixedDatabaseUpdateRequest {
        subscription_id: None,
        database_id: None,
        name: Some("updated-fixed-database".to_string()),
        memory_limit_in_gb: Some(2.0),
        dataset_size_in_gb: None,
        data_persistence: None,
        data_eviction_policy: Some("allkeys-lru".to_string()),
        replication: None,
        resp_version: None,
        support_oss_cluster_api: None,
        use_external_endpoint_for_oss_cluster_api: None,
        password: None,
        enable_database_clustering: None,
        number_of_shards: None,
        periodic_backup_path: None,
        source_ips: None,
        regex_rules: None,
        replica_of: None,
        replica: None,
        client_ssl_certificate: None,
        client_tls_certificates: None,
        enable_tls: None,
        enable_default_user: None,
        alerts: None,
        command_type: None,
    };

    let result = handler.update(123, 456, &request).await.unwrap();
    assert_eq!(result.task_id, Some("task-update-fixed-db".to_string()));
}

#[tokio::test]
async fn test_delete_fixed_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/fixed/subscriptions/123/databases/456"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-delete-fixed-db",
            "commandType": "DELETE_FIXED_DATABASE",
            "status": "processing",
            "description": "Deleting fixed database"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.delete_by_id(123, 456).await.unwrap();

    assert_eq!(result.task_id, Some("task-delete-fixed-db".to_string()));
    assert_eq!(
        result.command_type,
        Some("DELETE_FIXED_DATABASE".to_string())
    );
}

#[tokio::test]
async fn test_backup_fixed_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fixed/subscriptions/123/databases/456/backup"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-backup-fixed-db",
            "commandType": "BACKUP_FIXED_DATABASE",
            "status": "processing"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::FixedDatabaseBackupRequest {
        subscription_id: None,
        database_id: None,
        adhoc_backup_path: None,
        command_type: None,
    };
    let result = handler.backup(123, 456, &request).await.unwrap();

    assert_eq!(result.task_id, Some("task-backup-fixed-db".to_string()));
}

#[tokio::test]
async fn test_import_fixed_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fixed/subscriptions/123/databases/456/import"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-import-fixed-db",
            "commandType": "IMPORT_FIXED_DATABASE",
            "status": "processing",
            "description": "Importing data into fixed database"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::FixedDatabaseImportRequest {
        source_type: "s3".to_string(),
        import_from_uri: vec!["s3://my-bucket/backup.rdb".to_string()],
        subscription_id: None,
        database_id: None,
        command_type: None,
    };

    let result = handler.import(123, 456, &request).await.unwrap();
    assert_eq!(result.task_id, Some("task-import-fixed-db".to_string()));
}

#[tokio::test]
async fn test_tag_operations() {
    let mock_server = MockServer::start().await;

    // Test create tag
    Mock::given(method("POST"))
        .and(path("/fixed/subscriptions/123/databases/456/tags"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "key": "environment",
            "value": "production"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::DatabaseTagCreateRequest {
        key: "environment".to_string(),
        value: "production".to_string(),
        subscription_id: None,
        database_id: None,
        command_type: None,
    };

    let result = handler.create_tag(123, 456, &request).await.unwrap();
    assert!(result.key.is_some());
}

#[tokio::test]
async fn test_error_handling_401() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fixed/subscriptions/123/databases"))
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

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.list(123, None, None).await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::AuthenticationFailed { .. }) => {}
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[tokio::test]
async fn test_error_handling_403() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/fixed/subscriptions/123/databases/456"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden: Insufficient permissions"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.delete_by_id(123, 456).await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::Forbidden { .. }) => {}
        _ => panic!("Expected Forbidden error"),
    }
}

#[tokio::test]
async fn test_error_handling_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fixed/subscriptions/999/databases/999"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Database not found"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.get_by_id(999, 999).await;

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

    Mock::given(method("POST"))
        .and(path("/fixed/subscriptions/123/databases"))
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

    let handler = FixedDatabaseHandler::new(client);
    let request = redis_cloud::fixed_databases::FixedDatabaseCreateRequest {
        name: "test-database".to_string(),
        subscription_id: None,
        protocol: None,
        memory_limit_in_gb: None,
        dataset_size_in_gb: None,
        support_oss_cluster_api: None,
        redis_version: None,
        resp_version: None,
        use_external_endpoint_for_oss_cluster_api: None,
        enable_database_clustering: None,
        number_of_shards: None,
        data_persistence: None,
        data_eviction_policy: None,
        replication: None,
        periodic_backup_path: None,
        source_ips: None,
        regex_rules: None,
        replica_of: None,
        replica: None,
        client_ssl_certificate: None,
        client_tls_certificates: None,
        enable_tls: None,
        password: None,
        alerts: None,
        modules: None,
        command_type: None,
    };
    let result = handler.create(123, &request).await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::InternalServerError { .. }) => {}
        _ => panic!("Expected InternalServerError error"),
    }
}

#[tokio::test]
async fn test_get_fixed_database_traffic() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fixed/subscriptions/123/databases/456/traffic"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bdbId": 456,
            "trafficStatus": "active",
            "canResume": false,
            "resumeInProgress": false
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.get_traffic(123, 456).await.unwrap();

    assert_eq!(result.bdb_id, Some(456));
    assert_eq!(result.traffic_status.as_deref(), Some("active"));
    assert_eq!(result.can_resume, Some(false));
}

#[tokio::test]
async fn test_resume_fixed_database_traffic() {
    let mock_server = MockServer::start().await;

    // The resume endpoint returns 204 No Content (empty body).
    Mock::given(method("POST"))
        .and(path(
            "/fixed/subscriptions/123/databases/456/traffic/resume",
        ))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    // Empty 204 body must deserialize cleanly into `()`.
    handler.resume_traffic(123, 456).await.unwrap();
}

// Regression for #119: the live API returns `modules[].parameters` as a JSON
// array (`[]` when empty, objects otherwise), but the field used to be typed
// as a map, so real database reads failed to deserialize. This mirrors the
// real `GET /fixed/subscriptions/{id}/databases` response shape — including the
// extra module metadata and nested `security`/`clustering`/`backup` objects the
// API actually sends — and asserts the call succeeds.
#[tokio::test]
async fn test_fixed_database_modules_parameters_array_deserializes() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/fixed/subscriptions/123/databases"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "accountId": 456,
            "subscription": {
                "subscriptionId": 123,
                "numberOfDatabases": 1,
                "databases": [
                    {
                        "databaseId": 13926356,
                        "name": "with-modules",
                        "protocol": "stack",
                        "provider": "AWS",
                        "region": "us-east-1",
                        "redisVersion": "8.2",
                        "status": "active",
                        "planMemoryLimit": 12.0,
                        "memoryLimitMeasurementUnit": "MB",
                        "memoryStorage": "ram",
                        "networkMonthlyUsageInByte": 1.83615528E+10,
                        "security": {
                            "defaultUserEnabled": false,
                            "enableTls": false,
                            "sourceIps": ["0.0.0.0/0"]
                        },
                        "clustering": {
                            "enabled": false,
                            "regexRules": [{ "ordinal": 0, "pattern": ".*" }],
                            "hashingPolicy": "standard"
                        },
                        "modules": [
                            {
                                "id": 7778767,
                                "name": "RedisTimeSeries",
                                "capabilityName": "Time series",
                                "version": "8.2.9",
                                "description": "Time-Series data structure for redis",
                                "parameters": []
                            },
                            {
                                "id": 8131105,
                                "name": "RedisBloom",
                                "version": "8.2.13",
                                "parameters": [{ "name": "error_rate", "value": "0.01" }]
                            }
                        ],
                        "backup": { "remoteBackupEnabled": false }
                    }
                ],
                "links": []
            },
            "links": []
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = FixedDatabaseHandler::new(client);
    let result = handler.list(123, None, None).await.unwrap();

    let db = result
        .subscription
        .and_then(|s| s.databases.into_iter().next())
        .expect("response should include the database");
    let modules = db.modules.expect("modules should deserialize");
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].name, "RedisTimeSeries");
    // Empty array parameters.
    assert_eq!(modules[0].parameters, Some(json!([])));
    // Populated array parameters round-trip as a JSON array.
    assert_eq!(
        modules[1].parameters,
        Some(json!([{ "name": "error_rate", "value": "0.01" }]))
    );

    // #121: the nested security/clustering/backup objects are now captured.
    let security = db.security.expect("security should deserialize (see #121)");
    assert_eq!(security.source_ips, Some(vec!["0.0.0.0/0".to_string()]));
    assert_eq!(security.enable_tls, Some(false));
    let clustering = db.clustering.expect("clustering should deserialize");
    assert_eq!(clustering.enabled, Some(false));
    assert_eq!(clustering.regex_rules.as_ref().map(Vec::len), Some(1));
}
