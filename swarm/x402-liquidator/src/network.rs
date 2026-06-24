use core_ipc::IpcBridge;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::{config, engine};
use alloy::providers::ProviderBuilder;
use url::Url;

pub async fn run_liquidator_loop() {
    let mut ipc = IpcBridge::new();
    
    // Initialize Mantle Provider (Supports both Testnet and Mainnet via Env Var)
    let rpc_str = std::env::var("MANTLE_RPC_URL").unwrap_or_else(|_| "https://rpc.mantle.xyz".to_string());
    let rpc_url = Url::parse(&rpc_str).unwrap();
    let provider = ProviderBuilder::new().connect_http(rpc_url);
    println!("[Liquidator Daemon] Connected to Mantle RPC: {}", rpc_str);

    loop {
        if let Some(target) = engine::scan_for_targets(&provider).await {
            println!("[Liquidator Daemon] TARGET ACQUIRED: {} (Health: {})", target.address, target.health_factor);
            println!("[Liquidator Daemon] Human weakness detected. Preparing Agni Finance Flash Loan...");

            let mut state = ipc.read_state().unwrap_or_default();
            
            state.liquidation_target = Some(target.address);
            state.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();

            ipc.write_state(&state);
            println!("[Liquidator Daemon] L0 IPC Memmap Updated: Flash Loan execution request broadcasted to Swarm.");
        }

        tokio::time::sleep(Duration::from_secs(config::SCAN_INTERVAL_SECS)).await;
    }
}
