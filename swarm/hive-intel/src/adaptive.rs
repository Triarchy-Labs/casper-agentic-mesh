/// Adaptive Weights — обучаемые веса для OWM scoring.
///
/// Вместо захардкоженных W_Q=0.30, W_SIM=0.25... каждый символ
/// обучает свои оптимальные веса на основе исходов трейдов.
///
/// Метод: Online gradient-free optimization (EWA — Exponentially Weighted Average).
/// После каждого трейда: если recall правильно предсказал → усилить вес
/// доминирующего компонента. Если ошибся → ослабить.
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Набор адаптивных весов для одного символа.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolWeights {
    pub symbol: String,
    pub w_q: f64,       // Outcome Quality weight
    pub w_sim: f64,     // Context Similarity weight
    pub w_rec: f64,     // Recency weight
    pub w_conf: f64,    // Confidence weight
    pub w_aff: f64,     // Affective weight
    pub updates: u32,   // Сколько раз обновлялись
}

impl SymbolWeights {
    /// Дефолтные веса (как в recall.rs).
    pub fn default_for(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            w_q: 0.30,
            w_sim: 0.25,
            w_rec: 0.20,
            w_conf: 0.15,
            w_aff: 0.10,
            updates: 0,
        }
    }

    /// Получить веса как slice для быстрого доступа.
    #[allow(dead_code)]
    pub fn as_array(&self) -> [f64; 5] {
        [self.w_q, self.w_sim, self.w_rec, self.w_conf, self.w_aff]
    }

    /// Нормализовать веса (сумма = 1.0, каждый ≥ 0.05).
    fn normalize(&mut self) {
        let min_w = 0.05;
        self.w_q = self.w_q.max(min_w);
        self.w_sim = self.w_sim.max(min_w);
        self.w_rec = self.w_rec.max(min_w);
        self.w_conf = self.w_conf.max(min_w);
        self.w_aff = self.w_aff.max(min_w);

        let sum = self.w_q + self.w_sim + self.w_rec + self.w_conf + self.w_aff;
        self.w_q /= sum;
        self.w_sim /= sum;
        self.w_rec /= sum;
        self.w_conf /= sum;
        self.w_aff /= sum;
    }

    /// Обновить веса на основе исхода трейда.
    ///
    /// `components`: [Q, Sim, Rec, Conf, Aff] значения из recall
    /// `outcome`: true = recall помог (profitable), false = recall ошибся
    /// `learning_rate`: скорость обучения (0.01–0.1)
    pub fn update(&mut self, components: &[f64; 5], outcome: bool, learning_rate: f64) {
        let lr = learning_rate.clamp(0.001, 0.2);

        // Найти доминирующий компонент (с наибольшим вкладом)
        let total: f64 = components.iter().sum();
        if total < 1e-10 { return; }

        let normalized: Vec<f64> = components.iter().map(|c| c / total).collect();

        // Если recall правильный → усилить доминирующий компонент
        // Если ошибся → ослабить доминирующий, усилить остальные
        let direction = if outcome { 1.0 } else { -1.0 };

        let weights = [&mut self.w_q, &mut self.w_sim, &mut self.w_rec,
                       &mut self.w_conf, &mut self.w_aff];

        for (i, w) in weights.into_iter().enumerate() {
            let gradient = direction * (normalized[i] - 0.2); // 0.2 = uniform
            *w += lr * gradient;
        }

        self.normalize();
        self.updates += 1;
    }
}

/// Хранилище адаптивных весов для всех символов.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveWeightStore {
    pub weights: HashMap<String, SymbolWeights>,
}

impl Default for AdaptiveWeightStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveWeightStore {
    pub fn new() -> Self {
        Self { weights: HashMap::new() }
    }

    /// Получить веса для символа (или дефолтные).
    pub fn get_weights(&self, symbol: &str) -> SymbolWeights {
        self.weights.get(symbol)
            .cloned()
            .unwrap_or_else(|| SymbolWeights::default_for(symbol))
    }

    /// Обновить веса символа.
    pub fn update(&mut self, symbol: &str, components: &[f64; 5], outcome: bool, lr: f64) {
        let entry = self.weights
            .entry(symbol.to_string())
            .or_insert_with(|| SymbolWeights::default_for(symbol));
        entry.update(components, outcome, lr);
    }

    /// Символы с наибольшим числом обновлений (самые обученные).
    pub fn most_trained(&self, limit: usize) -> Vec<&SymbolWeights> {
        let mut sorted: Vec<_> = self.weights.values().collect();
        sorted.sort_by_key(|w| std::cmp::Reverse(w.updates));
        sorted.truncate(limit);
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_sum_to_one() {
        let w = SymbolWeights::default_for("BTCUSDT");
        let sum: f64 = w.as_array().iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "Default weights should sum to 1.0");
    }

    #[test]
    fn test_update_preserves_sum() {
        let mut w = SymbolWeights::default_for("BTCUSDT");
        let components = [0.8, 0.5, 0.7, 0.9, 1.0];
        w.update(&components, true, 0.1);
        let sum: f64 = w.as_array().iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "After update, weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_positive_outcome_shifts_weights() {
        let mut w = SymbolWeights::default_for("BTCUSDT");
        let w_q_before = w.w_q;
        // Q доминирует → после positive outcome, w_q должен вырасти
        let components = [0.9, 0.1, 0.1, 0.1, 0.1];
        w.update(&components, true, 0.1);
        assert!(w.w_q > w_q_before, "Dominant Q with positive outcome should increase w_q");
    }

    #[test]
    fn test_negative_outcome_shifts_away() {
        let mut w = SymbolWeights::default_for("ETHUSDT");
        let w_q_before = w.w_q;
        // Q доминирует → после negative outcome, w_q должен УМЕНЬШИТЬСЯ
        let components = [0.9, 0.1, 0.1, 0.1, 0.1];
        w.update(&components, false, 0.1);
        assert!(w.w_q < w_q_before, "Dominant Q with negative outcome should decrease w_q");
    }

    #[test]
    fn test_store_get_or_default() {
        let store = AdaptiveWeightStore::new();
        let w = store.get_weights("NEWCOIN");
        assert_eq!(w.updates, 0);
        assert!((w.w_q - 0.30).abs() < 1e-10);
    }

    #[test]
    fn test_min_weight_floor() {
        let mut w = SymbolWeights::default_for("TEST");
        // Force extreme imbalance
        w.w_aff = 0.0;
        w.normalize();
        assert!(w.w_aff >= 0.05, "Weights should have a 5% floor");
    }
}
