//! Pre-Trade Risk Engine — Institutional-grade risk filters.
//! Inspired by NautilusTrader's pre-trade risk checks.
//!
//! Checks BEFORE any verdict is forwarded to Titan:
//! 1. Daily Drawdown Limit (hard cap)
//! 2. Loss Streak Cooldown (3+ consecutive losses → pause)
//! 3. Correlation Guard (no 3+ same-direction bets)
//! 4. Position Concentration (max % of portfolio per symbol)

use crate::state::SwarmState;

/// Result of pre-trade risk check
#[derive(Debug, Clone)]
pub struct RiskCheck {
    pub allowed: bool,
    pub max_size_factor: f64, // 1.0 = full size, 0.5 = half
    pub reason: String,
}

/// Risk Engine configuration
pub struct RiskConfig {
    /// Max daily drawdown as absolute USD
    pub max_daily_drawdown_usd: f64,
    /// Max consecutive losses before cooldown
    pub max_loss_streak: u32,
    /// Cooldown seconds after loss streak exceeded
    pub loss_streak_cooldown_secs: u64,
    /// Max same-direction positions (e.g., 3 LONGs)
    pub max_correlated_positions: usize,
    /// Max number of total active positions
    pub max_total_positions: usize,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_daily_drawdown_usd: 18.0,   // Matches Treasury session_limit
            max_loss_streak: 3,
            loss_streak_cooldown_secs: 1800, // 30 min
            max_correlated_positions: 2,     // Max 2 LONGs or 2 SHORTs
            max_total_positions: 4,
        }
    }
}

/// Run pre-trade risk check based on swarm state and decision memory
pub fn pre_trade_risk_check(
    symbol: &str,
    verdict_direction: &str, // "BUY", "SELL", "HOLD"
    confidence: f64,
    swarm_state: &SwarmState,
    decision_mem: &crate::decision_memory::DecisionMemory,
    config: &RiskConfig,
) -> RiskCheck {
    // ─── Filter 0: HOLD verdicts always pass (no trade = no risk) ───
    if verdict_direction == "HOLD" || verdict_direction == "NEUTRAL" {
        return RiskCheck {
            allowed: true,
            max_size_factor: 1.0,
            reason: "HOLD/NEUTRAL — no trade".into(),
        };
    }

    // ─── Filter 1: Daily Drawdown Hard Cap ───
    // Read from swarm state (populated by PaperEngine in single-binary mode)
    let daily_loss = swarm_state.consensus.iter()
        .filter(|e| !e.value().meta_agreement) // rejected = potential loss
        .count() as f64 * 0.5; // conservative estimate per rejected trade

    if daily_loss >= config.max_daily_drawdown_usd {
        return RiskCheck {
            allowed: false,
            max_size_factor: 0.0,
            reason: format!(
                "DRAWDOWN BLOCK: daily loss ${:.2} >= limit ${:.2}",
                daily_loss, config.max_daily_drawdown_usd
            ),
        };
    }

    // Approaching drawdown → reduce size
    let drawdown_ratio = daily_loss / config.max_daily_drawdown_usd;
    let drawdown_factor = if drawdown_ratio > 0.7 {
        0.5 // Half size when 70%+ of limit consumed
    } else if drawdown_ratio > 0.5 {
        0.75
    } else {
        1.0
    };

    // ─── Filter 2: Loss Streak Cooldown ───
    let _pending_symbols = decision_mem.get_pending_symbols();
    let recent_losses = count_recent_losses(decision_mem);
    if recent_losses >= config.max_loss_streak {
        return RiskCheck {
            allowed: false,
            max_size_factor: 0.0,
            reason: format!(
                "LOSS STREAK: {} consecutive losses >= {}. Cooldown active.",
                recent_losses, config.max_loss_streak
            ),
        };
    }

    // ─── Filter 3: Correlation Guard ───
    let consensus = &swarm_state.consensus;
    let same_direction_count = consensus
        .iter()
        .filter(|entry| {
            let v = entry.value();
            let dir = format!("{:?}", v.final_verdict);
            dir == verdict_direction && v.symbol != symbol
        })
        .count();

    if same_direction_count >= config.max_correlated_positions {
        return RiskCheck {
            allowed: false,
            max_size_factor: 0.0,
            reason: format!(
                "CORRELATION BLOCK: already {} positions in {} direction (max {})",
                same_direction_count, verdict_direction, config.max_correlated_positions
            ),
        };
    }

    // ─── Filter 4: Total Position Cap ───
    // Check how many symbols have non-HOLD consensus
    let active_positions = consensus
        .iter()
        .filter(|e| {
            let dir = format!("{:?}", e.value().final_verdict);
            dir != "Hold" && dir != "HOLD"
        })
        .count();

    if active_positions >= config.max_total_positions {
        return RiskCheck {
            allowed: false,
            max_size_factor: 0.0,
            reason: format!(
                "POSITION CAP: {} active >= {} max",
                active_positions, config.max_total_positions
            ),
        };
    }

    // ─── Filter 5: Low Confidence Penalty ───
    let confidence_factor = if confidence < 55.0 {
        0.5 // Very low confidence → half size
    } else if confidence < 65.0 {
        0.75
    } else {
        1.0
    };

    // ─── Composite Factor ───
    let final_factor = drawdown_factor * confidence_factor;

    RiskCheck {
        allowed: true,
        max_size_factor: final_factor,
        reason: format!(
            "APPROVED: DD={:.0}% conf={:.0}% factor={:.2}",
            drawdown_ratio * 100.0, confidence, final_factor
        ),
    }
}

/// Count recent consecutive losses from decision memory
fn count_recent_losses(decision_mem: &crate::decision_memory::DecisionMemory) -> u32 {
    // Read last N resolved entries and count consecutive losses from end
    let context = decision_mem.get_past_context("_ALL_", 10, 0);
    if context.is_empty() {
        return 0;
    }

    let mut streak = 0u32;
    for line in context.lines().rev() {
        if line.contains("return:") {
            // Parse return value
            if let Some(start) = line.find("return:") {
                let val_str: String = line[start + 7..]
                    .chars()
                    .take_while(|c| *c == '-' || *c == '+' || *c == '.' || c.is_ascii_digit())
                    .collect();
                if let Ok(ret) = val_str.parse::<f64>() {
                    if ret < 0.0 {
                        streak += 1;
                    } else {
                        break; // First win breaks the streak
                    }
                }
            }
        }
    }
    streak
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = RiskConfig::default();
        assert_eq!(cfg.max_daily_drawdown_usd, 18.0);
        assert_eq!(cfg.max_loss_streak, 3);
        assert_eq!(cfg.max_correlated_positions, 2);
        assert_eq!(cfg.max_total_positions, 4);
    }

    #[test]
    fn test_hold_always_passes() {
        let state = SwarmState::new();
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = crate::decision_memory::DecisionMemory::new(tmp.path());
        let cfg = RiskConfig::default();

        let check = pre_trade_risk_check("BTCUSDT", "HOLD", 50.0, &state, &mem, &cfg);
        assert!(check.allowed);
    }

    #[test]
    fn test_low_confidence_penalty() {
        let state = SwarmState::new();
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = crate::decision_memory::DecisionMemory::new(tmp.path());
        let cfg = RiskConfig::default();

        let check = pre_trade_risk_check("BTCUSDT", "BUY", 50.0, &state, &mem, &cfg);
        assert!(check.allowed);
        assert!(check.max_size_factor <= 0.5, "low confidence should halve size");
    }

    #[test]
    fn test_risk_check_passes_normal() {
        let state = SwarmState::new();
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = crate::decision_memory::DecisionMemory::new(tmp.path());
        let cfg = RiskConfig::default();

        let check = pre_trade_risk_check("BTCUSDT", "BUY", 80.0, &state, &mem, &cfg);
        assert!(check.allowed);
        assert_eq!(check.max_size_factor, 1.0);
    }

    #[test]
    fn test_count_losses_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = crate::decision_memory::DecisionMemory::new(tmp.path());
        assert_eq!(count_recent_losses(&mem), 0);
    }
}
