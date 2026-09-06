# Attested-inference first end-to-end run — record

Date: 2026-09-06. Run by the maintainer's session against the pilot, on
macOS, from a release-candidate build of `main`
(`trace-commons-contributor 0.10.0-rc`, commit 47697e76) with a device
enrolled in `tenant-zaki-pilot`.

**Outcome: the run reached the receipt and stopped there, on a defect in
our own design. A hosted exchange, its receipt, and every leg up to
verification all work. The receipt is signed by a per-model provider TEE
key that our client and witness do not and cannot pin. Attested inference
cannot currently succeed. No trace was submitted on the attested path.**

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

## First obstacle (solved): model substitution

Through the daemon-hosted proxy, **every request was initially routed to
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

The cause, established from IronWire's control API rather than guessed: the
backend disables DID take effect (only `nearai` was registered), but the
backend's `models` field is not an allowlist — the backend still advertised
the full 49-model upstream catalogue, leaving `claude-fable-5` available to
the router's fidelity ladder.

**The fix is a first-class IronWire feature, not a code change:**
`POST /_ironwire/pin {"backend":"nearai","model":"<hosted model>"}` with the
home's `control.token` forces both. With the pin set, the same request
served `Qwen/Qwen3.6-35B-A3B-FP8` with a bare-hex chat id
(`4eaaa9d8bc3d...`), correlated to its session, with the body captured. A
receipt for it fetched successfully (HTTP 200, ed25519).

## The real blocker: we verify receipts against the wrong key

The receipt for that hosted exchange is signed by
`aba45f0b8f90869baab26db02e8b01354bb8f8730769c60650cb7a635da602d4`, with
`signature_kind: provider_tee`. A second hosted model
(`Qwen/Qwen3.8-27B`) produced a receipt signed by a **different** key,
`73cf225ab4f09154ad8b299d4ac89425c7f25468a42ba9a87d09fcd4e87b8bf5`.

Neither is the gateway ed25519 key
(`cb6fc58f6bd685919fa42fb54d3fcfe03222e324bdda91f0bac6d5c73dc4f1c6`), and
neither appears anywhere in the attestation report — not in
`gateway_attestation`, and not in `model_attestations`, whose only entry for
the model carries an unrelated ECDSA address.

Both our verification paths pin the gateway key:

- the client, via `routing/attestation_report.rs::gateway_ed25519_key`,
  which reads `gateway_attestation.signing_address`;
- the witness, via `TRACE_COMMONS_WITNESS_GATEWAY_KEY_PIN` (#650), deployed
  to production on 2026-09-06 with that same value.

So **every real hosted-model receipt is refused by our own verification**,
client-side and at the witness. And because the signer varies per model, a
single pinned key cannot be made correct by changing its value.

This was never caught because the tests prove the report *parses* and that a
signature verifies under a key supplied by the same fixture. Nothing ever
checked a real receipt's signer against a real report's gateway key. It is
the fixture-matches-the-bug pattern, and it is exactly what this task
existed to find.

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
- **The deployed witness pin should be reconsidered immediately.** As set,
  it refuses every real hosted receipt. It ships behind the per-submission,
  invite-gated bypass so ordinary traffic is unaffected, but the control is
  not doing what it was deployed to do.
- **The design question is open:** the receipt signer has no attested
  binding available from the report endpoint today. Verifying the signature
  proves a receipt came from *someone*; nothing ties that someone to an
  attested enclave. This is the same gap already recorded for the ECDSA
  signer, now confirmed for ed25519 and shown to be per-model. Resolving it
  needs NEAR AI to publish a per-model attestation for the `provider_tee`
  signing key, or a different binding entirely.
- The remaining legs (dry-run submit, witness call, and the Step 9
  stored-envelope body-absence check) were not reached.

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
