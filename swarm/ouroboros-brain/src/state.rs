//! Shared state for the Ouroboros swarm.
//! DashMap for lock-free concurrent access (same pattern as Memory Castle).

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

// ═══════════════════════════════════════════════════════════
// SWARM STATE — Центральное хранилище состояния
// ═══════════════════════════════════════════════════════════

pub struct SwarmState {
    /// Текущие данные по каждому символу (цена, OI, funding, etc.)
    pub symbols: DashMap<String, SymbolData>,

    /// Результаты последнего consensus цикла
    pub consensus: DashMap<String, ConsensusResult>,

    /// Счётчик циклов
    pub cycle_count: AtomicU64,

    /// Circuit breaker level: 0=GREEN, 1=YELLOW, 2=RED
    pub circuit_level: AtomicU64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolData {
    pub symbol: String,
    pub price: f64,
    pub price_24h_change: f64,
    pub volume_24h: f64,
    pub volume_ratio: f64,      // vs previous period
    pub funding_rate: f64,
    pub open_interest: f64,
    pub oi_change_pct: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub symbol: String,
    pub final_verdict: Verdict,
    pub confidence: f64,
    pub bull_argument: String,
    pub bear_argument: String,
    pub macro_bias: String,
    pub judge_score: f64,
    pub meta_agreement: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Buy,
    Sell,
    Hold,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Buy => write!(f, "BUY"),
            Verdict::Sell => write!(f, "SELL"),
            Verdict::Hold => write!(f, "HOLD"),
        }
    }
}

impl SwarmState {
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
            consensus: DashMap::new(),
            cycle_count: AtomicU64::new(0),
            circuit_level: AtomicU64::new(0),
        }
    }

    pub fn increment_cycle(&self) -> u64 {
        self.cycle_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_circuit_level(&self, level: u64) {
        self.circuit_level.store(level, Ordering::Relaxed);
        let label = match level {
            0 => "🟢 GREEN",
            1 => "🟡 YELLOW",
            _ => "🔴 RED",
        };
        tracing::info!("Circuit Breaker → {label}");
    }

    pub fn is_trading_allowed(&self) -> bool {
        self.circuit_level.load(Ordering::Relaxed) < 2
    }
}

impl Default for SwarmState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_state_init() {
        let state = SwarmState::new();
        assert_eq!(state.cycle_count.load(Ordering::Relaxed), 0);
        assert!(state.is_trading_allowed());
    }

    #[test]
    fn test_circuit_breaker() {
        let state = SwarmState::new();

        // GREEN → trading allowed
        state.set_circuit_level(0);
        assert!(state.is_trading_allowed());

        // YELLOW → still allowed
        state.set_circuit_level(1);
        assert!(state.is_trading_allowed());

        // RED → trading blocked
        state.set_circuit_level(2);
        assert!(!state.is_trading_allowed());
    }

    #[test]
    fn test_cycle_increment() {
        let state = SwarmState::new();
        assert_eq!(state.increment_cycle(), 1);
        assert_eq!(state.increment_cycle(), 2);
        assert_eq!(state.increment_cycle(), 3);
    }

    #[test]
    fn test_symbol_data() {
        let state = SwarmState::new();
        state.symbols.insert("BTCUSDT".into(), SymbolData {
            symbol: "BTCUSDT".into(),
            price: 96000.0,
            price_24h_change: 2.5,
            volume_24h: 1_000_000.0,
            volume_ratio: 1.5,
            funding_rate: -0.0005,
            open_interest: 500_000.0,
            oi_change_pct: 3.2,
            timestamp: 1715100000,
        });
        assert!(state.symbols.contains_key("BTCUSDT"));
        assert_eq!(state.symbols.get("BTCUSDT").unwrap().price, 96000.0);
    }
}
