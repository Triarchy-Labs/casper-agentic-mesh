// src/modules/unstuck.rs
// ═══════════════════════════════════════════════════════════════
// UNSTUCKING ENGINE (Passivbot v7 Inspired, Titan-Adapted)
// ═══════════════════════════════════════════════════════════════
// Институциональный модуль управления «застрявшими» позициями.
// 
// Архитектура основана на 3 принципах Passivbot v7:
//   1. PRIORITIZATION — закрываем ту, что ближе всего к рынку (наименьший gap)
//   2. GRADUATED RELEASE — не 50% одним ударом, а по ATR-тикам
//   3. PEAK BALANCE GUARD — не позволяем unstucking'у дренить счёт ниже пика
//
// Отличие от Passivbot: мы НЕ grid-трейдеры. У нас momentum+scoring.
// Поэтому unstucking = последний рубеж обороны, когда scoring ошибся.

use chrono::Utc;

/// Конфигурация Unstucking Engine
pub struct UnstuckConfig {
    /// Минимальное время удержания перед unstucking (часы)
    pub min_hold_hours: f64,
    /// Порог unrealized loss (%) для TRIGGER
    pub loss_threshold_pct: f64,
    /// Доля позиции для первого unstuck (0.0-1.0)
    pub first_release_pct: f64,
    /// Доля для второго unstuck (если first не помог, ещё N часов)
    pub second_release_pct: f64,
    /// Задержка между первым и вторым unstuck (часы)
    pub second_stage_delay_hours: f64,
    /// Максимально допустимый realized loss от peak balance (%)
    pub max_realized_loss_from_peak_pct: f64,
}

impl Default for UnstuckConfig {
    fn default() -> Self {
        Self {
            min_hold_hours: 2.0,
            loss_threshold_pct: -3.0,     // Trigger при >3% unrealized loss
            first_release_pct: 0.30,      // Первый этап: 30% позиции
            second_release_pct: 0.50,     // Второй этап: 50% от остатка
            second_stage_delay_hours: 1.5, // Ждём 1.5ч между этапами
            max_realized_loss_from_peak_pct: 5.0, // Не дренить больше 5% от пика
        }
    }
}

/// Результат анализа Unstucking Engine
#[derive(Debug, Clone)]
pub enum UnstuckVerdict {
    /// Позиция здорова, не трогать
    Healthy,
    /// Позиция болеет, но ещё рано
    Monitoring { hold_hours: f64, loss_pct: f64 },
    /// ПЕРВЫЙ ЭТАП: закрыть first_release_pct
    ReleaseStage1 { close_pct: f64, reason: String },
    /// ВТОРОЙ ЭТАП: закрыть second_release_pct от остатка
    ReleaseStage2 { close_pct: f64, reason: String },
    /// ПОЛНАЯ ЭВАКУАЦИЯ: momentum развернулся против нас + loss > 5%
    FullEvacuation { reason: String },
    /// Заблокировано: realized loss уже слишком большой для сессии
    BlockedByPeakGuard,
    /// RE-ENTRY: после partial close, пересесть на лучшую цену
    /// Passivbot v7: partial close at loss → re-enter at ATR-offset (better avg entry)
    #[allow(dead_code)]
    ReentryAfterTrim { side: String, target_price: f64, size_pct: f64, reason: String },
}

pub struct UnstuckEngine;

impl UnstuckEngine {
    /// Главная функция — анализирует позицию и выдаёт вердикт
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        side: &str,
        entry_price: f64,
        current_price: f64,
        _amount: f64,
        entry_time_ms: i64,
        atr: f64,
        btc_score: f64,
        daily_loss: f64,
        peak_balance: f64,
        stage1_done: bool,
        stage1_time_ms: i64,
        config: &UnstuckConfig,
    ) -> UnstuckVerdict {
        let now_ms = Utc::now().timestamp_millis();
        let hold_hours = if entry_time_ms > 0 {
            (now_ms - entry_time_ms) as f64 / 3_600_000.0
        } else {
            return UnstuckVerdict::Healthy;
        };

        // Unrealized PnL %
        let unrealized_pct = if side == "Buy" {
            (current_price - entry_price) / entry_price * 100.0
        } else {
            (entry_price - current_price) / entry_price * 100.0
        };

        // === HEALTHY: Позиция в плюсе или слабый минус ===
        if unrealized_pct > config.loss_threshold_pct {
            if hold_hours > 1.0 && unrealized_pct < -1.0 {
                return UnstuckVerdict::Monitoring { hold_hours, loss_pct: unrealized_pct };
            }
            return UnstuckVerdict::Healthy;
        }

        // === ТОО YOUNG: Недостаточно времени ===
        if hold_hours < config.min_hold_hours {
            return UnstuckVerdict::Monitoring { hold_hours, loss_pct: unrealized_pct };
        }

        // === PEAK BALANCE GUARD ===
        // Если daily_loss уже > X% от peak_balance → не unstuck'ить дальше
        if peak_balance > 0.0 {
            let loss_from_peak = (daily_loss / peak_balance) * 100.0;
            if loss_from_peak > config.max_realized_loss_from_peak_pct {
                return UnstuckVerdict::BlockedByPeakGuard;
            }
        }

        // === FULL EVACUATION: momentum катастрофически против + >5% loss ===
        let momentum_against = match side {
            "Buy" => btc_score < -2.0,   // BTC рушится, мы в лонге
            "Sell" => btc_score > 2.0,   // BTC ракетит, мы в шорте
            _ => false,
        };
        if momentum_against && unrealized_pct < -5.0 && hold_hours > 3.0 {
            return UnstuckVerdict::FullEvacuation {
                reason: format!(
                    "Momentum catastrophe: {unrealized_pct}% loss, BTC score {btc_score:.1}, held {hold_hours:.1}h"
                ),
            };
        }

        // === STAGE 2: первый unstuck уже был, позиция всё ещё болеет ===
        if stage1_done {
            let since_stage1_hours = (now_ms - stage1_time_ms) as f64 / 3_600_000.0;
            if since_stage1_hours >= config.second_stage_delay_hours {
                return UnstuckVerdict::ReleaseStage2 {
                    close_pct: config.second_release_pct,
                    reason: format!(
                        "Stage2: {unrealized_pct:.1}% loss after {since_stage1_hours:.1}h since stage1, ATR={atr:.4}"
                    ),
                };
            }
            return UnstuckVerdict::Monitoring { hold_hours, loss_pct: unrealized_pct };
        }

        // === STAGE 1: первый graduated release ===
        UnstuckVerdict::ReleaseStage1 {
            close_pct: config.first_release_pct,
            reason: format!(
                "Stage1: {unrealized_pct:.1}% loss over {hold_hours:.1}h, ATR={atr:.4}"
            ),
        }
    }

    /// Приоритизация: из нескольких stuck позиций выбрать ту, что БЛИЖЕ к рынку
    /// (Passivbot v7: "smallest gap between entry and market = easiest to unstuck")
    #[allow(dead_code)] // Reserved for multi-position unstuck prioritization
    pub fn prioritize_unstuck(
        positions: &[(String, f64, f64, f64)], // (symbol, entry_price, current_price, loss_pct)
    ) -> Option<String> {
        if positions.is_empty() { return None; }
        
        // Находим позицию с наименьшим абсолютным расстоянием от entry до market
        positions.iter()
            .min_by(|a, b| {
                let gap_a = ((a.1 - a.2) / a.1).abs();
                let gap_b = ((b.1 - b.2) / b.1).abs();
                gap_a.partial_cmp(&gap_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.0.clone())
    }

    /// PASSIVBOT RE-ENTRY: после partial close, рассчитать цену re-entry
    /// Логика: после trim, пересаживаемся на 0.5*ATR ниже (для лонга) / выше (для шорта)
    /// Это сдвигает average entry ближе к рынку = быстрее выход в безубыток
    #[allow(dead_code)]
    pub fn reentry_target(
        side: &str,
        current_price: f64,
        atr: f64,
    ) -> f64 {
        let offset = atr * 0.5; // Re-enter на полсвечи лучше текущей цены
        if side == "Buy" {
            current_price - offset // Лонг: re-entry ниже рынка (лучше)
        } else {
            current_price + offset // Шорт: re-entry выше рынка (лучше)
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn now_ms() -> i64 { Utc::now().timestamp_millis() }
    fn hours_ago(h: f64) -> i64 { now_ms() - (h * 3_600_000.0) as i64 }

    #[test]
    fn test_healthy_in_profit() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 105.0, 1.0, hours_ago(5.0),
            2.0, 0.0, 0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::Healthy));
    }

    #[test]
    fn test_monitoring_small_loss() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 98.5, 1.0, hours_ago(1.5),
            2.0, 0.0, 0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::Monitoring { .. }));
    }

    #[test]
    fn test_too_young() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 95.0, 1.0, hours_ago(0.5), // only 30min
            2.0, 0.0, 0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::Monitoring { .. }));
    }

    #[test]
    fn test_stage1_triggered() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 95.0, 1.0, hours_ago(3.0), // 5% loss, 3h held
            2.0, 0.0, 0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        match v {
            UnstuckVerdict::ReleaseStage1 { close_pct, .. } => {
                assert!((close_pct - 0.30).abs() < 0.01);
            }
            _ => panic!("Expected Stage1, got {:?}", v),
        }
    }

    #[test]
    fn test_stage2_after_stage1() {
        let stage1_time = hours_ago(2.0); // Stage1 was 2h ago (> 1.5h delay)
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 95.0, 1.0, hours_ago(5.0),
            2.0, 0.0, 0.0, 100.0, true, stage1_time, &UnstuckConfig::default(),
        );
        match v {
            UnstuckVerdict::ReleaseStage2 { close_pct, .. } => {
                assert!((close_pct - 0.50).abs() < 0.01);
            }
            _ => panic!("Expected Stage2, got {:?}", v),
        }
    }

    #[test]
    fn test_full_evacuation() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 93.0, 1.0, hours_ago(4.0), // 7% loss
            2.0, -3.0, // BTC crashing, we're long
            0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::FullEvacuation { .. }));
    }

    #[test]
    fn test_peak_guard_blocks() {
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 95.0, 1.0, hours_ago(3.0),
            2.0, 0.0,
            6.0, 100.0, // daily_loss=6, peak=100 → 6% > 5% guard
            false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::BlockedByPeakGuard));
    }

    #[test]
    fn test_short_healthy() {
        let v = UnstuckEngine::evaluate(
            "Sell", 100.0, 95.0, 1.0, hours_ago(2.0), // Short in profit
            2.0, 0.0, 0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::Healthy));
    }

    #[test]
    fn test_short_evacuation() {
        let v = UnstuckEngine::evaluate(
            "Sell", 100.0, 108.0, 1.0, hours_ago(4.0), // Short -8% loss
            2.0, 3.0, // BTC pumping, we're short
            0.0, 100.0, false, 0, &UnstuckConfig::default(),
        );
        assert!(matches!(v, UnstuckVerdict::FullEvacuation { .. }));
    }

    #[test]
    fn test_prioritize_smallest_gap() {
        let positions = vec![
            ("ETHUSDT".to_string(), 100.0, 90.0, -10.0),  // 10% gap
            ("SOLUSDT".to_string(), 50.0, 48.0, -4.0),    // 4% gap — CLOSEST
            ("BTCUSDT".to_string(), 70000.0, 63000.0, -10.0), // 10% gap
        ];
        let winner = UnstuckEngine::prioritize_unstuck(&positions);
        assert_eq!(winner, Some("SOLUSDT".to_string()));
    }

    #[test]
    fn test_reentry_target_long() {
        let target = UnstuckEngine::reentry_target("Buy", 100.0, 4.0);
        assert!((target - 98.0).abs() < 0.01); // 100 - 0.5*4 = 98
    }

    #[test]
    fn test_reentry_target_short() {
        let target = UnstuckEngine::reentry_target("Sell", 100.0, 4.0);
        assert!((target - 102.0).abs() < 0.01); // 100 + 0.5*4 = 102
    }

    #[test]
    fn test_custom_config() {
        let config = UnstuckConfig {
            min_hold_hours: 0.5,
            loss_threshold_pct: -1.0, // trigger at 1% loss
            first_release_pct: 0.20,
            ..Default::default()
        };
        let v = UnstuckEngine::evaluate(
            "Buy", 100.0, 98.5, 1.0, hours_ago(1.0), // 1.5% loss, 1h held
            2.0, 0.0, 0.0, 100.0, false, 0, &config,
        );
        match v {
            UnstuckVerdict::ReleaseStage1 { close_pct, .. } => {
                assert!((close_pct - 0.20).abs() < 0.01);
            }
            _ => panic!("Expected Stage1 with custom config"),
        }
    }
}
