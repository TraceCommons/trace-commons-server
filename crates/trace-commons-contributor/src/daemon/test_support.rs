//! Test-only helpers shared across the `daemon` module's unit tests.
//!
//! `fn at(s: &str) -> DateTime<Utc> { s.parse().unwrap() }` was written out
//! byte-for-byte in eleven of this module's test modules. It lives here
//! once instead, alongside the [`test_paths`](super::test_paths) precedent.

use chrono::{DateTime, Utc};

/// Parse an RFC 3339 timestamp, panicking on anything that is not one.
///
/// Fixture timestamps are literals in the test source, so a parse failure
/// is a typo in the test rather than a condition to handle.
pub(crate) fn at(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}
