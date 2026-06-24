// src/modules/execution.rs
// ═══════════════════════════════════════════════════════════════
// P4: Extracted execution helpers from main.rs
// Contains: fetch_klines (market data) + update_stop_loss (exchange API)
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use serde_json::Value;
use chrono::Local;

use crate::api_pool::ApiPool;
use crate::network::BybitNetwork;
use crate::types::ActivePosition;

/// Kline data fetcher — returns (price, ATR, closes, volumes)
pub async fn fetch_klines(client: &Client, symbol: &str, interval: &str) -> Option<(f64, f64, Vec<f64>, Vec<f64>)> {
    let url = format!("https://api.bybit.com/v5/market/kline?category=linear&symbol={symbol}&interval={interval}&limit=50");
    if let Ok(res) = client.get(&url).send().await {
        if let Ok(json) = res.json::<Value>().await {
            if let Some(mut list) = json["result"]["list"].as_array().cloned() {
                list.reverse(); // oldest first
                if list.is_empty() { return None; }
                let price: f64 = match list.last() {
                    Some(v) => v[4].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                    None => return None,
                };
                // ATR
                let mut tr_list: Vec<f64> = Vec::new();
                for i in 1..list.len() {
                    let h: f64 = list[i][2].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    let l: f64 = list[i][3].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    let pc: f64 = list[i-1][4].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                    tr_list.push((h-l).max((h-pc).abs()).max((l-pc).abs()));
                }
                let atr = if tr_list.is_empty() { 0.0 } else { tr_list.iter().sum::<f64>() / tr_list.len() as f64 };
                let closes: Vec<f64> = list.iter().map(|c| c[4].as_str().unwrap_or("0").parse().unwrap_or(0.0)).collect();
                let vols: Vec<f64> = list.iter().map(|c| c[5].as_str().unwrap_or("0").parse().unwrap_or(0.0)).collect();
                return Some((price, atr, closes, vols));
            }
        }
    }
    None
}

/// Update server-side stop-loss via Bybit API
pub async fn update_stop_loss(client: &Client, api_pool: &Arc<ApiPool>, time_offset: &Arc<RwLock<i64>>,
    symbol: &str, stop_loss: f64, positions: &Arc<RwLock<HashMap<String, ActivePosition>>>,
    cooldowns: &Arc<RwLock<HashMap<String, i64>>>, cooldown_time: i64) -> bool {
    let timestamp = BybitNetwork::get_synced_timestamp(time_offset).await;
    let recv_window = "5000"; // BUG-21 FIX: was 20000
    let sl_str = format!("{stop_loss:.6}");
    let payload = serde_json::json!({"category":"linear","symbol":symbol,"stopLoss":sl_str,"tpslMode":"Full","positionIdx":0});
    let payload_str = payload.to_string();
    let (key, secret) = api_pool.get_current_keys();
    let sig = BybitNetwork::generate_signature(&timestamp, recv_window, &payload_str, &key, &secret);
    let res = client.post("https://api.bybit.com/v5/position/trading-stop")
        .header("X-BAPI-API-KEY", &key).header("X-BAPI-TIMESTAMP", &timestamp)
        .header("X-BAPI-SIGN", &sig).header("X-BAPI-RECV-WINDOW", recv_window)
        .header("Content-Type", "application/json").body(payload_str).send().await;
    if let Ok(response) = res {
        if let Ok(text) = response.text().await {
            if text.contains("\"retCode\":0") { return true; }
            if text.contains("10001") || text.contains("zero position") {
                positions.write().await.remove(symbol);
                cooldowns.write().await.insert(symbol.to_string(), Local::now().timestamp() + cooldown_time);
            }
            if text.contains("10006") || text.contains("429") { api_pool.rotate_keys(); }
        }
    }
    false
}
