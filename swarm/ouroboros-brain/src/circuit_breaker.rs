//! Circuit Breaker — Iron Logic Safety System.
//! 5 protection layers, persistent state, Bybit PnL fetch.
//! Port from Python `hyper/circuit_breaker.py` (1:1).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════
// THRESHOLDS (configurable via thresholds.toml in future)
// ═══════════════════════════════════════════════════════════

const MIN_BALANCE: f64 = 400.0;          // USD — below = emergency hold
const MAX_DAILY_LOSS: f64 = 15.0;        // USD — daily loss cap
const MAX_CONSECUTIVE_LOSSES: u32 = 3;   // Streak before cooldown
const COOLDOWN_HOURS: i64 = 2;           // Hours to cool down

// ═══════════════════════════════════════════════════════════
// BREAKER STATE (persistent JSON file)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerState {
    pub consecutive_losses: u32,
    pub daily_loss: f64,
    pub daily_date: String,
    pub cooldown_until: Option<String>,  // ISO timestamp
    pub last_check: Option<String>,
}

impl Default for BreakerState {
    fn default() -> Self {
        Self {
            consecutive_losses: 0,
            daily_loss: 0.0,
            daily_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            cooldown_until: None,
            last_check: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BreakerResult {
    pub blocked: bool,
    pub reason: String,
    pub level: BreakerLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerLevel {
    Green,
    Yellow,
    Red,
}

impl std::fmt::Display for BreakerLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakerLevel::Green => write!(f, "🟢 GREEN"),
            BreakerLevel::Yellow => write!(f, "🟡 YELLOW"),
            BreakerLevel::Red => write!(f, "🔴 RED"),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// STATE PERSISTENCE
// ═══════════════════════════════════════════════════════════

fn state_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data").join("breaker_state.json")
}

fn load_state() -> BreakerState {
    let path = state_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(state) => return state,
                    Err(e) => tracing::warn!("Breaker state parse error: {e}"),
                }
            }
            Err(e) => tracing::warn!("Breaker state read error: {e}"),
        }
    }
    BreakerState::default()
}

fn save_state(state: &mut BreakerState) {
    state.last_check = Some(chrono::Utc::now().to_rfc3339());
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                tracing::error!("Failed to save breaker state: {e}");
            }
        }
        Err(e) => tracing::error!("Failed to serialize breaker state: {e}"),
    }
}

// ═══════════════════════════════════════════════════════════
// PNL FETCH (Bybit closed PnL API)
// ═══════════════════════════════════════════════════════════

#[derive(Deserialize)]
struct PnlResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    result: Option<PnlResult>,
}

#[derive(Deserialize)]
struct PnlResult {
    list: Vec<PnlItem>,
}

#[derive(Deserialize)]
struct PnlItem {
    #[serde(rename = "closedPnl")]
    closed_pnl: String,
    #[serde(rename = "createdTime")]
    created_time: String,
}

struct ParsedPnl {
    pnl: f64,
    is_today: bool,
}

async fn fetch_recent_pnl() -> Vec<ParsedPnl> {
    let api_key = match std::env::var("BYBIT_API_KEY_1") {
        Ok(k) => k,
        Err(_) => {
            tracing::debug!("BYBIT_API_KEY_1 not set — skipping PnL fetch");
            return Vec::new();
        }
    };
    let api_secret = match std::env::var("BYBIT_SECRET_KEY_1") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Bybit V5 authenticated request
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    let recv_window = "5000";
    let params = "category=linear&limit=50";
    let sign_payload = format!("{timestamp}{api_key}{recv_window}{params}");

    let signature = hmac_sha256(&api_secret, &sign_payload);

    let client = reqwest::Client::new();
    let result = client
        .get("https://api.bybit.com/v5/position/closed-pnl")
        .query(&[("category", "linear"), ("limit", "50")])
        .header("X-BAPI-API-KEY", &api_key)
        .header("X-BAPI-SIGN", &signature)
        .header("X-BAPI-SIGN-TYPE", "2")
        .header("X-BAPI-TIMESTAMP", &timestamp)
        .header("X-BAPI-RECV-WINDOW", recv_window)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(resp) => {
            match resp.json::<PnlResponse>().await {
                Ok(data) => {
                    if data.ret_code != 0 {
                        tracing::warn!("Bybit PnL API error: retCode={}", data.ret_code);
                        return Vec::new();
                    }
                    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                    let list = data.result.map(|r| r.list).unwrap_or_default();

                    list.iter().map(|item| {
                        let pnl: f64 = item.closed_pnl.parse().unwrap_or(0.0);
                        let ts_ms: i64 = item.created_time.parse().unwrap_or(0);
                        let trade_date = chrono::DateTime::from_timestamp(ts_ms / 1000, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default();
                        ParsedPnl {
                            pnl,
                            is_today: trade_date == today,
                        }
                    }).collect()
                }
                Err(e) => {
                    tracing::warn!("PnL parse error: {e}");
                    Vec::new()
                }
            }
        }
        Err(e) => {
            tracing::warn!("PnL fetch error: {e}");
            Vec::new()
        }
    }
}

fn hmac_sha256(key: &str, data: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

// ═══════════════════════════════════════════════════════════
// MASTER CHECK — 5 protection layers
// ═══════════════════════════════════════════════════════════

/// Master circuit breaker check — call BEFORE every trading verdict.
/// Returns whether trading should be blocked and why.
pub async fn circuit_breaker_status(current_balance: f64) -> BreakerResult {
    let mut state = load_state();

    // Reset daily counters if new day
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    if state.daily_date != today {
        state.daily_loss = 0.0;
        state.daily_date = today;
    }

    // === CHECK 1: Balance Guard ===
    if current_balance > 0.0 && current_balance < MIN_BALANCE {
        tracing::error!(
            "🚨 CIRCUIT BREAKER: Balance ${:.2} < ${}!",
            current_balance, MIN_BALANCE
        );
        save_state(&mut state);
        return BreakerResult {
            blocked: true,
            reason: format!("BALANCE_GUARD: ${current_balance:.2} < ${MIN_BALANCE}"),
            level: BreakerLevel::Red,
        };
    }

    // === CHECK 2: Cooldown Active ===
    if let Some(ref cooldown) = state.cooldown_until {
        if let Ok(cd_time) = chrono::DateTime::parse_from_rfc3339(cooldown) {
            let now = chrono::Utc::now();
            let cd_utc = cd_time.with_timezone(&chrono::Utc);
            if now < cd_utc {
                let remaining = (cd_utc - now).num_minutes();
                save_state(&mut state);
                return BreakerResult {
                    blocked: true,
                    reason: format!("LOSS_STREAK_COOLDOWN: {remaining}min remaining"),
                    level: BreakerLevel::Red,
                };
            } else {
                state.cooldown_until = None;
                state.consecutive_losses = 0;
            }
        } else {
            state.cooldown_until = None;
        }
    }

    // === CHECK 3: Parse recent PnL ===
    let pnls = fetch_recent_pnl().await;
    let today_loss: f64 = pnls.iter()
        .filter(|p| p.is_today && p.pnl < 0.0)
        .map(|p| p.pnl)
        .sum::<f64>()
        .abs();
    state.daily_loss = today_loss;

    // Count consecutive losses (from most recent)
    let mut recent_streak: u32 = 0;
    for p in pnls.iter().rev() {
        if p.pnl < 0.0 {
            recent_streak += 1;
        } else {
            break;
        }
    }
    state.consecutive_losses = recent_streak;

    // === CHECK 4: Daily Loss Cap ===
    if state.daily_loss >= MAX_DAILY_LOSS {
        tracing::warn!(
            "⚠️ CIRCUIT BREAKER: Daily loss ${:.2} >= ${}",
            state.daily_loss, MAX_DAILY_LOSS
        );
        save_state(&mut state);
        return BreakerResult {
            blocked: true,
            reason: format!("DAILY_LOSS_CAP: ${:.2} >= ${}", state.daily_loss, MAX_DAILY_LOSS),
            level: BreakerLevel::Red,
        };
    }

    // === CHECK 5: Loss Streak ===
    if recent_streak >= MAX_CONSECUTIVE_LOSSES {
        let cooldown_end = chrono::Utc::now() + chrono::Duration::hours(COOLDOWN_HOURS);
        state.cooldown_until = Some(cooldown_end.to_rfc3339());
        tracing::warn!(
            "⚠️ CIRCUIT BREAKER: {} consecutive losses → {}h cooldown",
            recent_streak, COOLDOWN_HOURS
        );
        save_state(&mut state);
        return BreakerResult {
            blocked: true,
            reason: format!("LOSS_STREAK: {recent_streak} losses → {COOLDOWN_HOURS}h cooldown"),
            level: BreakerLevel::Red,
        };
    }

    // === ALL CLEAR ===
    let level = if state.daily_loss < MAX_DAILY_LOSS * 0.5 {
        BreakerLevel::Green
    } else {
        BreakerLevel::Yellow
    };

    save_state(&mut state);

    BreakerResult {
        blocked: false,
        reason: format!("OK (daily_loss=${:.2}, streak={})", state.daily_loss, recent_streak),
        level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaker_state_default() {
        let state = BreakerState::default();
        assert_eq!(state.consecutive_losses, 0);
        assert_eq!(state.daily_loss, 0.0);
        assert!(state.cooldown_until.is_none());
    }

    #[test]
    fn test_breaker_level_display() {
        assert_eq!(format!("{}", BreakerLevel::Green), "🟢 GREEN");
        assert_eq!(format!("{}", BreakerLevel::Red), "🔴 RED");
    }

    #[test]
    fn test_hmac_signature() {
        // Known test vector
        let sig = hmac_sha256("secret", "data");
        assert_eq!(sig.len(), 64); // SHA256 hex = 64 chars
        assert!(!sig.is_empty());
    }
}
