//! Account management operations and models
//!
//! This module provides comprehensive account management functionality for Redis Cloud,
//! including account information retrieval, settings management, API keys, owners,
//! payment methods, SSO/SAML configuration, and billing address management.
//!
//! # Overview
//!
//! The account module is the central point for managing organization-wide settings and
//! configurations in Redis Cloud. It handles everything from basic account information
//! to advanced features like SSO integration and API key management.
//!
//! # Key Features
//!
//! - **Account Information**: Get current account details and metadata
//! - **API Key Management**: Create, list, and manage API keys for programmatic access
//! - **Owner Management**: Manage account owners and their permissions
//! - **Payment Methods**: Handle payment methods and billing configuration
//! - **SSO/SAML**: Configure single sign-on and SAML integration
//! - **Billing Address**: Manage billing address information
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, AccountHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = AccountHandler::new(client);
//!
//! // Get current account information
//! let account = handler.get_current_account().await?;
//! println!("Account info: {:?}", account);
//!
//! // Get payment methods
//! let payment_methods = handler.get_account_payment_methods().await?;
//! println!("Payment methods: {:?}", payment_methods);
//! # Ok(())
//! # }
//! ```

use crate::types::Link;
use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};

// ============================================================================
// Models
// ============================================================================

/// Database modules/capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<Module>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Root account response from GET /
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootAccount {
    /// Account information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<Account>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Account name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Timestamp when the account was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_timestamp: Option<String>,

    /// Timestamp when the account was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_timestamp: Option<String>,

    /// Marketplace status (e.g., "active", "deleted")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketplace_status: Option<String>,

    /// API key information used for this request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<AccountApiKeyInfo>,
}

/// API key information returned in account response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountApiKeyInfo {
    /// API key name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Account ID this key belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// Account name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,

    /// Allowed source IP addresses/CIDRs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_source_ips: Option<Vec<String>>,

    /// Owner information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountApiKeyOwner>,

    /// User account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_account_id: Option<i32>,

    /// HTTP source IP of the current request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_source_ip: Option<String>,

    /// Account marketplace ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_marketplace_id: Option<String>,
}

/// API key owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountApiKeyOwner {
    /// Owner's name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Owner's email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Account system log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSystemLogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub originator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// Resource ID associated with this log entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Available regions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<Region>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// Region ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Region name (e.g., "us-east-1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Cloud provider (e.g., "AWS", "GCP", "Azure")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Account payment methods response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethods {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of payment methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_methods: Option<Vec<PaymentMethod>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Payment method information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    /// Payment method ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Card type (e.g., "Mastercard", "Visa")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Last 4 digits of the credit card
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_ends_with: Option<i32>,

    /// Name on the card
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_on_card: Option<String>,

    /// Expiration month (1-12)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_month: Option<i32>,

    /// Expiration year
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_year: Option<i32>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Database module/capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Module name (e.g., "RedisJSON", "RediSearch")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Capability name (e.g., "JSON", "Search and query")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_name: Option<String>,

    /// Module description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Module parameters configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ModuleParameter>>,
}

/// Module parameter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleParameter {
    /// Parameter name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter type (e.g., "integer")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Default value for the parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<i64>,

    /// Whether this parameter is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Account system log entries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSystemLogEntries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<AccountSystemLogEntry>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Query performance factors (search scaling) response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchScalingFactorsData {
    /// Available query performance factors (e.g., "Standard", "2x", "4x")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_performance_factors: Option<Vec<String>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Account session log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSessionLogEntry {
    /// Session log entry ID (UUID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Timestamp of the session event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    /// User who performed the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// IP address of the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User role (e.g., "owner")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_role: Option<String>,

    /// Session type (e.g., "sso")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Action performed (e.g., "Successful login", "Successful logout")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// Data persistence option entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPersistenceEntry {
    /// Persistence option name (e.g., "none", "aof-every-1-second")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Data persistence options response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPersistenceOptions {
    /// Available data persistence options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_persistence: Option<Vec<DataPersistenceEntry>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Account session log entries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSessionLogEntries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<AccountSessionLogEntry>>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

// ============================================================================
// Handler
// ============================================================================

/// Account operations handler
/// Handler for account management operations
///
/// Provides methods for managing account information, API keys, owners,
/// payment methods, SSO/SAML configuration, and billing addresses.
pub struct AccountHandler {
    client: CloudClient,
}

impl AccountHandler {
    /// Create a new handler
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get current account
    /// Gets information on this account.
    ///
    /// GET /
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
    /// let root = client.account().get_current_account().await?;
    /// if let Some(account) = &root.account {
    ///     println!("Account ID: {:?}", account.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_account(&self) -> Result<RootAccount> {
        self.client.get("/").await
    }

    /// Get data persistence options
    /// Gets a list of all [data persistence](https://redis.io/docs/latest/operate/rc/databases/configuration/data-persistence/) options for this account.
    ///
    /// GET /data-persistence
    pub async fn get_data_persistence_options(&self) -> Result<DataPersistenceOptions> {
        self.client.get("/data-persistence").await
    }

    /// Get advanced capabilities
    /// Gets a list of Redis [advanced capabilities](https://redis.io/docs/latest/operate/rc/databases/configuration/advanced-capabilities/) (also known as modules) available for this account. Advanced capability support may differ based on subscription and database settings.
    ///
    /// GET /database-modules
    pub async fn get_supported_database_modules(&self) -> Result<ModulesData> {
        self.client.get("/database-modules").await
    }

    /// Get system logs
    /// Gets [system logs](https://redis.io/docs/latest/operate/rc/api/examples/audit-system-logs/) for this account.
    ///
    /// GET /logs
    pub async fn get_account_system_logs(
        &self,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AccountSystemLogEntries> {
        let mut query = Vec::new();
        if let Some(v) = offset {
            query.push(format!("offset={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        let query_string = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.client.get(&format!("/logs{}", query_string)).await
    }

    /// Get payment methods
    /// Gets a list of all payment methods for this account.
    ///
    /// GET /payment-methods
    pub async fn get_account_payment_methods(&self) -> Result<PaymentMethods> {
        self.client.get("/payment-methods").await
    }

    /// Get query performance factors
    /// Gets a list of available [query performance factors](https://redis.io/docs/latest/operate/rc/databases/configuration/advanced-capabilities/#query-performance-factor).
    ///
    /// GET /query-performance-factors
    pub async fn get_supported_search_scaling_factors(&self) -> Result<SearchScalingFactorsData> {
        self.client.get("/query-performance-factors").await
    }

    /// Get available Pro plan regions
    /// Gets a list of available regions for Pro subscriptions. For Essentials subscriptions, use 'GET /fixed/plans'.
    ///
    /// GET /regions
    pub async fn get_supported_regions(&self, provider: Option<String>) -> Result<Regions> {
        let mut query = Vec::new();
        if let Some(v) = provider {
            query.push(format!("provider={}", v));
        }
        let query_string = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.client.get(&format!("/regions{}", query_string)).await
    }

    /// Get session logs
    /// Gets session logs for this account.
    ///
    /// GET /session-logs
    pub async fn get_account_session_logs(
        &self,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AccountSessionLogEntries> {
        let mut query = Vec::new();
        if let Some(v) = offset {
            query.push(format!("offset={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        let query_string = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.client
            .get(&format!("/session-logs{}", query_string))
            .await
    }
}
