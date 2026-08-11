use redis_cloud::{CloudClient, CloudError};
use serde_json::Value;
use std::error::Error as _;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn public_error_variants_have_stable_display_and_no_source_chain() {
    let cases = [
        (
            CloudError::Request("request failed".to_string()),
            "HTTP request failed: request failed",
        ),
        (
            CloudError::BadRequest {
                message: "bad input".to_string(),
            },
            "Bad Request (400): bad input",
        ),
        (
            CloudError::AuthenticationFailed {
                message: "bad credentials".to_string(),
            },
            "Authentication failed (401): bad credentials",
        ),
        (
            CloudError::Forbidden {
                message: "access denied".to_string(),
            },
            "Forbidden (403): access denied",
        ),
        (
            CloudError::NotFound {
                message: "missing".to_string(),
            },
            "Not Found (404): missing",
        ),
        (
            CloudError::PreconditionFailed,
            "Precondition Failed (412): Feature flag for this flow is off",
        ),
        (
            CloudError::RateLimited {
                message: "slow down".to_string(),
            },
            "Rate Limited (429): slow down",
        ),
        (
            CloudError::InternalServerError {
                message: "server broke".to_string(),
            },
            "Internal Server Error (500): server broke",
        ),
        (
            CloudError::ServiceUnavailable {
                message: "try later".to_string(),
            },
            "Service Unavailable (503): try later",
        ),
        (
            CloudError::ApiError {
                code: 418,
                message: "teapot".to_string(),
            },
            "API error (418): teapot",
        ),
        (
            CloudError::ConnectionError("dns failure".to_string()),
            "Connection error: dns failure",
        ),
        (
            CloudError::JsonError("invalid json".to_string()),
            "JSON error: invalid json",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(
            error.source().is_none(),
            "string-backed CloudError variants do not retain source errors"
        );
    }
}

#[test]
fn classifier_helpers_cover_positive_and_negative_cases() {
    let retryable = [
        CloudError::RateLimited {
            message: "slow down".to_string(),
        },
        CloudError::ServiceUnavailable {
            message: "try later".to_string(),
        },
        CloudError::Request("socket closed".to_string()),
        CloudError::ConnectionError("dns failed".to_string()),
    ];
    assert!(retryable.iter().all(CloudError::is_retryable));
    assert!(
        !CloudError::BadRequest {
            message: "bad input".to_string()
        }
        .is_retryable()
    );

    assert!(
        CloudError::NotFound {
            message: "missing".to_string()
        }
        .is_not_found()
    );
    assert!(!CloudError::PreconditionFailed.is_not_found());

    assert!(
        CloudError::AuthenticationFailed {
            message: "bad credentials".to_string()
        }
        .is_unauthorized()
    );
    assert!(
        CloudError::Forbidden {
            message: "access denied".to_string()
        }
        .is_unauthorized()
    );
    assert!(!CloudError::PreconditionFailed.is_unauthorized());

    assert!(
        CloudError::InternalServerError {
            message: "server broke".to_string()
        }
        .is_server_error()
    );
    assert!(
        CloudError::ServiceUnavailable {
            message: "try later".to_string()
        }
        .is_server_error()
    );
    assert!(!CloudError::PreconditionFailed.is_server_error());

    assert!(CloudError::Request("request TIMEOUT".to_string()).is_timeout());
    assert!(CloudError::ConnectionError("timeout".to_string()).is_timeout());
    assert!(!CloudError::Request("connection reset".to_string()).is_timeout());
    assert!(!CloudError::PreconditionFailed.is_timeout());

    assert!(
        CloudError::RateLimited {
            message: "slow down".to_string()
        }
        .is_rate_limited()
    );
    assert!(!CloudError::PreconditionFailed.is_rate_limited());
    assert!(CloudError::PreconditionFailed.is_conflict());
    assert!(
        !CloudError::BadRequest {
            message: "bad input".to_string()
        }
        .is_conflict()
    );
    assert!(
        CloudError::BadRequest {
            message: "bad input".to_string()
        }
        .is_bad_request()
    );
    assert!(!CloudError::PreconditionFailed.is_bad_request());
}

#[test]
fn serde_json_conversion_preserves_the_message() {
    let source = serde_json::from_str::<Value>("{").expect_err("JSON should be malformed");
    let expected = source.to_string();
    let error = CloudError::from(source);

    match error {
        CloudError::JsonError(message) => assert_eq!(message, expected),
        other => panic!("expected JsonError, got {other:?}"),
    }
}

#[test]
fn reqwest_conversion_preserves_the_message() {
    let source = reqwest::Client::new()
        .get("://invalid-url")
        .build()
        .expect_err("request URL should be invalid");
    let expected = source.to_string();
    let error = CloudError::from(source);

    match error {
        CloudError::Request(message) => assert_eq!(message, expected),
        other => panic!("expected Request, got {other:?}"),
    }
}

#[derive(Clone, Copy)]
enum ExpectedError {
    BadRequest,
    AuthenticationFailed,
    Forbidden,
    NotFound,
    PreconditionFailed,
    RateLimited,
    InternalServerError,
    ServiceUnavailable,
    ApiError,
}

#[tokio::test]
async fn http_statuses_map_to_public_error_variants() {
    let server = MockServer::start().await;
    let cases = [
        (400, ExpectedError::BadRequest),
        (401, ExpectedError::AuthenticationFailed),
        (403, ExpectedError::Forbidden),
        (404, ExpectedError::NotFound),
        (412, ExpectedError::PreconditionFailed),
        (429, ExpectedError::RateLimited),
        (500, ExpectedError::InternalServerError),
        (503, ExpectedError::ServiceUnavailable),
        (418, ExpectedError::ApiError),
    ];

    for (status, _) in cases {
        Mock::given(method("GET"))
            .and(path(format!("/status/{status}")))
            .respond_with(
                ResponseTemplate::new(status).set_body_string(format!("response {status}")),
            )
            .mount(&server)
            .await;
    }

    let client = CloudClient::builder()
        .api_key("test-key")
        .api_secret("test-secret")
        .base_url(server.uri())
        .build()
        .expect("test client should build");

    for (status, expected) in cases {
        let error = client
            .get_raw(&format!("/status/{status}"))
            .await
            .expect_err("non-success status should return an error");
        let expected_message = format!("response {status}");

        match (expected, error) {
            (ExpectedError::BadRequest, CloudError::BadRequest { message })
            | (ExpectedError::AuthenticationFailed, CloudError::AuthenticationFailed { message })
            | (ExpectedError::Forbidden, CloudError::Forbidden { message })
            | (ExpectedError::NotFound, CloudError::NotFound { message })
            | (ExpectedError::RateLimited, CloudError::RateLimited { message })
            | (ExpectedError::InternalServerError, CloudError::InternalServerError { message })
            | (ExpectedError::ServiceUnavailable, CloudError::ServiceUnavailable { message }) => {
                assert_eq!(message, expected_message)
            }
            (ExpectedError::PreconditionFailed, CloudError::PreconditionFailed) => {}
            (ExpectedError::ApiError, CloudError::ApiError { code, message }) => {
                assert_eq!(code, status);
                assert_eq!(message, expected_message);
            }
            (_, other) => panic!("status {status} mapped to unexpected error {other:?}"),
        }
    }
}
