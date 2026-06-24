/// Experience Replay Buffer — хранилище конкретных эпизодов для обучения.
///
/// В отличие от entity.rs (агрегированная статистика: avg PnL, win rate),
/// replay.rs хранит КОНКРЕТНЫЕ трейды с полным контекстом.
///
/// Зачем: Когда Castle видит похожий контекст, он вспоминает
/// КОНКРЕТНЫЙ трейд: "В прошлый раз ETH в volatile при overlap
/// упал на 5% за 10 минут" — а не абстрактную статистику.
///
/// Prioritized Experience Replay (PER):
/// - Более информативные опыты (аномальные, экстремальные) имеют
///   БОЛЬШИЙ приоритет и вспоминаются чаще.
/// - priority = |TD_error| + ε (temporal difference error)
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Один опыт (experience) — полный снимок трейда.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub symbol: String,
    pub pnl: f64,
    pub side: String,           // "Buy" / "Sell"
    pub regime: String,         // "trending_up" / "volatile" / etc.
    pub session: String,        // "Tokyo" / "London" / etc.
    pub hold_duration_ms: i64,
    pub timestamp_ms: i64,
    pub strategy: String,
    /// Приоритет для PER (чем выше — тем чаще вспоминается)
    pub priority: f64,
}

/// Ring buffer с приоритизацией.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBuffer {
    buffer: VecDeque<Experience>,
    capacity: usize,
    pub total_inserted: u64,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            total_inserted: 0,
        }
    }

    /// Добавить опыт. Если буфер полон — выкидываем НАИМЕНЕЕ приоритетный.
    pub fn push(&mut self, mut exp: Experience) {
        // Минимальный приоритет
        if exp.priority < 0.01 {
            exp.priority = 0.01;
        }

        if self.buffer.len() >= self.capacity {
            // Найти и удалить элемент с наименьшим приоритетом
            if let Some(min_idx) = self.buffer.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
            {
                // Только заменяем если новый приоритетнее
                if exp.priority > self.buffer[min_idx].priority {
                    self.buffer.remove(min_idx);
                } else {
                    return; // Новый менее приоритетен — не вставляем
                }
            }
        }

        self.buffer.push_back(exp);
        self.total_inserted += 1;
    }

    /// Количество опытов в буфере.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Найти похожие опыты по символу.
    pub fn recall_by_symbol(&self, symbol: &str, limit: usize) -> Vec<&Experience> {
        let mut results: Vec<&Experience> = self.buffer.iter()
            .filter(|e| e.symbol == symbol)
            .collect();
        results.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    /// Найти похожие опыты по контексту (символ + режим + сессия).
    pub fn recall_by_context(&self, symbol: &str, regime: &str, session: &str, limit: usize) -> Vec<&Experience> {
        let mut results: Vec<&Experience> = self.buffer.iter()
            .filter(|e| {
                e.symbol == symbol && e.regime == regime && e.session == session
            })
            .collect();
        results.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    /// Top-K самых приоритетных опытов (для анализа).
    pub fn top_experiences(&self, limit: usize) -> Vec<&Experience> {
        let mut all: Vec<&Experience> = self.buffer.iter().collect();
        all.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        all.truncate(limit);
        all
    }

    /// Средний PnL по сохранённым опытам для символа.
    pub fn avg_pnl_for_symbol(&self, symbol: &str) -> Option<f64> {
        let pnls: Vec<f64> = self.buffer.iter()
            .filter(|e| e.symbol == symbol)
            .map(|e| e.pnl)
            .collect();
        if pnls.is_empty() { return None; }
        Some(pnls.iter().sum::<f64>() / pnls.len() as f64)
    }
}

/// Вычислить приоритет опыта.
/// Чем больше |PnL| и чем аномальнее — тем выше приоритет.
pub fn calculate_priority(pnl: f64, z_score: f64, is_novel: bool) -> f64 {
    let base = pnl.abs(); // Абсолютный PnL
    let anomaly_boost = z_score.abs().max(1.0); // Z-score множитель
    let novelty_boost = if is_novel { 2.0 } else { 1.0 };
    
    (base * anomaly_boost * novelty_boost).max(0.01)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exp(symbol: &str, pnl: f64, priority: f64) -> Experience {
        Experience {
            symbol: symbol.to_string(),
            pnl,
            side: "Buy".to_string(),
            regime: "trending_up".to_string(),
            session: "London".to_string(),
            hold_duration_ms: 60000,
            timestamp_ms: 1700000000000,
            strategy: "default".to_string(),
            priority,
        }
    }

    #[test]
    fn test_push_and_len() {
        let mut rb = ReplayBuffer::new(10);
        rb.push(make_exp("BTC", 5.0, 1.0));
        rb.push(make_exp("ETH", -3.0, 2.0));
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn test_capacity_evicts_lowest() {
        let mut rb = ReplayBuffer::new(3);
        rb.push(make_exp("A", 1.0, 1.0)); // priority 1
        rb.push(make_exp("B", 2.0, 5.0)); // priority 5
        rb.push(make_exp("C", 3.0, 3.0)); // priority 3
        // Full. Push high priority → should evict A (priority 1)
        rb.push(make_exp("D", 4.0, 10.0));
        assert_eq!(rb.len(), 3);
        assert!(rb.recall_by_symbol("A", 1).is_empty(), "A (lowest priority) should be evicted");
        assert!(!rb.recall_by_symbol("D", 1).is_empty(), "D should be present");
    }

    #[test]
    fn test_low_priority_rejected() {
        let mut rb = ReplayBuffer::new(2);
        rb.push(make_exp("A", 1.0, 5.0));
        rb.push(make_exp("B", 2.0, 10.0));
        // Push something with priority 1 → should be rejected
        rb.push(make_exp("C", 0.5, 0.5));
        assert_eq!(rb.len(), 2);
        assert!(rb.recall_by_symbol("C", 1).is_empty(), "Low priority should be rejected");
    }

    #[test]
    fn test_recall_by_symbol() {
        let mut rb = ReplayBuffer::new(10);
        rb.push(make_exp("BTC", 5.0, 1.0));
        rb.push(make_exp("BTC", -3.0, 2.0));
        rb.push(make_exp("ETH", 7.0, 3.0));

        let btc_results = rb.recall_by_symbol("BTC", 10);
        assert_eq!(btc_results.len(), 2);
        // Highest priority first
        assert!(btc_results[0].priority >= btc_results[1].priority);
    }

    #[test]
    fn test_recall_by_context() {
        let mut rb = ReplayBuffer::new(10);
        rb.push(make_exp("BTC", 5.0, 1.0)); // trending_up + London
        let mut exp2 = make_exp("BTC", -3.0, 2.0);
        exp2.regime = "volatile".to_string();
        rb.push(exp2);

        let results = rb.recall_by_context("BTC", "trending_up", "London", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].pnl, 5.0);
    }

    #[test]
    fn test_top_experiences() {
        let mut rb = ReplayBuffer::new(10);
        rb.push(make_exp("A", 1.0, 1.0));
        rb.push(make_exp("B", 50.0, 100.0));
        rb.push(make_exp("C", 5.0, 5.0));

        let top = rb.top_experiences(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].symbol, "B"); // Highest priority
    }

    #[test]
    fn test_avg_pnl() {
        let mut rb = ReplayBuffer::new(10);
        rb.push(make_exp("BTC", 10.0, 1.0));
        rb.push(make_exp("BTC", -4.0, 1.0));
        rb.push(make_exp("ETH", 20.0, 1.0));

        let avg = rb.avg_pnl_for_symbol("BTC").unwrap();
        assert!((avg - 3.0).abs() < 1e-10, "Avg of 10 and -4 = 3.0, got {}", avg);
        assert!(rb.avg_pnl_for_symbol("DOGE").is_none());
    }

    #[test]
    fn test_priority_calculation() {
        let normal = calculate_priority(5.0, 0.5, false);
        let anomalous = calculate_priority(5.0, 3.0, false);
        let novel_anomalous = calculate_priority(5.0, 3.0, true);

        assert!(anomalous > normal, "Anomalous should have higher priority");
        assert!(novel_anomalous > anomalous, "Novel + anomalous = highest");
    }

    #[test]
    fn test_empty_buffer() {
        let rb = ReplayBuffer::new(10);
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        assert!(rb.recall_by_symbol("BTC", 5).is_empty());
        assert!(rb.top_experiences(5).is_empty());
    }
}
