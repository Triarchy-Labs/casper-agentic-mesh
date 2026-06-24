/// Decision Legitimacy Gate — "Has the agent EARNED the right to trade?"
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/owm/legitimacy.py (178 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Дополняет DQS как META-GATE: DQS оценивает КАЧЕСТВО трейда,
/// Legitimacy оценивает ПРАВО агента торговать вообще.
///
/// 5 факторов (взвешенное среднее):
///   1. Sample Sufficiency (0.30) — достаточно ли трейдов в памяти?
///   2. Memory Quality (0.15) — насколько стабилен context drift?
///   3. Regime Confidence (0.25) — есть ли опыт в текущем режиме?
///   4. Streak State (0.15) — серия проигрышей?
///   5. Drawdown State (0.15) — уровень просадки?
///
/// Три уровня: Full (1.0x) → Reduced (0.5x) → Skip (0.0x)
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Tier
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum LegitimacyTier {
    Full,     // score >= 0.7 → position_multiplier = 1.0
    Reduced,  // score >= 0.4 → position_multiplier = 0.5
    Skip,     // score < 0.4  → position_multiplier = 0.0
}

impl LegitimacyTier {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Full => "full",
            Self::Reduced => "reduced",
            Self::Skip => "skip",
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Result
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct LegitimacyResult {
    pub score: f64,                // 0.0 - 1.0
    pub tier: LegitimacyTier,
    pub position_multiplier: f64,  // 1.0 / 0.5 / 0.0
    pub factors: LegitimacyFactors,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegitimacyFactors {
    pub sample_sufficiency: f64,
    pub memory_quality: f64,
    pub regime_confidence: f64,
    pub streak_state: f64,
    pub drawdown_state: f64,
}

// ═══════════════════════════════════════════════════════════════
// Factor scoring (порт: legitimacy.py:8-55)
// ═══════════════════════════════════════════════════════════════

/// Score based on total trade count for this strategy.
/// Порт: legitimacy.py:8-16
fn sample_sufficiency(n: usize) -> f64 {
    if n >= 30 { 1.0 }
    else if n >= 15 { 0.7 }
    else if n >= 5 { 0.4 }
    else { 0.1 }
}

/// Score based on average context drift (lower drift = better).
/// Порт: legitimacy.py:19-22
fn memory_quality(avg_context_drift: f64) -> f64 {
    (1.0 - avg_context_drift).clamp(0.0, 1.0)
}

/// Score based on trades in the current regime.
/// Порт: legitimacy.py:25-33
fn regime_confidence(regime_trade_count: usize) -> f64 {
    if regime_trade_count >= 10 { 1.0 }
    else if regime_trade_count >= 5 { 0.7 }
    else if regime_trade_count >= 2 { 0.4 }
    else { 0.1 }
}

/// Score based on consecutive loss count.
/// Порт: legitimacy.py:36-44
fn streak_state(consecutive_losses: usize) -> f64 {
    if consecutive_losses == 0 { 1.0 }
    else if consecutive_losses <= 2 { 0.8 }
    else if consecutive_losses <= 4 { 0.5 }
    else { 0.2 }
}

/// Score based on account drawdown percentage (0-100 scale).
/// Порт: legitimacy.py:47-55
fn drawdown_state(drawdown_pct: f64) -> f64 {
    if drawdown_pct < 5.0 { 1.0 }
    else if drawdown_pct < 10.0 { 0.7 }
    else if drawdown_pct < 20.0 { 0.4 }
    else { 0.1 }
}

// ═══════════════════════════════════════════════════════════════
// Weights (порт: legitimacy.py:60-66)
// ═══════════════════════════════════════════════════════════════

const W_SAMPLE: f64 = 0.30;
const W_MEMORY: f64 = 0.15;
const W_REGIME: f64 = 0.25;
const W_STREAK: f64 = 0.15;
const W_DRAWDOWN: f64 = 0.15;

// ═══════════════════════════════════════════════════════════════
// Main computation (порт: legitimacy.py:69-134)
// ═══════════════════════════════════════════════════════════════

/// Compute whether an agent has "earned the right" to trade at full confidence.
///
/// Порт: legitimacy.py:69-134 (compute_legitimacy_score)
pub fn compute_legitimacy(
    memory_count: usize,
    avg_context_drift: f64,
    regime_trade_count: usize,
    consecutive_losses: usize,
    drawdown_pct: f64,
) -> LegitimacyResult {
    let factors = LegitimacyFactors {
        sample_sufficiency: sample_sufficiency(memory_count),
        memory_quality: memory_quality(avg_context_drift),
        regime_confidence: regime_confidence(regime_trade_count),
        streak_state: streak_state(consecutive_losses),
        drawdown_state: drawdown_state(drawdown_pct),
    };

    // Weighted average (порт: legitimacy.py:103)
    let score = factors.sample_sufficiency * W_SAMPLE
        + factors.memory_quality * W_MEMORY
        + factors.regime_confidence * W_REGIME
        + factors.streak_state * W_STREAK
        + factors.drawdown_state * W_DRAWDOWN;

    // Tier + multiplier (порт: legitimacy.py:106-114)
    let (tier, multiplier) = if score >= 0.7 {
        (LegitimacyTier::Full, 1.0)
    } else if score >= 0.4 {
        (LegitimacyTier::Reduced, 0.5)
    } else {
        (LegitimacyTier::Skip, 0.0)
    };

    LegitimacyResult {
        score: (score * 10000.0).round() / 10000.0,  // 4 decimal places
        tier,
        position_multiplier: multiplier,
        factors,
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_legitimacy() {
        let r = compute_legitimacy(50, 0.1, 15, 0, 2.0);
        assert_eq!(r.tier, LegitimacyTier::Full);
        assert_eq!(r.position_multiplier, 1.0);
        assert!(r.score >= 0.7);
    }

    #[test]
    fn test_reduced_legitimacy() {
        // Few trades, new regime, some losses, moderate DD
        let r = compute_legitimacy(8, 0.3, 3, 3, 8.0);
        assert_eq!(r.tier, LegitimacyTier::Reduced);
        assert_eq!(r.position_multiplier, 0.5);
    }

    #[test]
    fn test_skip_legitimacy() {
        // Zero experience, high DD, loss streak
        let r = compute_legitimacy(2, 0.8, 0, 6, 25.0);
        assert_eq!(r.tier, LegitimacyTier::Skip);
        assert_eq!(r.position_multiplier, 0.0);
    }

    #[test]
    fn test_sample_sufficiency_thresholds() {
        assert_eq!(sample_sufficiency(0), 0.1);
        assert_eq!(sample_sufficiency(4), 0.1);
        assert_eq!(sample_sufficiency(5), 0.4);
        assert_eq!(sample_sufficiency(15), 0.7);
        assert_eq!(sample_sufficiency(30), 1.0);
        assert_eq!(sample_sufficiency(100), 1.0);
    }

    #[test]
    fn test_memory_quality_clamp() {
        assert_eq!(memory_quality(0.0), 1.0);
        assert_eq!(memory_quality(1.0), 0.0);
        assert_eq!(memory_quality(1.5), 0.0);  // clamped
        assert_eq!(memory_quality(-0.5), 1.0); // clamped
    }

    #[test]
    fn test_streak_state_thresholds() {
        assert_eq!(streak_state(0), 1.0);
        assert_eq!(streak_state(2), 0.8);
        assert_eq!(streak_state(4), 0.5);
        assert_eq!(streak_state(5), 0.2);
    }

    #[test]
    fn test_drawdown_thresholds() {
        assert_eq!(drawdown_state(0.0), 1.0);
        assert_eq!(drawdown_state(4.9), 1.0);
        assert_eq!(drawdown_state(5.0), 0.7);
        assert_eq!(drawdown_state(10.0), 0.4);
        assert_eq!(drawdown_state(20.0), 0.1);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let sum = W_SAMPLE + W_MEMORY + W_REGIME + W_STREAK + W_DRAWDOWN;
        assert!((sum - 1.0).abs() < 0.001, "Weights must sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_perfect_score() {
        let r = compute_legitimacy(100, 0.0, 50, 0, 0.0);
        assert!((r.score - 1.0).abs() < 0.001, "Perfect inputs → score ~1.0, got {}", r.score);
        assert_eq!(r.tier, LegitimacyTier::Full);
    }

    #[test]
    fn test_worst_score() {
        let r = compute_legitimacy(0, 1.0, 0, 10, 50.0);
        assert!(r.score < 0.2, "Worst inputs → low score, got {}", r.score);
        assert_eq!(r.tier, LegitimacyTier::Skip);
    }
}
