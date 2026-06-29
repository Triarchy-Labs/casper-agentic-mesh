//! The Tower — overseer meta-agent for the Triarchy mesh.
//!
//! The brain at the top of the tower. It performs a single on-demand SCAN (no
//! autonomous background loop): reads the on-chain world from the deployed
//! contracts, builds one world-model, applies the Antifragile-Mesh
//! Proof-of-Liveness rule, and prints the dispatch decisions it *would* take.
//!
//! By default it is READ-ONLY and dry-run — nothing is spent, nothing is moved.
//! It is meant to be triggered by a button / a human, exactly once per click.

use serde_json::json;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

struct Rpc {
    node: String,
    client: reqwest::Client,
}

impl Rpc {
    fn new() -> Self {
        Self {
            node: env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc"),
            client: reqwest::Client::new(),
        }
    }

    async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let r = self
            .client
            .post(&self.node)
            .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("RPC HTTP {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    async fn state_root(&self) -> Result<String, String> {
        let d = self.call("chain_get_state_root_hash", json!([])).await?;
        d["result"]["state_root_hash"].as_str().map(|s| s.to_string()).ok_or("no state root".into())
    }

    async fn dict_string(&self, srh: &str, uref: &str, key: &str) -> Option<String> {
        let d = self
            .call(
                "state_get_dictionary_item",
                json!({"state_root_hash":srh,"dictionary_identifier":{"URef":{"seed_uref":uref,"dictionary_item_key":key}}}),
            )
            .await
            .ok()?;
        let hex = d["result"]["stored_value"]["CLValue"]["bytes"].as_str()?;
        decode_cl_string(hex)
    }

    async fn dict_u64(&self, srh: &str, uref: &str, key: &str) -> Option<u64> {
        let d = self
            .call(
                "state_get_dictionary_item",
                json!({"state_root_hash":srh,"dictionary_identifier":{"URef":{"seed_uref":uref,"dictionary_item_key":key}}}),
            )
            .await
            .ok()?;
        let hex = d["result"]["stored_value"]["CLValue"]["bytes"].as_str()?;
        decode_cl_u64(hex)
    }
}

fn decode_cl_string(hex: &str) -> Option<String> {
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect();
    if bytes.len() < 4 {
        return None;
    }
    let len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    String::from_utf8(bytes.get(4..4 + len)?.to_vec()).ok()
}

fn decode_cl_u64(hex: &str) -> Option<u64> {
    let b: Vec<u8> = (0..hex.len().min(16))
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect();
    if b.len() == 8 {
        Some(u64::from_le_bytes(b.try_into().ok()?))
    } else {
        None
    }
}

fn reading_value(record: &str) -> Option<u64> {
    record.split(';').find_map(|p| p.strip_prefix("value=")).and_then(|v| v.parse().ok())
}

#[tokio::main]
async fn main() {
    let readings_uref = env_or("CASPER_ORACLE_READINGS_UREF", "uref-0635112c4d2ae2dd60d333bbcc5c9ec1858361101435a1e2d7a2bd7fd2242105-007");
    let reputation_uref = env_or("CASPER_ORACLE_REPUTATION_UREF", "uref-87be256170aab9d412b2e3ee649943ab082b07b1fcf40816c7a017183e8b4567-007");
    let agent = env_or("CASPER_AGENT_ACCOUNT", "334f6577fd29b3c939d35f8c3c386b5eaebbb1435f088487485980ed2acb6867");
    // An agent is considered alive if its heartbeat is younger than this many seconds.
    let liveness_window: u64 = env_or("TOWER_LIVENESS_WINDOW_SECS", "3600").parse().unwrap_or(3600);

    let rpc = Rpc::new();

    println!("\n🗼 ════ THE TOWER · overseer scan ════");
    let srh = match rpc.state_root().await {
        Ok(s) => s,
        Err(e) => {
            println!("🛑 SERVICE DEGRADED: cannot reach the Casper node ({e}). Functions are frozen — we are working on it.");
            std::process::exit(2);
        }
    };
    println!("   ledger state root: {}\n", &srh[..16]);

    // ── World model ────────────────────────────────────────────────
    let price = rpc.dict_string(&srh, &readings_uref, "CSPR-USD").await;
    let reputation = rpc.dict_u64(&srh, &reputation_uref, &agent).await;
    let heartbeat = rpc.dict_string(&srh, &readings_uref, &format!("HEARTBEAT-{agent}")).await;

    println!("📡 WORLD MODEL");
    match &price {
        Some(r) => {
            let micro = reading_value(r).unwrap_or(0);
            println!("   • Oracle CSPR-USD : ${:.6} (on-chain)", micro as f64 / 1e6);
        }
        None => println!("   • Oracle CSPR-USD : (no reading yet)"),
    }
    println!("   • Agent reputation: {}", reputation.unwrap_or(0));

    // ── Antifragile Mesh: Proof-of-Liveness ───────────────────────
    let now = now_secs();
    let (alive, liveness_line) = match heartbeat.as_ref().and_then(|h| reading_value(h)) {
        Some(ts) => {
            let age = now.saturating_sub(ts);
            if age <= liveness_window {
                (true, format!("ALIVE · last heartbeat {age}s ago (window {liveness_window}s)"))
            } else {
                (false, format!("STALE · last heartbeat {age}s ago > window {liveness_window}s"))
            }
        }
        None => (false, "NO HEARTBEAT on record".to_string()),
    };
    println!("   • Agent liveness  : {liveness_line}");

    // ── Dispatch decisions (dry-run by default) ───────────────────
    println!("\n🧠 DISPATCH DECISIONS (dry-run)");
    let mut actions = 0;
    if price.is_none() {
        actions += 1;
        println!("   {actions}. Oracle has no CSPR-USD reading → dispatch `rwa-oracle` to seed the feed.");
    }
    if !alive {
        actions += 1;
        println!("   {actions}. 🚨 Agent {} is not alive → AUTONOMOUS SUCCESSION:", &agent[..12]);
        println!("        nominate the highest-reputation LIVE agent as successor and have the");
        println!("        Tribunal ratify the handover; open escrows are rescued, never frozen.");
    }
    if alive && reputation.unwrap_or(0) == 0 {
        actions += 1;
        println!("   {actions}. Agent alive but reputation 0 → require Tribunal sign-off before high-value payouts.");
    }
    if actions == 0 {
        println!("   ✓ Mesh healthy: oracle fresh, agent alive, reputation sufficient. No action needed.");
    }

    println!("\n🗼 Scan complete. (read-only; no funds moved — dispatch shown as recommendations.)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_cl_string_and_u64() {
        // "hi" as CLValue String = 02000000 6869
        assert_eq!(decode_cl_string("026869").is_none(), true); // too short length prefix guards
        assert_eq!(decode_cl_string("020000006869").as_deref(), Some("hi"));
        assert_eq!(decode_cl_u64("0200000000000000"), Some(2));
    }

    #[test]
    fn parses_reading_value() {
        assert_eq!(reading_value("value=1825;by=abc;seq=3"), Some(1825));
        assert_eq!(reading_value("nope"), None);
    }
}
