//! Bounty Judge — an autonomous verifier agent for the Triarchy mesh.
//!
//! Flow (the agentic loop the buildathon asks for):
//!   1. Read a bounty (task id, description, submitted proof) from CLI/env.
//!   2. Ask an LLM (via OpenRouter) to APPROVE or REJECT the proof, with reasoning.
//!   3. On APPROVE, autonomously submit a REAL `release_bounty` transaction to the
//!      deployed Casper escrow contract via the project signer, paying the hunter.
//!
//! There is no simulation: the verdict is a real model call and the payout is a
//! real signed Casper TransactionV1. No API key / failed call => it aborts (it
//! never invents a verdict or a tx hash).

use serde_json::json;
use std::process::Command;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Classify a model answer: Some(true)=APPROVE, Some(false)=REJECT, None=unclear.
fn classify_verdict(answer: &str) -> Option<bool> {
    let upper = answer.trim().to_uppercase();
    if upper.starts_with("APPROVE") {
        Some(true)
    } else if upper.starts_with("REJECT") {
        Some(false)
    } else {
        None
    }
}

/// Parse a little-endian u64 from the first 8 bytes of a CLValue hex string.
fn parse_u64_le_hex(hex: &str) -> Option<u64> {
    let bytes: Vec<u8> = (0..hex.len().min(16))
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    if bytes.len() == 8 {
        Some(u64::from_le_bytes(bytes.try_into().unwrap()))
    } else {
        None
    }
}

/// Pulls `--flag value` out of argv (simple, dependency-free).
fn arg(flags: &[&str]) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if flags.contains(&args[i].as_str()) && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

struct Verdict {
    approve: bool,
    reasoning: String,
}

/// Real OpenRouter chat completion. Returns the model's raw answer text.
async fn llm_judge(
    task_id: &str,
    description: &str,
    proof: &str,
) -> Result<Verdict, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY is required (no key, no verdict — we never fake one)")?;
    let model = env_or("OPENROUTER_MODEL", "anthropic/claude-opus-4.8-fast");

    let system = "You are a strict on-chain bounty verifier in an autonomous agent mesh. \
Given a task and a submitted proof, decide if the proof genuinely satisfies the task. \
Reply with a single line starting with exactly APPROVE or REJECT, then a colon and one \
sentence of reasoning. Be skeptical of placeholder or empty proofs.";
    let user = format!(
        "TASK_ID: {task_id}\nTASK: {description}\nSUBMITTED_PROOF: {proof}\n\nVerdict:"
    );

    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.0
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("OpenRouter HTTP {}: {}", resp.status(), resp.text().await?).into());
    }

    let v: serde_json::Value = resp.json().await?;
    let answer = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("malformed OpenRouter response")?
        .trim()
        .to_string();

    match classify_verdict(&answer) {
        Some(approve) => Ok(Verdict { approve, reasoning: answer }),
        None => Err(
            format!("model did not return a clear APPROVE/REJECT verdict: '{answer}'").into(),
        ),
    }
}

/// Submit a REAL `release_bounty` transaction via the project signer.
fn release_on_chain(task_id: &str, target_purse: &str) -> Result<String, String> {
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let chain = env_or("CASPER_CHAIN", "casper-test");
    let secret_key = env_or("CASPER_SECRET_KEY", "swarm/casper-client/key.pem");
    let signer = env_or("CASPER_SIGNER_BIN", "./swarm/casper-client/go-signer/casper-tx-signer");
    let package = env_or(
        "CASPER_CONTRACT_PACKAGE",
        "a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d",
    );

    let args = format!("task_id:string:{task_id},target_purse:uref:{target_purse}");
    let output = Command::new(&signer)
        .args([
            "--mode", "call-entrypoint",
            "--node", &node,
            "--chain", &chain,
            "--secret-key", &secret_key,
            "--algo", "ed25519",
            "--payment", "12000000000",
            "--contract-hash", &package,
            "--entrypoint", "release_bounty",
            "--args", &args,
        ])
        .output()
        .map_err(|e| format!("failed to launch signer '{signer}': {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let tx = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tx.len() != 64 || !tx.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("unexpected tx hash: '{tx}'"));
    }
    Ok(tx)
}

/// Second agent in the consensus: a risk gate that reads the hunter's REAL
/// on-chain reputation from the deployed oracle contract and enforces a payout
/// policy. Release happens only if BOTH the LLM judge and this gate approve —
/// cross-contract agent coordination (escrow release gated by oracle reputation).
async fn risk_gate(hunter_hex: &str, amount_motes: u128) -> Result<String, String> {
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let rep_uref = env_or(
        "CASPER_ORACLE_REPUTATION_UREF",
        "uref-87be256170aab9d412b2e3ee649943ab082b07b1fcf40816c7a017183e8b4567-007",
    );
    let min_rep: u64 = env_or("RISK_MIN_REPUTATION", "1").parse().unwrap_or(1);
    // Default max payout 1000 CSPR.
    let max_payout: u128 = env_or("RISK_MAX_PAYOUT_MOTES", "1000000000000")
        .parse()
        .unwrap_or(1_000_000_000_000);

    if amount_motes > max_payout {
        return Err(format!(
            "amount {amount_motes} motes exceeds policy cap {max_payout}"
        ));
    }

    let client = reqwest::Client::new();
    let srh: serde_json::Value = client
        .post(&node)
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"chain_get_state_root_hash","params":[]}))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let state_root = srh["result"]["state_root_hash"].as_str().ok_or("no state root")?;

    let item: serde_json::Value = client
        .post(&node)
        .json(&json!({
            "jsonrpc":"2.0","id":1,"method":"state_get_dictionary_item",
            "params":{"state_root_hash":state_root,
                "dictionary_identifier":{"URef":{"seed_uref":rep_uref,"dictionary_item_key":hunter_hex}}}
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;

    // CLValue bytes for a u64 are 8 little-endian bytes (hex).
    let reputation: u64 = item["result"]["stored_value"]["CLValue"]["bytes"]
        .as_str()
        .and_then(parse_u64_le_hex)
        .unwrap_or(0);

    if reputation < min_rep {
        return Err(format!(
            "hunter on-chain reputation {reputation} < required {min_rep}"
        ));
    }
    Ok(format!(
        "policy OK — hunter reputation {reputation} ≥ {min_rep}, amount within cap"
    ))
}

#[tokio::main]
async fn main() {
    let task_id = arg(&["--task-id", "-t"]).unwrap_or_else(|| "bounty-alpha-003".to_string());
    let description = arg(&["--description", "-d"])
        .unwrap_or_else(|| "Optimize the AST hypergraph for the Odra escrow modules.".to_string());
    let proof = arg(&["--proof", "-p"])
        .unwrap_or_else(|| "https://github.com/Triarchy-Labs/casper-agentic-mesh/pull/1".to_string());
    let target_purse = arg(&["--target-purse"]).unwrap_or_else(|| {
        env_or(
            "HUNTER_PURSE",
            "uref-37dde43070560294ae0799f91c53b1948d33e4649b0ec9ae79fbf3be37a0a27a-007",
        )
    });
    let hunter = arg(&["--hunter"]).unwrap_or_else(|| {
        env_or(
            "HUNTER_ACCOUNT",
            "334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867",
        )
    });
    let amount_motes: u128 = arg(&["--amount-motes"])
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000_000);

    println!("⚖️  [Bounty Judge] Evaluating {task_id} via LLM...");
    let verdict = match llm_judge(&task_id, &description, &proof).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ verdict aborted: {e}");
            std::process::exit(1);
        }
    };
    println!("🧠 [Bounty Judge] {}", verdict.reasoning);

    if !verdict.approve {
        println!("🛑 [Bounty Judge] REJECTED — no funds released.");
        return;
    }

    println!("✅ [Bounty Judge] APPROVED by LLM — consulting risk agent...");
    match risk_gate(&hunter, amount_motes).await {
        Ok(report) => println!("🛡️  [Risk Agent] {report}"),
        Err(e) => {
            println!("🛑 [Risk Agent] BLOCKED release: {e}");
            return;
        }
    }

    println!("🤝 [Consensus] Both agents approve — releasing escrow on-chain...");
    match release_on_chain(&task_id, &target_purse) {
        Ok(tx) => {
            println!("💸 [Bounty Judge] Paid hunter. Real tx: {tx}");
            println!("🔗 https://testnet.cspr.live/transaction/{tx}");
        }
        Err(e) => {
            eprintln!("❌ on-chain release failed: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_verdict, parse_u64_le_hex};

    #[test]
    fn verdict_classification() {
        assert_eq!(classify_verdict("APPROVE: solid proof"), Some(true));
        assert_eq!(classify_verdict("  reject: weak"), Some(false));
        assert_eq!(classify_verdict("maybe later"), None);
    }

    #[test]
    fn reputation_hex_parses_le_u64() {
        // 2u64 little-endian = 0200000000000000
        assert_eq!(parse_u64_le_hex("0200000000000000"), Some(2));
        assert_eq!(parse_u64_le_hex("0000000000000000"), Some(0));
        assert_eq!(parse_u64_le_hex("zz"), None);
    }
}
