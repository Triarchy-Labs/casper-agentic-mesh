//! RWA Oracle Agent — bridges real off-chain asset data onto Casper.
//!
//! Flow:
//!   1. Fetch a REAL price for an asset from a public feed (CoinGecko by default).
//!   2. Convert to integer micro-USD and submit a real `post_reading` transaction
//!      to the deployed oracle contract via the project signer.
//!   3. The contract records the reading, appends an event, and accrues the
//!      agent's on-chain reputation.
//!
//! No simulation: if the feed or the signer fails, the agent aborts — it never
//! posts a fabricated value or a fake tx hash.

use std::process::Command;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Convert a USD price to integer micro-USD (1e6) for on-chain storage.
fn to_micro_usd(price: f64) -> u64 {
    (price * 1_000_000.0).round() as u64
}

/// Fetch a real USD price for a CoinGecko asset id.
async fn fetch_price_usd(coingecko_id: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={coingecko_id}&vs_currencies=usd"
    );
    let client = reqwest::Client::builder()
        .user_agent("triarchy-rwa-oracle/0.1")
        .build()?;
    let v: serde_json::Value = client.get(&url).send().await?.json().await?;
    let price = v[coingecko_id]["usd"]
        .as_f64()
        .ok_or("price not found in feed response")?;
    if price <= 0.0 {
        return Err("feed returned a non-positive price".into());
    }
    Ok(price)
}

/// Submit a real `post_reading` transaction via the project signer.
fn post_reading_on_chain(asset: &str, micro_usd: u64) -> Result<String, String> {
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let chain = env_or("CASPER_CHAIN", "casper-test");
    let secret_key = env_or("CASPER_SECRET_KEY", "swarm/casper-client/key.pem");
    let signer = env_or("CASPER_SIGNER_BIN", "./swarm/casper-client/go-signer/casper-tx-signer");
    let package = env_or(
        "CASPER_ORACLE_PACKAGE",
        "16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2",
    );

    let args = format!("asset:string:{asset},value:u512:{micro_usd}");
    let output = Command::new(&signer)
        .args([
            "--mode", "call-entrypoint",
            "--node", &node,
            "--chain", &chain,
            "--secret-key", &secret_key,
            "--algo", "ed25519",
            "--payment", "6000000000",
            "--contract-hash", &package,
            "--entrypoint", "post_reading",
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

#[tokio::main]
async fn main() {
    // Defaults map the on-chain asset symbol to a CoinGecko id.
    let asset = env_or("ORACLE_ASSET", "CSPR-USD");
    let cg_id = env_or("ORACLE_COINGECKO_ID", "casper-network");

    println!("🛰️  [RWA Oracle] Fetching real price for {asset} ({cg_id})...");
    let price = match fetch_price_usd(&cg_id).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ feed error: {e}");
            std::process::exit(1);
        }
    };
    let micro_usd = to_micro_usd(price);
    println!("📈 [RWA Oracle] {asset} = ${price} ({micro_usd} micro-USD)");

    println!("⛓️  [RWA Oracle] Posting reading on-chain...");
    match post_reading_on_chain(&asset, micro_usd) {
        Ok(tx) => {
            println!("✅ [RWA Oracle] Reading anchored on Casper. Reputation +1.");
            println!("🔗 https://testnet.cspr.live/transaction/{tx}");
        }
        Err(e) => {
            eprintln!("❌ on-chain post failed: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::to_micro_usd;

    #[test]
    fn converts_usd_to_micro_usd() {
        assert_eq!(to_micro_usd(0.00182548), 1825);
        assert_eq!(to_micro_usd(1.0), 1_000_000);
        assert_eq!(to_micro_usd(0.0), 0);
    }
}
