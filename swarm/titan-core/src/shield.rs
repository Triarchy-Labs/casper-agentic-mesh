// src/modules/shield.rs
use reqwest::Client;
use serde_json::Value;

/// Модуль Брони. Запрет на статические стоп-лоссы.
pub struct WhaleShield;

impl WhaleShield {
    pub async fn calculate_whale_stop_loss(client: &Client, symbol: &str, current_price: f64, side: &str) -> f64 {
        // DEX orderbook proxy: try DexScreener first, no CEX dependency
        let url = format!("https://api.dexscreener.com/latest/dex/search?q={symbol}");
        
        let search_depth = if side == "Buy" { 0.97 } else { 1.03 };
        let bound_price = current_price * search_depth;
        
        let mut whale_wall_price = current_price * (if side == "Buy" { 0.985 } else { 1.015 }); // Fallback на классический стоп 1.5%
        
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(json) = res.json::<Value>().await {
                let mut max_volume = 0.0;
                
                let target_book = if side == "Buy" { "b" } else { "a" }; // bids для лонга, asks для шорта
                
                if let Some(orders) = json["result"][target_book].as_array() {
                    // FORENSIC-12: compute total side volume for dynamic threshold
                    let total_side_vol: f64 = orders.iter()
                        .map(|o| {
                            let px: f64 = o[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                            let qty: f64 = o[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                            px * qty
                        }).sum();
                    let wall_threshold = (total_side_vol * 0.20).max(3000.0); // 20% of book, min $3k

                    for o in orders {
                        let px: f64 = o[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let qty: f64 = o[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let vol = px * qty;
                        
                        if side == "Buy" && px < bound_price { break; }
                        if side == "Sell" && px > bound_price { break; }
                        
                        // BUG-04 FIX: ignore walls too close to price (< 1%)
                        let distance_pct = ((px - current_price) / current_price * 100.0).abs();
                        if distance_pct < 1.0 { continue; }
                        
                        // FORENSIC-12: dynamic threshold based on % of total book
                        if vol > max_volume && vol > wall_threshold {
                            max_volume = vol;
                            whale_wall_price = px;
                        }
                    }
                }
            }
        }
        
        // Укрываемся ПОД стенкой (если лонг) или НАД стенкой (если шорт)
        if side == "Buy" {
            whale_wall_price * 0.9995 
        } else {
            whale_wall_price * 1.0005
        }
    }
}
