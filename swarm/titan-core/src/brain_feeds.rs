// src/modules/brain_feeds.rs
use serde_json::Value;
use crate::safe_io::data_file;
use std::sync::Mutex;
use std::time::Instant;

/// МЕНЕДЖЕР ВНЕШНИХ СИГНАЛОВ И ПАМЯТИ Ouroboros / Treasury
pub struct BrainFeeds;

// Treasury Cache: 10-second TTL to reduce 48 reads/cycle → 1
static TREASURY_CACHE: Mutex<Option<(Instant, Value)>> = Mutex::new(None);
// BUG-10 FIX: Blacklist Cache — 60-second TTL (was reading file 48+ times/cycle)
static BLACKLIST_CACHE: Mutex<Option<(Instant, Vec<String>)>> = Mutex::new(None);
// PREDATOR-09 FIX: Hour bias cache — 300s TTL (file updates every 24h)
static HOUR_BIAS_CACHE: Mutex<Option<(Instant, Value)>> = Mutex::new(None);

// V6.0: MICRO-exclusive memecoins (FALLBACK if JSON missing)
pub const MICRO_EXCLUSIVE_TICKERS: &[&str] = &[
    "DOGEUSDT", "1000PEPEUSDT", "WIFUSDT"
];

// OPUS 10.0: Hardcoded fallback — primary source is meme_blacklist.json
const MEME_TICKERS_FALLBACK: &[&str] = &[
    "SHIBUSDT", "MEMEUSDT", "TURBOUSDT", "NEIROUSDT",
    "ADAUSDT", "LDOUSDT", "SAPIENUSDT", "GRIFFAINUSDT", "MONUSDT",
    "MEWUSDT", "POPCATUSDT", "BRETTUSDT",
    "GOATUSDT", "ACTUSDT", "CHILLGUYUSDT",
    "RENDERUSDT", "VIRTUALUSDT",
    "RAVEUSDT", "BOMEUSDT", "BLURUSDT", "API3USDT",
    "GENIUSUSDT", "BLESSUSDT", "ARIAUSDT",
];

impl BrainFeeds {
    /// Протокол "Dead Man's Switch": Проверка жизнеспособности Роя (Treasury/Ouroboros)
    pub fn is_swarm_heartbeat_alive() -> bool {
        let treasury_path = data_file("treasury_state.json");
        let ouroboros_path = data_file("ouroboros_verdicts.json");
        
        let is_stale = |path: &str| -> bool {
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(modified) = meta.modified() {
                    let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                    return age.as_secs() > 300; // 5 минут (300 секунд)
                }
            }
            true // Если файл не читается, считаем пульс мертвым
        };

        if is_stale(&treasury_path) {
            tracing::warn!("💀 [DEAD_MAN_SWITCH] Казначей мертв (>5 мин). Входы заблокированы!");
            return false;
        }

        if is_stale(&ouroboros_path) {
            tracing::warn!("💀 [DEAD_MAN_SWITCH] Уроборос мертв (>5 мин). Входы заблокированы!");
            return false;
        }

        true // Пульс в норме
    }

    /// Adaptive MEME blacklist: cached 60s, reads from meme_blacklist.json, fallback to hardcoded
    fn load_meme_blacklist() -> Vec<String> {
        // BUG-10 FIX: check cache first (60s TTL)
        if let Ok(cache) = BLACKLIST_CACHE.lock() {
            if let Some((ts, ref list)) = *cache {
                if ts.elapsed().as_secs() < 60 { return list.clone(); }
            }
        }
        let path = data_file("meme_blacklist.json");
        let result = if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                let mut list: Vec<String> = Vec::new();
                for key in &["meme_tickers", "micro_exclusive", "auto_blacklisted"] {
                    if let Some(arr) = json[key].as_array() {
                        for v in arr { if let Some(s) = v.as_str() { list.push(s.to_string()); } }
                    }
                }
                if !list.is_empty() { list } else { Self::fallback_blacklist() }
            } else { Self::fallback_blacklist() }
        } else { Self::fallback_blacklist() };
        // Update cache
        if let Ok(mut cache) = BLACKLIST_CACHE.lock() {
            *cache = Some((Instant::now(), result.clone()));
        }
        result
    }

    fn fallback_blacklist() -> Vec<String> {
        let mut fallback: Vec<String> = MEME_TICKERS_FALLBACK.iter().map(std::string::ToString::to_string).collect();
        fallback.extend(MICRO_EXCLUSIVE_TICKERS.iter().map(std::string::ToString::to_string));
        fallback
    }

    pub fn is_toxic_asset(symbol: &str) -> bool {
        let blacklist = Self::load_meme_blacklist();
        blacklist.iter().any(|s| s == symbol) || Self::is_symbol_tilt_locked(symbol)
    }

    /// [Omni-Memory] Проверка горячего Мозжечкового Кэша на Тилт
    pub fn is_symbol_tilt_locked(symbol: &str) -> bool {
        let path = data_file("tilt_lock.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(entry) = json.get(symbol) {
                    if let Some(locked) = entry["locked"].as_bool() {
                        if locked {
                            if let Some(unlock_ts) = entry["unlock_timestamp_ms"].as_u64() {
                                let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock before UNIX epoch").as_millis() as u64;
                                if now_ms < unlock_ts {
                                    tracing::info!(symbol = %symbol, reason = entry["reason"].as_str().unwrap_or(""), "[ANTI-TILT] Blocked by Left Hemisphere");
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// [Omni-Memory] ПРАВОЕ ПОЛУШАРИЕ: Чтение alpha_boost.json
    /// Возвращает (leverage_multiplier, score_bonus) для S-TIER монет.
    pub fn read_alpha_boost(symbol: &str) -> (f64, f64) {
        let path = data_file("alpha_boost.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(entry) = json.get(symbol) {
                    let multiplier = entry["leverage_multiplier"].as_f64().unwrap_or(1.0);
                    let status = entry["status"].as_str().unwrap_or("NONE");
                    
                    // Score bonus зависит от статуса
                    let score_bonus = match status {
                        "MONSTER" => 3.0,
                        "S_TIER" => 2.0,
                        "BOOSTED" => 1.0,
                        _ => 0.0,
                    };
                    
                    if score_bonus > 0.0 {
                        tracing::info!(symbol = %symbol, status = %status, multiplier = format!("{multiplier:.1}").as_str(), score_bonus = format!("{score_bonus:.0}").as_str(), "[ALPHA BOOST]");
                    }
                    return (multiplier, score_bonus);
                }
            }
        }
        (1.0, 0.0) // Нет бустинга
    }

    // ═══ TREASURY CACHED READER (BUG-1 fix: 48 I/O → 1) ═══
    fn read_treasury_json() -> Option<Value> {
        let path = data_file("treasury_state.json");
        let mut cache = TREASURY_CACHE.lock().ok()?;
        if let Some((ts, ref val)) = *cache {
            if ts.elapsed().as_secs() < 10 { return Some(val.clone()); }
        }
        let data = std::fs::read_to_string(path).ok()?;
        let json: Value = serde_json::from_str(&data).ok()?;
        *cache = Some((Instant::now(), json.clone()));
        Some(json)
    }



    pub fn read_ouroboros_verdict(symbol: &str) -> String {
        let path = data_file("ouroboros_verdicts.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(entry) = json.get(symbol) {
                    if let Some(ts_str) = entry["timestamp"].as_str() {
                        if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                            let age = chrono::Utc::now().signed_duration_since(ts.with_timezone(&chrono::Utc));
                            if age.num_seconds() > 120 {
                                return "NEUTRAL".to_string(); // Stale
                            }
                        }
                    }
                    return entry["verdict"].as_str().unwrap_or("NEUTRAL").to_string();
                }
            }
        }
        "NEUTRAL".to_string()
    }

    pub fn read_macro_bias() -> String {
        let v = Self::read_ouroboros_verdict("BTCUSDT");
        match v.as_str() {
            "BUY" => "LONG".to_string(),
            "SELL" => "SHORT".to_string(),
            _ => "NEUTRAL".to_string(),
        }
    }

    pub fn read_liq_magnet(symbol: &str) -> f64 {
        let path = data_file("liq_heatmap.json");
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d, Err(_) => return 0.0,
        };
        let json: serde_json::Value = match serde_json::from_str(&data) {
            Ok(j) => j, Err(_) => return 0.0,
        };
        if let Some(ts) = json["generated_at"].as_str() {
            if let Ok(gen) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f") {
                let age = chrono::Utc::now().naive_utc() - gen;
                if age.num_seconds() > 600 { return 0.0; } 
            }
        }
        if let Some(magnets) = json["top_magnets"].as_array() {
            for m in magnets {
                if m["symbol"].as_str() == Some(symbol) {
                    let pct = m["pct"].as_f64().unwrap_or(0.0);
                    let mtype = m["type"].as_str().unwrap_or("");
                    if mtype == "SHORT_LIQ" && pct > 1.0 && pct < 5.0 {
                        return 1.5;  
                    }
                    if mtype == "LONG_LIQ" && pct > -3.0 && pct < -0.5 {
                        return -1.0; 
                    }
                }
            }
        }
        0.0
    }

    pub fn read_treasury_session_limit() -> f64 {
        // Priority 1: Treasury (live from Казначей)
        if let Some(limit) = Self::read_treasury_json().and_then(|j| j["session_loss_limit"].as_f64()) {
            return limit;
        }
        // Priority 2: Calibration (from backtester V5 auto-tuning)
        let cal_path = data_file("titan_calibration.json");
        if let Ok(content) = std::fs::read_to_string(cal_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(limit) = json["session_limit"].as_f64() {
                    return limit;
                }
            }
        }
        18.0 // Fallback
    }

    pub fn read_treasury_size(score: f64) -> f64 {
        let json = match Self::read_treasury_json() {
            Some(j) => j, None => return 3.0,
        };
        
        let mut target_size = json["size_tier_base"].as_f64().unwrap_or(3.0);
        
        if score >= 10.0 {
            target_size = json["size_tier_god_class"].as_f64().unwrap_or(15.0);
        } else if score >= 7.0 {
            target_size = json["size_tier_s_class"].as_f64().unwrap_or(10.0);
        } else if score >= 5.0 {
            target_size = json["size_tier_strong"].as_f64().unwrap_or(6.0);
        }
        
        let max_cap = json["max_position_usdt"].as_f64().unwrap_or(75.0);
        if target_size > max_cap { target_size = max_cap; }
        
        target_size
    }

    pub fn read_swarm_idle_boost() -> f64 {
        let v7_idle = Self::is_bot_idle(&data_file("meme_momentum.json"), 3 * 3600);
        let v13_idle = Self::is_bot_idle(&data_file("vwap_state.json"), 3 * 3600);
        if v7_idle && v13_idle { 1.5 } 
        else if v7_idle || v13_idle { 1.25 } 
        else { 1.0 }
    }

    fn is_bot_idle(state_path: &str, max_age_secs: i64) -> bool {
        if let Ok(meta) = std::fs::metadata(state_path) {
            if let Ok(modified) = meta.modified() {
                let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                return age.as_secs() > max_age_secs as u64;
            }
        }
        true
    }

    /// BUG-2 FIX: Проверяет реальные ownership файлы V7/V13, а не пустое поле Treasury
    pub fn is_symbol_held_by_other_bot(symbol: &str) -> bool {
        let paths = [
            data_file("meme_ownership.json"),
            data_file("vwap_ownership.json"),
        ];
        for path in &paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = json.as_object() {
                        if obj.contains_key(symbol) {
                            tracing::info!(symbol = %symbol, bot = std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or(""), "[ANTI-DUP] Already held by another bot");
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn read_whale_radar(symbol: &str) -> f64 {
        let path = data_file("whale_alerts.json");
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d, Err(_) => return 0.0,
        };
        let json: serde_json::Value = match serde_json::from_str(&data) {
            Ok(j) => j, Err(_) => return 0.0,
        };
        // BUG-24 FIX: staleness check (>10 min = ignore)
        if let Some(ts) = json["timestamp"].as_str() {
            if let Ok(gen) = chrono::DateTime::parse_from_rfc3339(ts) {
                let age = chrono::Utc::now().signed_duration_since(gen.with_timezone(&chrono::Utc));
                if age.num_seconds() > 600 { return 0.0; }
            }
        } else if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if modified.elapsed().unwrap_or_default().as_secs() > 600 { return 0.0; }
            }
        }
        if let Some(entry) = json.get(symbol) {
            let action = entry["action"].as_str().unwrap_or("NONE");
            if action == "BUY" { return 3.0; }
            if action == "SELL" { return -3.0; }
        }
        0.0
    }

    pub fn read_oi_tracker(symbol: &str) -> f64 {
        let path = data_file("oi_alerts.json");
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d, Err(_) => return 0.0,
        };
        let json: serde_json::Value = match serde_json::from_str(&data) {
            Ok(j) => j, Err(_) => return 0.0,
        };
        // BUG-24 FIX: staleness check (>10 min = ignore)
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if modified.elapsed().unwrap_or_default().as_secs() > 600 { return 0.0; }
            }
        }
        if let Some(entry) = json.get(symbol) {
            let status = entry["status"].as_str().unwrap_or("NONE");
            if status == "OVERHEATED_LONG" { return -2.0; }
            if status == "OVERHEATED_SHORT" { return 2.0; }
        }
        0.0
    }

    /// [Vector 6] Часовой модификатор на основе исторической статистики
    /// НЕ блокирует, а корректирует Score: -1.0 (осторожность) / +1.0 (агрессия) / 0.0 (нейтрально)
    pub fn read_hour_bias() -> f64 {
        let path = data_file("hour_performance.json");
        // PREDATOR-09 FIX: cached read (300s TTL)
        let json = {
            if let Ok(mut cache) = HOUR_BIAS_CACHE.lock() {
                if let Some((ts, ref val)) = *cache {
                    if ts.elapsed().as_secs() < 300 { Some(val.clone()) }
                    else { None }
                } else { None }
                .or_else(|| {
                    let data = std::fs::read_to_string(path).ok()?;
                    let j: Value = serde_json::from_str(&data).ok()?;
                    *cache = Some((Instant::now(), j.clone()));
                    Some(j)
                })
            } else { None }
        };
        if let Some(json) = json {
            let now = chrono::Utc::now();
            let hour_key = now.format("%H").to_string();
            if let Some(entry) = json.get(&hour_key) {
                let is_toxic = entry["toxic"].as_bool().unwrap_or(false);
                let is_golden = entry["golden"].as_bool().unwrap_or(false);
                let pnl = entry["pnl"].as_f64().unwrap_or(0.0);
                
                if is_golden {
                    return 1.0; // Исторически прибыльный час — чуть агрессивнее
                }
                if is_toxic && pnl < -30.0 {
                    return -1.5; // Сильно токсичный (>$30 потерь) — осторожность
                }
                if is_toxic {
                    return -0.5; // Умеренно токсичный — лёгкая осторожность
                }
            }
        }
        0.0
    }
}
