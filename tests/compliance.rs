//! API compliance harness — validates the typed client against the **live**
//! Redis Cloud API across the spec's full operation surface, and tracks drift
//! over time against a committed baseline.
//!
//! The live API is the reference: for each operation we check that the real
//! response deserializes into our typed model with **no hard error** and **no
//! silently-dropped fields** (the round-trip check that caught #121/#128/#130).
//! Every spec operation that has no check yet is reported as `Uncovered`, so
//! the matrix shows 100% of the surface even before we cover 100% of it.
//!
//! ## Running
//!
//! ```bash
//! REDIS_CLOUD_API_KEY=... REDIS_CLOUD_API_SECRET=... \
//!   cargo test --test compliance -- --ignored --nocapture
//! ```
//!
//! Also needs the `REDIS_CLOUD_TEST_*` resource ids (see the gitignored `.env`)
//! for per-resource operations. Run outside a sandbox that blocks the network.
//!
//! ## Drift gate
//!
//! The run is compared against `tests/fixtures/compliance_baseline.json`. Any
//! difference (a new `Fail`/`Drift`, a newly-covered op, changed dropped-field
//! set, …) fails the test until the baseline is deliberately re-blessed:
//!
//! ```bash
//! COMPLIANCE_BLESS=1 cargo test --test compliance -- --ignored --nocapture
//! ```
//!
//! ## Tiers
//!
//! The matrix is fully classified — every operation in the bundled spec has a
//! status, no `Uncovered`:
//!
//! - **T1 reads** (every GET): `Pass` / `Drift` / `KnownDiff` (an endpoint the
//!   API 404s/500s) / `Skip` (needs Active-Active or configured connectivity,
//!   or a non-JSON body).
//! - **T2 non-destructive writes**: reversible, self-cleaning lifecycles —
//!   database tags (Pro + Essentials), an ACL redis-rule, and subscription
//!   renames — validating request serialization and response shape.
//! - **T4 destructive / mutating writes** (create/delete of subscriptions,
//!   databases, cloud accounts; flush; import; upgrade; …) and
//!   connectivity/Active-Active writes are `Skip`ped pending the deliberate,
//!   guarded destructive pass.

#![allow(clippy::type_complexity)]

use redis_cloud::CloudClient;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;

const BASELINE_PATH: &str = "tests/fixtures/compliance_baseline.json";
const SPEC: &str = include_str!("fixtures/cloud_openapi.json");
const HTTP_METHODS: &[&str] = &[
    "get", "put", "post", "delete", "patch", "head", "options", "trace",
];

// ---------------------------------------------------------------------------
// Status + report
// ---------------------------------------------------------------------------

/// Compliance status for a single operation. Serialized into the baseline, so
/// only stable detail is retained (drift carries its sorted dropped-key paths;
/// a hard `Fail` carries only its kind, since serde error strings are volatile).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum Status {
    /// Response deserialized and round-tripped with no dropped fields.
    Pass,
    /// Deserialized, but these non-null response keys were dropped by the model.
    Drift { dropped: Vec<String> },
    /// Hard failure (deserialize error, or request rejected).
    Fail,
    /// Documented intentional divergence (not a bug).
    KnownDiff { note: String },
    /// Not exercised yet, with a reason (e.g. destructive, needs setup).
    Skip { reason: String },
    /// A spec operation with no check registered yet.
    Uncovered,
}

impl Status {
    fn label(&self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Drift { .. } => "DRIFT",
            Status::Fail => "FAIL",
            Status::KnownDiff { .. } => "KNOWN-DIFF",
            Status::Skip { .. } => "SKIP",
            Status::Uncovered => "UNCOVERED",
        }
    }
}

type Matrix = BTreeMap<String, Status>;

fn key(method: &str, spec_path: &str) -> String {
    format!("{method} {spec_path}")
}

// ---------------------------------------------------------------------------
// Round-trip drift engine
// ---------------------------------------------------------------------------

/// Walk the real response and the re-serialized model in parallel, collecting
/// keys present (and non-null) in the response but absent from the model output
/// — i.e. fields the model silently dropped.
fn collect_missing(raw: &Value, got: &Value, path: &str, out: &mut Vec<String>) {
    match (raw, got) {
        (Value::Object(rm), Value::Object(gm)) => {
            for (k, rv) in rm {
                if rv.is_null() {
                    continue;
                }
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match gm.get(k) {
                    None => out.push(p),
                    Some(gv) => collect_missing(rv, gv, &p, out),
                }
            }
        }
        (Value::Array(ra), Value::Array(ga)) => {
            if let (Some(r), Some(g)) = (ra.first(), ga.first()) {
                collect_missing(r, g, &format!("{path}[0]"), out);
            }
        }
        _ => {}
    }
}

/// Deserialize `raw` into `T`, re-serialize, and report dropped keys. `Err` =
/// hard deserialize failure.
fn roundtrip<T: DeserializeOwned + Serialize>(raw: &Value) -> Result<Vec<String>, String> {
    let model: T = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
    let got = serde_json::to_value(&model).map_err(|e| e.to_string())?;
    let mut dropped = Vec::new();
    collect_missing(raw, &got, "", &mut dropped);
    dropped.sort();
    Ok(dropped)
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

/// Response-compliance check for a read: fetch raw, deserialize into `T`,
/// round-trip, classify.
async fn check<T: DeserializeOwned + Serialize>(
    m: &mut Matrix,
    c: &CloudClient,
    method: &str,
    spec_path: &str,
    live_path: &str,
) {
    let status = match c.get_raw(live_path).await {
        Ok(raw) => match roundtrip::<T>(&raw) {
            Ok(dropped) if dropped.is_empty() => Status::Pass,
            Ok(dropped) => Status::Drift { dropped },
            Err(e) => {
                eprintln!("  [FAIL detail] {method} {spec_path}: {e}");
                Status::Fail
            }
        },
        Err(e) => {
            eprintln!("  [FAIL detail] {method} {spec_path}: {e}");
            Status::Fail
        }
    };
    m.insert(key(method, spec_path), status);
}

/// Like [`check`], but an API error whose message contains `tolerate` is a
/// documented known-diff rather than a failure — e.g. the spec lists
/// `GET .../traffic` but the API 404s it for an active database, or
/// `GET /fixed/redis-versions` returns a server-side 500. A success still
/// round-trips normally, so a later API fix shows up as a status change.
async fn check_tolerating<T: DeserializeOwned + Serialize>(
    m: &mut Matrix,
    c: &CloudClient,
    method: &str,
    spec_path: &str,
    live_path: &str,
    tolerate: &str,
    note: &str,
) {
    let status = match c.get_raw(live_path).await {
        Ok(raw) => match roundtrip::<T>(&raw) {
            Ok(dropped) if dropped.is_empty() => Status::Pass,
            Ok(dropped) => Status::Drift { dropped },
            Err(_) => Status::Fail,
        },
        Err(e) if e.to_string().contains(tolerate) => Status::KnownDiff {
            note: note.to_string(),
        },
        Err(e) => {
            eprintln!("  [FAIL detail] {method} {spec_path}: {e}");
            Status::Fail
        }
    };
    m.insert(key(method, spec_path), status);
}

/// Record an intentionally-skipped operation (e.g. needs Active-Active / PSC
/// setup, or a non-JSON body) with a reason.
fn skip(m: &mut Matrix, method: &str, spec_path: &str, reason: &str) {
    m.insert(
        key(method, spec_path),
        Status::Skip {
            reason: reason.to_string(),
        },
    );
}

/// Fetch a list endpoint and extract an id to drill into a by-id operation.
async fn discover(
    c: &CloudClient,
    list_path: &str,
    extract: impl Fn(&Value) -> Option<String>,
) -> Option<String> {
    extract(&c.get_raw(list_path).await.ok()?)
}

/// Classify the raw result of a (non-destructive, reversible) write: `Ok` means
/// the API accepted the request body; the response is then round-tripped into
/// `T` to validate the response shape. `Err` is a `Fail` (rejected request or
/// deserialize error).
async fn record_write<T: DeserializeOwned + Serialize>(
    m: &mut Matrix,
    method: &str,
    spec_path: &str,
    result: Result<Value, redis_cloud::CloudError>,
) {
    let status = match result {
        Ok(raw) => match roundtrip::<T>(&raw) {
            Ok(dropped) if dropped.is_empty() => Status::Pass,
            Ok(dropped) => Status::Drift { dropped },
            Err(e) => {
                eprintln!("  [FAIL detail] {method} {spec_path}: {e}");
                Status::Fail
            }
        },
        Err(e) => {
            eprintln!("  [FAIL detail] {method} {spec_path}: {e}");
            Status::Fail
        }
    };
    m.insert(key(method, spec_path), status);
}

/// Exercise the reversible tag surface without replacing unrelated tags.
/// Failure to snapshot the current collection fails closed: the baseline will
/// see these operations change to `Skip`, and no write is attempted.
async fn run_tag_lifecycle(
    m: &mut Matrix,
    c: &CloudClient,
    live_base: &str,
    spec_base: &str,
    spec_item: &str,
) {
    const TEST_KEY: &str = "rcrs-compliance";
    let preserved = match c.get_raw(live_base).await {
        Ok(raw) => raw.get("tags").and_then(Value::as_array).map(|tags| {
            tags.iter()
                .filter(|tag| tag.get("key").and_then(Value::as_str) != Some(TEST_KEY))
                .cloned()
                .collect::<Vec<_>>()
        }),
        Err(error) => {
            eprintln!("  [FAIL detail] GET {spec_base}: {error}");
            None
        }
    };
    let Some(mut replacement) = preserved else {
        for (method, path) in [
            ("POST", spec_base),
            ("PUT", spec_item),
            ("PUT", spec_base),
            ("DELETE", spec_item),
        ] {
            skip(m, method, path, "could not snapshot existing tags safely");
        }
        return;
    };

    let live_item = format!("{live_base}/{TEST_KEY}");
    let _ = c.delete_raw(&live_item).await;

    let result = c
        .post_raw(
            live_base,
            serde_json::json!({"key": TEST_KEY, "value": "v1"}),
        )
        .await;
    record_write::<redis_cloud::types::CloudTag>(m, "POST", spec_base, result).await;

    let result = c
        .put_raw(&live_item, serde_json::json!({"value": "v2"}))
        .await;
    record_write::<redis_cloud::types::CloudTag>(m, "PUT", spec_item, result).await;

    replacement.push(serde_json::json!({"key": TEST_KEY, "value": "v3"}));
    let result = c
        .put_raw(live_base, serde_json::json!({"tags": replacement}))
        .await;
    record_write::<redis_cloud::types::CloudTags>(m, "PUT", spec_base, result).await;

    let result = c.delete_raw(&live_item).await;
    record_write::<Value>(m, "DELETE", spec_item, result).await;
}

/// Auto-classify every write operation not explicitly wired above as a `Skip`,
/// with a reason: connectivity/Active-Active ops need resources we don't have;
/// everything else is a mutating/destructive op deferred to the systematic
/// write pass. Keeps the matrix fully classified (no `Uncovered` writes).
fn classify_remaining_writes(m: &mut Matrix) {
    let conn = [
        "peering",
        "private-link",
        "private-service-connect",
        "transitGateway",
        "/regions",
    ];
    for (method, path) in spec_operations() {
        if method == "GET" {
            continue;
        }
        let k = key(&method, &path);
        if m.contains_key(&k) {
            continue;
        }
        let reason = if conn.iter().any(|p| path.contains(p)) {
            "needs Active-Active / configured connectivity"
        } else {
            "mutating/destructive — deferred to the systematic write pass (T4)"
        };
        m.insert(
            k,
            Status::Skip {
                reason: reason.to_string(),
            },
        );
    }
}

/// [`check_tolerating`] specialized to a tolerated `NotFound` (the common case).
async fn check_known_404<T: DeserializeOwned + Serialize>(
    m: &mut Matrix,
    c: &CloudClient,
    method: &str,
    spec_path: &str,
    live_path: &str,
    note: &str,
) {
    check_tolerating::<T>(m, c, method, spec_path, live_path, "Not Found", note).await;
}

// ---------------------------------------------------------------------------
// Spec enumeration + baseline
// ---------------------------------------------------------------------------

/// Every `METHOD path` operation in the bundled spec.
fn spec_operations() -> Vec<(String, String)> {
    let spec: Value = serde_json::from_str(SPEC).expect("bundled spec should parse");
    let mut ops = Vec::new();
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            if let Some(obj) = item.as_object() {
                for m in obj.keys() {
                    if HTTP_METHODS.contains(&m.as_str()) {
                        let path = match path.strip_prefix("/v1") {
                            Some(rest) if rest.is_empty() || rest.starts_with('/') => rest,
                            _ => path,
                        };
                        ops.push((m.to_uppercase(), path.to_string()));
                    }
                }
            }
        }
    }
    ops
}

#[test]
fn bundled_spec_operations_are_base_relative() {
    let ops = spec_operations();
    assert!(!ops.is_empty());
    assert!(ops.iter().all(|(_, path)| !path.starts_with("/v1/")));
}

#[tokio::test]
async fn tag_lifecycle_preserves_unrelated_tags() {
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let live_base = "/subscriptions/1/databases/2/tags";
    let live_item = "/subscriptions/1/databases/2/tags/rcrs-compliance";

    Mock::given(method("GET"))
        .and(path(live_base))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tags": [
                {"key": "keep", "value": "original"},
                {"key": "rcrs-compliance", "value": "stale"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(live_item))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(live_base))
        .and(body_json(
            serde_json::json!({"key": "rcrs-compliance", "value": "v1"}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"key": "rcrs-compliance", "value": "v1"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(live_item))
        .and(body_json(serde_json::json!({"value": "v2"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"key": "rcrs-compliance", "value": "v2"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(live_base))
        .and(body_json(serde_json::json!({
            "tags": [
                {"key": "keep", "value": "original"},
                {"key": "rcrs-compliance", "value": "v3"}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tags": [
                {"key": "keep", "value": "original"},
                {"key": "rcrs-compliance", "value": "v3"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = CloudClient::builder()
        .api_key("test-key")
        .api_secret("test-secret")
        .base_url(server.uri())
        .build()
        .expect("mock client should build");
    let mut matrix = Matrix::new();
    let spec_base = "/subscriptions/{subscriptionId}/databases/{databaseId}/tags";
    let spec_item = "/subscriptions/{subscriptionId}/databases/{databaseId}/tags/{tagKey}";

    run_tag_lifecycle(&mut matrix, &client, live_base, spec_base, spec_item).await;

    for operation in [
        key("POST", spec_base),
        key("PUT", spec_item),
        key("PUT", spec_base),
        key("DELETE", spec_item),
    ] {
        assert_eq!(matrix.get(&operation), Some(&Status::Pass));
    }
}

/// Add `Uncovered` for every spec op without a registered check, and detect
/// registered ops that don't exist in the spec (typos / non-spec routes).
fn reconcile_with_spec(m: &mut Matrix) {
    let spec: std::collections::BTreeSet<String> = spec_operations()
        .into_iter()
        .map(|(meth, path)| key(&meth, &path))
        .collect();

    let registered: Vec<String> = m.keys().cloned().collect();
    for r in &registered {
        assert!(
            spec.contains(r),
            "registered compliance check {r:?} does not match any spec operation \
             (typo, or a non-spec route)"
        );
    }
    for op in spec {
        m.entry(op).or_insert(Status::Uncovered);
    }
}

fn print_report(m: &Matrix) {
    use std::collections::BTreeMap as Counts;
    let mut counts: Counts<&str, usize> = Counts::new();
    println!("\n=== API compliance matrix ===");
    for (op, status) in m {
        *counts.entry(status.label()).or_default() += 1;
        match status {
            Status::Pass | Status::Uncovered => {}
            Status::Drift { dropped } => {
                println!("  DRIFT       {op}  (dropped: {})", dropped.join(", "))
            }
            Status::Fail => println!("  FAIL        {op}"),
            Status::KnownDiff { note } => println!("  KNOWN-DIFF  {op}  ({note})"),
            Status::Skip { reason } => println!("  SKIP        {op}  ({reason})"),
        }
    }
    let total = m.len();
    let summary: Vec<String> = ["PASS", "DRIFT", "FAIL", "KNOWN-DIFF", "SKIP", "UNCOVERED"]
        .iter()
        .map(|k| format!("{}={}", k, counts.get(k).copied().unwrap_or(0)))
        .collect();
    let summary = format!("{total} operations: {}", summary.join(" "));
    println!("--- {summary} ---");

    // Scheduled CI may request an aggregate-only artifact. This deliberately
    // contains no operation paths, response bodies, or captured live values.
    if let Ok(path) = std::env::var("COMPLIANCE_SUMMARY_PATH") {
        std::fs::write(path, format!("{summary}\n")).expect("write compliance summary");
    }
}

fn load_baseline() -> Option<Matrix> {
    let raw = std::fs::read_to_string(BASELINE_PATH).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_baseline(m: &Matrix) {
    let json = serde_json::to_string_pretty(m).expect("serialize baseline");
    std::fs::write(BASELINE_PATH, json + "\n").expect("write baseline");
}

/// Diff the current matrix against the baseline; return human-readable change
/// lines (empty == in sync).
fn diff(baseline: &Matrix, current: &Matrix) -> Vec<String> {
    let mut changes = Vec::new();
    let mut keys: std::collections::BTreeSet<&String> = baseline.keys().collect();
    keys.extend(current.keys());
    for k in keys {
        match (baseline.get(k), current.get(k)) {
            (Some(b), Some(c)) if b != c => {
                changes.push(format!("  CHANGED  {k}: {} -> {}", b.label(), c.label()))
            }
            (None, Some(c)) => changes.push(format!("  NEW      {k}: {}", c.label())),
            (Some(b), None) => changes.push(format!("  REMOVED  {k}: was {}", b.label())),
            _ => {}
        }
    }
    changes
}

// ---------------------------------------------------------------------------
// Env helpers (mirrors live_integration.rs)
// ---------------------------------------------------------------------------

fn client() -> Option<CloudClient> {
    let key = std::env::var("REDIS_CLOUD_API_KEY")
        .or_else(|_| std::env::var("REDIS_CLOUD_API_ACCOUNT_KEY"))
        .ok()?;
    let secret = std::env::var("REDIS_CLOUD_API_SECRET")
        .or_else(|_| std::env::var("REDIS_CLOUD_API_USER_KEY"))
        .ok()?;
    Some(
        CloudClient::builder()
            .api_key(key)
            .api_secret(secret)
            .build()
            .expect("client should build"),
    )
}

fn env_i32(k: &str) -> Option<i32> {
    std::env::var(k).ok()?.parse().ok()
}

fn canonical_test_subscription_name(name: &str) -> String {
    let mut name = name.to_string();
    loop {
        let Some(stripped) = ["-rcrs-upd-test", "-rcrs-compliance"]
            .iter()
            .find_map(|suffix| name.strip_suffix(suffix))
        else {
            return name;
        };
        name = stripped.to_string();
    }
}

// ---------------------------------------------------------------------------
// The harness
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "live API compliance; run with --ignored (needs creds + REDIS_CLOUD_TEST_* ids)"]
async fn api_compliance() {
    let Some(c) = client() else {
        eprintln!("SKIP api_compliance: no credentials in env");
        return;
    };
    let (Some(pro_sub), Some(pro_db), Some(ess_sub), Some(ess_db)) = (
        env_i32("REDIS_CLOUD_TEST_PRO_SUB_ID"),
        env_i32("REDIS_CLOUD_TEST_PRO_DB_ID"),
        env_i32("REDIS_CLOUD_TEST_ESSENTIALS_SUB_ID"),
        env_i32("REDIS_CLOUD_TEST_ESSENTIALS_DB_ID"),
    ) else {
        eprintln!("SKIP api_compliance: REDIS_CLOUD_TEST_* resource ids not set");
        return;
    };

    use redis_cloud::account::{
        DataPersistenceOptions, ModulesData, PaymentMethods, Regions, RootAccount,
    };
    use redis_cloud::acl::{AccountACLRedisRules, AccountACLRoles, AccountACLUsers};
    use redis_cloud::cloud_accounts::CloudAccounts;
    use redis_cloud::databases::{AccountSubscriptionDatabases, Database};
    use redis_cloud::fixed_databases::{AccountFixedSubscriptionDatabases, FixedDatabase};
    use redis_cloud::fixed_subscriptions::{FixedSubscription, FixedSubscriptions};
    use redis_cloud::subscriptions::{
        AccountSubscriptions, Subscription, SubscriptionMaintenanceWindows, SubscriptionPricings,
    };
    use redis_cloud::types::{
        CloudTags, DatabaseTrafficStateResponse, TaskStateUpdate, TasksStateUpdate,
    };
    use redis_cloud::users::AccountUsers;

    let mut m = Matrix::new();

    // -- T1: account-level reads (no path params) --
    check::<RootAccount>(&mut m, &c, "GET", "/", "/").await;
    check::<PaymentMethods>(&mut m, &c, "GET", "/payment-methods", "/payment-methods").await;
    check::<DataPersistenceOptions>(&mut m, &c, "GET", "/data-persistence", "/data-persistence")
        .await;
    check::<ModulesData>(&mut m, &c, "GET", "/database-modules", "/database-modules").await;
    check::<Regions>(&mut m, &c, "GET", "/regions", "/regions").await;
    check::<AccountSubscriptions>(&mut m, &c, "GET", "/subscriptions", "/subscriptions").await;
    check::<FixedSubscriptions>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions",
        "/fixed/subscriptions",
    )
    .await;
    check::<AccountACLRedisRules>(&mut m, &c, "GET", "/acl/redisRules", "/acl/redisRules").await;
    check::<AccountACLRoles>(&mut m, &c, "GET", "/acl/roles", "/acl/roles").await;
    check::<AccountACLUsers>(&mut m, &c, "GET", "/acl/users", "/acl/users").await;
    check::<AccountUsers>(&mut m, &c, "GET", "/users", "/users").await;
    check::<CloudAccounts>(&mut m, &c, "GET", "/cloud-accounts", "/cloud-accounts").await;
    check::<TasksStateUpdate>(&mut m, &c, "GET", "/tasks", "/tasks").await;
    check::<Value>(
        &mut m,
        &c,
        "GET",
        "/data-integration-workspaces",
        "/data-integration-workspaces",
    )
    .await;

    // A workspace is provisioned separately from the Redis subscription. The
    // dedicated test subscription currently has none, so a clean 404 is the
    // expected live contract for the root operation.
    check_known_404::<Value>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/data-integration-workspace",
        &format!("/subscriptions/{pro_sub}/data-integration-workspace"),
        "requires a configured Data Integration workspace",
    )
    .await;
    skip(
        &mut m,
        "GET",
        "/subscriptions/{subscriptionId}/data-integration-workspace/**",
        "requires a configured Data Integration workspace and proxy path",
    );
    skip(
        &mut m,
        "GET",
        "/endpoint-redirections/{redirectionId}",
        "requires an endpoint redirection lifecycle and redirection id",
    );

    // -- T1: Pro subscription + database reads (pinned) --
    let ps = pro_sub;
    let pd = pro_db;
    check::<Subscription>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}",
        &format!("/subscriptions/{ps}"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/cidr",
        &format!("/subscriptions/{ps}/cidr"),
    )
    .await;
    check::<SubscriptionMaintenanceWindows>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/maintenance-windows",
        &format!("/subscriptions/{ps}/maintenance-windows"),
    )
    .await;
    check::<SubscriptionPricings>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/pricing",
        &format!("/subscriptions/{ps}/pricing"),
    )
    .await;
    check::<AccountSubscriptionDatabases>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases",
        &format!("/subscriptions/{ps}/databases"),
    )
    .await;
    check::<Database>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}",
        &format!("/subscriptions/{ps}/databases/{pd}"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/backup",
        &format!("/subscriptions/{ps}/databases/{pd}/backup"),
    )
    .await;
    check::<CloudTags>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/tags",
        &format!("/subscriptions/{ps}/databases/{pd}/tags"),
    )
    .await;
    check_known_404::<DatabaseTrafficStateResponse>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/traffic",
        &format!("/subscriptions/{ps}/databases/{pd}/traffic"),
        "API 404s traffic for an active database",
    )
    .await;

    // -- T1: connectivity reads (Pro sub) --
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/peerings",
        &format!("/subscriptions/{ps}/peerings"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/transitGateways",
        &format!("/subscriptions/{ps}/transitGateways"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/private-link",
        &format!("/subscriptions/{ps}/private-link"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/private-service-connect",
        &format!("/subscriptions/{ps}/private-service-connect"),
    )
    .await;

    // -- T1: Essentials subscription + database reads (pinned) --
    let es = ess_sub;
    let ed = ess_db;
    check::<FixedSubscription>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}",
        &format!("/fixed/subscriptions/{es}"),
    )
    .await;
    check::<AccountFixedSubscriptionDatabases>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases",
        &format!("/fixed/subscriptions/{es}/databases"),
    )
    .await;
    check::<FixedDatabase>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}",
        &format!("/fixed/subscriptions/{es}/databases/{ed}"),
    )
    .await;
    check::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/backup",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/backup"),
    )
    .await;
    check::<CloudTags>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/tags",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/tags"),
    )
    .await;
    check_known_404::<DatabaseTrafficStateResponse>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/traffic",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/traffic"),
        "API 404s traffic for an active database",
    )
    .await;

    // -- T1 (phase 2): remaining account-level reads --
    check::<redis_cloud::account::AccountSystemLogEntries>(&mut m, &c, "GET", "/logs", "/logs")
        .await;
    check::<redis_cloud::account::AccountSessionLogEntries>(
        &mut m,
        &c,
        "GET",
        "/session-logs",
        "/session-logs",
    )
    .await;
    check::<redis_cloud::account::SearchScalingFactorsData>(
        &mut m,
        &c,
        "GET",
        "/query-performance-factors",
        "/query-performance-factors",
    )
    .await;
    check::<redis_cloud::subscriptions::RedisVersions>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/redis-versions",
        "/subscriptions/redis-versions",
    )
    .await;
    // The API returns a server-side 500 for this spec-documented endpoint (the
    // Pro equivalent works). Tolerated as a known-diff so a later fix surfaces.
    check_tolerating::<redis_cloud::fixed_subscriptions::RedisVersions>(
        &mut m,
        &c,
        "GET",
        "/fixed/redis-versions",
        "/fixed/redis-versions",
        "Internal Server Error",
        "API returns a server-side 500 on this spec-documented endpoint",
    )
    .await;
    check::<redis_cloud::fixed_subscriptions::FixedSubscriptionsPlans>(
        &mut m,
        &c,
        "GET",
        "/fixed/plans",
        "/fixed/plans",
    )
    .await;
    check::<redis_cloud::fixed_subscriptions::FixedSubscriptionsPlans>(
        &mut m,
        &c,
        "GET",
        "/fixed/plans/subscriptions/{subscriptionId}",
        &format!("/fixed/plans/subscriptions/{es}"),
    )
    .await;

    // -- T1 (phase 2): by-id reads (id discovered from the matching list) --
    if let Some(id) = discover(&c, "/users", |v| {
        v["users"][0]["id"].as_i64().map(|n| n.to_string())
    })
    .await
    {
        check::<redis_cloud::users::AccountUser>(
            &mut m,
            &c,
            "GET",
            "/users/{userId}",
            &format!("/users/{id}"),
        )
        .await;
    } else {
        skip(&mut m, "GET", "/users/{userId}", "no user to drill into");
    }
    if let Some(id) = discover(&c, "/acl/users", |v| {
        v["users"][0]["id"].as_i64().map(|n| n.to_string())
    })
    .await
    {
        check::<redis_cloud::acl::ACLUser>(
            &mut m,
            &c,
            "GET",
            "/acl/users/{aclUserId}",
            &format!("/acl/users/{id}"),
        )
        .await;
    } else {
        skip(
            &mut m,
            "GET",
            "/acl/users/{aclUserId}",
            "no ACL user to drill into",
        );
    }
    if let Some(id) = discover(&c, "/cloud-accounts", |v| {
        v["cloudAccounts"][0]["id"].as_i64().map(|n| n.to_string())
    })
    .await
    {
        check::<redis_cloud::cloud_accounts::CloudAccount>(
            &mut m,
            &c,
            "GET",
            "/cloud-accounts/{cloudAccountId}",
            &format!("/cloud-accounts/{id}"),
        )
        .await;
    } else {
        skip(
            &mut m,
            "GET",
            "/cloud-accounts/{cloudAccountId}",
            "no cloud account to drill into",
        );
    }
    if let Some(id) = discover(&c, "/tasks", |v| {
        v["tasks"][0]["taskId"].as_str().map(String::from)
    })
    .await
    {
        check::<redis_cloud::types::TaskStateUpdate>(
            &mut m,
            &c,
            "GET",
            "/tasks/{taskId}",
            &format!("/tasks/{id}"),
        )
        .await;
    } else {
        skip(&mut m, "GET", "/tasks/{taskId}", "no task to drill into");
    }
    if let Some(id) = discover(&c, "/fixed/plans", |v| {
        v["plans"][0]["id"].as_i64().map(|n| n.to_string())
    })
    .await
    {
        check::<redis_cloud::fixed_subscriptions::FixedSubscriptionsPlan>(
            &mut m,
            &c,
            "GET",
            "/fixed/plans/{planId}",
            &format!("/fixed/plans/{id}"),
        )
        .await;
    } else {
        skip(
            &mut m,
            "GET",
            "/fixed/plans/{planId}",
            "no plan to drill into",
        );
    }

    // -- T1 (phase 2): Pro database sub-resources --
    check::<redis_cloud::databases::DatabaseCertificate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/certificate",
        &format!("/subscriptions/{ps}/databases/{pd}/certificate"),
    )
    .await;
    check::<redis_cloud::databases::DatabaseSlowLogEntries>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/slow-log",
        &format!("/subscriptions/{ps}/databases/{pd}/slow-log"),
    )
    .await;
    check::<redis_cloud::types::TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/import",
        &format!("/subscriptions/{ps}/databases/{pd}/import"),
    )
    .await;
    check_known_404::<redis_cloud::databases::BdbVersionUpgradeStatus>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/upgrade",
        &format!("/subscriptions/{ps}/databases/{pd}/upgrade"),
        "404 when no version upgrade is pending",
    )
    .await;
    check::<Value>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/available-target-versions",
        &format!("/subscriptions/{ps}/databases/{pd}/available-target-versions"),
    )
    .await;

    // -- T1 (phase 2): Essentials database sub-resources --
    check::<redis_cloud::fixed_databases::DatabaseSlowLogEntries>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/slow-log",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/slow-log"),
    )
    .await;
    check::<redis_cloud::types::TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/import",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/import"),
    )
    .await;
    check_known_404::<Value>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/upgrade",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/upgrade"),
        "404 when no version upgrade is pending",
    )
    .await;
    check::<Value>(
        &mut m,
        &c,
        "GET",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/available-target-versions",
        &format!("/fixed/subscriptions/{es}/databases/{ed}/available-target-versions"),
    )
    .await;

    // -- T1 (phase 2): sub-level connectivity that may 404 without config --
    check_known_404::<redis_cloud::subscriptions::ActiveActiveSubscriptionRegions>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/regions",
        &format!("/subscriptions/{ps}/regions"),
        "Active-Active subscriptions only",
    )
    .await;
    check_known_404::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/regions/peerings",
        &format!("/subscriptions/{ps}/regions/peerings"),
        "Active-Active subscriptions only",
    )
    .await;
    check_known_404::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/private-link/endpoint-script",
        &format!("/subscriptions/{ps}/private-link/endpoint-script"),
        "needs a configured private link",
    )
    .await;
    check_known_404::<TaskStateUpdate>(
        &mut m,
        &c,
        "GET",
        "/subscriptions/{subscriptionId}/transitGateways/invitations",
        &format!("/subscriptions/{ps}/transitGateways/invitations"),
        "needs Transit Gateway invitations",
    )
    .await;

    // -- region/PSC-specific reads need a regionId / pscServiceId / endpointId we
    //    don't have without Active-Active or configured connectivity --
    for p in [
        "/subscriptions/{subscriptionId}/private-service-connect/{pscServiceId}",
        "/subscriptions/{subscriptionId}/private-service-connect/{pscServiceId}/endpoints/{endpointId}/creationScripts",
        "/subscriptions/{subscriptionId}/private-service-connect/{pscServiceId}/endpoints/{endpointId}/deletionScripts",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-link",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-link/endpoint-script",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-service-connect",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-service-connect/{pscServiceId}",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-service-connect/{pscServiceId}/endpoints/{endpointId}/creationScripts",
        "/subscriptions/{subscriptionId}/regions/{regionId}/private-service-connect/{pscServiceId}/endpoints/{endpointId}/deletionScripts",
        "/subscriptions/{subscriptionId}/regions/{regionId}/transitGateways",
        "/subscriptions/{subscriptionId}/regions/{regionId}/transitGateways/invitations",
    ] {
        skip(
            &mut m,
            "GET",
            p,
            "needs Active-Active / configured connectivity (no test resource)",
        );
    }
    skip(
        &mut m,
        "GET",
        "/cost-report/{costReportId}",
        "binary (CSV) download, not JSON — covered by the live cost-report test",
    );

    // -- T2: non-destructive write lifecycles (reversible, self-cleaning) --

    // Database tags (Pro): create -> update one -> replace all -> delete,
    // preserving every pre-existing non-test tag throughout the lifecycle.
    run_tag_lifecycle(
        &mut m,
        &c,
        &format!("/subscriptions/{ps}/databases/{pd}/tags"),
        "/subscriptions/{subscriptionId}/databases/{databaseId}/tags",
        "/subscriptions/{subscriptionId}/databases/{databaseId}/tags/{tagKey}",
    )
    .await;

    // Database tags (Essentials): same preserving lifecycle.
    run_tag_lifecycle(
        &mut m,
        &c,
        &format!("/fixed/subscriptions/{es}/databases/{ed}/tags"),
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/tags",
        "/fixed/subscriptions/{subscriptionId}/databases/{databaseId}/tags/{tagKey}",
    )
    .await;

    // ACL redis rule (account-global): create -> update -> delete.
    {
        const RULE: &str = "rcrs-compliance-rule";
        let find = |v: &Value| -> Option<String> {
            v["redisRules"]
                .as_array()?
                .iter()
                .find(|x| x["name"].as_str() == Some(RULE))
                .and_then(|x| x["id"].as_i64())
                .map(|n| n.to_string())
        };
        // pre-clean a stray rule from an interrupted prior run
        if let Some(id) = discover(&c, "/acl/redisRules", find).await {
            let _ = c.delete_raw(&format!("/acl/redisRules/{id}")).await;
        }
        let r = c
            .post_raw(
                "/acl/redisRules",
                serde_json::json!({"name": RULE, "redisRule": "+@read ~*"}),
            )
            .await;
        record_write::<TaskStateUpdate>(&mut m, "POST", "/acl/redisRules", r).await;
        // poll for the created rule id (creation is async)
        let mut id = None;
        for _ in 0..15 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Some(found) = discover(&c, "/acl/redisRules", find).await {
                id = Some(found);
                break;
            }
        }
        if let Some(id) = id {
            let r = c
                .put_raw(
                    &format!("/acl/redisRules/{id}"),
                    serde_json::json!({"name": RULE, "redisRule": "+@read +@write ~*"}),
                )
                .await;
            record_write::<TaskStateUpdate>(&mut m, "PUT", "/acl/redisRules/{aclRedisRuleId}", r)
                .await;
            let r = c.delete_raw(&format!("/acl/redisRules/{id}")).await;
            record_write::<TaskStateUpdate>(
                &mut m,
                "DELETE",
                "/acl/redisRules/{aclRedisRuleId}",
                r,
            )
            .await;
            // The delete is async; wait until the rule is actually gone so we
            // don't leave a stray account-global rule behind.
            for _ in 0..15 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if discover(&c, "/acl/redisRules", find).await.is_none() {
                    break;
                }
            }
        } else {
            skip(
                &mut m,
                "PUT",
                "/acl/redisRules/{aclRedisRuleId}",
                "created rule did not appear in time",
            );
            skip(
                &mut m,
                "DELETE",
                "/acl/redisRules/{aclRedisRuleId}",
                "created rule did not appear in time",
            );
        }
    }

    // Subscription update (Pro): rename, then restore.
    if let Some(observed) = c
        .get_raw(&format!("/subscriptions/{ps}"))
        .await
        .ok()
        .and_then(|v| v["name"].as_str().map(String::from))
    {
        let orig = canonical_test_subscription_name(&observed);
        let r = c
            .put_raw(
                &format!("/subscriptions/{ps}"),
                serde_json::json!({"name": format!("{orig}-rcrs-compliance")}),
            )
            .await;
        record_write::<TaskStateUpdate>(&mut m, "PUT", "/subscriptions/{subscriptionId}", r).await;
        let _ = c
            .put_raw(
                &format!("/subscriptions/{ps}"),
                serde_json::json!({"name": orig}),
            )
            .await; // restore
    } else {
        skip(
            &mut m,
            "PUT",
            "/subscriptions/{subscriptionId}",
            "could not read current name to rename/restore",
        );
    }

    // Subscription update (Essentials): rename, then restore.
    if let Some(observed) = c
        .get_raw(&format!("/fixed/subscriptions/{es}"))
        .await
        .ok()
        .and_then(|v| v["name"].as_str().map(String::from))
    {
        let orig = canonical_test_subscription_name(&observed);
        let r = c
            .put_raw(
                &format!("/fixed/subscriptions/{es}"),
                serde_json::json!({"name": format!("{orig}-rcrs-compliance")}),
            )
            .await;
        record_write::<TaskStateUpdate>(&mut m, "PUT", "/fixed/subscriptions/{subscriptionId}", r)
            .await;
        let _ = c
            .put_raw(
                &format!("/fixed/subscriptions/{es}"),
                serde_json::json!({"name": orig}),
            )
            .await; // restore
    } else {
        skip(
            &mut m,
            "PUT",
            "/fixed/subscriptions/{subscriptionId}",
            "could not read current name to rename/restore",
        );
    }

    // Classify every remaining (unwired) write as Skip, then fill reads with
    // Uncovered (and catch any typo'd path).
    classify_remaining_writes(&mut m);
    reconcile_with_spec(&mut m);
    print_report(&m);

    if std::env::var("COMPLIANCE_BLESS").is_ok() {
        write_baseline(&m);
        eprintln!(
            "blessed baseline at {BASELINE_PATH} ({} operations)",
            m.len()
        );
        return;
    }

    let Some(baseline) = load_baseline() else {
        panic!("no baseline at {BASELINE_PATH}; generate it with COMPLIANCE_BLESS=1");
    };
    let changes = diff(&baseline, &m);
    assert!(
        changes.is_empty(),
        "compliance drifted from baseline (re-bless with COMPLIANCE_BLESS=1 once reviewed):\n{}",
        changes.join("\n")
    );
}
