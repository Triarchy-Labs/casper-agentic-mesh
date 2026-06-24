use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use dashmap::DashMap;
use crate::entity::MemoryEntity;

/// ЛЕВОЕ ПОЛУШАРИЕ V2: tilt_lock с учётом profit_factor.
/// Блокирует токсичные монеты (loss streak, low PF, pain threshold).
pub fn evaluate_tilt_reflex(graph: &DashMap<String, MemoryEntity>, output_path: &str) {
    let mut reflex_map = BTreeMap::new();
    
    for entry in graph.iter() {
        let symbol = entry.key();
        let entity = entry.value();
        let should_lock = should_tilt_lock(entity); // F10: ADAPTIVE thresholds

        if should_lock {
            let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            // Длительность бана зависит от тяжести
            let ban_hours = if entity.profit_factor < 0.2 { 12 }
                else if entity.current_loss_streak >= 5 { 8 }
                else { 4 };
            let unlock_ts = now_ms + (ban_hours as u64 * 3600 * 1000);

            let reason = if entity.profit_factor < 0.3 {
                format!("TOXIC_PF: PF={:.2}, Net={:.2}", entity.profit_factor, entity.net_pnl)
            } else if entity.net_pnl < -15.0 {
                format!("PAIN_THRESHOLD: Net={:.2}", entity.net_pnl)
            } else {
                format!("LOSS_STREAK: {} consecutive losses", entity.current_loss_streak)
            };

            reflex_map.insert(symbol.clone(), serde_json::json!({
                "locked": true,
                "reason": reason,
                "unlock_timestamp_ms": unlock_ts,
                "profit_factor": entity.profit_factor,
                "net_pnl": entity.net_pnl,
                "ban_hours": ban_hours
            }));
        }
    }

    if let Ok(json_str) = serde_json::to_string_pretty(&reflex_map) {
        let _ = std::fs::write(output_path, json_str);
        if !reflex_map.is_empty() {
            println!("[LEFT HEMISPHERE] tilt_lock.json: {} assets blocked.", reflex_map.len());
        }
    }
}

/// Проверяет, должен ли символ быть заблокирован (без записи в файл).
/// Использует АДАПТИВНЫЕ пороги (F10 Blueprint).
pub fn should_tilt_lock(entity: &MemoryEntity) -> bool {
    let streak_threshold = adaptive_streak_threshold(entity);
    let pain_threshold = adaptive_pain_threshold(entity);

    entity.current_loss_streak >= streak_threshold ||
    (entity.profit_factor < 0.3 && entity.trade_count >= 5) ||
    entity.net_pnl < pain_threshold
}

/// Адаптивный порог loss streak (F10).
/// Волатильные монеты (высокий avg loss) получают бо́льший порог.
/// Минимум 3, максимум 8.
pub fn adaptive_streak_threshold(entity: &MemoryEntity) -> i32 {
    if entity.trade_count < 10 {
        return 3; // Недостаточно данных — дефолт
    }
    // Если avg_loss маленький → монета стабильная → низкий порог
    // Если avg_loss большой → монета волатильная → высокий порог (стрики нормальны)
    let avg_loss = entity.avg_loss_size.abs();
    let avg_win = entity.avg_win_size.abs().max(0.01);
    let ratio = avg_loss / avg_win; // < 1 = хороший RR, > 1 = плохой RR

    if ratio > 1.5 {
        3 // Плохой RR — блокируем быстро
    } else if ratio > 0.8 {
        4 // Средний RR
    } else if entity.win_rate > 0.55 {
        5 // Хороший WR + RR — даём больше пространства
    } else {
        3
    }
}

/// Адаптивный порог боли (net_pnl) — масштабируется под среднюю позицию.
/// Для BTC ($100 avg loss) → -$300 порог.
/// Для SHIB ($0.5 avg loss) → -$1.5 порог.
pub fn adaptive_pain_threshold(entity: &MemoryEntity) -> f64 {
    if entity.trade_count < 5 {
        return -15.0; // Дефолт
    }
    let avg_loss = entity.avg_loss_size.abs().max(0.5);
    -(avg_loss * 3.0).max(5.0) // 3x avg loss, минимум -$5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(symbol: &str, loss_streak: i32, pf: f64, net_pnl: f64, trades: i32) -> MemoryEntity {
        let mut e = MemoryEntity::new(symbol);
        e.current_loss_streak = loss_streak;
        e.profit_factor = pf;
        e.net_pnl = net_pnl;
        e.trade_count = trades;
        e
    }

    #[test]
    fn test_tilt_loss_streak() {
        let e = make_entity("BAD", 3, 1.0, 0.0, 10);
        assert!(should_tilt_lock(&e), "3 loss streak should trigger tilt");
    }

    #[test]
    fn test_tilt_toxic_pf() {
        let e = make_entity("TOXIC", 0, 0.2, 5.0, 10);
        assert!(should_tilt_lock(&e), "PF < 0.3 with 5+ trades should trigger");
    }

    #[test]
    fn test_tilt_pain_threshold() {
        let e = make_entity("PAIN", 0, 1.5, -20.0, 3);
        assert!(should_tilt_lock(&e), "Net PnL < -15 should trigger");
    }

    #[test]
    fn test_no_tilt_healthy() {
        let e = make_entity("GOOD", 1, 1.5, 10.0, 10);
        assert!(!should_tilt_lock(&e), "Healthy entity should NOT be tilted");
    }

    #[test]
    fn test_no_tilt_low_pf_few_trades() {
        let e = make_entity("NEW", 0, 0.2, 0.0, 3);
        assert!(!should_tilt_lock(&e), "Low PF with <5 trades should NOT trigger (insufficient data)");
    }

    #[test]
    fn test_adaptive_streak_default_for_new() {
        let e = make_entity("NEW", 0, 1.0, 0.0, 5);
        assert_eq!(adaptive_streak_threshold(&e), 3, "New entity (<10 trades) should use default 3");
    }

    #[test]
    fn test_adaptive_streak_scales_with_rr() {
        let mut e = make_entity("GOOD", 0, 1.5, 20.0, 15);
        e.avg_loss_size = 2.0;
        e.avg_win_size = 5.0;
        e.win_rate = 0.6;
        // ratio = 2/5 = 0.4 < 0.8 and WR > 0.55 → should be 5
        assert_eq!(adaptive_streak_threshold(&e), 5, "Good RR + WR should get streak threshold 5");
    }

    #[test]
    fn test_adaptive_pain_scales_with_avg_loss() {
        let mut e = make_entity("BTC", 0, 1.0, 0.0, 10);
        e.avg_loss_size = 100.0;
        let threshold = adaptive_pain_threshold(&e);
        assert!(threshold < -200.0, "BTC with $100 avg loss should have pain < -$200, got {}", threshold);

        let mut e2 = make_entity("SHIB", 0, 1.0, 0.0, 10);
        e2.avg_loss_size = 0.5;
        let threshold2 = adaptive_pain_threshold(&e2);
        assert!(threshold2 > -10.0, "SHIB with $0.5 avg loss should have modest threshold, got {}", threshold2);
    }
}
