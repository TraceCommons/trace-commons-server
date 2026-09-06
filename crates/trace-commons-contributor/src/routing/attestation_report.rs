//! Reading signing keys out of a NEAR AI attestation report.
//!
//! `GET {base}/attestation/report?model=..&signing_algo=ed25519&nonce=..`
//! returns, among much else, a `gateway_attestation` and a
//! `model_attestations` array. Each is an attestation object binding a key to
//! a nonce this client chose: inside its TDX quote, `report_data` is
//! `signing_address || request_nonce`. A key read from an attestation whose
//! `report_data` carries OUR nonce is one that was attested for us, now,
//! rather than one copied from an old report.
//!
//! `report_data` is read out of `intel_quote` at the fixed v4 TDX offset, not
//! from a JSON field: a `model_attestations` entry **has no `report_data`
//! field**. Only `gateway_attestation` carries one, as an echo of its own
//! quote, and it is checked against the quote rather than trusted instead of
//! it. See [`trace_commons_attestation::receipt::attested_ed25519_key`].
//!
//! # Which key signs a receipt: both, and the protocol decides
//!
//! NEAR AI issues two legitimate kinds of receipt for the **same hosted
//! model**. A Chat Completions call gets `signature_kind: "provider_tee"`,
//! signed by that model's key in `model_attestations`. A Responses API call
//! gets `signature_kind: "gateway"`, signed by the single key in
//! `gateway_attestation`. Both were captured live.
//!
//! This is not symmetric in importance: the Codex CLI speaks the Responses
//! API exclusively, having dropped `wire_api = "chat"`, so a client that
//! consults `model_attestations` alone reports every Codex-driven receipt as
//! unattested.
//!
//! So **both** readers below are receipt-relevant, and the receipt's own
//! `signature_kind` picks between them -- see
//! [`trace_commons_attestation::receipt::attested_keys_for_receipt`], which
//! is the router this module's callers go through. Neither key is ever tried
//! against the other kind: that would let a key attested for one role vouch
//! for the other. One fetch returns both objects, so routing costs no second
//! request.
//!
//! `signing_algo` is a **query parameter** of the report endpoint, and that is
//! the piece this originally missed: without `signing_algo=ed25519` the
//! endpoint answers with the ECDSA model attestations, whose keys sign nothing
//! we verify. [`attestation_report_url`] always sends it.
//!
//! **This module does not verify the quote.** It reads the report's
//! self-description and checks its internal consistency. Until quote
//! verification exists, a key from here is a claim by NEAR AI, not a proof,
//! and the config gate that enables this check says so.
//!
//! Nothing here logs. The report holds keys and identifiers; none of them
//! belong on an operational surface.

use trace_commons_attestation::receipt::{AttestedKeyError, ReceiptAlgo, attested_ed25519_key};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttestationReportError {
    #[error("attestation report is not the expected JSON shape")]
    Malformed,
    #[error("attestation report gateway is not ed25519")]
    NotEd25519,
    #[error("attestation report was issued for a different nonce")]
    NonceMismatch,
    #[error("attestation report_data does not commit to the listed key and nonce")]
    ReportDataMismatch,
    #[error("attestation report gateway key is not 32 bytes of hex")]
    KeyMalformed,
    #[error("attestation report base URL is not a valid https URL")]
    UrlInvalid,
    #[error("attestation report attests no ed25519 signer for this model")]
    ModelNotAttested,
    /// The receipt named a `signature_kind` this client cannot resolve to an
    /// attested key source -- an unknown value, or no value at all. Refused
    /// rather than checked against every key.
    #[error("receipt signature kind names no attested key source")]
    SignatureKindUnrecognised,
}

impl From<AttestedKeyError> for AttestationReportError {
    fn from(error: AttestedKeyError) -> Self {
        match error {
            AttestedKeyError::Malformed => Self::Malformed,
            AttestedKeyError::NotEd25519 => Self::NotEd25519,
            AttestedKeyError::NonceMismatch => Self::NonceMismatch,
            AttestedKeyError::ReportDataMismatch => Self::ReportDataMismatch,
            AttestedKeyError::KeyMalformed => Self::KeyMalformed,
            AttestedKeyError::ModelNotAttested => Self::ModelNotAttested,
            AttestedKeyError::SignatureKindUnrecognised => Self::SignatureKindUnrecognised,
        }
    }
}

/// The URL a fetch would call.
///
/// # Errors
///
/// [`AttestationReportError::UrlInvalid`] when `base` does not parse or is
/// not https.
pub fn attestation_report_url(
    base: &str,
    model: &str,
    nonce: &str,
) -> Result<url::Url, AttestationReportError> {
    let mut url = url::Url::parse(base).map_err(|_| AttestationReportError::UrlInvalid)?;
    if url.scheme() != "https" {
        return Err(AttestationReportError::UrlInvalid);
    }
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| AttestationReportError::UrlInvalid)?;
        segments.pop_if_empty().push("attestation").push("report");
    }
    url.query_pairs_mut()
        .append_pair("model", model)
        // Not optional and not configurable. `signing_algo` selects which
        // `model_attestations` the endpoint returns, and the default is ECDSA
        // -- whose keys sign nothing this client verifies. A report fetched
        // without it looks entirely well formed and attests the wrong thing.
        .append_pair("signing_algo", ReceiptAlgo::Ed25519.as_wire())
        .append_pair("nonce", nonce);
    Ok(url)
}

/// The gateway's ed25519 signing key, if the report binds it to `expected_nonce`.
///
/// This is the key a `gateway` receipt is signed by -- the Responses API form
/// -- and it is **not** the key a `provider_tee` receipt is signed by. See the
/// module docs: the receipt's own `signature_kind` picks between this and
/// [`model_ed25519_keys`], and neither key is ever tried against the other
/// kind. Both apply the same binding discipline to a different attestation
/// object.
///
/// The parsing and the binding check live in
/// [`trace_commons_attestation::receipt::attested_ed25519_key`], so a gateway
/// attestation and a model attestation cannot come to be checked differently.
///
/// # Errors
///
/// [`AttestationReportError`] when the report is not the expected shape, the
/// gateway is not ed25519, the report was issued for a different nonce, or
/// `report_data` does not commit to the listed key and nonce.
pub fn gateway_ed25519_key(
    report_json: &str,
    expected_nonce: &str,
) -> Result<String, AttestationReportError> {
    let document: serde_json::Value =
        serde_json::from_str(report_json).map_err(|_| AttestationReportError::Malformed)?;
    let gateway = document
        .get("gateway_attestation")
        .ok_or(AttestationReportError::Malformed)?;
    Ok(attested_ed25519_key(gateway, expected_nonce)?)
}

/// The ed25519 keys the report attests **for `model`** -- the set a receipt's
/// signer is actually checked against.
///
/// # Errors
///
/// [`AttestationReportError`] when the report is not the expected shape,
/// attests no ed25519 signer for this model, or carries an entry for this
/// model that does not bind to `expected_nonce`.
pub fn model_ed25519_keys(
    report_json: &str,
    expected_nonce: &str,
    model: &str,
) -> Result<Vec<String>, AttestationReportError> {
    Ok(trace_commons_attestation::receipt::model_ed25519_keys(
        report_json,
        expected_nonce,
        model,
    )?)
}

/// Whether a verified receipt's signer is a key a report attested.
///
/// Re-exported from `trace_commons_attestation::receipt`, where they moved so
/// that the hosted witness -- which compares a receipt's signer against keys
/// an operator configured rather than ones read from a live report -- makes
/// the same comparison this client does, from the same code.
pub use trace_commons_attestation::receipt::{signer_is_attested, signer_is_attested_for_model};

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape NEAR AI actually returns, reduced to the fields read.
    /// `report_data` is `signing_address || request_nonce`, which is what
    /// binds the key to a caller-chosen nonce inside the TDX quote.
    const NONCE: &str = "482934fb749d13aa81b2e543a253cf4d8cc847dab55a8d49989effd5023ddb5d";
    const KEY: &str = "cb6fc58f6bd685919fa42fb54d3fcfe03222e324bdda91f0bac6d5c73dc4f1c6";

    /// The `gateway_attestation` subtree of a report NEAR AI actually
    /// returned (2026-09-05), reduced to the fields this parser reads. Every
    /// other fixture in this module is authored beside the code; this one
    /// pins that the real response nests these fields the way the parser
    /// expects and that `report_data` really is `signing_address ||
    /// request_nonce`. `NONCE` and `KEY` above are the live values, so this
    /// composes with them directly.
    const LIVE_REPORT: &str = r#"{"gateway_attestation":{"signing_address":"cb6fc58f6bd685919fa42fb54d3fcfe03222e324bdda91f0bac6d5c73dc4f1c6","signing_algo":"ed25519","request_nonce":"482934fb749d13aa81b2e543a253cf4d8cc847dab55a8d49989effd5023ddb5d","report_data":"cb6fc58f6bd685919fa42fb54d3fcfe03222e324bdda91f0bac6d5c73dc4f1c6482934fb749d13aa81b2e543a253cf4d8cc847dab55a8d49989effd5023ddb5d"},"model_attestations":[],"ohttp_attestation":{}}"#;

    fn report(nonce_in_report_data: &str) -> String {
        format!(
            r#"{{"gateway_attestation":{{"signing_address":"{KEY}","signing_algo":"ed25519","request_nonce":"{NONCE}","report_data":"{KEY}{nonce_in_report_data}"}}}}"#
        )
    }

    #[test]
    fn the_gateway_key_is_read_when_report_data_binds_the_nonce_we_sent() {
        assert_eq!(gateway_ed25519_key(&report(NONCE), NONCE).unwrap(), KEY);
    }

    /// A report whose report_data carries a different nonce is stale or
    /// replayed, and its key is not accepted. This is the whole point of
    /// sending a nonce.
    #[test]
    fn a_report_for_a_different_nonce_is_refused() {
        let other = "0".repeat(64);
        assert_eq!(
            gateway_ed25519_key(&report(&other), NONCE).unwrap_err(),
            AttestationReportError::NonceMismatch
        );
    }

    /// report_data must equal key || nonce exactly. A report that lists the
    /// key but whose report_data does not commit to it is not a binding.
    #[test]
    fn report_data_that_does_not_commit_to_the_key_is_refused() {
        let body = format!(
            r#"{{"gateway_attestation":{{"signing_address":"{KEY}","signing_algo":"ed25519","request_nonce":"{NONCE}","report_data":"{}{NONCE}"}}}}"#,
            "f".repeat(64)
        );
        assert_eq!(
            gateway_ed25519_key(&body, NONCE).unwrap_err(),
            AttestationReportError::ReportDataMismatch
        );
    }

    /// `strip_prefix` on a too-short (or empty) key would trivially succeed
    /// against any `report_data`, so this checks the key's shape before the
    /// split ever happens. Each case's `report_data` is set to exactly the
    /// key followed by the real nonce -- the shape that would otherwise
    /// pass -- so the test would pass the pre-guard code and fails only on
    /// this check.
    #[test]
    fn a_report_whose_key_is_not_32_bytes_is_refused() {
        let cases = ["", "cb", &"g".repeat(64)];
        for bad_key in cases {
            let body = format!(
                r#"{{"gateway_attestation":{{"signing_address":"{bad_key}","signing_algo":"ed25519","request_nonce":"{NONCE}","report_data":"{bad_key}{NONCE}"}}}}"#
            );
            assert_eq!(
                gateway_ed25519_key(&body, NONCE).unwrap_err(),
                AttestationReportError::KeyMalformed,
                "key {bad_key:?} should be refused as malformed"
            );
        }
    }

    #[test]
    fn a_non_ed25519_gateway_is_refused() {
        let body = format!(
            r#"{{"gateway_attestation":{{"signing_address":"{KEY}","signing_algo":"ecdsa","request_nonce":"{NONCE}","report_data":"{KEY}{NONCE}"}}}}"#
        );
        assert_eq!(
            gateway_ed25519_key(&body, NONCE).unwrap_err(),
            AttestationReportError::NotEd25519
        );
    }

    #[test]
    fn the_report_url_carries_model_and_nonce() {
        let u = attestation_report_url(
            "https://cloud-api.near.ai/v1",
            "Qwen/Qwen3.6-35B-A3B-FP8",
            NONCE,
        )
        .unwrap();
        assert_eq!(u.path(), "/v1/attestation/report");
        assert!(u.query().unwrap().contains(&format!("nonce={NONCE}")));
        assert!(
            u.query()
                .unwrap()
                .contains("model=Qwen%2FQwen3.6-35B-A3B-FP8")
        );
        // Without this the endpoint answers with ECDSA model attestations,
        // which attest no key that signs a receipt we verify.
        assert!(
            u.query().unwrap().contains("signing_algo=ed25519"),
            "the report request must select the ed25519 model attestations"
        );
    }

    /// The live per-model `provider_tee` keys, captured 2026-09-06, alongside
    /// the gateway key. All three differ.
    const MODEL_A: &str = "Qwen/Qwen3.6-35B-A3B-FP8";
    const MODEL_A_KEY: &str = "aba45f0b8f90869baab26db02e8b01354bb8f8730769c60650cb7a635da602d4";
    const MODEL_B: &str = "Qwen/Qwen3.8-27B";
    const MODEL_B_KEY: &str = "73cf225ab4f09154ad8b299d4ac89425c7f25468a42ba9a87d09fcd4e87b8bf5";

    fn full_report(nonce: &str) -> String {
        let entry = |model: &str, key: &str| {
            format!(
                r#"{{"model_name":"{model}","signing_address":"{key}","signing_algo":"ed25519","request_nonce":"{nonce}","report_data":"{key}{nonce}"}}"#
            )
        };
        format!(
            r#"{{"gateway_attestation":{{"signing_address":"{KEY}","signing_algo":"ed25519","request_nonce":"{nonce}","report_data":"{KEY}{nonce}"}},"model_attestations":[{},{}]}}"#,
            entry(MODEL_A, MODEL_A_KEY),
            entry(MODEL_B, MODEL_B_KEY)
        )
    }

    /// The two readers on one report: the gateway reader still returns the
    /// gateway key, and the model reader returns a *different* key. Both are
    /// correct; only the second one is what a receipt is checked against.
    #[test]
    fn the_model_key_and_the_gateway_key_are_different_keys() {
        let report = full_report(NONCE);
        assert_eq!(gateway_ed25519_key(&report, NONCE).unwrap(), KEY);
        assert_eq!(
            model_ed25519_keys(&report, NONCE, MODEL_A).unwrap(),
            vec![MODEL_A_KEY.to_string()]
        );
        assert_ne!(MODEL_A_KEY, KEY);
        assert!(!signer_is_attested(MODEL_A_KEY, KEY));
    }

    #[test]
    fn a_model_the_report_does_not_attest_is_refused() {
        assert_eq!(
            model_ed25519_keys(&full_report(NONCE), NONCE, "Qwen/Qwen3.9-Nonexistent").unwrap_err(),
            AttestationReportError::ModelNotAttested
        );
    }

    #[test]
    fn a_model_attestation_for_another_nonce_is_refused() {
        assert_eq!(
            model_ed25519_keys(&full_report(&"0".repeat(64)), NONCE, MODEL_A).unwrap_err(),
            AttestationReportError::NonceMismatch
        );
    }

    #[test]
    fn a_signer_is_matched_against_the_whole_attested_set() {
        let keys = model_ed25519_keys(&full_report(NONCE), NONCE, MODEL_B).unwrap();
        assert!(signer_is_attested_for_model(MODEL_B_KEY, &keys));
        assert!(!signer_is_attested_for_model(MODEL_A_KEY, &keys));
        assert!(!signer_is_attested_for_model(KEY, &keys));
    }

    #[test]
    fn the_signer_matches_the_attested_key_case_insensitively() {
        assert!(signer_is_attested(&KEY.to_ascii_uppercase(), KEY));
        assert!(signer_is_attested(KEY, KEY));
    }

    #[test]
    fn a_different_signer_does_not_match() {
        assert!(!signer_is_attested(
            "0x614bc66ff0407dbb70b9c7ca1f5e983e4a02c921",
            KEY
        ));
        assert!(
            !signer_is_attested(&KEY[..62], KEY),
            "a prefix is not a match"
        );
        assert!(!signer_is_attested("", KEY));
        assert!(
            !signer_is_attested(KEY, ""),
            "an empty attested key matches nothing"
        );
    }

    /// The gateway_attestation subtree of a report NEAR AI actually returned,
    /// reduced to the fields this parser reads. Every other fixture here is
    /// authored beside the code; this one pins that the real response nests
    /// these fields the way the parser expects and that report_data really is
    /// signing_address || request_nonce. The nonce is the one that fetch sent.
    #[test]
    fn the_live_report_parses_to_the_attested_gateway_key() {
        assert_eq!(gateway_ed25519_key(LIVE_REPORT, NONCE).unwrap(), KEY);
    }
}
