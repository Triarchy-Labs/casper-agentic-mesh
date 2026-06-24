//! AI vs Human Benchmark — Comparative scoring engine.
//!
//! Runs a naive SMA crossover strategy in parallel with the AI pipeline
//! to demonstrate AI superiority in real market conditions.
//!
//! The "human" baseline uses a simple technical analysis approach:
//! - SMA(20) > SMA(50) → BUY
//! - SMA(20) < SMA(50) → SELL
//! - Otherwise → HOLD
//!
//! Scoring tracks:
//! - Win rate (AI vs Human)
//! - Risk-adjusted returns (Sharpe-like ratio)
//! - Directional accuracy
//! - Decision speed (AI inference time)
use serde::Serialize;

/// SMA Crossover Strategy — "Human" baseline.
pub struct SmaCrossover {
    short_period: usize,
    long_period: usize,
    price_history: Vec<f64>,
}

/// Benchmark comparison results.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub ai_verdict: String,
    pub human_verdict: String,
    pub ai_score: f64,
    pub human_score: f64,
    pub ai_confidence: f64,
    pub agreement: bool,
    pub ai_inference_ms: u64,
    pub cycle: u64,
    pub symbol: String,
    pub price: f64,
}

/// Cumulative benchmark statistics.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BenchmarkStats {
    pub total_cycles: u64,
    pub ai_correct: u64,
    pub human_correct: u64,
    pub agreements: u64,
    pub ai_avg_confidence: f64,
    pub ai_total_score: f64,
    pub human_total_score: f64,
    pub ai_win_rate: f64,
    pub human_win_rate: f64,
}

impl BenchmarkStats {
    pub fn update(&mut self, result: &BenchmarkResult) {
        self.total_cycles += 1;
        self.ai_total_score += result.ai_score;
        self.human_total_score += result.human_score;
        self.ai_avg_confidence = (self.ai_avg_confidence * (self.total_cycles - 1) as f64
            + result.ai_confidence) / self.total_cycles as f64;
        if result.agreement {
            self.agreements += 1;
        }
    }

    /// Calculate win rates from tracked correct predictions.
    pub fn calculate_rates(&mut self) {
        if self.total_cycles > 0 {
            self.ai_win_rate = self.ai_correct as f64 / self.total_cycles as f64;
            self.human_win_rate = self.human_correct as f64 / self.total_cycles as f64;
        }
    }
}

impl SmaCrossover {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
            price_history: Vec::new(),
        }
    }

    /// Standard SMA(20)/SMA(50) config.
    pub fn default_config() -> Self {
        Self::new(20, 50)
    }

    /// Feed a new price and get a signal.
    pub fn signal(&mut self, price: f64) -> HumanSignal {
        self.price_history.push(price);

        // Need enough data for the long period
        if self.price_history.len() < self.long_period {
            return HumanSignal {
                verdict: "HOLD".into(),
                score: 0.0,
                sma_short: 0.0,
                sma_long: 0.0,
                reason: format!("Insufficient data ({}/{})", self.price_history.len(), self.long_period),
            };
        }

        let len = self.price_history.len();
        let sma_short = self.sma(len - self.short_period, len);
        let sma_long = self.sma(len - self.long_period, len);

        let spread = (sma_short - sma_long) / sma_long * 100.0;

        let (verdict, score, reason) = if spread > 0.5 {
            ("BUY".into(), spread.min(3.0), "SMA20 > SMA50 (golden cross)".into())
        } else if spread < -0.5 {
            ("SELL".into(), spread.max(-3.0), "SMA20 < SMA50 (death cross)".into())
        } else {
            ("HOLD".into(), 0.0, "SMA20 ≈ SMA50 (no clear signal)".into())
        };

        HumanSignal { verdict, score, sma_short, sma_long, reason }
    }

    fn sma(&self, start: usize, end: usize) -> f64 {
        let slice = &self.price_history[start..end];
        slice.iter().sum::<f64>() / slice.len() as f64
    }

    /// Seed initial price history for immediate signal generation.
    pub fn seed(&mut self, prices: &[f64]) {
        self.price_history.extend_from_slice(prices);
    }

    /// Generate synthetic price history from a single price point.
    /// Simulates recent price action around the current price with realistic noise.
    pub fn seed_synthetic(&mut self, current_price: f64, change_24h: f64) {
        let n = self.long_period;
        // Walk backward from current price using 24h change as trend
        let hourly_change = change_24h / 24.0 / 100.0;
        for i in (0..n).rev() {
            let noise = ((i as f64 * 7.3).sin() * 0.002) + ((i as f64 * 3.1).cos() * 0.001);
            let price = current_price * (1.0 - hourly_change * i as f64 + noise);
            self.price_history.push(price);
        }
    }
}

/// Signal from the "human" SMA strategy.
#[derive(Debug, Clone, Serialize)]
pub struct HumanSignal {
    pub verdict: String,
    pub score: f64,
    pub sma_short: f64,
    pub sma_long: f64,
    pub reason: String,
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_crossover_bullish() {
        let mut sma = SmaCrossover::new(3, 5);
        // Feed rising prices
        for p in &[10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0] {
            sma.signal(*p);
        }
        let sig = sma.signal(13.5);
        assert_eq!(sig.verdict, "BUY");
        assert!(sig.score > 0.0);
    }

    #[test]
    fn test_sma_crossover_bearish() {
        let mut sma = SmaCrossover::new(3, 5);
        // Feed declining prices
        for p in &[13.0, 12.5, 12.0, 11.5, 11.0, 10.5, 10.0] {
            sma.signal(*p);
        }
        let sig = sma.signal(9.5);
        assert_eq!(sig.verdict, "SELL");
        assert!(sig.score < 0.0);
    }

    #[test]
    fn test_sma_insufficient_data() {
        let mut sma = SmaCrossover::new(3, 5);
        let sig = sma.signal(10.0);
        assert_eq!(sig.verdict, "HOLD");
        assert!(sig.reason.contains("Insufficient"));
    }

    #[test]
    fn test_seed_synthetic() {
        let mut sma = SmaCrossover::default_config();
        sma.seed_synthetic(0.6443, 2.33);
        assert!(sma.price_history.len() >= 50);
        let sig = sma.signal(0.6443);
        assert_ne!(sig.verdict, ""); // Should produce a valid signal
    }

    #[test]
    fn test_benchmark_stats() {
        let mut stats = BenchmarkStats::default();
        let result = BenchmarkResult {
            ai_verdict: "BUY".into(), human_verdict: "HOLD".into(),
            ai_score: 2.5, human_score: 0.0,
            ai_confidence: 75.0, agreement: false,
            ai_inference_ms: 150, cycle: 1,
            symbol: "MNT".into(), price: 0.6443,
        };
        stats.update(&result);
        assert_eq!(stats.total_cycles, 1);
        assert_eq!(stats.agreements, 0);
        assert!((stats.ai_avg_confidence - 75.0).abs() < 0.01);
    }
}
