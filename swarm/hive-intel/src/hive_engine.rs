use dashmap::DashMap;
use crate::entity::MemoryEntity;
use crate::snapshot;
use std::collections::HashMap;

/// F1 Blueprint: Lock-free concurrent HashMap.
/// 100+ потоков читают одновременно, НОЛЬ contention.
pub struct HiveMindEngine {
    pub graph: DashMap<String, MemoryEntity>,
}

impl Default for HiveMindEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HiveMindEngine {
    pub fn new() -> Self {
        Self { graph: DashMap::new() }
    }

    pub fn load_snapshot(path: &str) -> Self {
        let hashmap = snapshot::load_snapshot(path);
        let graph = DashMap::new();
        for (k, v) in hashmap {
            graph.insert(k, v);
        }
        Self { graph }
    }

    pub fn save_snapshot(&self, path: &str) {
        let hashmap: HashMap<String, MemoryEntity> = self.graph.iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        snapshot::save_snapshot(&hashmap, path);
    }

    /// Экспорт в HashMap для совместимости (brain, tilt, boost).
    pub fn snapshot_hashmap(&self) -> HashMap<String, MemoryEntity> {
        self.graph.iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect()
    }

    /// V3: insert_trade — lock-free через DashMap.
    pub fn insert_trade(&self, symbol: &str, pnl: f64, side: &str, hold_duration_ms: i64, timestamp_ms: i64) {
        let mut entry = self.graph.entry(symbol.to_string())
            .or_insert_with(|| MemoryEntity::new(symbol));

        entry.trade_count += 1;
        entry.net_pnl += pnl;
        entry.last_trade_ts = timestamp_ms;

        // Ring buffer для когнитивных модулей
        entry.push_pnl(pnl);
        entry.push_hold_ms(hold_duration_ms);

        // Directional stats
        match side {
            "Buy" => { entry.buy_pnl += pnl; entry.buy_count += 1; }
            "Sell" => { entry.sell_pnl += pnl; entry.sell_count += 1; }
            _ => {}
        }

        // Hold duration (running average)
        if hold_duration_ms > 0 {
            let n = entry.trade_count as i64;
            entry.avg_hold_duration_ms = ((entry.avg_hold_duration_ms * (n - 1)) + hold_duration_ms) / n;
        }

        if pnl < 0.0 {
            // Левое полушарие: регистрация боли
            entry.current_loss_streak += 1;
            entry.current_win_streak = 0;
            entry.total_loss += pnl;
            entry.loss_count += 1;
            if pnl < entry.max_loss { entry.max_loss = pnl; }
            if entry.current_loss_streak > entry.max_loss_streak {
                entry.max_loss_streak = entry.current_loss_streak;
            }
        } else if pnl > 0.0 {
            // Правое полушарие: регистрация успеха
            entry.current_win_streak += 1;
            entry.current_loss_streak = 0;
            entry.total_profit += pnl;
            entry.win_count += 1;
            if pnl > entry.max_win { entry.max_win = pnl; }
            if entry.current_win_streak > entry.max_win_streak {
                entry.max_win_streak = entry.current_win_streak;
            }
        }

        // Recalculate derived (PF, WR, avg sizes, best_side)
        entry.recalculate_derived();
        
        println!("[HIVE_MIND] {} | T:{} | W:{} L:{} | LS:{} | WS:{} | PF:{:.2} | WR:{:.0}% | Net:{:.2} | Best:{}",
            symbol, entry.trade_count, entry.win_count, entry.loss_count,
            entry.current_loss_streak, entry.current_win_streak,
            entry.profit_factor, entry.win_rate * 100.0, entry.net_pnl, entry.best_side);
    }

    /// Обратная совместимость: вызов без доп. полей
    pub fn insert_trade_simple(&self, symbol: &str, pnl: f64) {
        self.insert_trade(symbol, pnl, "", 0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_trade_updates_counts() {
        let engine = HiveMindEngine::new();
        engine.insert_trade("BTCUSDT", 10.0, "Buy", 5000, 1000);
        engine.insert_trade("BTCUSDT", -3.0, "Sell", 3000, 2000);
        engine.insert_trade("BTCUSDT", 7.0, "Buy", 4000, 3000);
        
        let e = engine.graph.get("BTCUSDT").unwrap();
        assert_eq!(e.trade_count, 3);
        assert_eq!(e.win_count, 2);
        assert_eq!(e.loss_count, 1);
        assert!((e.win_rate - 2.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_recent_pnl_populated() {
        let engine = HiveMindEngine::new();
        engine.insert_trade("ETH", 5.0, "", 0, 0);
        engine.insert_trade("ETH", -2.0, "", 0, 0);
        
        let e = engine.graph.get("ETH").unwrap();
        assert_eq!(e.recent_pnl.len(), 2);
        assert_eq!(e.recent_pnl[0], 5.0);
        assert_eq!(e.recent_pnl[1], -2.0);
    }

    #[test]
    fn test_persist_survives_snapshot() {
        let engine = HiveMindEngine::new();
        engine.insert_trade("SOL", 20.0, "Buy", 1000, 1);
        engine.insert_trade("SOL", -5.0, "Sell", 2000, 2);
        
        let tmp = std::env::temp_dir().join("test_engine_persist.json");
        let path = tmp.to_str().unwrap();
        engine.save_snapshot(path);
        
        let loaded = HiveMindEngine::load_snapshot(path);
        let e = loaded.graph.get("SOL").unwrap();
        assert_eq!(e.win_count, 1, "win_count must persist");
        assert_eq!(e.loss_count, 1, "loss_count must persist");
        assert_eq!(e.recent_pnl.len(), 2, "recent_pnl must persist");
        
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.replace(".json", ".bin"));
    }

    #[test]
    fn test_snapshot_hashmap_export() {
        let engine = HiveMindEngine::new();
        engine.insert_trade("A", 10.0, "Buy", 0, 0);
        engine.insert_trade("B", -5.0, "Sell", 0, 0);
        let map = engine.snapshot_hashmap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("A"));
        assert!(map.contains_key("B"));
    }
}
