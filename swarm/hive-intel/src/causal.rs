/// Causal Chain Graph — каузальная память.
///
/// Не просто "BTC и ETH коррелируют" →
/// "Когда BTC падает >3% за 1 час → ETH падает >5% в следующие 2 часа (P=0.72)"
///
/// Это НАПРАВЛЕННЫЙ ГРАФ причинно-следственных связей с:
/// - Временны́м лагом (delay)
/// - Статистической силой (Bayesian confidence)
/// - Авто-обнаружением новых цепочек
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Каузальное ребро: A → B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    pub cause: String,        // "BTCUSDT_drop_3pct"
    pub effect: String,       // "ETHUSDT_drop_5pct"
    pub avg_delay_ms: f64,    // Средний временной лаг в ms
    pub observations: u32,    // Сколько раз наблюдали
    pub confirmations: u32,   // Сколько раз подтвердилось
    pub alpha: f64,           // Beta(α, β) — Bayesian strength
    pub beta: f64,
}

impl CausalEdge {
    pub fn new(cause: &str, effect: &str) -> Self {
        Self {
            cause: cause.to_string(),
            effect: effect.to_string(),
            avg_delay_ms: 0.0,
            observations: 0,
            confirmations: 0,
            alpha: 1.0,  // Uniform prior
            beta: 1.0,
        }
    }

    /// Вероятность каузальной связи: P(cause → effect | data)
    pub fn strength(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Наблюдали пару (cause, effect) → обновить статистику.
    pub fn observe(&mut self, confirmed: bool, delay_ms: f64) {
        self.observations += 1;

        if confirmed {
            self.confirmations += 1;
            self.alpha += 1.0;
            // Update running average of delay
            let n = self.confirmations as f64;
            self.avg_delay_ms = self.avg_delay_ms * (n - 1.0) / n + delay_ms / n;
        } else {
            self.beta += 1.0;
        }
    }

    /// Достаточно ли данных для доверия?
    pub fn is_significant(&self) -> bool {
        self.observations >= 10 && self.strength() > 0.6
    }
}

/// Граф каузальных связей.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraph {
    /// Ключ: "cause|effect", Значение: CausalEdge
    pub edges: HashMap<String, CausalEdge>,
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalGraph {
    pub fn new() -> Self {
        Self { edges: HashMap::new() }
    }

    fn edge_key(cause: &str, effect: &str) -> String {
        format!("{cause}|{effect}")
    }

    /// Записать наблюдение каузальной пары.
    pub fn observe(&mut self, cause: &str, effect: &str, confirmed: bool, delay_ms: f64) {
        let key = Self::edge_key(cause, effect);
        let edge = self.edges
            .entry(key)
            .or_insert_with(|| CausalEdge::new(cause, effect));
        edge.observe(confirmed, delay_ms);
    }

    /// Получить все эффекты для данной причины.
    pub fn effects_of(&self, cause: &str) -> Vec<&CausalEdge> {
        self.edges.values()
            .filter(|e| e.cause == cause)
            .collect()
    }

    /// Получить все причины для данного эффекта.
    pub fn causes_of(&self, effect: &str) -> Vec<&CausalEdge> {
        self.edges.values()
            .filter(|e| e.effect == effect)
            .collect()
    }

    /// Все значимые (подтверждённые) каузальные связи.
    pub fn significant_edges(&self) -> Vec<&CausalEdge> {
        let mut edges: Vec<_> = self.edges.values()
            .filter(|e| e.is_significant())
            .collect();
        edges.sort_by(|a, b| b.strength().partial_cmp(&a.strength()).unwrap());
        edges
    }

    /// Предсказание: "если сейчас произошло X, что произойдёт дальше?"
    pub fn predict(&self, cause: &str) -> Vec<(&str, f64, f64)> {
        self.effects_of(cause)
            .into_iter()
            .filter(|e| e.is_significant())
            .map(|e| (e.effect.as_str(), e.strength(), e.avg_delay_ms))
            .collect()
    }

    /// Детектор каузальных цепочек из потока событий.
    ///
    /// `events`: slice of (event_name, timestamp_ms)
    /// `max_lag_ms`: максимальный лаг для считания пары каузальной
    pub fn discover_from_events(&mut self, events: &[(&str, i64)], max_lag_ms: i64) {
        for i in 0..events.len() {
            for j in (i + 1)..events.len() {
                let (cause, t1) = events[i];
                let (effect, t2) = events[j];
                let lag = t2 - t1;

                if lag <= 0 || lag > max_lag_ms { continue; }
                if cause == effect { continue; }

                // Наблюдаем пару с подтверждением
                self.observe(cause, effect, true, lag as f64);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_strength_starts_neutral() {
        let e = CausalEdge::new("A", "B");
        assert!((e.strength() - 0.5).abs() < 1e-10, "Uniform prior = 0.5");
    }

    #[test]
    fn test_edge_confirms_increase_strength() {
        let mut e = CausalEdge::new("BTC_drop", "ETH_drop");
        for _ in 0..10 {
            e.observe(true, 5000.0);
        }
        assert!(e.strength() > 0.8, "10 confirmations should give strength > 0.8");
        assert!(e.is_significant());
    }

    #[test]
    fn test_graph_observe_and_query() {
        let mut g = CausalGraph::new();
        g.observe("BTC_drop_3pct", "ETH_drop_5pct", true, 3600000.0);
        g.observe("BTC_drop_3pct", "ETH_drop_5pct", true, 4200000.0);

        let effects = g.effects_of("BTC_drop_3pct");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].confirmations, 2);
    }

    #[test]
    fn test_graph_discover_from_events() {
        let mut g = CausalGraph::new();
        let events = vec![
            ("BTC_dump", 1000_i64),
            ("ETH_dump", 2000),
            ("SOL_dump", 3000),
        ];
        g.discover_from_events(&events, 5000);

        // Should discover: BTC→ETH, BTC→SOL, ETH→SOL
        assert_eq!(g.edges.len(), 3);

        let btc_effects = g.effects_of("BTC_dump");
        assert_eq!(btc_effects.len(), 2);
    }

    #[test]
    fn test_predict_returns_significant_only() {
        let mut g = CausalGraph::new();

        // Добавим 15 подтверждений BTC→ETH
        for _ in 0..15 {
            g.observe("BTC_drop", "ETH_drop", true, 5000.0);
        }
        // 3 наблюдения BTC→SOL (не значимо)
        for _ in 0..3 {
            g.observe("BTC_drop", "SOL_drop", true, 8000.0);
        }

        let predictions = g.predict("BTC_drop");
        assert_eq!(predictions.len(), 1, "Only significant edges should be predicted");
        assert_eq!(predictions[0].0, "ETH_drop");
    }
}
