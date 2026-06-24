use core_ipc::IpcBridge;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config;
use crate::engine;

pub async fn run_oracle_loop() {
    let mut ipc = IpcBridge::new();
    let client = reqwest::Client::new();

    loop {
        match client.get(config::GAMMA_API_URL).send().await {
            Ok(response) => {
                if let Ok(markets) = response.json::<Vec<Value>>().await
                    && let Some(avg_sentiment) = engine::extract_macro_sentiment(markets) {
                        println!("[Polymarket Oracle] Global Macro Sentiment Extracted: {avg_sentiment:.4}");

                        let mut state = ipc.read_state().unwrap_or_default();
                        state.global_sentiment_modifier = avg_sentiment;
                        state.timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        ipc.write_state(&state);
                        println!("[Polymarket Oracle] L0 IPC Memmap Updated: Sniper Agent leverage modified.");
                    }
            }
            Err(e) => {
                println!("[Polymarket Oracle] API Error: {e}");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(config::POLL_INTERVAL_SECS)).await;
    }
}
