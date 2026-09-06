# Shell Copy Migration, Slice 0 + ReadGate — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the macOS and GTK shells the wording ratchet the Windows shell
got in #644, and then move the first four sentences — the read gate's safety
claim, shown at the instant of consent — into a Rust-owned copy bundle that all
three shells read. At the end of this slice no shell authors any of those four
sentences, and each of the three shells has a number that can only go down.

**Architecture:** The spec's rule is core-owns-the-words. A surface's fixed
strings are one `#[derive(Serialize)]` struct in
`crates/trace-commons-contributor`, built by one function, exported across the C
ABI as one JSON object by one export in `trace-commons-contributor-ffi`, and
declared in both byte-identical copies of `trace_commons.h`. GTK links the
contributor crate directly and re-exports it; macOS decodes in `TCShellCore`
after `TCBridge` fetches the JSON; Windows decodes in `TraceCommons.Interop`
after `NativeMethods` fetches it. Where a sentence is *chosen* by a condition
the branch crosses too — `tc_consent_gate_help(pinned)` returns the chosen
sentence, so three shells cannot each keep their own `? :`.

Slice 0 is the prerequisite and moves no copy: two source-scanning guards, one
per unratcheted shell, each with an exact per-file baseline that is a ceiling
and a floor, a zero list of surfaces the Rust already owns, and a coverage floor
so the guard cannot pass over nothing.

**Tech Stack:** Rust (`serde`, the existing C ABI); Swift 6 / XCTest in
`macos/`; C# / xunit in `windows/tests/TraceCommons.Interop.Tests`; the GTK
crate's own cargo workspace.

**Spec:** `docs/superpowers/specs/2026-09-06-shell-copy-migration-design.md`
(PR #648, branch `shell-copy-migration-spec`). This plan covers **slice 0 and
slice 1 only** — the spec's own recommended first unit. Slices 2 through 8 are
out of scope and get their own plans.

## Global Constraints

Copied from the spec and from this repository's standing rules. Where the spec
decided something, this plan repeats the decision rather than reopening it.

- **No emojis** anywhere: commits, PR bodies, code, comments, reports. Commit
  subjects short and imperative, no `feat:` / `fix:` prefix.
- **TDD.** Every task writes its failing test first, watches it fail for the
  right reason, then implements. The guards in this slice *are* the product; a
  guard written after the thing it guards has never been seen to fail.
- **Verify with `RUSTFLAGS='-D warnings'`.** Plain `cargo check` does not apply
  it; CI does.
  ```bash
  RUSTFLAGS='-D warnings' cargo check -p trace-commons-contributor -p trace-commons-contributor-ffi
  RUSTFLAGS='-D warnings' cargo test -p trace-commons-contributor -p trace-commons-contributor-ffi --no-run
  ```
- **A workspace-scoped check misses four configurations CI gates.** After ANY
  change to `-contributor` or `-contributor-ffi`, also run:
  1. **the permissive crates standalone** —
     `cargo check -p trace-commons-contributor --no-default-features` and
     `cargo check -p trace-commons-contributor-ffi --no-default-features`.
     Cargo unifies features across a workspace build, so a permissive crate can
     silently come to need one of this workspace's optional features; this is
     the configuration a third-party harness gets and the only job that sees it.
  2. **the GTK workspace** (excluded from the root workspace) —
     `cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml`
     and
     `cargo fmt --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml -- --check`.
  3. **the Swift package** — `cargo build -p trace-commons-contributor-ffi`
     first (the package links the dylib), then `swift test` from `macos/`.
  4. **the interop suite** — `cargo build -p trace-commons-contributor-ffi`,
     then, from `windows/`,
     `dotnet test tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj`.
- **Clippy allow-list, verbatim:** `-A clippy::type_complexity
  -A clippy::collapsible_if -A clippy::manual_option_as_slice
  -A clippy::useless_vec -A clippy::redundant_pattern_matching`. Do not widen it.
- **Both header copies, in the same commit.**
  `crates/trace-commons-contributor-ffi/include/trace_commons.h` and
  `macos/Sources/CTraceCommons/include/trace_commons.h` are held byte-for-byte
  identical by `crates/trace-commons-contributor-ffi/tests/abi_header_surface.rs`,
  which also derives the expected surface from the Rust `extern "C"` functions.
  Edit the FFI crate's copy, then `cp` it over the macOS copy. Never hand-edit
  the second copy.
- **Licensing.** `-contributor`, `-contributor-ffi` and `-contributor-gtk` are
  `MIT OR Apache-2.0` and must stay that way. Nothing in this slice may add a
  dependency on `-server`, `-gate-api` or `-gate-enclave`. Never edit
  `crates/trace-commons-server/tests/license_boundary.rs`.
- **No new dependencies.** `serde` and `serde_json` are already in both Rust
  crates; everything else here is stdlib, XCTest and xunit.
- **A migration commit moves a sentence byte-for-byte.** No rewording in
  passing. Where the shells disagree the PR must say so and pick one, and *that
  choice* is the reviewable content of the PR (Task 6).
- **Spec non-goals, all five:** no localization, no server-delivered or
  reloadable copy, no version field on the bundle, no moving GTK onto the C ABI,
  no view-model rewrites beyond replacing a literal with a field read.
- **Never invent a baseline number.** Every count in a ratchet is pasted from
  the scanner's own dump, in the same commit that adds the scanner.
- **Never give a Rust-owned surface a baseline allowance.** If a file on a zero
  list measures non-zero, that is a finding: the literal moves to Rust, or the
  file leaves the zero list before the first commit and the PR says why. It does
  not get an entry.

## What is true today, before any of this lands

Refresh these at the start of Task 1; the spec requires it.

- **#644 (`c6f097e5`, branch `windows-wording-guard-coverage`) is NOT merged
  into main** as of `8c796948`. It adds
  `windows/tests/TraceCommons.Interop.Tests/ShellWordingTests.cs` (392 lines)
  and the `shell-source/**` copy step in the test csproj. Task 6 edits that
  file, so **this slice depends on #644 landing first** or on being stacked on
  that branch. Confirm before starting; if it is still open, stack and say so in
  the PR.
- **#611 (`source_copy` / `tc_source_settings_copy`) and #612 (`onboarding_copy`
  / `tc_onboarding_copy`) are open.** Neither collides with `consent_copy`. Do
  not recreate their bundles here.
- **#610 is merged:** `compute/controller.rs::ComputeCopy` and
  `tc_compute_copy_json` already provide core copy. Reuse the contract shape; do
  not touch compute's separate controller lifetime, its consent handling, or its
  disabled-by-default production behaviour.
- The four sentences this slice moves live in three places today:
  - `windows/src/TraceCommons.Interop/ReadGate.cs` — `Statement`, `ReadyHelp`,
    `UnenrolledHelp` (the last written as two adjacent literals, which is why
    #644 counts this file at 4).
  - `macos/Sources/TCShellCore/ReadGate.swift` — `statement`, `readyHelp`,
    `notPinnedHelp`.
  - `crates/trace-commons-contributor-gtk/src/copy.rs` — `GATE_STATEMENT` only.
    GTK has no tooltip on `Contribute`, so it holds neither help sentence.
- Two Rust tests in `crates/trace-commons-contributor-gtk/src/copy.rs` open the
  other shells' sources and grep them:
  `the_three_shells_print_the_same_statement` (this slice retires it, in Task 7,
  after narrowing it in Task 6) and
  `the_correction_disclosure_is_intact_in_all_three_shells` (this slice
  **keeps** it — see the note in Task 7).

## The seven tasks

| # | Shell / crate | What lands | Commit |
|---|---|---|---|
| 1 | macOS | `ShellWordingTests.swift`: scanner, measured baseline, zero list, coverage floor | own |
| 2 | GTK | `tests/shell_wording.rs`: scanner, measured baseline, zero list, coverage floor | own |
| 3 | `-contributor` | `consent_copy.rs`: the four sentences, the bundle, the branch | own |
| 4 | `-contributor-ffi` | `tc_consent_copy`, `tc_consent_gate_help`, both headers, ABI tests | own |
| 5 | GTK | re-export inside `COPY-MIGRATED` markers, marker sweep, ratchet down | own |
| 6 | Windows | `ConsentCopy.cs` / `ConsentSurface.cs`, `ReadGate.cs` stripped, ratchet to zero | own |
| 7 | macOS | `ConsentCopy.swift` / `TCConsentCopy.swift`, `ReadGate.swift` stripped, ratchet to zero | own |

Tasks 1 and 2 are independent of each other and of 3. Task 4 needs 3. Task 5
needs 3 (not 4 — GTK never touches the ABI). Tasks 6 and 7 need 4, and 6 must
precede 7 so that the three-shell parity test is narrowed before it is deleted.

---

### Task 1 (slice 0, macOS): the macOS wording ratchet

**Files:**
- Create: `macos/Tests/TCShellCoreTests/ShellWordingTests.swift`
- Nothing else. `TCShellCoreTests` is an existing target; a new file in it needs
  no `Package.swift` edit, and the target links no dylib, so this runs under a
  plain `swift test`.

**Interfaces:**
- Produces (all `private` to the test file):
  ```swift
  final class ShellWordingTests: XCTestCase
  private enum ShellSources {
      static func root() -> URL                       // <repo>/macos/Sources
      static func scan() -> [String: [String]]        // relative path -> authored sentences
  }
  private func authoredWording(in source: String) -> [String]
  private func readsAsASentence(_ literal: String) -> Bool
  ```
- Consumes: nothing. The scan reads `macos/Sources/**/*.swift` off disk,
  located from `#filePath`, the way `ScrubDetectorsTests` locates its fixture.

**Why a scanner and not a list of forbidden strings:** the same reason #644
gives. A hand-kept list only ever covers the sentences somebody remembered to
add. This counts, per file, the literals that read as a sentence a contributor
would be shown, and pins each file at the number it holds today.

- [ ] **Step 1: Refresh the facts the spec asks for**

```bash
gh pr view 644 --repo TraceCommons/trace-commons --json state,mergedAt
gh pr view 611 --repo TraceCommons/trace-commons --json state,mergedAt
gh pr view 612 --repo TraceCommons/trace-commons --json state,mergedAt
```

If #644 is still open, base this branch on `windows-wording-guard-coverage` and
record that in the PR body. Nothing in Tasks 1 through 5 depends on it; Task 6
does.

- [ ] **Step 2: Write the scanner and the two failing tests**

Create `macos/Tests/TCShellCoreTests/ShellWordingTests.swift`:

```swift
import XCTest

/// Wording authored in this shell, over the whole shell.
///
/// The mirror of `ShellWordingTests.cs` on Windows, for the same reason and
/// with the same shape: a sentence hand-written in one shell survives a
/// rename in the other two, and until this file existed nothing here
/// noticed. Every Swift source under `macos/Sources` is read and the
/// literals that read as a sentence a contributor would be shown are
/// counted, per file.
///
/// It is a ratchet, not a clean bill of health. This shell today authors
/// most of its own wording: `TCShellCore`'s `*Copy` types were transcribed
/// from the same shared design the Windows interop classes were, and the
/// SwiftUI views carry their own labels and help text. Moving all of that
/// behind the ABI is a project of its own -- see the migration spec. Every
/// file that authors wording is recorded below with the exact number it
/// holds today, so nothing new can be added to it and nothing new can start
/// doing it.
final class ShellWordingTests: XCTestCase {
    /// Files that author wording today, and exactly how much.
    ///
    /// TODO(shell-copy): every entry here is a file whose wording should be
    /// composed in the Rust contributor crate and read across the C ABI, the
    /// way `routing_copy.rs` and `witness_copy.rs` already are. Until then
    /// the number is a CEILING AND A FLOOR both: adding a sentence fails,
    /// and removing one fails too, so the entry has to be lowered
    /// deliberately as copy moves out. Never raise a number. A new file must
    /// never be added here.
    ///
    /// MEASURED, NOT ESTIMATED. Every number came from this file's own
    /// scanner via TC_WORDING_DUMP=1; none was typed by hand.
    private static let wordingBaseline: [String: Int] = [
        // PASTE THE DUMP HERE IN STEP 4.
    ]

    /// The surfaces whose wording already comes from Rust. Nothing may ever
    /// buy them an allowance in the baseline: they hold no sentence at all,
    /// and an entry here would be a quiet way of undoing that.
    ///
    /// Task 7 adds `TCShellCore/ConsentCopy.swift`,
    /// `TCShellCore/ReadGate.swift` and `TCBridge/TCConsentCopy.swift` to
    /// this list.
    private static let rustOwnedSurfaces = [
        "TCBridge/TCRoutingCopy.swift",
        "TCShellCore/RoutingCopy.swift",
        "TCShellCore/RoutingSurface.swift",
    ]

    /// Words a sentence has and an identifier, a wire key, a symbol name or
    /// a format pattern does not.
    ///
    /// The same list the Windows guard uses, deliberately: two shells
    /// counting the same corpus by different rules would produce two numbers
    /// nobody could compare. It is a function-word test rather than a "has a
    /// space" test, because `"chevron.right"`, `"MMMM d, yyyy"` and
    /// `"en_US_POSIX"` are not wording and `"Watch this folder"` is.
    private static let functionWords: Set<String> = Set(
        """
        a an and are as at be been being but by can cannot could did do does for from
        had has have how if in into is isn't it it's its just may never no not nothing of off on once only or
        so some still such than that the their them then there they this those to until up was we were what
        when where which while who will with would you your yours yet anything something everything already
        always about after again all any because before both each else ever every here more most much
        must need needs same see should since take takes tell these too under use used using very
        """
        .split(whereSeparator: { $0 == " " || $0 == "\n" })
        .map(String.init))

    /// No file in this shell authors more wording than it did when this
    /// guard was written, and no file starts authoring wording that did not.
    func testNoWordingIsAuthoredInThisShellBeyondTheRecordedBaseline() {
        let scanned = ShellSources.scan()

        // A scan that found nothing would turn this test into a pass over
        // nothing, which is the failure mode the Windows guard names
        // explicitly. There are 89 Swift sources under macos/Sources today.
        XCTAssertGreaterThanOrEqual(
            scanned.count, 80,
            "only \(scanned.count) Swift sources were scanned under \(ShellSources.root().path); "
                + "the whole tree is expected")

        if ProcessInfo.processInfo.environment["TC_WORDING_DUMP"] == "1" {
            for path in scanned.keys.sorted() where !scanned[path]!.isEmpty {
                print("        \"\(path)\": \(scanned[path]!.count),")
            }
        }

        var failures: [String] = []
        for path in scanned.keys.sorted() {
            let wording = scanned[path]!
            let allowed = Self.wordingBaseline[path] ?? 0
            if wording.count == allowed { continue }
            if wording.count > allowed {
                failures.append(
                    "\(path): \(wording.count) authored sentences, baseline allows \(allowed). "
                        + "First one over the line: \"\(wording[allowed])\"")
            } else {
                failures.append(
                    "\(path): \(wording.count) authored sentences, baseline still allows "
                        + "\(allowed). Wording moved out -- lower the entry (or delete it at zero).")
            }
        }
        for recorded in Self.wordingBaseline.keys.sorted() where scanned[recorded] == nil {
            failures.append(
                "\(recorded): recorded in the baseline but no longer in the shell. Delete the entry.")
        }

        XCTAssertTrue(
            failures.isEmpty,
            "Wording on this shell's surfaces comes from the Rust contributor crate across the "
                + "ABI.\nA sentence written here is one the other two shells will not get, and one "
                + "a rename in the Rust will not reach.\n\n" + failures.joined(separator: "\n"))
    }

    /// The surfaces the Rust already owns hold no wording at all, and hold
    /// no baseline entry either.
    func testTheRustOwnedSurfacesAreNotGivenAWordingAllowance() {
        let scanned = ShellSources.scan()
        for surface in Self.rustOwnedSurfaces {
            XCTAssertNil(
                Self.wordingBaseline[surface],
                "\(surface) has a wording baseline entry. Its wording comes from Rust; an "
                    + "allowance here would quietly undo that.")
            guard let wording = scanned[surface] else {
                XCTFail("\(surface) was not among the scanned sources; the guard would pass over nothing.")
                continue
            }
            XCTAssertEqual(wording, [], "\(surface) authors wording: \(wording)")
        }
    }
}

/// Every Swift source of this shell, and the sentences each one authors.
private enum ShellSources {
    /// `.../macos/Tests/TCShellCoreTests/ShellWordingTests.swift` ->
    /// `.../macos/Sources`. Located from the source file's own path, the way
    /// `ScrubDetectorsTests` locates the shared scrub-label fixture: the
    /// working directory of `swift test` is not something to depend on.
    static func root() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()   // TCShellCoreTests
            .deletingLastPathComponent()   // Tests
            .appendingPathComponent("Sources")
    }

    /// Keyed by path relative to `macos/Sources`, so an entry reads the way
    /// the Windows baseline's do.
    static func scan() -> [String: [String]] {
        let base = root()
        var scanned: [String: [String]] = [:]
        guard
            let walker = FileManager.default.enumerator(
                at: base, includingPropertiesForKeys: nil)
        else {
            return scanned
        }
        for case let url as URL in walker where url.pathExtension == "swift" {
            let relative = url.path
                .replacingOccurrences(of: base.path + "/", with: "")
            guard let text = try? String(contentsOf: url, encoding: .utf8) else { continue }
            scanned[relative] = authoredWording(in: text)
        }
        return scanned
    }

    /// The sentences a Swift source authors.
    ///
    /// A character walk rather than a regex, because this shell writes copy
    /// in `"""` blocks and a line-oriented scan reads one of those as three
    /// unterminated literals. Comments go first, for the reason the routing
    /// guard gives: prose about the wire may quote it, and nothing in a
    /// comment is rendered.
    ///
    /// Known and accepted limitation: an interpolation is folded away with
    /// its escape, so `"\(n) sessions"` is read as `"n) sessions"`. That is
    /// deterministic, and the baseline is measured with it, so it ratchets
    /// correctly -- it is not a claim that interpolated sentences are not
    /// wording.
    static func authoredWording(in source: String) -> [String] {
        let chars = Array(source)
        var literals: [(text: String, line: String)] = []
        var i = 0
        var lineStart = 0

        func currentLine() -> String {
            var end = lineStart
            while end < chars.count && chars[end] != "\n" { end += 1 }
            return String(chars[lineStart..<end])
        }

        while i < chars.count {
            if chars[i] == "\n" {
                i += 1
                lineStart = i
                continue
            }
            // Line comment.
            if chars[i] == "/" && i + 1 < chars.count && chars[i + 1] == "/" {
                while i < chars.count && chars[i] != "\n" { i += 1 }
                continue
            }
            // Block comment, nested the way Swift allows.
            if chars[i] == "/" && i + 1 < chars.count && chars[i + 1] == "*" {
                var depth = 1
                i += 2
                while i + 1 < chars.count && depth > 0 {
                    if chars[i] == "/" && chars[i + 1] == "*" { depth += 1; i += 2; continue }
                    if chars[i] == "*" && chars[i + 1] == "/" { depth -= 1; i += 2; continue }
                    if chars[i] == "\n" { lineStart = i + 1 }
                    i += 1
                }
                continue
            }
            // Multiline literal.
            if chars[i] == "\"" && i + 2 < chars.count && chars[i + 1] == "\"" && chars[i + 2] == "\"" {
                let line = currentLine()
                i += 3
                var text = ""
                while i + 2 < chars.count
                    && !(chars[i] == "\"" && chars[i + 1] == "\"" && chars[i + 2] == "\"")
                {
                    if chars[i] == "\\" { i += 2; continue }
                    text.append(chars[i] == "\n" ? " " : chars[i])
                    i += 1
                }
                i += 3
                literals.append((text, line))
                continue
            }
            // Single-line literal.
            if chars[i] == "\"" {
                let line = currentLine()
                i += 1
                var text = ""
                while i < chars.count && chars[i] != "\"" {
                    if chars[i] == "\\" { i += 2; continue }
                    text.append(chars[i])
                    i += 1
                }
                i += 1
                literals.append((text, line))
                continue
            }
            i += 1
        }

        // A message handed to fatalError, an assertion or a log is read by
        // whoever is debugging this and by nobody else. Holding those to the
        // shared copy would be a refinement of nothing.
        let developerFacing = [
            "fatalError", "preconditionFailure", "assertionFailure", "precondition(",
            "assert(", "print(", "NSLog", "os_log", "Logger(", "logger.",
        ]
        return literals
            .filter { literal in !developerFacing.contains { literal.line.contains($0) } }
            .map(\.text)
            .filter(readsAsASentence)
    }

    /// True where the literal reads as a sentence somebody wrote for a
    /// contributor to read.
    static func readsAsASentence(_ literal: String) -> Bool {
        let text = literal.replacingOccurrences(of: "\n", with: " ")
        guard text.contains(" ") else { return false }
        let words = text
            .split(separator: " ")
            .map { token in
                String(token.filter { $0.isLetter || $0 == "'" }).lowercased()
            }
            .filter { !$0.isEmpty }
        guard words.count >= 2 else { return false }
        return words.contains { ShellWordingTests.functionWordsContains($0) }
    }
}

extension ShellWordingTests {
    /// Exposed so `ShellSources` can reach the one list both shells share.
    fileprivate static func functionWordsContains(_ word: String) -> Bool {
        functionWords.contains(word)
    }
}
```

- [ ] **Step 3: Run and watch it fail**

```bash
cd macos && swift test --filter ShellWordingTests
```

Expected: `testNoWordingIsAuthoredInThisShellBeyondTheRecordedBaseline` fails
with a long list of `... authored sentences, baseline allows 0` — one line per
file that authors wording. That failure list is the measurement.

- [ ] **Step 4: Measure and paste the baseline**

```bash
cd macos && TC_WORDING_DUMP=1 swift test --filter testNoWordingIsAuthoredInThisShellBeyondTheRecordedBaseline 2>&1 \
  | grep -E '^\s+".*": [0-9]+,$' | sort
```

Paste the printed lines verbatim into `wordingBaseline`, keeping them sorted.
Group them with short comments the way the Windows baseline does — the
`TCShellCore` copy types, the `TraceCommonsApp` views, the app model — so the
next slice can find its own entries. **Do not adjust a number to make a test
pass.** If a count looks wrong, the scanner is wrong; fix the scanner and
re-measure.

- [ ] **Step 5: Confirm the zero list really is zero**

The three files in `rustOwnedSurfaces` must not appear in the dump at all. If
one does, it is authoring a sentence today and that is a finding: report it in
the PR body and either move the literal into the existing routing bundle or
remove the file from the zero list with a stated reason. Do not give it a
baseline entry.

- [ ] **Step 6: Green, and the ratchet proven in both directions**

```bash
cd macos && swift test --filter ShellWordingTests
```

Then prove the floor by hand before committing: add `let x = "This is a test
sentence for the guard."` to any scanned file, re-run, watch it fail with
"authored sentences, baseline allows"; delete a sentence from a file with an
entry, re-run, watch it fail with "Wording moved out"; revert both.

- [ ] **Step 7: Commit**

```
Count the wording the macOS shell authors
```

---

### Task 2 (slice 0, GTK): the GTK wording ratchet

**Files:**
- Create: `crates/trace-commons-contributor-gtk/tests/shell_wording.rs`
- Nothing else. An integration test in `tests/` needs no manifest entry, and
  the linux CI job already runs
  `cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml`.

**Interfaces:**
- Produces:
  ```rust
  fn scan_shell_sources() -> BTreeMap<String, Vec<String>>;  // path under src/ -> sentences
  fn authored_wording(source: &str) -> Vec<String>;
  fn reads_as_a_sentence(literal: &str) -> bool;
  const WORDING_BASELINE: &[(&str, usize)];
  const RUST_OWNED_SURFACES: &[&str];
  #[test] fn no_wording_is_authored_in_this_shell_beyond_the_recorded_baseline();
  #[test] fn the_rust_owned_surfaces_are_not_given_a_wording_allowance();
  ```
- Consumes: nothing from the crate. It reads
  `crates/trace-commons-contributor-gtk/src/**/*.rs` from `CARGO_MANIFEST_DIR`.
  Deliberately no `use trace_commons_contributor_gtk::...`: the guard is about
  source text, and importing the crate would tempt a later edit into asserting
  against the constants instead of against what is written.

**The decision this task has to make, and its answer.** GTK's `copy.rs` *is*
the migration target: the 3,188-line file the other two shells historically
transcribed from. So "authored in the shell" for GTK is defined as **every
string literal outside a `COPY-MIGRATED` region and outside a `#[cfg(test)]`
module that reads as a sentence — the `pub const` copy constants included**.
Those constants are exactly the sentences slices 2 through 7 have to move; a
definition that exempted them would leave the GTK ratchet with nothing to
count and the guard passing over the whole point. A `pub use` re-export is not
a literal, so migrating a constant lowers the number by itself, which is the
property the ratchet needs.

Test modules are excluded because a test's fixture text is not shown to a
contributor, and `copy.rs` interleaves three `#[cfg(test)]` modules with real
copy (lines 1985, 2124 and 3157 today) — so this cannot be a "cut the file at
the first test module" rule and is implemented as a brace-matched skip.

- [ ] **Step 1: Write the scanner and the two failing tests**

Create `crates/trace-commons-contributor-gtk/tests/shell_wording.rs`:

```rust
//! Wording authored in this shell, over the whole shell.
//!
//! The mirror of `ShellWordingTests.cs` on Windows and
//! `ShellWordingTests.swift` on macOS, for the same reason: a sentence
//! hand-written in one shell survives a rename in the other two, and until
//! this file existed nothing here noticed.
//!
//! What counts as authored here needs saying, because this shell is the one
//! the other two transcribed from. `copy.rs` is the migration target, so its
//! `pub const` sentences are counted like any others. A `pub use` re-export
//! is not a literal, so moving a constant into
//! `trace_commons_contributor::*_copy` lowers the number by itself -- which
//! is exactly the ratchet the migration needs.
//!
//! It is a ratchet, not a clean bill of health.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Files that author wording today, and exactly how much.
///
/// TODO(shell-copy): every entry here is a file whose wording should be
/// composed in `trace_commons_contributor` and re-exported, the way
/// `routing_copy` and `witness_copy` already are. Until then the number is a
/// CEILING AND A FLOOR both: adding a sentence fails, and removing one fails
/// too, so the entry has to be lowered deliberately as copy moves out. Never
/// raise a number. A new file must never be added here.
///
/// MEASURED, NOT ESTIMATED. Every number came from this file's own scanner
/// via `TC_WORDING_DUMP=1`; none was typed by hand.
const WORDING_BASELINE: &[(&str, usize)] = &[
    // PASTE THE DUMP HERE IN STEP 3.
];

/// The surfaces whose wording already comes from the shared crate. Nothing
/// may ever buy them an allowance in the baseline.
///
/// `ui/style.rs` and `ui/css_contract.rs` are not here: they hold CSS, which
/// is not wording, and the scanner does not count it.
const RUST_OWNED_SURFACES: &[&str] = &[];

/// Words a sentence has and an identifier, a wire key, a CSS class or a
/// format pattern does not.
///
/// The same list the Windows and macOS guards use, deliberately: three
/// shells counting the same corpus by different rules would produce three
/// numbers nobody could compare.
const FUNCTION_WORDS: &str = "\
a an and are as at be been being but by can cannot could did do does for from \
had has have how if in into is isn't it it's its just may never no not nothing of off on once only or \
so some still such than that the their them then there they this those to until up was we were what \
when where which while who will with would you your yours yet anything something everything already \
always about after again all any because before both each else ever every here more most much \
must need needs same see should since take takes tell these too under use used using very";

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every Rust source of this shell, keyed by its path relative to `src/`.
fn scan_shell_sources() -> BTreeMap<String, Vec<String>> {
    let root = src_root();
    let mut scanned = BTreeMap::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("the shell's sources are readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let relative = path
                .strip_prefix(&root)
                .expect("every scanned path is under src/")
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(&path).expect("a readable source");
            scanned.insert(relative, authored_wording(&source));
        }
    }
    scanned
}

/// The sentences a Rust source authors.
///
/// A character walk, for the reasons the routing sweep's scanner gives and
/// two more this file needs: raw strings hold CSS with braces in it, which
/// would wreck the brace counting below, and `#[cfg(test)]` modules are
/// interleaved with real copy in `copy.rs` rather than sitting at the end.
fn authored_wording(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let mut literals = Vec::new();
    let mut i = 0;
    // Set when `#[cfg(test)]` is seen; consumed by the next `{`, which
    // records the depth to skip back down to.
    let mut test_attribute_pending = false;
    let mut depth: usize = 0;
    let mut skip_below: Option<usize> = None;

    while i < chars.len() {
        // Raw string, borrowed or not: r"..." and r#"..."# .
        if chars[i] == 'r' && i + 1 < chars.len() && (chars[i + 1] == '"' || chars[i + 1] == '#') {
            let mut hashes = 0;
            let mut j = i + 1;
            while j < chars.len() && chars[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if j < chars.len() && chars[j] == '"' {
                j += 1;
                'raw: while j < chars.len() {
                    if chars[j] == '"' {
                        let mut closing = 0;
                        while closing < hashes && j + 1 + closing < chars.len()
                            && chars[j + 1 + closing] == '#'
                        {
                            closing += 1;
                        }
                        if closing == hashes {
                            j += 1 + hashes;
                            break 'raw;
                        }
                    }
                    j += 1;
                }
                // Raw strings in this shell are CSS and shell-out fixtures,
                // never sentences. Skipped whole rather than scanned.
                i = j;
                continue;
            }
        }
        // Line comment. Prose about the wire may quote it, and nothing in a
        // comment is rendered.
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment.
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        // A char literal, so that '"' and '{' cannot unbalance the scan.
        // Guarded so that the lifetime in `&'static str` is not read as one.
        if chars[i] == '\''
            && i + 2 < chars.len()
            && (chars[i + 2] == '\'' || chars[i + 1] == '\\')
        {
            i += 2;
            while i < chars.len() && chars[i] != '\'' {
                i += 1;
            }
            i += 1;
            continue;
        }
        if chars[i] == '#' && chars[i..].starts_with(&"#[cfg(test)]".chars().collect::<Vec<_>>()[..])
        {
            test_attribute_pending = true;
            i += "#[cfg(test)]".len();
            continue;
        }
        if chars[i] == '{' {
            depth += 1;
            if test_attribute_pending && skip_below.is_none() {
                skip_below = Some(depth);
                test_attribute_pending = false;
            }
            i += 1;
            continue;
        }
        if chars[i] == '}' {
            if let Some(floor) = skip_below {
                if depth == floor {
                    skip_below = None;
                }
            }
            depth = depth.saturating_sub(1);
            i += 1;
            continue;
        }
        if chars[i] == '"' {
            i += 1;
            let mut literal = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    // Only the escapes this shell uses: a line continuation,
                    // whose payload is whitespace, and an escaped quote.
                    // Both fold to a space rather than being decoded,
                    // because this reads words and not punctuation.
                    i += 2;
                    literal.push(' ');
                    continue;
                }
                literal.push(if chars[i] == '\n' { ' ' } else { chars[i] });
                i += 1;
            }
            i += 1;
            if skip_below.is_none() && reads_as_a_sentence(&literal) {
                literals.push(literal);
            }
            continue;
        }
        i += 1;
    }
    literals
}

/// True where the literal reads as a sentence somebody wrote for a
/// contributor to read.
fn reads_as_a_sentence(literal: &str) -> bool {
    if !literal.contains(' ') {
        return false;
    }
    let function_words: Vec<&str> = FUNCTION_WORDS.split_whitespace().collect();
    let words: Vec<String> = literal
        .split_whitespace()
        .map(|token| {
            token
                .chars()
                .filter(|c| c.is_ascii_alphabetic() || *c == '\'')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect();
    words.len() >= 2 && words.iter().any(|w| function_words.contains(&w.as_str()))
}

/// No file in this shell authors more wording than it did when this guard
/// was written, and no file starts authoring wording that did not.
#[test]
fn no_wording_is_authored_in_this_shell_beyond_the_recorded_baseline() {
    let scanned = scan_shell_sources();

    // A scan that found nothing would turn this test into a pass over
    // nothing, which is the failure mode the Windows guard names by name.
    // There are 33 sources under src/ today.
    assert!(
        scanned.len() >= 30,
        "only {} sources were scanned under {}; the whole tree is expected",
        scanned.len(),
        src_root().display()
    );
    assert!(
        scanned.contains_key("copy.rs"),
        "copy.rs was not scanned; it is the file this ratchet exists for"
    );

    if std::env::var("TC_WORDING_DUMP").as_deref() == Ok("1") {
        for (path, wording) in &scanned {
            if !wording.is_empty() {
                println!("    (\"{path}\", {}),", wording.len());
            }
        }
    }

    let baseline: BTreeMap<&str, usize> = WORDING_BASELINE.iter().copied().collect();
    let mut failures = Vec::new();
    for (path, wording) in &scanned {
        let allowed = baseline.get(path.as_str()).copied().unwrap_or(0);
        if wording.len() == allowed {
            continue;
        }
        if wording.len() > allowed {
            failures.push(format!(
                "{path}: {} authored sentences, baseline allows {allowed}. \
                 First one over the line: {:?}",
                wording.len(),
                wording[allowed]
            ));
        } else {
            failures.push(format!(
                "{path}: {} authored sentences, baseline still allows {allowed}. \
                 Wording moved out -- lower the entry (or delete it at zero).",
                wording.len()
            ));
        }
    }
    for recorded in baseline.keys() {
        if !scanned.contains_key(*recorded) {
            failures.push(format!(
                "{recorded}: recorded in the baseline but no longer in the shell. Delete the entry."
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Wording on this shell's surfaces should come from trace_commons_contributor.\n\
         A sentence written here is one the other two shells will not get, and one a rename \
         in the shared crate will not reach.\n\n{}",
        failures.join("\n")
    );
}

/// The surfaces the shared crate already owns hold no wording at all, and
/// hold no baseline entry either.
#[test]
fn the_rust_owned_surfaces_are_not_given_a_wording_allowance() {
    let scanned = scan_shell_sources();
    let baseline: BTreeMap<&str, usize> = WORDING_BASELINE.iter().copied().collect();
    for surface in RUST_OWNED_SURFACES {
        assert!(
            !baseline.contains_key(surface),
            "{surface} has a wording baseline entry. Its wording comes from the shared crate; \
             an allowance here would quietly undo that."
        );
        let wording = scanned
            .get(*surface)
            .unwrap_or_else(|| panic!("{surface} was not scanned; the guard would pass over nothing"));
        assert!(wording.is_empty(), "{surface} authors wording: {wording:?}");
    }
}
```

`RUST_OWNED_SURFACES` starts empty on purpose: no GTK *file* is wording-free
today, because `copy.rs` mixes migrated re-exports with unmigrated constants in
one file. The file-level zero list is not the mechanism for GTK — the
region-level `COPY-MIGRATED` sweep in Task 5 is, and it is what the empty list
grows into. The test is still worth having now: it fails loudly the moment
somebody adds a name to the list without the file actually being clean.

- [ ] **Step 2: Run and watch it fail**

```bash
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml --test shell_wording
```

Expected: `no_wording_is_authored_in_this_shell_beyond_the_recorded_baseline`
fails with one line per file that authors wording, `copy.rs` far in the lead.

- [ ] **Step 3: Measure and paste the baseline**

```bash
TC_WORDING_DUMP=1 cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml \
  --test shell_wording -- --nocapture no_wording_is_authored \
  | grep -E '^\s+\("' | sort
```

Paste verbatim into `WORDING_BASELINE`. **Do not adjust a number to make a test
pass.**

- [ ] **Step 4: Green, formatted, and the ratchet proven in both directions**

```bash
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml --test shell_wording
cargo fmt --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml -- --check
```

Then, by hand: add a sentence literal to `src/toast.rs`, re-run, watch it fail;
delete one from `src/copy.rs`, re-run, watch it fail the other way; revert both.

- [ ] **Step 5: Report the first real count**

The spec's open question 5 asks whether the macOS and GTK counts are much larger
than Windows' 390, because the slicing depends on it. Put the three totals
(Windows 390, macOS from Task 1, GTK from this task) in the PR body.

- [ ] **Step 6: Commit**

```
Count the wording the GTK shell authors
```

---

### Task 3 (ReadGate, core): `consent_copy.rs`

**Files:**
- Create: `crates/trace-commons-contributor/src/consent_copy.rs`
- Modify: `crates/trace-commons-contributor/src/lib.rs` (declare the module,
  in alphabetical order between `consent` and `daemon`)

**Interfaces:**
- Produces:
  ```rust
  pub const GATE_STATEMENT: &str;
  pub const GATE_READY_HELP: &str;
  pub const GATE_NOT_PINNED_HELP: &str;

  #[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
  pub struct ConsentCopy {
      pub gate_statement: &'static str,
      pub ready_help: &'static str,
      pub not_pinned_help: &'static str,
  }

  #[must_use] pub fn consent_copy() -> ConsentCopy;
  #[must_use] pub fn gate_help(pinned: bool) -> &'static str;
  ```
- Consumes: nothing. The module is constants and one branch.

**Naming.** The spec's slice table calls this `consent_copy.rs`; that decision
stands. The neighbouring `consent.rs` is unrelated — it validates upload-claim
consent scopes and holds no copy — so there is no collision, only adjacency, and
the module docs below say so.

**Fields, not one export per sentence.** Three sentences is not three exports,
for the reason `routing_copy` gives: a per-sentence export lets a shell take two
of the three and hand-write the third, and each one is a `NULL` a shell needs a
policy for.

- [ ] **Step 1: Write the failing tests**

At the bottom of `crates/trace-commons-contributor/src/consent_copy.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// The statement, character for character.
    ///
    /// Written out here rather than compared to itself: this is the claim
    /// the product makes about redaction at the instant of consent, and the
    /// point of the assertion is that changing the sentence is a decision
    /// somebody has to make twice. This is the assertion the three shells
    /// used to hold one copy of each.
    ///
    /// The expectation is one unbroken literal on purpose. A `\` line
    /// continuation here would swallow the following indentation and pin a
    /// sentence with the wrong spacing in it, which is the shape of bug this
    /// assertion exists to catch.
    #[test]
    fn the_consent_statement_is_exactly_what_was_agreed() {
        assert_eq!(
            GATE_STATEMENT,
            "\"Exactly what would be sent\" is the exact text that would leave this machine. Pattern-based scrubbing may have missed something in it, and nothing here checks that you looked."
        );
    }

    /// The two things the removed checkbox used to make a contributor say
    /// out loud. Neither may quietly drop out of the sentence.
    #[test]
    fn the_statement_keeps_both_halves_of_what_the_checkbox_used_to_say() {
        assert!(GATE_STATEMENT.contains("Pattern-based scrubbing may have missed something"));
        assert!(GATE_STATEMENT.contains("nothing here checks that you looked"));
    }

    /// The branch crosses, not only the words.
    ///
    /// Without this function each shell keeps its own `? :` between the two
    /// help sentences, and three copies of a two-way branch can drift apart
    /// silently while every string stays identical.
    #[test]
    fn the_help_sentence_is_chosen_here_and_not_in_a_shell() {
        assert_eq!(gate_help(true), GATE_READY_HELP);
        assert_eq!(gate_help(false), GATE_NOT_PINNED_HELP);
    }

    /// The not-pinned sentence explains why the button is off without
    /// claiming the app knows something it does not.
    #[test]
    fn the_not_pinned_sentence_names_the_condition_the_shells_actually_test() {
        assert!(GATE_NOT_PINNED_HELP.contains("isn't connected yet"));
        assert!(GATE_NOT_PINNED_HELP.contains("nothing here can be contributed"));
        // Not a promise that pressing it later will work, and not an error.
        assert!(!GATE_NOT_PINNED_HELP.to_lowercase().contains("failed"));
        assert!(!GATE_NOT_PINNED_HELP.to_lowercase().contains("try again"));
    }

    /// Every field of the payload is a non-empty sentence, and the payload
    /// is exactly the three of them.
    ///
    /// Both shells refuse the whole payload when a field is empty, so an
    /// empty field here would blank a screen rather than fail a build.
    #[test]
    fn the_payload_is_three_non_empty_sentences() {
        let value = serde_json::to_value(consent_copy()).expect("the payload serialises");
        let object = value.as_object().expect("a JSON object");
        let mut keys: Vec<&String> = object.keys().collect();
        keys.sort();
        assert_eq!(keys, ["gate_statement", "not_pinned_help", "ready_help"]);
        for (field, value) in object {
            assert!(
                !value.as_str().expect("every field is a string").is_empty(),
                "{field} is empty"
            );
        }
    }
}
```

What may not change in that first test is that the expected text is spelled out
rather than compared to `GATE_STATEMENT` itself. Comparing a constant to itself
asserts nothing; the point is that changing this sentence takes two deliberate
edits. This is the assertion the three shells used to hold one copy of each, and
it is now held once.

- [ ] **Step 2: Run and watch them fail**

```bash
cargo test -p trace-commons-contributor --lib consent_copy
```

Expected: compile error — the module does not exist.

- [ ] **Step 3: Write the module**

```rust
//! The consent surface's words, in one place, for all three shells.
//!
//! Four sentences, and the highest-value four in the app: they are the
//! safety claim shown at the instant of consent, above an irreversible
//! button. Until this module existed the claim was written three times --
//! `windows/src/TraceCommons.Interop/ReadGate.cs`,
//! `macos/Sources/TCShellCore/ReadGate.swift` and the GTK shell's
//! `copy.rs` -- and held together by a Rust test that opened the other two
//! shells' source files and grepped them for the exact text. That scaffold
//! is O(n) hand-written needles and only ever covered the sentences
//! somebody remembered to add.
//!
//! Not to be confused with `crate::consent`, which validates upload-claim
//! consent scopes and holds no copy at all.
//!
//! # What crosses the boundary
//!
//! The sentences cross already assembled, and so does the *branch*. A shell
//! does not receive two help sentences and choose between them: it calls
//! [`gate_help`] -- across the ABI, `tc_consent_gate_help` -- and receives
//! the chosen one. Three native copies of a two-way branch drift apart
//! silently while every string they return stays identical, which is the
//! failure this module exists to remove.
//!
//! GTK links this crate directly and re-exports these names; the macOS and
//! Windows shells reach them through `tc_consent_copy` and
//! `tc_consent_gate_help`.

/// The sentence that replaced the acknowledgement checkbox.
///
/// `Contribute` used to wait on three things: a pinned preview, the
/// "Exactly what would be sent" text having been on screen, and an
/// acknowledgement ticked by hand. Two of them are gone -- the checkbox as
/// friction, and the transcript-shown condition with it, because a queue
/// row's Submit approves the same session with no preview opened at all, so
/// the gate never stood between anybody and a blind approval.
///
/// What the checkbox *said* is not gone. It is this sentence, and it keeps
/// both halves of what the old gate was honest about: scrubbing is
/// pattern-based and may have missed something, and nothing in the app can
/// tell whether anyone read anything. Do not shorten it for layout; change
/// the layout.
pub const GATE_STATEMENT: &str = "\"Exactly what would be sent\" is the exact text that would leave this machine. Pattern-based scrubbing may have missed something in it, and nothing here checks that you looked.";

/// The tooltip on an armed `Contribute`.
///
/// The whole claim in four words: this button sends this session, and it
/// does not do anything else.
pub const GATE_READY_HELP: &str = "Sends this session. Nothing else.";

/// Why `Contribute` is off.
///
/// An approval binds to the envelope a preview pinned, and a preview built
/// without an enrollment pinned nothing, so there is nothing for an
/// approval to cover. Saying that beats a button that fails when pressed.
///
/// # The divergence this sentence settles
///
/// Windows said this ("This device isn't connected yet...") and macOS said
/// something else ("This preview hasn't loaded yet, so there is nothing
/// here to contribute.") -- two shells, two different explanations of why
/// the same button is off, because the two shells were also testing two
/// different conditions. This wording is the one that survived, for two
/// reasons: it names the condition both shells now test (an enrolled,
/// pinned preview), and the GTK shell already prints a near-identical
/// sentence in `UNENROLLED_PREVIEW`, so choosing it leaves one story rather
/// than two. The macOS condition moved to match; see the migration plan's
/// Task 7.
pub const GATE_NOT_PINNED_HELP: &str = "This device isn't connected yet, so this preview was built without your identity and nothing here can be contributed.";

/// Every fixed string on this surface, in one payload.
///
/// Shaped for the C ABI: `tc_consent_copy` serialises this and hands the
/// shell one owned JSON object. One call and not one per string -- a
/// per-string export would let a shell take two of the three sentences and
/// hand-write the third, and the third is a claim about what leaves the
/// machine.
///
/// No version field, deliberately. A version implies a shell that can serve
/// two of them, and the cdylib and the shell ship together in one DMG, one
/// MSIX, one Flatpak. What is actually needed -- detection of a field that
/// stopped being exported -- is what each shell's refuse-on-any-empty-field
/// decode already does.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct ConsentCopy {
    pub gate_statement: &'static str,
    pub ready_help: &'static str,
    pub not_pinned_help: &'static str,
}

/// The payload, built from the constants above.
#[must_use]
pub fn consent_copy() -> ConsentCopy {
    ConsentCopy {
        gate_statement: GATE_STATEMENT,
        ready_help: GATE_READY_HELP,
        not_pinned_help: GATE_NOT_PINNED_HELP,
    }
}

/// The tooltip that explains the current answer.
///
/// THE BRANCH CROSSES, NOT ONLY THE WORDS. A shell that received both
/// sentences and chose between them would be keeping a third copy of this
/// decision, in a third language, with nothing to notice when one of them
/// stops matching. `pinned` is the shell's one condition: a preview that
/// parsed and carries an enrollment.
#[must_use]
pub fn gate_help(pinned: bool) -> &'static str {
    if pinned {
        GATE_READY_HELP
    } else {
        GATE_NOT_PINNED_HELP
    }
}
```

In `crates/trace-commons-contributor/src/lib.rs`, add `pub mod consent_copy;`
immediately after `pub mod consent;`.

- [ ] **Step 4: Green, and every configuration CI gates**

```bash
cargo test -p trace-commons-contributor --lib consent_copy
RUSTFLAGS='-D warnings' cargo check -p trace-commons-contributor --bins
cargo check -p trace-commons-contributor --no-default-features
cargo clippy -p trace-commons-contributor --all-targets -- \
  -A clippy::type_complexity -A clippy::collapsible_if \
  -A clippy::manual_option_as_slice -A clippy::useless_vec \
  -A clippy::redundant_pattern_matching
cargo fmt --all
```

The three shells still hold their own literals at this point. That duplication
is deliberate and lasts three commits; nothing breaks, because the GTK parity
test still finds the text in all three files.

- [ ] **Step 5: Commit**

```
Compose the consent surface's four sentences once
```

---

### Task 4 (ReadGate, ABI): two exports and both headers

**Files:**
- Modify: `crates/trace-commons-contributor-ffi/src/lib.rs`
- Modify: `crates/trace-commons-contributor-ffi/include/trace_commons.h`
- Modify: `macos/Sources/CTraceCommons/include/trace_commons.h` (by `cp`)
- Modify: `crates/trace-commons-contributor-ffi/tests/abi.rs`

**Interfaces:**
- Produces:
  ```rust
  #[unsafe(no_mangle)] pub extern "C" fn tc_consent_copy() -> *mut c_char;
  #[unsafe(no_mangle)] pub extern "C" fn tc_consent_gate_help(pinned: i32) -> *mut c_char;
  ```
  ```c
  char*       tc_consent_copy(void);
  char*       tc_consent_gate_help(int32_t pinned);
  ```
- Consumes: `trace_commons_contributor::consent_copy::{consent_copy, gate_help}`
  from Task 3, plus the crate's existing `guarded_string_no_err` and
  `to_owned_cstring` helpers and `tc_string_free` for the caller.

**The unknown-value policy, which is surface-specific.** Routing's unknown
values become Neutral; witness's become Refused. This surface's is: **only `1`
means pinned. `0`, and any value this build has never heard of, get the
not-pinned sentence.** That is the fail-closed direction here — a shell built
against a later header, or one that passed a value from a state this build does
not know, must not be told the button is armed and safe. It is written into the
header so a shell author reads it before mapping a bool.

- [ ] **Step 1: Write the failing ABI tests**

In `crates/trace-commons-contributor-ffi/tests/abi.rs`, add
`tc_consent_copy, tc_consent_gate_help` to the existing `use` list at the top
and append:

```rust
#[test]
fn the_consent_bundle_crossing_the_abi_is_the_one_in_the_rust() {
    // The whole point of the slice: the sentences a shell renders at the
    // moment of consent are the ones this repo defines, not a transcription
    // in Swift or C# that stops matching the day one of them changes.
    let json = take_owned(tc_consent_copy());
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    // Compared against the payload itself, not against words written here:
    // pinning the literals in this test would be the same transcription bug
    // one layer down. The shells' own suites pin nothing either -- they
    // assert the field set, which is what a rename must break.
    let expected = serde_json::to_value(trace_commons_contributor::consent_copy::consent_copy())
        .expect("the payload serialises");
    assert_eq!(
        parsed, expected,
        "the ABI must hand over the payload unchanged"
    );
}

#[test]
fn the_gate_help_branch_crosses_the_abi() {
    use trace_commons_contributor::consent_copy as copy;
    assert_eq!(take_owned(tc_consent_gate_help(1)), copy::GATE_READY_HELP);
    assert_eq!(
        take_owned(tc_consent_gate_help(0)),
        copy::GATE_NOT_PINNED_HELP
    );
}

#[test]
fn an_unknown_pinned_value_is_not_told_the_button_is_armed() {
    // Fail-closed, and specific to this surface: routing's unknown values
    // answer Neutral because Neutral claims nothing, but there is no
    // sentence here that claims nothing. The one that claims less is the
    // one an unknown value gets.
    use trace_commons_contributor::consent_copy as copy;
    for value in [-1, 2, 7, i32::MIN, i32::MAX] {
        assert_eq!(
            take_owned(tc_consent_gate_help(value)),
            copy::GATE_NOT_PINNED_HELP,
            "{value} must not arm the button"
        );
    }
}
```

- [ ] **Step 2: Run and watch them fail**

```bash
cargo test -p trace-commons-contributor-ffi --test abi consent
```

Expected: unresolved imports — the exports do not exist.

- [ ] **Step 3: Add the exports**

In `crates/trace-commons-contributor-ffi/src/lib.rs`, immediately after
`tc_routing_last_checked`'s definition (keeping the copy exports together):

```rust
/// Every fixed sentence on the consent surface, in one call.
///
/// Needs no handle: it describes the build, not a running daemon.
///
/// Returns an owned JSON object whose keys are `ConsentCopy`'s fields; free
/// it with [`tc_string_free`].
///
/// ONE CALL, NOT ONE PER SENTENCE. Three sentences is not three exports: a
/// per-sentence export would let a shell take two of them and hand-write the
/// third, and one of the three is the claim about what leaves this machine
/// that a contributor reads immediately above an irreversible button.
///
/// Returns NULL only on a caught panic.
#[unsafe(no_mangle)]
pub extern "C" fn tc_consent_copy() -> *mut c_char {
    guarded_string_no_err(|| {
        let copy = trace_commons_contributor::consent_copy::consent_copy();
        let json = serde_json::to_string(&copy).unwrap_or_else(|_| "{}".to_string());
        Ok(to_owned_cstring(&json))
    })
}

/// The tooltip that explains why `Contribute` is armed or off.
///
/// `pinned` is 1 when a preview parsed and carries an enrollment, and 0
/// otherwise. ANY OTHER VALUE IS NOT PINNED -- see the header. Routing's
/// unknown values answer the tone that claims nothing; there is no sentence
/// here that claims nothing, so an unknown value gets the one that claims
/// less.
///
/// THE BRANCH CROSSES, NOT ONLY THE WORDS. Without this call each shell
/// keeps its own two-way choice between the sentences from
/// [`tc_consent_copy`], and three copies of that choice drift apart in
/// silence while every string stays identical.
///
/// Returns an owned string; free it with [`tc_string_free`]. Returns NULL
/// only on a caught panic.
#[unsafe(no_mangle)]
pub extern "C" fn tc_consent_gate_help(pinned: i32) -> *mut c_char {
    guarded_string_no_err(|| {
        let line = trace_commons_contributor::consent_copy::gate_help(pinned == 1);
        Ok(to_owned_cstring(line))
    })
}
```

- [ ] **Step 4: Declare them in the header, then copy it**

In `crates/trace-commons-contributor-ffi/include/trace_commons.h`, immediately
after the `char*       tc_source_check_line(const char* tool, const char* source_mode);`
declaration:

```c
/* Every fixed sentence on the consent surface, as an owned JSON object; free
 * it with tc_string_free. NULL only on a caught panic.
 *
 * Keys: gate_statement, ready_help, not_pinned_help.
 *
 * ONE CALL, NOT ONE PER SENTENCE. Three sentences is not three exports: a
 * per-sentence export would let a shell take two of them and hand-write the
 * third, and one of the three is the claim about what leaves this machine that
 * a contributor reads immediately above an irreversible button.
 *
 * Refuse the WHOLE payload if any field is empty rather than rendering a blank
 * label. A missing sentence here is a missing claim.
 */
char*       tc_consent_copy(void);

/* The tooltip that explains why Contribute is armed or off, chosen here.
 *
 * pinned is 1 when a preview parsed and carries an enrollment, and 0
 * otherwise.
 *
 * ONLY 1 IS PINNED. 0 and every other value answer the not-pinned sentence.
 * This is the fail-closed direction on this surface and it is not routing's:
 * routing answers Neutral for a value it does not know because Neutral claims
 * nothing, and there is no sentence here that claims nothing. A shell built
 * against a later header must not be told the button is armed.
 *
 * THE BRANCH CROSSES, NOT ONLY THE WORDS. Do not take both sentences from
 * tc_consent_copy and choose between them natively: three copies of that
 * choice drift apart in silence while every string stays identical.
 *
 * Returns an owned string; free it with tc_string_free. NULL only on a caught
 * panic.
 */
char*       tc_consent_gate_help(int32_t pinned);
```

Then, from the repository root:

```bash
cp crates/trace-commons-contributor-ffi/include/trace_commons.h \
   macos/Sources/CTraceCommons/include/trace_commons.h
```

- [ ] **Step 5: Green, including the header identity test**

```bash
cargo test -p trace-commons-contributor-ffi --test abi consent
cargo test -p trace-commons-contributor-ffi --test abi_header_surface
RUSTFLAGS='-D warnings' cargo check -p trace-commons-contributor-ffi
cargo check -p trace-commons-contributor-ffi --no-default-features
cargo fmt --all
```

`abi_header_surface.rs` derives the expected surface from the Rust, so a
declaration missing from either copy, or differing between them, fails here.
There is no list to maintain.

- [ ] **Step 6: Commit**

```
Export the consent surface's sentences across the C ABI
```

---

### Task 5 (ReadGate, GTK): re-export inside a marked region

**Files:**
- Modify: `crates/trace-commons-contributor-gtk/Cargo.toml` — nothing to add;
  `trace-commons-contributor` is already a path dependency. Confirm and move on.
- Modify: `crates/trace-commons-contributor-gtk/src/copy.rs`
- Modify: `crates/trace-commons-contributor-gtk/tests/shell_wording.rs`
  (the marker sweep, and the `copy.rs` baseline entry)

**Interfaces:**
- Produces: `crates/trace-commons-contributor-gtk/src/copy.rs` re-exports
  `GATE_STATEMENT`, `GATE_READY_HELP`, `GATE_NOT_PINNED_HELP` and `gate_help`
  from `trace_commons_contributor::consent_copy`.
- Consumes: Task 3's module. GTK does not touch the ABI and never will — the C
  ABI would buy it nothing but marshalling, which is a spec non-goal.
- Unchanged: `src/ui/preview.rs:441` still reads `copy::GATE_STATEMENT`. The
  re-export keeps the call site identical, which is the point.

**Intentionally unused, and recorded as such:** GTK has no tooltip on
`Contribute`, so `GATE_READY_HELP`, `GATE_NOT_PINNED_HELP` and `gate_help` are
re-exported and not rendered. The spec asks for intentionally unused fields to
be documented; the doc comment below is that documentation. They are re-exported
rather than omitted so that a GTK screen that later grows a tooltip reaches for
the shared sentence instead of writing a fourth one.

- [ ] **Step 1: Write the failing marker sweep**

Append to `crates/trace-commons-contributor-gtk/tests/shell_wording.rs`:

```rust
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
```

- [ ] **Step 2: Run and watch it fail**

```bash
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml \
  --test shell_wording a_migrated_region
```

Expected: `copy.rs has no COPY-MIGRATED region`.

- [ ] **Step 3: Replace the constant with a marked re-export**

In `crates/trace-commons-contributor-gtk/src/copy.rs`, delete the
`GATE_STATEMENT` constant and its doc comment (line 552 today, with the doc
block above it) and put this in its place:

```rust
// --- The consent surface ------------------------------------------------
//
// The sentence printed above `Contribute`, and the two tooltips beside it.
//
// The words are NOT here. They live in
// `trace_commons_contributor::consent_copy`, because the macOS and Windows
// shells print the same claim and reach it across the C ABI, and a claim
// about what leaves this machine kept in three places is three claims that
// have not diverged yet.
//
// `GATE_READY_HELP`, `GATE_NOT_PINNED_HELP` and `gate_help` are re-exported
// and not rendered: this shell puts no tooltip on `Contribute`. That is
// deliberate rather than an oversight -- they are here so that a screen
// which later grows one reaches for the shared sentence instead of writing
// a fourth.
//
// COPY-MIGRATED-BEGIN
//
// Everything between this marker and COPY-MIGRATED-END is swept by
// `a_migrated_region_of_copy_rs_holds_no_words_of_its_own`, which reads this
// file. The region may hold `pub use` and nothing else: a literal left
// beside a re-export is the word this shell would render while the other
// two render the shared one.
pub use trace_commons_contributor::consent_copy::{
    GATE_NOT_PINNED_HELP, GATE_READY_HELP, GATE_STATEMENT, gate_help,
};
// COPY-MIGRATED-END
```

`src/ui/preview.rs` is untouched: `copy::GATE_STATEMENT` still resolves.

- [ ] **Step 4: Ratchet the baseline down**

In `tests/shell_wording.rs`, lower the `("copy.rs", N)` entry by exactly 1 — the
one literal that left. Do not touch any other entry. Re-run the ratchet; it
fails if the number is off by one in either direction, which is the check.

- [ ] **Step 5: The parity test still passes, and that is expected**

`the_three_shells_print_the_same_statement` in `copy.rs`'s own test module reads
`GATE_STATEMENT` (now the re-export) and greps the Swift and C# sources for it.
Both still hold their transcriptions at this point, so it passes unchanged.
Task 6 narrows it and Task 7 deletes it.

- [ ] **Step 6: Green**

```bash
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml
cargo build --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml --bin trace-commons-shell
cargo fmt --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml -- --check
```

- [ ] **Step 7: Commit**

```
Read the consent statement from the shared crate on Linux
```

---

### Task 6 (ReadGate, Windows): decode it, and settle the divergence

**Files:**
- Create: `windows/src/TraceCommons.Interop/ConsentCopy.cs`
- Create: `windows/src/TraceCommons.Interop/ConsentSurface.cs`
- Modify: `windows/src/TraceCommons.Interop/ReadGate.cs` (the four sentences out)
- Modify: `windows/src/TraceCommons.Interop/NativeMethods.cs` (two `DllImport`s)
- Modify: `windows/src/TraceCommons.App/ViewModels/PreviewSheetViewModel.cs`
- Create: `windows/tests/TraceCommons.Interop.Tests/ConsentCopyTests.cs`
- Modify: `windows/tests/TraceCommons.Interop.Tests/PreviewTests.cs`
- Modify: `windows/tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj`
  (three `.cs.txt` copies)
- Modify: `windows/tests/TraceCommons.Interop.Tests/ShellWordingTests.cs`
  (baseline entry out, three names into `RustOwnedSurfaces`)
- Modify: `crates/trace-commons-contributor-gtk/src/copy.rs` (narrow the parity test)

**Interfaces:**
- Produces:
  ```csharp
  public sealed record ConsentCopy
  {
      [JsonPropertyName("gate_statement")] public string GateStatement { get; init; }
      [JsonPropertyName("ready_help")] public string ReadyHelp { get; init; }
      [JsonPropertyName("not_pinned_help")] public string NotPinnedHelp { get; init; }
      public string[] Sentences { get; }
      public static IReadOnlyList<string> ConsumedFields { get; }
  }

  public static class ConsentSurface
  {
      public static ConsentCopy? Copy();
      internal static ConsentCopy? Parse(string? json);
      public static string? GateHelp(bool pinned);
  }
  ```
  and `ReadGate` reduced to the rule: `HasPinnedPreview`, `CanContribute`,
  `Changed`, `SetPinnedPreview(bool)`, `Reset()`. `Statement`, `ReadyHelp`,
  `UnenrolledHelp` and `Help` are gone from it.
- Consumes: `tc_consent_copy` and `tc_consent_gate_help` from Task 4, through
  `NativeMethods.TakeOwnedString`.

**The reviewable decision in this PR, stated plainly.** Windows'
`UnenrolledHelp` and macOS' `notPinnedHelp` had diverged: two shells, two
different explanations of why the same button is off. **This slice keeps the
Windows wording verbatim** — "This device isn't connected yet, so this preview
was built without your identity and nothing here can be contributed." — because
it names the condition the shells actually test, and because GTK already prints
a near-identical sentence in `UNENROLLED_PREVIEW`, so one story survives instead
of two. The alternative (keep macOS' "This preview hasn't loaded yet...", which
is false of a preview that loaded but is unenrolled) is recorded in the PR body
so a maintainer can overrule it in one place. The condition moves to match in
Task 7.

**Why the help sentence leaves `ReadGate` entirely.** `ReadGate` is testable
without the cdylib, and that is load-bearing: this suite runs on developers'
macOS machines. Keeping a `Help` property on it would either re-author the
sentence or make the rule depend on the dylib. So the rule stays in `ReadGate`,
the sentence moves to `ConsentSurface.GateHelp`, and the view model puts the two
together. That is a field read replacing a literal, not a view-model rewrite.

- [ ] **Step 1: Write the failing tests**

Create `windows/tests/TraceCommons.Interop.Tests/ConsentCopyTests.cs`:

```csharp
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using TraceCommons.Interop;
using Xunit;

namespace TraceCommons.Interop.Tests;

/// <summary>
/// The consent surface's sentences, across the C ABI.
///
/// <para>
/// These replaced the three constants this shell used to hold in
/// <c>ReadGate.cs</c>, and the parity test in the GTK crate that opened this
/// file and grepped it for the exact text. Nothing here spells a sentence
/// out: the words are asserted against the payload, and what this shell is
/// held to is that it authors none of them.
/// </para>
/// </summary>
public sealed class ConsentCopyTests
{
    /// <summary>
    /// A payload with every field present decodes, and the sentences arrive
    /// intact.
    /// </summary>
    [Fact]
    public void TheContractShapeParses()
    {
        const string json = """
            {
              "gate_statement": "The statement.",
              "ready_help": "The armed tooltip.",
              "not_pinned_help": "The disarmed tooltip."
            }
            """;

        ConsentCopy copy = Assert.IsType<ConsentCopy>(ConsentSurface.Parse(json));
        Assert.Equal("The statement.", copy.GateStatement);
        Assert.Equal("The armed tooltip.", copy.ReadyHelp);
        Assert.Equal("The disarmed tooltip.", copy.NotPinnedHelp);
    }

    /// <summary>
    /// A field the Rust stopped exporting refuses the WHOLE payload.
    ///
    /// Null, never a partly-filled record: a missing sentence above
    /// Contribute is a missing claim, and rendering a blank where a safety
    /// claim goes is worse than rendering nothing.
    /// </summary>
    [Theory]
    [InlineData("""{"ready_help":"a","not_pinned_help":"b"}""")]
    [InlineData("""{"gate_statement":"","ready_help":"a","not_pinned_help":"b"}""")]
    [InlineData("""{"gate_statement":"a","ready_help":"","not_pinned_help":"b"}""")]
    [InlineData("""{"gate_statement":"a","ready_help":"b","not_pinned_help":""}""")]
    [InlineData("not json at all")]
    [InlineData("")]
    [InlineData(null)]
    public void AnIncompletePayloadIsRefusedWhole(string? json)
    {
        Assert.Null(ConsentSurface.Parse(json));
    }

    /// <summary>
    /// The live payload's field set is exactly what this shell decodes.
    ///
    /// <para>
    /// The round-trip test below proves no required field is missing. It
    /// cannot prove the reverse -- a field ADDED in Rust that this shell
    /// silently ignores would be a sentence the other two shells show and
    /// this one does not. This compares the exported inventory against the
    /// declared consumed set, so adding a field in Rust fails here until
    /// somebody decides what this shell does with it.
    /// </para>
    /// </summary>
    [Fact]
    public void TheExportedFieldsAreExactlyTheOnesThisShellConsumes()
    {
        string json = NativeMethods.TakeOwnedString(NativeMethods.tc_consent_copy())
            ?? throw new InvalidOperationException("tc_consent_copy returned NULL");
        using JsonDocument document = JsonDocument.Parse(json);
        var exported = document.RootElement.EnumerateObject()
            .Select(property => property.Name)
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToList();

        Assert.Equal(
            ConsentCopy.ConsumedFields.OrderBy(name => name, StringComparer.Ordinal).ToList(),
            exported);
    }

    /// <summary>The real cdylib hands over a payload this shell can use.</summary>
    [Fact]
    public void TheLivePayloadDecodes()
    {
        ConsentCopy copy = Assert.IsType<ConsentCopy>(ConsentSurface.Copy());
        Assert.All(copy.Sentences, sentence => Assert.False(string.IsNullOrEmpty(sentence)));
    }

    /// <summary>
    /// The branch crosses. This shell asks which sentence, it does not
    /// choose.
    /// </summary>
    [Fact]
    public void TheHelpSentenceComesFromTheAbiForBothAnswers()
    {
        ConsentCopy copy = Assert.IsType<ConsentCopy>(ConsentSurface.Copy());
        Assert.Equal(copy.ReadyHelp, ConsentSurface.GateHelp(true));
        Assert.Equal(copy.NotPinnedHelp, ConsentSurface.GateHelp(false));
    }

    /// <summary>
    /// No wording is authored in the consent surface's own sources.
    ///
    /// <para>
    /// The strict rule, the same one <c>RoutingTools.cs</c> is held to:
    /// every string literal in these three files must be a wire value.
    /// Asserted about the source rather than about behaviour because a
    /// hand-written sentence that happened to match the Rust would pass
    /// every behavioural test and then survive a rename in exactly one of
    /// the three shells.
    /// </para>
    /// </summary>
    [Theory]
    [InlineData("ConsentCopy.cs.txt")]
    [InlineData("ConsentSurface.cs.txt")]
    [InlineData("ReadGate.cs.txt")]
    public void NoWordingIsAuthoredInTheConsentSurface(string copied)
    {
        string path = Path.Combine(AppContext.BaseDirectory, copied);
        Assert.True(File.Exists(path), $"the implementation source was not copied to {path}");

        // Strip doc comments and line comments: prose about the claim quotes
        // the claim, and nothing in a comment is ever rendered.
        string uncommented = string.Join(
            "\n",
            File.ReadAllText(path)
                .Split('\n')
                .Where(line => !line.TrimStart().StartsWith("//", StringComparison.Ordinal)));

        var allowed = new HashSet<string>(StringComparer.Ordinal)
        {
            // The payload's wire keys, and nothing else.
            "gate_statement", "ready_help", "not_pinned_help",
        };

        foreach (Match match in Regex.Matches(uncommented, "\"([^\"\\\\]|\\\\.)*\""))
        {
            string literal = match.Value[1..^1];
            Assert.True(
                allowed.Contains(literal),
                $"\"{literal}\" is a string literal in {copied} that is not a wire value. "
                + "Wording on this surface comes from consent_copy.rs across the ABI.");
        }
    }
}
```

In `windows/tests/TraceCommons.Interop.Tests/PreviewTests.cs`:

- delete the whole `ReadGateCopyTests` class (its two tests move to Rust, where
  `the_consent_statement_is_exactly_what_was_agreed` now lives);
- change line 46 from `Assert.Equal(ReadGate.ReadyHelp, gate.Help);` to
  `Assert.Equal(ConsentSurface.GateHelp(true), ConsentSurface.GateHelp(gate.CanContribute));`
  and line 58 likewise with `false`. Those two assertions are now about the
  rule, which is what `ReadGate` still owns.

- [ ] **Step 2: Run and watch them fail**

```bash
cd windows && dotnet test tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj \
  --filter ConsentCopyTests
```

Expected: compile errors — `ConsentCopy`, `ConsentSurface` and the two
`NativeMethods` entry points do not exist.

- [ ] **Step 3: Write the record and the surface**

`windows/src/TraceCommons.Interop/ConsentCopy.cs`:

```csharp
using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace TraceCommons.Interop;

/// <summary>
/// The consent surface's fixed sentences, read from the Rust rather than
/// kept here.
///
/// <para>
/// Every property is filled from the payload and none has a default worth
/// rendering: a sentence this shell invented would be a sentence the Linux
/// and macOS shells do not print, and <see cref="GateStatement"/> is the
/// claim a contributor reads immediately above an irreversible button, so
/// inventing one is inventing a claim.
/// </para>
/// </summary>
public sealed record ConsentCopy
{
    /// <summary>
    /// The claim that replaced the acknowledgement checkbox. Both halves of
    /// what the tick used to assert: scrubbing is pattern-based and may have
    /// missed something, and nothing here can tell whether anyone looked.
    /// </summary>
    [JsonPropertyName("gate_statement")] public string GateStatement { get; init; } = "";

    /// <summary>The tooltip on an armed Contribute.</summary>
    [JsonPropertyName("ready_help")] public string ReadyHelp { get; init; } = "";

    /// <summary>
    /// The tooltip on a Contribute with nothing to bind to. Never chosen
    /// here: <see cref="ConsentSurface.GateHelp"/> asks the ABI which of the
    /// two applies, because a branch kept in three shells drifts the same
    /// way words do.
    /// </summary>
    [JsonPropertyName("not_pinned_help")] public string NotPinnedHelp { get; init; } = "";

    /// <summary>Every sentence, for the refuse-on-any-empty-field check.</summary>
    public string[] Sentences => new[] { GateStatement, ReadyHelp, NotPinnedHelp };

    /// <summary>
    /// The payload fields this shell decodes, by wire name.
    ///
    /// <para>
    /// Compared against the live export by
    /// <c>TheExportedFieldsAreExactlyTheOnesThisShellConsumes</c>. A field
    /// added in Rust and not added here is a sentence the other two shells
    /// show and this one does not, and no round-trip test can see that.
    /// </para>
    /// </summary>
    public static IReadOnlyList<string> ConsumedFields { get; } =
        new[] { "gate_statement", "ready_help", "not_pinned_help" };
}
```

`windows/src/TraceCommons.Interop/ConsentSurface.cs`:

```csharp
using System;
using System.Text.Json;

namespace TraceCommons.Interop;

/// <summary>
/// The consent surface's wording, across the C ABI.
///
/// <para>
/// Nothing in this file is a word. The sentences cross as JSON, already
/// assembled, and the choice between the two tooltips crosses as its own
/// call, so this shell fills in no template and takes no branch of its own.
/// </para>
/// </summary>
public static class ConsentSurface
{
    /// <summary>
    /// Every fixed sentence on the surface, or null when the call failed or
    /// the payload will not parse.
    ///
    /// Null, never a partly-filled record: a blank where a safety claim goes
    /// is worse than nothing, and a C#-authored claim is worse than both.
    /// The caller decides what to show when the words are not available.
    /// </summary>
    public static ConsentCopy? Copy() =>
        Parse(NativeMethods.TakeOwnedString(NativeMethods.tc_consent_copy()));

    /// <summary>
    /// The payload half of <see cref="Copy"/>, split out so it is testable
    /// without the cdylib. The native call is a one-liner; this is where the
    /// behaviour that can actually be wrong lives.
    /// </summary>
    internal static ConsentCopy? Parse(string? json)
    {
        if (string.IsNullOrWhiteSpace(json))
        {
            return null;
        }

        try
        {
            ConsentCopy? copy = JsonSerializer.Deserialize<ConsentCopy>(json);
            if (copy is null)
            {
                return null;
            }

            foreach (string sentence in copy.Sentences)
            {
                if (string.IsNullOrEmpty(sentence))
                {
                    return null;
                }
            }

            return copy;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>
    /// The tooltip that explains the current answer, chosen by the ABI.
    ///
    /// Null only on a caught panic. Do not recover this by picking between
    /// <see cref="ConsentCopy.ReadyHelp"/> and
    /// <see cref="ConsentCopy.NotPinnedHelp"/> here: the branch crosses so
    /// that three shells cannot each keep their own copy of it.
    /// </summary>
    public static string? GateHelp(bool pinned) =>
        NativeMethods.TakeOwnedString(NativeMethods.tc_consent_gate_help(pinned ? 1 : 0));
}
```

In `windows/src/TraceCommons.Interop/NativeMethods.cs`, beside the other copy
imports:

```csharp
    /// <summary>
    /// Every fixed sentence on the consent surface, as an owned JSON object;
    /// free it with <see cref="tc_string_free"/>, which
    /// <see cref="TakeOwnedString"/> does. NULL only on a caught panic.
    /// </summary>
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr tc_consent_copy();

    /// <summary>
    /// Which of the two Contribute tooltips applies, chosen on the Rust
    /// side. 1 is pinned; 0 and anything else are not.
    /// </summary>
    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr tc_consent_gate_help(int pinned);
```

- [ ] **Step 4: Strip `ReadGate.cs` to the rule**

Delete `Statement`, `ReadyHelp`, `UnenrolledHelp` and the `Help` property.
Keep `Changed`, `HasPinnedPreview`, `CanContribute`, `SetPinnedPreview` and
`Reset` exactly as they are. Replace the class doc's last paragraph — the one
that says the macOS shell holds the same sentence and a Rust test greps this
file — with:

```csharp
/// <para>
/// The sentences moved. They are composed once in
/// <c>crates/trace-commons-contributor/src/consent_copy.rs</c> and read here
/// through <see cref="ConsentSurface"/>; what is left in this class is the
/// rule, which is testable on a machine that cannot build WinUI and cannot
/// load the cdylib. The Rust test that used to open this file and grep it
/// for the claim is gone with them: three shells reading one constant is a
/// stronger thing than three shells grepping each other.
/// </para>
```

- [ ] **Step 5: Wire the view model**

In `windows/src/TraceCommons.App/ViewModels/PreviewSheetViewModel.cs`:

```csharp
    /// <summary>
    /// The consent surface's sentences, read once. Null if the payload did
    /// not arrive or would not parse, in which case the sheet shows no
    /// claim rather than a blank one -- see ConsentSurface.Parse.
    /// </summary>
    private readonly ConsentCopy? _consent = ConsentSurface.Copy();
```

then line 451 `public string ContributeHelp => Gate.Help;` becomes

```csharp
    public string ContributeHelp => ConsentSurface.GateHelp(Gate.CanContribute) ?? string.Empty;
```

and line 603 `public string GateStatement => ReadGate.Statement;` becomes

```csharp
    public string GateStatement => _consent?.GateStatement ?? string.Empty;
```

`PreviewSheet.xaml` is unchanged — it binds `ViewModel.GateStatement`, which
still exists. Its comment on line 27 naming `TraceCommons.Interop.ReadGate`
stays accurate: the rule is still there.

- [ ] **Step 6: Copy the three sources for the strict guard**

In `windows/tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj`,
beside the existing routing and witness copy blocks:

```xml
  <!--
    The consent surface's implementation sources, copied next to the test
    assembly and read by NoWordingIsAuthoredInTheConsentSurface.

    All three, not just the two new ones: ReadGate.cs is where the four
    sentences used to live, and the strict rule is the thing that stops one
    of them coming back beside the rule that survived.
  -->
  <ItemGroup>
    <None Include="$(MSBuildThisFileDirectory)../../src/TraceCommons.Interop/ConsentCopy.cs"
          Link="ConsentCopy.cs.txt"
          CopyToOutputDirectory="PreserveNewest" />
    <None Include="$(MSBuildThisFileDirectory)../../src/TraceCommons.Interop/ConsentSurface.cs"
          Link="ConsentSurface.cs.txt"
          CopyToOutputDirectory="PreserveNewest" />
    <None Include="$(MSBuildThisFileDirectory)../../src/TraceCommons.Interop/ReadGate.cs"
          Link="ReadGate.cs.txt"
          CopyToOutputDirectory="PreserveNewest" />
  </ItemGroup>
```

- [ ] **Step 7: Ratchet the Windows baseline**

In `windows/tests/TraceCommons.Interop.Tests/ShellWordingTests.cs`:

- delete the `{ "TraceCommons.Interop/ReadGate.cs", 4 },` entry and the comment
  block above it (the one calling those four sentences the highest priority on
  the list — they are done);
- add three names to `RustOwnedSurfaces`, in the array's existing order:
  `"TraceCommons.Interop/ConsentCopy.cs"`, `"TraceCommons.Interop/ConsentSurface.cs"`,
  `"TraceCommons.Interop/ReadGate.cs"`;
- extend that field's doc to say the consent trio is held to the strict rule by
  `NoWordingIsAuthoredInTheConsentSurface`.

Deleting the entry and adding the name are not redundant, and the spec says why:
`TheRustOwnedSurfacesAreNotGivenAWordingAllowance` additionally asserts the file
*was scanned*, which is what catches the guard silently passing over a file the
csproj glob stopped copying, and it makes re-adding a baseline entry a test
failure rather than a quiet allowance.

- [ ] **Step 8: Narrow the GTK parity test to the one shell still transcribing**

In `crates/trace-commons-contributor-gtk/src/copy.rs`, in
`the_three_shells_print_the_same_statement`, remove the Windows path from the
loop and rename nothing yet:

```rust
        for relative in [
            // The Windows shell reads this sentence from `consent_copy.rs`
            // across the ABI now, so there is nothing in its source to grep.
            // macOS is the last transcription standing; this test goes with
            // it in the next commit.
            "../../macos/Sources/TCShellCore/ReadGate.swift",
        ] {
```

Leaving the Windows path in would fail the moment its literal is gone, and
deleting the whole test here would leave the remaining transcription unguarded
for a commit. Neither is acceptable; narrowing is.

- [ ] **Step 9: Green**

```bash
cargo build -p trace-commons-contributor-ffi
cd windows && dotnet test tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml
```

The WinUI project itself only builds on Windows with MSBuild from Visual
Studio; if this task is done off Windows, say so in the PR and let the
`windows-app` CI job be the check on `PreviewSheetViewModel.cs`.

- [ ] **Step 10: Commit**

```
Read the consent surface's sentences from Rust on Windows
```

---

### Task 7 (ReadGate, macOS): decode it, and align the condition

**Files:**
- Create: `macos/Sources/TCShellCore/ConsentCopy.swift`
- Create: `macos/Sources/TCBridge/TCConsentCopy.swift`
- Modify: `macos/Sources/TCShellCore/ReadGate.swift` (the three sentences out)
- Modify: `macos/Sources/TraceCommonsApp/Views/PreviewSheet.swift`
- Create: `macos/Tests/TCShellCoreTests/ConsentCopyTests.swift`
- Modify: `macos/Tests/TCShellCoreTests/ReadGateTests.swift`
- Create: `macos/Tests/TCBridgeTests/ConsentCopyBridgeTests.swift`
- Modify: `macos/Tests/TCShellCoreTests/ShellWordingTests.swift` (ratchet)
- Modify: `crates/trace-commons-contributor-gtk/src/copy.rs` (delete the parity test)

**Interfaces:**
- Produces:
  ```swift
  // TCShellCore
  public struct ConsentCopy: Decodable, Equatable, Sendable {
      public let gateStatement: String
      public let readyHelp: String
      public let notPinnedHelp: String
      public static func decode(fromJSON json: String) -> ConsentCopy?
      public static let consumedFields: [String]
      public var sentences: [String] { get }
  }
  public enum ReadGate {
      public static func canContribute(hasPinnedPreview: Bool) -> Bool
  }

  // TCBridge
  public enum TCConsentCopy {
      public static func copyJSON() -> String?
      public static func gateHelp(pinned: Bool) -> String?
  }
  ```
- Consumes: `tc_consent_copy` and `tc_consent_gate_help` from Task 4.
- `TCBridge` returns JSON strings and decodes nothing, exactly as
  `TCRoutingCopy` does, so no `Package.swift` dependency changes. `TCBridge`
  gains no dependency on `TCShellCore`.

**The behaviour change this task carries, and why it is here.** macOS arms
`Contribute` on `summary != nil`; Windows arms it on
`PreviewSummary.Enrolled`. One sentence cannot be true of two conditions, so
settling the wording (Task 6) requires settling the condition. `PreviewSummary`
already carries `enrolled` on macOS (`Models.swift:598`), so this is one line:
`ReadGate.canContribute(hasPinnedPreview: summary?.enrolled == true)`. It makes
macOS refuse to arm `Contribute` on an unenrolled preview, which is what the
other two shells already do and what the sentence says. **Flag it in the PR as a
behaviour change, not as a copy move.**

- [ ] **Step 1: Write the failing tests**

Create `macos/Tests/TCShellCoreTests/ConsentCopyTests.swift`:

```swift
import XCTest

@testable import TCShellCore

/// Decoding the consent surface's sentences, without the dylib.
///
/// Nothing here spells a sentence out. The words are asserted against the
/// payload, and what this shell is held to is that it authors none of them:
/// `ShellWordingTests` pins that, and `ConsentCopyBridgeTests` checks the
/// same properties against the real export.
final class ConsentCopyTests: XCTestCase {
    func testTheContractShapeParses() {
        let json = """
            {
              "gate_statement": "The statement.",
              "ready_help": "The armed tooltip.",
              "not_pinned_help": "The disarmed tooltip."
            }
            """
        guard let copy = ConsentCopy.decode(fromJSON: json) else {
            XCTFail("the contract shape must decode")
            return
        }
        XCTAssertEqual(copy.gateStatement, "The statement.")
        XCTAssertEqual(copy.readyHelp, "The armed tooltip.")
        XCTAssertEqual(copy.notPinnedHelp, "The disarmed tooltip.")
    }

    /// A field the Rust stopped exporting refuses the WHOLE payload.
    ///
    /// Nil, never a partly-filled value: a blank where a safety claim goes
    /// is worse than nothing, and a Swift-authored claim is worse than both.
    func testAnIncompletePayloadIsRefusedWhole() {
        for json in [
            #"{"ready_help":"a","not_pinned_help":"b"}"#,
            #"{"gate_statement":"","ready_help":"a","not_pinned_help":"b"}"#,
            #"{"gate_statement":"a","ready_help":"","not_pinned_help":"b"}"#,
            #"{"gate_statement":"a","ready_help":"b","not_pinned_help":""}"#,
            "not json at all",
            "",
        ] {
            XCTAssertNil(ConsentCopy.decode(fromJSON: json), "\(json) must be refused")
        }
    }

    /// The declared inventory is the shape the decoder actually reads.
    func testTheConsumedFieldSetMatchesTheDecodedShape() {
        XCTAssertEqual(
            ConsentCopy.consumedFields.sorted(),
            ["gate_statement", "not_pinned_help", "ready_help"])
    }
}
```

Create `macos/Tests/TCBridgeTests/ConsentCopyBridgeTests.swift`:

```swift
import TCBridge
import TCShellCore
import XCTest

/// The consent bundle through the real dylib.
final class ConsentCopyBridgeTests: XCTestCase {
    func testTheLivePayloadDecodes() {
        guard let json = TCConsentCopy.copyJSON(),
            let copy = ConsentCopy.decode(fromJSON: json)
        else {
            XCTFail("the live payload must decode")
            return
        }
        for sentence in copy.sentences {
            XCTAssertFalse(sentence.isEmpty)
        }
    }

    /// The exported field set is exactly what this shell decodes.
    ///
    /// The decode above proves no required field is missing. It cannot prove
    /// the reverse -- a field ADDED in Rust that this shell silently ignores
    /// would be a sentence the other two shells show and this one does not.
    func testTheExportedFieldsAreExactlyTheOnesThisShellConsumes() {
        guard let json = TCConsentCopy.copyJSON(),
            let data = json.data(using: .utf8),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            XCTFail("the live payload must be a JSON object")
            return
        }
        XCTAssertEqual(object.keys.sorted(), ConsentCopy.consumedFields.sorted())
    }

    /// The branch crosses. This shell asks which sentence, it does not
    /// choose.
    func testTheHelpSentenceComesFromTheAbiForBothAnswers() {
        guard let json = TCConsentCopy.copyJSON(),
            let copy = ConsentCopy.decode(fromJSON: json)
        else {
            XCTFail("the live payload must decode")
            return
        }
        XCTAssertEqual(TCConsentCopy.gateHelp(pinned: true), copy.readyHelp)
        XCTAssertEqual(TCConsentCopy.gateHelp(pinned: false), copy.notPinnedHelp)
    }
}
```

In `macos/Tests/TCShellCoreTests/ReadGateTests.swift`: delete `statement`,
`testTheConsentStatementIsExactlyWhatWasAgreed` and
`testTheStatementKeepsBothHalvesOfWhatTheCheckboxUsedToSay` (all three now live
in `consent_copy.rs`), and rewrite the two remaining tests against the rule
alone:

```swift
    func testAPreviewThatHasNotLoadedCannotBeContributed() {
        XCTAssertFalse(ReadGate.canContribute(hasPinnedPreview: false))
    }

    func testALoadedPreviewArmsContributeWithNothingElseRequired() {
        // The change this test exists to pin down: no transcript tab, no
        // acknowledgement, no second step. Contribute is live as soon as
        // there is something to contribute. The tooltip that explains either
        // answer is `TCConsentCopy.gateHelp`, chosen in Rust.
        XCTAssertTrue(ReadGate.canContribute(hasPinnedPreview: true))
    }
```

- [ ] **Step 2: Run and watch them fail**

```bash
cargo build -p trace-commons-contributor-ffi
cd macos && swift test --filter Consent
```

Expected: `cannot find 'ConsentCopy' in scope` and `cannot find 'TCConsentCopy'
in scope`.

- [ ] **Step 3: Write the decode and the bridge**

`macos/Sources/TCShellCore/ConsentCopy.swift`:

```swift
import Foundation

/// The consent surface's fixed sentences, decoded from what the Rust
/// exported.
///
/// Every property here is filled from the payload and none is written in
/// Swift: a sentence this shell invented would be one the Linux and Windows
/// shells do not print, and `gateStatement` is the claim a contributor reads
/// immediately above an irreversible button, so inventing one is inventing a
/// claim.
///
/// Decoding is here rather than in `TCBridge` so it can be tested without
/// linking the dylib; `TCBridgeTests` checks the same properties against the
/// real export.
public struct ConsentCopy: Decodable, Equatable, Sendable {
    /// The claim that replaced the acknowledgement checkbox.
    public let gateStatement: String
    /// The tooltip on an armed `Contribute`.
    public let readyHelp: String
    /// The tooltip on a `Contribute` with nothing to bind to. Never chosen
    /// here: `TCConsentCopy.gateHelp(pinned:)` asks the ABI which of the two
    /// applies, because a branch kept in three shells drifts the same way
    /// words do.
    public let notPinnedHelp: String

    enum CodingKeys: String, CodingKey {
        case gateStatement = "gate_statement"
        case readyHelp = "ready_help"
        case notPinnedHelp = "not_pinned_help"
    }

    /// The payload fields this shell decodes, by wire name. Compared against
    /// the live export by `TCBridgeTests`.
    public static let consumedFields = ["gate_statement", "ready_help", "not_pinned_help"]

    /// Every sentence, for the refuse-on-any-empty-field check.
    public var sentences: [String] { [gateStatement, readyHelp, notPinnedHelp] }

    /// Decode the payload, or nil if it will not parse or a field is empty.
    ///
    /// Nil, never a partly-filled value: a screen that renders "" where a
    /// safety claim goes is worse than one that renders nothing, and one
    /// that renders a Swift-authored claim is worse than both.
    public static func decode(fromJSON json: String) -> ConsentCopy? {
        guard let data = json.data(using: .utf8),
            let copy = try? JSONDecoder().decode(ConsentCopy.self, from: data)
        else {
            return nil
        }
        return copy.sentences.contains(where: \.isEmpty) ? nil : copy
    }
}
```

`macos/Sources/TCBridge/TCConsentCopy.swift`:

```swift
import CTraceCommons
import Foundation

/// The consent surface's sentences, read from the Rust rather than written
/// here.
///
/// Handle-free for the same reason `TCRoutingCopy` is: it describes the
/// build, not a running daemon.
///
/// Nothing in this file is a word, and nothing in it is a branch. The
/// sentences cross as JSON and the choice between the two tooltips crosses
/// as its own call.
public enum TCConsentCopy {
    /// Every fixed sentence on the surface, as a JSON object, or nil if the
    /// ABI reported a caught panic. Decoded by `TCShellCore.ConsentCopy`.
    public static func copyJSON() -> String? {
        guard let raw = tc_consent_copy() else { return nil }
        defer { tc_string_free(raw) }
        return String(cString: raw)
    }

    /// The tooltip that explains the current answer, chosen by the ABI.
    ///
    /// Nil only on a caught panic. Do not recover this by picking between
    /// the two sentences from `copyJSON`: the branch crosses so that three
    /// shells cannot each keep their own copy of it.
    public static func gateHelp(pinned: Bool) -> String? {
        guard let raw = tc_consent_gate_help(pinned ? 1 : 0) else { return nil }
        defer { tc_string_free(raw) }
        return String(cString: raw)
    }
}
```

- [ ] **Step 4: Strip `ReadGate.swift` to the rule**

Delete `statement`, `readyHelp`, `notPinnedHelp` and the `help(hasPinnedPreview:)`
function. Keep `canContribute(hasPinnedPreview:)`. Replace the "Why it lives in
TCShellCore" paragraph's last two sentences (the ones naming the other shells'
files and the Rust grep test) with:

```swift
/// The sentences moved. They are composed once in
/// `crates/trace-commons-contributor/src/consent_copy.rs` and read here
/// through `TCBridge.TCConsentCopy`; what is left in this enum is the rule,
/// which is testable in a target that links no dylib. The Rust test that
/// used to open this file and grep it for the claim is gone with them.
```

- [ ] **Step 5: Wire the sheet, and align the condition**

In `macos/Sources/TraceCommonsApp/Views/PreviewSheet.swift`:

```swift
    /// The consent surface's sentences, read once. Nil if the payload did
    /// not arrive or would not parse, in which case the sheet shows no claim
    /// rather than a blank one -- see `ConsentCopy.decode`.
    private var consent: ConsentCopy? {
        TCConsentCopy.copyJSON().flatMap(ConsentCopy.decode(fromJSON:))
    }

    private var canContribute: Bool {
        // `enrolled`, not `summary != nil`. An approval binds to the
        // envelope a preview pinned, and a preview built without an
        // enrollment pinned nothing -- which is the condition the shared
        // sentence names and the one the other two shells already test.
        ReadGate.canContribute(hasPinnedPreview: summary?.enrolled == true)
    }

    private var gateHelp: String {
        TCConsentCopy.gateHelp(pinned: canContribute) ?? ""
    }
```

and line 644's `Text(ReadGate.statement)` becomes `Text(consent?.gateStatement ?? "")`.

- [ ] **Step 6: Ratchet the macOS baseline**

In `macos/Tests/TCShellCoreTests/ShellWordingTests.swift`:

- delete the `TCShellCore/ReadGate.swift` entry (it measured 3 in Task 1 and is
  now 0);
- add `"TCBridge/TCConsentCopy.swift"`, `"TCShellCore/ConsentCopy.swift"` and
  `"TCShellCore/ReadGate.swift"` to `rustOwnedSurfaces`, keeping it sorted;
- check the `TraceCommonsApp/Views/PreviewSheet.swift` entry: `?? ""` is not a
  sentence, so the number should be unchanged. If it moved, the measurement
  says so and the entry follows it.

- [ ] **Step 7: Delete the parity scaffold — one of the two**

In `crates/trace-commons-contributor-gtk/src/copy.rs`, delete
`the_three_shells_print_the_same_statement` entirely, along with the local
`STATEMENT` constant and `the_consent_statement_is_exactly_what_was_agreed` if
it is not used by anything else in that module. All three assertions now live in
`consent_copy.rs`, where they are asserted once against the definition instead
of three times against three transcriptions.

**Keep `the_correction_disclosure_is_intact_in_all_three_shells`.** The spec
says slice 1 deletes two file-grepping parity tests; that is right about the
statement test and premature about the correction one. `CorrectionCopy` does not
migrate until slice 2, so deleting its guard here would leave a load-bearing
disclosure — the only place a contributor is told a correction is stored
verbatim — with nothing holding the three shells to it. Leave it, and add one
line to its doc:

```rust
    /// TODO(shell-copy slice 2): this goes when `CorrectionCopy` moves into
    /// `correction_copy.rs`. It is the scaffold the migration spec wants
    /// gone, and it stays exactly as long as the transcriptions it guards do.
```

- [ ] **Step 8: Green, everywhere**

```bash
cargo build -p trace-commons-contributor-ffi
cd macos && swift test
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml
cd windows && dotnet test tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj
cargo test -p trace-commons-contributor-ffi
cargo test -p trace-commons-contributor --lib consent_copy
```

- [ ] **Step 9: Commit**

```
Read the consent surface's sentences from Rust on macOS
```

---

## Verification matrix for the PR

Run all of these before claiming green, and paste the output. A passing run in
the wrong worktree proves nothing, so state which worktree they ran in.

```bash
# Core and ABI, with warnings as errors
RUSTFLAGS='-D warnings' cargo check -p trace-commons-contributor -p trace-commons-contributor-ffi --bins
RUSTFLAGS='-D warnings' cargo test -p trace-commons-contributor -p trace-commons-contributor-ffi --no-run
cargo test -p trace-commons-contributor --lib consent_copy
cargo test -p trace-commons-contributor-ffi --test abi
cargo test -p trace-commons-contributor-ffi --test abi_header_surface

# The permissive-standalone configuration nothing else exercises
cargo check -p trace-commons-contributor --no-default-features
cargo check -p trace-commons-contributor-ffi --no-default-features

# The separate GTK workspace
cargo test --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml
cargo build --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml --bin trace-commons-shell
cargo fmt --manifest-path crates/trace-commons-contributor-gtk/Cargo.toml -- --check

# Clippy, with the allow-list unchanged
cargo clippy -p trace-commons-contributor -p trace-commons-contributor-ffi --all-targets -- \
  -A clippy::type_complexity -A clippy::collapsible_if \
  -A clippy::manual_option_as_slice -A clippy::useless_vec \
  -A clippy::redundant_pattern_matching

# The two shells with their own suites
cargo build -p trace-commons-contributor-ffi
(cd macos && swift test)
(cd windows && dotnet test tests/TraceCommons.Interop.Tests/TraceCommons.Interop.Tests.csproj)

# Formatting, last, because the repo's post-edit hook rewrites whole files
cargo fmt --all
git show --stat HEAD
```

Two things a green run does not cover, and the PR should say so:

- **The WinUI project** (`TraceCommons.App`) builds only on Windows with MSBuild
  from Visual Studio. `PreviewSheetViewModel.cs` is verified by the `windows-app`
  CI job, not locally.
- **The macOS app target's SwiftUI view.** `swift test` compiles
  `TraceCommonsApp`, so a type error in `PreviewSheet.swift` fails; whether the
  new binding reaches the screen is not testable and is not claimed.

## Where the spec left a choice, and what this plan chose

Six places. Each one is a decision this plan made because the work could not
start without it, not a reopening of something the spec settled.

1. **Two exports, not one.** The spec's carrier section says one bundle export
   per surface; its §2 also says a sentence chosen by a condition must not cross
   as a table for the shell to branch on — "the branch crosses too". The help
   tooltip is exactly such a sentence, so this plan ships `tc_consent_copy` *and*
   `tc_consent_gate_help`, the same `_line`-beside-the-bundle shape routing
   already has. There is no tone pair, because this surface paints nothing.
2. **Module name: `consent_copy.rs`.** The spec's slice table says so, and it
   stands, even though `consent.rs` already exists next to it. That file
   validates upload-claim consent scopes and holds no copy; the module doc says
   as much so nobody merges the two.
3. **Swift decode in `TCShellCore`, fetch in `TCBridge`.** The spec's table is
   explicit (`tc_*_copy()` → `TCBridge`, decode in `TCShellCore`, testable
   without the dylib) and `TCRoutingCopy` is the working precedent, so `TCBridge`
   returns JSON strings and gains no dependency.
4. **Only one of the two parity tests is deleted.** The spec says slice 1 deletes
   both. `the_correction_disclosure_is_intact_in_all_three_shells` guards copy
   that does not migrate until slice 2, and deleting it here would leave the one
   disclosure a contributor gets about verbatim-stored corrections with nothing
   holding the three shells to it. It stays, with a TODO naming its slice.
5. **What "authored in the shell" means for GTK.** `copy.rs` is the migration
   target, so its `pub const` sentences count. A `pub use` is not a literal, so a
   migration lowers the number by itself. Test modules are excluded by a
   brace-matched skip rather than by cutting the file at the first
   `#[cfg(test)]`, because `copy.rs` interleaves three of them with real copy.
6. **Settling the wording forced settling the condition.** Windows and macOS
   armed `Contribute` on different facts, so no single sentence was true of both.
   The Windows wording and the Windows condition both win, and macOS moves. This
   is the one behaviour change in the slice and the PR flags it as such.

## What the PR body has to say

1. That it is **slice 0 plus slice 1 only** — the spec's recommended first unit
   — and that slices 2 through 8 are separate plans.
2. The three authorship totals: Windows 390 (from #644), macOS and GTK measured
   here for the first time. If either is much larger than 390, the spec's open
   question 5 is answered and the later slicing needs revisiting.
3. **The divergence decision, prominently.** `UnenrolledHelp` won,
   `notPinnedHelp` lost, and the macOS `Contribute` condition moved from "a
   preview arrived" to "a preview arrived and carries an enrollment" so that one
   sentence is true of all three shells. That is a behaviour change on macOS and
   the one thing in this PR a maintainer should overrule if they disagree.
4. That `the_three_shells_print_the_same_statement` is deleted and
   `the_correction_disclosure_is_intact_in_all_three_shells` deliberately is not,
   with the reason.
5. Whether this is stacked on `windows-wording-guard-coverage` (#644).
