// src/modules/deallow.rs
// ═══════════════════════════════════════════════════════════════
// AUTO-DEALLOW — Per-Symbol Performance Ban
// ═══════════════════════════════════════════════════════════════
// V11: Inspired by MySwarmbots/Swarmbots Architecture
//
// Scans every symbol's historical performance from DNA snapshot.
// If a symbol underperforms → ban it from hype_list.
// If it recovers → allow it back after cooldown.
//
// Ban criteria:   trade_count ≥ 30 AND win_rate < 0.40
// Reallow criteria: win_rate ≥ 0.55 AND cooldown expired (24h)

use serde_json::Value;
use crate::safe_io::data_file;

fn snapshot_path() -> String { data_file("hive_mind_snapshot.json") }
const MIN_TRADES_FOR_BAN: i64 = 30;
const BAN_WIN_RATE: f64 = 0.40;
const REALLOW_WIN_RATE: f64 = 0.55;

pub struct Deallow;

impl Deallow {
    /// Scan snapshot and return list of symbols that should be BANNED
    /// These symbols have enough trades AND underperform
    pub fn scan_underperformers() -> Vec<String> {
        let mut banned = Vec::new();

        if let Some(json) = Self::read_snapshot() {
            if let Some(obj) = json.as_object() {
                for (symbol, entity) in obj {
                    let trade_count = entity["trade_count"].as_i64().unwrap_or(0);
                    let win_rate = entity["win_rate"].as_f64().unwrap_or(0.5);

                    if trade_count >= MIN_TRADES_FOR_BAN && win_rate < BAN_WIN_RATE {
                        tracing::warn!(
                            symbol = %symbol,
                            wr = format!("{:.0}%", win_rate * 100.0).as_str(),
                            trades = trade_count,
                            "🚫 [DEALLOW] Symbol banned — underperformance"
                        );
                        banned.push(symbol.clone());
                    }
                }
            }
        }

        banned
    }

    /// Check if a specific symbol qualifies for reallow
    /// Returns true if symbol has recovered (WR ≥ 55%)
    pub fn check_reallow(symbol: &str) -> bool {
        if let Some(json) = Self::read_snapshot() {
            if let Some(entity) = json.get(symbol) {
                let trade_count = entity["trade_count"].as_i64().unwrap_or(0);
                let win_rate = entity["win_rate"].as_f64().unwrap_or(0.0);

                if trade_count >= MIN_TRADES_FOR_BAN && win_rate >= REALLOW_WIN_RATE {
                    tracing::info!(
                        symbol = %symbol,
                        wr = format!("{:.0}%", win_rate * 100.0).as_str(),
                        "✅ [DEALLOW] Symbol recovered — eligible for reallow"
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Filter a hype_list by removing underperformers
    /// Returns (filtered_list, removed_symbols)
    pub fn filter_hype_list(hype_list: &[String]) -> (Vec<String>, Vec<String>) {
        let banned = Self::scan_underperformers();
        let mut filtered = Vec::new();
        let mut removed = Vec::new();

        for sym in hype_list {
            if banned.contains(sym) {
                removed.push(sym.clone());
            } else {
                filtered.push(sym.clone());
            }
        }

        if !removed.is_empty() {
            tracing::warn!(
                removed = removed.len(),
                symbols = removed.join(",").as_str(),
                "🚫 [DEALLOW] Filtered from hype_list"
            );
        }

        (filtered, removed)
    }

    fn read_snapshot() -> Option<Value> {
        std::fs::read_to_string(&snapshot_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ban_threshold() {
        assert_eq!(MIN_TRADES_FOR_BAN, 30);
        assert!(BAN_WIN_RATE < REALLOW_WIN_RATE); // hysteresis: ban < reallow
    }

    #[test]
    fn test_filter_empty_list() {
        let (filtered, removed) = Deallow::filter_hype_list(&[]);
        assert!(filtered.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn test_filter_preserves_unknown() {
        // Symbols not in snapshot should pass through
        let list = vec!["UNKNOWNCOIN123".to_string()];
        let (filtered, removed) = Deallow::filter_hype_list(&list);
        assert_eq!(filtered.len(), 1);
        assert!(removed.is_empty());
    }
}
