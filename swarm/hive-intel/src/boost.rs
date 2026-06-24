use std::collections::BTreeMap;
use dashmap::DashMap;
use crate::entity::MemoryEntity;

/// ПРАВОЕ ПОЛУШАРИЕ V2: alpha_boost с DNA и directional bias.
/// Выявляет и усиливает прибыльные монеты (win streaks, high PF).
pub fn evaluate_alpha_boost(graph: &DashMap<String, MemoryEntity>, output_path: &str) {
    let mut boost_map = BTreeMap::new();

    for entry in graph.iter() {
        let symbol = entry.key();
        let entity = entry.value();
        let is_hot_streak = entity.current_win_streak >= 3;
        let is_proven_alpha = entity.profit_factor >= 1.5 && entity.net_pnl > 0.0 && entity.trade_count >= 5;
        let is_pf_monster = entity.profit_factor >= 2.5 && entity.trade_count >= 3;

        if is_hot_streak || is_proven_alpha || is_pf_monster {
            let multiplier = if entity.profit_factor >= 3.0 || entity.current_win_streak >= 5 {
                2.0
            } else if entity.profit_factor >= 2.0 || entity.current_win_streak >= 3 {
                1.5
            } else {
                1.25
            };

            let status = if multiplier >= 2.0 { "MONSTER" }
                else if multiplier >= 1.5 { "S_TIER" }
                else { "BOOSTED" };

            boost_map.insert(symbol.clone(), serde_json::json!({
                "status": status,
                "leverage_multiplier": multiplier,
                "current_win_streak": entity.current_win_streak,
                "max_win_streak": entity.max_win_streak,
                "net_pnl": entity.net_pnl,
                "profit_factor": entity.profit_factor,
                "win_rate": entity.win_rate,
                "best_side": entity.best_side,
                "buy_pnl": entity.buy_pnl,
                "sell_pnl": entity.sell_pnl,
                "reason": format!("PF={:.2} WR={:.0}% WS:{} Net:{:.2} Best:{}",
                    entity.profit_factor, entity.win_rate * 100.0,
                    entity.current_win_streak, entity.net_pnl, entity.best_side)
            }));
        }
    }

    if let Ok(json_str) = serde_json::to_string_pretty(&boost_map) {
        let _ = std::fs::write(output_path, json_str);
        if !boost_map.is_empty() {
            println!("[RIGHT HEMISPHERE] alpha_boost.json: {} assets boosted.", boost_map.len());
        }
    }
}

/// Проверяет, заслуживает ли символ буста (без записи в файл).
pub fn should_boost(entity: &MemoryEntity) -> bool {
    let is_hot_streak = entity.current_win_streak >= 3;
    let is_proven_alpha = entity.profit_factor >= 1.5 && entity.net_pnl > 0.0 && entity.trade_count >= 5;
    let is_pf_monster = entity.profit_factor >= 2.5 && entity.trade_count >= 3;
    is_hot_streak || is_proven_alpha || is_pf_monster
}

/// Вычисляет множитель буста.
pub fn boost_multiplier(entity: &MemoryEntity) -> f64 {
    if entity.profit_factor >= 3.0 || entity.current_win_streak >= 5 { 2.0 }
    else if entity.profit_factor >= 2.0 || entity.current_win_streak >= 3 { 1.5 }
    else if should_boost(entity) { 1.25 }
    else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(symbol: &str, win_streak: i32, pf: f64, net_pnl: f64, trades: i32) -> MemoryEntity {
        let mut e = MemoryEntity::new(symbol);
        e.current_win_streak = win_streak;
        e.profit_factor = pf;
        e.net_pnl = net_pnl;
        e.trade_count = trades;
        e
    }

    #[test]
    fn test_boost_hot_streak() {
        let e = make_entity("HOT", 3, 1.0, 5.0, 5);
        assert!(should_boost(&e), "3 win streak should trigger boost");
    }

    #[test]
    fn test_boost_proven_alpha() {
        let e = make_entity("ALPHA", 0, 1.8, 20.0, 10);
        assert!(should_boost(&e), "PF >= 1.5 + positive PnL + 5 trades = alpha");
    }

    #[test]
    fn test_boost_pf_monster() {
        let e = make_entity("MONSTER", 0, 3.0, 1.0, 3);
        assert!(should_boost(&e), "PF >= 2.5 with 3 trades = monster");
    }

    #[test]
    fn test_no_boost_mediocre() {
        let e = make_entity("MEH", 1, 1.2, 5.0, 10);
        assert!(!should_boost(&e), "Mediocre entity should NOT be boosted");
    }

    #[test]
    fn test_multiplier_tiers() {
        let monster = make_entity("A", 5, 3.5, 100.0, 20);
        assert_eq!(boost_multiplier(&monster), 2.0);

        let solid = make_entity("B", 3, 2.0, 50.0, 10);
        assert_eq!(boost_multiplier(&solid), 1.5);

        let normal = make_entity("C", 0, 1.0, 5.0, 5);
        assert_eq!(boost_multiplier(&normal), 1.0);
    }
}
