/// F8: Reinforcement Learning Reward Signal.
///
/// После каждого трейда генерирует reward signal:
/// - regime_reward:    +1 если вошли в правильном режиме, -1 если нет
/// - timing_reward:    +1 если golden hour, -1 если worst hour
/// - direction_reward: +1 если best_side совпал, -1 если нет
/// - novelty_reward:   +0.5 за новый контекст, -0.5 за повторение ошибки
/// - total_reward:     взвешенная сумма → UDP обратно в Titan
///
/// Замыкает цикл: Titan → Castle → Titan (self-improving feedback loop).
use serde::Serialize;
use crate::entity::MemoryEntity;
use crate::brain::BrainDiagnostics;
use crate::patterns::TradingSession;

/// Reward signal — отправляется обратно в Titan.
#[derive(Debug, Clone, Serialize)]
pub struct RewardSignal {
    pub symbol: String,
    /// +1.0 если вошли в профитабельном режиме, -1.0 если в убыточном.
    pub regime_reward: f64,
    /// +1.0 если golden hour (London/Overlap), -0.5 если off-hours.
    pub timing_reward: f64,
    /// +1.0 если best_side совпал с направлением, -0.5 если нет.
    pub direction_reward: f64,
    /// +0.5 если novel context (первый раз), -0.5 если повтор ошибки.
    pub novelty_reward: f64,
    /// Взвешенная сумма всех компонент [-3.0 .. +3.0].
    pub total_reward: f64,
    /// OWM recall score для этого контекста.
    pub owm_score: f64,
    /// Рекомендуемый множитель позиции (0.5 .. 2.0).
    pub recommended_size_mult: f64,
}

/// Веса компонент reward signal.
const W_REGIME: f64 = 0.35;
const W_TIMING: f64 = 0.20;
const W_DIRECTION: f64 = 0.25;
const W_NOVELTY: f64 = 0.20;

/// Генерирует reward signal на основе результата трейда.
///
/// `pnl` — PnL текущего трейда.
/// `side` — сторона текущего трейда ("Buy" / "Sell").
/// `timestamp_ms` — время входа.
/// `entity` — DNA символа ПОСЛЕ обработки трейда.
/// `diag` — результат brain.process_trade().
pub fn generate_reward(
    pnl: f64,
    side: &str,
    timestamp_ms: i64,
    entity: &MemoryEntity,
    diag: &BrainDiagnostics,
) -> RewardSignal {
    let was_profitable = pnl > 0.0;

    // ═══ 1. REGIME REWARD ═══
    // Вошли в тренд и профит → +1. Вошли в volatile и слив → -1.
    let regime_reward = match diag.regime.as_str() {
        "trending_up" | "trending_down" => {
            if was_profitable { 1.0 } else { -0.3 } // Тренд прощает ошибки
        }
        "volatile" => {
            if was_profitable { 0.5 } else { -1.0 } // Volatile наказывает сильнее
        }
        "ranging" => {
            if was_profitable { 0.3 } else { -0.5 } // Рендж — нейтральный
        }
        _ => 0.0,
    };

    // ═══ 2. TIMING REWARD ═══
    let session = TradingSession::from_timestamp_ms(timestamp_ms);
    let timing_reward = match session {
        TradingSession::LondonNYOverlap => {
            if was_profitable { 1.0 } else { -0.2 } // Overlap = максимальная ликвидность
        }
        TradingSession::London => {
            if was_profitable { 0.8 } else { -0.3 }
        }
        TradingSession::NewYork => {
            if was_profitable { 0.6 } else { -0.3 }
        }
        TradingSession::Tokyo => {
            if was_profitable { 0.3 } else { -0.5 } // Токио = слабая ликвидность
        }
        TradingSession::OffHours => {
            if was_profitable { 0.1 } else { -1.0 } // Off-hours = наказание
        }
    };

    // ═══ 3. DIRECTION REWARD ═══
    // Совпала ли сторона с лучшей стороной по DNA?
    let side_lower = side.to_lowercase();
    let best_lower = entity.best_side.to_lowercase();
    let direction_reward = if side.is_empty() || best_lower == "neutral" {
        0.0 // Нет данных — нейтраль
    } else if side_lower == best_lower {
        if was_profitable { 1.0 } else { -0.2 } // Правильная сторона
    } else {
        if was_profitable { 0.3 } else { -0.8 } // Неправильная сторона = наказание
    };

    // ═══ 4. NOVELTY REWARD ═══
    let novelty_reward = if diag.is_novel_context {
        if was_profitable { 0.5 } else { 0.0 } // Новый контекст, ещё нет данных
    } else {
        if was_profitable { 0.3 } else { -0.5 } // Повторение — уже должны были знать
    };

    // ═══ TOTAL ═══
    let total_reward = regime_reward * W_REGIME
        + timing_reward * W_TIMING
        + direction_reward * W_DIRECTION
        + novelty_reward * W_NOVELTY;

    // ═══ SIZING RECOMMENDATION ═══
    // total_reward [-3.0 .. +3.0] → size_mult [0.5 .. 2.0]
    let recommended_size_mult = sigmoid_size(total_reward);

    RewardSignal {
        symbol: entity.entity_id.clone(),
        regime_reward,
        timing_reward,
        direction_reward,
        novelty_reward,
        total_reward,
        owm_score: diag.owm_score,
        recommended_size_mult,
    }
}

/// Sigmoid для размера позиции: maps [-3, +3] → [0.5, 2.0]
fn sigmoid_size(x: f64) -> f64 {
    let sig = 1.0 / (1.0 + (-x * 1.5).exp()); // steepness = 1.5
    0.5 + sig * 1.5 // [0.5 .. 2.0]
}

/// Сериализует reward signal в JSON для UDP отправки.
pub fn reward_to_json(reward: &RewardSignal) -> String {
    serde_json::to_string(reward).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(symbol: &str, best_side: &str) -> MemoryEntity {
        let mut e = MemoryEntity::new(symbol);
        e.best_side = best_side.to_string();
        e.trade_count = 10;
        e
    }

    fn make_diag(symbol: &str, regime: &str, novel: bool, owm: f64) -> BrainDiagnostics {
        BrainDiagnostics {
            symbol: symbol.to_string(),
            regime: regime.to_string(),
            regime_confidence: 0.8,
            ewma_confidence: 0.7,
            risk_appetite: 0.9,
            drift_detected: false,
            drift_cusum: 0.0,
            disposition_detected: false,
            disposition_severity: "normal".to_string(),
            is_novel_context: novel,
            belief_confidence: Some(0.6),
            causal_predictions: vec![],
            owm_score: owm,
            dqs_score: 7.0,
            dqs_tier: "go".to_string(),
            dqs_position_multiplier: 1.0,
            changepoint_probability: 0.0,
            kelly_fraction: 0.1,
            obi_ratio: 0.0,
            obi_bias: "neutral".to_string(),
            obi_confidence: 0.0,
        }
    }

    #[test]
    fn test_reward_profitable_trending() {
        let entity = make_entity("ETH", "buy");
        let diag = make_diag("ETH", "trending_up", false, 0.7);
        let reward = generate_reward(5.0, "Buy", 3600_000 * 14, &entity, &diag); // 14:00 = Overlap
        assert!(reward.total_reward > 0.5, "Profitable + trending + overlap + right side = strong reward, got {}", reward.total_reward);
        assert!(reward.recommended_size_mult > 1.2, "Should recommend larger position");
    }

    #[test]
    fn test_reward_loss_volatile_offhours() {
        let entity = make_entity("DOGE", "sell");
        let diag = make_diag("DOGE", "volatile", false, 0.3);
        let reward = generate_reward(-10.0, "Buy", 3600_000 * 23, &entity, &diag); // 23:00 = OffHours
        assert!(reward.total_reward < -0.5, "Loss + volatile + off-hours + wrong side = strong punishment, got {}", reward.total_reward);
        assert!(reward.recommended_size_mult < 1.0, "Should recommend smaller position");
    }

    #[test]
    fn test_reward_novel_context_neutral() {
        let entity = make_entity("SOL", "neutral");
        let diag = make_diag("SOL", "ranging", true, 0.5);
        let reward = generate_reward(1.0, "", 3600_000 * 10, &entity, &diag); // London
        assert!(reward.novelty_reward >= 0.0, "Novel + profit should be non-negative");
        assert!(reward.direction_reward == 0.0, "No side data = neutral direction");
    }

    #[test]
    fn test_sigmoid_size_range() {
        // Extreme negative
        let small = sigmoid_size(-3.0);
        assert!(small >= 0.5 && small < 0.7, "Very negative should be near 0.5, got {}", small);
        // Extreme positive
        let large = sigmoid_size(3.0);
        assert!(large > 1.8 && large <= 2.0, "Very positive should be near 2.0, got {}", large);
        // Neutral
        let mid = sigmoid_size(0.0);
        assert!((mid - 1.25).abs() < 0.1, "Neutral should be ~1.25, got {}", mid);
    }

    #[test]
    fn test_reward_json_serializable() {
        let entity = make_entity("BTC", "buy");
        let diag = make_diag("BTC", "trending_up", false, 0.8);
        let reward = generate_reward(50.0, "Buy", 3600_000 * 14, &entity, &diag);
        let json = reward_to_json(&reward);
        assert!(json.contains("BTC"), "JSON should contain symbol");
        assert!(json.contains("regime_reward"), "JSON should contain all fields");
        assert!(json.contains("recommended_size_mult"), "JSON should contain sizing");
    }

    #[test]
    fn test_reward_weights_sum_to_one() {
        let sum = W_REGIME + W_TIMING + W_DIRECTION + W_NOVELTY;
        assert!((sum - 1.0).abs() < 1e-10, "Weights must sum to 1.0, got {}", sum);
    }
}
