// src/modules/scanner.rs
// Модуль сканирования рынка. Hype Scanner + BTC Gravity.
use serde_json::Value;
use reqwest::Client;
use crate::brain_feeds::BrainFeeds;

pub struct MarketScanner;

impl MarketScanner {
    /// BTC Gravity: returns % change for BTC via CoinGecko (no CEX dependency)
    pub async fn check_btc_gravity(client: &Client) -> f64 {
        // Use CoinGecko 24h change as gravity proxy (CEX-free)
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true";
        if let Ok(res) = client.get(url).send().await {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(change) = json["bitcoin"]["usd_24h_change"].as_f64() {
                    return change;
                }
            }
        }
        0.0
    }

    /// Hype Scanner: top tokens by volume from DexScreener (Mantle chain)
    pub async fn get_hype_coins(client: &Client) -> Vec<String> {
        // DexScreener Mantle chain tokens, sorted by volume
        let url = "https://api.dexscreener.com/latest/dex/tokens/0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8";
        if let Ok(res) = client.get(url).send().await {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(pairs) = json["pairs"].as_array() {
                    let mut results: Vec<(String, f64)> = Vec::new();
                    for pair in pairs {
                        let sym = pair["baseToken"]["symbol"].as_str().unwrap_or("");
                        if sym.is_empty() { continue; }
                        let ticker = format!("{sym}USDT");
                        if BrainFeeds::is_toxic_asset(&ticker) { continue; }
                        let vol = pair["volume"]["h24"].as_f64().unwrap_or(0.0);
                        if vol > 50_000.0 { results.push((ticker, vol)); }
                    }
                    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    return results.into_iter().take(12).map(|(s,_)| s).collect();
                }
            }
        }
        // Fallback: Mantle ecosystem defaults
        vec!["MNTUSDT".to_string(), "WMNTUSDT".to_string()]
    }

    /// Проверяет, является ли символ мемом (делегирует BrainFeeds с адаптивным JSON)
    #[allow(dead_code)] // Reserved for future entry gate integration
    pub fn is_meme(symbol: &str) -> bool {
        BrainFeeds::is_toxic_asset(symbol)
    }
}
