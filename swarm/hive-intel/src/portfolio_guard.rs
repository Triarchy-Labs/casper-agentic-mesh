/// Portfolio-level Risk Guards + SPRT Hypothesis Test.
///
/// ПОРТИРОВАНО ИЗ:
///   - nautilus_trader/crates/risk (LGPL-3.0) — концепции portfolio guards
///   - deep_causality/uncertain/algos/hypothesis/sprt_eval.rs (MIT) — SPRT formula
///
/// Nautilus RiskEngine слишком привязан к их типам (1799 строк), поэтому мы
/// извлекаем КОНЦЕПЦИИ (max_notional, drawdown_circuit, daily_loss_circuit)
/// и SPRT формулу (95 строк) в наш чистый standalone модуль.
///
/// Portfolio guards — мета-уровень НАД individual position risk:
///   1. MaxNotionalGuard — максимальный размер позиции в USDT
///   2. DrawdownCircuitBreaker — полная остановка при X% drawdown
///   3. DailyLossCircuitBreaker — стоп на день при Y% дневного убытка
///   4. OrderRateThrottle — не более N ордеров в секунду
///
/// SPRT — Sequential Probability Ratio Test для statistically valid decisions:
///   "Достаточно ли данных чтобы заключить что стратегия прибыльна?"
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// 1. Portfolio Guards (вдохновлено: nautilus risk engine)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioGuardConfig {
    pub max_notional_usdt: f64,      // макс позиция в USDT
    pub max_drawdown_pct: f64,       // circuit breaker на X% drawdown (0.10 = 10%)
    pub max_daily_loss_pct: f64,     // стоп торговли на день при X% убытке
    pub max_orders_per_second: u32,  // throttle
    pub max_open_positions: usize,   // макс одновременных позиций
}

impl Default for PortfolioGuardConfig {
    fn default() -> Self {
        Self {
            max_notional_usdt: 1000.0,
            max_drawdown_pct: 0.10,       // 10% drawdown → стоп
            max_daily_loss_pct: 0.03,     // 3% daily loss → стоп на день
            max_orders_per_second: 10,
            max_open_positions: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum GuardVerdict {
    Approved,
    DeniedMaxNotional,
    DeniedDrawdownCircuit,
    DeniedDailyLossCircuit,
    DeniedOrderRate,
    DeniedMaxPositions,
}

/// Portfolio state snapshot for guard evaluation.
#[derive(Debug, Clone)]
pub struct PortfolioState {
    pub proposed_notional_usdt: f64,
    pub current_equity: f64,
    pub peak_equity: f64,
    pub daily_start_equity: f64,
    pub orders_this_second: u32,
    pub open_positions: usize,
}

/// Evaluate ALL portfolio-level guards. Returns first denial or Approved.
///
/// Порядок проверок (от критичного к мягкому):
/// 1. Drawdown circuit breaker
/// 2. Daily loss circuit
/// 3. Max positions
/// 4. Order rate throttle
/// 5. Max notional
pub fn evaluate_portfolio_guards(
    config: &PortfolioGuardConfig,
    state: &PortfolioState,
) -> GuardVerdict {
    // 1. Drawdown from peak
    if state.peak_equity > 0.0 {
        let drawdown = (state.peak_equity - state.current_equity) / state.peak_equity;
        if drawdown >= config.max_drawdown_pct {
            return GuardVerdict::DeniedDrawdownCircuit;
        }
    }

    // 2. Daily loss
    if state.daily_start_equity > 0.0 {
        let daily_loss = (state.daily_start_equity - state.current_equity) / state.daily_start_equity;
        if daily_loss >= config.max_daily_loss_pct {
            return GuardVerdict::DeniedDailyLossCircuit;
        }
    }

    // 3. Max open positions
    if state.open_positions >= config.max_open_positions {
        return GuardVerdict::DeniedMaxPositions;
    }

    // 4. Order rate
    if state.orders_this_second >= config.max_orders_per_second {
        return GuardVerdict::DeniedOrderRate;
    }

    // 5. Notional size
    if state.proposed_notional_usdt > config.max_notional_usdt {
        return GuardVerdict::DeniedMaxNotional;
    }

    GuardVerdict::Approved
}

// ═══════════════════════════════════════════════════════════════
// 2. SPRT — Sequential Probability Ratio Test
//    (порт: deep_causality/uncertain/algos/hypothesis/sprt_eval.rs)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SprtResult {
    AcceptH1,     // стратегия прибыльна (p > threshold)
    AcceptH0,     // стратегия НЕ прибыльна (p <= threshold)
    Inconclusive, // недостаточно данных
}

/// Sequential Probability Ratio Test.
///
/// Проверяет гипотезу: "Доля прибыльных сделок > threshold?"
///
/// H0: P(win) <= threshold - epsilon  (стратегия плоха)
/// H1: P(win) > threshold + epsilon   (стратегия хороша)
///
/// Порт: deep_causality sprt_eval.rs:22-94
///
/// * `wins` — количество прибыльных сделок
/// * `total` — общее количество сделок
/// * `threshold` — порог (обычно 0.5)
/// * `confidence` — уровень уверенности (0.95)
/// * `epsilon` — зона безразличия (0.05)
pub fn sprt_evaluate(
    wins: usize,
    total: usize,
    threshold: f64,
    confidence: f64,
    epsilon: f64,
) -> SprtResult {
    if total == 0 {
        return SprtResult::Inconclusive;
    }

    let alpha = 1.0 - confidence;
    let beta = alpha; // Type II error = Type I error

    // SPRT boundaries (порт: sprt_eval.rs:36-37)
    let a_boundary = (beta / (1.0 - alpha)).ln();
    let b_boundary = ((1.0 - beta) / alpha).ln();

    let p0 = (threshold - epsilon).clamp(f64::EPSILON, 1.0 - f64::EPSILON);
    let p1 = (threshold + epsilon).clamp(f64::EPSILON, 1.0 - f64::EPSILON);

    let n = total as f64;
    let x = wins as f64;

    // Log-likelihood ratio (порт: sprt_eval.rs:69-80)
    let term1 = (p1 / p0).ln();
    let term2 = ((1.0 - p1) / (1.0 - p0)).ln();
    let llr = x * term1 + (n - x) * term2;

    if llr <= a_boundary {
        SprtResult::AcceptH0 // стратегия плоха
    } else if llr >= b_boundary {
        SprtResult::AcceptH1 // стратегия прибыльна
    } else {
        SprtResult::Inconclusive // нужно больше данных
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state() -> PortfolioState {
        PortfolioState {
            proposed_notional_usdt: 500.0,
            current_equity: 9900.0,
            peak_equity: 10000.0,
            daily_start_equity: 10000.0,
            orders_this_second: 2,
            open_positions: 1,
        }
    }

    #[test]
    fn test_approved_normal() {
        let config = PortfolioGuardConfig::default();
        let state = default_state();
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::Approved);
    }

    #[test]
    fn test_denied_drawdown() {
        let config = PortfolioGuardConfig { max_drawdown_pct: 0.05, ..Default::default() };
        let state = PortfolioState {
            current_equity: 9000.0, // 10% drawdown
            ..default_state()
        };
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedDrawdownCircuit);
    }

    #[test]
    fn test_denied_daily_loss() {
        let config = PortfolioGuardConfig { max_daily_loss_pct: 0.02, ..Default::default() };
        let state = PortfolioState {
            current_equity: 9700.0, // 3% daily loss
            ..default_state()
        };
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedDailyLossCircuit);
    }

    #[test]
    fn test_denied_max_positions() {
        let config = PortfolioGuardConfig { max_open_positions: 3, ..Default::default() };
        let state = PortfolioState {
            open_positions: 3,
            ..default_state()
        };
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedMaxPositions);
    }

    #[test]
    fn test_denied_order_rate() {
        let config = PortfolioGuardConfig { max_orders_per_second: 5, ..Default::default() };
        let state = PortfolioState {
            orders_this_second: 5,
            ..default_state()
        };
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedOrderRate);
    }

    #[test]
    fn test_denied_max_notional() {
        let config = PortfolioGuardConfig { max_notional_usdt: 200.0, ..Default::default() };
        let state = default_state();
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedMaxNotional);
    }

    #[test]
    fn test_guard_priority_drawdown_first() {
        // drawdown AND daily loss — drawdown should fire first
        let config = PortfolioGuardConfig {
            max_drawdown_pct: 0.05,
            max_daily_loss_pct: 0.02,
            ..Default::default()
        };
        let state = PortfolioState {
            current_equity: 8000.0, // both triggers hit
            ..default_state()
        };
        assert_eq!(evaluate_portfolio_guards(&config, &state), GuardVerdict::DeniedDrawdownCircuit);
    }

    // ── SPRT Tests ──

    #[test]
    fn test_sprt_profitable() {
        // 80 wins out of 100 → clearly above 0.5
        let result = sprt_evaluate(80, 100, 0.5, 0.95, 0.05);
        assert_eq!(result, SprtResult::AcceptH1);
    }

    #[test]
    fn test_sprt_unprofitable() {
        // 20 wins out of 100 → clearly below 0.5
        let result = sprt_evaluate(20, 100, 0.5, 0.95, 0.05);
        assert_eq!(result, SprtResult::AcceptH0);
    }

    #[test]
    fn test_sprt_inconclusive_few_trades() {
        // 3 wins out of 5 → not enough data
        let result = sprt_evaluate(3, 5, 0.5, 0.95, 0.05);
        assert_eq!(result, SprtResult::Inconclusive);
    }

    #[test]
    fn test_sprt_empty() {
        let result = sprt_evaluate(0, 0, 0.5, 0.95, 0.05);
        assert_eq!(result, SprtResult::Inconclusive);
    }

    #[test]
    fn test_sprt_boundary_50_50() {
        // 50 wins out of 100 → exactly at threshold, should be inconclusive or H0
        let result = sprt_evaluate(50, 100, 0.5, 0.95, 0.05);
        assert!(result == SprtResult::Inconclusive || result == SprtResult::AcceptH0);
    }

    #[test]
    fn test_sprt_high_confidence() {
        // 90 wins out of 100 with 99% confidence
        let result = sprt_evaluate(90, 100, 0.5, 0.99, 0.03);
        assert_eq!(result, SprtResult::AcceptH1);
    }
}
