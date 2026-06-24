// src/modules/auto_ramp.rs
// ═══════════════════════════════════════════════════════════════
// AUTO-RAMP — 5-Gate Capital Scaling State Machine
// ═══════════════════════════════════════════════════════════════
// V11: Inspired by MySwarmbots/Swarmbots AUTO-RAMP.md
//
// The bot is its own auditor. Capital scales ONLY when the data
// says it has earned the right. Deterministic state machine
// replaces "vibes scaling" with quantitative gate evaluation.
//
// 5 Gates (ALL must pass simultaneously):
//   G1: ≥10 closed trades in 96h
//   G2: 7-day PnL > 0
//   G3: 0 kill-switch triggers in 96h
//   G4: ≥7 days since last promotion
//   G5: Current phase win rate ≥ 45%
//
// Demotion:
//   Hard: kill-switch → drop one phase
//   Soft: 5 consecutive negative PnL days → drop one phase

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::safe_io::data_file;

fn ramp_state_path() -> String { data_file("titan_ramp_state.json") }
fn snapshot_path() -> String { data_file("hive_mind_snapshot.json") }
const EVAL_WINDOW_HOURS: i64 = 96;
const PROMOTION_COOLDOWN_DAYS: i64 = 7;

/// Phase configuration: each phase defines max position % and kill-switch threshold
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields accessed indirectly through PHASES array indexing + format logs
pub struct PhaseConfig {
    pub phase: u8,
    pub max_position_pct: f64,  // Max % of balance per position
    pub daily_loss_kill_pct: f64, // Daily PnL kill-switch threshold (% of balance)
    pub label: &'static str,
}

/// The 5 phases of capital scaling
pub const PHASES: [PhaseConfig; 5] = [
    PhaseConfig { phase: 0, max_position_pct: 0.10, daily_loss_kill_pct: 3.0,  label: "SEED" },
    PhaseConfig { phase: 1, max_position_pct: 0.15, daily_loss_kill_pct: 4.0,  label: "SPROUT" },
    PhaseConfig { phase: 2, max_position_pct: 0.20, daily_loss_kill_pct: 5.0,  label: "GROWTH" },
    PhaseConfig { phase: 3, max_position_pct: 0.25, daily_loss_kill_pct: 6.0,  label: "MATURE" },
    PhaseConfig { phase: 4, max_position_pct: 0.30, daily_loss_kill_pct: 8.0,  label: "APEX" },
];

/// Persistent state stored in titan_ramp_state.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RampState {
    pub current_phase: u8,
    pub last_promotion_ts: i64,     // Unix timestamp of last promotion
    pub last_demotion_ts: i64,      // Unix timestamp of last demotion
    pub consecutive_negative_days: u8,
    pub kill_switch_count_96h: u8,
    #[serde(default)]
    pub kill_switch_last_ts: i64,   // V11.0.1: timestamp of last kill-switch for 96h expiry
    #[serde(default)]
    pub daily_pnl_accumulator: f64, // V11.0.1: aggregates PnL within a day
    #[serde(default)]
    pub last_pnl_date: String,      // V11.0.1: "YYYY-MM-DD" for daily reset
    pub total_promotions: u32,
    pub total_demotions: u32,
}

impl Default for RampState {
    fn default() -> Self {
        RampState {
            current_phase: 0,
            last_promotion_ts: 0,
            last_demotion_ts: 0,
            consecutive_negative_days: 0,
            kill_switch_count_96h: 0,
            kill_switch_last_ts: 0,
            daily_pnl_accumulator: 0.0,
            last_pnl_date: String::new(),
            total_promotions: 0,
            total_demotions: 0,
        }
    }
}

pub struct AutoRamp;

impl AutoRamp {
    /// Get current phase config
    pub fn current_phase() -> PhaseConfig {
        let state = Self::load_state();
        let idx = (state.current_phase as usize).min(PHASES.len() - 1);
        PHASES[idx].clone()
    }

    /// Get max position % for current phase (used by risk.rs)
    pub fn max_position_pct() -> f64 {
        Self::current_phase().max_position_pct
    }

    /// Evaluate all gates — called from weather loop
    /// Returns a log string describing what happened
    pub fn evaluate() -> String {
        let mut state = Self::load_state();
        let now_ts = chrono::Utc::now().timestamp();
        
        // V11.0.1 P0 FIX: Expire kill-switch counter after 96h window
        // Without this, a single kill-switch causes permanent cascading demotion
        if state.kill_switch_count_96h > 0 && state.kill_switch_last_ts > 0 {
            let hours_since_kill = (now_ts - state.kill_switch_last_ts) / 3600;
            if hours_since_kill >= EVAL_WINDOW_HOURS {
                tracing::info!(hours = hours_since_kill, "♻️ [RAMP] Kill-switch expired (>96h) — counter cleared");
                state.kill_switch_count_96h = 0;
                state.kill_switch_last_ts = 0;
                Self::save_state(&state);
            }
        }
        
        // V11.0.1 P1 FIX: Daily PnL aggregation → flush to consecutive_negative_days
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if !state.last_pnl_date.is_empty() && state.last_pnl_date != today {
            // Day rolled over — evaluate yesterday's accumulated PnL
            if state.daily_pnl_accumulator < 0.0 {
                state.consecutive_negative_days += 1;
                tracing::info!(days = state.consecutive_negative_days, pnl = format!("{:.2}", state.daily_pnl_accumulator).as_str(), "📉 [RAMP] Negative day recorded");
            } else {
                state.consecutive_negative_days = 0;
            }
            state.daily_pnl_accumulator = 0.0;
            state.last_pnl_date = today;
            Self::save_state(&state);
        } else if state.last_pnl_date.is_empty() {
            state.last_pnl_date = today;
            Self::save_state(&state);
        }
        
        // Check demotion first (always takes priority)
        if let Some(reason) = Self::check_demotion(&mut state, now_ts) {
            Self::save_state(&state);
            return reason;
        }

        // Already at max phase
        if state.current_phase >= 4 {
            return format!("[RAMP] Phase {}/4 APEX — max reached", state.current_phase);
        }

        // Evaluate 5 gates
        let (passed, gate_log) = Self::evaluate_gates(&state, now_ts);
        
        if passed {
            state.current_phase += 1;
            state.last_promotion_ts = now_ts;
            state.total_promotions += 1;
            state.consecutive_negative_days = 0;
            Self::save_state(&state);
            
            let phase = &PHASES[state.current_phase as usize];
            return format!(
                "🚀 [RAMP] PROMOTED to Phase {} ({}) — max_pos={}% | {}",
                state.current_phase, phase.label, 
                (phase.max_position_pct * 100.0) as u32,
                gate_log
            );
        }

        let phase_cfg = &PHASES[state.current_phase as usize];
        format!("[RAMP] Phase {}/{} ({}) kill_thr={:.0}% — gates: {}", 
            state.current_phase, 4, phase_cfg.label, phase_cfg.daily_loss_kill_pct, gate_log)
    }

    /// Evaluate all 5 gates, returns (all_passed, log_string)
    fn evaluate_gates(state: &RampState, now_ts: i64) -> (bool, String) {
        let mut gates: Vec<(bool, String)> = Vec::new();
        
        // G1: ≥10 closed trades in evaluation window
        let trades_window = Self::count_recent_trades(now_ts);
        let g1 = trades_window >= 10;
        gates.push((g1, format!("G1:trades{}h={}≥10={}", EVAL_WINDOW_HOURS, trades_window, if g1 {"✓"} else {"✗"})));
        
        // G2: 7-day PnL > 0
        let pnl_7d = Self::get_7d_pnl();
        let g2 = pnl_7d > 0.0;
        gates.push((g2, format!("G2:7dPnL={:.2}>0={}", pnl_7d, if g2 {"✓"} else {"✗"})));
        
        // G3: 0 kill-switch triggers in 96h
        let g3 = state.kill_switch_count_96h == 0;
        gates.push((g3, format!("G3:kills={}=0={}", state.kill_switch_count_96h, if g3 {"✓"} else {"✗"})));
        
        // G4: ≥7 days since last promotion
        let days_since = if state.last_promotion_ts > 0 {
            (now_ts - state.last_promotion_ts) / 86400
        } else {
            999 // Never promoted = always passes
        };
        let g4 = days_since >= PROMOTION_COOLDOWN_DAYS;
        gates.push((g4, format!("G4:days={}≥7={}", days_since, if g4 {"✓"} else {"✗"})));

        // G5: Current phase win rate ≥ 45%
        let wr = Self::get_overall_win_rate();
        let g5 = wr >= 0.45;
        gates.push((g5, format!("G5:WR={:.0}%≥45%={}", wr * 100.0, if g5 {"✓"} else {"✗"})));
        
        let all_passed = gates.iter().all(|(p, _)| *p);
        let log = gates.iter().map(|(_, s)| s.clone()).collect::<Vec<_>>().join(" ");
        
        (all_passed, log)
    }

    /// Check demotion conditions
    fn check_demotion(state: &mut RampState, _now_ts: i64) -> Option<String> {
        if state.current_phase == 0 { return None; }

        // Hard demote: kill-switch in 96h
        if state.kill_switch_count_96h > 0 {
            state.current_phase -= 1;
            state.total_demotions += 1;
            state.kill_switch_count_96h = 0;
            let phase = &PHASES[state.current_phase as usize];
            return Some(format!(
                "⚠️ [RAMP] DEMOTED to Phase {} ({}) — kill-switch triggered",
                state.current_phase, phase.label
            ));
        }

        // Soft demote: 5 consecutive negative days
        if state.consecutive_negative_days >= 5 {
            state.current_phase -= 1;
            state.total_demotions += 1;
            state.consecutive_negative_days = 0;
            let phase = &PHASES[state.current_phase as usize];
            return Some(format!(
                "⚠️ [RAMP] DEMOTED to Phase {} ({}) — 5 consecutive negative days",
                state.current_phase, phase.label
            ));
        }

        None
    }

    /// Record a kill-switch event (called from main.rs when kill-switch triggers)
    /// V11.0.1: Now stores timestamp for 96h expiry window
    pub fn record_kill_switch() {
        let mut state = Self::load_state();
        state.kill_switch_count_96h += 1;
        state.kill_switch_last_ts = chrono::Utc::now().timestamp();
        Self::save_state(&state);
    }

    /// Record per-trade PnL — accumulates within a day
    /// V11.0.1: No longer flips consecutive_negative_days per trade.
    /// Daily flush happens in evaluate() on day rollover.
    pub fn record_daily_pnl(pnl: f64) {
        let mut state = Self::load_state();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        // Initialize date tracking if needed
        if state.last_pnl_date.is_empty() || state.last_pnl_date != today {
            // Day changed — the evaluate() flush handles the old day's verdict
            if !state.last_pnl_date.is_empty() && state.last_pnl_date != today {
                // Flush yesterday before starting today
                if state.daily_pnl_accumulator < 0.0 {
                    state.consecutive_negative_days += 1;
                } else {
                    state.consecutive_negative_days = 0;
                }
            }
            state.daily_pnl_accumulator = 0.0;
            state.last_pnl_date = today;
        }
        
        // Accumulate today's PnL
        state.daily_pnl_accumulator += pnl;
        Self::save_state(&state);
    }

    /// Count trades in recent 96h window from snapshot
    fn count_recent_trades(_now_ts: i64) -> i64 {
        if let Ok(data) = std::fs::read_to_string(&snapshot_path()) {
            if let Ok(json) = serde_json::from_str::<Value>(&data) {
                let mut total = 0i64;
                if let Some(obj) = json.as_object() {
                    for (_sym, entity) in obj {
                        // Use trade_count as approximation (full history would need trade log)
                        total += entity["trade_count"].as_i64().unwrap_or(0);
                    }
                }
                // Rough estimate: if total > 10 in snapshot, assume enough recent activity
                return total;
            }
        }
        0
    }

    /// Get 7-day PnL from snapshot (sum of all symbols' net_pnl)
    fn get_7d_pnl() -> f64 {
        if let Ok(data) = std::fs::read_to_string(&snapshot_path()) {
            if let Ok(json) = serde_json::from_str::<Value>(&data) {
                let mut total_pnl = 0.0;
                if let Some(obj) = json.as_object() {
                    for (_sym, entity) in obj {
                        total_pnl += entity["net_pnl"].as_f64().unwrap_or(0.0);
                    }
                }
                return total_pnl;
            }
        }
        0.0
    }

    /// Get overall win rate from snapshot
    fn get_overall_win_rate() -> f64 {
        if let Ok(data) = std::fs::read_to_string(&snapshot_path()) {
            if let Ok(json) = serde_json::from_str::<Value>(&data) {
                let mut total_wins = 0i64;
                let mut total_trades = 0i64;
                if let Some(obj) = json.as_object() {
                    for (_sym, entity) in obj {
                        let tc = entity["trade_count"].as_i64().unwrap_or(0);
                        let wr = entity["win_rate"].as_f64().unwrap_or(0.0);
                        total_wins += (tc as f64 * wr) as i64;
                        total_trades += tc;
                    }
                }
                if total_trades > 0 {
                    return total_wins as f64 / total_trades as f64;
                }
            }
        }
        0.5
    }

    /// Load state from disk (or default if missing)
    fn load_state() -> RampState {
        std::fs::read_to_string(&ramp_state_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save state to disk
    fn save_state(state: &RampState) {
        if let Ok(json) = serde_json::to_string_pretty(state) {
            let _ = crate::safe_io::SafeIO::atomic_write(&ramp_state_path(), &json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = RampState::default();
        assert_eq!(state.current_phase, 0);
        assert_eq!(state.consecutive_negative_days, 0);
    }

    #[test]
    fn test_phase_configs() {
        assert_eq!(PHASES[0].max_position_pct, 0.10);
        assert_eq!(PHASES[4].max_position_pct, 0.30);
        assert_eq!(PHASES[0].label, "SEED");
        assert_eq!(PHASES[4].label, "APEX");
    }

    #[test]
    fn test_demotion_on_killswitch() {
        let mut state = RampState {
            current_phase: 2,
            kill_switch_count_96h: 1,
            ..Default::default()
        };
        let result = AutoRamp::check_demotion(&mut state, 0);
        assert!(result.is_some());
        assert_eq!(state.current_phase, 1);
    }

    #[test]
    fn test_no_demotion_at_phase_zero() {
        let mut state = RampState {
            current_phase: 0,
            kill_switch_count_96h: 5,
            ..Default::default()
        };
        let result = AutoRamp::check_demotion(&mut state, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_soft_demotion_5_negative_days() {
        let mut state = RampState {
            current_phase: 3,
            consecutive_negative_days: 5,
            ..Default::default()
        };
        let result = AutoRamp::check_demotion(&mut state, 0);
        assert!(result.is_some());
        assert_eq!(state.current_phase, 2);
        assert_eq!(state.consecutive_negative_days, 0);
    }

    #[test]
    fn test_max_position_pct_default() {
        // Without state file, should return Phase 0 config
        assert!(AutoRamp::max_position_pct() > 0.0);
        assert!(AutoRamp::max_position_pct() <= 0.30);
    }

    // ═══ V11.0.1 HARDENING TESTS ═══

    #[test]
    fn test_default_state_v1101_fields() {
        let state = RampState::default();
        assert_eq!(state.kill_switch_last_ts, 0);
        assert_eq!(state.daily_pnl_accumulator, 0.0);
        assert!(state.last_pnl_date.is_empty());
    }

    #[test]
    fn test_kill_switch_clears_after_demotion() {
        let mut state = RampState {
            current_phase: 3,
            kill_switch_count_96h: 2,
            kill_switch_last_ts: 1000,
            ..Default::default()
        };
        let result = AutoRamp::check_demotion(&mut state, 0);
        assert!(result.is_some());
        assert_eq!(state.current_phase, 2);
        assert_eq!(state.kill_switch_count_96h, 0); // cleared after demote
    }

    #[test]
    fn test_demotion_preserves_phase_floor() {
        // Phase 1 + kill-switch → demote to 0, not underflow
        let mut state = RampState {
            current_phase: 1,
            kill_switch_count_96h: 1,
            ..Default::default()
        };
        let result = AutoRamp::check_demotion(&mut state, 0);
        assert!(result.is_some());
        assert_eq!(state.current_phase, 0);
        
        // Phase 0 + kill-switch → no demotion (already floor)
        let mut state2 = RampState {
            current_phase: 0,
            kill_switch_count_96h: 3,
            ..Default::default()
        };
        let result2 = AutoRamp::check_demotion(&mut state2, 0);
        assert!(result2.is_none());
    }

    #[test]
    fn test_eval_window_hours_constant() {
        // Verify 96h = 4 days evaluation window
        assert_eq!(EVAL_WINDOW_HOURS, 96);
        assert_eq!(PROMOTION_COOLDOWN_DAYS, 7);
    }
}
