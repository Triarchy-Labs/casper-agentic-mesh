// src/modules/trailing.rs
// ═══════════════════════════════════════════════════════════════
// TRAILING ENGINE — Server-side ATR-based Stop-Loss Management
// ═══════════════════════════════════════════════════════════════
// Вынесено из main.rs:352-391 в отдельный модуль (ARCHITECTURE RULE #1).
//
// Ответственность:
//   - Расчёт нового trailing SL для LONG и SHORT
//   - Curator BE-Lock (breakeven при >0.15% profit)
//   - Ratchet guard (SHORT SL never widens)
//   - Profit-lock threshold (1.5% → lock at +0.3%)

use crate::confidence::ConfidenceEngine;

/// Конфигурация Trailing Engine
pub struct TrailingConfig {
    /// Порог прибыли для lock-in стопа (%)
    pub profit_lock_threshold_pct: f64,
    /// Множитель прибыли для lock-in (1.003 = +0.3%)
    pub profit_lock_entry_mult_long: f64,
    /// Множитель для SHORT lock-in (0.997 = -0.3%)
    pub profit_lock_entry_mult_short: f64,
    /// Порог для BE-lock curator (%)
    pub curator_be_threshold_pct: f64,
    /// Fee cushion для BE-lock (0.0015 = 0.15%)
    pub curator_fee_cushion: f64,
    /// Ratchet tolerance для LONG (1.002 = 0.2%)
    pub long_ratchet_tolerance: f64,
    /// Ratchet tolerance для SHORT (0.998 = 0.2%)
    pub short_ratchet_tolerance: f64,
}

impl Default for TrailingConfig {
    fn default() -> Self {
        Self {
            profit_lock_threshold_pct: 1.5,
            profit_lock_entry_mult_long: 1.003,
            profit_lock_entry_mult_short: 0.997,
            curator_be_threshold_pct: 0.15,
            curator_fee_cushion: 0.0015,
            long_ratchet_tolerance: 1.002,
            short_ratchet_tolerance: 0.998,
        }
    }
}

/// Результат расчёта trailing
#[derive(Debug, Clone)]
#[allow(dead_code)] // CuratorBeLock variant reserved for future curator reporting
pub enum TrailingVerdict {
    /// SL не изменился (нет улучшения)
    NoChange,
    /// Curator BE-Lock активирован
    CuratorBeLock { new_sl: f64 },
    /// Trailing SL подтянут
    Tightened { new_sl: f64, reason: String },
    /// V11: Adverse Selection — profit reached 0.7R then reversed below 0.4R → market close
    AdverseSelection { peak_r: f64, current_r: f64 },
}

pub struct TrailingEngine;

impl TrailingEngine {
    /// Расчёт Curator BE-Lock
    pub fn check_curator_be_lock(
        side: &str,
        entry_price: f64,
        current_price: f64,
        last_pushed_sl: f64,
        config: &TrailingConfig,
    ) -> Option<f64> {
        let unrealized_pct = if side == "Buy" {
            (current_price - entry_price) / entry_price * 100.0
        } else {
            (entry_price - current_price) / entry_price * 100.0
        };

        if unrealized_pct > config.curator_be_threshold_pct {
            let be_price = if side == "Buy" {
                entry_price * (1.0 + config.curator_fee_cushion)
            } else {
                entry_price * (1.0 - config.curator_fee_cushion)
            };
            let should_lock = if side == "Buy" {
                last_pushed_sl < be_price
            } else {
                last_pushed_sl == 0.0 || last_pushed_sl > be_price
            };
            if should_lock { return Some(be_price); }
        }
        None
    }

    /// V11: Adverse Selection Guard (Swarmbots RISK-MODEL inspired)
    /// Detects trades that reached ≥0.7R profit then reversed below 0.4R
    /// V11.0.1: Added minimum holding time (15min) to prevent false triggers on memecoins
    pub fn check_adverse_selection(
        side: &str,
        entry_price: f64,
        current_price: f64,
        highest_price: f64,
        lowest_price: f64,
        atr: f64,
        entry_time_ms: i64,
    ) -> Option<TrailingVerdict> {
        // V11.0.1 P1 FIX: Don't trigger on positions held < 15 minutes
        // Memecoins oscillate fast — need breathing room
        let now_ms = chrono::Utc::now().timestamp_millis();
        let holding_minutes = (now_ms - entry_time_ms) / 60_000;
        if holding_minutes < 15 { return None; }
        
        let stop_distance = atr * 3.5; // approximate R using default ATR mult
        if stop_distance <= 0.0 { return None; }

        let (peak_profit, current_profit) = if side == "Buy" {
            (highest_price - entry_price, current_price - entry_price)
        } else {
            (entry_price - lowest_price, entry_price - current_price)
        };

        let peak_r = peak_profit / stop_distance;
        let current_r = current_profit / stop_distance;

        // If we reached +0.7R but reversed below +0.4R → adverse selection
        if peak_r >= 0.7 && current_r < 0.4 && current_r > 0.0 {
            return Some(TrailingVerdict::AdverseSelection {
                peak_r,
                current_r,
            });
        }
        None
    }

    /// Расчёт нового trailing SL для позиции
    /// Возвращает новый SL если он улучшился, иначе None
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_trailing_sl(
        side: &str,
        entry_price: f64,
        current_price: f64,
        highest_price: f64,
        lowest_price: f64,
        atr: f64,
        atr_multiplier: f64,
        last_pushed_sl: f64,
        symbol: &str,
        config: &TrailingConfig,
    ) -> TrailingVerdict {
        // Adaptive ATR multiplier from DNA
        let adaptive_mult = ConfidenceEngine::get_adaptive_atr_mult(symbol, atr_multiplier);

        if side == "Buy" {
            let hp = if current_price > highest_price { current_price } else { highest_price };
            let actual_drop = atr * adaptive_mult;

            // Profit-lock: если макс. прибыль > 1.5%, ставим стоп на +0.3%
            let mut stop = 0.0;
            let max_profit_pct = (hp - entry_price) / entry_price * 100.0;
            if max_profit_pct > config.profit_lock_threshold_pct {
                stop = entry_price * config.profit_lock_entry_mult_long;
            }

            let trail = hp - actual_drop;
            if trail > stop { stop = trail; }

            let mut final_sl = entry_price - actual_drop;
            if stop > final_sl { final_sl = stop; }

            // Ratchet: only tighten, never widen
            if final_sl > last_pushed_sl * config.long_ratchet_tolerance {
                return TrailingVerdict::Tightened {
                    new_sl: final_sl,
                    reason: format!("LONG trail {final_sl:.4} (hp={hp:.4}, drop={actual_drop:.4})"),
                };
            }
        } else {
            let lp = if lowest_price == 0.0 || current_price < lowest_price { current_price } else { lowest_price };
            let actual_bounce = atr * adaptive_mult;

            let mut stop = f64::MAX;
            let max_profit_pct = (entry_price - lp) / entry_price * 100.0;
            if max_profit_pct > config.profit_lock_threshold_pct {
                stop = entry_price * config.profit_lock_entry_mult_short;
            }

            let trail = lp + actual_bounce;
            if trail < stop { stop = trail; }

            let mut final_sl = entry_price + (atr * adaptive_mult);
            if stop < final_sl { final_sl = stop; }

            // FORENSIC-10: explicit ratchet — never WIDEN restored SL for SHORT
            if last_pushed_sl > 0.0 && final_sl > last_pushed_sl {
                final_sl = last_pushed_sl;
            }

            // Ratchet: only tighten (for SHORT: new < old)
            if last_pushed_sl == 0.0 || final_sl < last_pushed_sl * config.short_ratchet_tolerance {
                return TrailingVerdict::Tightened {
                    new_sl: final_sl,
                    reason: format!("SHORT trail {final_sl:.4} (lp={lp:.4}, bounce={actual_bounce:.4})"),
                };
            }
        }

        TrailingVerdict::NoChange
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adverse_selection_blocks_young_position() {
        // Position held < 15 min → should NOT trigger adverse selection
        let now_ms = chrono::Utc::now().timestamp_millis();
        let entry_5min_ago = now_ms - 5 * 60_000; // 5 minutes ago
        
        let result = TrailingEngine::check_adverse_selection(
            "Buy", 100.0, 100.5, 101.0, 99.0, 0.3, entry_5min_ago,
        );
        assert!(result.is_none(), "Should not trigger on young position (<15min)");
    }

    #[test]
    fn test_adverse_selection_triggers_on_reversal() {
        // Position held > 15 min, peak ≥ 0.7R, current < 0.4R
        let now_ms = chrono::Utc::now().timestamp_millis();
        let entry_30min_ago = now_ms - 30 * 60_000;
        let atr = 1.0; // stop_distance = 3.5
        // peak = 3.0 (peak_r = 0.86), current = 1.0 (current_r = 0.29)
        let result = TrailingEngine::check_adverse_selection(
            "Buy", 100.0, 101.0, 103.0, 99.0, atr, entry_30min_ago,
        );
        assert!(result.is_some(), "Should trigger: peak 0.86R → current 0.29R");
    }

    #[test]
    fn test_adverse_selection_no_trigger_still_healthy() {
        // Position held > 15 min, peak ≥ 0.7R, current still ≥ 0.4R → NO trigger
        let now_ms = chrono::Utc::now().timestamp_millis();
        let entry_30min_ago = now_ms - 30 * 60_000;
        let atr = 1.0; // stop_distance = 3.5
        // peak = 3.0 (peak_r = 0.86), current = 2.0 (current_r = 0.57)
        let result = TrailingEngine::check_adverse_selection(
            "Buy", 100.0, 102.0, 103.0, 99.0, atr, entry_30min_ago,
        );
        assert!(result.is_none(), "Should NOT trigger: current 0.57R > 0.4R threshold");
    }

    #[test]
    fn test_adverse_zero_atr_safe() {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let entry_old = now_ms - 60 * 60_000;
        // ATR = 0 → stop_distance = 0 → should return None safely
        let result = TrailingEngine::check_adverse_selection(
            "Buy", 100.0, 101.0, 103.0, 99.0, 0.0, entry_old,
        );
        assert!(result.is_none(), "Zero ATR should return None safely");
    }
}
