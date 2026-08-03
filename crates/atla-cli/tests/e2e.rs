//! End-to-end tests: the real `atla` binary against a mock Atlassian API.
//!
//! Tests are grouped by contract responsibility so focused suites can evolve without
//! turning this integration crate into one large orchestration file.

#[path = "e2e/auth.rs"]
mod auth;
#[path = "e2e/confluence.rs"]
mod confluence;
#[path = "e2e/formats.rs"]
mod formats;
#[path = "e2e/output.rs"]
mod output;
#[path = "e2e/pagination.rs"]
mod pagination;
#[path = "e2e/plans.rs"]
mod plans;
#[path = "e2e/policy.rs"]
mod policy;
#[path = "e2e/support.rs"]
mod support;
#[path = "e2e/transport.rs"]
mod transport;
