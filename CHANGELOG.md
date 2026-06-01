# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking:** connectivity handlers now return `TaskStateUpdate` instead of
  `serde_json::Value` for every endpoint the spec marks task-returning
  ([#78](https://github.com/redis-developer/redis-cloud-rs/issues/78)). This
  covers all 11 `private_link` methods, the PSC/Transit Gateway/VPC peering
  deletes (which previously discarded the body and returned `Value::Null`), and
  the corresponding `ConnectivityHandler` façade deletes. Callers can now poll
  the returned `task_id` to completion instead of dropping to raw HTTP. Added
  `CloudClient::delete_typed` for bodyless DELETEs that return a parsed body.

- **Breaking:** consolidated shared task and tag models onto canonical types in
  `crate::types` ([#64](https://github.com/redis-developer/redis-cloud-rs/issues/64)).
  The per-module `TaskStateUpdate` copies (in `tasks`, `users`, `cloud_accounts`,
  `acl`, `flexible::{subscriptions,databases}`, `fixed::{subscriptions,databases}`)
  are now re-exports of `types::TaskStateUpdate`.
  - `TaskStateUpdate::status` is now the typed `Option<TaskStatus>` enum instead
    of `Option<String>`. Unrecognized wire values deserialize to
    `TaskStatus::Unknown` rather than failing. `TaskStateUpdate` also gains a
    `progress: Option<f64>` field.
  - `types::CloudTag` and `types::CloudTags` now match the wire shapes the
    database tag endpoints actually return (`CloudTag` carries
    `created_at`/`updated_at`/`links`; `CloudTags` is the `account_id`/`links`
    HATEOAS envelope). The key/value request pair is now `types::Tag`.
    `flexible::databases` and `fixed::databases` re-export these.

## [0.9.5](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.4...v0.9.5) - 2026-02-06

### Fixed

- handle empty object response from /tasks endpoint ([#56](https://github.com/redis-developer/redis-cloud-rs/pull/56))
- sync Python package version with Rust crate ([#54](https://github.com/redis-developer/redis-cloud-rs/pull/54))

## [0.9.4](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.3...v0.9.4) - 2026-02-05

### Other

- update rust-version to 1.89 and author email ([#52](https://github.com/redis-developer/redis-cloud-rs/pull/52))

## [0.9.3](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.2...v0.9.3) - 2026-02-04

### Added

- harmonize API patterns with redis-enterprise ([#49](https://github.com/redis-developer/redis-cloud-rs/pull/49))

## [0.9.2](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.1...v0.9.2) - 2026-02-03

### Other

- upgrade reqwest to 0.13 ([#46](https://github.com/redis-developer/redis-cloud-rs/pull/46))

## [0.9.1](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.0...v0.9.1) - 2026-01-31

### Added

- add test-support feature for consumer testing ([#43](https://github.com/redis-developer/redis-cloud-rs/pull/43))

### Fixed

- correct mock response formats for tasks and databases ([#45](https://github.com/redis-developer/redis-cloud-rs/pull/45))

## [0.9.0](https://github.com/redis-developer/redis-cloud-rs/compare/v0.8.0...v0.9.0) - 2026-01-30

### Added

- update Python bindings with new methods and add tests ([#33](https://github.com/redis-developer/redis-cloud-rs/pull/33))

### Fixed

- use Link type instead of HashMap for HATEOAS links ([#28](https://github.com/redis-developer/redis-cloud-rs/pull/28))
- address multiple bugs in client and cost_report modules ([#25](https://github.com/redis-developer/redis-cloud-rs/pull/25))
- add README to PyPI package ([#3](https://github.com/redis-developer/redis-cloud-rs/pull/3))

### Other

- code cleanup and add examples ([#42](https://github.com/redis-developer/redis-cloud-rs/pull/42))
- align Rust types with Go client (rediscloud-go-api) ([#41](https://github.com/redis-developer/redis-cloud-rs/pull/41))
- add dependency audit and code coverage ([#34](https://github.com/redis-developer/redis-cloud-rs/pull/34))
- reduce VPC peering duplication and add pagination helpers ([#32](https://github.com/redis-developer/redis-cloud-rs/pull/32))
- fix README examples and add handler method documentation ([#31](https://github.com/redis-developer/redis-cloud-rs/pull/31))
- API cleanup and ergonomic improvements ([#30](https://github.com/redis-developer/redis-cloud-rs/pull/30))
- improve type safety for response and request types ([#29](https://github.com/redis-developer/redis-cloud-rs/pull/29))
- consolidate duplicate ProcessorResponse and error handling ([#26](https://github.com/redis-developer/redis-cloud-rs/pull/26))

## [0.8.0](https://github.com/redis-developer/redis-cloud-rs/compare/v0.7.6...v0.8.0) - 2026-01-30

### Added

- add Python bindings ([#2](https://github.com/redis-developer/redis-cloud-rs/pull/2))
- initial standalone redis-cloud crate

## [0.7.6](https://github.com/redis-developer/redisctl/compare/redis-cloud-v0.7.5...redis-cloud-v0.7.6) - 2026-01-23

### Fixed

- use local README.md for crates to fix sdist build ([#580](https://github.com/redis-developer/redisctl/pull/580))

## [0.7.5](https://github.com/redis-developer/redisctl/compare/redis-cloud-v0.7.4...redis-cloud-v0.7.5) - 2025-12-17

### Fixed

- correct repository URLs broken by PR #500 ([#506](https://github.com/redis-developer/redisctl/pull/506))

### Other

- update documentation URLs to new hosting location ([#509](https://github.com/redis-developer/redisctl/pull/509))

## [0.7.4](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.7.3...redis-cloud-v0.7.4) - 2025-12-13

### Other

- remove outdated implementation tracking file ([#492](https://github.com/joshrotenberg/redisctl/pull/492))

## [0.7.3](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.7.2...redis-cloud-v0.7.3) - 2025-12-09

### Added

- *(cloud)* add delete endpoint for PrivateLink ([#487](https://github.com/joshrotenberg/redisctl/pull/487))
- *(cloud)* add upgrade endpoints for Essentials databases ([#488](https://github.com/joshrotenberg/redisctl/pull/488))

## [0.7.2](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.7.1...redis-cloud-v0.7.2) - 2025-12-09

### Added

- *(cloud)* add task list, database flush, and available-versions commands ([#477](https://github.com/joshrotenberg/redisctl/pull/477))
- *(cloud)* add cost-report API support (Beta) ([#479](https://github.com/joshrotenberg/redisctl/pull/479))
- add user agent header to HTTP requests ([#473](https://github.com/joshrotenberg/redisctl/pull/473))
- *(redis-cloud)* add tracing instrumentation to API client ([#452](https://github.com/joshrotenberg/redisctl/pull/452))
- Add optional Tower service integration to API clients ([#447](https://github.com/joshrotenberg/redisctl/pull/447))

### Fixed

- *(release)* improve Homebrew formula auto-update ([#433](https://github.com/joshrotenberg/redisctl/pull/433))

## [0.7.1](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.7.0...redis-cloud-v0.7.1) - 2025-10-29

### Added

- *(redis-cloud)* add AWS PrivateLink connectivity support ([#406](https://github.com/joshrotenberg/redisctl/pull/406))

### Other

- rewrite README for presentation readiness ([#408](https://github.com/joshrotenberg/redisctl/pull/408))
- implement fixture-based validation for Enterprise API ([#352](https://github.com/joshrotenberg/redisctl/pull/352)) ([#398](https://github.com/joshrotenberg/redisctl/pull/398))

## [0.7.0](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.6.2...redis-cloud-v0.7.0) - 2025-10-07

### Added

- *(redis-cloud)* medium priority API coverage improvements
- *(redis-cloud)* high priority API coverage improvements
- *(redis-cloud)* expand additional response types with list fields
- *(redis-cloud)* expose all known API fields as first-class struct members

### Fixed

- add OpenAPI spec fixture for CI

### Other

- add support package optimization and upload documentation
- add Homebrew installation instructions

## [0.6.1](https://github.com/joshrotenberg/redisctl/compare/redis-cloud-v0.6.0...redis-cloud-v0.6.1) - 2025-09-16

### Added

- add serde_path_to_error for better deserialization error messages ([#349](https://github.com/joshrotenberg/redisctl/pull/349))