# Local compute adapter

The production `ComputeController::open` / `tc_compute_open` path remains
unavailable. Only the explicit `open_local` constructor enables a Unix debug-build
development worker. Builds without debug assertions refuse that constructor. The native app does
not discover a worker through PATH, environment overrides, or persisted settings.

The local harness accepts an absolute state root, an absolute worker binary, its
expected SHA-256, a loopback WebSocket coordinator, a RAM scheduling allowance in
GiB, and an optional observation duration (default 45 seconds, maximum 300):

```sh
cargo run -p trace-commons-contributor --example compute-local -- \
  /absolute/test-state /absolute/holonear EXPECTED_SHA256 \
  ws://127.0.0.1:PORT 1 45
```

The harness explicitly grants compute consent, prints label-only JSON snapshots,
and requests shutdown. It exits nonzero if startup fails, no authenticated status
arrives, or worker termination remains unconfirmed. A synthetic public payout
identifier `compute-pilot.testnet` satisfies the local worker CLI preflight; this
does not configure a wallet, contact a production pool, or move funds.

## Contract and dependency decision

The IPC v0 executable contract is pinned to Holonear revision
`ef4e6e2479e8395f7d972d3342bad97851f2104e`, with committed source-derived
vectors in `crates/trace-commons-contributor/tests/fixtures/worker_ipc_v0.json`.
Trace Commons independently signs and verifies those exact body bytes/signatures
using its existing `ring` dependency. Golden cases cover Status and Drain;
tampering with direction, nonce, version, instance, or payload is refused.

A direct Holonear desktop-core dependency would pull protocol/crypto/store,
account, and AWS components, and couple the contributor to Holonear's newer
compiler. The native adapter instead uses existing ring, serde, sha2, hex and
Tokio. It enables Tokio's existing `process` feature; no package or lockfile
dependency is added. This preserves the contributor's Rust 1.92 boundary.

Frames are four-byte big-endian lengths plus JSON, capped at 65,536 bytes before
allocation. Every request uses a new random nonce. Status has a two-second
deadline, Drain ten seconds. There is no legacy transport fallback. A signed
response proves current endpoint liveness and the worker's reported state and
assignment; it does not prove training progress, measured memory, or earnings.

## Lifecycle and ownership

One background actor owns settings transactions and the child process. Ordinary
start commands allow one in flight, with visible busy rejection. Pause, Disable,
and Shutdown use durable coalesced stop intent outside that queue. Revocation
cannot be replaced by a weaker Pause. Stops interrupt startup readiness, and
discard pending starts. Settings remain independent of trace enrollment; all
consent restores paused and every later launch needs explicit Enable or Resume.

Shutdown is terminal for a controller: subsequent starts are refused. The C
shutdown call waits at most 30 seconds and returns `worker_stopped`, separately
from `drain_outcome` and `stop_outcome`. A timeout returns unconfirmed termination;
retain the handle and retry. Only after confirmed termination may a shell reopen
a controller (restored paused). Free is not a synchronous drain barrier; shells
must explicitly shut down before freeing a live controller.

The adapter holds `compute/worker/node/controller.lock` until its owned child is
reaped and probes `worker.lock` before launch. The worker independently owns its
lifetime lock; a competing process is refused, never adopted or killed by PID.
Lock files are never unlinked. Readiness requires the atomically published
`node/worker-endpoint.json` with matching version, freshly generated launch public
key and loopback/nonzero address, then a signed response. Endpoint reads are
bounded to 4096 bytes. Readiness defaults to 30 seconds in the harness.

Three consecutive failed status observations initiate stop. The snapshot also
marks telemetry older than six seconds stale. Cooperative Drain is followed by
three seconds for process exit, then owned-child kill and a two-second reap wait.
Forced termination never implies coordinator acknowledgement. Failed reaping
retains ownership. Duplicate stops preserve previously observed drain evidence.

Child environment is cleared. HOME/USERPROFILE, caches and temporary paths are
under the worker home; only fixed system PATH, required Windows SystemRoot,
HOLONEAR_HOME, the ephemeral IPC capability, and relay-only peer transport are
supplied. Standard streams are null. The worker cannot inherit contributor
tokens through the environment, but this is not an OS filesystem sandbox.

## Remaining release gates

Local hashing immediately before launch is a development integrity check, not
signed packaging or protection against a same-user executable replacement race.
The child-environment capability is not isolated from a compromised host.
Cross-instance machine-wide resource arbitration, enforced resource limits,
filesystem restrictions, platform signing/attestation acceptance, and production
account/pool configuration remain unimplemented. The production capability must
remain closed until those gates and real-device lifecycle validation pass.


## Evidence scope and local credential boundary

The original `worker_ipc_v0.json` vectors were derived from reading revision
`ef4e6e2479e8395f7d972d3342bad97851f2104e`. They are reproducible local protocol
fixtures, not captured responses or upstream-generated interoperability evidence.
The signed fake-worker child exercises real endpoint publication, request checking,
readiness, state transitions and drain, but implements the same source-derived
model. A running upstream worker transcript remains a release gate.

### Orchard-generated compatibility fixture (UNMERGED source)

`crates/trace-commons-contributor/tests/fixtures/orchard_worker_ipc_v0.json`
contains exact bytes generated in the `nearai/orchard` repository
(configured remote `https://github.com/nearai/orchard.git`, now redirected by
GitHub to `https://github.com/nearai/holonear.git`) at revision
`4d2227661d9a0feab8aa1e1f0baeea011b11d001`, branch
`resume/worker-ipc-protocol`, based on main
`e366e5c8d3ff705d10cc7e738191ae6fa2bc5e26`. **This generating revision is
UNMERGED**; its protocol proposal still requires Orchard
maintainer approval. The branch is published within the nearai organization in a private
repository and is not publicly reproducible. Reproduction below requires
authorized access; this is not an approved upstream release or dependency.

These vectors pin a **proposed** protocol and remain green even if the unmerged
proposal changes. On Orchard merge, regenerate at the actual merged revision
and update the fixture digest and metadata pins; [tracking issue #640](https://github.com/TraceCommons/trace-commons/issues/640)
remains open until that regeneration is reviewed. This Trace issue does not
substitute for Orchard maintainer approval.

In an authorized internal checkout containing that revision, reproduce with:

```sh
scripts/generate-worker-ipc-vectors.sh
```

The script runs `UPDATE_WORKER_IPC_VECTORS=1 cargo test -p holonear-protocol
--lib worker_ipc::tests::orchard_generated_vectors_are_stable --locked`.
Copy `crates/holonear-protocol/tests/fixtures/worker_ipc/orchard_v0.json`
without reformatting. Its SHA-256 is
`eb7a86d64173203a39e332f40cc795da5f3e631c0951526b523258b88c30b23b`.
The fixture also records the generator module's SHA-256
`402e6d791a890030f863e2f246b5f82f71b12371143f8faf30deee78bd589d68` and the
original worker implementation revision
`7d6f70512fb6cd9faf936fc27ca367a5cd539de5`.
The generating Orchard protocol crate is licensed MIT OR Apache-2.0.

#### What a reader without access can and cannot check

A third party with only this repository **can**:

- recompute the fixture's SHA-256 and confirm it is the digest published
  above -- `orchard_fixture_provenance_fields_are_present_and_match_the_documentation`
  does exactly this in-tree, over `include_bytes!` of the committed file, and
  also asserts that the fixture's provenance fields are present, well formed,
  and identical to the values named in this document;
- re-verify every signature in the fixture against Trace's own production
  request signer and response verifier, and confirm the tampering arms fail.

That same reader **cannot**:

- regenerate the fixture. The generating repository is private to the nearai
  organization, so the reproduce command below runs only in an authorized
  checkout;
- establish from this repository that Orchard, rather than a local
  regeneration by Trace, produced the bytes. The seeds are public and the
  signature scheme is deterministic, so bytes produced by Trace's own signer
  would satisfy every cryptographic assertion here identically. The digest
  and metadata pins are drift tripwires -- they detect a *changed* fixture --
  not evidence of origin.

The claim this fixture supports is therefore cross-implementation message
compatibility as captured at a named revision, not independently verifiable
upstream provenance.

Orchard's actual `holonear-crypto` / ed25519-dalek path generated these
signatures using two distinct public fixed test seeds for Status and Drain.
The generator test bakes in seeds `[9; 32]` / `[17; 32]` and nonces
`[10; 32]` / `[18; 32]`; these are not command-line inputs. The original
source-derived fixture intentionally uses `[7; 32]` for both cases.
The Trace test pins both metadata fields and these inputs against constants.
Those tripwires detect metadata/input drift; because the seeds are public,
they are not cryptographic proof of generator origin, and an external reader
cannot independently inspect the private generator repository.
The Trace `orchard_generated_vectors_pin_both_seeds_and_reject_tampering`
test calls its production ring request signer and response verifier: exact
request signatures and serialized bodies must agree, and wrong direction,
nonce, version, instance or payload must fail. Both crypto implementations
therefore agree on these messages; the original source-derived fixture remains
separate and retains its existing test.

This is cross-implementation message compatibility, not a running-node
transcript, pool assignment, completed workload, signed package, independently
trusted package manifest, release provenance or production launch authorization.
No test executes Orchard code or needs an Orchard checkout or network access.
Production compute and its remaining release gates stay unchanged.

The per-launch seed is passed through the child process environment. After spawn,
the parent removes the credential from its `Command` configuration and clears the
original byte array; this is not a guarantee that allocator copies or the child's
environment have been erased. Same-user process inspection remains inside the
local trust boundary. No seed is included in logs or status snapshots.

The development gate is `debug_assertions`, not the Cargo profile name. A release
profile explicitly built with debug assertions enables the local-development
entrypoint; such a build is not a production compute capability. Ordinary app
construction stays unavailable regardless. Removing the existing C ABI symbol
would change the published header contract; an explicit build capability and
release-policy decision remain required before any packaged launch path exists.

When an unknown process still holds `worker.lock`, the controller reports
`worker-already-running` and tells the user to stop its owning app or restart the
machine after a crash, then explicitly Resume. It never adopts an endpoint or kills
by PID. `tc_compute_free` deliberately retains a handle while worker termination
is unconfirmed; losing that ownership would be less safe than retaining it.
