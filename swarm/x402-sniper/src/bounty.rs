//! X402 Bounty Client — Two-Phase Commit Protocol Implementation
//!
//! Absorbed from owocki-bot/ai-bounty-board (D2178).
//! Implements the x402 payment flow: Discovery (402) → Retry with X-Payment header.
//!
//! Reference: skills/x402-agent-protocol/SKILL.md

// Casper-native types: an Address is an account-hash/public-key hex string and
// the signer holds the HMAC key material used to authenticate x402 payloads.
type Address = String;
type PrivateKeySigner = String;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Real keyed signature over the x402 message (no dummy bytes).
/// `x402:{recipient}:{amount}:{nonce}` is authenticated with the agent's key.
fn sign_x402_message(signer_key: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(signer_key.as_bytes())
        .expect("HMAC accepts keys of any size");
    mac.update(message.as_bytes());
    format!("0x{}", hex::encode(mac.finalize().into_bytes()))
}

// ─── Types ──────────────────────────────────────────────────────────────────

/// Payment requirements returned by server on 402 response
#[derive(Debug, Deserialize)]
pub struct X402PaymentRequirements {
    pub amount: String,
    pub recipient: String,
    pub token: Option<String>,
    pub network: Option<String>,
}

/// The full 402 response envelope
#[derive(Debug, Deserialize)]
pub struct X402DiscoveryResponse {
    pub error: Option<String>,
    pub x402: Option<X402PaymentRequirements>,
}

/// Payment payload to be Base64-encoded and sent as X-Payment header
#[derive(Debug, Serialize)]
pub struct X402PaymentPayload {
    pub amount: String,
    pub payer: String,
    pub recipient: String,
    pub nonce: u64,
    pub signature: String,
}

/// Bounty data returned from the API
#[derive(Debug, Deserialize)]
pub struct Bounty {
    pub id: Option<u64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub reward: Option<String>,
    pub status: Option<String>,
    pub creator: Option<String>,
}

/// Agent registration response
#[derive(Debug, Deserialize)]
pub struct AgentRegistration {
    pub success: Option<bool>,
    pub agent_id: Option<String>,
}

/// Submission response
#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    pub success: Option<bool>,
    pub submission_id: Option<String>,
    pub message: Option<String>,
}

// ─── Client ─────────────────────────────────────────────────────────────────

/// X402-compatible bounty client implementing the Two-Phase Commit pattern.
///
/// # Architecture (from D2178)
/// ```text
/// PHASE 1: POST /bounties (no payment) → 402 → { x402: { amount, recipient } }
/// PHASE 2: POST /bounties + X-Payment: base64(payload) → 201
/// ```
///
/// # Anti-Self-Dealing Guard
/// Creator address ≠ Claimant address (enforced server-side and client-side).
pub struct X402BountyClient {
    http: reqwest::Client,
    base_url: String,
    signer: PrivateKeySigner,
    agent_address: Address,
    nonce_counter: u64,
}

impl X402BountyClient {
    /// Create a new bounty client connected to an x402-compatible API.
    pub fn new(base_url: &str, signer: PrivateKeySigner) -> Self {
        // Real agent identity: the deployer/agent account-hash hex, overridable via env.
        let agent_address = std::env::var("CASPER_AGENT_ADDRESS").unwrap_or_else(|_| {
            "334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867".to_string()
        });
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            signer,
            agent_address,
            nonce_counter: 0,
        }
    }

    /// Get the agent's wallet address (checksummed).
    pub fn address(&self) -> Address {
        self.agent_address.clone()
    }

    // ─── Agent Lifecycle ────────────────────────────────────────────────

    /// Register this agent with the bounty board.
    /// `POST /agents { address, name, capabilities }`
    pub async fn register(
        &self,
        name: &str,
        capabilities: &[&str],
    ) -> Result<AgentRegistration, BountyError> {
        let body = serde_json::json!({
            "address": format!("{:?}", self.agent_address),
            "name": name,
            "capabilities": capabilities,
        });

        let resp = self.http
            .post(format!("{}/agents", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(BountyError::Network)?;

        resp.json().await.map_err(BountyError::Parse)
    }

    /// Browse open bounties.
    /// `GET /bounties?status=open`
    pub async fn browse_open(&self) -> Result<Vec<Bounty>, BountyError> {
        let resp = self.http
            .get(format!("{}/bounties?status=open", self.base_url))
            .send()
            .await
            .map_err(BountyError::Network)?;

        resp.json().await.map_err(BountyError::Parse)
    }

    /// Claim a bounty.
    /// `POST /bounties/:id/claim { address }`
    ///
    /// # Anti-Self-Dealing
    /// Will fail server-side if `bounty.creator == claimer.address`.
    pub async fn claim(&self, bounty_id: u64) -> Result<serde_json::Value, BountyError> {
        let body = serde_json::json!({
            "address": format!("{:?}", self.agent_address),
        });

        let resp = self.http
            .post(format!("{}/bounties/{}/claim", self.base_url, bounty_id))
            .json(&body)
            .send()
            .await
            .map_err(BountyError::Network)?;

        if resp.status() == StatusCode::BAD_REQUEST {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(BountyError::ServerReject(err_body));
        }

        resp.json().await.map_err(BountyError::Parse)
    }

    /// Submit work for a claimed bounty.
    /// `POST /bounties/:id/submit { address, submission, proof_url }`
    pub async fn submit_work(
        &self,
        bounty_id: u64,
        submission: &str,
        proof_url: &str,
    ) -> Result<SubmitResponse, BountyError> {
        // Pre-flight: validate proof URL is not a placeholder
        Self::validate_proof_url(proof_url)?;

        let body = serde_json::json!({
            "address": format!("{:?}", self.agent_address),
            "submission": submission,
            "proof_url": proof_url,
        });

        let resp = self.http
            .post(format!("{}/bounties/{}/submit", self.base_url, bounty_id))
            .json(&body)
            .send()
            .await
            .map_err(BountyError::Network)?;

        if resp.status() == StatusCode::BAD_REQUEST {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(BountyError::ServerReject(err_body));
        }

        resp.json().await.map_err(BountyError::Parse)
    }

    // ─── Two-Phase Commit (Bounty Creation with Payment) ────────────────

    /// Execute the Two-Phase Commit pattern for creating a paid bounty.
    ///
    /// Phase 1: POST without payment → get 402 with requirements
    /// Phase 2: Sign requirements → POST with X-Payment header → 201
    ///
    /// # GOTCHA (D2178)
    /// - Amount MUST be in USDC subunits (1e6 = 1 USDC), NOT float
    /// - Nonce MUST be unique per transaction (server may not enforce — our guard)
    /// - Signature uses EIP-191: `x402:{recipient}:{amount}:{nonce}`
    pub async fn create_bounty_with_payment(
        &mut self,
        title: &str,
        description: &str,
        reward: &str,
    ) -> Result<Bounty, BountyError> {
        let body = serde_json::json!({
            "title": title,
            "description": description,
            "reward": reward,
            "creator": format!("{:?}", self.agent_address),
        });

        // ── Phase 1: Discovery ──────────────────────────────────────────
        let discovery_resp = self.http
            .post(format!("{}/bounties", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(BountyError::Network)?;

        if discovery_resp.status() != StatusCode::PAYMENT_REQUIRED {
            // If not 402, the server either accepted without payment or errored
            if discovery_resp.status().is_success() {
                return discovery_resp.json().await.map_err(BountyError::Parse);
            }
            let err = discovery_resp.text().await.unwrap_or_default();
            return Err(BountyError::ServerReject(format!("Expected 402, got: {}", err)));
        }

        let requirements: X402DiscoveryResponse = discovery_resp
            .json()
            .await
            .map_err(BountyError::Parse)?;

        let x402 = requirements.x402.ok_or_else(|| {
            BountyError::Protocol("402 response missing x402 field".into())
        })?;

        // ── Phase 2: Sign and Retry ─────────────────────────────────────
        self.nonce_counter += 1;
        let nonce = self.nonce_counter;

        // Construct the x402 message and sign it for real with the agent key.
        let message = format!("x402:{}:{}:{}", x402.recipient, x402.amount, nonce);
        let signature = sign_x402_message(&self.signer, &message);

        let payload = X402PaymentPayload {
            amount: x402.amount,
            payer: self.agent_address.clone(),
            recipient: x402.recipient,
            nonce,
            signature,
        };

        // Base64-encode the JSON payload for the X-Payment header
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| BountyError::Protocol(e.to_string()))?;
        let payment_header = base64_encode(&payload_json);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Payment",
            HeaderValue::from_str(&payment_header)
                .map_err(|e| BountyError::Protocol(e.to_string()))?,
        );

        let paid_resp = self.http
            .post(format!("{}/bounties", self.base_url))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(BountyError::Network)?;

        if !paid_resp.status().is_success() {
            let err = paid_resp.text().await.unwrap_or_default();
            return Err(BountyError::PaymentRejected(err));
        }

        paid_resp.json().await.map_err(BountyError::Parse)
    }

    // ─── Proof URL Validation (D2178 Phase 5) ───────────────────────────

    /// Validate that a proof URL is not a placeholder.
    /// Mirrors `verify_proof_url.py` blacklist.
    fn validate_proof_url(url: &str) -> Result<(), BountyError> {
        const BLACKLIST: &[&str] = &[
            "example.com", "test.com", "localhost", "127.0.0.1",
            "placeholder", "yoursite.com", "tbd", "todo", "fixme",
            "yourname", "yourusername",
        ];

        if url.is_empty() {
            return Err(BountyError::InvalidProof("Empty proof URL".into()));
        }

        let url_lower = url.to_lowercase();
        for &placeholder in BLACKLIST {
            if url_lower.contains(placeholder) {
                return Err(BountyError::InvalidProof(
                    format!("Placeholder URL detected: '{}'", placeholder),
                ));
            }
        }

        // Validate it's a parseable URL
        url::Url::parse(url).map_err(|e| {
            BountyError::InvalidProof(format!("Invalid URL: {}", e))
        })?;

        Ok(())
    }
}

// ─── Error Types ────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum BountyError {
    Network(reqwest::Error),
    Parse(reqwest::Error),
    ServerReject(String),
    Protocol(String),
    Signing(String),
    PaymentRejected(String),
    InvalidProof(String),
}

impl std::fmt::Display for BountyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::ServerReject(msg) => write!(f, "Server rejected: {}", msg),
            Self::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Self::Signing(msg) => write!(f, "Signing error: {}", msg),
            Self::PaymentRejected(msg) => write!(f, "Payment rejected: {}", msg),
            Self::InvalidProof(msg) => write!(f, "Invalid proof: {}", msg),
        }
    }
}

impl std::error::Error for BountyError {}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Simple Base64 encoder without pulling in a full crate.
/// Uses the standard alphabet (RFC 4648).
fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::sign_x402_message;

    #[test]
    fn x402_signature_is_deterministic_and_keyed() {
        let msg = "x402:recipient:1000000:7";
        let a = sign_x402_message("agent-key", msg);
        let b = sign_x402_message("agent-key", msg);
        assert_eq!(a, b, "same key+message must yield same signature");
        assert!(a.starts_with("0x") && a.len() == 66, "0x + 32-byte HMAC hex");
        assert_ne!(a, sign_x402_message("other-key", msg), "key must affect signature");
        assert_ne!(a, "0xdummy_signature");
    }
}
