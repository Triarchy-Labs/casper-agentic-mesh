/// Order Book Imbalance — Институциональный анализ перекоса ордеров.
///
/// DONOR: Концепция из HFT Imbalance Predator Skill + nautilus_trader orderbook.
///
/// Суть: Smart Money смотрит НЕ на RSI/MACD (запаздывающие индикаторы),
/// а на РЕАЛЬНЫЙ перекос bid/ask в стакане (ORDER FLOW).
///
/// Если покупателей в 3x больше чем продавцов → цена СКОРЕЕ ВСЕГО пойдёт вверх.
/// Это РЕНТГЕН рынка — видишь куда пойдёт цена ДО того как она пойдёт.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ════════════════════════════════════════════════════════════════
// Order Book Snapshot
// ════════════════════════════════════════════════════════════════

/// Один уровень в стакане (bid или ask).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Снимок order book (bid/ask).
#[derive(Debug, Clone)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub bids: Vec<BookLevel>,  // Покупатели (отсортированы по цене DESC)
    pub asks: Vec<BookLevel>,  // Продавцы (отсортированы по цене ASC)
    pub timestamp_ms: i64,
}

// ════════════════════════════════════════════════════════════════
// Imbalance Analysis Result
// ════════════════════════════════════════════════════════════════

/// Направление перекоса.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImbalanceBias {
    /// Перекос в сторону покупателей → цена ВВЕРХ.
    BullishPressure,
    /// Перекос в сторону продавцов → цена ВНИЗ.
    BearishPressure,
    /// Равновесие → НЕ ТОРГУЕМ.
    Neutral,
}

/// Результат анализа перекоса.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImbalanceResult {
    /// Imbalance Ratio: (bid_vol - ask_vol) / (bid_vol + ask_vol)
    /// Диапазон: [-1.0, +1.0]
    pub ratio: f64,
    /// Направление перекоса.
    pub bias: ImbalanceBias,
    /// Суммарный объём bid.
    pub bid_volume: f64,
    /// Суммарный объём ask.
    pub ask_volume: f64,
    /// Объёмный дисбаланс на ПЕРВЫХ N уровнях (top-of-book).
    pub top_ratio: f64,
    /// Volume-Weighted Average Price покупателей.
    pub bid_vwap: f64,
    /// Volume-Weighted Average Price продавцов.
    pub ask_vwap: f64,
    /// Spread в % от mid price.
    pub spread_pct: f64,
    /// Уверенность сигнала [0.0, 1.0].
    pub confidence: f64,
}

// ════════════════════════════════════════════════════════════════
// Cumulative Volume Delta (CVD)
// ════════════════════════════════════════════════════════════════

/// Кумулятивная дельта объёмов — разница между агрессивными покупателями и продавцами.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvdTracker {
    pub cumulative_delta: f64,
    pub window: VecDeque<f64>,
    pub window_size: usize,
}

impl CvdTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            cumulative_delta: 0.0,
            window: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Обновить CVD новым тиком.
    /// `delta` = buy_volume - sell_volume для этого тика.
    pub fn update(&mut self, delta: f64) {
        self.cumulative_delta += delta;
        self.window.push_back(delta);
        if self.window.len() > self.window_size {
            self.window.pop_front();
        }
    }

    /// Текущий тренд CVD за окно.
    pub fn trend(&self) -> f64 {
        if self.window.len() < 2 { return 0.0; }
        let first_half: f64 = self.window.iter().take(self.window.len() / 2).sum();
        let second_half: f64 = self.window.iter().skip(self.window.len() / 2).sum();
        second_half - first_half
    }

    /// CVD ускоряется или замедляется?
    pub fn momentum(&self) -> f64 {
        if self.window.len() < 3 { return 0.0; }
        let n = self.window.len();
        let recent: f64 = self.window.iter().rev().take(n / 3).sum();
        let earlier: f64 = self.window.iter().take(n / 3).sum();
        recent - earlier
    }
}

// ════════════════════════════════════════════════════════════════
// Imbalance Engine
// ════════════════════════════════════════════════════════════════

/// Движок анализа Order Book Imbalance.
#[derive(Debug, Clone)]
pub struct ImbalanceEngine {
    /// Порог Imbalance Ratio для генерации сигнала.
    /// По умолчанию 0.3 (30% перекос).
    pub threshold: f64,
    /// Сколько верхних уровней анализировать для top_ratio.
    pub top_levels: usize,
    /// CVD трекер.
    pub cvd: CvdTracker,
    /// История imbalance ratios (для трендового анализа).
    pub history: VecDeque<f64>,
    pub history_size: usize,
}

impl Default for ImbalanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ImbalanceEngine {
    pub fn new() -> Self {
        Self {
            threshold: 0.3,
            top_levels: 5,
            cvd: CvdTracker::new(100),
            history: VecDeque::with_capacity(200),
            history_size: 200,
        }
    }

    /// Настроить пороги.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_top_levels(mut self, levels: usize) -> Self {
        self.top_levels = levels;
        self
    }

    /// Анализировать снимок order book.
    pub fn analyze(&mut self, book: &OrderBookSnapshot) -> ImbalanceResult {
        // ═══ 1. Полный Imbalance Ratio ═══
        let bid_volume: f64 = book.bids.iter().map(|l| l.quantity).sum();
        let ask_volume: f64 = book.asks.iter().map(|l| l.quantity).sum();
        let total = bid_volume + ask_volume;
        let ratio = if total > 0.0 {
            (bid_volume - ask_volume) / total
        } else { 0.0 };

        // ═══ 2. Top-of-Book Ratio (первые N уровней — самые важные!) ═══
        let top_bid: f64 = book.bids.iter().take(self.top_levels).map(|l| l.quantity).sum();
        let top_ask: f64 = book.asks.iter().take(self.top_levels).map(|l| l.quantity).sum();
        let top_total = top_bid + top_ask;
        let top_ratio = if top_total > 0.0 {
            (top_bid - top_ask) / top_total
        } else { 0.0 };

        // ═══ 3. VWAP для bid/ask ═══
        let bid_vwap = vwap(&book.bids);
        let ask_vwap = vwap(&book.asks);

        // ═══ 4. Spread ═══
        let best_bid = book.bids.first().map(|l| l.price).unwrap_or(0.0);
        let best_ask = book.asks.first().map(|l| l.price).unwrap_or(0.0);
        let mid = (best_bid + best_ask) / 2.0;
        let spread_pct = if mid > 0.0 { ((best_ask - best_bid) / mid) * 100.0 } else { 0.0 };

        // ═══ 5. CVD Update ═══
        self.cvd.update(bid_volume - ask_volume);

        // ═══ 6. Определить bias ═══
        let bias = if ratio > self.threshold && top_ratio > 0.0 {
            ImbalanceBias::BullishPressure
        } else if ratio < -self.threshold && top_ratio < 0.0 {
            ImbalanceBias::BearishPressure
        } else {
            ImbalanceBias::Neutral
        };

        // ═══ 7. Confidence: комбинация ratio + top_ratio + CVD trend ═══
        let ratio_conf = ratio.abs().min(1.0);
        let top_conf = top_ratio.abs().min(1.0);
        let cvd_trend = self.cvd.trend();
        let cvd_conf = if (cvd_trend > 0.0 && ratio > 0.0) || (cvd_trend < 0.0 && ratio < 0.0) {
            0.3  // CVD confirms direction
        } else if cvd_trend.abs() < 0.01 {
            0.0  // CVD neutral
        } else {
            -0.2  // CVD diverges — DANGER
        };
        let confidence = ((ratio_conf * 0.4 + top_conf * 0.4 + cvd_conf) * 0.3).clamp(0.0, 1.0);

        // Track history
        self.history.push_back(ratio);
        if self.history.len() > self.history_size {
            self.history.pop_front();
        }

        ImbalanceResult {
            ratio,
            bias,
            bid_volume,
            ask_volume,
            top_ratio,
            bid_vwap,
            ask_vwap,
            spread_pct,
            confidence,
        }
    }

    /// Imbalance trend: растёт перекос или падает?
    pub fn trend(&self) -> f64 {
        if self.history.len() < 10 { return 0.0; }
        let n = self.history.len();
        let recent: f64 = self.history.iter().rev().take(5).sum::<f64>() / 5.0;
        let earlier: f64 = self.history.iter().skip(n.saturating_sub(10)).take(5).sum::<f64>() / 5.0;
        recent - earlier
    }
}

/// Volume-Weighted Average Price.
fn vwap(levels: &[BookLevel]) -> f64 {
    let total_vol: f64 = levels.iter().map(|l| l.quantity).sum();
    if total_vol <= 0.0 { return 0.0; }
    let weighted: f64 = levels.iter().map(|l| l.price * l.quantity).sum();
    weighted / total_vol
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_book(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> OrderBookSnapshot {
        OrderBookSnapshot {
            symbol: "BTCUSDT".to_string(),
            bids: bids.iter().map(|&(p, q)| BookLevel { price: p, quantity: q }).collect(),
            asks: asks.iter().map(|&(p, q)| BookLevel { price: p, quantity: q }).collect(),
            timestamp_ms: 1000,
        }
    }

    #[test]
    fn test_bullish_imbalance() {
        let mut engine = ImbalanceEngine::new().with_threshold(0.3);
        let book = make_book(
            &[(100.0, 50.0), (99.0, 40.0), (98.0, 30.0)],  // bids = 120
            &[(101.0, 10.0), (102.0, 5.0), (103.0, 5.0)],   // asks = 20
        );
        let result = engine.analyze(&book);

        assert!(result.ratio > 0.3, "Ratio should be bullish: {}", result.ratio);
        assert_eq!(result.bias, ImbalanceBias::BullishPressure);
        assert!(result.bid_volume > result.ask_volume);
    }

    #[test]
    fn test_bearish_imbalance() {
        let mut engine = ImbalanceEngine::new().with_threshold(0.3);
        let book = make_book(
            &[(100.0, 5.0), (99.0, 5.0)],                    // bids = 10
            &[(101.0, 40.0), (102.0, 30.0), (103.0, 20.0)],  // asks = 90
        );
        let result = engine.analyze(&book);

        assert!(result.ratio < -0.3, "Ratio should be bearish: {}", result.ratio);
        assert_eq!(result.bias, ImbalanceBias::BearishPressure);
    }

    #[test]
    fn test_neutral_balanced_book() {
        let mut engine = ImbalanceEngine::new().with_threshold(0.3);
        let book = make_book(
            &[(100.0, 50.0), (99.0, 50.0)],   // bids = 100
            &[(101.0, 48.0), (102.0, 52.0)],   // asks = 100
        );
        let result = engine.analyze(&book);

        assert!(result.ratio.abs() < 0.3, "Should be neutral: {}", result.ratio);
        assert_eq!(result.bias, ImbalanceBias::Neutral);
    }

    #[test]
    fn test_spread_calculation() {
        let mut engine = ImbalanceEngine::new();
        let book = make_book(
            &[(99.95, 10.0)],
            &[(100.05, 10.0)],
        );
        let result = engine.analyze(&book);

        let expected_spread = (100.05 - 99.95) / 100.0 * 100.0; // 0.1%
        assert!((result.spread_pct - expected_spread).abs() < 0.01);
    }

    #[test]
    fn test_vwap_calculation() {
        let levels = vec![
            BookLevel { price: 100.0, quantity: 10.0 },
            BookLevel { price: 99.0, quantity: 20.0 },
        ];
        let v = vwap(&levels);
        // VWAP = (100*10 + 99*20) / 30 = 2980/30 = 99.333...
        assert!((v - 99.3333).abs() < 0.01);
    }

    #[test]
    fn test_cvd_tracker() {
        let mut cvd = CvdTracker::new(10);
        cvd.update(100.0);  // More buys
        cvd.update(50.0);
        cvd.update(-30.0);  // Some sells

        assert!((cvd.cumulative_delta - 120.0).abs() < 1e-10);
        assert_eq!(cvd.window.len(), 3);
    }

    #[test]
    fn test_cvd_trend() {
        let mut cvd = CvdTracker::new(10);
        // Increasing buy pressure
        for i in 0..6 {
            cvd.update(i as f64 * 10.0);
        }
        assert!(cvd.trend() > 0.0, "Increasing buys = positive trend");
    }

    #[test]
    fn test_top_of_book_ratio() {
        let mut engine = ImbalanceEngine::new().with_top_levels(2);
        let book = make_book(
            &[(100.0, 100.0), (99.0, 100.0), (98.0, 1.0)],   // Top 2 = 200
            &[(101.0, 10.0), (102.0, 10.0), (103.0, 1000.0)], // Top 2 = 20
        );
        let result = engine.analyze(&book);

        // Top 2 bids=200, top 2 asks=20 → strong bullish at top
        assert!(result.top_ratio > 0.5, "Top of book very bullish: {}", result.top_ratio);
    }

    #[test]
    fn test_empty_book() {
        let mut engine = ImbalanceEngine::new();
        let book = make_book(&[], &[]);
        let result = engine.analyze(&book);

        assert_eq!(result.ratio, 0.0);
        assert_eq!(result.bias, ImbalanceBias::Neutral);
    }

    #[test]
    fn test_imbalance_history_tracking() {
        let mut engine = ImbalanceEngine::new();
        for i in 0..10 {
            let bid_qty = 50.0 + i as f64 * 10.0; // Growing bid pressure
            let book = make_book(
                &[(100.0, bid_qty)],
                &[(101.0, 30.0)],
            );
            engine.analyze(&book);
        }

        assert_eq!(engine.history.len(), 10);
        assert!(engine.trend() > 0.0, "Growing bid pressure = positive trend");
    }
}
