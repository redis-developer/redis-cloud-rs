//! Shared types for the Redis Cloud REST API.
//!
//! Types in this module are referenced from many endpoints and modules in the
//! crate. They model the cross-cutting concepts the API itself reuses:
//!
//! - **Task tracking** — [`TaskStateUpdate`], [`TaskStatus`],
//!   [`TasksStateUpdate`], and [`ProcessorResponse`] surface the async
//!   workflow the Cloud API uses for most mutating operations: a request
//!   returns a task ID; the caller polls `GET /tasks/{taskId}` until
//!   completion.
//! - **HATEOAS navigation** — [`Link`] and [`Links`] capture the `links`
//!   arrays the API returns alongside most resources.
//! - **Tagging** — [`CloudTag`] and [`CloudTags`] model the key/value tag
//!   pairs used for billing and resource filtering.
//! - **Common enums** — [`CloudProvider`], [`Protocol`],
//!   [`DataPersistence`], [`SubscriptionStatus`], [`DatabaseStatus`] are
//!   shared across the database, subscription, and connectivity modules.
//! - **Generic wrappers** — [`PaginatedResponse`], [`EmptyResponse`],
//!   [`ErrorResponse`] for cross-cutting response shapes.
//!
//! Consolidating these shared models into a single canonical location
//! is the goal of #64 (currently several endpoint modules redefine their
//! own copies of `TaskStateUpdate` and tag types).

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Task Types (Most common - appears in 37 endpoints)
// ============================================================================

/// State of an asynchronous Redis Cloud task.
///
/// Returned by most mutating operations in the API. The `task_id` can be
/// polled via [`TasksHandler::get_task_by_id`] until `status` reaches a
/// terminal value (see [`TaskStatus`]).
///
/// [`TasksHandler::get_task_by_id`]: crate::tasks::TasksHandler::get_task_by_id
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStateUpdate {
    /// UUID of the task. Use with `TasksHandler::get_task_by_id` to poll.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Type of command being executed (e.g. `"CREATE_DATABASE"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    /// Current task status. See [`TaskStatus`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,

    /// Human-readable description of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timestamp of the last task update (ISO-8601 string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Result of the task once it has completed. See [`ProcessorResponse`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ProcessorResponse>,

    /// HATEOAS links for related resources (e.g. the resource the task
    /// produced).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Terminal and intermediate states of a Redis Cloud task.
///
/// Status values use kebab-case on the wire (e.g. `"processing-completed"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    /// Task has been created but not yet picked up by the processor.
    Initialized,
    /// Task has been received by the processor.
    Received,
    /// Task is currently executing. Poll again later.
    ProcessingInProgress,
    /// Task completed successfully. See `response.resource_id` for the
    /// resulting resource ID.
    ProcessingCompleted,
    /// Task failed during processing. See `response.error` for details.
    ProcessingError,
}

/// Result payload included on a completed [`TaskStateUpdate`].
///
/// On success, `resource_id` points at the created/modified resource.
/// On failure, `error` carries a message and the optional `additional_info`
/// may carry more context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessorResponse {
    /// ID of the primary resource created or modified by the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<i32>,

    /// ID of an additional resource, if the operation produced one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_resource_id: Option<i32>,

    /// Free-form resource details. Shape varies by operation type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<std::collections::HashMap<String, Value>>,

    /// Error message, populated only when the task failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Free-form additional context attached by the processor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<String>,
}

/// Coarse subset of Redis Cloud's processor error codes.
///
/// The full server-side enumeration has 600+ values; this type captures the
/// few that callers typically branch on. Unknown codes fall through to
/// [`Self::Other`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorError {
    /// Authentication failed for the requested operation.
    #[serde(rename = "UNAUTHORIZED")]
    Unauthorized,
    /// The target resource was not found.
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    /// The request was malformed or missing required fields.
    #[serde(rename = "BAD_REQUEST")]
    BadRequest,
    /// Unspecified processor failure.
    #[serde(rename = "GENERAL_ERROR")]
    GeneralError,
    /// Catch-all for codes not enumerated here.
    #[serde(other)]
    Other,
}

/// Wrapper for the `GET /tasks` response (`{tasks: [...]}`).
///
/// See [`TasksHandler::get_all_tasks`] for the public ergonomic API that
/// unwraps to `Vec<TaskStateUpdate>`.
///
/// [`TasksHandler::get_all_tasks`]: crate::tasks::TasksHandler::get_all_tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksStateUpdate {
    /// Tasks returned by the server.
    pub tasks: Vec<TaskStateUpdate>,
}

// ============================================================================
// Tag Types (Used in database and subscription endpoints)
// ============================================================================

/// Key-value tag attached to a database or subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTag {
    /// Tag name.
    pub key: String,
    /// Tag value.
    pub value: String,
}

/// Collection wrapper for a list of [`CloudTag`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTags {
    /// Tags in the collection.
    pub tags: Vec<CloudTag>,
}

// ============================================================================
// Common Response Types
// ============================================================================

/// Generic paginated-response wrapper.
///
/// The `data` field is flattened so the inner shape's fields appear at the
/// same JSON level as the pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Inner page payload.
    #[serde(flatten)]
    pub data: T,

    /// Zero-based offset of the first item in this page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    /// Maximum number of items returned per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Total number of items across all pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
}

/// HATEOAS link to a related API resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Relationship name (e.g. `"self"`, `"databases"`).
    pub rel: String,
    /// Absolute URL the link points to.
    pub href: String,

    /// HTTP method to use when following the link, if specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Media type the linked resource will return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// Collection wrapper for a list of [`Link`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    /// Links in the collection.
    pub links: Vec<Link>,
}

// ============================================================================
// Common Enums used across multiple endpoints
// ============================================================================

/// Cloud provider hosting a Redis Cloud resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CloudProvider {
    /// Amazon Web Services.
    Aws,
    /// Google Cloud Platform.
    Gcp,
    /// Microsoft Azure.
    Azure,
}

/// Redis protocol exposed by a database endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// Standard Redis protocol.
    Redis,
    /// Memcached-compatible protocol.
    Memcached,
    /// Redis Stack — Redis with bundled modules (JSON, Search, etc.).
    Stack,
}

/// Database persistence policy.
///
/// Variant wire names follow the Redis Cloud convention (e.g.
/// `"aof-every-1-sec"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataPersistence {
    /// No persistence; data lives only in memory.
    #[serde(rename = "none")]
    None,
    /// Append-only file flushed every second.
    #[serde(rename = "aof-every-1-sec")]
    AofEvery1Sec,
    /// Append-only file flushed on every write.
    #[serde(rename = "aof-every-write")]
    AofEveryWrite,
    /// RDB snapshot every hour.
    #[serde(rename = "snapshot-every-1-hour")]
    SnapshotEvery1Hour,
    /// RDB snapshot every six hours.
    #[serde(rename = "snapshot-every-6-hours")]
    SnapshotEvery6Hours,
    /// RDB snapshot every twelve hours.
    #[serde(rename = "snapshot-every-12-hours")]
    SnapshotEvery12Hours,
}

/// Lifecycle status of a Pro subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    /// Subscription is being created; not yet ready for use.
    Pending,
    /// Subscription is operational.
    Active,
    /// Subscription is in the process of being deleted.
    Deleting,
    /// Subscription is in an error state and may require operator action.
    Error,
}

/// Lifecycle status of a database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseStatus {
    /// Database is being created; not yet ready for connections.
    Pending,
    /// Database is operational.
    Active,
    /// Database is operational but has pending configuration changes.
    ActiveChangePending,
    /// Database is being populated from an import source.
    ImportPending,
    /// Database is queued for deletion.
    DeletePending,
    /// Database is in recovery from a failure.
    Recovery,
    /// Database is in an error state.
    Error,
}

// ============================================================================
// Utility Types
// ============================================================================

/// Empty response body. Returned by operations that succeed without payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse {}

/// Generic error response body.
///
/// The Redis Cloud API uses several different error shapes; this struct
/// captures the most common keys and leaves all of them optional so callers
/// can match on whichever fields the server returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Short error code or category (e.g. `"NOT_FOUND"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Human-readable error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Additional free-form context about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// HTTP status code echoed in the body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
}
