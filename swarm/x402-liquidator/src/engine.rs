pub struct LiquidationTarget {
    pub address: String,
    pub health_factor: f64,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Scan a target account on Casper for a real, on-chain health factor.
///
/// The "health factor" is derived from the account's live main-purse balance
/// against a configured maintenance threshold — read directly from the Casper
/// node via JSON-RPC. There is no simulated balance: if the RPC call fails, no
/// target is produced (the daemon never invents one).
pub async fn scan_for_targets() -> Option<LiquidationTarget> {
    let rpc = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let target = env_or(
        "TARGET_ACCOUNT_HASH",
        "334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867",
    );
    // Maintenance threshold in motes (default 100 CSPR). Below 1.0 == underwater.
    let threshold_motes: f64 = env_or("LIQUIDATION_THRESHOLD_MOTES", "100000000000")
        .parse()
        .unwrap_or(100_000_000_000.0);

    let account_hash = format!("account-hash-{}", target.trim_start_matches("account-hash-"));
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "query_balance",
        "params": { "purse_identifier": { "main_purse_under_account_hash": account_hash } }
    });

    let client = reqwest::Client::new();
    let resp = client.post(&rpc).json(&body).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;

    let balance_str = json
        .get("result")
        .and_then(|r| r.get("balance"))
        .and_then(|b| b.as_str())?;
    let balance: f64 = balance_str.parse().ok()?;

    let health_factor = balance / threshold_motes;
    println!(
        "[Liquidator Daemon] On-chain scan via Casper RPC. Balance: {} motes | HF: {:.4}",
        balance_str, health_factor
    );

    if health_factor < 1.0 {
        Some(LiquidationTarget {
            address: account_hash,
            health_factor,
        })
    } else {
        None
    }
}
