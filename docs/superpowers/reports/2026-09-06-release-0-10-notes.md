# Release notes draft — 0.10.0

The body below is the text to publish for **both** `app-v0.10.0` and
`contributor-v0.10.0`. The 0.9.0 pair carried the same body, and the two
release workflows append their own install paragraphs (`release-apps.yml`
"Publish", `release-contributor.yml`) on top of whatever notes the tag is
created with, so this text is the part that has to be written by hand and set
with `gh release edit`.

Everything below the rule is the notes text. Nothing above it is.

**State of this draft.** It is written against `origin/main` at `0ed4f466`.
Three pull requests from the release's merge chain — #660, #662 and #666 —
were still open when it was drafted, and Task 3's live end-to-end run had not
produced a record in `docs/operator/verification-records/`. The places that
depend on those are marked `TO CONFIRM` and must be resolved or deleted
before publishing; do not publish a `TO CONFIRM` line.

---

## Contribute now waits for enrolment on macOS

On macOS, `Contribute` is now disabled until this device is enrolled.
Previously it armed as soon as a preview loaded. The disabled button explains
why:

> This device isn't connected yet, so this preview was built without your
> identity and nothing here can be contributed.

This brings macOS into line with Windows and GTK, which already required
enrolment. An approval binds to the envelope a preview pinned, and a preview
built without an enrollment pinned nothing — so the button that approves it
should not have been pressable.

The same change makes `Contribute` disarm, on macOS and on Windows both, when
the consent sentences cannot be decoded. The statement above the button is
the whole of what a contributor is told before pressing it; a build that
cannot render it must not offer a pressable button above a blank space where
the claim should be.

The sentences themselves now come from one place —
`crates/trace-commons-contributor/src/consent_copy.rs` — and cross to all
three shells already assembled, rather than being written out three times and
drifting.

## Answering model calls on this computer

The daemon can now run a local inference proxy: it answers the model calls
your tools make, from this machine, and keeps the record of them here. Each
call is still passed on to whoever you have set up to answer it. All three
shells offer it once on first start, and it stays in Settings either way.

It is **off by default**, and turning it on exposes something that has to be
said plainly. In the shipped wording:

> While it is on, anything else running on this computer can send calls
> through it as well, charged to the accounts you have set up here. On a
> computer only you use that is your own software; on a shared one it is
> anyone who can log in.

The longer form, for anyone deciding on a shared machine: the proxy's control
API requires a token, but the `/openai` and `/anthropic` inference paths do
not. Any process on the machine that can reach loopback can send calls
through it, authenticated and billed with whatever upstream credentials the
proxy home is configured with. On a single-user computer that is your own
software; on a shared or multi-user machine it is anyone with a local shell.
Turning it on also starts the proxy's own background work, including
catalogue discovery, which makes network calls the daemon did not previously
make.

The internal setting is named `private_inference`. That name is not a claim
and the shipped surface does not repeat it as one: turning this on does not
make your calls private. It moves where they are answered from and keeps the
record here.

### Quick start

Two settings keys, in the order they take effect:

```
private_inference          # the daemon runs a local inference proxy
ironwire_attested_bodies   # a call's bodies go to the witness
```

Both are set over the daemon's settings channel:

```bash
trace-commons-contributor daemon settings --set private_inference=true
```

There is no dedicated subcommand. For the witness pin, the receipt endpoint,
the refusal labels and the rollback, see
[`docs/operator/attested-inference.md`](docs/operator/attested-inference.md).
For the full key list and the `private_inference_state` labels, see
[`docs/contributor-daemon-ipc-v1_1.md`](docs/contributor-daemon-ipc-v1_1.md).

## The witness pins the gateway signing key

The witness can now be given the ed25519 key that NEAR AI's gateway signs
receipts with, via `TRACE_COMMONS_WITNESS_GATEWAY_KEY_PIN`. With a pin set, a
receipt signed by any other key is refused — at the witness, where the
decision is enforced and where a patched client cannot skip it. Verifying a
receipt's signature alone says only that it is self-consistent against the
key the receipt itself names, which any key satisfies, including one the
submitter holds.

The pin is independent of `--require-attested-inference`: a witness that
requires nothing still refuses a receipt from an unpinned key when one is
offered, because certifying it would be a silent downgrade. A malformed pin,
the empty string included, is a startup failure rather than a control that
silently matches nothing. Enabling it on a deployment is the procedure in
[`docs/operator/attested-inference.md`](docs/operator/attested-inference.md).

TO CONFIRM — one of: "the production witness now pins the gateway signing
key, so a receipt signed by any other key is refused where the decision is
enforced", or "the pin ships dormant, as in 0.9.0".

TO CONFIRM — the live end-to-end run: whether it ran, on what date, and what
it proved, including that the stored envelope carried no bodies, which is the
privacy claim and is not something a certificate establishes; or which leg
failed and that attested inference therefore remains unproven end to end.
Name the run record either way.

## The witness measurement is not printed here

The witness signing address and current measurement are published in
[`deploy/witness/README.md`](deploy/witness/README.md) under "The production
deployment". Read them from there, not from these notes: every witness
redeploy moves the measurement, and a release note is only a snapshot of what
was current when it was written. The `app-v0.9.0` notes carry an `mrconfigid`
that was already stale when they published, which is why this convention
exists.

The signing address is stable across an upgrade —
`0x655a17fcf6d0b9069e1b1dd07a7f5535d0c76798`, unchanged since 0.9.0 — and an
address that had changed would mean a new app rather than an upgraded one.

## Dependency pins

IronClaw is held at reviewed revision `6dccbfbc`; upstream has advanced
since, and re-advancing it is its own reviewed change rather than part of
this release. It is pinned in the two lockfiles, because upstream IronWire
still names `branch = main` for it, and both workspaces now build `--locked`
in CI and in the app release builds — a lockfile that is not committed in
step with a manifest is a failure rather than a silent re-resolve.

IronWire is pinned to revision `d1c21c1c`.

## Everything else in this release

**macOS.** Waiting decisions are badged on the menu-bar mark (#605). Welcome
comes before folder consent, and onboarding has a Back step (#607).
Permissions and watched folders can be changed from Settings (#611), and the
notification-permission prompt now comes after onboarding rather than during
it (#612). The queue health banner opens the waiting-folder list (#633).
Review controls are easier to open, search and resize (#635). Odd-sized type
scales correctly (#636). Development notes are out of the credit record
(#638), and stale action messages can be dismissed (#639).

**Windows.** The window closing to the tray no longer stops watching (#621).
Invite launches are redirected to the running app (#624), and a packaged
cold-start invite activation is read correctly (#618). Recovery actions route
to connection and waiting sessions (#617), review can be restored for armed
projects (#613), Ignore can be undone during onboarding (#649), and daemon
error codes stay out of the status label (#629). "Look inside" is the
accented button on a review card and both Submit buttons are secondary, so
the screen no longer recommends sending without reading (#614).

**Linux/GTK.** Label and routing presentation helpers are shared rather than
duplicated (#637).

**CLI.** Destructive actions are confirmed, and an offline daemon is
identified as offline rather than reported as an error (#608).

**Wording.** The consent gate's sentences moved into the core and are shared
by all three shells (#668). The wording scanners now count what each shell actually
authors — Windows (#644), macOS (#669), GTK (#670) — and the
onboarding-window baseline was lowered to the scanner's real count (#659).

**Local inference proxy.** IronWire runs inside the daemon behind an
off-by-default switch (#652). The offer is made once on first start in all
three shells (#656). Settings are persisted before the proxy is activated
(#657). The daemon retains ownership of the proxy through cancellation and
shuts it down within a bound (#665), and discovery health probes are confined
(#667). Upstream pins moved as described above (#658).

**Attested inference and the witness.** The receipt key that is actually
attested is the one verified (#603), the witness image that reads the receipt
scheme is pinned (#604), and the witness pins the gateway signing key (#650).
The stale-container window is recorded in the deploy README (#647), and the
path from dormant to enforced is documented (#642).

**Server and infrastructure.** The PostgreSQL-backed test suite was repaired
(#655), and a migration and its recording are now committed together (#651).
Worker IPC is verified against Orchard-generated signatures (#653). Native
onboarding connects to verified NEAR admission (#602), and HoloNear compute
lifecycle and package verification are integrated (#610).

**Build and CI.** Committed dependency locks are required in root and GTK CI
(#661) and in the app release builds (#663). CI runs on merge-queue groups
(#664).

**Internal consolidation, no behaviour change.** Operator CLI plumbing
(#606), contributor source-root resolution (#616), prefixed protocol digests
(#622), source event shape constructors (#620), the chain-agnostic claim
plumbing (#625), ingest free-text validation (#628), migration execution
(#630), daemon test fixtures (#631), daemon IPC dispatch and entry lookup
(#632), and the FFI string panic tails (#634, #646). Submission test options
are defaulted without changing enrollment checks (#623), Antigravity
dead-code allowances are narrowed to fixture metadata (#626), and fixture
defaults and async-only dispatch refusals are guarded (#643).

**Designs and plans, no shipped behaviour.** The private-inference design and
plan (#609), corrected against the API that shipped (#615); the ed25519
receipt verification plan, corrected against what was built (#641); and the
plan for moving the shells' words into the core (#648, #654).
