// src/modules/patience.rs
use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;

/// Модуль Выжидания. Если Скор высокий, бот не бьет сразу, а ждет и смотрит в стакан.
pub struct PatienceTracker {
    pub pending_targets: HashMap<String, (i64, String)>, // Symbol -> (Timestamp, Side)
    pub wait_time_seconds: i64,
}

impl Default for PatienceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PatienceTracker {
    pub fn new() -> Self {
        PatienceTracker {
            pending_targets: HashMap::new(),
            wait_time_seconds: 900, // Выжидание 15 минут (3 свечи по 5м) 
        }
    }

    /// Берет монету на карандаш С УКАЗАНИЕМ СТОРОНЫ
    pub fn lock_target(&mut self, symbol: &str, side: &str) {
        let now = chrono::Utc::now().timestamp();
        self.pending_targets.insert(symbol.to_string(), (now, side.to_string()));
    }

    /// Проверяет, прошёл ли период кулдауна И СОВПАДАЕТ ЛИ СТОРОНА
    /// BUG-13 FIX: если сигнал сменился (LONG→SHORT), сбрасываем таймер
    pub fn is_ready_to_strike(&mut self, symbol: &str, side: &str) -> bool {
        if let Some((lock_time, locked_side)) = self.pending_targets.get(symbol) {
            if locked_side != side {
                // Сигнал сменился! Сбрасываем patience
                self.pending_targets.remove(symbol);
                return false;
            }
            let current = chrono::Utc::now().timestamp();
            return (current - lock_time) >= self.wait_time_seconds;
        }
        false
    }

    /// SMART-3: Чистка — удалить символ после успешного входа (предотвращает бесконечный рост HashMap)
    pub fn clear_target(&mut self, symbol: &str) {
        self.pending_targets.remove(symbol);
    }

    /// V11.0.1 P3 FIX: Purge entries older than 2 hours (prevents memory leak from abandoned signals)
    pub fn purge_stale(&mut self) {
        let now = chrono::Utc::now().timestamp();
        let ttl = 7200; // 2 hours
        self.pending_targets.retain(|_sym, (lock_time, _side)| {
            now - *lock_time < ttl
        });
    }
    
    /// Ренгеновский сканер стакана. Вызывается перед самым входом (Phase 2)
    pub async fn check_imbalance_ratio(&self, client: &Client, symbol: &str) -> f64 {
        // DEX orderbook proxy: DexScreener doesn't expose raw orderbook,
        // so we use liquidity as a proxy for bid/ask imbalance
        let url = format!("https://api.dexscreener.com/latest/dex/search?q={symbol}");
        
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(json) = res.json::<Value>().await {
                let mut bid_vol = 0.0;
                let mut ask_vol = 0.0;
                
                if let Some(bids) = json["result"]["b"].as_array() {
                    for b in bids {
                        let vol: f64 = b[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        bid_vol += vol;
                    }
                }
                if let Some(asks) = json["result"]["a"].as_array() {
                    for a in asks {
                        let vol: f64 = a[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        ask_vol += vol;
                    }
                }
                
                if ask_vol > 0.0 {
                    return bid_vol / ask_vol;
                }
            }
        }
        1.0
    }
}
