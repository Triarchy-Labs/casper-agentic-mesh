/// Institutional Risk Sizing — Fixed Risk & Kelly Hybrid Position Sizing.
///
/// ПОРТИРОВАНО ИЗ:
///   - nautilus_trader/crates/risk/src/sizing.rs (338 строк) — Формула Fixed Risk
///   - tradememory-protocol/src/tradememory/owm/kelly.py — Kelly Criterion
///
/// Комбинирует два подхода:
///   1. Fixed Risk: size = riskable_money / (risk_ticks × tick_value)
///      Nautilus: Nautech Systems Pty Ltd (LGPL-3.0)
///   2. Kelly Criterion: f* = p/a - q/b (generalized weighted)
///      TradMemory: mnemox-ai (MIT)
///
/// Для Hive Mind: выбираем min(fixed_risk, kelly) — консервативный подход.
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Fixed Risk Position Sizing (порт: nautilus sizing.rs:34-101)
// ═══════════════════════════════════════════════════════════════

/// Fixed-risk position sizing.
///
/// Формула: size = riskable_money / (risk_ticks × tick_value × exchange_rate)
///
/// Порт: nautilus_trader/crates/risk/src/sizing.rs:34-85
#[derive(Debug, Clone, Serialize)]
pub struct FixedRiskParams {
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub equity: f64,
    pub risk_pct: f64,           // e.g., 0.01 = 1%
    pub commission_rate: f64,    // e.g., 0.0002 = 0.02%
    pub exchange_rate: f64,      // quote-to-account currency rate
    pub tick_size: f64,          // minimum price increment
    pub hard_limit: Option<f64>, // absolute max position size
    pub unit_batch_size: f64,    // round to nearest batch (e.g., 1000)
}

/// Calculate position size using fixed-risk method.
/// Порт: nautilus_trader/crates/risk/src/sizing.rs:34-85
pub fn fixed_risk_position_size(params: &FixedRiskParams) -> f64 {
    if params.exchange_rate <= 0.0 || params.equity <= 0.0 {
        return 0.0;
    }

    // risk_ticks = |entry - stop| / tick_size
    // Порт: sizing.rs:88-90
    let risk_ticks = (params.entry_price - params.stop_loss_price).abs() / params.tick_size;

    if risk_ticks <= 0.0 {
        return 0.0;
    }

    // riskable_money = equity × risk% - commission (round-turn)
    // Порт: sizing.rs:92-101
    let risk_money = params.equity * params.risk_pct;
    let commission = risk_money * params.commission_rate * 2.0; // round-turn
    let riskable = risk_money - commission;

    if riskable <= 0.0 {
        return 0.0;
    }

    // position_size = riskable / (risk_ticks × tick_size × exchange_rate)
    let mut size = (riskable / params.exchange_rate) / (risk_ticks * params.tick_size);

    // Apply hard limit
    if let Some(limit) = params.hard_limit {
        size = size.min(limit);
    }

    // Round to batch size
    if params.unit_batch_size > 0.0 {
        size = (size / params.unit_batch_size).floor() * params.unit_batch_size;
    }

    size.max(0.0)
}

// ═══════════════════════════════════════════════════════════════
// Kelly Criterion (порт: tradememory kelly.py + dqs.rs)
// ═══════════════════════════════════════════════════════════════

/// Generalized Kelly Criterion: f* = p/a - q/b
///
/// p = weighted win probability
/// q = weighted loss probability (1 - p)
/// a = average loss magnitude
/// b = average win magnitude
///
/// Half-Kelly: f*/2 (standard risk reduction)
///
/// Порт: tradememory/owm/kelly.py
pub fn kelly_fraction(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
    if avg_win <= 0.0 || avg_loss <= 0.0 || win_rate <= 0.0 || win_rate >= 1.0 {
        return 0.0;
    }

    let p = win_rate;
    let q = 1.0 - p;
    let a = avg_loss.abs(); // loss magnitude (positive)
    let b = avg_win.abs();  // win magnitude (positive)

    let f_star = (p / a) - (q / b);

    // Clamp to [0, 1]
    f_star.clamp(0.0, 1.0)
}

/// Half-Kelly — industry standard conservative approach.
pub fn half_kelly(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
    kelly_fraction(win_rate, avg_win, avg_loss) * 0.5
}

// ═══════════════════════════════════════════════════════════════
// Hybrid Sizing: min(Fixed Risk, Kelly)
// ═══════════════════════════════════════════════════════════════

/// Hybrid position sizing — conservative minimum of Fixed Risk and Kelly.
///
/// 1. Calculate Fixed Risk size from params
/// 2. Calculate Kelly fraction from historical performance
/// 3. Kelly-adjusted size = equity × kelly_f × leverage_factor
/// 4. Final = min(fixed_risk_size, kelly_size)
///
/// This prevents both:
///   - Over-sizing (Kelly alone can be aggressive)
///   - Under-sizing (Fixed risk alone ignores edge quality)
pub fn hybrid_position_size(
    params: &FixedRiskParams,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
) -> HybridSizeResult {
    let fixed = fixed_risk_position_size(params);
    let kelly_f = half_kelly(win_rate, avg_win, avg_loss);

    // Kelly-adjusted size: equity × kelly_fraction / tick_value
    let kelly_size = if kelly_f > 0.0 && params.tick_size > 0.0 {
        (params.equity * kelly_f) / (params.tick_size * params.exchange_rate.max(1.0))
    } else {
        0.0
    };

    let final_size = if kelly_size > 0.0 {
        fixed.min(kelly_size)
    } else {
        fixed
    };

    HybridSizeResult {
        fixed_risk_size: fixed,
        kelly_fraction: kelly_f,
        kelly_adjusted_size: kelly_size,
        final_size,
        method_used: if kelly_size > 0.0 && kelly_size < fixed {
            SizingMethod::Kelly
        } else {
            SizingMethod::FixedRisk
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HybridSizeResult {
    pub fixed_risk_size: f64,
    pub kelly_fraction: f64,
    pub kelly_adjusted_size: f64,
    pub final_size: f64,
    pub method_used: SizingMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SizingMethod {
    FixedRisk,
    Kelly,
}

// ═══════════════════════════════════════════════════════════════
// Adaptive Risk Engine (порт: tradememory/adaptive_risk.py)
// ═══════════════════════════════════════════════════════════════

/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/adaptive_risk.py (431 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// 5 алгоритмов → единый RiskConstraints:
///   1. Kelly → risk_per_trade_pct
///   2. Drawdown scale (DD>10% → 0.5x, >5% → 0.75x)
///   3. Session adjustments (WR per session → lot multiplier)
///   4. Consecutive losses → STOPPED/REDUCED/ACTIVE
///   5. Daily loss limit → hard stop
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum RiskStatus {
    Active,   // Normal trading
    Reduced,  // Approaching limits → scale 0.5x
    Stopped,  // Hard stop — no trading
}

impl RiskStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Reduced => "reduced",
            Self::Stopped => "stopped",
        }
    }

    fn priority(&self) -> u8 {
        match self {
            Self::Active => 0,
            Self::Reduced => 1,
            Self::Stopped => 2,
        }
    }
}

/// Результат адаптивного расчёта риска.
#[derive(Debug, Clone, Serialize)]
pub struct RiskConstraints {
    pub status: RiskStatus,
    pub reason: String,
    pub scale_factor: f64,         // 0.0 - 1.0 global position multiplier
    pub risk_per_trade_pct: f64,   // 0.5% - 5.0% (Kelly-driven)
    pub kelly_fraction: f64,       // raw quarter-Kelly
    pub session_adjustments: [f64; 3], // [asian, london, new_york]
}

/// Адаптивный движок рисков.
/// Порт: adaptive_risk.py:36-424 (AdaptiveRisk class)
pub struct AdaptiveRiskEngine {
    pub consecutive_loss_limit: u32,
    pub daily_loss_limit: f64,
}

impl AdaptiveRiskEngine {
    pub fn new(consecutive_loss_limit: u32, daily_loss_limit: f64) -> Self {
        Self {
            consecutive_loss_limit,
            daily_loss_limit,
        }
    }

    /// Рассчитать risk constraints из PnL и метаданных.
    ///
    /// Порт: adaptive_risk.py:357-424 (_combine_constraints)
    ///
    /// * `pnl_history` — хронологический массив PnL
    /// * `session_pnls` — [asian_pnls, london_pnls, new_york_pnls]
    /// * `today_realized_loss` — сумма реализованных лоссов за текущий день
    /// * `win_rate` — текущий win rate
    /// * `avg_win` — средний выигрыш
    /// * `avg_loss` — средний проигрыш (положительное число)
    pub fn calculate_constraints(
        &self,
        pnl_history: &[f64],
        session_pnls: &[Vec<f64>; 3],  // [asian, london, ny]
        today_realized_loss: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> RiskConstraints {
        if pnl_history.len() < 5 {
            return RiskConstraints {
                status: RiskStatus::Active,
                reason: "Insufficient data — using safe defaults".to_string(),
                scale_factor: 1.0,
                risk_per_trade_pct: 2.0,
                kelly_fraction: 0.0,
                session_adjustments: [0.75, 0.75, 0.75], // conservative
            };
        }

        // 1. Kelly → risk_per_trade_pct (порт: adaptive_risk.py:198-226)
        let kelly_f = half_kelly(win_rate, avg_win, avg_loss);
        let risk_pct = if kelly_f > 0.0 {
            (kelly_f * 100.0).clamp(0.5, 5.0)
        } else {
            2.0 // default when no edge
        };

        // 2. Drawdown scale (порт: adaptive_risk.py:228-259)
        let dd_scale = Self::drawdown_scale(pnl_history);

        // 3. Session adjustments (порт: adaptive_risk.py:261-298)
        let sess_adj = [
            Self::session_multiplier(&session_pnls[0]),
            Self::session_multiplier(&session_pnls[1]),
            Self::session_multiplier(&session_pnls[2]),
        ];

        // 4. Consecutive losses → status (порт: adaptive_risk.py:300-327)
        let consec_status = Self::check_consecutive_losses(pnl_history, self.consecutive_loss_limit);

        // 5. Daily loss → status (порт: adaptive_risk.py:329-351)
        let daily_status = self.check_daily_loss(today_realized_loss);

        // Worst status wins (порт: adaptive_risk.py:372-381)
        let worst = if consec_status.priority() >= daily_status.priority() {
            consec_status
        } else {
            daily_status
        };

        // Build reason
        let mut reasons = Vec::new();
        if worst == RiskStatus::Stopped {
            if consec_status == RiskStatus::Stopped {
                reasons.push(format!("Consecutive loss limit ({}) reached", self.consecutive_loss_limit));
            }
            if daily_status == RiskStatus::Stopped {
                reasons.push(format!("Daily loss limit (${:.0}) exceeded", self.daily_loss_limit));
            }
        } else if worst == RiskStatus::Reduced {
            if consec_status == RiskStatus::Reduced {
                reasons.push("Approaching consecutive loss limit".to_string());
            }
            if daily_status == RiskStatus::Reduced {
                reasons.push("Approaching daily loss limit (>80%)".to_string());
            }
        }

        // Apply extra 0.5x for REDUCED status (порт: adaptive_risk.py:401-403)
        let final_scale = if worst == RiskStatus::Reduced {
            dd_scale * 0.5
        } else {
            dd_scale
        };

        let reason_text = if reasons.is_empty() {
            "Calculated from trade history".to_string()
        } else {
            reasons.join("; ")
        };

        RiskConstraints {
            status: worst,
            reason: reason_text,
            scale_factor: final_scale,
            risk_per_trade_pct: risk_pct,
            kelly_fraction: kelly_f,
            session_adjustments: sess_adj,
        }
    }

    /// Drawdown → scale factor.
    /// Порт: adaptive_risk.py:228-259
    /// DD > 10% → 0.5x, > 5% → 0.75x, else 1.0
    fn drawdown_scale(pnl_history: &[f64]) -> f64 {
        if pnl_history.is_empty() { return 1.0; }

        let equity_base = 10_000.0;
        let mut cumulative = 0.0;
        let mut peak = equity_base;
        let mut max_dd_pct = 0.0;

        for &pnl in pnl_history {
            cumulative += pnl;
            let equity = equity_base + cumulative;
            if equity > peak {
                peak = equity;
            }
            if peak > 0.0 {
                let dd_pct = (peak - equity) / peak;
                if dd_pct > max_dd_pct {
                    max_dd_pct = dd_pct;
                }
            }
        }

        if max_dd_pct > 0.10 {
            0.5
        } else if max_dd_pct > 0.05 {
            0.75
        } else {
            1.0
        }
    }

    /// Per-session lot multiplier based on win rate.
    /// Порт: adaptive_risk.py:261-298
    /// WR < 40% → 0.5x, < 50% → 0.75x, else 1.0
    /// Insufficient data (< 3 trades) → 0.75 (conservative)
    fn session_multiplier(session_pnls: &[f64]) -> f64 {
        if session_pnls.len() < 3 {
            return 0.75;
        }
        let wins = session_pnls.iter().filter(|&&p| p > 0.0).count();
        let wr = wins as f64 / session_pnls.len() as f64;

        if wr < 0.40 {
            0.5
        } else if wr < 0.50 {
            0.75
        } else {
            1.0
        }
    }

    /// Consecutive loss streak → RiskStatus.
    /// Порт: adaptive_risk.py:300-327
    /// >= limit → STOPPED, >= limit-1 → REDUCED, else ACTIVE
    fn check_consecutive_losses(pnl_history: &[f64], limit: u32) -> RiskStatus {
        let mut streak: u32 = 0;
        // Check from the end (most recent first)
        for &pnl in pnl_history.iter().rev() {
            if pnl < 0.0 {
                streak += 1;
            } else {
                break;
            }
        }

        if streak >= limit {
            RiskStatus::Stopped
        } else if limit > 0 && streak >= limit - 1 {
            RiskStatus::Reduced
        } else {
            RiskStatus::Active
        }
    }

    /// Daily loss → RiskStatus.
    /// Порт: adaptive_risk.py:329-351
    /// >= limit → STOPPED, >= 80% → REDUCED, else ACTIVE
    fn check_daily_loss(&self, today_realized_loss: f64) -> RiskStatus {
        if today_realized_loss >= self.daily_loss_limit {
            RiskStatus::Stopped
        } else if today_realized_loss >= self.daily_loss_limit * 0.8 {
            RiskStatus::Reduced
        } else {
            RiskStatus::Active
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> FixedRiskParams {
        FixedRiskParams {
            entry_price: 1.00100,
            stop_loss_price: 1.00000,
            equity: 100_000.0,
            risk_pct: 0.01, // 1%
            commission_rate: 0.0,
            exchange_rate: 1.0,
            tick_size: 0.00001,
            hard_limit: None,
            unit_batch_size: 0.0,
        }
    }

    // --- Fixed Risk tests (порт Nautilus test suite) ---

    #[test]
    fn test_zero_equity() {
        let mut p = default_params();
        p.equity = 0.0;
        assert_eq!(fixed_risk_position_size(&p), 0.0);
    }

    #[test]
    fn test_zero_exchange_rate() {
        let mut p = default_params();
        p.exchange_rate = 0.0;
        assert_eq!(fixed_risk_position_size(&p), 0.0);
    }

    #[test]
    fn test_zero_risk_ticks() {
        let mut p = default_params();
        p.stop_loss_price = p.entry_price; // same = zero risk
        assert_eq!(fixed_risk_position_size(&p), 0.0);
    }

    #[test]
    fn test_basic_sizing() {
        let p = default_params();
        let size = fixed_risk_position_size(&p);
        // 100k × 1% = 1000 riskable / (100 ticks × 0.00001) = 1,000,000
        assert!((size - 1_000_000.0).abs() < 1.0, "Expected ~1M, got {}", size);
    }

    #[test]
    fn test_hard_limit() {
        let mut p = default_params();
        p.hard_limit = Some(500_000.0);
        let size = fixed_risk_position_size(&p);
        assert!((size - 500_000.0).abs() < 1.0, "Should be capped at 500k, got {}", size);
    }

    #[test]
    fn test_batch_rounding() {
        let mut p = default_params();
        p.equity = 50_000.0;
        p.unit_batch_size = 100_000.0;
        let size = fixed_risk_position_size(&p);
        assert_eq!(size % p.unit_batch_size, 0.0, "Should be rounded to batch: {}", size);
    }

    #[test]
    fn test_with_commission() {
        let p1 = default_params();
        let mut p2 = default_params();
        p2.commission_rate = 0.01; // 1% commission
        let s1 = fixed_risk_position_size(&p1);
        let s2 = fixed_risk_position_size(&p2);
        assert!(s2 < s1, "Commission should reduce size: {} vs {}", s2, s1);
    }

    // --- Kelly tests ---

    #[test]
    fn test_kelly_profitable() {
        let f = kelly_fraction(0.6, 2.0, 1.0);
        // f* = 0.6/1.0 - 0.4/2.0 = 0.6 - 0.2 = 0.4
        assert!((f - 0.4).abs() < 1e-6, "Expected 0.4, got {}", f);
    }

    #[test]
    fn test_kelly_losing() {
        let f = kelly_fraction(0.3, 1.0, 1.0);
        // f* = 0.3/1.0 - 0.7/1.0 = -0.4 → clamped to 0
        assert_eq!(f, 0.0, "Losing strategy should return 0");
    }

    #[test]
    fn test_kelly_breakeven() {
        let f = kelly_fraction(0.5, 1.0, 1.0);
        // f* = 0.5/1.0 - 0.5/1.0 = 0
        assert_eq!(f, 0.0, "Breakeven should return 0");
    }

    #[test]
    fn test_half_kelly() {
        let full = kelly_fraction(0.6, 2.0, 1.0);
        let half = half_kelly(0.6, 2.0, 1.0);
        assert!((half - full * 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_kelly_edge_cases() {
        assert_eq!(kelly_fraction(0.0, 1.0, 1.0), 0.0);
        assert_eq!(kelly_fraction(1.0, 1.0, 1.0), 0.0);
        assert_eq!(kelly_fraction(0.5, 0.0, 1.0), 0.0);
        assert_eq!(kelly_fraction(0.5, 1.0, 0.0), 0.0);
    }

    // --- Hybrid tests ---

    #[test]
    fn test_hybrid_uses_minimum() {
        let p = default_params();
        let result = hybrid_position_size(&p, 0.6, 2.0, 1.0);
        assert!(result.final_size <= result.fixed_risk_size);
        assert!(result.final_size <= result.kelly_adjusted_size || result.kelly_adjusted_size == 0.0);
    }

    #[test]
    fn test_hybrid_falls_back_to_fixed() {
        let p = default_params();
        // Losing strategy → kelly=0 → use fixed risk only
        let result = hybrid_position_size(&p, 0.3, 1.0, 1.0);
        assert_eq!(result.method_used, SizingMethod::FixedRisk);
        assert!(result.final_size > 0.0);
    }

    #[test]
    fn test_hybrid_result_fields() {
        let p = default_params();
        let result = hybrid_position_size(&p, 0.6, 2.0, 1.0);
        assert!(result.fixed_risk_size > 0.0);
        assert!(result.kelly_fraction > 0.0);
        assert!(result.final_size > 0.0);
    }

    // --- Adaptive Risk Engine tests (порт adaptive_risk.py) ---

    #[test]
    fn test_adaptive_insufficient_data() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, -5.0]; // < 5 trades
        let sessions = [vec![], vec![], vec![]];
        let result = engine.calculate_constraints(&pnl, &sessions, 0.0, 0.5, 1.0, 1.0);
        assert_eq!(result.status, RiskStatus::Active);
        assert_eq!(result.risk_per_trade_pct, 2.0, "Should use default 2%");
    }

    #[test]
    fn test_adaptive_healthy() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, 15.0, -5.0, 8.0, 12.0, -3.0, 20.0];
        let sessions = [
            vec![10.0, -5.0, 8.0],   // asian: 66% WR → 1.0
            vec![15.0, 12.0, 20.0],   // london: 100% WR → 1.0
            vec![-3.0, 5.0, 7.0],     // ny: 66% WR → 1.0
        ];
        let result = engine.calculate_constraints(&pnl, &sessions, 0.0, 0.7, 15.0, 5.0);
        assert_eq!(result.status, RiskStatus::Active);
        assert_eq!(result.scale_factor, 1.0);
    }

    #[test]
    fn test_adaptive_consecutive_stopped() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, -1.0, -1.0, -1.0, -1.0, -1.0]; // 5 consec losses
        let sessions = [vec![], vec![], vec![]];
        let result = engine.calculate_constraints(&pnl, &sessions, 0.0, 0.5, 1.0, 1.0);
        assert_eq!(result.status, RiskStatus::Stopped);
    }

    #[test]
    fn test_adaptive_daily_loss_stopped() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, 15.0, -5.0, 8.0, 12.0];
        let sessions = [vec![], vec![], vec![]];
        let result = engine.calculate_constraints(&pnl, &sessions, 600.0, 0.6, 2.0, 1.0);
        assert_eq!(result.status, RiskStatus::Stopped);
    }

    #[test]
    fn test_adaptive_daily_loss_reduced() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, 15.0, -5.0, 8.0, 12.0];
        let sessions = [vec![], vec![], vec![]];
        // 80% of $500 = $400
        let result = engine.calculate_constraints(&pnl, &sessions, 410.0, 0.6, 2.0, 1.0);
        assert_eq!(result.status, RiskStatus::Reduced);
        assert!(result.scale_factor < 1.0, "Reduced should have scale < 1.0");
    }

    #[test]
    fn test_adaptive_drawdown_scaling() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        // Create >10% drawdown from $10k base
        let pnl = vec![100.0, 200.0, -500.0, -700.0, -200.0]; // total -1100 from peak 10300
        let sessions = [vec![], vec![], vec![]];
        let result = engine.calculate_constraints(&pnl, &sessions, 0.0, 0.5, 1.0, 1.0);
        assert!(result.scale_factor <= 0.5, "DD>10% should be 0.5x, got {}", result.scale_factor);
    }

    #[test]
    fn test_adaptive_session_weak() {
        let engine = AdaptiveRiskEngine::new(5, 500.0);
        let pnl = vec![10.0, -5.0, 8.0, -3.0, 12.0];
        let sessions = [
            vec![-5.0, -3.0, -2.0, 1.0], // asian: 25% WR → 0.5
            vec![10.0, 8.0, 12.0],         // london: 100% → 1.0
            vec![-1.0, -2.0, 5.0],         // ny: 33% → 0.5
        ];
        let result = engine.calculate_constraints(&pnl, &sessions, 0.0, 0.5, 1.0, 1.0);
        assert!((result.session_adjustments[0] - 0.5).abs() < 0.01, "Asian should be 0.5");
        assert!((result.session_adjustments[1] - 1.0).abs() < 0.01, "London should be 1.0");
        assert!((result.session_adjustments[2] - 0.5).abs() < 0.01, "NY should be 0.5");
    }
}

