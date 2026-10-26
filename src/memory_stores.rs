//! Agent Memory store operations.
//!
//! A memory store gives AI agents short-term (session) and long-term memory
//! backed by a Redis Cloud database. Creating, updating, deleting and
//! flushing a store are asynchronous and return a [`TaskStateUpdate`] that can
//! be polled with [`TasksHandler::get_task_by_id`]. Listing, reading and API
//! key management are synchronous.
//!
//! When a synchronous memory store request fails, the Agent Memory service
//! returns an RFC 7807 problem document. The client surfaces it as an ordinary
//! [`CloudError`] whose message is the raw body; use
//! [`MemoryProblemDetails::from_error`] to recover the structured fields.
//!
//! [`TasksHandler::get_task_by_id`]: crate::tasks::TasksHandler::get_task_by_id

use crate::types::TaskStateUpdate;
use crate::{CloudClient, CloudError, Result};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

// ============================================================================
// Enums
// ============================================================================

/// Lifecycle status of a memory store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MemoryStoreStatus {
    /// The store is being provisioned.
    Provisioning,
    /// The store is ready to serve requests.
    Ready,
    /// The store is unavailable.
    Unavailable,
    /// A status added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// How long-term memories are extracted from sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MemoryExtractionStrategy {
    /// LLM tool calling creates, updates and deletes granular memories.
    Instruct,
    /// A strategy added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// What triggers a session summarization cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MemorySummarizationTrigger {
    /// Summarize once the active session reaches an event count.
    EventCount,
    /// A trigger added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// How a model provider credential is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum MemoryCredentialType {
    /// The secret is a provider API key.
    ApiKey,
    /// A credential type added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// What happens to a memory that an exclusion detector matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MemoryDetectorAction {
    /// Redact the matched content (the API default).
    Redact,
    /// Drop the whole memory.
    Drop,
    /// An action added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// Matching technique used by a custom detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MemoryMatcherKind {
    /// Match with an RE2 regular expression.
    Regex,
    /// A matcher kind added by the API after this client release.
    #[serde(other)]
    Unknown,
}

/// Data type of a custom memory type field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MemoryCustomFieldType {
    /// String value.
    #[serde(rename = "str")]
    Str,
    /// Integer value.
    #[serde(rename = "int")]
    Int,
    /// Floating-point value.
    #[serde(rename = "float")]
    Float,
    /// Boolean value.
    #[serde(rename = "bool")]
    Bool,
    /// List of strings.
    #[serde(rename = "list[str]")]
    ListStr,
    /// List of floats.
    #[serde(rename = "list[float]")]
    ListFloat,
    /// Nested object.
    #[serde(rename = "object")]
    Object,
    /// A field type added by the API after this client release.
    #[serde(other)]
    Unknown,
}

// ============================================================================
// Configuration blocks (shared by requests and responses)
// ============================================================================

/// Short-term (working) memory configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreShortMemoryConfig {
    /// Time-to-live of short-term memory entries, in seconds (1 second to 1 year).
    pub ttl_seconds: i32,
}

/// Customer-supplied credential for a model provider. Never returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreModelCredentials {
    /// How the supplied secret is interpreted.
    #[serde(rename = "type")]
    pub credential_type: MemoryCredentialType,

    /// The provider API key. Required on create; on update an omitted value
    /// keeps the current key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

impl MemoryStoreModelCredentials {
    /// An API-key credential.
    #[must_use]
    pub fn api_key(key: impl Into<String>) -> Self {
        Self {
            credential_type: MemoryCredentialType::ApiKey,
            api_key: Some(key.into()),
        }
    }
}

/// Requested model selection for one purpose (LLM or embedding).
///
/// On create, `llm` and `longTermMemory.embedding` must be supplied together
/// to opt the store into customer-selected models.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreModelConfig {
    /// Model provider (e.g. `"openai"`). Immutable after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Model name as advertised by the provider. Required on create.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Credential used to call the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<MemoryStoreModelCredentials>,
}

/// Resolved model in use for one purpose.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreModelStatus {
    /// Model provider in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Model in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Long-term memory configuration accepted on creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreLongTermMemoryCreateConfig {
    /// Time-to-live of long-term memory entries, in seconds (1 second to 1 year).
    pub ttl_seconds: i32,

    /// Embedding model selection. Must be supplied together with `llm`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<MemoryStoreModelConfig>,
}

/// Long-term memory configuration accepted on update. Only the TTL and the
/// embedding credential can change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreLongTermMemoryUpdateConfig {
    /// New time-to-live, in seconds. Omit to leave unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<i32>,

    /// Embedding credential update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<MemoryStoreModelConfig>,
}

/// Long-term memory configuration as stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreLongTermMemoryStatus {
    /// Time-to-live of long-term memory entries, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<i32>,

    /// Embedding model in use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<MemoryStoreModelStatus>,
}

/// Event-count trigger for session summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreSummarizationThresholdConfig {
    /// Number of active session events that triggers a cycle. Must be greater
    /// than `retain_count`.
    pub threshold: i32,

    /// Number of most recent events kept unsummarized after each cycle.
    pub retain_count: i32,
}

/// Session summarization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreSummarizationConfig {
    /// Whether summarization is enabled for the store.
    pub enabled: bool,

    /// What triggers a summarization cycle. Defaults to `event_count`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_strategy: Option<MemorySummarizationTrigger>,

    /// Event-count trigger thresholds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_count: Option<MemoryStoreSummarizationThresholdConfig>,
}

/// How often the long-term memory extraction pipeline runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreExtractionCadenceConfig {
    /// Interval in seconds between extraction runs while a session is active (60-600).
    pub active_interval_seconds: i32,
}

/// Advisory, LLM-mediated exclusion of sensitive content from long-term memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreSemanticExclusionConfig {
    /// Whether semantic exclusion is enabled.
    pub enabled: bool,

    /// Instruction describing what must not be remembered. On update, omit to
    /// keep the stored prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// One selected platform built-in detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreDetectorSelection {
    /// Built-in detector ID: `credit-card`, `email`, `ip-address`, `phone` or `us-ssn`.
    pub id: String,

    /// Whether this detector is active.
    pub enabled: bool,

    /// What happens to a matched memory. Defaults to redact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<MemoryDetectorAction>,
}

/// Deterministic detection using platform-owned built-in detectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreBuiltInDetectorsConfig {
    /// Toggle for the whole built-in group.
    pub enabled: bool,

    /// Selected detectors. On update, omitting keeps the stored selections, a
    /// non-empty list replaces them and an empty list clears them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detectors: Option<Vec<MemoryStoreDetectorSelection>>,
}

/// Regular expression matcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreRegexSpec {
    /// Pattern in RE2 syntax (1-512 characters).
    pub pattern: String,
}

/// How a custom detector decides what to match.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreMatcherSpec {
    /// Matching technique. The block it names must be present.
    pub kind: MemoryMatcherKind,

    /// Regular expression matcher, when `kind` is `regex`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<MemoryStoreRegexSpec>,
}

impl MemoryStoreMatcherSpec {
    /// A regular expression matcher.
    #[must_use]
    pub fn regex(pattern: impl Into<String>) -> Self {
        Self {
            kind: MemoryMatcherKind::Regex,
            regex: Some(MemoryStoreRegexSpec {
                pattern: pattern.into(),
            }),
        }
    }
}

/// One customer-authored detector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCustomDetector {
    /// Detector name, unique within the store.
    pub name: String,

    /// Whether this detector is active.
    pub enabled: bool,

    /// What happens to a matched memory. Defaults to redact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<MemoryDetectorAction>,

    /// How the detector matches content.
    pub matcher: MemoryStoreMatcherSpec,
}

/// Deterministic detection using customer-authored detectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCustomDetectorsConfig {
    /// Toggle for the whole custom group.
    pub enabled: bool,

    /// Authored detectors (at most 32). Omitting on update keeps the stored
    /// detectors, a non-empty list replaces them and an empty list clears them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detectors: Option<Vec<MemoryStoreCustomDetector>>,
}

/// Store-level policy that keeps sensitive data out of long-term memory.
///
/// On update the policy is replaced as a whole, so prefer a read-modify-write
/// of the complete policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreLongTermMemoryExclusions {
    /// Master toggle gating enforcement of all exclusion groups.
    pub enabled: bool,

    /// Semantic (LLM-mediated) exclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic: Option<MemoryStoreSemanticExclusionConfig>,

    /// Built-in detector exclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_in_detectors: Option<MemoryStoreBuiltInDetectorsConfig>,

    /// Custom detector exclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_detectors: Option<MemoryStoreCustomDetectorsConfig>,
}

/// A structured field of a custom memory type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCustomField {
    /// Field name.
    pub name: String,

    /// Human-readable description of the field.
    pub description: String,

    /// Field data type.
    #[serde(rename = "type")]
    pub field_type: MemoryCustomFieldType,
}

/// How memories of a custom type are extracted from session events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCustomExtractionStrategy {
    /// Prompt used to extract memories of this type.
    pub prompt: String,

    /// Whether extraction for this type is active. Defaults to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// A custom category of long-term memory. Name, description and fields are
/// immutable once registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCustomMemoryType {
    /// Unique type name within the store.
    pub name: String,

    /// Human-readable description of what this type captures.
    pub description: String,

    /// Structured fields of a record of this type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<MemoryStoreCustomField>>,

    /// Extraction strategy for this type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_strategy: Option<MemoryStoreCustomExtractionStrategy>,
}

/// Edit of the extraction strategy of an existing custom memory type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreTypeStrategyUpdate {
    /// Name of the existing custom memory type to edit.
    pub type_name: String,

    /// Replacement extraction prompt. Omit to leave unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Replacement enabled flag. Omit to leave unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// A regional endpoint through which the memory store is served.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreEndpoint {
    /// Endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Cloud provider hosting the endpoint (e.g. `"AWS"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Cloud region of the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Egress IP addresses used by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress_ips: Option<Vec<String>>,

    /// Whether the endpoint is reachable from the store's database network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_accessible: Option<bool>,
}

// ============================================================================
// Requests
// ============================================================================

/// Request to create a memory store.
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreCreateRequest {
    /// Memory store name.
    #[builder(setter(into))]
    pub name: String,

    /// ID of the database used by the memory store.
    pub database_id: i32,

    /// Short-term memory configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub short_memory: Option<MemoryStoreShortMemoryConfig>,

    /// Long-term memory configuration. Defaults to a 1 year TTL.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub long_term_memory: Option<MemoryStoreLongTermMemoryCreateConfig>,

    /// LLM selection. Must be supplied together with the embedding model.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub llm: Option<MemoryStoreModelConfig>,

    /// How long-term memories are extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub extraction_strategy: Option<MemoryExtractionStrategy>,

    /// Session summarization configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub summarization: Option<MemoryStoreSummarizationConfig>,

    /// Sensitive-data exclusion policy for long-term memory.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub long_term_memory_exclusions: Option<MemoryStoreLongTermMemoryExclusions>,

    /// Custom memory types to register (at most 3 per store).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub custom_memory_types: Option<Vec<MemoryStoreCustomMemoryType>>,

    /// Extraction pipeline cadence.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub extraction_cadence: Option<MemoryStoreExtractionCadenceConfig>,
}

/// Request to update a memory store. Every field is optional; omitted fields
/// are left unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreUpdateRequest {
    /// New memory store name (1-64 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,

    /// Short-term memory configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub short_memory: Option<MemoryStoreShortMemoryConfig>,

    /// Long-term memory TTL and embedding credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub long_term_memory: Option<MemoryStoreLongTermMemoryUpdateConfig>,

    /// LLM model or credential update within the stored provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub llm: Option<MemoryStoreModelConfig>,

    /// Session summarization configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub summarization: Option<MemoryStoreSummarizationConfig>,

    /// Replacement sensitive-data exclusion policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub long_term_memory_exclusions: Option<MemoryStoreLongTermMemoryExclusions>,

    /// New custom memory types to append to the store's registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub add_custom_memory_types: Option<Vec<MemoryStoreCustomMemoryType>>,

    /// Extraction strategy edits for existing custom memory types.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub update_custom_memory_type_strategies: Option<Vec<MemoryStoreTypeStrategyUpdate>>,

    /// Extraction pipeline cadence.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub extraction_cadence: Option<MemoryStoreExtractionCadenceConfig>,
}

/// Request to create a memory store API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreApiKeyCreateRequest {
    /// Name of the API key (no whitespace, at most 100 characters).
    pub key_name: String,
}

impl MemoryStoreApiKeyCreateRequest {
    /// Create a request for a key with the given name.
    #[must_use]
    pub fn new(key_name: impl Into<String>) -> Self {
        Self {
            key_name: key_name.into(),
        }
    }
}

// ============================================================================
// Responses
// ============================================================================

/// Summary of a memory store, as returned by the list operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreSummary {
    /// Memory store ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,

    /// Memory store name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Creation timestamp (ISO-8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Backing database ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<String>,

    /// Backing database name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,

    /// Subscription that owns the backing database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    /// How long-term memories are extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_strategy: Option<MemoryExtractionStrategy>,

    /// Session summarization configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarization: Option<MemoryStoreSummarizationConfig>,

    /// LLM in use, for stores with an explicit model selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<MemoryStoreModelStatus>,

    /// Lifecycle status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MemoryStoreStatus>,
}

/// List of memory stores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreListResponse {
    /// Memory stores visible to the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<MemoryStoreSummary>>,
}

/// Memory store configuration and status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStore {
    /// Memory store ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,

    /// Memory store name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Creation timestamp (ISO-8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// Backing database ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<String>,

    /// Backing database name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_name: Option<String>,

    /// Cloud provider of the backing database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_provider: Option<String>,

    /// Region of the backing database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_region: Option<String>,

    /// CIDRs allowed to access the backing database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_access_cidrs: Option<Vec<String>>,

    /// Subscription that owns the backing database.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    /// Short-term memory configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_memory: Option<MemoryStoreShortMemoryConfig>,

    /// Long-term memory configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_term_memory: Option<MemoryStoreLongTermMemoryStatus>,

    /// LLM in use, for stores with an explicit model selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<MemoryStoreModelStatus>,

    /// Primary endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Regional endpoints serving the store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<MemoryStoreEndpoint>>,

    /// How long-term memories are extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_strategy: Option<MemoryExtractionStrategy>,

    /// Session summarization configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarization: Option<MemoryStoreSummarizationConfig>,

    /// Sensitive-data exclusion policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_term_memory_exclusions: Option<MemoryStoreLongTermMemoryExclusions>,

    /// Registered custom memory types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_memory_types: Option<Vec<MemoryStoreCustomMemoryType>>,

    /// Extraction pipeline cadence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_cadence: Option<MemoryStoreExtractionCadenceConfig>,

    /// Lifecycle status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MemoryStoreStatus>,

    /// Most recent error, when the store is unhealthy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Time of the most recent error, as epoch seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_timestamp: Option<f64>,
}

/// A newly created memory store API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStoreApiKeyCreateResponse {
    /// Generated API key. Returned only once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

// ============================================================================
// Errors
// ============================================================================

/// A single field validation failure in a [`MemoryProblemDetails`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProblemFieldError {
    /// Path of the offending request field (e.g. `shortMemory.ttlSeconds`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,

    /// Validation rule that failed (e.g. `min`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,

    /// Human-readable message for the failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// RFC 7807 problem details returned by the Agent Memory service when a
/// synchronous memory store request fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProblemDetails {
    /// Stable, machine-readable error type (e.g. `/errors/resource-not-found`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub problem_type: Option<String>,

    /// Short human-readable summary of the error type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// HTTP status code, repeated in the body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,

    /// Explanation specific to this occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Identifier of the specific occurrence, when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Per-field validation failures for `/errors/invalid-data` responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<MemoryProblemFieldError>>,
}

impl MemoryProblemDetails {
    /// Recover the problem document carried by a failed memory store call.
    ///
    /// The client maps every failed response onto [`CloudError`] by status
    /// code and keeps the raw body as the message. This parses that body back
    /// into a problem document, returning `None` when the error carries no
    /// body or the body is not a problem document.
    ///
    /// ```
    /// use redis_cloud::CloudError;
    /// use redis_cloud::memory_stores::MemoryProblemDetails;
    ///
    /// let error = CloudError::ApiError {
    ///     code: 424,
    ///     message: r#"{"type":"/errors/resource-not-found","status":404}"#.to_string(),
    /// };
    /// let problem = MemoryProblemDetails::from_error(&error).unwrap();
    /// assert_eq!(problem.problem_type.as_deref(), Some("/errors/resource-not-found"));
    /// ```
    #[must_use]
    pub fn from_error(error: &CloudError) -> Option<Self> {
        let body = match error {
            CloudError::BadRequest { message }
            | CloudError::AuthenticationFailed { message }
            | CloudError::Forbidden { message }
            | CloudError::NotFound { message }
            | CloudError::RateLimited { message }
            | CloudError::InternalServerError { message }
            | CloudError::ServiceUnavailable { message }
            | CloudError::ApiError { message, .. } => message,
            _ => return None,
        };
        let problem: Self = serde_json::from_str(body).ok()?;
        // Any JSON object deserializes into an all-optional struct; require at
        // least one RFC 7807 member so unrelated error bodies are not mistaken
        // for problem documents.
        (problem.problem_type.is_some() || problem.title.is_some() || problem.detail.is_some())
            .then_some(problem)
    }
}

// ============================================================================
// Handler
// ============================================================================

/// Handler for Agent Memory store operations.
pub struct MemoryStoreHandler {
    client: CloudClient,
}

impl MemoryStoreHandler {
    /// Create a memory store handler.
    #[must_use]
    pub fn new(client: CloudClient) -> Self {
        Self { client }
    }

    /// List memory stores.
    pub async fn list(&self) -> Result<MemoryStoreListResponse> {
        self.client.get("/memory-stores").await
    }

    /// Create a memory store. Returns a task to poll.
    pub async fn create(&self, request: &MemoryStoreCreateRequest) -> Result<TaskStateUpdate> {
        self.client.post("/memory-stores", request).await
    }

    /// Get a memory store's configuration and status.
    pub async fn get(&self, store_id: &str) -> Result<MemoryStore> {
        self.client.get(&format!("/memory-stores/{store_id}")).await
    }

    /// Update a memory store. Returns a task to poll.
    pub async fn update(
        &self,
        store_id: &str,
        request: &MemoryStoreUpdateRequest,
    ) -> Result<TaskStateUpdate> {
        self.client
            .patch(&format!("/memory-stores/{store_id}"), request)
            .await
    }

    /// Delete a memory store. Returns a task to poll.
    ///
    /// `flush` controls whether the store's data is flushed from the backing
    /// database as part of the deletion; `None` uses the API default.
    pub async fn delete(&self, store_id: &str, flush: Option<bool>) -> Result<TaskStateUpdate> {
        let query = flush.map(|f| format!("?flush={f}")).unwrap_or_default();
        self.client
            .delete_typed(&format!("/memory-stores/{store_id}{query}"))
            .await
    }

    /// Delete all long-term memory entries in a store. Returns a task to poll.
    pub async fn flush_entries(&self, store_id: &str) -> Result<TaskStateUpdate> {
        self.client
            .delete_typed(&format!("/memory-stores/{store_id}/entries"))
            .await
    }

    /// Delete all session (short-term) memory in a store. Returns a task to poll.
    pub async fn empty_session_memory(&self, store_id: &str) -> Result<TaskStateUpdate> {
        self.client
            .delete_typed(&format!("/memory-stores/{store_id}/session-memory"))
            .await
    }

    /// List a memory store's API keys.
    ///
    /// The spec does not document a success body for this operation, so the
    /// response is returned as raw JSON.
    pub async fn list_api_keys(&self, store_id: &str) -> Result<serde_json::Value> {
        self.client
            .get_raw(&format!("/memory-stores/{store_id}/api-keys"))
            .await
    }

    /// Create a memory store API key. The key is returned only once.
    pub async fn create_api_key(
        &self,
        store_id: &str,
        request: &MemoryStoreApiKeyCreateRequest,
    ) -> Result<MemoryStoreApiKeyCreateResponse> {
        self.client
            .post(&format!("/memory-stores/{store_id}/api-keys"), request)
            .await
    }

    /// Delete a memory store API key.
    pub async fn delete_api_key(&self, store_id: &str, api_key_id: &str) -> Result<()> {
        self.client
            .delete(&format!("/memory-stores/{store_id}/api-keys/{api_key_id}"))
            .await
    }
}
