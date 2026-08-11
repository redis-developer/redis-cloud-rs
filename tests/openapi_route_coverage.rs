//! Executable route-coverage checks between the typed client handlers and the
//! bundled Redis Cloud OpenAPI spec (#67).
//!
//! Unlike [`openapi_validation`](./openapi_validation.rs), which checks the
//! spec's *structure*, this suite proves the client actually covers the
//! advertised routes — and that any gap is an explicit, tracked exception
//! rather than silent drift.
//!
//! ## How it works
//!
//! 1. **Client routes** are extracted at test time from `src/**/*.rs` by
//!    scanning for `self.client.<verb>(...)` calls and the request-path string
//!    literal that follows. No registry to hand-maintain: adding a handler
//!    updates coverage automatically.
//! 2. **Spec routes** come from the bundled `cloud_openapi.json`.
//! 3. Both sides are normalized (path params → `{}`, query strings dropped) so
//!    `/subscriptions/{subscriptionId}/databases` matches
//!    `/subscriptions/{subscription_id}/databases`.
//! 4. Two allowlists capture the known, intentional gaps:
//!    - `fixtures/openapi_unsupported_routes.txt` — spec routes with no handler
//!    - `fixtures/openapi_non_spec_routes.txt` — handler routes not in the spec
//!
//! The tests fail when a spec route becomes uncovered (or a handler route goes
//! off-spec) *without* a matching allowlist entry, and also when an allowlist
//! entry goes stale — keeping the lists small, intentional exceptions. Shrinking
//! them is tracked in #72.

use std::collections::BTreeSet;
use std::path::Path;

const OPENAPI_SPEC: &str = include_str!("fixtures/cloud_openapi.json");
const UNSUPPORTED_ALLOWLIST: &str = include_str!("fixtures/openapi_unsupported_routes.txt");
const NON_SPEC_ALLOWLIST: &str = include_str!("fixtures/openapi_non_spec_routes.txt");

/// A normalized `(METHOD, /path/with/{})` route key.
type Route = (String, String);

/// `CloudClient` helper methods that issue an HTTP request, mapped to the verb
/// they use. Listed longest-first so `get` does not shadow `get_raw` when
/// matching `.get(` against `.get_raw(` (the trailing `(` already prevents
/// that, but the ordering keeps intent clear).
const CLIENT_VERBS: &[(&str, &str)] = &[
    ("get_bytes", "GET"),
    ("get_raw", "GET"),
    ("get", "GET"),
    ("post_empty", "POST"),
    ("post_raw", "POST"),
    ("post", "POST"),
    ("put_raw", "PUT"),
    ("put", "PUT"),
    ("patch_raw", "PATCH"),
    ("delete_with_body", "DELETE"),
    ("delete_typed", "DELETE"),
    ("delete_raw", "DELETE"),
    ("delete", "DELETE"),
];

/// Source files that define request plumbing or examples rather than handler
/// routes; scanning them would surface non-API paths (e.g. doc-comment URLs).
const EXCLUDED_FILES: &[&str] = &[
    "client.rs",
    "error.rs",
    "types.rs",
    "lib.rs",
    "lib_tests.rs",
];

/// Normalize a path for comparison: drop the API's optional `/v1` base prefix
/// and any query string, collapse every `{param}` to `{}`, strip a trailing
/// `{}` that is glued to a segment (an interpolated query string like
/// `/regions{query}`), and trim a trailing `/`.
fn normalize(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);
    let path = match path.strip_prefix("/v1") {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => rest,
        _ => path,
    };
    let path = path.replace("{workspace_path}", "**");

    // Collapse {anything} to {}.
    let mut out = String::with_capacity(path.len());
    let mut depth = 0u32;
    for ch in path.chars() {
        match ch {
            '{' => {
                if depth == 0 {
                    out.push_str("{}");
                }
                depth += 1;
            }
            '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }

    // Strip a trailing `{}` glued to a path segment (interpolated query string).
    if out.ends_with("{}") {
        let stem = &out[..out.len() - 2];
        if !stem.ends_with('/') {
            out.truncate(out.len() - 2);
        }
    }

    out.trim_end_matches('/').to_string()
}

#[test]
fn test_normalize_treats_v1_as_the_api_base_prefix() {
    assert_eq!(
        normalize("/v1/subscriptions/{subscriptionId}"),
        "/subscriptions/{}"
    );
    assert_eq!(normalize("/subscriptions/{id}"), "/subscriptions/{}");
    assert_eq!(
        normalize("/subscriptions/{id}/data-integration-workspace/{workspace_path}"),
        "/subscriptions/{}/data-integration-workspace/**"
    );
    assert_eq!(
        normalize("/v10/subscriptions/{id}"),
        "/v10/subscriptions/{}"
    );
}

/// Extract `(verb, normalized_path)` routes from one source file's contents.
fn extract_routes(src: &str) -> Vec<Route> {
    let bytes = src.as_bytes();
    let mut routes = Vec::new();

    for &(verb_fn, http) in CLIENT_VERBS {
        let needle = format!(".{verb_fn}(");
        let mut from = 0;
        while let Some(rel) = src[from..].find(&needle) {
            let after = from + rel + needle.len();
            from = after; // continue past this match next iteration
            if let Some(path) = path_literal_after(bytes, after)
                && path.starts_with('/')
            {
                routes.push((http.to_string(), normalize(&path)));
            }
        }
    }
    routes
}

/// Starting just after a `.verb(`, skip an optional `&`, an optional
/// `format!(`, and whitespace, then read the immediately-following string
/// literal. Returns `None` if the direct argument is not a string literal
/// (e.g. `map.get(&key)`), which filters out non-request calls.
fn path_literal_after(bytes: &[u8], mut i: usize) -> Option<String> {
    let skip_ws = |bytes: &[u8], mut i: usize| {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    };

    i = skip_ws(bytes, i);
    if i < bytes.len() && bytes[i] == b'&' {
        i = skip_ws(bytes, i + 1);
    }
    // optional `format!(`
    const FMT: &[u8] = b"format!";
    if bytes[i..].starts_with(FMT) {
        i = skip_ws(bytes, i + FMT.len());
        if i < bytes.len() && bytes[i] == b'(' {
            i = skip_ws(bytes, i + 1);
        } else {
            return None;
        }
    }
    if i >= bytes.len() || bytes[i] != b'"' {
        return None;
    }
    i += 1;
    let start = i;
    while i < bytes.len() && bytes[i] != b'"' {
        // raw byte scan is fine: API paths contain no escaped quotes
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    std::str::from_utf8(&bytes[start..i])
        .ok()
        .map(str::to_string)
}

/// Recursively collect `*.rs` files under `dir`, skipping [`EXCLUDED_FILES`]
/// and the `testing` support module.
fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("read_dir src");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("testing") {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !EXCLUDED_FILES.contains(&name) {
                out.push(path);
            }
        }
    }
}

/// The full set of routes implemented by the typed client handlers.
fn client_routes() -> BTreeSet<Route> {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);
    assert!(
        files.len() > 5,
        "expected to scan multiple handler files, found {}",
        files.len()
    );

    let mut routes = BTreeSet::new();
    for file in files {
        let src = std::fs::read_to_string(&file).expect("read source file");
        routes.extend(extract_routes(&src));
    }
    routes
}

/// The full set of routes advertised by the bundled OpenAPI spec.
fn spec_routes() -> BTreeSet<Route> {
    let spec: serde_json::Value = serde_json::from_str(OPENAPI_SPEC).expect("parse spec");
    let paths = spec["paths"].as_object().expect("spec paths object");
    let mut routes = BTreeSet::new();
    for (path, ops) in paths {
        let ops = ops.as_object().expect("operations object");
        for method in ops.keys() {
            let http = method.to_uppercase();
            if matches!(http.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
                routes.insert((http, normalize(path)));
            }
        }
    }
    routes
}

/// Parse an allowlist fixture into a route set (skips blank/`#` comment lines).
fn parse_allowlist(contents: &str) -> BTreeSet<Route> {
    contents
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (verb, path) = l
                .split_once(char::is_whitespace)
                .unwrap_or_else(|| panic!("malformed allowlist line: {l:?}"));
            (verb.to_string(), normalize(path.trim()))
        })
        .collect()
}

fn fmt_routes(routes: &BTreeSet<Route>) -> String {
    routes
        .iter()
        .map(|(v, p)| format!("  {v} {p}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_every_spec_route_is_covered_or_allowlisted() {
    let client = client_routes();
    let spec = spec_routes();
    let allowed = parse_allowlist(UNSUPPORTED_ALLOWLIST);

    let uncovered: BTreeSet<Route> = spec.difference(&client).cloned().collect();
    let unexpected: BTreeSet<Route> = uncovered.difference(&allowed).cloned().collect();

    assert!(
        unexpected.is_empty(),
        "{} spec route(s) are not covered by a client handler and not in \
         tests/fixtures/openapi_unsupported_routes.txt:\n{}\n\nImplement the \
         handler, or add the route to the allowlist if intentionally deferred.",
        unexpected.len(),
        fmt_routes(&unexpected)
    );
}

#[test]
fn test_every_client_route_is_in_spec_or_allowlisted() {
    let client = client_routes();
    let spec = spec_routes();
    let allowed = parse_allowlist(NON_SPEC_ALLOWLIST);

    let off_spec: BTreeSet<Route> = client.difference(&spec).cloned().collect();
    let unexpected: BTreeSet<Route> = off_spec.difference(&allowed).cloned().collect();

    assert!(
        unexpected.is_empty(),
        "{} client route(s) do not match any spec path and are not in \
         tests/fixtures/openapi_non_spec_routes.txt:\n{}\n\nFix the handler path \
         to match the spec, or add it to the allowlist if intentionally retained.",
        unexpected.len(),
        fmt_routes(&unexpected)
    );
}

#[test]
fn test_unsupported_allowlist_has_no_stale_entries() {
    let client = client_routes();
    let spec = spec_routes();
    let allowed = parse_allowlist(UNSUPPORTED_ALLOWLIST);

    // Every entry must (a) be a real spec route and (b) still be uncovered.
    let not_in_spec: BTreeSet<Route> = allowed.difference(&spec).cloned().collect();
    let now_covered: BTreeSet<Route> = allowed.intersection(&client).cloned().collect();

    assert!(
        not_in_spec.is_empty(),
        "openapi_unsupported_routes.txt lists route(s) that are not in the spec \
         (remove them):\n{}",
        fmt_routes(&not_in_spec)
    );
    assert!(
        now_covered.is_empty(),
        "openapi_unsupported_routes.txt lists route(s) that ARE now covered by a \
         handler (remove them — coverage improved 🎉):\n{}",
        fmt_routes(&now_covered)
    );
}

#[test]
fn test_non_spec_allowlist_has_no_stale_entries() {
    let client = client_routes();
    let spec = spec_routes();
    let allowed = parse_allowlist(NON_SPEC_ALLOWLIST);

    // Every entry must (a) be a real client route and (b) still be off-spec.
    let not_in_client: BTreeSet<Route> = allowed.difference(&client).cloned().collect();
    let now_in_spec: BTreeSet<Route> = allowed.intersection(&spec).cloned().collect();

    assert!(
        not_in_client.is_empty(),
        "openapi_non_spec_routes.txt lists route(s) no client handler emits \
         (remove them):\n{}",
        fmt_routes(&not_in_client)
    );
    assert!(
        now_in_spec.is_empty(),
        "openapi_non_spec_routes.txt lists route(s) that now match the spec \
         (remove them — drift resolved 🎉):\n{}",
        fmt_routes(&now_in_spec)
    );
}

#[test]
fn test_coverage_summary_is_reasonable() {
    let client = client_routes();
    let spec = spec_routes();
    let covered = client.intersection(&spec).count();

    // Guardrails: the extractor found a sensible number of routes and the
    // majority of the spec is covered. Tune only with intent.
    assert!(
        client.len() > 100,
        "extracted only {} client routes — extractor may be broken",
        client.len()
    );
    assert!(
        covered * 100 / spec.len() >= 75,
        "covered only {covered}/{} spec routes (<75%)",
        spec.len()
    );
}
