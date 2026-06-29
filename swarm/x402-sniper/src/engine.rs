use std::process::Command;

pub fn calculate_leverage_multiplier(sentiment: f64) -> f64 {
    1.0 + sentiment
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Sanitize a value for the `key:type:value` arg parser of casper-tx-signer
/// (commas separate args, colons separate fields).
fn sanitize(v: &str) -> String {
    v.replace([',', ':'], "-")
}

/// Execute a REAL on-chain action against the deployed Casper escrow contract.
///
/// This shells out to the project's signer (`casper-tx-signer`), which builds,
/// signs and submits a `TransactionV1` to the Casper testnet, and returns the
/// real 64-char transaction hash. There is no simulation: if the signer or the
/// node fails, this returns an error — the agent never fabricates a hash.
pub async fn execute_flash_loan_tx(
    target: &str,
    sentiment: f64,
) -> Result<String, Box<dyn std::error::Error>> {
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let chain = env_or("CASPER_CHAIN", "casper-test");
    let secret_key = env_or("CASPER_SECRET_KEY", "swarm/casper-client/key.pem");
    let signer = env_or(
        "CASPER_SIGNER_BIN",
        "./swarm/casper-client/go-signer/casper-tx-signer",
    );
    let package = env_or(
        "CASPER_CONTRACT_PACKAGE",
        "a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d",
    );
    let agent_pubkey = env_or(
        "CASPER_AGENT_PUBKEY",
        "013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35",
    );

    let sentiment_score = (sentiment * 100.0) as u64;
    let metadata = sanitize(&format!("arb-target-{target}-sentiment-{sentiment_score}"));
    let args = format!("public_key:string:{agent_pubkey},metadata_uri:string:{metadata}");

    println!("[Sniper Agent] Submitting real on-chain call to Casper escrow {package} via {node}...");

    let output = Command::new(&signer)
        .args([
            "--mode", "call-entrypoint",
            "--node", &node,
            "--chain", &chain,
            "--secret-key", &secret_key,
            "--algo", "ed25519",
            "--payment", "5000000000",
            "--contract-hash", &package,
            "--entrypoint", "register_agent",
            "--args", &args,
        ])
        .output()
        .map_err(|e| format!("failed to launch signer '{signer}': {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "signer rejected transaction: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    let tx_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tx_hash.len() != 64 || !tx_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("signer returned an unexpected tx hash: '{tx_hash}'").into());
    }

    println!("[Sniper Agent] Broadcasted on-chain. Real tx hash: {tx_hash}");
    Ok(tx_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leverage_scales_with_sentiment() {
        assert_eq!(calculate_leverage_multiplier(0.0), 1.0);
        assert!(calculate_leverage_multiplier(0.5) > calculate_leverage_multiplier(0.1));
    }

    #[test]
    fn sanitize_strips_arg_separators() {
        // commas and colons would break the signer's key:type:value parser
        assert_eq!(sanitize("a:b,c"), "a-b-c");
        assert_eq!(sanitize("mcp_pool_0x000a"), "mcp_pool_0x000a");
    }
}
