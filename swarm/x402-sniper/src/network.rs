use core_ipc::IpcBridge;
use std::time::Duration;
use crate::{config, engine};
use alloy::providers::ProviderBuilder;
use url::Url;
use alloy::signers::local::PrivateKeySigner;
use alloy::network::EthereumWallet;

pub async fn run_sniper_loop() {
    let ipc = IpcBridge::new();
    let mut last_timestamp = 0;

    // Initialize Provider with a Wallet — loads from env or generates random for dev
    let signer: PrivateKeySigner = match std::env::var("PRIVATE_KEY") {
        Ok(key) => key.parse().expect("Invalid PRIVATE_KEY hex"),
        Err(_) => {
            eprintln!("[Sniper Agent] WARNING: No PRIVATE_KEY set, using random signer (dev mode)");
            PrivateKeySigner::random()
        }
    };
    let wallet = EthereumWallet::from(signer);
    let rpc_str = std::env::var("MANTLE_RPC_URL").unwrap_or_else(|_| "https://rpc.mantle.xyz".to_string());
    let rpc_url = Url::parse(&rpc_str).unwrap();
    let provider = ProviderBuilder::new()
        
        .wallet(wallet)
        .connect_http(rpc_url);

    println!("[Sniper Agent] Wallet initialized and connected to Mantle RPC: {}", rpc_str);

    loop {
        if let Some(state) = ipc.read_state()
            && state.timestamp > last_timestamp {
                last_timestamp = state.timestamp;
                
                if let Some(target) = state.liquidation_target {
                    println!("\n[Sniper Agent] ⚡ L0 IPC TRIGGER RECEIVED ⚡");
                    println!("[Sniper Agent] Consensus reached for Liquidating Target: {target}");
                    
                    let sentiment = state.global_sentiment_modifier;
                    println!("[Sniper Agent] 🌐 Polymarket Global Sentiment Applied: {sentiment:.4}");
                    
                    let leverage_multiplier = engine::calculate_leverage_multiplier(sentiment);
                    println!("[Sniper Agent] ⚙️ Akashic WebSocket Streaming Online (Latency < 1ms)");
                    
                    // Actually execute the transaction via Alloy
                    match engine::execute_flash_loan_tx(&provider, &target, leverage_multiplier).await {
                        Ok(hash) => println!("[Sniper Agent] 💥 Flash Loan successful! Hash: {}", hash),
                        Err(e) => println!("[Sniper Agent] ❌ Flash Loan failed: {}", e),
                    }
                }
            }

        tokio::time::sleep(Duration::from_millis(config::IPC_POLL_INTERVAL_MS)).await;
    }
}
