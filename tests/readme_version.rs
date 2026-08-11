const README: &str = include_str!("../README.md");

#[test]
fn installation_dependency_versions_track_crate_minor() {
    let mut version_parts = env!("CARGO_PKG_VERSION").split('.');
    let major = version_parts
        .next()
        .expect("crate version should have a major");
    let minor = version_parts
        .next()
        .expect("crate version should have a minor");
    let expected_version = format!("{major}.{minor}");

    let installation = README
        .split_once("## Installation")
        .expect("README should have an Installation section")
        .1
        .split("\n## ")
        .next()
        .expect("Installation section should have content");
    let dependency_lines: Vec<_> = installation
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("redis-cloud ="))
        .map(str::to_owned)
        .collect();

    let expected_lines = vec![
        format!(r#"redis-cloud = "{expected_version}""#),
        format!(
            r#"redis-cloud = {{ version = "{expected_version}", features = ["tower-integration"] }}"#
        ),
    ];

    assert_eq!(
        dependency_lines, expected_lines,
        "README installation dependencies must track Cargo.toml's major/minor version"
    );
}
