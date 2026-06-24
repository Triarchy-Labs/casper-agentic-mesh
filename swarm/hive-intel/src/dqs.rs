/// Decision Quality Score (DQS) — pre-trade evaluation gate.
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/owm/dqs.py (579 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// В отличие от reward.rs (POST-trade), DQS работает ДО открытия позиции:
/// "Стоит ли вообще входить в трейд?"
///
/// 5 факторов, каждый 0-2, итого 0-10:
///   1. Regime Match — WR стратегии в текущем режиме vs общий WR
///   2. Position Sizing — отклонение от Kelly fraction (exp decay)
///   3. Process Adherence — OWM score похожих прошлых трейдов
///   4. Risk State — drawdown + consecutive losses + confidence
///   5. Historical Pattern — avg PnL похожих прошлых трейдов (tanh)
///
/// Adaptive thresholds: skip = μ - 2σ, caution = μ - 1σ (из распределения DQS)
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// DQS Result
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct DqsResult {
    pub score: f64,               // 0-10
    pub tier: DqsTier,
    pub position_multiplier: f64, // 1.0 / 0.5 / 0.0
    pub factors: DqsFactors,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DqsTier {
    Go,      // score >= caution_th → full size
    Caution, // skip_th <= score < caution_th → half size
    Skip,    // score < skip_th → don't trade
}

impl DqsTier {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Go => "go",
            Self::Caution => "caution",
            Self::Skip => "skip",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DqsFactors {
    pub regime_match: f64,       // 0-2
    pub position_sizing: f64,    // 0-2
    pub process_adherence: f64,  // 0-2
    pub risk_state: f64,         // 0-2
    pub historical_pattern: f64, // 0-2
}

// ═══════════════════════════════════════════════════════════════
// DQS Engine
// ═══════════════════════════════════════════════════════════════

pub struct DqsEngine {
    weights: [f64; 5],
    skip_threshold: f64,
    caution_threshold: f64,
}

impl Default for DqsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DqsEngine {
    pub fn new() -> Self {
        Self {
            weights: [1.0, 1.0, 1.0, 1.0, 1.0], // equal weights
            skip_threshold: 3.0,
            caution_threshold: 5.0,
        }
    }

    /// Adaptive thresholds из распределения DQS scores.
    /// Порт: dqs.py:51-72 (set_adaptive_thresholds)
    /// skip = mean - 2*std, caution = mean - 1*std
    pub fn calibrate_thresholds(&mut self, dqs_history: &[f64]) {
        if dqs_history.len() < 10 { return; }

        let n = dqs_history.len() as f64;
        let mean = dqs_history.iter().sum::<f64>() / n;
        let variance = dqs_history.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>() / n;
        let std = variance.sqrt();

        self.skip_threshold = (mean - 2.0 * std).max(0.0);
        self.caution_threshold = (mean - 1.0 * std).max(self.skip_threshold + 0.1);
    }

    /// Порт: dqs.py:436-528 (calibrate — logistic regression)
    /// Обучает веса на исторических трейдах через gradient descent.
    pub fn calibrate_weights(&mut self, trade_features: &[[f64; 5]], trade_outcomes: &[f64]) {
        if trade_features.len() < 50 || trade_features.len() != trade_outcomes.len() { return; }

        let n = trade_features.len();
        let mut weights = [0.0_f64; 5];
        let lr = 0.01;
        let epochs = 200;

        // Logistic regression via gradient descent (порт: dqs.py:538-562)
        for _ in 0..epochs {
            let mut gradients = [0.0_f64; 5];
            for i in 0..n {
                let z: f64 = (0..5).map(|j| trade_features[i][j] * weights[j]).sum();
                let pred = sigmoid(z);
                let error = pred - trade_outcomes[i];
                for j in 0..5 {
                    gradients[j] += error * trade_features[i][j];
                }
            }
            for j in 0..5 {
                weights[j] -= lr * (gradients[j] / n as f64);
            }
        }

        // Normalize: shift to positive, scale to sum=5 (порт: dqs.py:508-512)
        let min_w = weights.iter().cloned().fold(f64::INFINITY, f64::min);
        let shifted: Vec<f64> = weights.iter().map(|w| w - min_w + 0.1).collect();
        let total: f64 = shifted.iter().sum();
        for (w, s) in self.weights.iter_mut().zip(shifted.iter()) {
            *w = s / total * 5.0;
        }
    }

    /// Главная функция: compute DQS
    /// Порт: dqs.py:328-401 (compute)
    pub fn compute(&self, input: &DqsInput) -> DqsResult {
        let f1 = factor_regime_match(input.regime_win_rate, input.overall_win_rate);
        let f2 = factor_position_sizing(input.proposed_lot, input.kelly_fraction);
        let f3 = factor_process_adherence(input.owm_score);
        let f4 = factor_risk_state(input.drawdown_pct, input.consecutive_losses, input.confidence);
        let f5 = factor_historical_pattern(input.avg_pnl_r_similar);

        let raw = [f1, f2, f3, f4, f5];

        // Weighted sum → 0-10 scale (порт: dqs.py:357-363)
        let weighted_sum: f64 = raw.iter().zip(self.weights.iter())
            .map(|(s, w)| s * w)
            .sum();
        let weight_total: f64 = self.weights.iter().sum();
        let score = if weight_total > 0.0 {
            (weighted_sum / weight_total) * 5.0 // ×5 because max per factor is 2
        } else {
            5.0
        }.clamp(0.0, 10.0);

        // Tier (порт: dqs.py:370-381)
        let (tier, multiplier) = if score >= self.caution_threshold {
            (DqsTier::Go, 1.0)
        } else if score >= self.skip_threshold {
            (DqsTier::Caution, 0.5)
        } else {
            (DqsTier::Skip, 0.0)
        };

        DqsResult {
            score,
            tier,
            position_multiplier: multiplier,
            factors: DqsFactors {
                regime_match: f1,
                position_sizing: f2,
                process_adherence: f3,
                risk_state: f4,
                historical_pattern: f5,
            },
        }
    }
}

/// Input for DQS computation.
pub struct DqsInput {
    pub regime_win_rate: Option<f64>,  // WR в текущем режиме
    pub overall_win_rate: f64,         // общий WR
    pub proposed_lot: f64,             // предложенный размер
    pub kelly_fraction: Option<f64>,   // Kelly optimal fraction
    pub owm_score: f64,                // OWM recall score от Castle
    pub drawdown_pct: f64,             // текущий drawdown %
    pub consecutive_losses: u32,       // серия лоссов
    pub confidence: f64,               // Bayesian confidence
    pub avg_pnl_r_similar: f64,        // avg PnL похожих трейдов
}

// ═══════════════════════════════════════════════════════════════
// Factor Functions (порт формул из dqs.py)
// ═══════════════════════════════════════════════════════════════

/// F1: Regime Match — dqs.py:78-119
/// score = 2.0 × (regime_WR / overall_WR), clamped [0, 2]
fn factor_regime_match(regime_wr: Option<f64>, overall_wr: f64) -> f64 {
    match regime_wr {
        None => 1.0, // neutral if no regime data
        Some(rwr) => {
            if overall_wr <= 0.0 { return 1.0; }
            let ratio = rwr / overall_wr;
            (2.0 * ratio).clamp(0.0, 2.0)
        }
    }
}

/// F2: Position Sizing — dqs.py:125-146
/// score = 2.0 × exp(-1.5 × |lot - kelly| / kelly)
fn factor_position_sizing(proposed_lot: f64, kelly: Option<f64>) -> f64 {
    match kelly {
        None => 1.0,
        Some(k) if k <= 0.0 => 1.0,
        Some(k) => {
            let deviation = (proposed_lot - k).abs() / k;
            (2.0 * (-1.5 * deviation).exp()).clamp(0.0, 2.0)
        }
    }
}

/// F3: Process Adherence — dqs.py:152-214
/// score = 0.5 + (owm_score / 0.7) × 1.5
fn factor_process_adherence(owm_score: f64) -> f64 {
    (0.5 + (owm_score / 0.7) * 1.5).clamp(0.0, 2.0)
}

/// F4: Risk State — dqs.py:220-258
/// Hard stops: dd > 20% or losses > 4 → 0.0
/// Penalties: dd ≥ 5% (-0.5), dd ≥ 10% (-0.5), losses ≥ 2 (-0.3), losses ≥ 3 (-0.3), conf < 0.6 (-0.4)
fn factor_risk_state(drawdown_pct: f64, consecutive_losses: u32, confidence: f64) -> f64 {
    // Hard stops
    if drawdown_pct > 20.0 || consecutive_losses > 4 {
        return 0.0;
    }

    let mut score: f64 = 2.0;
    if drawdown_pct >= 5.0  { score -= 0.5; }
    if drawdown_pct >= 10.0 { score -= 0.5; }
    if consecutive_losses >= 2 { score -= 0.3; }
    if consecutive_losses >= 3 { score -= 0.3; }
    if confidence < 0.6 { score -= 0.4; }

    score.max(0.0)
}

/// F5: Historical Pattern — dqs.py:264-322
/// score = 1.0 + tanh(avg_pnl_r)
fn factor_historical_pattern(avg_pnl_r: f64) -> f64 {
    (1.0 + avg_pnl_r.tanh()).clamp(0.0, 2.0)
}

// ═══════════════════════════════════════════════════════════════
// Kelly Criterion (порт: kelly.py)
// ═══════════════════════════════════════════════════════════════

/// Kelly criterion from OWM-weighted memories.
/// Порт: kelly.py:17-83 (kelly_from_memory)
///
/// f* = p/a - q/b
///   p = weighted win probability
///   q = 1 - p
///   b = weighted avg win magnitude
///   a = weighted avg loss magnitude
///
/// Returns fractional Kelly, clamped to [0, 0.5].
pub fn kelly_from_trades(
    pnl_values: &[f64],
    owm_weights: &[f64],
    fractional: f64,    // default 0.25 = quarter-Kelly
    risk_appetite: f64,  // 0.0 - 1.0+
) -> f64 {
    if pnl_values.len() < 10 || pnl_values.len() != owm_weights.len() {
        return 0.0;
    }

    let mut win_weights = 0.0_f64;
    let mut loss_weights = 0.0_f64;
    let mut win_pnl_weighted = 0.0_f64;
    let mut loss_pnl_weighted = 0.0_f64;

    for (pnl, w) in pnl_values.iter().zip(owm_weights.iter()) {
        let w = w.max(0.0);
        if *pnl > 0.0 {
            win_weights += w;
            win_pnl_weighted += w * pnl;
        } else {
            loss_weights += w;
            loss_pnl_weighted += w * pnl.abs();
        }
    }

    let total_weight = win_weights + loss_weights;
    if total_weight <= 0.0 { return 0.0; }

    let p = win_weights / total_weight; // weighted win probability
    let q = 1.0 - p;

    let b = if win_weights > 0.0 { win_pnl_weighted / win_weights } else { 0.0 }; // avg win
    let a = if loss_weights > 0.0 { loss_pnl_weighted / loss_weights } else { 0.0 }; // avg loss

    if b <= 0.0 { return 0.0; }

    let f_star = if a <= 0.0 {
        f64::MAX // no losses → kelly says bet max
    } else {
        p / a - q / b // Generalized Kelly: f* = p/a - q/b
    };

    (f_star * fractional * risk_appetite).clamp(0.0, 0.5)
}

// ═══════════════════════════════════════════════════════════════
// Legitimacy Gate (порт: legitimacy.py)
// ═══════════════════════════════════════════════════════════════

/// Порт: legitimacy.py — "Has the agent EARNED the right to trade?"
/// Hard gate (go/no-go) в отличие от DQS (scoring).
/// Weights: sample=0.30, memory_quality=0.15, regime=0.25, streak=0.15, drawdown=0.15
pub fn compute_legitimacy(
    trade_count: usize,
    regime_trade_count: usize,
    avg_context_drift: f64,
    consecutive_losses: u32,
    drawdown_pct: f64,
) -> LegitimacyResult {
    // Factor scores (порт: legitimacy.py:8-55)
    let f_sample = match trade_count {
        n if n >= 30 => 1.0,
        n if n >= 15 => 0.7,
        n if n >= 5  => 0.4,
        _ => 0.1,
    };

    let f_memory = (1.0 - avg_context_drift).clamp(0.0, 1.0);

    let f_regime = match regime_trade_count {
        n if n >= 10 => 1.0,
        n if n >= 5  => 0.7,
        n if n >= 2  => 0.4,
        _ => 0.1,
    };

    let f_streak = match consecutive_losses {
        0 => 1.0,
        1..=2 => 0.8,
        3..=4 => 0.5,
        _ => 0.2,
    };

    let f_dd = if drawdown_pct < 5.0 { 1.0 }
        else if drawdown_pct < 10.0 { 0.7 }
        else if drawdown_pct < 20.0 { 0.4 }
        else { 0.1 };

    // Weighted sum (порт: legitimacy.py:60-66)
    let score = f_sample * 0.30
        + f_memory * 0.15
        + f_regime * 0.25
        + f_streak * 0.15
        + f_dd * 0.15;

    let (tier, multiplier) = if score >= 0.7 {
        ("full", 1.0)
    } else if score >= 0.4 {
        ("reduced", 0.5)
    } else {
        ("skip", 0.0)
    };

    LegitimacyResult {
        score,
        tier: tier.to_string(),
        position_multiplier: multiplier,
        factors: LegitimacyFactors {
            sample_sufficiency: f_sample,
            memory_quality: f_memory,
            regime_confidence: f_regime,
            streak_state: f_streak,
            drawdown_state: f_dd,
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LegitimacyResult {
    pub score: f64,
    pub tier: String,
    pub position_multiplier: f64,
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
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Numerically stable sigmoid (порт: dqs.py:530-536)
fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let ex = x.exp();
        ex / (1.0 + ex)
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_input() -> DqsInput {
        DqsInput {
            regime_win_rate: Some(0.6),
            overall_win_rate: 0.55,
            proposed_lot: 0.1,
            kelly_fraction: Some(0.1),
            owm_score: 0.5,
            drawdown_pct: 3.0,
            consecutive_losses: 0,
            confidence: 0.7,
            avg_pnl_r_similar: 0.3,
        }
    }

    #[test]
    fn test_dqs_go_tier() {
        let engine = DqsEngine::new();
        let result = engine.compute(&default_input());
        assert_eq!(result.tier, DqsTier::Go, "Good conditions should be GO");
        assert!(result.score >= 5.0, "Score should be >= 5.0, got {:.2}", result.score);
        assert_eq!(result.position_multiplier, 1.0);
    }

    #[test]
    fn test_dqs_skip_in_danger() {
        let engine = DqsEngine::new();
        let input = DqsInput {
            regime_win_rate: Some(0.2),
            overall_win_rate: 0.55,
            proposed_lot: 0.5,
            kelly_fraction: Some(0.05),
            owm_score: 0.1,
            drawdown_pct: 25.0,   // Hard stop: > 20%
            consecutive_losses: 5, // Hard stop: > 4
            confidence: 0.3,
            avg_pnl_r_similar: -0.8,
        };
        let result = engine.compute(&input);
        assert_eq!(result.tier, DqsTier::Skip, "Danger conditions should SKIP");
        assert_eq!(result.position_multiplier, 0.0);
    }

    #[test]
    fn test_factor_regime_match_equal() {
        let score = factor_regime_match(Some(0.5), 0.5);
        assert!((score - 2.0).abs() < 0.01, "Equal WR = max score 2.0");
    }

    #[test]
    fn test_factor_regime_match_half() {
        let score = factor_regime_match(Some(0.25), 0.5);
        assert!((score - 1.0).abs() < 0.01, "Half WR = score 1.0");
    }

    #[test]
    fn test_factor_position_sizing_exact_kelly() {
        let score = factor_position_sizing(0.1, Some(0.1));
        assert!((score - 2.0).abs() < 0.01, "Exact Kelly = max score 2.0");
    }

    #[test]
    fn test_factor_position_sizing_far_from_kelly() {
        let score = factor_position_sizing(0.5, Some(0.1));
        assert!(score < 0.5, "4x Kelly deviation should give low score, got {:.2}", score);
    }

    #[test]
    fn test_factor_risk_hard_stop_drawdown() {
        let score = factor_risk_state(25.0, 0, 0.8);
        assert_eq!(score, 0.0, "DD > 20% = hard stop 0.0");
    }

    #[test]
    fn test_factor_risk_hard_stop_losses() {
        let score = factor_risk_state(5.0, 5, 0.8);
        assert_eq!(score, 0.0, "Losses > 4 = hard stop 0.0");
    }

    #[test]
    fn test_factor_historical_positive() {
        let score = factor_historical_pattern(1.0);
        assert!(score > 1.5, "Positive history should give > 1.5, got {:.2}", score);
    }

    #[test]
    fn test_factor_historical_negative() {
        let score = factor_historical_pattern(-1.0);
        assert!(score < 0.5, "Negative history should give < 0.5, got {:.2}", score);
    }

    // Kelly tests

    #[test]
    fn test_kelly_profitable_strategy() {
        let pnls = vec![10.0, -5.0, 15.0, -3.0, 8.0, -4.0, 12.0, -6.0, 7.0, -2.0, 20.0, -5.0];
        let weights = vec![1.0; 12];
        let k = kelly_from_trades(&pnls, &weights, 0.25, 1.0);
        assert!(k > 0.0, "Profitable strategy should have positive Kelly, got {:.4}", k);
        assert!(k <= 0.5, "Kelly should be clamped to 0.5, got {:.4}", k);
    }

    #[test]
    fn test_kelly_losing_strategy() {
        let pnls = vec![-10.0, -5.0, 2.0, -8.0, -3.0, -7.0, 1.0, -6.0, -4.0, -9.0, -2.0];
        let weights = vec![1.0; 11];
        let k = kelly_from_trades(&pnls, &weights, 0.25, 1.0);
        assert_eq!(k, 0.0, "Losing strategy should have Kelly = 0, got {:.4}", k);
    }

    #[test]
    fn test_kelly_insufficient_data() {
        let pnls = vec![10.0, -5.0, 3.0];
        let weights = vec![1.0; 3];
        let k = kelly_from_trades(&pnls, &weights, 0.25, 1.0);
        assert_eq!(k, 0.0, "< 10 trades = Kelly 0");
    }

    // Legitimacy tests

    #[test]
    fn test_legitimacy_full() {
        let result = compute_legitimacy(30, 10, 0.1, 0, 3.0);
        assert_eq!(result.tier, "full");
        assert_eq!(result.position_multiplier, 1.0);
    }

    #[test]
    fn test_legitimacy_skip_new_strategy() {
        let result = compute_legitimacy(2, 0, 0.9, 3, 15.0);
        assert_eq!(result.tier, "skip");
        assert_eq!(result.position_multiplier, 0.0);
    }

    #[test]
    fn test_legitimacy_reduced() {
        let result = compute_legitimacy(20, 3, 0.3, 2, 8.0);
        assert!(result.score >= 0.4 && result.score < 0.7,
            "Score {:.2} should be in reduced range", result.score);
    }

    // Sigmoid test

    #[test]
    fn test_sigmoid_symmetry() {
        let s1 = sigmoid(5.0);
        let s2 = sigmoid(-5.0);
        assert!((s1 + s2 - 1.0).abs() < 1e-10, "sigmoid(x) + sigmoid(-x) = 1");
    }

    #[test]
    fn test_sigmoid_zero() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10, "sigmoid(0) = 0.5");
    }

    // Calibrate tests

    #[test]
    fn test_calibrate_thresholds() {
        let mut engine = DqsEngine::new();
        let scores: Vec<f64> = (0..100).map(|i| 3.0 + (i as f64 * 0.05)).collect();
        engine.calibrate_thresholds(&scores);
        assert!(engine.skip_threshold > 0.0);
        assert!(engine.caution_threshold > engine.skip_threshold);
    }
}
