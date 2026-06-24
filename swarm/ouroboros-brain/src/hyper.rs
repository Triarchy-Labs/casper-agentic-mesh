//! Hyper Modules — Factors 6, 7, 9-14 (Alpha Station + MTF + ML + HiveMind).
//! Reads JSON files from Alpha Station demons + fetches 4H klines.
//! Port of: hyper_reader.py, mtf_reader.py, agent_alpha_intel, agent_ml_intel, v10_memory_bridge.

use reqwest::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

// ═══════════════════════════════════════════════════════════
// PATHS (matching Python hyper/ layout)
// ═══════════════════════════════════════════════════════════

const MAX_AGE_HOURS: u64 = 6;
const ML_MAX_AGE_MINUTES: u64 = 15;

fn alpha_root() -> PathBuf {
    PathBuf::from(
        std::env::var("ALPHA_STATION_PATH")
            .unwrap_or_else(|_| "data/alpha-station".to_string())
    )
}

fn alpha_path(filename: &str) -> PathBuf {
    alpha_root().join(filename)
}

fn normalize_symbol(sym: &str) -> String {
    sym.replace(['/', '-'], "").replace(":USDT", "").to_uppercase()
}

// ═══════════════════════════════════════════════════════════
// SAFE JSON LOADER (with file age check)
// ═══════════════════════════════════════════════════════════

fn load_json_fresh(path: &Path, max_age_secs: u64) -> Option<serde_json::Value> {
    if !path.exists() {
        return None;
    }
    // Check file age
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                if age > Duration::from_secs(max_age_secs) {
                    tracing::debug!("STALE: {:?} (age={}s)", path.file_name(), age.as_secs());
                    return None;
                }
            }
        }
    }
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(e) => {
            tracing::warn!("Failed to load {:?}: {}", path, e);
            None
        }
    }
}

// ═══════════════════════════════════════════════════════════
// FACTOR 6: ALPHA STATION (whale_alerts.json)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct AlphaResult {
    pub squeeze_signal: Option<String>, // "SHORT_SQUEEZE" or "LONG_SQUEEZE"
    pub squeeze_active: bool,
    pub fresh: bool,
    pub alerts_count: usize,
}

pub fn read_alpha_intel(symbol: &str) -> AlphaResult {
    let path = alpha_path("whale_alerts.json");
    let data = match load_json_fresh(&path, MAX_AGE_HOURS * 3600) {
        Some(d) => d,
        None => return AlphaResult::default(),
    };

    let alerts = if data.is_array() {
        data.as_array().cloned().unwrap_or_default()
    } else {
        data.get("alerts").and_then(|a| a.as_array()).cloned().unwrap_or_default()
    };

    let sym_norm = normalize_symbol(symbol);
    let mut squeeze = None;
    let mut squeeze_active = false;
    let mut count = 0;

    for alert in &alerts {
        let wallet_count = alert.get("wallet_count").and_then(serde_json::Value::as_u64).unwrap_or(0);
        if wallet_count >= 3 {
            squeeze_active = true;
            let action = alert.get("action").and_then(|v| v.as_str()).unwrap_or("");
            if action.contains("BUY") || action.contains("LONG") {
                squeeze = Some("SHORT_SQUEEZE".to_string());
            } else if action.contains("SELL") || action.contains("SHORT") {
                squeeze = Some("LONG_SQUEEZE".to_string());
            }
        }
        count += 1;
    }

    if count > 0 {
        tracing::debug!("[{}] Alpha: {} alerts, squeeze={}", sym_norm, count, squeeze_active);
    }

    AlphaResult {
        squeeze_signal: squeeze,
        squeeze_active,
        fresh: count > 0,
        alerts_count: count,
    }
}

// ═══════════════════════════════════════════════════════════
// FACTOR 7: ML PREDICTIONS (ml_predictions.json)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct MlResult {
    pub direction: i32,    // -1, 0, +1
    pub confidence: f64,   // 0.0 - 1.0
    pub fresh: bool,
}

pub fn read_ml_predictions(symbol: &str) -> MlResult {
    let path = alpha_path("ml_predictions.json");
    let data = match load_json_fresh(&path, ML_MAX_AGE_MINUTES * 60) {
        Some(d) => d,
        None => return MlResult::default(),
    };

    let sym_norm = normalize_symbol(symbol);
    let predictions = data.get("predictions").and_then(|p| p.as_object());

    if let Some(preds) = predictions {
        for (key, val) in preds {
            if normalize_symbol(key) == sym_norm {
                let direction = val.get("direction").and_then(serde_json::Value::as_i64).unwrap_or(0) as i32;
                let confidence = val.get("confidence").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
                tracing::info!("[{}] ML: dir={} conf={:.2}", sym_norm, direction, confidence);
                return MlResult { direction, confidence, fresh: true };
            }
        }
    }

    MlResult::default()
}

// ═══════════════════════════════════════════════════════════
// FACTORS 10-13: HYPER READER (Alpha Station demons)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct HyperResult {
    pub funding_score: f64,  // Factor 10
    pub oi_score: f64,       // Factor 11
    pub liq_score: f64,      // Factor 12
    pub whale_score: f64,    // Factor 13
    pub hyper_total: f64,
    pub factors_alive: u32,
}

/// Factor 10: Funding Extremes
fn read_funding_factor(symbol: &str) -> f64 {
    let path = alpha_path("funding_alerts.json");
    let data = match load_json_fresh(&path, MAX_AGE_HOURS * 3600) {
        Some(d) => d,
        None => return 0.0,
    };

    let sym_norm = normalize_symbol(symbol);

    for key in ["top_funding", "all_rates"] {
        if let Some(entries) = data.get(key).and_then(|v| v.as_array()) {
            for entry in entries {
                let esym = entry.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
                if normalize_symbol(esym) == sym_norm {
                    return score_funding(entry, symbol);
                }
            }
        }
    }
    0.0
}

fn score_funding(entry: &serde_json::Value, symbol: &str) -> f64 {
    let mut funding_pct = entry.get("funding_pct").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    if funding_pct == 0.0 {
        let raw = entry.get("funding_rate").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
        if raw != 0.0 { funding_pct = raw * 100.0; }
    }
    let signal = entry.get("signal").and_then(|v| v.as_str()).unwrap_or("");

    if funding_pct < -0.1 || signal == "SHORT_SQUEEZE" {
        tracing::info!("[{symbol}] F10: funding={funding_pct:.3}% → SHORT_SQUEEZE (+1.5)");
        1.5
    } else if funding_pct < -0.03 { 0.5 }
    else if funding_pct > 0.1 || signal == "LONG_SQUEEZE" {
        tracing::info!("[{symbol}] F10: funding={funding_pct:.3}% → LONG_SQUEEZE (-1.5)");
        -1.5
    } else if funding_pct > 0.03 { -0.5 }
    else { 0.0 }
}

/// Factor 11: OI Divergence
fn read_oi_factor(symbol: &str) -> f64 {
    let path = alpha_path("oi_alerts.json");
    let data = match load_json_fresh(&path, MAX_AGE_HOURS * 3600) {
        Some(d) => d,
        None => return 0.0,
    };
    let sym_norm = normalize_symbol(symbol);

    let entry = ["top_oi", "all_data"].iter().find_map(|key| {
        data.get(key)?.as_array()?.iter().find(|e| {
            normalize_symbol(e.get("symbol").and_then(|v| v.as_str()).unwrap_or("")) == sym_norm
        }).cloned()
    });

    let entry = match entry {
        Some(e) => e,
        None => return 0.0,
    };

    let change_24h = entry.get("change_24h").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    let signal = entry.get("signal").and_then(|v| v.as_str()).unwrap_or("");
    let oi_vol_ratio = entry.get("oi_vol_ratio").and_then(serde_json::Value::as_f64).unwrap_or(0.0);

    if signal == "HIGH_CONVICTION" {
        if change_24h > 5.0 { return 0.5; }
        if change_24h < -5.0 { return -0.5; }
        if change_24h > 2.0 { return 0.3; }
        if change_24h < -2.0 { return -0.3; }
    }
    if oi_vol_ratio > 1.5 {
        if change_24h > 2.0 { return -0.8; }
        if change_24h < -2.0 { return 0.8; }
    }
    0.0
}

/// Factor 12: Liquidation Magnets
fn read_liq_factor(symbol: &str, current_price: f64) -> f64 {
    if current_price <= 0.0 { return 0.0; }
    let path = alpha_path("liq_heatmap.json");
    let data = match load_json_fresh(&path, MAX_AGE_HOURS * 3600) {
        Some(d) => d,
        None => return 0.0,
    };

    let sym_norm = normalize_symbol(symbol);
    let sym_data = match data.get("symbols").and_then(|s| s.get(&sym_norm)) {
        Some(d) => d,
        None => return 0.0,
    };

    let levels = match sym_data.get("levels").and_then(|l| l.as_array()) {
        Some(l) => l,
        None => return 0.0,
    };

    let mut nearest_short = 999.0_f64;
    let mut nearest_long = 999.0_f64;

    for level in levels {
        let short_liq = level.get("short_liq_price").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
        let long_liq = level.get("long_liq_price").and_then(serde_json::Value::as_f64).unwrap_or(0.0);

        if short_liq > current_price {
            let dist = (short_liq - current_price) / current_price * 100.0;
            nearest_short = nearest_short.min(dist);
        }
        if long_liq > 0.0 && long_liq < current_price {
            let dist = (current_price - long_liq) / current_price * 100.0;
            nearest_long = nearest_long.min(dist);
        }
    }

    let mut score = 0.0;
    if nearest_short < 5.0 { score += 1.0; }
    else if nearest_short < 10.0 { score += 0.3; }
    if nearest_long < 5.0 { score -= 1.0; }
    else if nearest_long < 10.0 { score -= 0.3; }

    score
}

/// Factor 13: Whale Footprints
fn read_whale_factor(symbol: &str) -> f64 {
    let path = alpha_path("whale_alerts.json");
    let data = match load_json_fresh(&path, MAX_AGE_HOURS * 3600) {
        Some(d) => d,
        None => return 0.0,
    };
    let sym_norm = normalize_symbol(symbol);
    let alerts = if data.is_array() {
        data.as_array().cloned().unwrap_or_default()
    } else {
        data.get("alerts").and_then(|a| a.as_array()).cloned().unwrap_or_default()
    };

    let mut score: f64 = 0.0;
    for alert in &alerts {
        let asym = alert.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
        if normalize_symbol(asym) == sym_norm {
            let direction = alert.get("direction").and_then(|v| v.as_str()).unwrap_or("").to_uppercase();
            let size_usd = alert.get("size_usd").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            if size_usd > 100_000.0 {
                if direction == "BUY" || direction == "LONG" { score += 1.0; }
                else if direction == "SELL" || direction == "SHORT" { score -= 1.0; }
            }
        }
    }
    score.clamp(-2.0, 2.0)
}

/// Master reader — all 4 hyper factors combined.
pub fn read_hyper_factors(symbol: &str, current_price: f64) -> HyperResult {
    let f10 = read_funding_factor(symbol);
    let f11 = read_oi_factor(symbol);
    let f12 = read_liq_factor(symbol, current_price);
    let f13 = read_whale_factor(symbol);

    let alive = [f10, f11, f12, f13].iter().filter(|x| **x != 0.0).count() as u32;
    let total = f10 + f11 + f12 + f13;

    if alive > 0 {
        tracing::info!(
            "[{symbol}] HYPER: total={total:+.2} (funding={f10:+.1} oi={f11:+.1} liq={f12:+.1} whale={f13:+.1})"
        );
    }

    HyperResult {
        funding_score: (f10 * 100.0).round() / 100.0,
        oi_score: (f11 * 100.0).round() / 100.0,
        liq_score: (f12 * 100.0).round() / 100.0,
        whale_score: (f13 * 100.0).round() / 100.0,
        hyper_total: (total * 100.0).round() / 100.0,
        factors_alive: alive,
    }
}

// ═══════════════════════════════════════════════════════════
// FACTOR 9: MTF 4H TREND (EMA20/50 + RSI)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct MtfResult {
    pub score: f64,
    pub ema_cross: String,
    pub rsi: f64,
}

impl Default for MtfResult {
    fn default() -> Self {
        Self { score: 0.0, ema_cross: "UNKNOWN".into(), rsi: 50.0 }
    }
}

fn calc_ema(values: &[f64], period: usize) -> f64 {
    if values.is_empty() { return 0.0; }
    if values.len() < period { return *values.last().unwrap_or(&0.0); }
    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = values[0];
    for &price in &values[1..] {
        ema = price * k + ema * (1.0 - k);
    }
    ema
}

fn calc_rsi(closes: &[f64], period: usize) -> f64 {
    if closes.len() < period + 1 { return 50.0; }
    let mut gains = Vec::new();
    let mut losses = Vec::new();
    for i in 1..closes.len() {
        let delta = closes[i] - closes[i - 1];
        gains.push(delta.max(0.0));
        losses.push((-delta).max(0.0));
    }
    let n = gains.len();
    let start = n.saturating_sub(period);
    let avg_gain: f64 = gains[start..].iter().sum::<f64>() / period as f64;
    let avg_loss: f64 = losses[start..].iter().sum::<f64>() / period as f64;
    if avg_loss == 0.0 { return 100.0; }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

#[derive(Deserialize)]
struct KlineResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    result: Option<KlineResult>,
}

#[derive(Deserialize)]
struct KlineResult {
    list: Vec<Vec<String>>,
}

/// Fetch 4H klines from Bybit and calculate EMA cross + RSI.
pub async fn fetch_4h_trend(client: &Client, symbol: &str) -> MtfResult {
    let url = format!(
        "https://api.bybit.com/v5/market/kline?category=linear&symbol={symbol}&interval=240&limit=50"
    );

    let resp = match client.get(&url)
        .timeout(Duration::from_secs(10))
        .send().await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("[{symbol}] MTF 4H fetch failed: {e}");
            return MtfResult::default();
        }
    };

    let data: KlineResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("[{symbol}] MTF 4H parse failed: {e}");
            return MtfResult::default();
        }
    };

    if data.ret_code != 0 {
        return MtfResult::default();
    }

    let klines = match data.result {
        Some(r) => r.list,
        None => return MtfResult::default(),
    };

    if klines.len() < 25 {
        return MtfResult::default();
    }

    // Bybit returns newest first — reverse for EMA calc
    let closes: Vec<f64> = klines.iter().rev()
        .filter_map(|k| k.get(4).and_then(|v| v.parse::<f64>().ok()))
        .collect();

    if closes.len() < 25 {
        return MtfResult::default();
    }

    // EMA Cross
    let ema20 = calc_ema(&closes, 20);
    let ema50 = calc_ema(&closes, closes.len().min(50));
    let prev_ema20 = calc_ema(&closes[..closes.len() - 1], 20);
    let prev_ema50 = calc_ema(&closes[..closes.len() - 1], (closes.len() - 1).min(50));

    // RSI
    let rsi = calc_rsi(&closes, 14);

    let mut score: f64 = 0.0;
    let ema_cross;

    if ema20 > ema50 {
        score += 1.0;
        ema_cross = "BULLISH".to_string();
    } else if ema20 < ema50 {
        score -= 1.0;
        ema_cross = "BEARISH".to_string();
    } else {
        ema_cross = "FLAT".to_string();
    }

    // Fresh cross
    if prev_ema20 <= prev_ema50 && ema20 > ema50 { score += 0.5; }
    if prev_ema20 >= prev_ema50 && ema20 < ema50 { score -= 0.5; }

    // RSI extremes
    if rsi > 75.0 { score -= 0.5; }
    else if rsi < 25.0 { score += 0.5; }

    score = score.clamp(-1.5, 1.5);

    if score != 0.0 {
        tracing::info!("[{symbol}] 4H MTF: {ema_cross} score={score:+.1} RSI={rsi:.0}");
    }

    MtfResult {
        score: (score * 100.0).round() / 100.0,
        ema_cross,
        rsi: (rsi * 10.0).round() / 10.0,
    }
}

// ═══════════════════════════════════════════════════════════
// FACTOR 14: HIVEMIND (Memory Castle bridge)
// ═══════════════════════════════════════════════════════════

/// Query Memory Castle for pattern memory scoring.
/// Falls back to 0.0 if Memory Castle is not running.
pub async fn hivemind_query(client: &Client, symbol: &str, current_score: f64) -> f64 {
    // Memory Castle V3.2 exposes HTTP API on port 8889
    // BUG-CASTLE-01 FIX: was port 9300, Castle listens on 8889
    // BUG-CASTLE-02 FIX: was /api/v1/pattern (didn't exist), now uses /api/pattern
    let url = format!(
        "http://127.0.0.1:8889/api/pattern?symbol={symbol}&score={current_score:.2}"
    );

    match client.get(&url).timeout(Duration::from_secs(3)).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                let factor = data.get("factor_score").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
                if factor != 0.0 {
                    let detail = data.get("detail").and_then(|v| v.as_str()).unwrap_or("");
                    tracing::info!("[{symbol}] F14 HiveMind: {factor:+.1} | {detail}");
                }
                return factor;
            }
            0.0
        }
        Err(_) => {
            // Memory Castle offline = graceful degradation
            0.0
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(normalize_symbol("EDU/USDT"), "EDUUSDT");
        assert_eq!(normalize_symbol("SOL-USDT"), "SOLUSDT");
        assert_eq!(normalize_symbol("ETH/USDT:USDT"), "ETHUSDT");
        assert_eq!(normalize_symbol("BTCUSDT"), "BTCUSDT");
    }

    #[test]
    fn test_calc_ema() {
        let data = vec![10.0, 11.0, 12.0, 11.0, 13.0, 14.0];
        let ema = calc_ema(&data, 3);
        assert!(ema > 12.0 && ema < 14.0, "EMA should be between 12 and 14, got {ema}");
    }

    #[test]
    fn test_calc_rsi_neutral() {
        let data: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64) * 0.1).collect();
        let rsi = calc_rsi(&data, 14);
        assert!(rsi > 50.0, "Uptrend should have RSI > 50, got {rsi}");
    }

    #[test]
    fn test_calc_rsi_oversold() {
        let data: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64) * 2.0).collect();
        let rsi = calc_rsi(&data, 14);
        assert!(rsi < 20.0, "Strong downtrend should have RSI < 20, got {rsi}");
    }

    #[test]
    fn test_hyper_no_files() {
        // When Alpha Station files don't exist, should return zeros gracefully
        let result = read_hyper_factors("BTCUSDT", 96000.0);
        assert_eq!(result.factors_alive, 0);
        assert_eq!(result.hyper_total, 0.0);
    }

    #[test]
    fn test_ml_no_file() {
        let result = read_ml_predictions("BTCUSDT");
        assert!(!result.fresh);
        assert_eq!(result.direction, 0);
    }

    #[test]
    fn test_alpha_no_file() {
        let result = read_alpha_intel("BTCUSDT");
        assert!(!result.fresh);
        assert!(!result.squeeze_active);
    }
}
