// src/modules/confidence.rs
// Vector 3: Confidence Score — непрерывная шкала доверия [-5..+5]
// Заменяет бинарный "locked/boosted" на плавную кривую на основе DNA профиля монеты.
use serde_json::Value;
use crate::safe_io::data_file;
use std::sync::Mutex;
use std::time::Instant;

pub struct ConfidenceEngine;

// BUG-11 FIX: Snapshot Cache — 15-second TTL (was reading 4×/cycle)
static SNAPSHOT_CACHE: Mutex<Option<(Instant, Value)>> = Mutex::new(None);

impl ConfidenceEngine {
    /// Читает alpha_boost.json и tilt_lock.json, вычисляет Confidence Score [-5..+5]
    /// Положительный = агрессивнее. Отрицательный = осторожнее/бан.
    /// 0.0 = неизвестная монета (работают только индикаторы).
    pub fn calculate(symbol: &str) -> f64 {
        let tilt_path = data_file("tilt_lock.json");

        // Tilt lock = hard negative
        if let Ok(content) = std::fs::read_to_string(tilt_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(entry) = json.get(symbol) {
                    if entry["locked"].as_bool().unwrap_or(false) {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).expect("clock before UNIX epoch").as_millis() as u64;
                        if let Some(unlock) = entry["unlock_timestamp_ms"].as_u64() {
                            if now_ms < unlock {
                                return -5.0; // Жёсткий бан
                            }
                        }
                    }
                }
            }
        }

        // Read DNA from cached snapshot
        if let Some(json) = Self::read_snapshot_cached() {
                if let Some(entity) = json.get(symbol) {
                    let trade_count = entity["trade_count"].as_i64().unwrap_or(0);
                    let profit_factor = entity["profit_factor"].as_f64().unwrap_or(1.0);
                    let win_rate = entity["win_rate"].as_f64().unwrap_or(0.5);
                    let net_pnl = entity["net_pnl"].as_f64().unwrap_or(0.0);
                    let current_loss_streak = entity["current_loss_streak"].as_i64().unwrap_or(0);
                    let current_win_streak = entity["current_win_streak"].as_i64().unwrap_or(0);

                    if trade_count < 2 { return 0.0; } // Недостаточно данных

                    // Формула Confidence:
                    // Base = (win_rate - 0.5) × 10  → [-5..+5]
                    // PF modifier: (PF - 1.0) × 2   → [-2..+6] clamped
                    // Streak modifier: ws × 0.5 или -ls × 0.7
                    // Volume of knowledge: log2(trade_count) × 0.3
                    
                    let wr_component = (win_rate - 0.5) * 10.0;
                    let pf_component = ((profit_factor - 1.0) * 2.0).clamp(-3.0, 4.0);
                    let streak_component = (current_win_streak as f64 * 0.5) - (current_loss_streak as f64 * 0.7);
                    let knowledge = (trade_count as f64).log2() * 0.3;
                    
                    // Penalty for deep net loss
                    let pain_penalty = if net_pnl < -10.0 { (net_pnl / 10.0).clamp(-3.0, 0.0) } else { 0.0 };

                    let raw = wr_component + pf_component + streak_component + knowledge + pain_penalty;
                    // FORENSIC-06: reduced from [-5..+5] to [-3..+3] to prevent double-counting with DirBias
                    return raw.clamp(-3.0, 3.0);
            }
        }

        0.0 // Unknown symbol
    }

    /// Vector 2: Directional Bias — возвращает оптимальную сторону
    /// None = нет данных, Some("Buy") = только лонги, Some("Sell") = только шорты
    pub fn get_directional_bias(symbol: &str) -> Option<String> {
        if let Some(json) = Self::read_snapshot_cached() {
                if let Some(entity) = json.get(symbol) {
                    let buy_pnl = entity["buy_pnl"].as_f64().unwrap_or(0.0);
                    let sell_pnl = entity["sell_pnl"].as_f64().unwrap_or(0.0);
                    let buy_count = entity["buy_count"].as_i64().unwrap_or(0);
                    let sell_count = entity["sell_count"].as_i64().unwrap_or(0);

                    // Нужно минимум 3 сделки в каждом направлении для вывода
                    if buy_count >= 3 && sell_count >= 3 {
                        if buy_pnl > 0.0 && sell_pnl < 0.0 { return Some("Buy".to_string()); }
                        if sell_pnl > 0.0 && buy_pnl < 0.0 { return Some("Sell".to_string()); }
                }
            }
        }

        None // Нет достаточных данных или обе стороны одинаковы
    }

    /// P1 #3: Adaptive Cooldown — масштабирует кулдаун по hold_duration из DNA
    /// Быстрые монеты (hold < 5 мин) → короткий кулдаун (5 мин)
    /// Средние (5-30 мин) → стандарт (15-20 мин)
    /// Медленные (hold > 30 мин) → длинный кулдаун (30-60 мин)
    /// Возвращает секунды. fallback_sec используется если нет DNA данных.
    pub fn get_adaptive_cooldown(symbol: &str, fallback_sec: i64) -> i64 {
        if let Some(json) = Self::read_snapshot_cached() {
                if let Some(entity) = json.get(symbol) {
                    let hold_ms = entity["avg_hold_duration_ms"].as_f64().unwrap_or(0.0);
                    let trade_count = entity["trade_count"].as_i64().unwrap_or(0);

                    if trade_count >= 3 && hold_ms > 0.0 {
                        // Cooldown = 2.5 × avg_hold_duration (не менее 5 мин, не более 60 мин)
                        let hold_sec = hold_ms / 1000.0;
                        let cooldown = (hold_sec * 2.5).clamp(300.0, 3600.0) as i64;
                        return cooldown;
                    }
            }
        }

        fallback_sec
    }

    /// P2 #5: Dynamic Imbalance Threshold — корректирует порог по рыночному режиму
    /// trending (|btc_score| > 2.0) → порог ниже (1.15) — тренд подтверждает imbalance
    /// ranging (|btc_score| < 1.0)  → порог выше (1.50) — нужно больше доказательств
    #[allow(dead_code)] // Reserved for future entry gate G5 dynamic integration
    pub fn get_dynamic_imbalance_threshold(btc_score: f64) -> f64 {
        let abs_score = btc_score.abs();
        if abs_score >= 3.0 {
            1.10 // Сильный тренд — imbalance при 1.10 уже сигнал
        } else if abs_score >= 2.0 {
            1.15
        } else if abs_score >= 1.0 {
            1.30 // Умеренный рынок
        } else {
            1.50 // Рендж — нужен сильный перекос
        }
    }

    /// P1 #2: Adaptive ATR Multiplier — корректирует ширину стопа по DNA
    /// Монеты с большим avg_loss → шире стоп (дышат рынком)
    /// Монеты с высоким win_rate → можно уже (точные входы)
    /// default_mult = fallback из головы (3.5 или 3.0)
    pub fn get_adaptive_atr_mult(symbol: &str, default_mult: f64) -> f64 {
        if let Some(json) = Self::read_snapshot_cached() {
                if let Some(entity) = json.get(symbol) {
                    let trade_count = entity["trade_count"].as_i64().unwrap_or(0);
                    let win_rate = entity["win_rate"].as_f64().unwrap_or(0.5);
                    let avg_loss = entity["avg_loss_size"].as_f64().unwrap_or(0.0).abs();
                    let avg_win = entity["avg_win_size"].as_f64().unwrap_or(0.0);

                    if trade_count >= 5 {
                        let mut mult = default_mult;

                        // Высокий WR (>60%) = можно ужать стоп (экономия на SL)
                        if win_rate >= 0.60 { mult -= 0.3; }
                        // Низкий WR (<35%) = надо расширить (даём больше дышать)
                        else if win_rate < 0.35 { mult += 0.5; }

                        // Если avg_loss значительно больше avg_win = слишком тесные стопы
                        if avg_loss > 0.0 && avg_win > 0.0 {
                            let rr_ratio = avg_win / avg_loss;
                            if rr_ratio < 0.5 { mult += 0.5; } // стопы выбивают, расширяем
                            if rr_ratio > 2.0 { mult -= 0.3; } // стопы работают, можно уже
                        }

                        return mult.clamp(2.0, 5.0);
                    }
            }
        }

        default_mult
    }

    /// BUG-11 FIX: Cached reader for hive_mind_snapshot.json (15s TTL)
    fn read_snapshot_cached() -> Option<Value> {
        let path = data_file("hive_mind_snapshot.json");
        if let Ok(mut cache) = SNAPSHOT_CACHE.lock() {
            if let Some((ts, ref val)) = *cache {
                if ts.elapsed().as_secs() < 15 { return Some(val.clone()); }
            }
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<Value>(&data) {
                    *cache = Some((Instant::now(), json.clone()));
                    return Some(json);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // get_dynamic_imbalance_threshold is pure — no I/O
    #[test]
    fn test_imbalance_strong_trend() {
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(3.5);
        assert_eq!(t, 1.10, "Strong trend should lower imbalance threshold to 1.10");
    }

    #[test]
    fn test_imbalance_moderate_trend() {
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(2.0);
        assert_eq!(t, 1.15);
    }

    #[test]
    fn test_imbalance_mild_market() {
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(1.5);
        assert_eq!(t, 1.30);
    }

    #[test]
    fn test_imbalance_ranging_market() {
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(0.5);
        assert_eq!(t, 1.50, "Ranging market should require strong imbalance");
    }

    #[test]
    fn test_imbalance_negative_btc_uses_abs() {
        // Negative BTC score should use abs value
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(-3.0);
        assert_eq!(t, 1.10, "Negative strong BTC should also trigger 1.10");
    }

    #[test]
    fn test_imbalance_zero_btc() {
        let t = ConfidenceEngine::get_dynamic_imbalance_threshold(0.0);
        assert_eq!(t, 1.50, "Zero BTC score = max caution");
    }
}
