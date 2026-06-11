//! Deserialization tests against hand-authored fixtures that reproduce real
//! Redis Cloud response shapes.
//!
//! Unlike the inline wiremock mocks (which are written to match our models, so
//! they agree with our bugs), these fixtures encode the shapes the live API
//! actually returns — captured by inspecting a real account, then re-authored
//! with synthetic values so they are safe to commit and run in CI without
//! credentials. They guard the type-fidelity regressions found via live
//! testing: #118, #119, #120.
//!
//! The fixtures live in `tests/fixtures/cloud/samples/`. To refresh against a
//! live account for comparison, see `scripts/generate-cloud-fixtures.sh`
//! (output is gitignored and must not be committed).

use redis_cloud::account::PaymentMethods;
use redis_cloud::databases::Database;
use redis_cloud::fixed_databases::AccountFixedSubscriptionDatabases;
use redis_cloud::fixed_subscriptions::FixedSubscription;
use redis_cloud::types::{TaskStateUpdate, TaskStatus};

// #119: modules[].parameters is an array on the wire (empty or objects), and a
// scientific-notation float appears in networkMonthlyUsageInByte. The whole
// response must deserialize.
#[test]
fn fixed_databases_response_deserializes() {
    let raw = include_str!("fixtures/cloud/samples/fixed_databases.json");
    let resp: AccountFixedSubscriptionDatabases =
        serde_json::from_str(raw).expect("fixed databases fixture should deserialize");

    let db = resp
        .subscription
        .and_then(|s| s.databases.into_iter().next())
        .expect("fixture should contain a database");

    // Scientific-notation float field parses as f64.
    assert_eq!(db.network_monthly_usage_in_byte, Some(1.83615528E+10));

    let modules = db.modules.expect("modules should deserialize");
    assert_eq!(modules.len(), 2);
    // Empty array parameters.
    assert_eq!(modules[0].parameters, Some(serde_json::json!([])));
    // Populated array parameters round-trip as a JSON array (#119).
    assert_eq!(
        modules[1].parameters,
        Some(serde_json::json!([{ "name": "error_rate", "value": "0.01" }]))
    );

    // #121: the nested security/clustering/backup objects are now captured
    // instead of silently dropped.
    let security = db
        .security
        .expect("security object should deserialize (see #121)");
    assert_eq!(security.source_ips, Some(vec!["0.0.0.0/0".to_string()]));
    assert_eq!(security.enable_tls, Some(false));
    assert_eq!(security.default_user_enabled, Some(false));

    let clustering = db.clustering.expect("clustering object should deserialize");
    assert_eq!(clustering.enabled, Some(false));
    assert_eq!(clustering.regex_rules.as_ref().map(Vec::len), Some(2));
    assert_eq!(clustering.hashing_policy.as_deref(), Some("standard"));

    let backup = db.backup.expect("backup object should deserialize");
    assert_eq!(backup.remote_backup_enabled, Some(false));
}

// #121 (Pro path): the flexible Database model nests security/clustering/backup
// the same way, plus OSS-cased fields. All must be captured.
#[test]
fn pro_database_response_deserializes() {
    let raw = include_str!("fixtures/cloud/samples/pro_database.json");
    let db: Database = serde_json::from_str(raw).expect("pro database fixture should deserialize");

    assert_eq!(db.database_id, 222333);
    // OSS-cased fields land (rename to supportOSSClusterApi / ...OSSClusterApi).
    assert_eq!(db.support_oss_cluster_api, Some(false));
    assert_eq!(db.use_external_endpoint_for_oss_cluster_api, Some(false));

    let security = db
        .security
        .expect("security object should deserialize (see #121)");
    assert_eq!(security.source_ips, Some(vec!["0.0.0.0/0".to_string()]));
    assert_eq!(security.enable_default_user, Some(true));
    assert_eq!(security.enable_tls, Some(false));

    let clustering = db.clustering.expect("clustering object should deserialize");
    assert_eq!(clustering.number_of_shards, Some(1));
    assert_eq!(clustering.regex_rules.as_ref().map(Vec::len), Some(2));

    let backup = db.backup.expect("backup object should deserialize");
    assert_eq!(backup.enable_remote_backup, Some(false));
    assert_eq!(backup.interval.as_deref(), Some("every-12-hours"));
    assert_eq!(backup.time_utc.as_deref(), Some("14:00"));
}

// #120: creditCardEndsWith is a JSON number; the number-or-string deserializer
// must accept it and stringify it.
#[test]
fn payment_methods_response_deserializes() {
    let raw = include_str!("fixtures/cloud/samples/payment_methods.json");
    let resp: PaymentMethods =
        serde_json::from_str(raw).expect("payment methods fixture should deserialize");

    let methods = resp
        .payment_methods
        .expect("should contain payment methods");
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].credit_card_ends_with.as_deref(), Some("1234"));
}

// #118: the cost-report task nests the id at response.resource.costReportId.
#[test]
fn cost_report_task_response_deserializes() {
    let raw = include_str!("fixtures/cloud/samples/cost_report_task.json");
    let task: TaskStateUpdate =
        serde_json::from_str(raw).expect("cost-report task fixture should deserialize");

    assert!(matches!(task.status, Some(TaskStatus::ProcessingCompleted)));
    let report_id = task
        .response
        .and_then(|r| r.resource)
        .and_then(|res| res.get("costReportId").cloned())
        .and_then(|v| v.as_str().map(str::to_string))
        .expect("response.resource.costReportId should be present");
    assert!(report_id.ends_with(".csv"));
}

// The Essentials subscription model is field-complete; notably `connections`
// is a string on the wire ("10000"), not a number.
#[test]
fn fixed_subscription_response_deserializes() {
    let raw = include_str!("fixtures/cloud/samples/fixed_subscription.json");
    let sub: FixedSubscription =
        serde_json::from_str(raw).expect("fixed subscription fixture should deserialize");

    assert_eq!(sub.id, Some(111111));
    assert_eq!(sub.connections.as_deref(), Some("10000"));
    assert_eq!(sub.database_status.as_deref(), Some("active"));
}
