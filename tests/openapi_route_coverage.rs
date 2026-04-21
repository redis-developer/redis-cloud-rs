//! Route coverage checks against the bundled OpenAPI spec.
//!
//! These tests compare normalized path templates extracted from the typed
//! handler implementation with the vendored OpenAPI fixture. Any gap between
//! the two must be tracked explicitly in an allowlist file so coverage drift
//! is visible in review.

use regex::Regex;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const OPENAPI_SPEC: &str = include_str!("fixtures/cloud_openapi.json");
const UNSUPPORTED_SPEC_ROUTES: &str = include_str!("fixtures/openapi_unsupported_routes.txt");
const NON_SPEC_HANDLER_ROUTES: &str = include_str!("fixtures/openapi_non_spec_routes.txt");

fn normalize_path(path: &str) -> String {
    let path = path.replace("{query_string}", "");
    let path = path.split('?').next().unwrap_or(&path);
    let params = Regex::new(r"\{[^}]+\}").expect("valid path parameter regex");
    let normalized = params.replace_all(path, "{}");
    let normalized = normalized.trim_end_matches('/');

    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized.to_string()
    }
}

fn spec_routes() -> BTreeSet<String> {
    let spec: Value = serde_json::from_str(OPENAPI_SPEC).expect("valid bundled OpenAPI spec");
    let mut routes = BTreeSet::new();

    for (path, methods) in spec["paths"]
        .as_object()
        .expect("OpenAPI paths should be an object")
    {
        for method in methods
            .as_object()
            .expect("OpenAPI path methods should be an object")
            .keys()
        {
            let upper = method.to_ascii_uppercase();
            if matches!(upper.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH") {
                routes.insert(format!("{upper} {}", normalize_path(path)));
            }
        }
    }

    routes
}

fn load_route_list(contents: &str) -> BTreeSet<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn source_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();

        if path.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some("testing") {
                continue;
            }

            files.extend(source_files(&path));
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let skip = matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("client.rs" | "lib.rs" | "lib_tests.rs")
        );

        if !skip {
            files.push(path);
        }
    }

    files
}

fn strip_line_comments(contents: &str) -> String {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extracted_handler_routes() -> BTreeSet<String> {
    let direct = Regex::new(
        r#"\.(?P<verb>get|get_raw|get_bytes|post|post_raw|put|put_raw|patch_raw|delete|delete_raw|delete_with_body)\(\s*"(?P<path>/[^"\n]*)""#,
    )
    .expect("valid direct call regex");
    let formatted = Regex::new(
        r#"\.(?P<verb>get|get_raw|get_bytes|post|post_raw|put|put_raw|patch_raw|delete|delete_raw|delete_with_body)\(\s*&?format!\(\s*"(?P<path>/[^"\n]*)""#,
    )
    .expect("valid format call regex");

    let verbs = [
        ("get", "GET"),
        ("get_raw", "GET"),
        ("get_bytes", "GET"),
        ("post", "POST"),
        ("post_raw", "POST"),
        ("put", "PUT"),
        ("put_raw", "PUT"),
        ("patch_raw", "PATCH"),
        ("delete", "DELETE"),
        ("delete_raw", "DELETE"),
        ("delete_with_body", "DELETE"),
    ];

    let verb_map = verbs
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    let mut routes = BTreeSet::new();
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    for file in source_files(&src_dir) {
        let contents = fs::read_to_string(&file).expect("source file should be readable");
        let contents = strip_line_comments(&contents);

        for captures in direct.captures_iter(&contents) {
            let verb = captures.name("verb").expect("verb capture").as_str();
            let path = captures.name("path").expect("path capture").as_str();
            let method = verb_map[verb];
            routes.insert(format!("{method} {}", normalize_path(path)));
        }

        for captures in formatted.captures_iter(&contents) {
            let verb = captures.name("verb").expect("verb capture").as_str();
            let path = captures.name("path").expect("path capture").as_str();
            let method = verb_map[verb];
            routes.insert(format!("{method} {}", normalize_path(path)));
        }
    }

    routes
}

fn unexpected_entries(set: &BTreeSet<String>, expected: &BTreeSet<String>) -> Vec<String> {
    set.difference(expected).cloned().collect()
}

#[test]
fn test_openapi_spec_routes_are_accounted_for() {
    let spec = spec_routes();
    let extracted = extracted_handler_routes();
    let allowlisted = load_route_list(UNSUPPORTED_SPEC_ROUTES);

    let uncovered_spec_routes: BTreeSet<_> = spec.difference(&extracted).cloned().collect();
    let unexpected = unexpected_entries(&uncovered_spec_routes, &allowlisted);
    assert!(
        unexpected.is_empty(),
        "Found spec routes without handler coverage or allowlist entries:\n{}",
        unexpected.join("\n")
    );

    let stale_allowlist = unexpected_entries(&allowlisted, &uncovered_spec_routes);
    assert!(
        stale_allowlist.is_empty(),
        "Found stale unsupported-spec route allowlist entries:\n{}",
        stale_allowlist.join("\n")
    );
}

#[test]
fn test_non_spec_handler_routes_are_explicitly_allowlisted() {
    let spec = spec_routes();
    let extracted = extracted_handler_routes();
    let allowlisted = load_route_list(NON_SPEC_HANDLER_ROUTES);

    let non_spec_routes: BTreeSet<_> = extracted.difference(&spec).cloned().collect();
    let unexpected = unexpected_entries(&non_spec_routes, &allowlisted);
    assert!(
        unexpected.is_empty(),
        "Found handler routes that do not exist in the bundled OpenAPI spec:\n{}",
        unexpected.join("\n")
    );

    let stale_allowlist = unexpected_entries(&allowlisted, &non_spec_routes);
    assert!(
        stale_allowlist.is_empty(),
        "Found stale non-spec handler route allowlist entries:\n{}",
        stale_allowlist.join("\n")
    );
}
