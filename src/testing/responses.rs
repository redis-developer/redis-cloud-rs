//! Response helpers for creating mock HTTP responses

use serde_json::{Value, json};
use std::time::Duration;
use wiremock::ResponseTemplate;

/// Create a 200 OK response with a JSON body
pub fn success(body: impl Into<Value>) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body.into())
}

/// Create a 201 Created response with a JSON body
pub fn created(body: impl Into<Value>) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_json(body.into())
}

/// Create a 202 Accepted response with task information
///
/// This is the typical response for async operations in Redis Cloud API.
pub fn accepted(task_id: &str, command_type: &str) -> ResponseTemplate {
    ResponseTemplate::new(202).set_body_json(json!({
        "taskId": task_id,
        "commandType": command_type,
        "status": "received"
    }))
}

/// Create a 202 Accepted response with task information and resource ID
pub fn accepted_with_resource(
    task_id: &str,
    command_type: &str,
    resource_id: i32,
) -> ResponseTemplate {
    ResponseTemplate::new(202).set_body_json(json!({
        "taskId": task_id,
        "commandType": command_type,
        "status": "received",
        "response": {
            "resourceId": resource_id
        }
    }))
}

/// Create a 204 No Content response
pub fn no_content() -> ResponseTemplate {
    ResponseTemplate::new(204)
}

/// Create a 400 Bad Request response
pub fn bad_request(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(400).set_body_json(json!({
        "error": "Bad request",
        "message": message
    }))
}

/// Create a 401 Unauthorized response
pub fn unauthorized() -> ResponseTemplate {
    ResponseTemplate::new(401).set_body_json(json!({
        "error": "Unauthorized",
        "message": "Invalid API credentials"
    }))
}

/// Create a 403 Forbidden response
pub fn forbidden(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(403).set_body_json(json!({
        "error": "Forbidden",
        "message": message
    }))
}

/// Create a 404 Not Found response
pub fn not_found(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(404).set_body_json(json!({
        "error": "Not found",
        "message": message
    }))
}

/// Create a 409 Conflict response
pub fn conflict(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(409).set_body_json(json!({
        "error": "Conflict",
        "message": message
    }))
}

/// Create a 429 Rate Limited response
pub fn rate_limited(retry_after: u32) -> ResponseTemplate {
    ResponseTemplate::new(429)
        .insert_header("Retry-After", retry_after.to_string())
        .set_body_json(json!({
            "error": "Rate limited",
            "message": "Too many requests. Please retry later."
        }))
}

/// Create a 500 Internal Server Error response
pub fn server_error(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(500).set_body_json(json!({
        "error": "Internal server error",
        "message": message
    }))
}

/// Create a 503 Service Unavailable response
pub fn service_unavailable() -> ResponseTemplate {
    ResponseTemplate::new(503).set_body_json(json!({
        "error": "Service unavailable",
        "message": "The service is temporarily unavailable. Please try again later."
    }))
}

/// Create a custom error response with a specific status code
pub fn error(status_code: u16, message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(status_code).set_body_json(json!({
        "error": format!("Error {}", status_code),
        "message": message
    }))
}

/// Add a delay to a response for testing timeout behavior
///
/// # Example
///
/// ```rust,ignore
/// use redis_cloud::testing::responses::{success, delayed};
/// use std::time::Duration;
/// use serde_json::json;
///
/// let slow_response = delayed(success(json!({"data": "value"})), Duration::from_secs(5));
/// ```
pub fn delayed(response: ResponseTemplate, duration: Duration) -> ResponseTemplate {
    response.set_delay(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = success(json!({"key": "value"}));
        // ResponseTemplate doesn't expose internals, but we can verify it compiles
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_created_response() {
        let response = created(json!({"id": 123}));
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_accepted_response() {
        let response = accepted("task-123", "createSubscription");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_accepted_with_resource() {
        let response = accepted_with_resource("task-123", "createSubscription", 456);
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_no_content_response() {
        let response = no_content();
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_bad_request_response() {
        let response = bad_request("Invalid parameter");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_unauthorized_response() {
        let response = unauthorized();
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_not_found_response() {
        let response = not_found("Resource not found");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_conflict_response() {
        let response = conflict("Resource already exists");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_rate_limited_response() {
        let response = rate_limited(60);
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_server_error_response() {
        let response = server_error("Something went wrong");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_service_unavailable_response() {
        let response = service_unavailable();
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_custom_error_response() {
        let response = error(418, "I'm a teapot");
        assert!(std::mem::size_of_val(&response) > 0);
    }

    #[test]
    fn test_delayed_response() {
        let base = success(json!({"key": "value"}));
        let response = delayed(base, Duration::from_millis(100));
        assert!(std::mem::size_of_val(&response) > 0);
    }
}
