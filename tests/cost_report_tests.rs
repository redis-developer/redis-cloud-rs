//! Integration tests for `CostReportHandler`.
//!
//! Closes #79: the cost-report handler is publicly exported and on the OpenAPI
//! spec but previously had only crate-internal unit tests covering the request
//! builder. These tests exercise the wire path for both `generate_cost_report`
//! (POST /cost-report) and `download_cost_report` (GET /cost-report/{id}).

use redis_cloud::cost_report::CostReportCreateRequest;
use redis_cloud::{CloudClient, CostReportFormat, CostReportHandler, SubscriptionType};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client(uri: String) -> CloudClient {
    CloudClient::builder()
        .api_key("test-key".to_string())
        .api_secret("test-secret".to_string())
        .base_url(uri)
        .build()
        .unwrap()
}

fn simple_request() -> CostReportCreateRequest {
    CostReportCreateRequest::builder()
        .start_date("2025-01-01")
        .end_date("2025-01-31")
        .format(CostReportFormat::Csv)
        .build()
        .expect("builder should succeed with required fields")
}

#[tokio::test]
async fn test_generate_cost_report_happy_path() {
    let mock_server = MockServer::start().await;
    let request = simple_request();

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .and(body_json(&request))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "cost-report-task-1",
            "commandType": "GENERATE_COST_REPORT",
            "status": "processing",
            "description": "Generating cost report",
            "timestamp": "2025-02-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let task = handler.generate_cost_report(request).await.unwrap();

    assert_eq!(task.task_id, Some("cost-report-task-1".to_string()));
    assert_eq!(task.status, Some("processing".to_string()));
    assert_eq!(task.command_type, Some("GENERATE_COST_REPORT".to_string()));
}

#[tokio::test]
async fn test_generate_cost_report_with_full_filters() {
    let mock_server = MockServer::start().await;

    // Build a richer request to exercise more of the wire shape.
    let request = CostReportCreateRequest::builder()
        .start_date("2025-01-01")
        .end_date("2025-01-31")
        .format(CostReportFormat::Json)
        .subscription_ids(vec![1, 2, 3])
        .database_ids(vec![10, 20])
        .subscription_type(SubscriptionType::Pro)
        .regions(vec!["us-east-1".to_string(), "eu-west-1".to_string()])
        .tag("env", "prod")
        .build()
        .unwrap();

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .and(body_json(&request))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "cost-report-task-2",
            "status": "received"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let task = handler.generate_cost_report(request).await.unwrap();

    assert_eq!(task.task_id, Some("cost-report-task-2".to_string()));
}

#[tokio::test]
async fn test_generate_cost_report_raw_returns_json_value() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "taskId": "raw-task",
            "extra": "fields-preserved"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let value = handler
        .generate_cost_report_raw(simple_request())
        .await
        .unwrap();

    assert_eq!(value["taskId"], "raw-task");
    assert_eq!(value["extra"], "fields-preserved");
}

#[tokio::test]
async fn test_generate_cost_report_400_bad_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "Date range exceeds 40 days"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let err = handler
        .generate_cost_report(simple_request())
        .await
        .unwrap_err();

    match err {
        redis_cloud::CloudError::BadRequest { .. } => {}
        other => panic!("expected BadRequest, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_generate_cost_report_401_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Invalid API credentials"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let err = handler
        .generate_cost_report(simple_request())
        .await
        .unwrap_err();

    match err {
        redis_cloud::CloudError::AuthenticationFailed { .. } => {}
        other => panic!("expected AuthenticationFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_generate_cost_report_500_internal_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/cost-report"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let err = handler
        .generate_cost_report(simple_request())
        .await
        .unwrap_err();

    match err {
        redis_cloud::CloudError::InternalServerError { .. } => {}
        other => panic!("expected InternalServerError, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_download_cost_report_happy_path() {
    let mock_server = MockServer::start().await;

    let csv_body = b"BilledCost,EffectiveCost,ListCost\n1.00,1.00,1.00\n";

    Mock::given(method("GET"))
        .and(path("/cost-report/cost-report-12345"))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(csv_body.to_vec()))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let bytes = handler
        .download_cost_report("cost-report-12345")
        .await
        .unwrap();

    assert_eq!(bytes, csv_body);
}

#[tokio::test]
async fn test_download_cost_report_404_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/cost-report/missing-report"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Cost report not found"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let err = handler
        .download_cost_report("missing-report")
        .await
        .unwrap_err();

    match err {
        redis_cloud::CloudError::NotFound { .. } => {}
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_download_cost_report_503_service_unavailable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/cost-report/transient-report"))
        .respond_with(ResponseTemplate::new(503).set_body_json(json!({
            "error": "Service temporarily unavailable"
        })))
        .mount(&mock_server)
        .await;

    let handler = CostReportHandler::new(test_client(mock_server.uri()));
    let err = handler
        .download_cost_report("transient-report")
        .await
        .unwrap_err();

    match err {
        redis_cloud::CloudError::ServiceUnavailable { .. } => {}
        other => panic!("expected ServiceUnavailable, got: {other:?}"),
    }
}
