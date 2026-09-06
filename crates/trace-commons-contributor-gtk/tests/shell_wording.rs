//! What this shell says, checked against its own source.
//!
//! Task 5 of the shell-copy slice moves the consent statement out of
//! `src/copy.rs` and into `trace_commons_contributor::consent_copy`, where
//! all three shells read it. The sweep below is what keeps the move from
//! half-happening: a re-export with the old literal still sitting beside it
//! compiles, passes every parity test, and renders the stale sentence on
//! this shell while the other two render the shared one.
//!
//! It lives here rather than in `copy.rs`'s own test module on purpose. A
//! sweep that reads the file it is written in also reads its own needles, so
//! the marker text inside the assertion would open a region that does not
//! exist. Reading the source from disk, from outside, is the only shape that
//! counts the real markers.

use std::path::{Path, PathBuf};

/// This crate's `src/`, resolved from the manifest rather than the cwd.
fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// A migrated region of `copy.rs` holds re-exports and nothing else.
///
/// The drift this catches is specific and has already happened once in
/// spirit: `copy.rs` is the file the other two shells historically
/// transcribed from, so a literal left beside the re-export renders the old
/// constant on the GTK screen while the ABI serves the new one to the other
/// two. Same technique as the routing sweep in `routing_copy.rs`, which
/// reads its own source between `TOOLS-SURFACE-BEGIN` and
/// `TOOLS-SURFACE-END`.
#[test]
fn a_migrated_region_of_copy_rs_holds_no_words_of_its_own() {
    let source = std::fs::read_to_string(src_root().join("copy.rs")).expect("copy.rs is readable");

    let regions: Vec<&str> = source
        .split("// COPY-MIGRATED-BEGIN")
        .skip(1)
        .map(|rest| {
            rest.split("// COPY-MIGRATED-END")
                .next()
                .expect("every COPY-MIGRATED-BEGIN is closed by a COPY-MIGRATED-END")
        })
        .collect();

    // A sweep over no regions is a sweep over nothing, which is the failure
    // mode this whole slice is written against.
    assert!(
        !regions.is_empty(),
        "copy.rs has no COPY-MIGRATED region; migrated copy must be marked"
    );
    assert_eq!(
        source.matches("// COPY-MIGRATED-BEGIN").count(),
        source.matches("// COPY-MIGRATED-END").count(),
        "every marker must be paired"
    );

    for region in regions {
        assert!(
            !region.contains('"'),
            "a migrated region of copy.rs holds a string literal. It may hold `pub use` and \
             nothing else -- a word left beside the re-export is the word this shell renders \
             while the other two render the shared one:\n{region}"
        );
        assert!(
            region.contains("pub use trace_commons_contributor::"),
            "a migrated region must re-export from the shared crate:\n{region}"
        );
    }
}
