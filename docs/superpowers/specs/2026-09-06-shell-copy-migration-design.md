**DRAFT — a proposal pending maintainer review. Nothing here is an approved plan, and no slice below should be started until this document is accepted or amended.**

# Moving the shells' words into the core

**Status:** draft proposal, 2026-09-06. Author: agent dispatch. Not reviewed.

The rule is core-owns-the-words: a sentence a contributor reads is composed
once, in `crates/trace-commons-contributor`, and every shell reads it. Routing,
witness and compute already carry core-owned copy; other surfaces still
transcribe it. The inventory below distinguishes merged code from open PRs.

## What is actually true today

Historical baseline, measured at PR #644 (`c6f097e5`), not a count of the
remaining work after subsequent copy changes. PR #644's `ShellWordingTests.cs` counts the string
literals that read as a sentence across both Windows shell projects and pins
each file at the number it holds:

| Category | Files | Sentences |
|---|---|---|
| `TraceCommons.Interop/*Copy.cs` and kin (transcriptions) | 22 | 229 |
| `TraceCommons.Interop/ReadGate.cs` | 1 | 4 |
| `TraceCommons.App/ViewModels/*.cs` | 7 | 63 |
| Window and control code-behind | 3 | 10 |
| XAML views | 6 | 84 |
| **Total** | **39** | **390** |

The same sentences are hand-transcribed a second time into
`macos/Sources/TCShellCore/*.swift` and a third time into
`crates/trace-commons-contributor-gtk/src/copy.rs` (3,188 lines). Neither of
those two shells has a ratchet at all, so their counts are not known.

Integration checkpoint (2026-09-06, main `8c796948`):

- **Merged #610:** `compute/controller.rs::ComputeCopy` and
  `contributor-ffi/src/compute.rs::tc_compute_copy_json` already provide core
  copy. Reuse that contract; copy migration must preserve compute's separate
  controller lifetime, consent and disabled production defaults.
- **Open #611, `9a31a286`:** adds `source_copy::source_settings_copy` and
  `tc_source_settings_copy`, including registry-backed unset-source semantics
  and the warning that turning a source off retains previously queued sessions.
- **Open #612, `023dd742`:** adds `onboarding_copy` and `tc_onboarding_copy`.
  All three shells consume the generic configured-source and digest wording;
  it no longer names two or four sources or promises a fixed four-hour interval.
  The Windows four-hour check is an additional conservative backstop, not the
  daemon's configurable scheduling policy. These fixes supersede the earlier
  divergence examples; do not recreate their bundles in a later slice.

Refresh these PR states and the per-file baseline at the start of slice 0.
The following divergence still needs a reviewed resolution:

- **A safety claim with a divergent neighbour.** `ReadGate.Statement` is
  identical in all three shells only because a Rust test
  (`the_three_shells_print_the_same_statement`) opens the Swift and C# files
  and greps them. The sentence *beside* it is not covered: Windows says
  `UnenrolledHelp` — "This device isn't connected yet…" — and macOS says
  `notPinnedHelp` — "This preview hasn't loaded yet…". Two shells, two
  different explanations of why the same button is off.

## The shape that already works

`routing_copy.rs`, `witness_copy.rs` and the merged compute copy are
precedents. Their error and tone contracts differ and must remain distinct;
the historical 39-file inventory needs reconciliation with the open PRs above.

- The fixed strings of a surface are one `#[derive(Serialize)]` struct built
  by one function (`routing_copy()`, `witness_copy()`), nested into
  sub-blocks where a surface has parts (`witness_copy().wallet`,
  `.onboarding`, `.admission`, `.review`).
- One C ABI export per bundle returns it as JSON —
  `tc_routing_copy()`, `tc_witness_copy()` — declared in both copies of
  `trace_commons.h`, which `abi_header_surface.rs` holds byte-identical.
- A sentence chosen by a condition does not cross as a table for the shell to
  branch on. The **branch crosses too**: `tc_routing_tool_word(source_mode,
  wiring)` returns the chosen word and `tc_routing_tool_tone(...)` returns
  how to paint it, from the same inputs, so the two cannot fall out of step.
- Tone namespaces and unknown-value behavior are surface-specific.
  Routing uses `TC_ROUTING_TONE_*` and unknown values become Neutral.
  Witness uses the disjoint `TC_WITNESS_TONE_*` range (10–14), including
  `TC_WITNESS_TONE_REFUSED`; unknown witness values must render Refused.
  `trace_commons.h` documents this fail-closed boundary explicitly.
- Each shell holds a thin decode: `RoutingSurface.Parse` (C#) refuses the
  *whole* payload if any field is empty rather than rendering a blank;
  `TCRoutingCopy` (Swift) returns the JSON and decodes in `TCShellCore` so it
  is testable without the dylib. GTK does not use the ABI at all — its
  `copy.rs` is `pub use trace_commons_contributor::routing_copy::{…}`.
- Per-shell guards then assert no local authorship:
  `NoWordingIsAuthoredInThisShell` scans `RoutingTools.cs.txt` (a build-copied
  `.cs` with a `.txt` suffix so it can never be mistaken for a compilable
  source) and requires every literal to be a wire value.

## 1. Goal and non-goals

**Goal.** Every sentence a contributor reads is composed in
`crates/trace-commons-contributor` and read by all three shells, and a guard
in each shell fails if a shell authors one. `WordingBaseline` in
`ShellWordingTests.cs` reaches zero entries and stays there.

**Non-goals.**

- **Localization.** The bundles are English. No message catalogue, no
  `gettext`, no locale argument on any export. If localization is ever wanted,
  the bundle is exactly where it would attach; building for it now buys
  nothing and constrains every slice.
- **Rewording.** A migration commit moves a sentence byte-for-byte. Where the
  three shells disagree the PR must say so and pick one, and *that choice*
  is the reviewable content of the PR — but no sentence is improved in
  passing. A diff that both moves and edits 44 sentences is not reviewable.
- **Runtime copy.** No copy delivered by the server, no reload, no
  copy-without-a-rebuild. The dylib and the shell ship in one installer.
- **Moving GTK onto the C ABI.** It links Rust directly and should keep doing
  so. Section 5 is about making that a third *reader* rather than a third
  *author*.
- **Restructuring the shells.** No view-model rewrites beyond replacing the
  literal with a field read.
- **Design-system or styling work.** Tone values already exist; nothing here
  proposes new ones except where a migrated sentence needs one.

## 2. The copy carrier

**Proposal: one JSON bundle per surface, exactly as `routing_copy` does it.
No new mechanism.**

A surface is roughly a screen or a sheet — the granularity `WithdrawCopy`,
`HistoryCopy`, `HealthCopy` already have. Each becomes
`crates/trace-commons-contributor/src/<surface>_copy.rs` with a `Serialize`
struct, a builder function, and one export `tc_<surface>_copy()`.

Alternatives weighed:

- **One FFI function per sentence.** Rejected. 390 sentences is 390 exports,
  390 header declarations in two byte-identical copies, and 390 chances for a
  `NULL` return the shell has to have a policy for. The existing per-sentence
  exports (`tc_routing_token_line`, `tc_routing_unreachable_line`) exist only
  because those sentences take arguments — see §6. A fixed string has no
  argument and no reason to be its own export.
- **One bundle for the whole app.** Rejected. `RoutingSurface.Parse` refuses
  the entire payload when one field is empty, which is right at screen scale
  and catastrophic at app scale: a single dropped field blanks everything. It
  also makes every copy change touch one struct that every shell decodes in
  full at startup.
- **A key-lookup API, `tc_copy(key) -> string`.** Rejected, and worth saying
  why loudly: it would *pass* the wording guards, because a key is a wire
  value and reads as one. It converts a compile-time contract into a runtime
  string match where a renamed key fails as a blank label on a screen nobody
  opened during testing. The struct is the contract; keep it a struct.
- **A version integer on the bundle.** Rejected — and this is the one place
  this proposal declines something the brief invited. A version number
  implies a shell that can serve two versions, which no shell can or should:
  the cdylib and the shell ship together in one DMG, one MSIX, one Flatpak.
  What is actually needed is detection of a field that stopped being
  exported, and the existing refuse-on-any-empty-field parse already does
  that. If a bundle ever has to break shape, the export name carries it —
  `tc_health_copy_v2` — which the header diff makes impossible to miss.

Tones: preserve each surface's existing numbering and unknown-value policy.
A paired line/tone API must share the same inputs and branch semantics, but
must not reuse routing's Neutral fallback for a refusal or consent surface.
Witness already has a Refused value and requires unknown values to render
Refused. New tone namespaces or values require explicit ABI review; migrating
copy is not authorization to change an existing fail-closed contract.

## 3. Migration order

Recommended order, with one deviation from the obvious one.

1. **Slice 0 — ratchets on the other two shells.** Mirror #644 for macOS
   (a Swift test scanning `macos/Sources/**`) and GTK (a Rust test scanning
   the GTK crate). Without these, migrating a sentence out of Windows proves
   nothing about the other two, and there is no number to ratchet down. This
   also produces the first real count of Swift and GTK authorship, which this
   document cannot state today.
2. **ReadGate.** Four sentences, the highest-value four in the app: they are
   the safety claim shown at the instant of consent. Migrating them deletes
   `the_three_shells_print_the_same_statement` and
   `the_correction_disclosure_is_intact_in_all_three_shells` — tests that
   open other shells' source files and grep them. That scaffold should not
   grow; it is O(n) hand-written needles and it only ever covered the
   sentences somebody remembered to add. Resolving `UnenrolledHelp` vs
   `notPinnedHelp` is the reviewable decision in this slice.
3. **The `*Copy` transcription classes** (229 sentences, 22 files). These are
   already literal-only classes with no rendering in them, so each one is a
   near-mechanical move: struct in Rust, export, decode, delete. Ordered by
   claim weight, not by size — redaction, scrubbing, verdict and correction
   copy before history and profile copy.
4. **View-model-composed sentences** (63) and code-behind (10). Harder,
   because these compose rather than transcribe; §6 covers them.
5. **XAML** (84) last, because it is the only part needing new machinery: a
   page-level copy object exposed for `x:Bind`. **The deviation:** do one
   small XAML file early as a spike — `SessionRootsWindow.xaml`, 3 sentences
   — at the end of step 3, so the binding path is proven before six views
   depend on it. If the spike is ugly, that is worth knowing while there is
   still an easy alternative (set the text in code-behind from the bundle).

The alternative order considered was **per-surface vertical slices** —
migrate `WithdrawCopy`, `WithdrawViewModel` and the withdraw XAML together.
Rejected as the default because the layers hold *different* sentences, not
duplicated ones, so layering creates no window where a sentence lives in two
places; and because the mechanical `*Copy` moves can be batched by one person
in a way a vertical slice cannot. Where a surface's three layers are small
and obviously one screen, doing them together is fine — this is a default,
not a rule.

## 4. The ratchet and the end-state pin

`ShellWordingTests.WordingBaseline` is already a floor *and* a ceiling: adding
a sentence fails, and removing one fails too, so the entry has to be lowered
deliberately in the same commit. That is exactly the property this migration
needs, and it needs no change.

Per migrated file:

- Lower the entry to the number that remains. **Delete the entry at zero**,
  at which point the unlisted-file default of `allowed = 0` holds it.
- Then **add the file to `RustOwnedSurfaces`**. This is not redundant with
  deleting the entry: `TheRustOwnedSurfacesAreNotGivenAWordingAllowance`
  additionally asserts the file *was scanned*, which is what catches the
  guard silently passing over a file the csproj glob stopped copying, and it
  makes re-adding a baseline entry for that file a test failure rather than a
  quiet allowance.
- Migrate the corresponding Swift and GTK sites in the same PR and ratchet
  their slice-0 baselines the same way. A sentence removed from one shell and
  left in the other two is worse than where we started: the shells now
  disagree *and* the Rust claims ownership.

End state, in one commit at the finish: `WordingBaseline` becomes empty and a
new assertion requires it to stay empty, and the class doc changes from "a
ratchet, not a clean bill of health" to the strict claim. Do not delete the
whole test — the scan is the guard.

## 5. GTK, and three access paths

GTK links `trace-commons-contributor` directly and must keep doing so; the C
ABI would buy it nothing but marshalling. So there are three readers:

| Shell | Path | Decode |
|---|---|---|
| GTK | direct Rust, `pub use` in `copy.rs` | none |
| macOS | `tc_*_copy()` → `TCBridge` | `TCShellCore`, testable without the dylib |
| Windows | `tc_*_copy()` → `NativeMethods` | `TraceCommons.Interop`, testable without the cdylib |

The drift risk is not that the three read different words — they read one
struct. It is that GTK's `copy.rs`, which is *also* the file the other two
shells historically transcribed from, keeps its own literal beside the
re-export, and the GTK screen then renders the old constant while the ABI
serves the new one. That has already happened once in spirit: `routing_copy`
records that it "used to be a region of the GTK shell's `copy.rs`".

Proposal, matching the existing `TOOLS-SURFACE-BEGIN` / `-END` marker sweep in
`routing_copy.rs`:

- Migrated regions of `copy.rs` are delimited by
  `COPY-MIGRATED-BEGIN` / `COPY-MIGRATED-END` markers and may contain
  `pub use` and nothing else. A GTK-crate test reads its own source and fails
  on any string literal inside a marked region. This is the same
  read-your-own-source technique the routing sweep uses, and it is checked
  by the compiler for the re-export half already.
- Slice 0's GTK ratchet covers the rest of `copy.rs` and the `ui/` modules
  outside the markers.
- The two `trace_commons.h` copies stay byte-identical under
  `abi_header_surface.rs`; every new bundle export lands in both in one
  commit. This is already enforced and needs no change.
- Each ABI shell keeps a live round-trip test through the real dylib
  (`NativeRoundTripTests` on Windows is the model) asserting the bundle
  decodes with no empty required field. That detects missing Rust fields
  required by the decoder, not newly added fields the decoder silently ignores.
  For complete migration coverage, separately compare the exported recursive
  field inventory against each shell's declared consumed fields (with any
  intentionally unused fields explicitly documented). A test must fail when a
  new Rust field is added without updating that inventory. Keep this parity
  check in tests; do not silently change runtime compatibility policy.

## 6. Parameterized sentences

The rule is stated in `routing_copy.rs` and this proposal adopts it
unchanged: **sentences cross already assembled**. Nothing is handed to a
shell as a template with a hole in it, because three shells' format calls are
a fourth place the wording drifts — a dropped full stop or a reordered clause
around the hole, in three languages, with nothing to notice.

Three cases, all with existing precedent:

1. **The value is knowable in Rust.** Compose it there and let nothing cross
   inward. `RoutingCopy::folder_note` is a `String` rather than a `&'static
   str` for exactly this reason: it names the folder this machine would read,
   and all three shells read it from the bundle. **Digest cadence is runtime
   state, not a single constant:** `DaemonSettings.digest_interval_secs` is
   writable and `daemon/mod.rs` passes the current setting to `digest_due`.
   Any numeric sentence must use the confirmed runtime value, with a refresh
   path after settings changes. Alternatively retain #612's truthful generic
   maximum-frequency wording. Windows' additional four-hour suppression gate
   must not be presented as the shared daemon interval.
2. **The value is shell-side and locale-free** — a port, a count, a model
   name, a path the shell resolved. A per-sentence export taking typed
   arguments: `tc_routing_unreachable_line(port)`,
   `tc_routing_token_line(path)`. Counts inflect in Rust
   (`folder_summary(1, …)` → "1 session", `folder_summary(2, …)` → "2
   sessions"); a shell must never choose between two strings on `n == 1`,
   because that is a branch, and branches drift the same way words do.
3. **The value is shell-side and locale-dependent** — a humanised timestamp,
   a platform-formatted date. This is the *only* text that crosses inward:
   the shell passes the rendered fragment and Rust owns the sentence around
   it. `last_checked_line(when)` is the precedent, and its doc states the
   boundary: "an hour ago" is a rendering of a `DateTime`, not wording about
   routing, and every shell already has a localised one. Keep this list
   short and named; it is the one hole in the rule.

And where a sentence is *chosen* rather than filled, the choice crosses:
`tc_<surface>_<thing>_line(...)` plus `tc_<surface>_<thing>_tone(...)`, the
tone preserving the surface-specific unknown-value policy in §2.

## 7. Rough size and slicing

Sentence counts below are the historical #644 Windows baseline, not a new
measurement; reconcile #611/#612 before allocating overlapping work. Each
accepted existing bundle is reused rather than replaced. Each slice carries the
corresponding Swift and GTK sites, which are not yet counted (slice 0 fixes
that). Estimates are relative, not calendar.

| Slice | Content | Windows sentences | Notes |
|---|---|---|---|
| 0 | macOS + GTK wording ratchets | 0 | Prerequisite. No copy moves. |
| 1 | `ReadGate` → `consent_copy.rs` | 4 | Deletes two file-grepping parity tests. Resolves the pinned-vs-enrolled divergence. |
| 2 | Claim-bearing copy: `RedactionSummary` 7, `ScrubDetectorCopy` 6, `CorrectionCopy` 4, `VerdictCopy` 4, `RedactionLabels` 3, `ScrubbingCaveatCopy` 3, `UnresolvedBucketCopy` 3 | 30 | Small, high value, proves the bundle-per-surface shape at scale. |
| 3 | `PublicProfileCopy` 44, `WithdrawCopy` 34 | 78 | The two largest; `PublicProfileCopy` probably wants nested sub-blocks like `witness_copy`. |
| 4 | `HistoryCopy` 30, `HealthCopy` 22, `TrayModel` 11, `SubmitToast` 10, `WeekBandCopy` 1 | 74 | `TrayModel` and `SubmitToast` carry branch tables; expect `_line`/`_tone` pairs. |
| 5 | `SessionRootsCopy` 14, `UpdateProtocol` 10, `ProjectIgnoreCopy` 8, `ArmingOffer` 4, `OriginalSearchOutcome` 4, `WatchCopy` 4, `SubagentCopy` 2, `PreviewCardOutcome` 1 | 47 | Remainder of the interop classes. Ends with the XAML spike (`SessionRootsWindow.xaml`, 3). |
| 6 | View models 63 + code-behind 10 | 73 | Composition, not transcription. §6 applies throughout; expect new argument-taking exports. |
| 7 | XAML: `PreviewSheet` 20, `MainWindow` 19, `OnboardingWindow` 18, `SettingsView` 16, `HistoryView` 8, `SessionRootsWindow` 3 | 84 | Needs the binding decision from the spike. Reconcile the already-shared #612 onboarding text; do not migrate it twice. |
| 8 | Empty the baseline, pin it, rewrite the guard's claim | 0 | One commit. |

Slices 2 through 5 are independent of each other once slice 1 has settled the
bundle shape, and can run in parallel — one PR per slice, each lowering only
its own baseline entries. Slices 6 and 7 are ordered.

Roughly 9 to 11 new `*_copy.rs` modules, the same number of bundle exports,
and an expected 15 to 25 argument-taking or tone exports on top — every one
of them landing in both copies of `trace_commons.h` in the commit that adds
it.

## Open questions for review

1. **Bundle granularity.** This proposes one per existing `*Copy` class,
   which is roughly one per screen. Is a coarser grain (one per shell area:
   onboarding, queue, settings, history) preferable, given that fewer exports
   means fewer header edits?
2. **The version decision (§2).** Declining a version field is the one place
   this deviates from the brief. If bundles must survive a shell built
   against an older dylib — a scenario no current packaging produces — that
   changes.
3. **Divergence resolution.** Should each divergence found during migration
   be a separate decision PR *ahead of* the move, keeping migration PRs pure
   moves? That is cleaner to review and roughly doubles the PR count.
4. **XAML binding.** Page-level copy object with `x:Bind`, or set from
   code-behind at load? The spike in slice 5 is meant to answer this, but a
   maintainer preference would save the spike.
5. **Slice 0 scope.** The macOS and GTK authorship counts are unknown. If
   they are much larger than 390, the slicing above is wrong and slice 0
   should report before slices 2 onward are planned.
