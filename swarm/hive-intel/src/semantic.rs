/// Semantic Memory — Байесовские знания.
///
/// Хранит обобщённые знания типа:
/// "breakout в trending_up работает с P=0.73"
///
/// Confidence обновляется через Beta(α, β) distribution:
/// - Успех → α += weight
/// - Провал → β += weight
/// - Posterior mean = α / (α + β)
use serde::{Deserialize, Serialize};

/// Семантическое знание (belief).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: String,
    pub proposition: String,      // "VolBreakout works in trending markets"
    pub domain_strategy: String,  // "VolBreakout"
    pub domain_regime: String,    // "trending_up"
    pub alpha: f64,               // Beta distribution α (successes)
    pub beta: f64,                // Beta distribution β (failures)
    pub evidence_count: u32,      // Total observations
}

impl Belief {
    /// Создаёт новое знание с uniform prior Beta(1, 1).
    pub fn new(id: &str, proposition: &str, strategy: &str, regime: &str) -> Self {
        Self {
            id: id.to_string(),
            proposition: proposition.to_string(),
            domain_strategy: strategy.to_string(),
            domain_regime: regime.to_string(),
            alpha: 1.0,
            beta: 1.0,
            evidence_count: 0,
        }
    }

    /// Bayesian update: наблюдали результат.
    /// `success` = true → α += weight, иначе β += weight.
    pub fn update(&mut self, success: bool, weight: f64) {
        let w = weight.max(0.01); // Защита от нулевого веса
        if success {
            self.alpha += w;
        } else {
            self.beta += w;
        }
        self.evidence_count += 1;
    }

    /// Posterior mean: P(proposition | data) = α / (α + β)
    pub fn confidence(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Posterior variance: α·β / ((α+β)² · (α+β+1))
    /// Меньше variance = больше уверенности.
    pub fn uncertainty(&self) -> f64 {
        let sum = self.alpha + self.beta;
        (self.alpha * self.beta) / (sum * sum * (sum + 1.0))
    }

    /// Есть ли достаточно данных для принятия решения?
    /// Порог: uncertainty < 0.01 И evidence >= 10
    pub fn is_mature(&self) -> bool {
        self.evidence_count >= 10 && self.uncertainty() < 0.01
    }
}

/// Хранилище знаний.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticStore {
    pub beliefs: Vec<Belief>,
}

impl Default for SemanticStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticStore {
    pub fn new() -> Self {
        Self { beliefs: Vec::new() }
    }

    /// Найти или создать belief для пары (strategy, regime).
    pub fn get_or_create(&mut self, strategy: &str, regime: &str) -> &mut Belief {
        let idx = self.beliefs.iter().position(|b| {
            b.domain_strategy == strategy && b.domain_regime == regime
        });

        match idx {
            Some(i) => &mut self.beliefs[i],
            None => {
                let id = format!("{strategy}_{regime}");
                let prop = format!("{strategy} performs well in {regime} regime");
                self.beliefs.push(Belief::new(&id, &prop, strategy, regime));
                self.beliefs.last_mut().unwrap()
            }
        }
    }

    /// Обновить знание на основе исхода трейда.
    pub fn record_outcome(&mut self, strategy: &str, regime: &str, pnl: f64) {
        let success = pnl > 0.0;
        let weight = pnl.abs().sqrt().max(0.1); // Больший PnL → больший вес
        let belief = self.get_or_create(strategy, regime);
        belief.update(success, weight);
    }

    /// Получить confidence для пары (strategy, regime).
    pub fn query_confidence(&self, strategy: &str, regime: &str) -> Option<f64> {
        self.beliefs.iter()
            .find(|b| b.domain_strategy == strategy && b.domain_regime == regime)
            .map(Belief::confidence)
    }

    /// Все зрелые beliefs, отсортированные по confidence.
    pub fn mature_beliefs(&self) -> Vec<&Belief> {
        let mut mature: Vec<_> = self.beliefs.iter().filter(|b| b.is_mature()).collect();
        mature.sort_by(|a, b| b.confidence().partial_cmp(&a.confidence()).unwrap());
        mature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_prior() {
        let b = Belief::new("test", "test prop", "VolBreakout", "trending_up");
        assert!((b.confidence() - 0.5).abs() < 1e-10, "Uniform prior should give 0.5");
    }

    #[test]
    fn test_update_success_increases_confidence() {
        let mut b = Belief::new("test", "test", "VB", "trending");
        for _ in 0..10 {
            b.update(true, 1.0);
        }
        assert!(b.confidence() > 0.8, "10 successes should give conf > 0.8, got {:.3}", b.confidence());
    }

    #[test]
    fn test_update_failure_decreases_confidence() {
        let mut b = Belief::new("test", "test", "VB", "trending");
        for _ in 0..10 {
            b.update(false, 1.0);
        }
        assert!(b.confidence() < 0.2, "10 failures should give conf < 0.2, got {:.3}", b.confidence());
    }

    #[test]
    fn test_uncertainty_decreases_with_evidence() {
        let mut b = Belief::new("test", "test", "VB", "trending");
        let u0 = b.uncertainty();
        for _ in 0..20 {
            b.update(true, 1.0);
        }
        assert!(b.uncertainty() < u0, "Uncertainty should decrease with evidence");
    }

    #[test]
    fn test_semantic_store_record() {
        let mut store = SemanticStore::new();
        store.record_outcome("VolBreakout", "trending_up", 50.0);
        store.record_outcome("VolBreakout", "trending_up", -20.0);
        store.record_outcome("VolBreakout", "ranging", -10.0);
        
        assert_eq!(store.beliefs.len(), 2);
        let conf = store.query_confidence("VolBreakout", "trending_up").unwrap();
        assert!(conf > 0.5, "More wins should give conf > 0.5");
    }
}
