use core_ipc::IpcBridge;
use std::time::Duration;
use crate::{config, engine};

pub async fn run_sniper_loop() {
    let ipc = IpcBridge::new();
    let mut last_timestamp = 0;

    let rpc_str = std::env::var("CASPER_RPC_URL").unwrap_or_else(|_| "https://rpc.testnet.casperlabs.io/rpc".to_string());
    println!("[Sniper Agent] Initialized and connected to Casper RPC: {}", rpc_str);

    loop {
        if let Some(state) = ipc.read_state() {
            if state.timestamp > last_timestamp {
                last_timestamp = state.timestamp;
                
                if let Some(target) = state.liquidation_target {
                    println!("\n[Sniper Agent] ⚡ COGNITIVE ARBITRAGE OPPORTUNITY DETECTED ⚡");
                    println!("[Sniper Agent] Target: {}", target);
                    
                    let sentiment = state.global_sentiment_modifier;
                    let leverage_multiplier = engine::calculate_leverage_multiplier(sentiment);
                    
                    // Vector Gamma: Integration with Go x402 Facilitator
                    // We interact with the local x402 Go sidecar to negotiate payment
                    let x402_payload = serde_json::json!({
                        "task_id": target,
                        "bid_amount": 500 * leverage_multiplier as i64,
                        "currency": "CSPR"
                    });
                    
                    println!("[x402 Facilitator] Executing M2M Micropayment Negotiation...");
                    
                    // Production-ready HTTP call to Go Facilitator
                    let client = reqwest::Client::new();
                    match client.post("http://localhost:8080/x402/negotiate")
                        .json(&x402_payload)
                        .send()
                        .await 
                    {
                        Ok(res) => {
                            if res.status().is_success() {
                                println!("[x402 Facilitator] Payment successful. Target unlocked.");
                                match engine::execute_flash_loan_tx(&target, leverage_multiplier).await {
                                    Ok(hash) => println!("[Sniper Agent] 💥 Arbitrage successful! Hash: {}", hash),
                                    Err(e) => println!("[Sniper Agent] ❌ Arbitrage failed: {}", e),
                                }
                            } else {
                                println!("[x402 Facilitator] Payment rejected. Status: {}", res.status());
                            }
                        },
                        Err(e) => {
                            println!("[x402 Facilitator] ⚠️ ERROR: Facilitator unreachable: {}", e);
                            println!("[Sniper Agent] ❌ Arbitrage aborted. No payment, no payload.");
                        }
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(config::IPC_POLL_INTERVAL_MS)).await;
    }
}
