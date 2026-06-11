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
//! These exercise the **read** surface and assert that real API responses
//! deserialize into the typed models. That is the class of drift wiremock
//! tests cannot catch — the mocks are written to match our models, so they
//! agree with our bugs. #119 (module `parameters` array-vs-map) and #120
//! (`creditCardEndsWith` number-vs-string) both shipped because no real
//! response was ever parsed in CI.
//!
//! They are strictly read-only: no resource is created, modified, or deleted,
//! so they are safe to run against a shared or billable account. IDs are
//! discovered dynamically, so the suite is not tied to one specific account.

use redis_cloud::CloudClient;

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
