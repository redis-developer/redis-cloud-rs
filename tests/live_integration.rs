//! Live, read-only integration tests against a real Redis Cloud account.
//!
//! Every test here is `#[ignore]`d, so a plain `cargo test` never touches the
//! network. Run them explicitly, with credentials, and outside any sandbox
//! that blocks outbound TLS:
//!
//! ```bash
//! REDIS_CLOUD_API_KEY=... REDIS_CLOUD_API_SECRET=... \
//!   cargo test --test live_integration -- --ignored
//! ```
//!
//! The credential helper also accepts the redisctl variable names
//! (`REDIS_CLOUD_API_ACCOUNT_KEY` / `REDIS_CLOUD_API_USER_KEY`).
//!
//! ## What these cover
//!
//! Most tests exercise the **read** surface and assert that real API responses
//! deserialize into the typed models. That is the class of drift wiremock
//! tests cannot catch — the mocks are written to match our models, so they
//! agree with our bugs. #119 (module `parameters` array-vs-map) and #120
//! (`creditCardEndsWith` number-vs-string) both shipped because no real
//! response was ever parsed in CI.
//!
//! A few tests exercise **non-destructive, reversible writes** (database tags,
//! a subscription rename, an ACL redis-rule lifecycle). These are strictly
//! reversible and self-cleaning: every write is undone, and write tests that
//! target a subscription/database are pinned to the dedicated `REDIS_CLOUD_TEST_*`
//! resources so they never touch other subscriptions. No database or
//! subscription is ever created or deleted, and nothing destructive is done —
//! so the suite is safe to run against a shared or billable account.

use redis_cloud::{CloudClient, CloudError};

/// Build a client from environment credentials, supporting both the documented
/// variable names and the redisctl convention. Returns `None` when no
/// credentials are present so an explicit `--ignored` run on a machine without
/// them skips loudly rather than failing.
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
            .expect("client should build from credentials"),
    )
}

/// Dedicated test-resource IDs, read from the `REDIS_CLOUD_TEST_*` env vars
/// (see the gitignored `.env`). These point at subscriptions created solely for
/// validation, so deeper per-resource reads (and, later, non-destructive
/// writes) target known-safe resources instead of discovering arbitrary ones.
struct TestResources {
    pro_sub: i32,
    pro_db: i32,
    essentials_sub: i32,
    essentials_db: i32,
}

fn env_i32(key: &str) -> Option<i32> {
    std::env::var(key).ok()?.parse().ok()
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

fn test_resources() -> Option<TestResources> {
    Some(TestResources {
        pro_sub: env_i32("REDIS_CLOUD_TEST_PRO_SUB_ID")?,
        pro_db: env_i32("REDIS_CLOUD_TEST_PRO_DB_ID")?,
        essentials_sub: env_i32("REDIS_CLOUD_TEST_ESSENTIALS_SUB_ID")?,
        essentials_db: env_i32("REDIS_CLOUD_TEST_ESSENTIALS_DB_ID")?,
    })
}

/// Define a `#[ignore]`d live test that receives a built `CloudClient`, or
/// skips with a notice when credentials are absent.
macro_rules! live_test {
    ($name:ident, $client:ident, $body:block) => {
        #[tokio::test]
        #[ignore = "requires live Redis Cloud credentials; run with --ignored"]
        async fn $name() {
            let Some($client) = client() else {
                eprintln!(
                    "SKIP {}: set REDIS_CLOUD_API_KEY/REDIS_CLOUD_API_SECRET to run",
                    stringify!($name)
                );
                return;
            };
            $body
        }
    };
}

/// Like [`live_test!`], but also requires the dedicated test-resource IDs.
/// Skips when either credentials or the `REDIS_CLOUD_TEST_*` vars are absent.
macro_rules! live_test_pinned {
    ($name:ident, $client:ident, $res:ident, $body:block) => {
        #[tokio::test]
        #[ignore = "requires live credentials + REDIS_CLOUD_TEST_* resource ids; run with --ignored"]
        async fn $name() {
            let (Some($client), Some($res)) = (client(), test_resources()) else {
                eprintln!(
                    "SKIP {}: needs credentials and REDIS_CLOUD_TEST_* resource ids",
                    stringify!($name)
                );
                return;
            };
            $body
        }
    };
}

// ---------------------------------------------------------------------------
// account
// ---------------------------------------------------------------------------

live_test!(live_account_get_current, c, {
    let account = c
        .account()
        .get_current_account()
        .await
        .expect("get_current_account should deserialize");
    assert!(
        account.account.is_some() || account.links.is_some(),
        "account response should carry an account or links"
    );
});

live_test!(live_account_data_persistence_options, c, {
    c.account()
        .get_data_persistence_options()
        .await
        .expect("get_data_persistence_options should deserialize");
});

live_test!(live_account_supported_database_modules, c, {
    c.account()
        .get_supported_database_modules()
        .await
        .expect("get_supported_database_modules should deserialize");
});

live_test!(live_account_supported_regions, c, {
    c.account()
        .get_supported_regions(None)
        .await
        .expect("get_supported_regions should deserialize");
});

// Regression guard for #120: `creditCardEndsWith` is a number on the wire.
live_test!(live_account_payment_methods, c, {
    c.account()
        .get_account_payment_methods()
        .await
        .expect("get_account_payment_methods should deserialize (see #120)");
});

// ---------------------------------------------------------------------------
// subscriptions (Pro) — may legitimately be empty
// ---------------------------------------------------------------------------

live_test!(live_subscriptions_list, c, {
    c.subscriptions()
        .list()
        .await
        .expect("subscriptions list should deserialize");
});

// ---------------------------------------------------------------------------
// fixed subscriptions (Essentials) — list, then drill into the first one
// ---------------------------------------------------------------------------

live_test!(live_fixed_subscriptions_flow, c, {
    let subs = c
        .fixed_subscriptions()
        .list()
        .await
        .expect("fixed_subscriptions list should deserialize");

    let Some(first) = subs.subscriptions.as_ref().and_then(|s| s.first()) else {
        eprintln!("SKIP drill-down: no Essentials subscriptions on this account");
        return;
    };
    let sub_id = first.id.expect("subscription should have an id");

    c.fixed_subscriptions()
        .get_by_id(sub_id)
        .await
        .expect("fixed_subscriptions get_by_id should deserialize");
    c.fixed_subscriptions()
        .get_redis_versions(sub_id)
        .await
        .expect("get_redis_versions should deserialize");
});

// Regression guard for #119: real Essentials databases carry modules whose
// `parameters` is an array. Discovers a subscription + database dynamically.
live_test!(live_fixed_databases_flow, c, {
    let subs = c
        .fixed_subscriptions()
        .list()
        .await
        .expect("fixed_subscriptions list should deserialize");
    let Some(sub_id) = subs
        .subscriptions
        .as_ref()
        .and_then(|s| s.first())
        .and_then(|s| s.id)
    else {
        eprintln!("SKIP: no Essentials subscriptions on this account");
        return;
    };

    let dbs = c
        .fixed_databases()
        .list(sub_id, None, None)
        .await
        .expect("fixed_databases list should deserialize (see #119)");

    if let Some(db_id) = dbs
        .subscription
        .and_then(|s| s.databases.into_iter().next())
        .and_then(|d| d.database_id)
    {
        c.fixed_databases()
            .get_by_id(sub_id, db_id)
            .await
            .expect("fixed_databases get_by_id should deserialize (see #119)");
    }
});

// ---------------------------------------------------------------------------
// acl
// ---------------------------------------------------------------------------

live_test!(live_acl_redis_rules, c, {
    c.acl()
        .get_all_redis_rules()
        .await
        .expect("acl get_all_redis_rules should deserialize");
});

live_test!(live_acl_roles, c, {
    c.acl()
        .get_roles()
        .await
        .expect("acl get_roles should deserialize");
});

live_test!(live_acl_users, c, {
    c.acl()
        .get_all_acl_users()
        .await
        .expect("acl get_all_acl_users should deserialize");
});

// ---------------------------------------------------------------------------
// users / cloud accounts / tasks
// ---------------------------------------------------------------------------

live_test!(live_users_list, c, {
    c.users()
        .get_all_users()
        .await
        .expect("users get_all_users should deserialize");
});

live_test!(live_cloud_accounts_list, c, {
    c.cloud_accounts()
        .get_cloud_accounts()
        .await
        .expect("cloud_accounts get_cloud_accounts should deserialize");
});

live_test!(live_tasks_list, c, {
    c.tasks()
        .get_all_tasks()
        .await
        .expect("tasks get_all_tasks should deserialize");
});

live_test!(live_data_integration_workspaces, c, {
    c.data_integration()
        .list_workspaces()
        .await
        .expect("Data Integration workspace list should return JSON");
});

// ---------------------------------------------------------------------------
// Pro subscription (pinned to REDIS_CLOUD_TEST_PRO_SUB_ID)
// ---------------------------------------------------------------------------

live_test_pinned!(live_pro_subscription_reads, c, res, {
    let sub = res.pro_sub;
    let subscription = c
        .subscriptions()
        .get_subscription_by_id(sub)
        .await
        .expect("get_subscription_by_id should deserialize");
    // #128: subscriptionPricing and nested cloudDetails must be captured.
    assert!(
        subscription.subscription_pricing.is_some(),
        "subscriptionPricing should be populated (see #128)"
    );
    assert!(
        subscription.cloud_details.is_some(),
        "cloudDetails should be populated"
    );
    c.subscriptions()
        .get_cidr_allowlist(sub)
        .await
        .expect("get_cidr_allowlist should deserialize");
    c.subscriptions()
        .get_subscription_maintenance_windows(sub)
        .await
        .expect("get_subscription_maintenance_windows should deserialize");
    c.subscriptions()
        .get_subscription_pricing(sub)
        .await
        .expect("get_subscription_pricing should deserialize");
    c.subscriptions()
        .get_redis_versions(Some(sub))
        .await
        .expect("get_redis_versions should deserialize");
});

// ---------------------------------------------------------------------------
// Pro databases (pinned)
// ---------------------------------------------------------------------------

live_test_pinned!(live_pro_database_reads, c, res, {
    let (sub, db) = (res.pro_sub, res.pro_db);
    c.databases()
        .list(sub)
        .await
        .expect("databases list should deserialize");
    let database = c
        .databases()
        .get_subscription_database_by_id(sub, db)
        .await
        .expect("get_subscription_database_by_id should deserialize");
    // #121: the nested security object (with its source-IP allowlist) must be
    // captured from the real response, not silently dropped.
    let security = database
        .security
        .expect("Pro database response should include a security object (see #121)");
    assert!(
        security.source_ips.is_some(),
        "security.sourceIps should be populated"
    );
    c.databases()
        .get_database_backup_status(sub, db, None)
        .await
        .expect("get_database_backup_status should deserialize");
    c.databases()
        .get_subscription_database_certificate(sub, db)
        .await
        .expect("get_subscription_database_certificate should deserialize");
    c.databases()
        .get_slow_log(sub, db, None)
        .await
        .expect("get_slow_log should deserialize");
    c.databases()
        .get_tags(sub, db)
        .await
        .expect("get_tags should deserialize");
    // The spec documents GET .../traffic, but the live API 404s it for a
    // normally-running database (traffic state is only meaningful once stopped).
    // Tolerate the 404; fail only on a deserialize or other unexpected error.
    match c.databases().get_traffic(sub, db).await {
        Ok(_) | Err(CloudError::NotFound { .. }) => {}
        Err(e) => panic!("get_traffic returned an unexpected error: {e}"),
    }
});

// ---------------------------------------------------------------------------
// Connectivity (pinned Pro sub) — typically unconfigured, so a clean NotFound
// is acceptable; only a deserialize/other error fails.
// ---------------------------------------------------------------------------

live_test_pinned!(live_connectivity_reads, c, res, {
    let sub = res.pro_sub;
    macro_rules! ok_or_not_found {
        ($call:expr, $what:literal) => {
            match $call.await {
                Ok(_) | Err(CloudError::NotFound { .. }) => {}
                Err(e) => panic!(concat!($what, " returned an unexpected error: {}"), e),
            }
        };
    }
    ok_or_not_found!(c.vpc_peering().get(sub), "vpc_peering.get");
    ok_or_not_found!(
        c.transit_gateway().get_attachments(sub),
        "transit_gateway.get_attachments"
    );
    ok_or_not_found!(c.psc().get_service(sub), "psc.get_service");
    ok_or_not_found!(c.private_link().get(sub), "private_link.get");
});

// ---------------------------------------------------------------------------
// Essentials database deep reads (pinned)
// ---------------------------------------------------------------------------

live_test_pinned!(live_essentials_database_reads, c, res, {
    let (sub, db) = (res.essentials_sub, res.essentials_db);
    let database = c
        .fixed_databases()
        .get_by_id(sub, db)
        .await
        .expect("fixed get_by_id should deserialize (see #119)");
    // #121: nested security object captured from the real Essentials response.
    let security = database
        .security
        .expect("Essentials database response should include a security object (see #121)");
    assert!(
        security.source_ips.is_some(),
        "security.sourceIps should be populated"
    );
    c.fixed_databases()
        .get_backup_status(sub, db)
        .await
        .expect("fixed get_backup_status should deserialize");
    c.fixed_databases()
        .get_tags(sub, db)
        .await
        .expect("fixed get_tags should deserialize");
    c.fixed_databases()
        .get_slow_log(sub, db)
        .await
        .expect("fixed get_slow_log should deserialize");
    match c.fixed_databases().get_traffic(sub, db).await {
        Ok(_) | Err(CloudError::NotFound { .. }) => {}
        Err(e) => panic!("fixed get_traffic returned an unexpected error: {e}"),
    }
});

// ---------------------------------------------------------------------------
// Non-destructive WRITE validation (pinned) — full database-tag lifecycle.
// Fully reversible and scoped to the pinned test database; cleans up after
// itself (and pre-cleans in case a prior run was interrupted).
// ---------------------------------------------------------------------------

live_test_pinned!(live_pro_database_tag_lifecycle, c, res, {
    use redis_cloud::databases::{DatabaseTagCreateRequest, DatabaseTagUpdateRequest};

    let (sub, db) = (res.pro_sub, res.pro_db);
    let key = "rcrs-write-test";

    // Pre-clean: drop a stray tag from an interrupted prior run (ignore errors).
    let _ = c.databases().delete_tag(sub, db, key.to_string()).await;

    // Create.
    let created = c
        .databases()
        .create_tag(
            sub,
            db,
            &DatabaseTagCreateRequest {
                key: key.to_string(),
                value: "v1".to_string(),
                subscription_id: None,
                database_id: None,
                command_type: None,
            },
        )
        .await
        .expect("create_tag should succeed");
    assert_eq!(created.value.as_deref(), Some("v1"));

    // Read back — the inline tags array must be captured (#130).
    let tags = c
        .databases()
        .get_tags(sub, db)
        .await
        .expect("get_tags should deserialize");
    let found = tags
        .tags
        .unwrap_or_default()
        .into_iter()
        .find(|t| t.key.as_deref() == Some(key));
    assert!(
        found.is_some_and(|t| t.value.as_deref() == Some("v1")),
        "created tag should appear in get_tags with its value (see #130)"
    );

    // Update the value (PUT request serialization).
    let updated = c
        .databases()
        .update_tag(
            sub,
            db,
            key.to_string(),
            &DatabaseTagUpdateRequest {
                subscription_id: None,
                database_id: None,
                key: None,
                value: "v2".to_string(),
                command_type: None,
            },
        )
        .await
        .expect("update_tag should succeed");
    assert_eq!(updated.value.as_deref(), Some("v2"));

    // Delete (cleanup).
    c.databases()
        .delete_tag(sub, db, key.to_string())
        .await
        .expect("delete_tag should succeed");

    // Verify gone.
    let after = c
        .databases()
        .get_tags(sub, db)
        .await
        .expect("get_tags should deserialize");
    let still_present = after
        .tags
        .unwrap_or_default()
        .iter()
        .any(|t| t.key.as_deref() == Some(key));
    assert!(!still_present, "tag should be gone after delete");
});

// Reversible subscription update: rename the pinned test subscription and
// restore it. Guards #133 — the update request must actually carry `name`
// (the old request type silently dropped it, making updates no-ops).
live_test_pinned!(live_pro_subscription_update_name, c, res, {
    use redis_cloud::subscriptions::SubscriptionUpdateRequest;
    use std::time::Duration;

    // Poll the subscription name until it reflects `want` (updates are async),
    // returning the last observed name.
    async fn wait_for_name(c: &CloudClient, sub: i32, want: &str) -> Option<String> {
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Ok(s) = c.subscriptions().get_subscription_by_id(sub).await
                && s.name.as_deref() == Some(want)
            {
                return s.name;
            }
        }
        c.subscriptions()
            .get_subscription_by_id(sub)
            .await
            .ok()
            .and_then(|s| s.name)
    }

    let sub = res.pro_sub;
    let observed = c
        .subscriptions()
        .get_subscription_by_id(sub)
        .await
        .expect("get_subscription_by_id should deserialize")
        .name
        .expect("test subscription should have a name");
    // A force-terminated prior live/compliance run may have left a reserved
    // suffix behind. Always derive the stable name before the new lifecycle so
    // this run heals it when restoring.
    let original = canonical_test_subscription_name(&observed);
    let temp = format!("{original}-rcrs-upd-test");

    // Rename to a temp value.
    c.subscriptions()
        .update_subscription(
            sub,
            &SubscriptionUpdateRequest::builder()
                .name(temp.clone())
                .build(),
        )
        .await
        .expect("update_subscription (rename) should succeed");
    let after = wait_for_name(&c, sub, &temp).await;

    // Restore the original name *before* asserting, so a failed assertion
    // doesn't leave the subscription renamed.
    c.subscriptions()
        .update_subscription(
            sub,
            &SubscriptionUpdateRequest::builder()
                .name(original.clone())
                .build(),
        )
        .await
        .expect("update_subscription (restore) should succeed");
    let restored = wait_for_name(&c, sub, &original).await;

    assert_eq!(
        after.as_deref(),
        Some(temp.as_str()),
        "rename should take effect (see #133)"
    );
    assert_eq!(
        restored.as_deref(),
        Some(original.as_str()),
        "name should be restored"
    );
});

// Reversible ACL redis-rule lifecycle (account-global, so not pinned to a
// subscription). Creates a clearly-named test rule and deletes it, with a
// pre-clean for any rule left by an interrupted prior run. Also guards that the
// response `acl` field is captured (the create request sends `redisRule`, but
// the read response returns `acl`).
live_test!(live_acl_redis_rule_lifecycle, c, {
    use redis_cloud::acl::AclRedisRuleCreateRequest;
    use std::time::Duration;

    const NAME: &str = "rcrs-acl-write-test";

    async fn rule_id(c: &CloudClient, name: &str) -> Option<i32> {
        c.acl()
            .get_all_redis_rules()
            .await
            .ok()
            .and_then(|r| r.redis_rules)
            .and_then(|rules| rules.into_iter().find(|x| x.name.as_deref() == Some(name)))
            .and_then(|x| x.id)
    }
    async fn poll_until<Fut>(mut f: impl FnMut() -> Fut) -> bool
    where
        Fut: std::future::Future<Output = bool>,
    {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_secs(3)).await;
            if f().await {
                return true;
            }
        }
        false
    }

    // Pre-clean a stray rule from an interrupted prior run.
    if let Some(id) = rule_id(&c, NAME).await {
        let _ = c.acl().delete_redis_rule(id).await;
        poll_until(|| async { rule_id(&c, NAME).await.is_none() }).await;
    }

    // Create.
    c.acl()
        .create_redis_rule(&AclRedisRuleCreateRequest {
            name: NAME.to_string(),
            redis_rule: "+@read ~*".to_string(),
            command_type: None,
        })
        .await
        .expect("create_redis_rule should succeed");

    // Wait for it to appear, then capture its acl (response uses `acl`).
    assert!(
        poll_until(|| async { rule_id(&c, NAME).await.is_some() }).await,
        "created rule should appear in get_all_redis_rules"
    );
    let acl = c
        .acl()
        .get_all_redis_rules()
        .await
        .expect("get_all_redis_rules should deserialize")
        .redis_rules
        .unwrap_or_default()
        .into_iter()
        .find(|x| x.name.as_deref() == Some(NAME))
        .and_then(|x| x.acl);

    // Delete (cleanup) before asserting, so a failed assertion can't orphan it.
    let id = rule_id(&c, NAME).await.expect("rule id");
    c.acl()
        .delete_redis_rule(id)
        .await
        .expect("delete_redis_rule should succeed");
    let gone = poll_until(|| async { rule_id(&c, NAME).await.is_none() }).await;

    assert_eq!(
        acl.as_deref(),
        Some("+@read ~*"),
        "response acl should be captured"
    );
    assert!(gone, "rule should be deleted");
});

// Cost-report generate -> poll -> download flow (non-destructive: produces a
// downloadable report, mutates nothing). Guards #118 — the completed task
// nests the id at `response.resource.costReportId`, and the report downloads.
live_test!(live_cost_report_generate_and_download, c, {
    use redis_cloud::cost_report::CostReportCreateRequest;
    use redis_cloud::types::TaskStatus;
    use std::time::Duration;

    // Fixed historical range (account exists since 2025-12; <= 40-day span).
    let task = c
        .cost_reports()
        .generate_cost_report(CostReportCreateRequest::new("2026-05-01", "2026-05-31"))
        .await
        .expect("generate_cost_report should kick off a task");
    let task_id = task.task_id.expect("generate should return a task id");

    let mut report_id = None;
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let state = c
            .tasks()
            .get_task_by_id(task_id.clone())
            .await
            .expect("get_task_by_id should deserialize");
        match state.status {
            Some(TaskStatus::ProcessingCompleted) => {
                report_id = state.response.and_then(|r| r.resource).and_then(|res| {
                    res.get("costReportId")
                        .and_then(|v| v.as_str().map(str::to_string))
                });
                break;
            }
            Some(TaskStatus::ProcessingError) => panic!("cost report generation failed"),
            _ => {}
        }
    }
    let report_id =
        report_id.expect("completed task should carry response.resource.costReportId (see #118)");

    let bytes = c
        .cost_reports()
        .download_cost_report(&report_id)
        .await
        .expect("download_cost_report should succeed");
    assert!(
        !bytes.is_empty(),
        "downloaded cost report should have content"
    );
});
