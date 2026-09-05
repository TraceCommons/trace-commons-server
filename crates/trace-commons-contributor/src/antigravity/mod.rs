//! Import command support for Antigravity IDE trajectories.
//!
//! This is not a `TraceSource`: the daemon never sees Antigravity directly.
//! The `discover` probe here finds the IDE's local language server API,
//! `client` reads the conversations it serves, `convert` turns them into
//! Trajectory-v1 records, and `import` stages those through the existing
//! `trajectory` source instead.
//!
//! A few items here are read only by this module's own tests -- five
//! descriptive fields on `TrajectoryDescription` and `ImportOutcome`'s
//! `into_result`. Each carries its own `allow(dead_code)` rather than the
//! module-wide one this used to have: the wide allow also covered whatever
//! else went unread, and the note explaining it had already gone stale
//! (`Candidate` is named there as dead and is in fact used by
//! `endpoint::candidates_from` and `probe_candidates`). The fields are part
//! of the API surface the recorded fixtures pin, so they are kept and
//! asserted rather than dropped.

mod client;
mod convert;
mod endpoint;
// The only submodule with a caller outside this module: `commands` drives
// the import. Everything else is reached from inside here, so there are no
// re-exports to drift out of sync with what is actually used.
pub(crate) mod import;
