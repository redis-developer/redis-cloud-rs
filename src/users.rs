//! User management and authentication
//!
//! This module provides comprehensive user management functionality for Redis Cloud,
//! including user creation, role assignment, password management, and multi-factor
//! authentication configuration.
//!
//! # Overview
//!
//! Users in Redis Cloud can have different roles and permissions that control their
//! access to subscriptions, databases, and account settings. The system supports both
//! local users and SSO/SAML integrated users.
//!
//! # User Roles
//!
//! - **Owner**: Full administrative access to all resources
//! - **Manager**: Can manage subscriptions and databases
//! - **Viewer**: Read-only access to resources
//! - **Billing Admin**: Access to billing and payment information
//! - **Custom Roles**: Organization-specific roles with custom permissions
//!
//! # Key Features
//!
//! - **User Lifecycle**: Create, update, delete, and invite users
//! - **Role Management**: Assign and modify user roles
//! - **Password Policies**: Enforce password complexity and rotation
//! - **MFA Support**: Two-factor authentication configuration
//! - **API Access**: Manage programmatic access for users
//! - **Audit Trail**: Track user actions and changes
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, UserHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = UserHandler::new(client);
//!
//! // List all users
//! let users = handler.get_all_users().await?;
//!
//! // Get specific user details (user ID 123)
//! let user = handler.get_user_by_id(123).await?;
//! # Ok(())
//! # }
//! ```

use crate::types::{Link, ProcessorResponse};
use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

// ============================================================================
// Models
// ============================================================================

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUserUpdateRequest {
    /// User ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,

    /// The account user's name.
    pub name: String,

    /// Changes the account user's role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_ACCOUNT_USER"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Account users response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUsers {
    /// Account ID the users belong to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<i32>,

    /// List of users
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<AccountUser>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// User options information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUserOptions {
    /// Whether the user has access to billing information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<bool>,

    /// Whether the user receives email alerts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_alerts: Option<bool>,

    /// Whether the user receives operational/maintenance emails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operational_emails: Option<bool>,

    /// Whether multi-factor authentication is enabled for this user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_enabled: Option<bool>,
}

/// Task state update response for user operations.
///
/// Local copy duplicating [`crate::types::TaskStateUpdate`]. Consolidation
/// onto the canonical type is tracked in #64.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStateUpdate {
    /// UUID of the task. Use with `TasksHandler::get_task_by_id` to poll.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Type of command being executed (e.g. `"DELETE_ACCOUNT_USER"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    /// Current task status (kebab-case, e.g. `"processing-completed"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Human-readable description of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timestamp of the last task update (ISO-8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Result of the task once it has completed. See [`ProcessorResponse`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ProcessorResponse>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUser {
    /// Unique user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// User's display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// User's email address (also the login identifier).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// User's role (e.g. `"Owner"`, `"Manager"`, `"Viewer"`, `"Billing Admin"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Timestamp the user signed up (ISO-8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sign_up: Option<String>,

    /// User type (e.g. `"local"`, `"saml"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,

    /// Whether the user has at least one API key configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_api_key: Option<bool>,

    /// Notification and access option flags for this user.
    /// See [`AccountUserOptions`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<AccountUserOptions>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

// ============================================================================
// Handler
// ============================================================================

/// Handler for user management operations
///
/// Manages user accounts, roles, permissions, invitations,
/// and authentication settings including MFA configuration.
pub struct UsersHandler {
    client: CloudClient,
}

impl UsersHandler {
    /// Create a new handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get users
    /// Gets a list of all account users.
    ///
    /// GET /users
    pub async fn get_all_users(&self) -> Result<AccountUsers> {
        self.client.get("/users").await
    }

    /// Delete user
    /// Deletes a user from this account.
    ///
    /// DELETE /users/{userId}
    pub async fn delete_user_by_id(&self, user_id: i32) -> Result<TaskStateUpdate> {
        let response = self.client.delete_raw(&format!("/users/{user_id}")).await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Get a single user
    /// Gets details about a single account user.
    ///
    /// GET /users/{userId}
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<AccountUser> {
        self.client.get(&format!("/users/{user_id}")).await
    }

    /// Update a user
    /// Updates an account user's name or role.
    ///
    /// PUT /users/{userId}
    pub async fn update_user(
        &self,
        user_id: i32,
        request: &AccountUserUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client.put(&format!("/users/{user_id}"), request).await
    }
}
