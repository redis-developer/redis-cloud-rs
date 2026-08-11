use redis_cloud::CloudClient;
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

async fn mount(server: &MockServer, verb: &str, request_path: &str, body: Option<Value>) {
    let mock = Mock::given(method(verb))
        .and(path(request_path))
        .and(header("x-api-key", "test-key"))
        .and(header("x-api-secret-key", "test-secret"));
    let mock = if let Some(body) = body {
        mock.and(body_json(body))
    } else {
        mock
    };
    mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .mount(server)
        .await;
}

#[tokio::test]
async fn test_workspace_root_operations() {
    let server = MockServer::start().await;
    let body = json!({"pipeline": "example"});

    mount(&server, "GET", "/data-integration-workspaces", None).await;
    for verb in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
        mount(
            &server,
            verb,
            "/subscriptions/123/data-integration-workspace",
            (verb != "GET").then(|| body.clone()),
        )
        .await;
    }

    let handler = client(&server).data_integration();
    assert_eq!(handler.list_workspaces().await.unwrap()["ok"], true);
    assert_eq!(handler.get_workspace(123).await.unwrap()["ok"], true);
    assert_eq!(
        handler.post_workspace(123, body.clone()).await.unwrap()["ok"],
        true
    );
    assert_eq!(
        handler.put_workspace(123, body.clone()).await.unwrap()["ok"],
        true
    );
    assert_eq!(
        handler.patch_workspace(123, body.clone()).await.unwrap()["ok"],
        true
    );
    assert_eq!(
        handler.delete_workspace(123, body).await.unwrap()["ok"],
        true
    );
}

#[tokio::test]
async fn test_workspace_proxy_operations() {
    let server = MockServer::start().await;
    let body = json!({"enabled": true});
    let request_path = "/subscriptions/123/data-integration-workspace/pipelines/one";

    for verb in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
        mount(
            &server,
            verb,
            request_path,
            (verb != "GET").then(|| body.clone()),
        )
        .await;
    }

    let handler = client(&server).data_integration();
    assert_eq!(
        handler
            .get_workspace_path(123, "/pipelines/one/")
            .await
            .unwrap()["ok"],
        true
    );
    assert_eq!(
        handler
            .post_workspace_path(123, "pipelines/one", body.clone())
            .await
            .unwrap()["ok"],
        true
    );
    assert_eq!(
        handler
            .put_workspace_path(123, "pipelines/one", body.clone())
            .await
            .unwrap()["ok"],
        true
    );
    assert_eq!(
        handler
            .patch_workspace_path(123, "pipelines/one", body.clone())
            .await
            .unwrap()["ok"],
        true
    );
    assert_eq!(
        handler
            .delete_workspace_path(123, "pipelines/one", body)
            .await
            .unwrap()["ok"],
        true
    );
}
