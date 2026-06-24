/// Affective Memory — эмоциональный слой памяти.
///
/// Отслеживает эмоциональный фон агента:
/// - EWMA Confidence (экспоненциальная уверенность)
/// - Risk Appetite (аппетит к риску на основе просадки)
///
/// Портировано из: tradememory-protocol/src/tradememory/owm/affective.py
/// EWMA Confidence — экспоненциально взвешенная уверенность.
///
/// Маппит сырой EWMA в [0, 1] через sigmoid.
/// λ = 0.9 по умолчанию (больше → больше вес истории).
pub fn ewma_confidence(outcomes: &[f64], lam: f64) -> f64 {
    debug_assert!(lam > 0.0 && lam < 1.0, "lam must be in (0, 1)");
    if outcomes.is_empty() {
        return 0.5;
    }

    let mut ewma = 0.0_f64;
    for &val in outcomes {
        ewma = lam * ewma + (1.0 - lam) * val;
    }

    // Sigmoid mapping: scale tuned for $0.5-$50 PnL range
    // 0.1 → $5 PnL gives ~0.62 confidence, $50 gives ~0.99
    let x = (-0.1 * ewma).clamp(-500.0, 500.0);
    1.0 / (1.0 + x.exp())
}

/// Risk Appetite — аппетит к риску.
///
/// `max(0.1, 1 - (dd / max_dd)²)`
///
/// - Нет просадки → 1.0 (полный аппетит)
/// - Максимальная просадка → 0.1 (минимальный)
/// - Квадратичное затухание: медленно сначала, резко потом.
pub fn risk_appetite(drawdown_pct: f64, max_dd_pct: f64) -> f64 {
    debug_assert!(max_dd_pct > 0.0, "max_dd_pct must be positive");
    debug_assert!(drawdown_pct >= 0.0, "drawdown_pct must be non-negative");

    let ratio = drawdown_pct / max_dd_pct;
    (1.0 - ratio * ratio).max(0.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewma_empty_returns_neutral() {
        assert!((ewma_confidence(&[], 0.9) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_ewma_all_wins_high_confidence() {
        let wins = vec![10.0, 20.0, 15.0, 25.0, 30.0];
        let conf = ewma_confidence(&wins, 0.9);
        assert!(conf > 0.5, "All wins should give confidence > 0.5, got {:.3}", conf);
    }

    #[test]
    fn test_ewma_all_losses_low_confidence() {
        let losses = vec![-10.0, -20.0, -15.0, -25.0, -30.0];
        let conf = ewma_confidence(&losses, 0.9);
        assert!(conf < 0.5, "All losses should give confidence < 0.5, got {:.3}", conf);
    }

    #[test]
    fn test_risk_appetite_no_drawdown() {
        let app = risk_appetite(0.0, 10.0);
        assert!((app - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_risk_appetite_max_drawdown() {
        let app = risk_appetite(10.0, 10.0);
        assert!((app - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_risk_appetite_half_drawdown() {
        let app = risk_appetite(5.0, 10.0);
        // 1 - (0.5)² = 1 - 0.25 = 0.75
        assert!((app - 0.75).abs() < 1e-10);
    }
}
