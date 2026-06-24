/// Regime Detection — HMM-inspired 4-state market regime classifier.
///
/// 4 состояния: Trending Up | Trending Down | Ranging | Volatile
///
/// Использует returns + volatility для классификации без полного HMM
/// (упрощённый детектор для production use — полный HMM в будущем).
use serde::Serialize;

/// Рыночный режим.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MarketRegime {
    TrendingUp,
    TrendingDown,
    Ranging,
    Volatile,
}

impl MarketRegime {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TrendingUp => "trending_up",
            Self::TrendingDown => "trending_down",
            Self::Ranging => "ranging",
            Self::Volatile => "volatile",
        }
    }
}

/// Результат классификации режима.
#[derive(Debug, Clone, Serialize)]
pub struct RegimeResult {
    pub regime: MarketRegime,
    pub confidence: f64,
    pub mean_return: f64,
    pub volatility: f64,
    pub trend_strength: f64,
}

/// Классифицирует рыночный режим на основе returns.
///
/// Алгоритм:
/// 1. Считаем mean return и std dev
/// 2. trend_strength = |mean| / std (Sharpe-like)
/// 3. Классифицируем:
///    - trend_strength > 0.5 + mean > 0 → TrendingUp
///    - trend_strength > 0.5 + mean < 0 → TrendingDown
///    - volatility > 2 × historical_vol → Volatile
///    - иначе → Ranging
///
/// `returns`: slice of log returns (или простых %-returns)
/// `historical_vol`: базовая волатильность для сравнения
pub fn classify_regime(returns: &[f64], historical_vol: f64) -> RegimeResult {
    if returns.is_empty() {
        return RegimeResult {
            regime: MarketRegime::Ranging,
            confidence: 0.0,
            mean_return: 0.0,
            volatility: 0.0,
            trend_strength: 0.0,
        };
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let vol = variance.sqrt();

    let trend_strength = if vol > 1e-10 { mean.abs() / vol } else { 0.0 };

    let vol_ratio = if historical_vol > 1e-10 { vol / historical_vol } else { 1.0 };

    let (regime, confidence) = if vol_ratio > 2.0 {
        // Волатильность в 2x выше нормы → Volatile
        (MarketRegime::Volatile, (vol_ratio / 3.0).min(1.0))
    } else if trend_strength > 0.5 {
        if mean > 0.0 {
            (MarketRegime::TrendingUp, (trend_strength / 1.5).min(1.0))
        } else {
            (MarketRegime::TrendingDown, (trend_strength / 1.5).min(1.0))
        }
    } else {
        (MarketRegime::Ranging, (1.0 - trend_strength).min(1.0))
    };

    RegimeResult {
        regime,
        confidence,
        mean_return: mean,
        volatility: vol,
        trend_strength,
    }
}

/// Детектирует смену режима между двумя окнами.
pub fn detect_regime_change(
    old_returns: &[f64],
    new_returns: &[f64],
    historical_vol: f64,
) -> Option<(MarketRegime, MarketRegime)> {
    let old = classify_regime(old_returns, historical_vol);
    let new = classify_regime(new_returns, historical_vol);

    if old.regime != new.regime {
        Some((old.regime, new.regime))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_returns_ranging() {
        let r = classify_regime(&[], 0.01);
        assert_eq!(r.regime, MarketRegime::Ranging);
    }

    #[test]
    fn test_trending_up() {
        let returns = vec![0.02, 0.015, 0.025, 0.01, 0.03, 0.02, 0.018, 0.022];
        let r = classify_regime(&returns, 0.01);
        assert_eq!(r.regime, MarketRegime::TrendingUp, "Consistent positive returns = trending up");
    }

    #[test]
    fn test_trending_down() {
        let returns = vec![-0.02, -0.015, -0.025, -0.01, -0.03, -0.02, -0.018];
        let r = classify_regime(&returns, 0.01);
        assert_eq!(r.regime, MarketRegime::TrendingDown, "Consistent negative returns = trending down");
    }

    #[test]
    fn test_volatile() {
        let returns = vec![0.05, -0.06, 0.07, -0.08, 0.04, -0.05]; // wild swings
        let r = classify_regime(&returns, 0.01); // historical vol = 0.01, actual ~5x
        assert_eq!(r.regime, MarketRegime::Volatile, "High vol vs historical = volatile");
    }

    #[test]
    fn test_ranging() {
        let returns = vec![0.001, -0.001, 0.002, -0.0015, 0.001, -0.001];
        let r = classify_regime(&returns, 0.01);
        assert_eq!(r.regime, MarketRegime::Ranging, "Low trend + normal vol = ranging");
    }

    #[test]
    fn test_regime_change_detection() {
        let old = vec![0.02, 0.015, 0.025, 0.01, 0.03];  // trending up
        let new = vec![-0.02, -0.015, -0.025, -0.01, -0.03]; // trending down
        let change = detect_regime_change(&old, &new, 0.01);
        assert!(change.is_some());
        let (from, to) = change.unwrap();
        assert_eq!(from, MarketRegime::TrendingUp);
        assert_eq!(to, MarketRegime::TrendingDown);
    }
}
