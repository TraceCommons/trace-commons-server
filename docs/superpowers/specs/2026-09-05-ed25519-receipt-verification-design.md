# Verifying the receipt key that is actually attested

**Status:** approved design, **partly superseded**. Trust boundary decided
2026-09-05; two of its premises were corrected on 2026-09-06 — see
"Correction, 2026-09-06" at the end of this document before acting on the
table below. The decision (verify the ed25519 receipt) stands; the account of
what the report contains, and of which key signs a receipt, does not.

`trace-commons-attestation` verifies NEAR AI receipts by EIP-191 secp256k1
recovery, so it fetches them with `signing_algo=ecdsa`. Measured live, that
recovers `0x614bc66ff0407dbb70b9c7ca1f5e983e4a02c921` — **a key that appears
nowhere in NEAR AI's attestation report**.

So a verified receipt currently proves NEAR AI produced it. It does not
prove any enclave did.

## What the attestation report actually contains

`GET /v1/attestation/report?model=..&nonce=..` — **note the missing
`signing_algo`; that omission is what makes the table below wrong. See the
correction at the end.**

| Key | Algo | Attested |
|---|---|---|
| `cb6fc58f…` | ed25519 | **Yes.** Gateway. `report_data == signing_address ‖ request_nonce` inside a TDX quote, with a caller-supplied nonce. |
| `0xe5d0fec4…` | ecdsa | Yes. The MODEL enclave — `dstack-nvidia-0.5.5`, TDX quote plus NVIDIA HOPPER evidence. |
| `0x614bc66f…` | ecdsa | **No.** The key we verify. Absent from the report. |

The same ECDSA signer appeared across two chat ids and two models, so it is
one gateway-level key rather than a per-request one. It is simply not
attested.

*[Superseded in part, 2026-09-06: the row reading "`cb6fc58f…` ed25519 —
Gateway" is right, but it is **not** the only attested ed25519 key, and the
receipt signer is not unattested. The fetch above omitted
`signing_algo=ed25519`, whose default is ECDSA. See the correction at the end
of this document.]*

## The decision

**The gateway is the trust boundary.** The report supports treating it as a
real one: `ohttp_key_config` and `ohttp_attestation` are signed by the
gateway's ed25519 key, so the gateway-to-model hop is Oblivious HTTP rather
than an unprotected internal call.

Accepting that does **not** accept the ECDSA key, and the two are easy to
conflate. `0x614bc66f…` is not attested *as the gateway* either. The attested
gateway key is the ed25519 one.

**So: fetch and verify the ed25519 receipt.** Then a verified receipt means
"signed by a key committed inside the gateway's TDX quote", which is exactly
the claim now accepted as sufficient.

## Why this makes the primitive stronger, not just different

The ECDSA path *recovers* a signer from a signature and compares it to a
claimed address. Recovery answers "some key produced this", and the
comparison is what turns it into "that key did" — a step that has to be
right, and which `ReceiptError::SignerMismatch` exists to catch.

Ed25519 verifies a signature **against a key you already hold**. There is no
recovered value to mis-compare. Given the attested address from the report,
verification is a direct yes or no.

## Scope

1. **Ed25519 receipt verification** in `trace-commons-attestation`, alongside
   the existing EIP-191 path rather than replacing it. Both forms exist on
   the wire and old receipts do not become unverifiable.
2. **Switch the fetch** in `trace-commons-contributor`'s
   `routing/receipt.rs` to `signing_algo=ed25519`.
3. **An optional check that the signer matches an attestation-report
   address.** Optional because it requires a second network call and a
   policy about how fresh a report must be; the verification itself is
   useful without it.

### Dependency

`ring = "0.17"` is already a **direct** dependency of
`trace-commons-contributor` and `trace-commons-server`, and already in
`trace-commons-attestation`'s transitive graph. Adding it there as a direct
dependency adds **no packages** — the same situation as the `receipt`
feature, whose manifest comment already records that it saves none. The GTK
vendored flatpak source set is therefore unaffected.

### Not in scope

- **Verifying the TDX quote itself.** Nothing in our code path checks the
  quote; we would be trusting the report's self-description. Closing that
  needs `dcap-qvl` and a collateral fetch, and it is a larger dependency and
  policy question. Until it lands, an attestation-report address is a claim
  by NEAR AI, not a proof.
- **Binding a receipt to the model enclave.** The model has its own quote and
  its own key, and nothing observed binds a receipt to it. With the gateway
  as the trust boundary this is not required, but it is why a receipt cannot
  say "this model ran it".

## What a verified receipt will then claim

That NEAR AI's gateway — running in a TDX enclave whose quote commits to the
signing key — produced this response over these exact request bytes.

It will still not say the trace is genuine, that the model enclave served it,
or that unattested turns did not occur.

## Open questions

- **Is `0x614bc66f…` attested anywhere else?** It was absent from one report
  for one model. Worth one question to NEAR AI before concluding it is
  unattested by design rather than by omission.
- **How fresh must an attestation report be** for its address to be trusted?
  The nonce makes a single fetch fresh; nothing says how long that lasts.
- **Does the ed25519 signer rotate?** If it does, a pinned address breaks and
  the report has to be re-fetched on a schedule.

## Related

`2026-09-04-attested-inference-release-design.md` — the system this verifies
for, and the limits it currently states.

---

## Correction, 2026-09-06

Two premises above are wrong. The decision they led to — verify the ed25519
receipt — is unaffected and was the right call; what needs correcting is the
account of the report and of the signer.

**1. `signing_algo` is a query parameter of `GET /v1/attestation/report`, and
its default is ECDSA.** Every fetch behind the table above omitted it, so the
endpoint returned the ECDSA model attestations. Fetching
`?model=..&signing_algo=ed25519&nonce=..` returns `model_attestations` entries
whose `signing_address` is the per-model ed25519 key that signs that model's
receipts. So "the key we verify is absent from the report" was an artefact of
asking the wrong question, not a property of the endpoint. The design's own
scope item 3 ("an optional check that the signer matches an
attestation-report address") is therefore not optional-because-unavailable;
it is implemented and it passes against live captures.

**2. The binding is not a JSON field.** A `model_attestations` entry carries
**no `report_data` field at all**. Only `gateway_attestation` has one, as an
echo of its own quote. On a model attestation the binding lives inside
`intel_quote` at the TDX `report_data` position — byte offset 568, 64 bytes,
`signing_address || request_nonce` — and is valid to read only once the quote
header shows version 4 and TEE type `0x81`. A verifier that requires a
`report_data` field refuses every real model attestation.

**3. There are two kinds of receipt, and the request protocol picks one.**
This design speaks of "the" receipt signer. There are two, both legitimate,
for the same hosted model:

| request protocol | `signature_kind` | attested by |
|---|---|---|
| Chat Completions, `POST /v1/chat/completions` | `provider_tee` | the per-model key in `model_attestations` |
| Responses API, `POST /v1/responses` | `gateway` | the shared key in `gateway_attestation` |

Each kind is checked against its own key source and never the other. This
matters because the Codex CLI speaks the Responses API exclusively, so a
deployment whose contributors use Codex sees only `gateway` receipts.

A gateway receipt's signed text is two-part, `{requestHash}:{responseHash}`,
and names no model — so it attests the bytes and not the model. The
three-part chat-completions form does bind the model.

**What is unchanged.** The gateway remains a real trust boundary for the
reasons this document gives (`ohttp_key_config` and `ohttp_attestation` are
signed by its ed25519 key). Quote verification is still out of scope: a key
read from a report is still a claim by NEAR AI checked for internal
consistency and freshness, not a proof.

Operator-facing consequences, including the retirement of
`TRACE_COMMONS_WITNESS_GATEWAY_KEY_PIN` in favour of
`TRACE_COMMONS_WITNESS_MODEL_KEY_PINS` and
`TRACE_COMMONS_WITNESS_GATEWAY_RECEIPT_KEY_PINS`, and the three-deployment
rollout the pins require, are in `deploy/witness/README.md` and
`docs/operator/attested-inference.md`.
