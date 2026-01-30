//! Role-based Access Control (RBAC) operations and models
//!
//! This module provides comprehensive access control management for Redis Cloud,
//! including ACL management for users, roles, Redis rules, and database-level
//! access controls.
//!
//! # Overview
//!
//! The ACL module implements Redis Cloud's role-based access control system, allowing
//! fine-grained control over who can access what resources and perform which operations.
//! It supports both user-level and database-level access controls.
//!
//! # Key Features
//!
//! - **User ACLs**: Manage user access control lists and permissions
//! - **Role Management**: Create and manage roles with specific permissions
//! - **Redis Rules**: Define Redis command-level access rules
//! - **Database ACLs**: Control access at the database level
//! - **Rule Association**: Link users and roles to specific databases
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, AclHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = AclHandler::new(client);
//!
//! // List all ACL users
//! let users = handler.get_all_acl_users().await?;
//!
//! // Get all Redis rules
//! let rules = handler.get_all_redis_rules().await?;
//! # Ok(())
//! # }
//! ```

use crate::types::{Link, ProcessorResponse};
use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

// ============================================================================
// Models
// ============================================================================

/// ACL role create request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRoleCreateRequest {
    /// Database access role name.
    pub name: String,

    /// A list of Redis ACL rules to assign to this database access role.
    pub redis_rules: Vec<AclRoleRedisRuleSpec>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// ACL user update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclUserUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,

    /// Optional. Changes the ACL role assigned to the user. Use GET '/acl/roles' to get a list of database access roles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Optional. Changes the user's database password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// ACL users response
///
/// Response from GET /acl/users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountACLUsers {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of ACL users
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<ACLUser>>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// ACL Redis rules response
///
/// Response from GET /acl/redisRules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountACLRedisRules {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of Redis ACL rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_rules: Option<Vec<ACLRedisRule>>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// ACL Redis rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACLRedisRule {
    /// Rule ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Rule name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// ACL pattern (e.g., "+@all ~lcm:*")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl: Option<String>,

    /// Whether this is a default rule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// Rule status (e.g., "active")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// ACL Redis rule create request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRedisRuleCreateRequest {
    /// Redis ACL rule name.
    pub name: String,

    /// Redis ACL rule pattern. See [ACL syntax](https://redis.io/docs/latest/operate/rc/security/access-control/data-access-control/configure-acls/#define-permissions-with-acl-syntax) to learn how to define rules.
    pub redis_rule: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// ACL roles response
///
/// Response from GET /acl/roles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountACLRoles {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of ACL roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<ACLRole>>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// ACL role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACLRole {
    /// Role ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Role name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Redis rules associated with this role
    ///
    /// Note: These use different field names (ruleId, ruleName) than standalone redis rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_rules: Option<Vec<ACLRoleRedisRule>>,

    /// Users assigned to this role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<ACLRoleUser>>,

    /// Role status (e.g., "active")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// User reference in an ACL role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACLRoleUser {
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// User name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Redis rule as embedded in an ACL role response
///
/// This has different field names than ACLRedisRule because the API
/// uses different JSON keys when rules are embedded in role responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACLRoleRedisRule {
    /// Rule ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<i32>,

    /// Rule name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,

    /// Databases this rule applies to within the role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases: Option<Vec<ACLRoleDatabase>>,
}

/// Database reference in an ACL role's redis rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACLRoleDatabase {
    /// Subscription ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Database ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<i32>,

    /// Database name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,

    /// Regions (for Active-Active databases)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<String>>,
}

/// ACL Redis rule update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRedisRuleUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_rule_id: Option<i32>,

    /// Optional. Changes the Redis ACL rule name.
    pub name: String,

    /// Optional. Changes the Redis ACL rule pattern. See [ACL syntax](https://redis.io/docs/latest/operate/rc/security/access-control/data-access-control/configure-acls/#define-permissions-with-acl-syntax) to learn how to define rules.
    pub redis_rule: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Database specification for ACL role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRoleDatabaseSpec {
    /// Subscription ID for the database's subscription. Use 'GET /subscriptions' or 'GET /fixed/subscriptions' to get a list of available subscriptions and their IDs.
    pub subscription_id: i32,

    /// The database's ID. Use 'GET /subscriptions/{subscriptionId}/databases' or 'GET /fixed/subscriptions/{subscriptionId}/databases' to get a list of databases in a subscription and their IDs.
    pub database_id: i32,

    /// (Active-Active databases only) Optional. A list of regions where this rule applies for this role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<String>>,
}

/// ACL user create request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclUserCreateRequest {
    /// Access control user name.
    pub name: String,

    /// Name of the database access role to assign to this user. Use GET '/acl/roles' to get a list of database access roles.
    pub role: String,

    /// The database password for this user.
    pub password: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// ACL user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACLUser {
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// User name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Assigned role name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// User status (e.g., "active", "error")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// ACL role update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRoleUpdateRequest {
    /// Optional. Changes the database access role name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional. Changes the Redis ACL rules to assign to this database access role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_rules: Option<Vec<AclRoleRedisRuleSpec>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Redis rule specification for role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclRoleRedisRuleSpec {
    /// The name of a Redis ACL rule to assign to the role. Use 'GET /acl/redisRules' to get a list of available rules for your account.
    pub rule_name: String,

    /// A list of databases where the specified rule applies for this role.
    pub databases: Vec<AclRoleDatabaseSpec>,
}

/// Task state update response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStateUpdate {
    /// Task ID (UUID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Command type (e.g., "subscriptionDeleteRequest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    /// Task status (e.g., "processing-completed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Task description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timestamp of the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Task response with resource info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ProcessorResponse>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

// ============================================================================
// Handler
// ============================================================================

/// Handler for Role-based Access Control (RBAC) operations
///
/// Manages ACLs for users, roles, Redis rules, and database-level access controls.
/// Provides fine-grained permission management for Redis Cloud resources.
pub struct AclHandler {
    client: CloudClient,
}

impl AclHandler {
    /// Create a new handler
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get Redis ACL rules
    /// Gets a list of all Redis ACL rules for this account.
    ///
    /// GET /acl/redisRules
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
    /// let rules = client.acl().get_all_redis_rules().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_redis_rules(&self) -> Result<AccountACLRedisRules> {
        self.client.get("/acl/redisRules").await
    }

    /// Create Redis ACL rule
    /// Creates a new Redis ACL rule.
    ///
    /// POST /acl/redisRules
    pub async fn create_redis_rule(
        &self,
        request: &AclRedisRuleCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client.post("/acl/redisRules", request).await
    }

    /// Delete Redis ACL rule
    /// Deletes a Redis ACL rule.
    ///
    /// DELETE /acl/redisRules/{aclRedisRuleId}
    pub async fn delete_redis_rule(&self, acl_redis_rule_id: i32) -> Result<TaskStateUpdate> {
        let response = self
            .client
            .delete_raw(&format!("/acl/redisRules/{}", acl_redis_rule_id))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Update Redis ACL rule
    /// Updates a Redis ACL rule.
    ///
    /// PUT /acl/redisRules/{aclRedisRuleId}
    pub async fn update_redis_rule(
        &self,
        acl_redis_rule_id: i32,
        request: &AclRedisRuleUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/acl/redisRules/{}", acl_redis_rule_id), request)
            .await
    }

    /// Get database access roles
    /// Gets a list of all database access roles for this account.
    ///
    /// GET /acl/roles
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
    /// let roles = client.acl().get_roles().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_roles(&self) -> Result<AccountACLRoles> {
        self.client.get("/acl/roles").await
    }

    /// Create database access role
    /// Creates a new database access role with the assigned permissions and associates it with the provided databases.
    ///
    /// POST /acl/roles
    pub async fn create_role(&self, request: &AclRoleCreateRequest) -> Result<TaskStateUpdate> {
        self.client.post("/acl/roles", request).await
    }

    /// Delete database access role
    /// Deletes a database access role.
    ///
    /// DELETE /acl/roles/{aclRoleId}
    pub async fn delete_acl_role(&self, acl_role_id: i32) -> Result<TaskStateUpdate> {
        let response = self
            .client
            .delete_raw(&format!("/acl/roles/{}", acl_role_id))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Update database access role
    /// Updates a database access role with new assigned permissions or associated databases.
    ///
    /// PUT /acl/roles/{aclRoleId}
    pub async fn update_role(
        &self,
        acl_role_id: i32,
        request: &AclRoleUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/acl/roles/{}", acl_role_id), request)
            .await
    }

    /// Get access control users
    /// Gets a list of all access control users for this account.
    ///
    /// GET /acl/users
    pub async fn get_all_acl_users(&self) -> Result<AccountACLUsers> {
        self.client.get("/acl/users").await
    }

    /// Create access control user
    /// Creates a new access control user with the assigned database access role.
    ///
    /// POST /acl/users
    pub async fn create_user(&self, request: &AclUserCreateRequest) -> Result<TaskStateUpdate> {
        self.client.post("/acl/users", request).await
    }

    /// Delete access control user
    /// Deletes a access control user.
    ///
    /// DELETE /acl/users/{aclUserId}
    pub async fn delete_user(&self, acl_user_id: i32) -> Result<TaskStateUpdate> {
        let response = self
            .client
            .delete_raw(&format!("/acl/users/{}", acl_user_id))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Get a single access control user
    /// Gets details and settings for single access control user.
    ///
    /// GET /acl/users/{aclUserId}
    pub async fn get_user_by_id(&self, acl_user_id: i32) -> Result<ACLUser> {
        self.client
            .get(&format!("/acl/users/{}", acl_user_id))
            .await
    }

    /// Update access control user
    /// Updates a access control user with a different role or database password.
    ///
    /// PUT /acl/users/{aclUserId}
    pub async fn update_acl_user(
        &self,
        acl_user_id: i32,
        request: &AclUserUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/acl/users/{}", acl_user_id), request)
            .await
    }
}
