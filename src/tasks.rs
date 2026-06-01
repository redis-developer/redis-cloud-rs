//! Asynchronous task tracking and management
//!
//! This module provides functionality for tracking long-running operations in
//! Redis Cloud. Many API operations are asynchronous and return a task ID that
//! can be used to monitor progress and completion status.
//!
//! # Overview
//!
//! Redis Cloud uses tasks for operations that may take time to complete, such as:
//! - Creating or deleting subscriptions
//! - Database creation, updates, and deletion
//! - Backup and restore operations
//! - Import/export operations
//! - Network configuration changes
//!
//! # Task Lifecycle
//!
//! 1. **Initiated**: Task is created and queued
//! 2. **Processing**: Task is being executed
//! 3. **Completed**: Task finished successfully
//! 4. **Failed**: Task encountered an error
//!
//! # Key Features
//!
//! - **Task Status**: Check current status of any task
//! - **Progress Tracking**: Monitor completion percentage for long operations
//! - **Result Retrieval**: Get operation results once completed
//! - **Error Information**: Access detailed error messages for failed tasks
//! - **Task History**: Query historical task information
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, TaskHandler};
//! use redis_cloud::types::TaskStatus;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = TaskHandler::new(client);
//!
//! // Get task status
//! let task = handler.get_task_by_id("task-123".to_string()).await?;
//!
//! // Check if task is complete
//! if matches!(task.status, Some(TaskStatus::ProcessingCompleted)) {
//!     println!("Task completed successfully");
//!     if let Some(response) = task.response {
//!         println!("Result: {:?}", response);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::CloudError;
use crate::{CloudClient, Result};

// Canonical task models live in `crate::types` (#64). Re-exported here so the
// historical `tasks::TaskStateUpdate` path keeps resolving.
pub use crate::types::TaskStateUpdate;
use crate::types::TasksStateUpdate;

// ============================================================================
// Handler
// ============================================================================

/// Handler for asynchronous task operations
///
/// Tracks and manages long-running operations, providing status updates,
/// progress monitoring, and result retrieval for asynchronous API calls.
pub struct TasksHandler {
    client: CloudClient,
}

impl TasksHandler {
    /// Create a new handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get tasks
    /// Gets a list of all currently running tasks for this account.
    ///
    /// The OpenAPI spec defines the response as `TasksStateUpdate { tasks: [...] }`
    /// (a wrapper object). In practice the API also returns:
    /// - an empty object `{}` when there are no tasks,
    /// - and historically a bare JSON array.
    ///
    /// All three shapes deserialize cleanly to `Vec<TaskStateUpdate>`. Any other
    /// shape surfaces as [`CloudError::JsonError`] rather than silently returning
    /// an empty list, so a future schema change is loud instead of invisible.
    ///
    /// GET /tasks
    pub async fn get_all_tasks(&self) -> Result<Vec<TaskStateUpdate>> {
        let value: serde_json::Value = self.client.get_raw("/tasks").await?;
        match value {
            // Canonical spec shape: {"tasks": [...]}
            serde_json::Value::Object(ref obj) if obj.contains_key("tasks") => {
                let wrapped: TasksStateUpdate = serde_json::from_value(value)?;
                Ok(wrapped.tasks)
            }
            // No tasks: API returns `{}` (or null) instead of the wrapper.
            serde_json::Value::Object(obj) if obj.is_empty() => Ok(Vec::new()),
            serde_json::Value::Null => Ok(Vec::new()),
            // Legacy bare-array shape, still tolerated.
            serde_json::Value::Array(_) => Ok(serde_json::from_value(value)?),
            other => Err(CloudError::JsonError(format!(
                "GET /tasks: expected {{\"tasks\": [...]}}, {{}}, null, or a bare array; got {other}"
            ))),
        }
    }

    /// Get tasks (raw JSON)
    /// Gets a list of all currently running tasks for this account.
    ///
    /// GET /tasks
    pub async fn get_all_tasks_raw(&self) -> Result<serde_json::Value> {
        self.client.get_raw("/tasks").await
    }

    /// Get a single task
    /// Gets details and status of a single task by the Task ID.
    ///
    /// GET /tasks/{taskId}
    ///
    /// # Example
    ///
    /// ```no_run
    /// use redis_cloud::CloudClient;
    ///
    /// # async fn example() -> redis_cloud::Result<()> {
    /// let client = CloudClient::builder()
    ///     .api_key("your-api-key")
    ///     .api_secret("your-api-secret")
    ///     .build()?;
    ///
    /// let task = client.tasks().get_task_by_id("task-id".to_string()).await?;
    /// println!("Task status: {:?}", task.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_task_by_id(&self, task_id: String) -> Result<TaskStateUpdate> {
        self.client.get(&format!("/tasks/{task_id}")).await
    }
}
