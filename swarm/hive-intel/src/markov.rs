/// Markov Regime Transition Matrix — предсказание СЛЕДУЮЩЕГО режима.
///
/// regime.rs отвечает: "КАКОЙ сейчас режим?" (trending/ranging/volatile)
/// markov.rs отвечает: "С КАКОЙ ВЕРОЯТНОСТЬЮ он сменится?"
///
/// Transition Matrix T[i][j] = P(next=j | current=i)
/// Строится из реальной истории переключений.
///
/// Пример:
///   trending_up → trending_up:   70%
///   trending_up → ranging:       20%
///   trending_up → volatile:      10%
///
/// Зачем: Titan ЗАРАНЕЕ готовится к смене режима вместо реактивного переключения.
use serde::Serialize;
use std::collections::HashMap;

/// Все возможные режимы (синхронизированы с regime.rs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum MarketRegime {
    TrendingUp,
    TrendingDown,
    Ranging,
    Volatile,
}

impl MarketRegime {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trending_up" => Self::TrendingUp,
            "trending_down" => Self::TrendingDown,
            "volatile" => Self::Volatile,
            _ => Self::Ranging, // default
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::TrendingUp => "trending_up",
            Self::TrendingDown => "trending_down",
            Self::Ranging => "ranging",
            Self::Volatile => "volatile",
        }
    }

    pub fn all() -> &'static [MarketRegime] {
        &[Self::TrendingUp, Self::TrendingDown, Self::Ranging, Self::Volatile]
    }
}

/// Матрица переходов: T\[from\]\[to\] = probability.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionMatrix {
    /// Количество наблюдённых переходов: counts[from][to]
    counts: HashMap<MarketRegime, HashMap<MarketRegime, u32>>,
    /// Общее число переходов
    pub total_transitions: u32,
}

impl Default for TransitionMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl TransitionMatrix {
    pub fn new() -> Self {
        let mut counts = HashMap::new();
        for &regime in MarketRegime::all() {
            let mut inner = HashMap::new();
            for &to in MarketRegime::all() {
                inner.insert(to, 0u32);
            }
            counts.insert(regime, inner);
        }
        Self { counts, total_transitions: 0 }
    }

    /// Записать наблюдённый переход from → to.
    pub fn observe_transition(&mut self, from: MarketRegime, to: MarketRegime) {
        if let Some(row) = self.counts.get_mut(&from) {
            *row.entry(to).or_insert(0) += 1;
        }
        self.total_transitions += 1;
    }

    /// Получить вероятность перехода from → to.
    /// С Laplace smoothing (α=1) чтобы избежать P=0.
    pub fn transition_probability(&self, from: MarketRegime, to: MarketRegime) -> f64 {
        let alpha = 1.0; // Laplace smoothing
        let n_states = MarketRegime::all().len() as f64;
        
        let row_sum: u32 = self.counts.get(&from)
            .map(|row| row.values().sum())
            .unwrap_or(0);
        let count = self.counts.get(&from)
            .and_then(|row| row.get(&to))
            .copied()
            .unwrap_or(0);

        (count as f64 + alpha) / (row_sum as f64 + alpha * n_states)
    }

    /// Получить все вероятности из текущего режима.
    pub fn predict_next(&self, current: MarketRegime) -> Vec<(MarketRegime, f64)> {
        let mut predictions: Vec<(MarketRegime, f64)> = MarketRegime::all()
            .iter()
            .map(|&to| (to, self.transition_probability(current, to)))
            .collect();
        
        // Сортировать по убыванию вероятности
        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predictions
    }

    /// Самый вероятный следующий режим.
    pub fn most_likely_next(&self, current: MarketRegime) -> (MarketRegime, f64) {
        self.predict_next(current)
            .into_iter()
            .next()
            .unwrap_or((MarketRegime::Ranging, 0.25))
    }

    /// Стабильность текущего режима (вероятность остаться в нём).
    pub fn regime_stability(&self, regime: MarketRegime) -> f64 {
        self.transition_probability(regime, regime)
    }

    /// "Опасность" — вероятность перехода в volatile.
    pub fn volatility_risk(&self, current: MarketRegime) -> f64 {
        self.transition_probability(current, MarketRegime::Volatile)
    }
}

/// Построить матрицу из истории режимов (хронологический порядок).
pub fn build_from_history(regime_history: &[MarketRegime]) -> TransitionMatrix {
    let mut matrix = TransitionMatrix::new();
    for window in regime_history.windows(2) {
        matrix.observe_transition(window[0], window[1]);
    }
    matrix
}

/// Предиктивный отчёт для текущего режима.
#[derive(Debug, Clone, Serialize)]
pub struct RegimeForecast {
    pub current_regime: String,
    pub stability: f64,
    pub volatility_risk: f64,
    pub predictions: Vec<(String, f64)>,
    pub recommendation: String,
}

/// Сгенерировать forecast для текущего режима.
pub fn forecast(matrix: &TransitionMatrix, current: MarketRegime) -> RegimeForecast {
    let stability = matrix.regime_stability(current);
    let vol_risk = matrix.volatility_risk(current);
    let predictions = matrix.predict_next(current);

    let recommendation = if stability > 0.7 {
        "HOLD: режим стабилен, продолжать текущую стратегию".to_string()
    } else if vol_risk > 0.3 {
        "REDUCE: высокий риск перехода в volatile — уменьшить позиции".to_string()
    } else if stability < 0.4 {
        "ADAPT: режим нестабилен — готовиться к переключению стратегии".to_string()
    } else {
        "MONITOR: нормальная динамика, мониторить".to_string()
    };

    RegimeForecast {
        current_regime: current.as_str().to_string(),
        stability,
        volatility_risk: vol_risk,
        predictions: predictions.iter().map(|(r, p)| (r.as_str().to_string(), *p)).collect(),
        recommendation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_matrix_uniform() {
        let m = TransitionMatrix::new();
        let p = m.transition_probability(MarketRegime::TrendingUp, MarketRegime::Ranging);
        // Laplace: 1/4 = 0.25 (uniform with no data)
        assert!((p - 0.25).abs() < 0.01, "Empty matrix should give ~0.25, got {}", p);
    }

    #[test]
    fn test_single_dominant_transition() {
        let mut m = TransitionMatrix::new();
        for _ in 0..100 {
            m.observe_transition(MarketRegime::TrendingUp, MarketRegime::TrendingUp);
        }
        let p_stay = m.transition_probability(MarketRegime::TrendingUp, MarketRegime::TrendingUp);
        assert!(p_stay > 0.9, "100 observations of stay should give >90%, got {:.2}", p_stay);
    }

    #[test]
    fn test_probabilities_sum_to_one() {
        let mut m = TransitionMatrix::new();
        m.observe_transition(MarketRegime::Ranging, MarketRegime::TrendingUp);
        m.observe_transition(MarketRegime::Ranging, MarketRegime::Volatile);
        m.observe_transition(MarketRegime::Ranging, MarketRegime::Ranging);

        let total: f64 = MarketRegime::all().iter()
            .map(|&to| m.transition_probability(MarketRegime::Ranging, to))
            .sum();
        assert!((total - 1.0).abs() < 1e-10, "Probabilities must sum to 1.0, got {}", total);
    }

    #[test]
    fn test_build_from_history() {
        let history = vec![
            MarketRegime::Ranging,
            MarketRegime::TrendingUp,
            MarketRegime::TrendingUp,
            MarketRegime::Volatile,
            MarketRegime::Ranging,
        ];
        let m = build_from_history(&history);
        assert_eq!(m.total_transitions, 4);
        // Ranging → TrendingUp: 1/1 + smoothing
        let p = m.transition_probability(MarketRegime::Ranging, MarketRegime::TrendingUp);
        assert!(p > 0.3, "Ranging→TrendingUp should be significant, got {:.2}", p);
    }

    #[test]
    fn test_most_likely_next() {
        let mut m = TransitionMatrix::new();
        for _ in 0..50 { m.observe_transition(MarketRegime::Volatile, MarketRegime::Ranging); }
        for _ in 0..10 { m.observe_transition(MarketRegime::Volatile, MarketRegime::TrendingDown); }
        
        let (next, prob) = m.most_likely_next(MarketRegime::Volatile);
        assert_eq!(next, MarketRegime::Ranging, "Most likely after volatile = ranging");
        assert!(prob > 0.7, "Ranging should be >70%, got {:.2}", prob);
    }

    #[test]
    fn test_stability_high() {
        let mut m = TransitionMatrix::new();
        for _ in 0..80 { m.observe_transition(MarketRegime::TrendingUp, MarketRegime::TrendingUp); }
        for _ in 0..20 { m.observe_transition(MarketRegime::TrendingUp, MarketRegime::Ranging); }
        
        let stability = m.regime_stability(MarketRegime::TrendingUp);
        assert!(stability > 0.7, "80/100 same should give stability >0.7, got {:.2}", stability);
    }

    #[test]
    fn test_volatility_risk() {
        let mut m = TransitionMatrix::new();
        for _ in 0..30 { m.observe_transition(MarketRegime::TrendingDown, MarketRegime::Volatile); }
        for _ in 0..70 { m.observe_transition(MarketRegime::TrendingDown, MarketRegime::Ranging); }

        let risk = m.volatility_risk(MarketRegime::TrendingDown);
        assert!(risk > 0.2 && risk < 0.4, "30/100 should give ~30% vol risk, got {:.2}", risk);
    }

    #[test]
    fn test_forecast_recommendation() {
        let mut m = TransitionMatrix::new();
        // Стабильный trending
        for _ in 0..90 { m.observe_transition(MarketRegime::TrendingUp, MarketRegime::TrendingUp); }
        for _ in 0..10 { m.observe_transition(MarketRegime::TrendingUp, MarketRegime::Ranging); }
        
        let fc = forecast(&m, MarketRegime::TrendingUp);
        assert!(fc.recommendation.contains("HOLD"), "Stable regime should recommend HOLD");
        assert!(fc.stability > 0.7);
    }

    #[test]
    fn test_forecast_high_vol_risk() {
        let mut m = TransitionMatrix::new();
        for _ in 0..40 { m.observe_transition(MarketRegime::Ranging, MarketRegime::Volatile); }
        for _ in 0..60 { m.observe_transition(MarketRegime::Ranging, MarketRegime::TrendingUp); }
        
        let fc = forecast(&m, MarketRegime::Ranging);
        assert!(fc.recommendation.contains("REDUCE"), "High vol risk should recommend REDUCE, got: {}", fc.recommendation);
    }
}
