use redis_cloud::types::TaskStatus;
use redis_cloud::{CloudClient, SubscriptionsHandler};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_all_subscriptions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "accountId": 456,
            "subscriptions": [
                {
                    "id": 123,
                    "name": "Production",
                    "status": "active",
                    "paymentMethodType": "credit-card"
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_all_subscriptions().await.unwrap();

    assert_eq!(result.account_id, Some(456));
    // The subscriptions are now first-class field
    assert!(result.subscriptions.is_some());
    let subs = result.subscriptions.unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].id, Some(123));
    assert_eq!(subs[0].name, Some("Production".to_string()));
    assert_eq!(subs[0].status, Some("active".to_string()));
}

#[tokio::test]
async fn test_create_subscription() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-create-sub",
            "commandType": "CREATE_SUBSCRIPTION",
            "status": "processing",
            "description": "Creating subscription",
            "response": {
                "resourceId": 125
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

    let handler = SubscriptionsHandler::new(client);
    // Using the SubscriptionCreateRequest with required fields
    let request = redis_cloud::subscriptions::SubscriptionCreateRequest {
        name: Some("New Subscription".to_string()),
        payment_method_id: Some(1001),
        payment_method: None,
        memory_storage: Some("ram".to_string()),
        persistent_storage_encryption_type: None,
        deployment_type: Some("single-region".to_string()),
        dry_run: None,
        cloud_providers: vec![redis_cloud::subscriptions::SubscriptionSpec {
            provider: Some("AWS".to_string()),
            cloud_account_id: Some(1001),
            regions: vec![redis_cloud::subscriptions::SubscriptionRegionSpec {
                region: "us-west-2".to_string(),
                multiple_availability_zones: None,
                preferred_availability_zones: None,
                networking: Some(
                    redis_cloud::subscriptions::SubscriptionRegionNetworkingSpec {
                        deployment_cidr: Some("10.0.0.0/20".to_string()),
                        vpc_id: None,
                        subnet_ids: None,
                        security_group_id: None,
                    },
                ),
            }],
        }],
        databases: vec![],
        redis_version: None,
        command_type: None,
    };

    let result = handler.create_subscription(&request).await.unwrap();
    assert_eq!(result.task_id, Some("task-create-sub".to_string()));
}

#[tokio::test]
async fn test_get_redis_versions() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/redis-versions"))
        .and(query_param("subscriptionId", "123"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "redisVersions": [
                {
                    "version": "7.2",
                    "isDefault": true
                },
                {
                    "version": "7.0",
                    "isDefault": false
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_redis_versions(Some(123)).await.unwrap();

    assert!(result.redis_versions.is_some());
}

#[tokio::test]
async fn test_delete_subscription_by_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/subscriptions/123"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-delete-sub",
            "commandType": "DELETE_SUBSCRIPTION",
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.delete_subscription_by_id(123).await.unwrap();

    assert_eq!(result.task_id, Some("task-delete-sub".to_string()));
}

#[tokio::test]
async fn test_get_subscription_by_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "Production",
            "status": "active",
            "paymentMethodType": "credit-card",
            "numberOfDatabases": 1,
            "cloudDetails": [{
                "provider": "AWS",
                "cloudAccountId": 1,
                "totalSizeInGb": 6.0,
                "regions": [{
                    "region": "us-east-1",
                    "networking": [{
                        "deploymentCIDR": "192.168.0.0/26",
                        "subnetId": "subnet-0example",
                        "securityGroupId": "sg-0example"
                    }]
                }],
                "resourceTags": [],
                "links": []
            }],
            "subscriptionPricing": [{
                "type": "Shards",
                "quantity": 6,
                "quantityMeasurement": "shards",
                "pricePerUnit": 0.043,
                "priceCurrency": "USD",
                "pricePeriod": "hour"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_subscription_by_id(123).await.unwrap();

    assert_eq!(result.id, Some(123));
    assert_eq!(result.name, Some("Production".to_string()));
    // #128: subscriptionPricing and the nested cloudDetails networking/tags are
    // captured from the real response shape.
    let pricing = result
        .subscription_pricing
        .expect("subscriptionPricing should deserialize (see #128)");
    assert_eq!(pricing[0].r#type.as_deref(), Some("Shards"));
    let net = result
        .cloud_details
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.regions)
        .and_then(|r| r.into_iter().next())
        .and_then(|r| r.networking)
        .and_then(|n| n.into_iter().next())
        .expect("networking should deserialize");
    assert_eq!(net.deployment_cidr.as_deref(), Some("192.168.0.0/26"));
    assert!(net.security_group_id.is_some());
}

#[tokio::test]
async fn test_update_subscription() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/subscriptions/123"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-update-sub",
            "commandType": "UPDATE_SUBSCRIPTION",
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

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::BaseSubscriptionUpdateRequest {
        subscription_id: None,
        command_type: None,
    };

    let result = handler.update_subscription(123, &request).await.unwrap();
    assert_eq!(result.task_id, Some("task-update-sub".to_string()));
}

#[tokio::test]
async fn test_get_cidr_allowlist() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/cidr"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": {
                "security": {
                    "cidrIps": ["192.168.1.0/24", "10.0.0.0/8"],
                    "defaultForNewDatabases": true
                }
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_cidr_allowlist(123).await.unwrap();

    assert!(result.response.is_some());
}

#[tokio::test]
async fn test_update_subscription_cidr_allowlist() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/subscriptions/123/cidr"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-update-cidr",
            "commandType": "UPDATE_CIDR_WHITELIST",
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

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::CidrAllowlistUpdateRequest {
        subscription_id: None,
        cidr_ips: Some(vec!["192.168.0.0/16".to_string()]),
        security_group_ids: None,
        command_type: None,
    };

    let result = handler
        .update_subscription_cidr_allowlist(123, &request)
        .await
        .unwrap();
    assert_eq!(result.task_id, Some("task-update-cidr".to_string()));
}

#[tokio::test]
async fn test_get_subscription_maintenance_windows() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/maintenance-windows"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "mode": "manual",
            "windows": [
                {
                    "startHour": 2,
                    "durationInHours": 4,
                    "days": ["Sunday", "Wednesday"]
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler
        .get_subscription_maintenance_windows(123)
        .await
        .unwrap();

    assert_eq!(result.mode, Some("manual".to_string()));
    assert!(result.windows.is_some());
}

#[tokio::test]
async fn test_update_subscription_maintenance_windows() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/subscriptions/123/maintenance-windows"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-update-maintenance",
            "commandType": "UPDATE_MAINTENANCE_WINDOWS",
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

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::SubscriptionMaintenanceWindowsSpec {
        mode: "automatic".to_string(),
        windows: None,
    };

    let result = handler
        .update_subscription_maintenance_windows(123, &request)
        .await
        .unwrap();
    assert_eq!(result.task_id, Some("task-update-maintenance".to_string()));
}

#[tokio::test]
async fn test_get_subscription_pricing() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/pricing"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "pricing": [
                {
                    "type": "Shards",
                    "typeDetails": "Standard",
                    "quantity": 2,
                    "quantityMeasurement": "shards",
                    "pricePerUnit": 0.124,
                    "priceCurrency": "USD",
                    "pricePeriod": "hour",
                    "region": "us-east-1"
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_subscription_pricing(123).await.unwrap();

    // Check that pricing data was parsed
    assert!(result.pricing.is_some());
    let pricing = result.pricing.unwrap();
    assert_eq!(pricing.len(), 1);
    assert_eq!(pricing[0].r#type, Some("Shards".to_string()));
}

#[tokio::test]
async fn test_delete_regions_from_active_active_subscription() {
    // Regression guard for #59: the AA region-delete handler must send
    // its JSON body on the wire, not drop it on the floor.
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/subscriptions/123/regions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .and(body_json(json!({
            "regions": [{"region": "us-east-1"}],
            "dryRun": false,
            "commandType": "DELETE_REGION"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-delete-region",
            "commandType": "DELETE_REGION",
            "status": "processing-in-progress"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::ActiveActiveRegionDeleteRequest {
        subscription_id: None,
        regions: Some(vec![
            redis_cloud::subscriptions::ActiveActiveRegionToDelete {
                region: Some("us-east-1".to_string()),
            },
        ]),
        dry_run: Some(false),
        command_type: Some("DELETE_REGION".to_string()),
    };

    let result = handler
        .delete_regions_from_active_active_subscription(123, &request)
        .await
        .unwrap();

    assert_eq!(result.task_id.as_deref(), Some("task-delete-region"));
    assert_eq!(result.command_type.as_deref(), Some("DELETE_REGION"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingInProgress));
}

#[tokio::test]
async fn test_get_regions_from_active_active_subscription() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/123/regions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "regions": [
                {
                    "region": "us-east-1",
                    "provider": "AWS",
                    "status": "active"
                },
                {
                    "region": "eu-west-1",
                    "provider": "AWS",
                    "status": "active"
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler
        .get_regions_from_active_active_subscription(123)
        .await
        .unwrap();

    // Verify the response was successfully parsed
    // Note: regions data would need a typed field to be accessible
    assert!(result.subscription_id.is_none()); // No subscription_id in the mock response
}

#[tokio::test]
async fn test_add_new_region_to_active_active_subscription() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/subscriptions/123/regions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-add-region",
            "commandType": "ADD_REGION",
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

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::ActiveActiveRegionCreateRequest {
        subscription_id: None,
        region: Some("ap-southeast-1".to_string()),
        vpc_id: None,
        deployment_cidr: "10.1.0.0/20".to_string(),
        dry_run: None,
        databases: None,
        resp_version: None,
        customer_managed_key_resource_name: None,
        command_type: None,
    };

    let result = handler
        .add_new_region_to_active_active_subscription(123, &request)
        .await
        .unwrap();
    assert_eq!(result.task_id, Some("task-add-region".to_string()));
}

#[tokio::test]
async fn test_error_handling_401() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions"))
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

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_all_subscriptions().await;

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
        .and(path("/subscriptions/999"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Subscription not found"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = SubscriptionsHandler::new(client);
    let result = handler.get_subscription_by_id(999).await;

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
        .and(path("/subscriptions"))
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

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::SubscriptionCreateRequest {
        name: Some("Test Subscription".to_string()),
        payment_method_id: None,
        payment_method: None,
        memory_storage: None,
        persistent_storage_encryption_type: None,
        deployment_type: Some("single-region".to_string()),
        dry_run: None,
        cloud_providers: vec![],
        databases: vec![],
        redis_version: None,
        command_type: None,
    };

    let result = handler.create_subscription(&request).await;

    assert!(result.is_err());
    match result {
        Err(redis_cloud::CloudError::InternalServerError { .. }) => {}
        _ => panic!("Expected InternalServerError error"),
    }
}

#[tokio::test]
async fn test_update_resource_tags() {
    let mock_server = MockServer::start().await;

    // The request must serialize tags under the `resourceTags` wire key.
    let expected_body = json!({
        "resourceTags": [
            { "key": "env", "value": "prod" },
            { "key": "team", "value": "data" }
        ]
    });

    Mock::given(method("PUT"))
        .and(path("/subscriptions/123/resource-tags"))
        .and(body_json(&expected_body))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "task-update-tags",
            "commandType": "SUBSCRIPTION_RESOURCE_TAGS_UPDATE",
            "status": "processing-in-progress",
            "description": "Updating resource tags"
        })))
        .mount(&mock_server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(mock_server.uri())
        .build()
        .unwrap();

    let handler = SubscriptionsHandler::new(client);
    let request = redis_cloud::subscriptions::SubscriptionResourceTagsUpdateRequest {
        subscription_id: None,
        resource_tags: vec![
            redis_cloud::types::Tag {
                key: "env".to_string(),
                value: "prod".to_string(),
                command_type: None,
            },
            redis_cloud::types::Tag {
                key: "team".to_string(),
                value: "data".to_string(),
                command_type: None,
            },
        ],
        command_type: None,
    };

    let result = handler.update_resource_tags(123, &request).await.unwrap();
    assert_eq!(result.task_id.as_deref(), Some("task-update-tags"));
    assert_eq!(result.status, Some(TaskStatus::ProcessingInProgress));
}

// Helper: build a Cloud client wired to the given mock server URI.
fn test_client(uri: String) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(uri)
        .build()
        .unwrap()
}

// Alias round-trip: `list()` delegates to `get_all_subscriptions()`.
#[tokio::test]
async fn test_subscriptions_list_alias() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "accountId": 5,
            "subscriptions": []
        })))
        .mount(&mock_server)
        .await;

    let handler = SubscriptionsHandler::new(test_client(mock_server.uri()));
    let result = handler.list().await.unwrap();

    assert_eq!(result.account_id, Some(5));
}

// Alias round-trip: `get(id)` delegates to `get_subscription_by_id()`.
#[tokio::test]
async fn test_subscription_get_alias() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/subscriptions/200"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 200,
            "name": "alias-sub"
        })))
        .mount(&mock_server)
        .await;

    let handler = SubscriptionsHandler::new(test_client(mock_server.uri()));
    let result = handler.get(200).await.unwrap();

    assert_eq!(result.id, Some(200));
}

// Alias round-trip: `delete(id)` delegates to `delete_subscription_by_id()`.
#[tokio::test]
async fn test_subscription_delete_alias() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/subscriptions/201"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "taskId": "task-del-sub",
            "status": "processing-completed"
        })))
        .mount(&mock_server)
        .await;

    let handler = SubscriptionsHandler::new(test_client(mock_server.uri()));
    let result = handler.delete(201).await.unwrap();

    assert_eq!(result.task_id, Some("task-del-sub".to_string()));
}
