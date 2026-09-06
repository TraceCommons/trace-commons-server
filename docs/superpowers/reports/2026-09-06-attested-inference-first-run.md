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
the model carries an unrelated ECDSA address. **[Superseded — see the
correction at the end of this report: the report endpoint was queried without
`signing_algo=ed25519`, and the ed25519 model attestation does exist.]**

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
  signing key, or a different binding entirely. **[Superseded — NEAR AI
  already publishes exactly that; see the correction below.]**
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

## Correction, later the same day

Two findings above are wrong, and the error was in how this run queried the
report endpoint rather than in what the endpoint offers.

**`signing_algo` is a query parameter of `GET /v1/attestation/report`, and its
default is ECDSA.** This run fetched
`?model=<model>&nonce=<nonce>` with no `signing_algo`, so the endpoint returned
the ECDSA model attestations — which is why the entry for the model appeared to
carry "an unrelated ECDSA address" and why the ed25519 receipt signer appeared
in no attestation. Fetching
`?model=<model>&signing_algo=ed25519&nonce=<nonce>` returns
`model_attestations` entries whose `signing_address` is exactly the
`provider_tee` ed25519 key that signs that model's receipts. Verified against
both models from this run:

| Model | receipt signer, and its ed25519 model attestation |
|---|---|
| `Qwen/Qwen3.6-35B-A3B-FP8` | `aba45f0b8f90869baab26db02e8b01354bb8f8730769c60650cb7a635da602d4` |
| `Qwen/Qwen3.8-27B` | `73cf225ab4f09154ad8b299d4ac89425c7f25468a42ba9a87d09fcd4e87b8bf5` |

So the open design question — "the receipt signer has no attested binding
available from the report endpoint today" — is **closed**, and it did not need
anything new from NEAR AI.

**Where the binding lives.** A `model_attestations` entry carries **no
`report_data` field**; only `gateway_attestation` has one, as an echo of what
its own quote already says. On a model attestation the binding is inside
`intel_quote`, at the TDX `report_data` position — byte offset 568, 64 bytes —
holding `signing_address || request_nonce` exactly. A verifier that requires a
`report_data` field refuses every real model attestation.

What stands unchanged from the findings above: the signer is per model for
the receipts this run captured, so one pinned key cannot be made correct by
changing its value; and the deployed gateway pin refused every real receipt.

*[Corrected again, 2026-09-06 — see "Second correction" at the end: "the
gateway key signs no receipt" is false. It signs every Responses API
receipt. It signed none of the ones captured here because every call in this
run went over Chat Completions.]*

Both verification paths were rewritten against the ed25519 model attestation,
and the gateway pin variable was retired — the witness now refuses to start if
it is still set, rather than starting unpinned while an operator believes it
pins. The fixtures are live captures of the reports and receipts named above,
so this specific mistake cannot recur silently.

The dormancy warning in `docs/operator/attested-inference.md` stays: this
correction fixes the verification, and does **not** mean a hosted exchange has
completed end to end. The remaining legs listed above were still not reached.

## Second correction, same day

The correction above is right about `signing_algo` and about where the
binding lives. One sentence in it is wrong.

**"The gateway key signs no receipt" is false.** NEAR AI issues two kinds of
receipt for the same hosted model, and the *request protocol* decides which:

| request protocol | `signature_kind` | signer |
|---|---|---|
| Chat Completions, `POST /v1/chat/completions` | `provider_tee` | the per-model ed25519 key in `model_attestations` |
| Responses API, `POST /v1/responses` | `gateway` | the shared ed25519 key in `gateway_attestation` |

Both were captured live against `Qwen/Qwen3.6-35B-A3B-FP8`. Every call in
this run went over Chat Completions, so every receipt it saw was
`provider_tee` — which is why the gateway key appeared to sign nothing.

This matters for the harness this report recommends. **The Codex CLI speaks
the Responses API exclusively**, having dropped `wire_api = "chat"`, so a
Codex-driven rerun of this harness produces `gateway` receipts throughout,
and a verifier consulting `model_attestations` alone refuses all of them. A
Responses exchange's receipt is also retrieved by the **full `resp_`-prefixed
identifier**; stripping the prefix returns 404.

Both verification paths now route on the receipt's own `signature_kind` and
check each kind against its own attested key source, never the other. The
witness pins them separately —
`TRACE_COMMONS_WITNESS_MODEL_KEY_PINS` and
`TRACE_COMMONS_WITNESS_GATEWAY_RECEIPT_KEY_PINS`, both replacing the retired
`TRACE_COMMONS_WITNESS_GATEWAY_KEY_PIN`, whose presence now refuses the boot.
Enabling them is a three-deployment rollout; step 1, the witness deployed
with both unset, is done in production. `deploy/witness/README.md` has the
derivation and the ordering.

One consequence worth recording here rather than only in the runbook: a
gateway receipt's signed text is the two-part `{requestHash}:{responseHash}`
and names **no model**, so it attests the bytes and not the model that served
them. Anything keying credit, scoring or eligibility off a declared model
must not read a gateway receipt as evidence of that model.

What this correction does **not** change: the dormancy warning in
`docs/operator/attested-inference.md` stays. The remaining legs listed above
were still not reached.
