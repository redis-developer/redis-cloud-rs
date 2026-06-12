//! Subscription management for Pro (Flexible) plans
//!
//! This module provides comprehensive management of Redis Cloud Pro subscriptions,
//! which offer flexible, scalable Redis deployments with advanced features like
//! auto-scaling, multi-region support, and Active-Active configurations.
//!
//! # Overview
//!
//! Pro subscriptions are Redis Cloud's most flexible offering, supporting everything
//! from small development instances to large-scale production deployments with
//! automatic scaling, clustering, and global distribution.
//!
//! # Key Features
//!
//! - **Flexible Scaling**: Auto-scaling based on usage patterns
//! - **Multi-Region**: Deploy across multiple regions and cloud providers
//! - **Active-Active**: Global database replication with local reads/writes
//! - **Advanced Networking**: VPC peering, Transit Gateway, Private endpoints
//! - **Maintenance Windows**: Configurable maintenance scheduling
//! - **CIDR Management**: IP allowlist and security group configuration
//! - **Custom Pricing**: Usage-based pricing with detailed cost tracking
//!
//! # Subscription Types
//!
//! - **Single-Region**: Standard deployment in one region
//! - **Multi-Region**: Replicated across multiple regions
//! - **Active-Active**: CRDB with conflict-free replicated data types
//!
//! # Example Usage
//!
//! ```no_run
//! use redis_cloud::{CloudClient, SubscriptionHandler};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CloudClient::builder()
//!     .api_key("your-api-key")
//!     .api_secret("your-api-secret")
//!     .build()?;
//!
//! let handler = SubscriptionHandler::new(client);
//!
//! // List all Pro subscriptions
//! let subscriptions = handler.get_all_subscriptions().await?;
//!
//! // Get subscription details (subscription ID 123)
//! let subscription = handler.get_subscription_by_id(123).await?;
//!
//! // Manage maintenance windows
//! let windows = handler.get_subscription_maintenance_windows(123).await?;
//! # Ok(())
//! # }
//! ```

pub use crate::types::TaskStateUpdate;
use crate::types::{Link, Tag};
use crate::{CloudClient, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use typed_builder::TypedBuilder;

// ============================================================================
// Models
// ============================================================================

/// Subscription update request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSubscriptionUpdateRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_SUBSCRIPTION"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Subscription update request message
///
/// # Example
///
/// ```
/// use redis_cloud::flexible::subscriptions::SubscriptionUpdateRequest;
///
/// let request = SubscriptionUpdateRequest::builder()
///     .name("updated-subscription")
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionUpdateRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub subscription_id: Option<i32>,

    /// Optional. Updated subscription name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,

    /// Optional. The payment method ID you'd like to use for this subscription. Must be a valid payment method ID for this account. Use GET /payment-methods to get all payment methods for your account. This value is optional if 'paymentMethod' is 'marketplace', but required if 'paymentMethod' is 'credit-card'.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub payment_method_id: Option<i32>,

    /// Optional. The payment method for the subscription. If set to 'credit-card' , 'paymentMethodId' must be defined.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub payment_method: Option<String>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_SUBSCRIPTION"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub command_type: Option<String>,
}

/// Cloud provider, region, and networking details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSpec {
    /// Optional. Cloud provider. Default: 'AWS'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Optional. Cloud account identifier. Default: Redis internal cloud account (Cloud Account ID = 1). Use GET /cloud-accounts to list all available cloud accounts. Note: A subscription on Google Cloud can be created only with Redis internal cloud account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_account_id: Option<i32>,

    /// The cloud provider region or list of regions (Active-Active only) and networking details.
    pub regions: Vec<SubscriptionRegionSpec>,
}

/// Object representing a customer managed key (CMK), along with the region it is associated to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerManagedKey {
    /// Required. Resource name of the customer managed key as defined by the cloud provider.
    pub resource_name: String,

    /// Name of region to for the customer managed key as defined by the cloud provider. Required for active-active subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Optional. Expected read and write throughput for this region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalThroughput {
    /// Specify one of the selected cloud provider regions for the subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Write operations for this region per second. Default: 1000 ops/sec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_operations_per_second: Option<i64>,

    /// Read operations for this region per second. Default: 1000 ops/sec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_operations_per_second: Option<i64>,
}

/// List of databases in the subscription with local throughput details. Default: 1000 read and write ops/sec for each database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrdbRegionSpec {
    /// Database name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional. Local throughput settings for this region. See [`LocalThroughput`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_throughput_measurement: Option<LocalThroughput>,
}

/// Subscription update request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionUpdateCMKRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_SUBSCRIPTION_CMK"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,

    /// Optional. The grace period for deleting the subscription. If not set, will default to immediate deletion grace period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_grace_period: Option<String>,

    /// The customer managed keys (CMK) to use for this subscription. If is active-active subscription, must set a key for each region.
    pub customer_managed_keys: Vec<CustomerManagedKey>,
}

/// `SubscriptionPricings`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPricings {
    /// Pricing breakdown entries for the subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<Vec<SubscriptionPricing>>,
}

/// Optional. Throughput measurement method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseThroughputSpec {
    /// Throughput measurement method. Use 'operations-per-second' for all new databases.
    pub by: String,

    /// Throughput value in the selected measurement method.
    pub value: i64,
}

/// Optional. Redis advanced capabilities (also known as modules) to be provisioned in the database. Use GET /database-modules to get a list of available advanced capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseModuleSpec {
    /// Redis advanced capability name. Use GET /database-modules for a list of available capabilities.
    pub name: String,

    /// Optional. Redis advanced capability parameters. Use GET /database-modules to get the available capabilities and their parameters.
    ///
    /// Kept as a [`Value`] because the wire shape is asymmetric: create
    /// requests send an object (capability name → parameter map), while
    /// database reads return an array. A typed map only matched the request
    /// side and failed to deserialize real responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

/// Update Pro subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CidrAllowlistUpdateRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// List of CIDR values. Example: ['10.1.1.0/32']
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_ips: Option<Vec<String>>,

    /// List of AWS Security group IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_group_ids: Option<Vec<String>>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"UPDATE_SUBSCRIPTION_CIDR_ALLOWLIST"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// `SubscriptionMaintenanceWindowsSpec`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionMaintenanceWindowsSpec {
    /// Maintenance window mode: either 'manual' or 'automatic'. Must provide 'windows' if manual.
    pub mode: String,

    /// Maintenance window timeframes if mode is set to 'manual'. Up to 7 maintenance windows can be provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<Vec<MaintenanceWindowSpec>>,
}

/// `MaintenanceWindowSkipStatus`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowSkipStatus {
    /// Number of remaining maintenance-window skips available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_skips: Option<i32>,

    /// Timestamp marking the end of the currently skipped window, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_skip_end: Option<String>,
}

/// List of active-active subscription regions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveSubscriptionRegions {
    /// Subscription identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// HATEOAS links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// `SubscriptionPricing`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPricing {
    /// Database name this pricing applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,

    /// Pricing line type (e.g. `"Shards"`, `"EBSVolume"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Additional details about the pricing line type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_details: Option<String>,

    /// Quantity of the priced unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,

    /// Unit used to measure `quantity` (e.g. `"shards"`, `"GB"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity_measurement: Option<String>,

    /// Price per unit in the configured currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_per_unit: Option<f64>,

    /// ISO currency code for `price_per_unit` (e.g. `"USD"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_currency: Option<String>,

    /// Billing period for the price (e.g. `"Month"`, `"Hour"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_period: Option<String>,

    /// Cloud region this pricing entry applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Request structure for creating a new Pro subscription
///
/// Defines configuration for flexible subscriptions including cloud providers,
/// regions, deployment type, and initial database specifications.
///
/// # Example
///
/// ```
/// use redis_cloud::flexible::subscriptions::{SubscriptionCreateRequest, SubscriptionSpec, SubscriptionDatabaseSpec, SubscriptionRegionSpec};
///
/// let request = SubscriptionCreateRequest::builder()
///     .name("my-subscription")
///     .cloud_providers(vec![
///         SubscriptionSpec {
///             provider: Some("AWS".to_string()),
///             cloud_account_id: Some(1),
///             regions: vec![SubscriptionRegionSpec {
///                 region: "us-east-1".to_string(),
///                 multiple_availability_zones: None,
///                 preferred_availability_zones: None,
///                 networking: None,
///             }],
///         }
///     ])
///     .databases(vec![
///         SubscriptionDatabaseSpec {
///             name: "my-database".to_string(),
///             protocol: "redis".to_string(),
///             memory_limit_in_gb: Some(1.0),
///             dataset_size_in_gb: None,
///             support_oss_cluster_api: None,
///             data_persistence: None,
///             replication: None,
///             throughput_measurement: None,
///             local_throughput_measurement: None,
///             modules: None,
///             quantity: None,
///             average_item_size_in_bytes: None,
///             resp_version: None,
///             redis_version: None,
///             sharding_type: None,
///             query_performance_factor: None,
///         }
///     ])
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionCreateRequest {
    /// Optional. New subscription name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,

    /// Optional. When 'false': Creates a deployment plan and deploys it, creating any resources required by the plan. When 'true': creates a read-only deployment plan and does not create any resources. Default: 'false'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub dry_run: Option<bool>,

    /// Optional. When 'single-region' or not set: Creates a single region subscription. When 'active-active': creates an Active-Active (multi-region) subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub deployment_type: Option<String>,

    /// Optional. The payment method for the subscription. If set to 'credit-card', 'paymentMethodId' must be defined. Default: 'credit-card'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub payment_method: Option<String>,

    /// Optional. A valid payment method ID for this account. Use GET /payment-methods to get a list of all payment methods for your account. This value is optional if 'paymentMethod' is 'marketplace', but required for all other account types.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub payment_method_id: Option<i32>,

    /// Optional. Memory storage preference: either 'ram' or a combination of 'ram-and-flash' (also known as Auto Tiering). Default: 'ram'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub memory_storage: Option<String>,

    /// Optional. Persistent storage encryption secures data-at-rest for database persistence. You can use 'cloud-provider-managed-key' or 'customer-managed-key'.  Default: 'cloud-provider-managed-key'
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub persistent_storage_encryption_type: Option<String>,

    /// Cloud provider, region, and networking details.
    pub cloud_providers: Vec<SubscriptionSpec>,

    /// One or more database specification(s) to create in this subscription.
    pub databases: Vec<SubscriptionDatabaseSpec>,

    /// Optional. Defines the Redis version of the databases created in this specific request. It doesn't determine future databases associated with this subscription. If not set, databases will use the default Redis version. This field is deprecated and will be removed in a future API version - use the database-level redisVersion property instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub redis_version: Option<String>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"CREATE_SUBSCRIPTION"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub command_type: Option<String>,
}

/// Configuration regarding customer managed persistent storage encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerManagedKeyAccessDetails {
    /// Redis service account that requires CMK access (GCP).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_service_account: Option<String>,

    /// GCP predefined roles the service account must be granted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_predefined_roles: Option<Vec<String>>,

    /// GCP custom permissions required on the customer managed key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_custom_permissions: Option<Vec<String>>,

    /// AWS IAM role used by Redis to access the customer managed key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_iam_role: Option<String>,

    /// AWS KMS key-policy statements required for Redis to use the CMK.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_key_policy_statements: Option<HashMap<String, Value>>,

    /// Supported deletion grace period options for the CMK.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_grace_period_options: Option<Vec<String>>,
}

/// One or more database specification(s) to create in this subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDatabaseSpec {
    /// Name of the database. Database name is limited to 40 characters or less and must include only letters, digits, and hyphens ('-'). It must start with a letter and end with a letter or digit.
    pub name: String,

    /// Optional. Database protocol. Only set to 'memcached' if you have a legacy application. Default: 'redis'
    pub protocol: String,

    /// Optional. Total memory in GB, including replication and other overhead. You cannot set both datasetSizeInGb and totalMemoryInGb.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_in_gb: Option<f64>,

    /// Optional. The maximum amount of data in the dataset for this database in GB. You cannot set both datasetSizeInGb and totalMemoryInGb. If ‘replication’ is 'true', the database’s total memory will be twice as large as the datasetSizeInGb.If ‘replication’ is false, the database’s total memory will be the datasetSizeInGb value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_size_in_gb: Option<f64>,

    /// Optional. Support Redis [OSS Cluster API](https://redis.io/docs/latest/operate/rc/databases/configuration/clustering/#oss-cluster-api). Default: 'false'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_oss_cluster_api: Option<bool>,

    /// Optional. Type and rate of data persistence in persistent storage. Default: 'none'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_persistence: Option<String>,

    /// Optional. Databases replication. Default: 'true'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication: Option<bool>,

    /// Optional. Throughput measurement spec. See [`DatabaseThroughputSpec`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput_measurement: Option<DatabaseThroughputSpec>,

    /// Optional. Expected throughput per region for an Active-Active database. Default: 1000 read and write ops/sec for each region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_throughput_measurement: Option<Vec<LocalThroughput>>,

    /// Optional. Redis advanced capabilities (also known as modules) to be provisioned in the database. Use GET /database-modules to get a list of available advanced capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<DatabaseModuleSpec>>,

    /// Optional. Number of databases that will be created with these settings. Default: 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,

    /// Optional. Relevant only to ram-and-flash (also known as Auto Tiering) subscriptions. Estimated average size in bytes of the items stored in the database. Default: 1000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_item_size_in_bytes: Option<i64>,

    /// Optional. Redis Serialization Protocol version. Must be compatible with Redis version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resp_version: Option<String>,

    /// Optional. If specified, redisVersion defines the Redis database version. If omitted, the Redis version will be set to the default version (available in 'GET /subscriptions/redis-versions')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_version: Option<String>,

    /// Optional. Database [Hashing policy](https://redis.io/docs/latest/operate/rc/databases/configuration/clustering/#manage-the-hashing-policy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharding_type: Option<String>,

    /// Optional. The query performance factor adds extra compute power specifically for search and query databases. You can increase your queries per second by the selected factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_performance_factor: Option<String>,
}

/// Optional. Cloud networking details, per region. Required if creating an Active-Active subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRegionNetworkingSpec {
    /// Optional. Deployment CIDR mask. Must be a valid CIDR format with a range of 256 IP addresses. Default for single-region subscriptions: If using Redis internal cloud account, 192.168.0.0/24
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_cidr: Option<String>,

    /// Optional. Enter a VPC identifier that exists in the hosted AWS account. Creates a new VPC if not set. VPC Identifier must be in a valid format (for example: 'vpc-0125be68a4625884ad') and must exist within the hosting account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// Optional. Enter a list of subnets identifiers that exists in the hosted AWS account. Subnet Identifier must exist within the hosting account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_ids: Option<Vec<String>>,

    /// Optional. Enter a security group identifier that exists in the hosted AWS account. Security group Identifier must be in a valid format (for example: 'sg-0125be68a4625884ad') and must exist within the hosting account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_group_id: Option<String>,
}

/// `RedisVersion`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisVersion {
    /// Redis version string (e.g. `"7.2"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// End-of-life date for this Redis version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eol_date: Option<String>,

    /// Whether this Redis version is a preview/early-access release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preview: Option<bool>,

    /// Whether this Redis version is the default for new databases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

/// `MaintenanceWindow`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindow {
    /// Days of the week the window is active (e.g. `["Monday", "Wednesday"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days: Option<Vec<String>>,

    /// Window start hour in 24-hour UTC time (0-23).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_hour: Option<i32>,

    /// Window duration in hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_in_hours: Option<i32>,
}

/// Cloud provider details for a subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudDetail {
    /// Cloud provider (e.g., "AWS", "GCP", "Azure")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Cloud account ID (Redis Cloud internal or BYOA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_account_id: Option<i32>,

    /// AWS account ID (for AWS deployments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_account_id: Option<String>,

    /// Total size of the subscription in GB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size_in_gb: Option<f64>,

    /// Regions configured for this cloud provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<SubscriptionRegion>>,

    /// Resource tags applied to the subscription's cloud resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_tags: Option<Vec<Tag>>,

    /// HATEOAS links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Region details in a subscription response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRegion {
    /// Region name (e.g., "us-east-1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Networking configuration for this region
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking: Option<Vec<SubscriptionNetworking>>,

    /// Preferred availability zones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_availability_zones: Option<Vec<String>>,

    /// Whether multiple availability zones are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_availability_zones: Option<bool>,
}

/// Networking configuration in a subscription region
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionNetworking {
    /// Deployment CIDR.
    ///
    /// Wire field is `deploymentCIDR` (capital CIDR), so an explicit rename is
    /// needed — `rename_all = "camelCase"` would produce `deploymentCidr` and
    /// silently drop the real value (same casing pitfall as #108/#121).
    #[serde(rename = "deploymentCIDR", skip_serializing_if = "Option::is_none")]
    pub deployment_cidr: Option<String>,

    /// VPC ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// Subnet ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// Security group ID associated with the deployment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_group_id: Option<String>,
}

/// `RedisLabs` Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Subscription
///
/// Represents a Redis Cloud subscription with all known API fields as first-class struct members.
/// The `extra` field is reserved only for truly unknown/future fields that may be added to the API.
pub struct Subscription {
    /// Subscription ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Subscription name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Subscription status (e.g., "active", "pending", "error")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Payment method ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<i32>,

    /// Payment method type (e.g., "credit-card", "marketplace")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_type: Option<String>,

    /// Payment method (e.g., "credit-card", "marketplace")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,

    /// Memory storage type: "ram" or "ram-and-flash" (Auto Tiering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_storage: Option<String>,

    /// Persistent storage encryption type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_storage_encryption_type: Option<String>,

    /// Deployment type: "single-region" or "active-active"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_type: Option<String>,

    /// Number of databases in this subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_databases: Option<i32>,

    /// Cloud provider details (AWS, GCP, Azure configurations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_details: Option<Vec<CloudDetail>>,

    /// Pricing details for the subscription.
    ///
    /// Wire field is `subscriptionPricing`; the field was previously named
    /// `pricing` (serialized as `pricing`) and silently dropped the real value.
    #[serde(
        rename = "subscriptionPricing",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_pricing: Option<Vec<SubscriptionPricing>>,

    /// Redis version for databases created in this subscription (deprecated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_version: Option<String>,

    /// Deletion grace period for customer-managed keys
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_grace_period: Option<String>,

    /// Customer-managed key access details for encryption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_managed_key_access_details: Option<CustomerManagedKeyAccessDetails>,

    /// Whether storage encryption is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_encryption: Option<bool>,

    /// Whether public endpoint access is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_endpoint_access: Option<bool>,

    /// Timestamp when subscription was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_timestamp: Option<String>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Maintenance window timeframes if mode is set to 'manual'. Up to 7 maintenance windows can be provided.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowSpec {
    /// Starting hour of the maintenance window. Can be between '0' (12 AM in the deployment region's local time) and '23' (11 PM in the deployment region's local time).
    pub start_hour: i32,

    /// The duration of the maintenance window in hours. Can be between 4-24 hours (or 8-24 hours if using 'ram-and-flash').
    pub duration_in_hours: i32,

    /// Days where this maintenance window applies. Can contain one or more of: "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", or "Sunday".
    pub days: Vec<String>,
}

/// `RedisLabs` list of subscriptions in current account
///
/// Response from GET /subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSubscriptions {
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i32>,

    /// List of subscriptions (typically in extra as 'subscriptions' array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscriptions: Option<Vec<Subscription>>,

    /// HATEOAS links for API navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

/// Active active region creation request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveRegionCreateRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Name of region to add as defined by the cloud provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Optional. Enter a VPC identifier that exists in the hosted AWS account. Creates a new VPC if not set. VPC Identifier must be in a valid format and must exist within the hosting account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpc_id: Option<String>,

    /// Deployment CIDR mask. Must be a valid CIDR format with a range of 256 IP addresses.
    pub deployment_cidr: String,

    /// Optional. When 'false': Creates a deployment plan and deploys it, creating any resources required by the plan. When 'true': creates a read-only deployment plan, and does not create any resources. Default: 'false'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,

    /// List of databases in the subscription with local throughput details. Default: 1000 read and write ops/sec for each database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases: Option<Vec<CrdbRegionSpec>>,

    /// Optional. RESP version must be compatible with Redis version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resp_version: Option<String>,

    /// Optional. Resource name of the customer managed key as defined by the cloud provider for customer managed subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_managed_key_resource_name: Option<String>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"CREATE_ACTIVE_ACTIVE_REGION"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// `RedisVersions`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisVersions {
    /// List of Redis versions available for the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_versions: Option<Vec<RedisVersion>>,
}

/// Active active region deletion request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveActiveRegionDeleteRequest {
    /// Subscription ID being updated. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// The names of the regions to delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<ActiveActiveRegionToDelete>>,

    /// Optional. When 'false': Creates a deployment plan and deploys it, deleting any resources required by the plan. When 'true': creates a read-only deployment plan and does not delete or modify any resources. Default: 'false'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,

    /// Read-only on the response; populated by the server with the
    /// operation type (e.g. `"DELETE_ACTIVE_ACTIVE_REGION"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// The names of the regions to delete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveActiveRegionToDelete {
    /// Name of the cloud provider region to delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// The cloud provider region or list of regions (Active-Active only) and networking details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRegionSpec {
    /// Deployment region as defined by the cloud provider.
    pub region: String,

    /// Optional. Support deployment on multiple availability zones within the selected region. Default: 'false'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_availability_zones: Option<bool>,

    /// Optional. List the zone ID(s) for your preferred availability zone(s) for the cloud provider and region. If ‘multipleAvailabilityZones’ is set to 'true', you must list three availability zones. Otherwise, list one availability zone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_availability_zones: Option<Vec<String>>,

    /// Optional. Per-region networking configuration. See [`SubscriptionRegionNetworkingSpec`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking: Option<SubscriptionRegionNetworkingSpec>,
}

/// `SubscriptionMaintenanceWindows`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionMaintenanceWindows {
    /// Maintenance window mode (e.g. `"manual"`, `"automatic"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// Time zone used to interpret window times (e.g. `"UTC"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,

    /// Configured maintenance windows when `mode` is `"manual"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<Vec<MaintenanceWindow>>,

    /// Current skip status for upcoming maintenance windows. See [`MaintenanceWindowSkipStatus`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_status: Option<MaintenanceWindowSkipStatus>,
}

// ============================================================================
// Handler
// ============================================================================

/// Request to replace the resource tags on a Pro subscription.
///
/// Matches the `SubscriptionResourceTagsUpdateRequest` schema. The supplied
/// tags replace all existing tags on the subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionResourceTagsUpdateRequest {
    /// Subscription to update. Server-populated from the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<i32>,

    /// Tags to apply to the subscription. Replaces all existing tags.
    pub resource_tags: Vec<Tag>,

    /// Read-only on the response; populated by the server with the operation
    /// type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_type: Option<String>,
}

/// Handler for Pro subscription operations
///
/// Manages flexible subscriptions with auto-scaling, multi-region support,
/// Active-Active configurations, and advanced networking features.
pub struct SubscriptionHandler {
    client: CloudClient,
}

impl SubscriptionHandler {
    /// Create a new handler
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// Get Pro subscriptions
    ///
    /// Gets a list of all Pro subscriptions in the current account.
    ///
    /// GET /subscriptions
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
    /// let subscriptions = client.subscriptions().get_all_subscriptions().await?;
    ///
    /// // Access subscription data
    /// if let Some(subs) = &subscriptions.subscriptions {
    ///     println!("Found {} subscriptions", subs.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_subscriptions(&self) -> Result<AccountSubscriptions> {
        self.client.get("/subscriptions").await
    }

    /// Create Pro subscription
    /// Creates a new Redis Cloud Pro subscription.
    ///
    /// POST /subscriptions
    pub async fn create_subscription(
        &self,
        request: &SubscriptionCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client.post("/subscriptions", request).await
    }

    /// Get available Redis database versions
    /// Gets a list of all available Redis database versions for Pro subscriptions.
    ///
    /// GET /subscriptions/redis-versions
    pub async fn get_redis_versions(&self, subscription_id: Option<i32>) -> Result<RedisVersions> {
        let mut query = Vec::new();
        if let Some(v) = subscription_id {
            query.push(format!("subscriptionId={v}"));
        }
        let query_string = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.client
            .get(&format!("/subscriptions/redis-versions{query_string}"))
            .await
    }

    /// Delete Pro subscription
    /// Delete the specified Pro subscription. All databases in the subscription must be deleted before deleting it.
    ///
    /// DELETE /subscriptions/{subscriptionId}
    pub async fn delete_subscription_by_id(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        let response = self
            .client
            .delete_raw(&format!("/subscriptions/{subscription_id}"))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Get a single Pro subscription
    ///
    /// Gets information on the specified Pro subscription.
    ///
    /// GET /subscriptions/{subscriptionId}
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
    /// let subscription = client.subscriptions().get_subscription_by_id(123).await?;
    ///
    /// println!("Subscription: {} (status: {:?})",
    ///     subscription.name.unwrap_or_default(),
    ///     subscription.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_subscription_by_id(&self, subscription_id: i32) -> Result<Subscription> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}"))
            .await
    }

    /// Update Pro subscription
    /// Updates the specified Pro subscription.
    ///
    /// PUT /subscriptions/{subscriptionId}
    pub async fn update_subscription(
        &self,
        subscription_id: i32,
        request: &BaseSubscriptionUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/subscriptions/{subscription_id}"), request)
            .await
    }

    /// Get Pro subscription CIDR allowlist
    /// (Self-hosted AWS subscriptions only) Gets a Pro subscription's CIDR allowlist.
    ///
    /// GET /subscriptions/{subscriptionId}/cidr
    pub async fn get_cidr_allowlist(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/cidr"))
            .await
    }

    /// Update Pro subscription CIDR allowlist
    /// (Self-hosted AWS subscriptions only) Updates a Pro subscription's CIDR allowlist.
    ///
    /// PUT /subscriptions/{subscriptionId}/cidr
    pub async fn update_subscription_cidr_allowlist(
        &self,
        subscription_id: i32,
        request: &CidrAllowlistUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(&format!("/subscriptions/{subscription_id}/cidr"), request)
            .await
    }

    /// Get Pro subscription maintenance windows
    /// Gets maintenance windows for the specified Pro subscription.
    ///
    /// GET /subscriptions/{subscriptionId}/maintenance-windows
    pub async fn get_subscription_maintenance_windows(
        &self,
        subscription_id: i32,
    ) -> Result<SubscriptionMaintenanceWindows> {
        self.client
            .get(&format!(
                "/subscriptions/{subscription_id}/maintenance-windows"
            ))
            .await
    }

    /// Update Pro subscription maintenance windows
    /// Updates maintenance windows for the specified Pro subscription.
    ///
    /// PUT /subscriptions/{subscriptionId}/maintenance-windows
    pub async fn update_subscription_maintenance_windows(
        &self,
        subscription_id: i32,
        request: &SubscriptionMaintenanceWindowsSpec,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!("/subscriptions/{subscription_id}/maintenance-windows"),
                request,
            )
            .await
    }

    /// Get Pro subscription pricing
    /// Gets pricing details for the specified Pro subscription.
    ///
    /// GET /subscriptions/{subscriptionId}/pricing
    pub async fn get_subscription_pricing(
        &self,
        subscription_id: i32,
    ) -> Result<SubscriptionPricings> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/pricing"))
            .await
    }

    /// Delete regions from an Active-Active subscription
    /// (Active-Active subscriptions only) Deletes one or more regions from the specified Active-Active subscription.
    ///
    /// DELETE /subscriptions/{subscriptionId}/regions
    pub async fn delete_regions_from_active_active_subscription(
        &self,
        subscription_id: i32,
        request: &ActiveActiveRegionDeleteRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .delete_with_body(
                &format!("/subscriptions/{subscription_id}/regions"),
                serde_json::to_value(request)?,
            )
            .await
    }

    /// Get regions in an Active-Active subscription
    /// (Active-Active subscriptions only) Gets a list of regions in the specified Active-Active subscription.
    ///
    /// GET /subscriptions/{subscriptionId}/regions
    pub async fn get_regions_from_active_active_subscription(
        &self,
        subscription_id: i32,
    ) -> Result<ActiveActiveSubscriptionRegions> {
        self.client
            .get(&format!("/subscriptions/{subscription_id}/regions"))
            .await
    }

    /// Add region to Active-Active subscription
    /// Adds a new region to an Active-Active subscription.
    ///
    /// POST /subscriptions/{subscriptionId}/regions
    pub async fn add_new_region_to_active_active_subscription(
        &self,
        subscription_id: i32,
        request: &ActiveActiveRegionCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .post(
                &format!("/subscriptions/{subscription_id}/regions"),
                request,
            )
            .await
    }

    /// Update Pro subscription resource tags
    /// Replaces all resource tags on the specified Pro subscription with the
    /// supplied set.
    ///
    /// PUT /subscriptions/{subscriptionId}/resource-tags
    pub async fn update_resource_tags(
        &self,
        subscription_id: i32,
        request: &SubscriptionResourceTagsUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .put(
                &format!("/subscriptions/{subscription_id}/resource-tags"),
                request,
            )
            .await
    }

    // ============================================================================
    // Simplified aliases
    // ============================================================================

    /// List Pro subscriptions (simplified)
    ///
    /// Alias for [`get_all_subscriptions`](Self::get_all_subscriptions).
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
    /// let subscriptions = client.subscriptions().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<AccountSubscriptions> {
        self.get_all_subscriptions().await
    }

    /// Create a Pro subscription (simplified)
    ///
    /// Alias for [`create_subscription`](Self::create_subscription).
    ///
    /// # Arguments
    ///
    /// * `request` - The subscription creation request
    pub async fn create(&self, request: &SubscriptionCreateRequest) -> Result<TaskStateUpdate> {
        self.create_subscription(request).await
    }

    /// Delete a Pro subscription (simplified)
    ///
    /// Alias for [`delete_subscription_by_id`](Self::delete_subscription_by_id).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let task = client.subscriptions().delete(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.delete_subscription_by_id(subscription_id).await
    }

    /// Get a Pro subscription by ID (simplified)
    ///
    /// Alias for [`get_subscription_by_id`](Self::get_subscription_by_id).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let subscription = client.subscriptions().get(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, subscription_id: i32) -> Result<Subscription> {
        self.get_subscription_by_id(subscription_id).await
    }

    /// Update a Pro subscription (simplified)
    ///
    /// Alias for [`update_subscription`](Self::update_subscription).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The subscription update request
    pub async fn update(
        &self,
        subscription_id: i32,
        request: &BaseSubscriptionUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.update_subscription(subscription_id, request).await
    }

    /// Get a Pro subscription's CIDR allowlist (simplified)
    ///
    /// Alias for [`get_cidr_allowlist`](Self::get_cidr_allowlist).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let allowlist = client.subscriptions().cidr_allowlist(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cidr_allowlist(&self, subscription_id: i32) -> Result<TaskStateUpdate> {
        self.get_cidr_allowlist(subscription_id).await
    }

    /// Update a Pro subscription's CIDR allowlist (simplified)
    ///
    /// Alias for
    /// [`update_subscription_cidr_allowlist`](Self::update_subscription_cidr_allowlist).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The CIDR allowlist update request
    pub async fn update_cidr_allowlist(
        &self,
        subscription_id: i32,
        request: &CidrAllowlistUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.update_subscription_cidr_allowlist(subscription_id, request)
            .await
    }

    /// Get a Pro subscription's maintenance windows (simplified)
    ///
    /// Alias for
    /// [`get_subscription_maintenance_windows`](Self::get_subscription_maintenance_windows).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let windows = client.subscriptions().maintenance_windows(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn maintenance_windows(
        &self,
        subscription_id: i32,
    ) -> Result<SubscriptionMaintenanceWindows> {
        self.get_subscription_maintenance_windows(subscription_id)
            .await
    }

    /// Update a Pro subscription's maintenance windows (simplified)
    ///
    /// Alias for
    /// [`update_subscription_maintenance_windows`](Self::update_subscription_maintenance_windows).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The maintenance windows specification
    pub async fn update_maintenance_windows(
        &self,
        subscription_id: i32,
        request: &SubscriptionMaintenanceWindowsSpec,
    ) -> Result<TaskStateUpdate> {
        self.update_subscription_maintenance_windows(subscription_id, request)
            .await
    }

    /// Get a Pro subscription's pricing (simplified)
    ///
    /// Alias for [`get_subscription_pricing`](Self::get_subscription_pricing).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let pricing = client.subscriptions().pricing(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pricing(&self, subscription_id: i32) -> Result<SubscriptionPricings> {
        self.get_subscription_pricing(subscription_id).await
    }

    /// Get the regions of an Active-Active subscription (simplified)
    ///
    /// Alias for
    /// [`get_regions_from_active_active_subscription`](Self::get_regions_from_active_active_subscription).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
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
    /// let regions = client.subscriptions().active_active_regions(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn active_active_regions(
        &self,
        subscription_id: i32,
    ) -> Result<ActiveActiveSubscriptionRegions> {
        self.get_regions_from_active_active_subscription(subscription_id)
            .await
    }

    /// Add a region to an Active-Active subscription (simplified)
    ///
    /// Alias for
    /// [`add_new_region_to_active_active_subscription`](Self::add_new_region_to_active_active_subscription).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The region creation request
    pub async fn add_active_active_region(
        &self,
        subscription_id: i32,
        request: &ActiveActiveRegionCreateRequest,
    ) -> Result<TaskStateUpdate> {
        self.add_new_region_to_active_active_subscription(subscription_id, request)
            .await
    }

    /// Delete regions from an Active-Active subscription (simplified)
    ///
    /// Alias for
    /// [`delete_regions_from_active_active_subscription`](Self::delete_regions_from_active_active_subscription).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The region deletion request
    pub async fn delete_active_active_regions(
        &self,
        subscription_id: i32,
        request: &ActiveActiveRegionDeleteRequest,
    ) -> Result<TaskStateUpdate> {
        self.delete_regions_from_active_active_subscription(subscription_id, request)
            .await
    }

    /// Update a Pro subscription's resource tags (simplified)
    ///
    /// Alias for [`update_resource_tags`](Self::update_resource_tags).
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The subscription ID
    /// * `request` - The resource tags update request
    pub async fn update_tags(
        &self,
        subscription_id: i32,
        request: &SubscriptionResourceTagsUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.update_resource_tags(subscription_id, request).await
    }
}
