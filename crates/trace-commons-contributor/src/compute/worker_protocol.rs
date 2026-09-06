//! Protocol model derived from reading Holonear revision ef4e6e2479e8395f7d972d3342bad97851f2104e.
//! Local vectors are not captured output or proof of upstream interoperability.
//! Field order and signature domains are part of the executable contract.

use ring::{
    rand::{SecureRandom, SystemRandom},
    signature::{self, KeyPair},
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const VERSION: u32 = 0;
pub const MAX_FRAME_BYTES: usize = 65_536;
pub const CREDENTIAL_ENV: &str = "HOLONEAR_WORKER_CREDENTIAL";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Status,
    Drain,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Connecting,
    Joining,
    Training,
    Serving,
    Idle,
    Draining,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Admission {
    Unknown,
    AwaitingAssignment,
    Assigned,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DrainOutcome {
    NotRequested,
    Pending,
    Acknowledged,
    TimedOut,
    Unavailable,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub state: State,
    pub admission: Admission,
    pub drain: DrainOutcome,
    pub free_mem_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub nonce: [u8; 32],
    pub command: Command,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedRequest {
    pub body: Request,
    pub signature: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub version: u32,
    pub instance: [u8; 32],
    pub nonce: [u8; 32],
    pub status: Status,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedResponse {
    pub body: Response,
    pub signature: Vec<u8>,
}

/// Ephemeral IPC capability, unrelated to payout or contributor credentials.
pub struct Credential {
    key: signature::Ed25519KeyPair,
}

impl Credential {
    pub fn from_seed(seed: &[u8; 32]) -> anyhow::Result<Self> {
        Ok(Self {
            key: signature::Ed25519KeyPair::from_seed_unchecked(seed)
                .map_err(|_| anyhow::anyhow!("worker-credential-invalid"))?,
        })
    }
    pub fn instance(&self) -> [u8; 32] {
        self.key
            .public_key()
            .as_ref()
            .try_into()
            .expect("Ed25519 public key length")
    }
    pub fn request(&self, command: Command) -> anyhow::Result<SignedRequest> {
        let mut nonce = [0; 32];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| anyhow::anyhow!("worker-random-failed"))?;
        self.request_with_nonce(command, nonce)
    }
    /// For deterministic conformance fixtures. Live calls use request().
    pub fn request_with_nonce(
        &self,
        command: Command,
        nonce: [u8; 32],
    ) -> anyhow::Result<SignedRequest> {
        let body = Request {
            version: VERSION,
            nonce,
            command,
        };
        let mut bytes = b"holonear-worker-request-v0\0".to_vec();
        bytes.extend(serde_json::to_vec(&body)?);
        Ok(SignedRequest {
            body,
            signature: self.key.sign(&bytes).as_ref().to_vec(),
        })
    }
    pub fn verify_response(&self, request: &SignedRequest, response: &SignedResponse) -> bool {
        if response.body.version != VERSION
            || response.body.instance != self.instance()
            || response.body.nonce != request.body.nonce
        {
            return false;
        }
        let Ok(body) = serde_json::to_vec(&response.body) else {
            return false;
        };
        let mut bytes = b"holonear-worker-response-v0\0".to_vec();
        bytes.extend(body);
        signature::UnparsedPublicKey::new(&signature::ED25519, self.instance())
            .verify(&bytes, &response.signature)
            .is_ok()
    }
    pub async fn exchange(&self, address: SocketAddr, command: Command) -> anyhow::Result<Status> {
        anyhow::ensure!(
            address.ip().is_loopback() && address.port() != 0,
            "worker-endpoint-invalid"
        );
        let deadline = match command {
            Command::Status => Duration::from_secs(2),
            Command::Drain => Duration::from_secs(10),
        };
        tokio::time::timeout(deadline, async {
            let request = self.request(command)?;
            let bytes = serde_json::to_vec(&request)?;
            let mut stream = tokio::net::TcpStream::connect(address).await?;
            stream.write_u32(bytes.len() as u32).await?;
            stream.write_all(&bytes).await?;
            let len = stream.read_u32().await? as usize;
            anyhow::ensure!((1..=MAX_FRAME_BYTES).contains(&len), "worker-frame-invalid");
            let mut bytes = vec![0; len];
            stream.read_exact(&mut bytes).await?;
            let response: SignedResponse = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("worker-response-invalid"))?;
            anyhow::ensure!(
                self.verify_response(&request, &response),
                "worker-authentication-failed"
            );
            Ok(response.body.status)
        })
        .await
        .map_err(|_| anyhow::anyhow!("worker-request-timeout"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    /// The exact committed bytes of the Orchard-generated fixture. Read as
    /// bytes, not as a string, because the digest test below is over these
    /// bytes; `.gitattributes` marks the file `-text` so a Windows checkout
    /// cannot rewrite its line endings out from under that digest.
    const ORCHARD_FIXTURE: &[u8] =
        include_bytes!("../../tests/fixtures/orchard_worker_ipc_v0.json");
    /// SHA-256 of `ORCHARD_FIXTURE`, published in `docs/compute-local-adapter.md`.
    /// The test recomputes it rather than trusting the doc.
    const ORCHARD_FIXTURE_SHA256: &str =
        "eb7a86d64173203a39e332f40cc795da5f3e631c0951526b523258b88c30b23b";
    const GENERATOR_SOURCE_SHA256: &str =
        "402e6d791a890030f863e2f246b5f82f71b12371143f8faf30deee78bd589d68";
    const IMPLEMENTATION_SOURCE_REVISION: &str = "7d6f70512fb6cd9faf936fc27ca367a5cd539de5";

    #[test]
    fn source_derived_vectors_pin_both_directions_and_reject_tampering() {
        // The original source-derived cases intentionally reuse the same [7; 32] seed.
        // Only the independently generated Orchard fixture exercises distinct seeds.
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/worker_ipc_v0.json")).unwrap();
        verify_compatibility_vectors(&fixture);
    }

    #[test]
    fn orchard_generated_vectors_pin_both_seeds_and_reject_tampering() {
        // Orchard 4d222766 is an UNMERGED local checkpoint; see compute-local-adapter.md.
        let fixture: serde_json::Value = serde_json::from_slice(ORCHARD_FIXTURE).unwrap();
        assert_eq!(fixture["generator_source_sha256"], GENERATOR_SOURCE_SHA256);
        assert_eq!(
            fixture["implementation_source_revision"],
            IMPLEMENTATION_SOURCE_REVISION
        );
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 2);
        assert_ne!(cases[0]["seed_hex"], cases[1]["seed_hex"]);
        for (case, seed_byte) in cases.iter().zip([9_u8, 17]) {
            assert_eq!(case["seed_hex"], hex::encode([seed_byte; 32]));
            let request: SignedRequest = serde_json::from_value(case["request"].clone()).unwrap();
            assert_eq!(request.body.nonce, [seed_byte + 1; 32]);
        }
        verify_compatibility_vectors(&fixture);
    }

    /// The provenance fields were previously prose in the fixture that no test
    /// read, so a fixture regenerated from a different source -- or one whose
    /// metadata was edited to say anything at all -- still passed every
    /// signature assertion. This is what a third party CAN check without access
    /// to the private generator repository: that the committed bytes are the
    /// bytes the documentation names, that the provenance fields are present
    /// and well formed, and that the fixture and the doc agree on them. It is
    /// NOT proof that Orchard produced the bytes; the seeds are public, so a
    /// local regeneration would still verify cryptographically. Only the digest
    /// pin below distinguishes the committed file from any other.
    #[test]
    fn orchard_fixture_provenance_fields_are_present_and_match_the_documentation() {
        let digest = hex::encode(Sha256::digest(ORCHARD_FIXTURE));
        assert_eq!(
            digest, ORCHARD_FIXTURE_SHA256,
            "orchard fixture bytes changed; regenerate per docs/compute-local-adapter.md \
             and update the published digest rather than editing this constant"
        );

        let fixture: serde_json::Value = serde_json::from_slice(ORCHARD_FIXTURE).unwrap();
        let generator = fixture["generator_source_sha256"].as_str().unwrap();
        let revision = fixture["implementation_source_revision"].as_str().unwrap();
        let notice = fixture["notice"].as_str().unwrap();
        assert_eq!(generator, GENERATOR_SOURCE_SHA256);
        assert_eq!(revision, IMPLEMENTATION_SOURCE_REVISION);
        assert!(is_lowercase_hex(generator, 64), "{generator}");
        assert!(is_lowercase_hex(revision, 40), "{revision}");
        assert!(
            notice.contains("Public fixed test seeds only"),
            "the fixture must keep stating that it is not a workload proof: {notice}"
        );

        // The doc is the only place the fixture's own digest is published, and
        // the doc is what an external reader reads. Drift between the two would
        // leave that reader checking a stale digest against new bytes.
        const DOC: &str = include_str!("../../../../docs/compute-local-adapter.md");
        for claim in [
            ORCHARD_FIXTURE_SHA256,
            GENERATOR_SOURCE_SHA256,
            IMPLEMENTATION_SOURCE_REVISION,
        ] {
            assert!(
                DOC.contains(claim),
                "docs/compute-local-adapter.md lost {claim}"
            );
        }
        assert!(
            DOC.contains("not publicly reproducible"),
            "the doc must keep saying the generating repository is private"
        );
    }

    fn is_lowercase_hex(value: &str, len: usize) -> bool {
        value.len() == len
            && value
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }

    fn verify_compatibility_vectors(fixture: &serde_json::Value) {
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 2);
        for case in cases {
            let seed: [u8; 32] = hex::decode(case["seed_hex"].as_str().unwrap())
                .unwrap()
                .try_into()
                .unwrap();
            let credential = Credential::from_seed(&seed).unwrap();
            let request: SignedRequest = serde_json::from_value(case["request"].clone()).unwrap();
            let response: SignedResponse =
                serde_json::from_value(case["response"].clone()).unwrap();
            let actual = credential
                .request_with_nonce(request.body.command, request.body.nonce)
                .unwrap();
            assert_eq!(actual.signature, request.signature);
            assert_eq!(
                serde_json::to_string(&actual.body).unwrap(),
                case["request_body_json"].as_str().unwrap()
            );
            assert_eq!(
                serde_json::to_string(&response.body).unwrap(),
                case["response_body_json"].as_str().unwrap()
            );
            assert!(credential.verify_response(&actual, &response));
            let mut wrong = response.clone();
            wrong.body.version += 1;
            assert!(!credential.verify_response(&actual, &wrong));
            let mut wrong = response.clone();
            wrong.body.nonce[0] ^= 1;
            assert!(!credential.verify_response(&actual, &wrong));
            let mut wrong = response.clone();
            wrong.body.instance[0] ^= 1;
            assert!(!credential.verify_response(&actual, &wrong));
            let mut wrong = response.clone();
            wrong.body.status.free_mem_bytes += 1;
            assert!(!credential.verify_response(&actual, &wrong));
            let mut wrong = response.clone();
            wrong.signature = request.signature;
            assert!(!credential.verify_response(&actual, &wrong));
        }
    }

    #[tokio::test]
    async fn rejects_invalid_frames_and_non_loopback_addresses() {
        for frame in [0, MAX_FRAME_BYTES as u32 + 1, 2] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let peer = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let len = stream.read_u32().await.unwrap();
                let mut bytes = vec![0; len as usize];
                stream.read_exact(&mut bytes).await.unwrap();
                stream.write_u32(frame).await.unwrap();
                if frame == 2 {
                    stream.write_all(b"{}").await.unwrap();
                }
            });
            let error = Credential::from_seed(&[3; 32])
                .unwrap()
                .exchange(address, Command::Status)
                .await
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                if frame == 2 {
                    "worker-response-invalid"
                } else {
                    "worker-frame-invalid"
                }
            );
            peer.await.unwrap();
        }
        assert!(
            Credential::from_seed(&[3; 32])
                .unwrap()
                .exchange("192.0.2.1:8".parse().unwrap(), Command::Status)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn actual_loopback_disconnect_is_an_io_error_after_accept() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let peer = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let len = stream.read_u32().await.unwrap();
            let mut bytes = vec![0; len as usize];
            stream.read_exact(&mut bytes).await.unwrap();
            // Close after a real request, without returning a frame.
        });
        let error = Credential::from_seed(&[3; 32])
            .unwrap()
            .exchange(address, Command::Status)
            .await
            .unwrap_err();
        peer.await.unwrap();
        assert!(error.downcast_ref::<std::io::Error>().is_some());
    }
}
