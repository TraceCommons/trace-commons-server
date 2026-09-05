//! Import command support for Antigravity IDE trajectories.
//!
//! This is not a `TraceSource`: the daemon never sees Antigravity directly.
//! The `discover` probe here finds the IDE's local language server API,
//! `client` reads the conversations it serves, `convert` turns them into
//! Trajectory-v1 records, and `import` stages those through the existing
//! `trajectory` source instead.
//!
//! Descriptive listing fields and `ImportOutcome::into_result` are exercised
//! by fixtures but not read by the production import path. Their non-test
//! builds carry item-level dead-code allowances; the rest of the module
//! remains checked, including endpoint candidate discovery.

mod client;
mod convert;
mod endpoint;
// The only submodule with a caller outside this module: `commands` drives
// the import. Everything else is reached from inside here, so there are no
// re-exports to drift out of sync with what is actually used.
pub(crate) mod import;
