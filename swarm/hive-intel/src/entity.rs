use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Сущность Памяти: один актив (монета) глазами Роя.
/// V3: 25 полей — полный генетический паспорт + история для МОЗГА.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryEntity {
    pub entity_id: String,
    pub trade_count: i32,

    // ═══ ЛЕВОЕ ПОЛУШАРИЕ (Боль / Щит) ═══
    pub current_loss_streak: i32,
    pub max_loss_streak: i32,
    pub max_loss: f64,
    pub total_loss: f64,
    #[serde(default)]
    pub loss_count: i32,

    // ═══ ПРАВОЕ ПОЛУШАРИЕ (Успех / Копьё) ═══
    pub current_win_streak: i32,
    pub max_win_streak: i32,
    pub max_win: f64,
    pub total_profit: f64,
    #[serde(default)]
    pub win_count: i32,

    // ═══ ИТОГ ═══
    pub net_pnl: f64,

    // ═══ DNA V3: Расширенный Профиль ═══
    #[serde(default)]
    pub avg_win_size: f64,
    #[serde(default)]
    pub avg_loss_size: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub avg_hold_duration_ms: i64,
    #[serde(default)]
    pub best_side: String,       // "Buy" или "Sell"
    #[serde(default)]
    pub buy_pnl: f64,
    #[serde(default)]
    pub sell_pnl: f64,
    #[serde(default)]
    pub buy_count: i32,
    #[serde(default)]
    pub sell_count: i32,
    #[serde(default)]
    pub last_trade_ts: i64,      // ms timestamp
    #[serde(default = "default_pf")]
    pub profit_factor: f64,      // total_profit / |total_loss|

    // ═══ МОЗГ V3: История для когнитивных модулей ═══
    /// Последние N значений PnL (для drift/CUSUM/EWMA). VecDeque = O(1) ring buffer.
    #[serde(default, serialize_with = "ser_deque_f64", deserialize_with = "de_deque_f64")]
    pub recent_pnl: VecDeque<f64>,
    /// Последние N hold durations (для disposition detection).
    #[serde(default, serialize_with = "ser_deque_i64", deserialize_with = "de_deque_i64")]
    pub recent_hold_ms: VecDeque<i64>,
}

const RECENT_BUFFER_SIZE: usize = 50;

fn default_pf() -> f64 { 1.0 }

// Serde helpers: VecDeque ↔ Vec for JSON compatibility
fn ser_deque_f64<S: serde::Serializer>(d: &VecDeque<f64>, s: S) -> Result<S::Ok, S::Error> {
    let v: Vec<f64> = d.iter().copied().collect();
    v.serialize(s)
}
fn de_deque_f64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<VecDeque<f64>, D::Error> {
    let v: Vec<f64> = Vec::deserialize(d)?;
    Ok(v.into())
}
fn ser_deque_i64<S: serde::Serializer>(d: &VecDeque<i64>, s: S) -> Result<S::Ok, S::Error> {
    let v: Vec<i64> = d.iter().copied().collect();
    v.serialize(s)
}
fn de_deque_i64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<VecDeque<i64>, D::Error> {
    let v: Vec<i64> = Vec::deserialize(d)?;
    Ok(v.into())
}

impl MemoryEntity {
    /// Создаёт пустую сущность для нового символа.
    pub fn new(symbol: &str) -> Self {
        Self {
            entity_id: symbol.to_string(),
            trade_count: 0,
            current_loss_streak: 0, max_loss_streak: 0,
            max_loss: 0.0, total_loss: 0.0, loss_count: 0,
            current_win_streak: 0, max_win_streak: 0,
            max_win: 0.0, total_profit: 0.0, win_count: 0,
            net_pnl: 0.0,
            avg_win_size: 0.0, avg_loss_size: 0.0,
            win_rate: 0.0, avg_hold_duration_ms: 0,
            best_side: String::new(),
            buy_pnl: 0.0, sell_pnl: 0.0,
            buy_count: 0, sell_count: 0,
            last_trade_ts: 0, profit_factor: 1.0,
            recent_pnl: VecDeque::new(),
            recent_hold_ms: VecDeque::new(),
        }
    }

    /// Добавить PnL в кольцевой буфер (последние 50). O(1) с VecDeque.
    pub fn push_pnl(&mut self, pnl: f64) {
        self.recent_pnl.push_back(pnl);
        if self.recent_pnl.len() > RECENT_BUFFER_SIZE {
            self.recent_pnl.pop_front();
        }
    }

    /// Добавить hold duration в кольцевой буфер. O(1).
    pub fn push_hold_ms(&mut self, hold_ms: i64) {
        if hold_ms > 0 {
            self.recent_hold_ms.push_back(hold_ms);
            if self.recent_hold_ms.len() > RECENT_BUFFER_SIZE {
                self.recent_hold_ms.pop_front();
            }
        }
    }

    /// recent_pnl как slice (для модулей которые ожидают &[f64]).
    pub fn recent_pnl_slice(&self) -> Vec<f64> {
        self.recent_pnl.iter().copied().collect()
    }

    /// recent_hold_ms как Vec (для drift::detect_disposition).
    pub fn recent_hold_ms_vec(&self) -> Vec<i64> {
        self.recent_hold_ms.iter().copied().collect()
    }

    /// Пересчитывает производные метрики (PF, WR, best_side, avg sizes).
    pub fn recalculate_derived(&mut self) {
        if self.trade_count > 0 {
            // Profit Factor
            let w = self.total_profit.max(0.0);
            let l = self.total_loss.abs().max(0.001);
            self.profit_factor = w / l;

            // Win Rate — ПРАВИЛЬНЫЙ: % выигрышных трейдов
            self.win_rate = if self.trade_count > 0 {
                self.win_count as f64 / self.trade_count as f64
            } else {
                0.0
            };

            // Average sizes (теперь из entity, не из внешних HashMap)
            if self.win_count > 0 {
                self.avg_win_size = self.total_profit / self.win_count as f64;
            }
            if self.loss_count > 0 {
                self.avg_loss_size = self.total_loss / self.loss_count as f64;
            }
        }

        // Best side
        self.best_side = if self.buy_pnl > self.sell_pnl {
            "Buy".to_string()
        } else if self.sell_pnl > self.buy_pnl {
            "Sell".to_string()
        } else {
            "Neutral".to_string()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entity() {
        let e = MemoryEntity::new("BTCUSDT");
        assert_eq!(e.entity_id, "BTCUSDT");
        assert_eq!(e.trade_count, 0);
        assert_eq!(e.win_count, 0);
        assert_eq!(e.loss_count, 0);
        assert!(e.recent_pnl.is_empty());
    }

    #[test]
    fn test_win_rate_correct() {
        let mut e = MemoryEntity::new("TEST");
        e.trade_count = 10;
        e.win_count = 7;
        e.loss_count = 3;
        e.total_profit = 100.0;
        e.total_loss = -30.0;
        e.recalculate_derived();
        assert!((e.win_rate - 0.7).abs() < 1e-10, "Win rate should be 0.7, got {}", e.win_rate);
    }

    #[test]
    fn test_push_pnl_ring_buffer() {
        let mut e = MemoryEntity::new("TEST");
        for i in 0..60 {
            e.push_pnl(i as f64);
        }
        assert_eq!(e.recent_pnl.len(), 50, "Buffer should cap at 50");
        assert_eq!(e.recent_pnl[0], 10.0, "Oldest should be 10 (0-9 dropped)");
    }

    #[test]
    fn test_avg_sizes_from_entity() {
        let mut e = MemoryEntity::new("TEST");
        e.trade_count = 4;
        e.win_count = 2;
        e.loss_count = 2;
        e.total_profit = 100.0;
        e.total_loss = -40.0;
        e.recalculate_derived();
        assert!((e.avg_win_size - 50.0).abs() < 1e-10);
        assert!((e.avg_loss_size - (-20.0)).abs() < 1e-10);
    }
}
