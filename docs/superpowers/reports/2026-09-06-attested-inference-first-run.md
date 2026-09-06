# Attested-inference first end-to-end run — record

Date: 2026-09-06. Run by the maintainer's session against the pilot, on
macOS, from a release-candidate build of `main`
(`trace-commons-contributor 0.10.0-rc`, commit 47697e76) with a device
enrolled in `tenant-zaki-pilot`.

**Outcome: PARTIAL. The attested-inference receipt leg could not complete,
for a reason outside trace-commons — IronWire's router substitutes the
requested model. Everything up to and including body capture and session
correlation works. No trace was submitted on the attested path.**

## What was verified working

- **Daemon-hosted IronWire (this release's new path).** `private_inference=true`
  brings the in-process proxy to `running` on a loopback port; the state
  object reports `running` / `running_elsewhere` / `off` truthfully across
  cycles, and cycling the switch correctly reclaimed the home from a
  standalone `ironwire` (`running_elsewhere` -> `running`). #652/#665
  lifecycle behaviour confirmed live.
- **Catalogue.** With `NEARAI_API_KEY` in the daemon environment the proxy
  serves a live model catalogue.
- **Body capture.** `~/.ironwire/config.toml` `[capture] enabled=true
  bodies=true`; exchanges are recorded to `ledger.sqlite` with request and
  response bodies on local disk.
- **Session correlation.** The contributor joins an IronWire exchange to a
  session by matching the exchange's `client_session_id` to the transcript's
  `conversation_id` (whole-UUID suffix, `routing/enriched.rs`). IronWire
  reads that id from the `x-ironwire-session-id` header (neutral, always
  wins) or the protocol-native `session-id`. A call carrying
  `x-ironwire-session-id: <uuid>` populated `client_session_id` correctly.
  (An arbitrary `x-session-id` does NOT — that was the first dead end.)
- **Receipts exist at the gateway.** Calling the NEAR AI gateway directly
  for `Qwen/Qwen3.6-35B-A3B-FP8` serves that model with a bare-hex chat id
  (`f731ede0...`), the shape a receipt is keyed on. Hosted receipts are
  reachable in principle.
- **Witness.** Deployed 2026-09-06 with the gateway-key pin set; new
  measurement `mrconfigid:01454992a4...` verified live and pinned on the
  pilot. (See the witness deploy PR and `deploy/witness/README.md`.)

## The blocker

Through the daemon-hosted proxy, **every request was routed to
`claude-fable-5`** — a brokered model (`msg_`-prefixed id), for which NEAR AI
serves no receipt (`GET /signature/{chat_id}` 404s for brokered ids). This
happened for `Qwen/Qwen3.6-35B-A3B-FP8` requests specifically, and did not
change after:

- setting the correct session header (correlation succeeded, model still
  substituted);
- a full `private_inference` off/on cycle with the port confirmed freed and
  rebound (so the embedded proxy restarted and re-read config);
- rewriting `~/.ironwire/config.toml` to disable every backend except
  `nearai` and pin its `models` list to only `Qwen/Qwen3.6-35B-A3B-FP8`.

IronWire's router (`ironwire_core::policy`, the `Rung` ladder) treats the
client's requested model as a hint and selects the backend/model by its own
fidelity policy; the backend `models` field sets tier/ordering, not an
allowlist. So a hosted model cannot currently be forced through the proxy
from configuration alone, and without a hosted exchange there is no receipt
to fetch, verify, or carry to the witness.

## Consequence and recommendation

- The attested-inference path has still **not** completed end to end. Per
  the release plan's own rule, a path that has never completed should not be
  presented as validated. The dormancy warning in
  `docs/operator/attested-inference.md` must stay until a hosted exchange
  yields a verified receipt and a body-free stored envelope.
- The witness gateway-key pin was deployed 2026-09-06 at the maintainer's
  direction, ahead of a completed run. This does not affect ordinary
  (non-attested) submissions — the witness bypass is per-submission and
  invite-gated — but it means enforcement is live before the client path it
  guards has been exercised.
- **Next step is an IronWire change, not a trace-commons one:** a routing
  mode (or a per-request pin) that sends a named NEAR-AI-hosted model
  through unsubstituted. Once that exists, the rest of the leg (dry-run
  submit, receipt fetch, witness call, and the Step 9 stored-envelope
  body-absence check) is ready to run and was not reached here.

## A reusable harness, for when the routing is fixed

- Driver: `codex exec` (headless) pointed at the proxy via
  `-c model_providers.<name>.base_url=http://127.0.0.1:<port>/openai/v1`.
  Codex sends `session-id` natively and its rollout stem ends with that
  UUID, so the contributor join is automatic — the one agent among the
  common options that correlates without extra work. opencode / pi /
  nanocodex are not ingestible (the contributor reads only Claude, Codex,
  Gemini, Cline transcripts); Cline is supported but is a VS Code extension,
  not headless, and its header-to-store correlation is unverified.
- Proxy: nearai backend only, capture bodies on, pinned to one hosted model
  — pending the routing fix above.
