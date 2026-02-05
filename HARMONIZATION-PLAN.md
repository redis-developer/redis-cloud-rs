# redis-cloud Harmonization Plan

Goal: Align redis-cloud with redis-enterprise patterns for consistency.

## Changes Overview

### 1. Add TypedBuilder to Request Types

**Priority: High**

Currently request types use manual `Option<T>` fields:
```rust
pub struct DatabaseCreateRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_in_gb: Option<f64>,
    // ... many more
}
```

Change to TypedBuilder pattern (matching redis-enterprise):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct DatabaseCreateRequest {
    #[builder(setter(into))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub memory_limit_in_gb: Option<f64>,
    // ...
}
```

**Files to update:**
- [ ] `src/flexible/databases.rs` - `DatabaseCreateRequest`, `DatabaseUpdateRequest`, `DatabaseImportRequest`, etc.
- [ ] `src/flexible/subscriptions.rs` - `SubscriptionCreateRequest`, `SubscriptionUpdateRequest`, etc.
- [ ] `src/fixed/databases.rs` - Fixed plan request types
- [ ] `src/fixed/subscriptions.rs` - Fixed plan request types
- [ ] `src/acl.rs` - ACL request types
- [ ] `src/users.rs` - User request types
- [ ] `src/connectivity/*.rs` - VPC peering, PSC, etc. request types

**Dependencies to add:**
```toml
typed-builder = "0.18"
```

### 2. Add Error Helper Methods

**Priority: High**

Currently CloudError only has `is_retryable()`. Add helpers matching redis-enterprise:

```rust
impl CloudError {
    pub fn is_retryable(&self) -> bool { /* existing */ }

    // Add these:
    pub fn is_not_found(&self) -> bool {
        matches!(self, CloudError::NotFound { .. })
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self, CloudError::AuthenticationFailed { .. } | CloudError::Forbidden { .. })
    }

    pub fn is_server_error(&self) -> bool {
        matches!(self, CloudError::InternalServerError { .. } | CloudError::ServiceUnavailable { .. })
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, CloudError::ConnectionError(msg) if msg.contains("timeout"))
    }

    pub fn is_rate_limited(&self) -> bool {
        matches!(self, CloudError::RateLimited { .. })
    }

    pub fn is_conflict(&self) -> bool {
        matches!(self, CloudError::PreconditionFailed)
    }

    pub fn is_bad_request(&self) -> bool {
        matches!(self, CloudError::BadRequest { .. })
    }
}
```

**Files to update:**
- [ ] `src/error.rs`

### 3. Consistent Handler Method Naming

**Priority: Medium**

Align method names where semantically equivalent:

| Operation | Current (Cloud) | Target | Enterprise |
|-----------|-----------------|--------|------------|
| List DBs | `get_subscription_databases` | `list` | `list` |
| Get DB | `get_subscription_database_by_id` | `get` | `get` / `info` |
| Create DB | `create_database` | `create` | `create` |
| Update DB | `update_database` | `update` | `update` |
| Delete DB | `delete_database_by_id` | `delete` | `delete` |

**Note:** Keep subscription_id as parameter (architectural difference), but simplify method names.

**Approach:** Add new methods with simpler names, deprecate old ones:
```rust
impl DatabaseHandler {
    /// List databases in a subscription
    pub async fn list(&self, subscription_id: i32) -> Result<Vec<Database>> {
        // Unwrap the nested response internally
        let response = self.get_subscription_databases(subscription_id, None, None).await?;
        Ok(Self::extract_databases_from_response(&response))
    }

    #[deprecated(since = "0.10.0", note = "Use `list` instead")]
    pub async fn get_subscription_databases(...) -> Result<AccountSubscriptionDatabases> {
        // existing implementation
    }
}
```

**Files to update:**
- [ ] `src/flexible/databases.rs`
- [ ] `src/flexible/subscriptions.rs`
- [ ] `src/fixed/databases.rs`
- [ ] `src/fixed/subscriptions.rs`

### 4. Simplify Return Types (New Methods)

**Priority: Medium**

Add simplified methods that unwrap nested containers:

```rust
// Current: Returns nested wrapper
pub async fn get_subscription_databases(...) -> Result<AccountSubscriptionDatabases>

// Add: Returns direct vector
pub async fn list(&self, subscription_id: i32) -> Result<Vec<Database>>
```

Keep original methods for backward compatibility but encourage new patterns.

### 5. Add Timeout Error Variant

**Priority: Low**

Enterprise has explicit `Timeout` variant. Consider adding to Cloud:

```rust
pub enum CloudError {
    // ... existing variants

    /// Request timed out
    Timeout,
}
```

**Files to update:**
- [ ] `src/error.rs`
- [ ] `src/client.rs` (map timeout errors)

## Non-Changes (Keep as Layer 2 Concern)

These are architectural differences that Layer 2 should handle:

- **Async task model** - `TaskStateUpdate` returns are by design (Cloud API is async)
- **Subscription context** - Required parameter is architectural (multi-tenant)
- **Nested response wrappers** - Original methods preserved for full API access

## Testing

- [ ] Ensure all existing tests pass
- [ ] Add tests for new TypedBuilder usage
- [ ] Add tests for error helper methods
- [ ] Add tests for new simplified methods

## Version

This will be a **minor version bump** (e.g., 0.9.x -> 0.10.0) due to:
- New dependency (typed-builder)
- New methods
- Deprecations (non-breaking)
