# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.0](https://github.com/redis-developer/redis-cloud-rs/compare/v0.10.0...v0.11.0) - 2026-06-16

### Added

- *(scripts)* add OpenAPI spec-drift check against upstream ([#132](https://github.com/redis-developer/redis-cloud-rs/pull/132))

### Fixed

- *(models)* batch-fix compliance drift — capture dropped response fields ([#140](https://github.com/redis-developer/redis-cloud-rs/pull/140)) ([#143](https://github.com/redis-developer/redis-cloud-rs/pull/143))
- *(subscriptions)* [**breaking**] make Pro update_subscription actually update ([#133](https://github.com/redis-developer/redis-cloud-rs/pull/133)) ([#134](https://github.com/redis-developer/redis-cloud-rs/pull/134))
- *(tags)* capture inline tags array in CloudTags + validate tag write path ([#130](https://github.com/redis-developer/redis-cloud-rs/pull/130)) ([#131](https://github.com/redis-developer/redis-cloud-rs/pull/131))
- *(subscriptions)* [**breaking**] capture subscriptionPricing + nested cloudDetails on reads ([#128](https://github.com/redis-developer/redis-cloud-rs/pull/128)) ([#129](https://github.com/redis-developer/redis-cloud-rs/pull/129))
- *(databases)* [**breaking**] capture nested security/clustering/backup on reads ([#121](https://github.com/redis-developer/redis-cloud-rs/pull/121)) ([#127](https://github.com/redis-developer/redis-cloud-rs/pull/127))
- *(account)* accept numeric `creditCardEndsWith` in payment methods ([#123](https://github.com/redis-developer/redis-cloud-rs/pull/123))
- *(databases)* accept array-shaped module `parameters` on reads ([#122](https://github.com/redis-developer/redis-cloud-rs/pull/122))

### Other

- *(compliance)* phase 3 — non-destructive writes; matrix fully classified ([#142](https://github.com/redis-developer/redis-cloud-rs/pull/142))
- *(compliance)* phase 2 — cover the full read surface ([#141](https://github.com/redis-developer/redis-cloud-rs/pull/141))
- *(compliance)* add API compliance harness (phase 1: reads) ([#139](https://github.com/redis-developer/redis-cloud-rs/pull/139))
- *(cost-report)* validate generate->poll->download flow live ([#138](https://github.com/redis-developer/redis-cloud-rs/pull/138))
- *(spec)* refresh bundled cloud_openapi.json from upstream ([#137](https://github.com/redis-developer/redis-cloud-rs/pull/137))
- *(acl)* validate reversible ACL redis-rule write lifecycle (live) ([#136](https://github.com/redis-developer/redis-cloud-rs/pull/136))
- *(databases)* assert update_database serializes the request body ([#135](https://github.com/redis-developer/redis-cloud-rs/pull/135))
- *(live)* expand read sweep to Pro tier + connectivity, pinned to test resources ([#126](https://github.com/redis-developer/redis-cloud-rs/pull/126))
- add live integration harness + hand-authored response fixtures ([#125](https://github.com/redis-developer/redis-cloud-rs/pull/125))
- *(examples)* fix cost-report id extraction and output filename ([#118](https://github.com/redis-developer/redis-cloud-rs/pull/118))

## [0.10.0](https://github.com/redis-developer/redis-cloud-rs/compare/v0.9.5...v0.10.0) - 2026-06-02

### Added

- *(python)* expand bindings to cover all major read domains, document parity scope (closes #66) ([#113](https://github.com/redis-developer/redis-cloud-rs/pull/113))
- *(api)* add simplified alias methods across remaining domain handlers (closes #65) ([#112](https://github.com/redis-developer/redis-cloud-rs/pull/112))
- *(api)* [**breaking**] implement remaining uncovered spec routes (#72, PR 2) ([#111](https://github.com/redis-developer/redis-cloud-rs/pull/111))

### Fixed

- *(flexible/databases)* correct timeUTC serde casing in DatabaseBackupConfig ([#108](https://github.com/redis-developer/redis-cloud-rs/pull/108)) ([#109](https://github.com/redis-developer/redis-cloud-rs/pull/109))
- *(tasks)* accept object-shaped response.error on failed tasks ([#103](https://github.com/redis-developer/redis-cloud-rs/pull/103))
- *(subscriptions)* send body when deleting Active-Active regions ([#99](https://github.com/redis-developer/redis-cloud-rs/pull/99))
- *(connectivity)* vpc_peering create body serializes the spec's wire keys ([#89](https://github.com/redis-developer/redis-cloud-rs/pull/89))
- *(account)* PaymentMethod.credit_card_ends_with should be String, not i32 ([#87](https://github.com/redis-developer/redis-cloud-rs/pull/87))
- *(flexible/databases)* rename Database.activated to activated_on (wire field is `activatedOn`) ([#86](https://github.com/redis-developer/redis-cloud-rs/pull/86))
- *(tasks)* handle the canonical TasksStateUpdate wrapper from GET /tasks ([#85](https://github.com/redis-developer/redis-cloud-rs/pull/85))
- *(fixed/databases)* add missing 'subscription' field to AccountFixedSubscriptionDatabases ([#81](https://github.com/redis-developer/redis-cloud-rs/pull/81))

### Other

- audit and expand coverage for alias layer and new domains ([#116](https://github.com/redis-developer/redis-cloud-rs/pull/116))
- audit and update documentation for release readiness (closes #114) ([#117](https://github.com/redis-developer/redis-cloud-rs/pull/117))
- *(connectivity)* [**breaking**] reconcile TGW + PSC handler paths with spec (#72, PR 1) ([#110](https://github.com/redis-developer/redis-cloud-rs/pull/110))
- *(openapi)* add executable route-coverage checks vs bundled spec ([#67](https://github.com/redis-developer/redis-cloud-rs/pull/67)) ([#107](https://github.com/redis-developer/redis-cloud-rs/pull/107))
- *(connectivity)* [**breaking**] return TaskStateUpdate for task-returning endpoints ([#78](https://github.com/redis-developer/redis-cloud-rs/pull/78)) ([#106](https://github.com/redis-developer/redis-cloud-rs/pull/106))
- *(types)* [**breaking**] consolidate shared task/tag models onto canonical types ([#64](https://github.com/redis-developer/redis-cloud-rs/pull/64)) ([#104](https://github.com/redis-developer/redis-cloud-rs/pull/104))
- *(spec)* refresh bundled cloud_openapi.json from upstream ([#101](https://github.com/redis-developer/redis-cloud-rs/pull/101))
- README + examples refresh for v0.10.0 ([#100](https://github.com/redis-developer/redis-cloud-rs/pull/100))
- enforce #![deny(missing_docs)] and rustdoc::broken_intra_doc_links ([#98](https://github.com/redis-developer/redis-cloud-rs/pull/98))
- *(models)* document fields in the four large request/response modules ([#96](https://github.com/redis-developer/redis-cloud-rs/pull/96))
- *(users)* document all undocumented fields in user request/response models ([#95](https://github.com/redis-developer/redis-cloud-rs/pull/95))
- *(small modules)* document the remaining undocumented fields in 6 short modules ([#94](https://github.com/redis-developer/redis-cloud-rs/pull/94))
- *(connectivity)* expand thin module headers (psc, transit_gateway, vpc_peering) ([#93](https://github.com/redis-developer/redis-cloud-rs/pull/93))
- *(types)* expand module header and document all variants/fields in shared types ([#92](https://github.com/redis-developer/redis-cloud-rs/pull/92))
- *(connectivity)* document ConnectivityHandler delegation methods and struct fields ([#91](https://github.com/redis-developer/redis-cloud-rs/pull/91))
- *(lib.rs)* refresh stale examples in crate-level docs ([#90](https://github.com/redis-developer/redis-cloud-rs/pull/90))
- *(cost_report)* add wiremock integration coverage for the exported handler ([#88](https://github.com/redis-developer/redis-cloud-rs/pull/88))

### Added

- Simplified alias methods across all domain handlers for a concise, ergonomic
  API surface ([#65](https://github.com/redis-developer/redis-cloud-rs/issues/65)).
  Every handler that previously exposed only verbose `get_all_X` / `get_X_by_id` /
  `create_X` style methods now also exposes short `list` / `get` / `create` /
  `update` / `delete` aliases. The verbose forms are retained for backward
  compatibility. Accessible via `client.subscriptions().list()`,
  `client.databases().list(sub_id)`, `client.acl().list_redis_rules()`, etc.

- Python bindings expanded to cover all major read domains: Account, Tasks,
  Users, ACL (redis rules, roles, users), Cloud Accounts, Essentials
  Subscriptions, and Essentials Databases, in addition to the previously covered
  Pro subscriptions and databases
  ([#66](https://github.com/redis-developer/redis-cloud-rs/issues/66)).
  Every domain method is available in both async and `_sync` blocking variants.
  Coverage table and parity-scope documentation updated in `python/README.md`.

- Implemented every remaining uncovered spec route, raising route coverage to
  **155/155 (100%)** and emptying the unsupported-route allowlist
  ([#72](https://github.com/redis-developer/redis-cloud-rs/issues/72)). New
  typed handler methods and request/response types:
  - Active-Active VPC peering CRUD on `VpcPeeringHandler` (see the Changed note
    above), with a new `ActiveActiveVpcPeeringCreateRequest`
    (`for_aws` / `for_gcp` constructors).
  - `PrivateLinkHandler::disassociate_connections`, `delete_active_active`, and
    `disassociate_connections_active_active`, with new
    `PrivateLinkConnectionsDisassociateRequest`,
    `PrivateLinkActiveActiveConnectionsDisassociateRequest`, and
    `PrivateLinkConnectionDisassociate` types.
  - `PscHandler::get_endpoints` and `get_endpoints_active_active`
    (`GET .../private-service-connect/{pscServiceId}`).
  - `get_traffic` and `resume_traffic` on both the Pro and Essentials database
    handlers, with a shared `types::DatabaseTrafficStateResponse`.
  - `SubscriptionHandler::update_resource_tags`
    (`PUT /subscriptions/{id}/resource-tags`) with a new
    `SubscriptionResourceTagsUpdateRequest`.

- Executable route-coverage checks between the typed client handlers and the
  bundled OpenAPI spec
  ([#67](https://github.com/redis-developer/redis-cloud-rs/issues/67)). A new
  `tests/openapi_route_coverage.rs` extracts the client's routes from source and
  diffs them against the spec; intentional gaps are tracked in two allowlists
  (`tests/fixtures/openapi_unsupported_routes.txt`,
  `openapi_non_spec_routes.txt`). CI now fails when a spec route becomes
  uncovered — or a handler route goes off-spec — without an explicit entry, and
  when an allowlist entry goes stale.

### Changed

- **Breaking:** reconciled the connectivity (Transit Gateway + Private Service
  Connect) handler paths with the refreshed OpenAPI spec
  ([#72](https://github.com/redis-developer/redis-cloud-rs/issues/72)). This
  clears the entire non-spec route allowlist (22 → 0) and raises spec coverage
  to 141/155. Signature/behavior changes:
  - TGW invitation accept/reject now issue `PUT .../transitGateways/invitations/{id}/{accept,reject}`
    (previously `POST .../tgw/shared-invitations/...`); `get_shared_invitations`
    now hits `.../transitGateways/invitations`.
  - Active-Active TGW methods (`get_attachments_active_active`,
    `get_shared_invitations_active_active`) gained a `region_id` parameter, and
    `create_attachment_active_active` / `update_attachment_cidrs_active_active` /
    `delete_attachment_active_active` now use the spec
    `.../regions/{regionId}/transitGateways/{tgwId}/attachment` shape (with a
    `tgw_id` parameter where required).
  - Removed `TransitGatewayHandler::create_attachment` (the no-id variant that
    posted to the non-spec `.../transitGateways/attachments`); use
    `create_attachment_with_id`.
  - PSC endpoint operations (`create_endpoint`, `delete_endpoint`,
    `update_endpoint`, creation/deletion scripts, and their Active-Active
    variants) now take an explicit `psc_service_id` and target the spec
    `.../private-service-connect/{pscServiceId}/endpoints/{endpointId}` shape;
    the Active-Active service methods gained a `region_id` parameter. Removed
    the non-spec `get_endpoints` / `get_endpoints_active_active` list methods.
  - `ConnectivityHandler::create_psc_endpoint` and `update_psc_service_endpoint`
    gained the corresponding `psc_service_id` parameter.

- **Breaking:** the Active-Active VPC peering methods on `VpcPeeringHandler`
  (`get_active_active`, `create_active_active`, `update_active_active`,
  `delete_active_active`) previously delegated to the standard
  `/subscriptions/{id}/peerings` endpoints; they now target the correct
  `/subscriptions/{id}/regions/peerings[/{peeringId}]` spec surface
  ([#72](https://github.com/redis-developer/redis-cloud-rs/issues/72)).
  `create_active_active` now takes `&ActiveActiveVpcPeeringCreateRequest`
  (was `&VpcPeeringCreateRequest`); `update_active_active` now takes
  `&VpcPeeringUpdateAwsRequest`.

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

### Fixed

- `DatabaseBackupConfig::time_utc` and `database_backup_time_utc` now serialize
  as `timeUTC` / `databaseBackupTimeUTC` (uppercase `UTC`) to match the OpenAPI
  spec ([#108](https://github.com/redis-developer/redis-cloud-rs/issues/108)).
  The previous camelCase (`timeUtc`) did not match the API, so a backup start
  hour set on a create/update request was silently dropped. Also corrected the
  docs on the paired `backup_interval`/`database_backup_time_utc`/
  `backup_storage_type` fields: they are alternate **request** field names the
  API accepts, not server-returned aliases (investigation from #97).

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