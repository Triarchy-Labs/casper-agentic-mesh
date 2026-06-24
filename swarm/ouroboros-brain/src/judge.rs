//! Score-Based Chief Judge V2 — port from Python orchestrator.py
//! 15 factors, all thresholds loaded from thresholds.toml.
//! Pure math — zero LLM calls.

use crate::state::{SymbolData, Verdict};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ═══════════════════════════════════════════════════════════
// THRESHOLD CONFIG (from thresholds.toml)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdsConfig {
    pub verdict: VerdictConfig,
    pub confidence: ConfidenceConfig,
    pub factor: FactorConfigs,
    pub gravity: HashMap<String, f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerdictConfig {
    pub threshold: f64,
    pub score_clamp: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfidenceConfig {
    pub base_active: f64,
    pub base_hold: f64,
    pub multiplier_active: f64,
    pub multiplier_hold: f64,
    pub cap: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactorConfigs {
    pub price_trend: PriceTrendConfig,
    pub funding_rate: FundingRateConfig,
    pub oi_change: OiChangeConfig,
    pub volume_surge: VolumeSurgeConfig,
    pub llm_sentiment: LlmSentimentConfig,
    pub alpha_station: AlphaStationConfig,
    pub ml_predictor: MlPredictorConfig,
    pub macro_bias: MacroBiasConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceTrendConfig {
    pub overbought_threshold: f64,
    pub overbought_score: f64,
    pub oversold_threshold: f64,
    pub oversold_score: f64,
    pub moderate_up_threshold: f64,
    pub moderate_up_score: f64,
    pub moderate_down_threshold: f64,
    pub moderate_down_score: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FundingRateConfig {
    pub squeeze_threshold: f64,
    pub squeeze_score: f64,
    pub overheat_threshold: f64,
    pub overheat_score: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OiChangeConfig {
    pub growth_threshold: f64,
    pub deleverge_threshold: f64,
    pub growth_up_score: f64,
    pub growth_down_score: f64,
    pub delev_up_score: f64,
    pub delev_down_score: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VolumeSurgeConfig {
    pub anomaly_threshold: f64,
    pub anomaly_multiplier: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmSentimentConfig {
    pub bias_score: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlphaStationConfig {
    pub squeeze_score: f64,
    pub long_squeeze_score: f64,
    pub squeeze_signal_bonus: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MlPredictorConfig {
    pub max_weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MacroBiasConfig {
    pub bullish_score: f64,
    pub bearish_score: f64,
}

pub fn load_thresholds(path: &Path) -> Result<ThresholdsConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let cfg: ThresholdsConfig = toml::from_str(&content)?;
    tracing::info!("✅ Loaded judge thresholds from {:?}", path);
    Ok(cfg)
}

// ═══════════════════════════════════════════════════════════
// JUDGE INPUT — all factors collected from agents
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct JudgeInput {
    pub data: SymbolData,

    // Factor 5: LLM debate
    pub bull_argument: String,
    pub bear_argument: String,

    // Factor 6: Alpha Station
    pub alpha_fresh: bool,
    pub alpha_squeeze: bool,

    // Factor 7: ML Predictor
    pub ml_fresh: bool,
    pub ml_direction: i32,    // -1, 0, +1
    pub ml_confidence: f64,   // 0.0 - 1.0

    // Factor 8: Macro Bias (LLM)
    pub macro_fresh: bool,
    pub macro_bias: String,   // "BULLISH", "BEARISH", "NEUTRAL"

    // Factor 9: MTF 4H Trend (EMA20/50 + RSI)
    pub mtf_score: f64,

    // Factors 10-13: Hyper Reader (Alpha Station demons)
    pub hyper_total: f64,

    // Factor 14: HiveMind (Memory Castle)
    pub hivemind_score: f64,

    // Factor 15: Macro Guard (FOMC/CPI event penalty)
    pub macro_guard_factor: f64,
}

// ═══════════════════════════════════════════════════════════
// JUDGE OUTPUT
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct JudgeVerdict {
    pub decision: Verdict,
    pub confidence: f64,
    pub score: f64,
}

// ═══════════════════════════════════════════════════════════
// CHIEF JUDGE V2 — 8 core factors (pure math)
// ═══════════════════════════════════════════════════════════

/// Bull keywords for Factor 5 (LLM sentiment analysis)
const BULL_KEYWORDS: &[&str] = &[
    "squeeze", "accumulation", "bounce", "support", "oversold",
    "reversal", "capitulation", "bullish",
];

/// Bear keywords for Factor 5
const BEAR_KEYWORDS: &[&str] = &[
    "liquidation", "pressure", "breakdown", "resistance", "overbought",
    "dump", "bearish", "overheated",
];

pub fn chief_judge_v2(input: &JudgeInput, cfg: &ThresholdsConfig) -> JudgeVerdict {
    let mut score = 0.0_f64;
    let f = &cfg.factor;
    let change = input.data.price_24h_change;
    let funding = input.data.funding_rate;
    let oi_delta = input.data.oi_change_pct;
    let vol_surge = input.data.volume_ratio;

    // ═══ FACTOR 1: Price Trend (Smart Money = counter-retail) ═══
    if change > f.price_trend.overbought_threshold {
        score += f.price_trend.overbought_score;   // -2.0
    } else if change < f.price_trend.oversold_threshold {
        score += f.price_trend.oversold_score;      // +2.0
    } else if change > f.price_trend.moderate_up_threshold {
        score += f.price_trend.moderate_up_score;   // +0.5
    } else if change < f.price_trend.moderate_down_threshold {
        score += f.price_trend.moderate_down_score; // -0.5
    }

    // ═══ FACTOR 2: Funding Rate ═══
    if funding < f.funding_rate.squeeze_threshold {
        score += f.funding_rate.squeeze_score;      // +1.5
    } else if funding > f.funding_rate.overheat_threshold {
        score += f.funding_rate.overheat_score;     // -1.5
    }

    // ═══ FACTOR 3: OI Change ═══
    if oi_delta > f.oi_change.growth_threshold {
        if change > 0.0 {
            score += f.oi_change.growth_up_score;   // +0.5
        } else {
            score += f.oi_change.growth_down_score; // -0.5
        }
    } else if oi_delta < f.oi_change.deleverge_threshold {
        if change > 0.0 {
            score += f.oi_change.delev_up_score;    // -0.3
        } else {
            score += f.oi_change.delev_down_score;  // +0.3
        }
    }

    // ═══ FACTOR 4: Volume Surge ═══
    if vol_surge > f.volume_surge.anomaly_threshold {
        score *= f.volume_surge.anomaly_multiplier; // ×1.3
    }

    // ═══ FACTOR 5: LLM Sentiment (keyword count) ═══
    let bull_text = input.bull_argument.to_lowercase();
    let bear_text = input.bear_argument.to_lowercase();
    let bull_strength: usize = BULL_KEYWORDS.iter().filter(|w| bull_text.contains(*w)).count();
    let bear_strength: usize = BEAR_KEYWORDS.iter().filter(|w| bear_text.contains(*w)).count();

    if bull_strength > bear_strength {
        score += f.llm_sentiment.bias_score;        // +0.5
    } else if bear_strength > bull_strength {
        score -= f.llm_sentiment.bias_score;        // -0.5
    }

    // ═══ FACTOR 6: Alpha Station ═══
    if input.alpha_fresh && input.alpha_squeeze {
        score += f.alpha_station.squeeze_signal_bonus; // +1.0 (squeeze detected)
    }

    // ═══ FACTOR 7: ML Prediction ═══
    if input.ml_fresh {
        let ml_weight = input.ml_direction as f64 * input.ml_confidence * f.ml_predictor.max_weight;
        score += ml_weight;
    }

    // ═══ FACTOR 8: Macro Bias (LLM) ═══
    if input.macro_fresh {
        match input.macro_bias.as_str() {
            "BULLISH" => score += f.macro_bias.bullish_score,  // +1.0
            "BEARISH" => score += f.macro_bias.bearish_score,  // -1.0 (stored as negative in toml)
            _ => {}
        }
    }

    // ═══ FACTOR 9: MTF 4H Trend ═══
    score += input.mtf_score; // ±1.5 max from EMA cross + RSI

    // ═══ FACTORS 10-13: Hyper Reader ═══
    score += input.hyper_total; // Combined: funding + OI + liq + whale

    // ═══ FACTOR 14: HiveMind (Memory Castle) ═══
    score += input.hivemind_score;

    // ═══ FACTOR 15: Macro Guard (event penalty) ═══
    score += input.macro_guard_factor; // -2.0 LOCKDOWN, -0.5 CAUTION, 0.0 CLEAR

    // ═══ GRAVITY ANCHOR — top assets get amplified ═══
    if let Some(&weight) = cfg.gravity.get(&input.data.symbol) {
        score *= weight;
    }

    // ═══ CLAMP ═══
    score = score.clamp(-cfg.verdict.score_clamp, cfg.verdict.score_clamp);

    // ═══ VERDICT ═══
    let (decision, confidence) = if score >= cfg.verdict.threshold {
        let conf = (cfg.confidence.base_active + score.abs() * cfg.confidence.multiplier_active)
            .min(cfg.confidence.cap);
        (Verdict::Buy, conf)
    } else if score <= -cfg.verdict.threshold {
        let conf = (cfg.confidence.base_active + score.abs() * cfg.confidence.multiplier_active)
            .min(cfg.confidence.cap);
        (Verdict::Sell, conf)
    } else {
        let conf = cfg.confidence.base_hold + score.abs() * cfg.confidence.multiplier_hold;
        (Verdict::Hold, conf)
    };

    JudgeVerdict {
        decision,
        confidence: (confidence * 10.0).round() / 10.0, // round to 1 decimal
        score: (score * 100.0).round() / 100.0,         // round to 2 decimals
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS — Python parity validation
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn load_test_cfg() -> ThresholdsConfig {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/thresholds.toml");
        load_thresholds(&path).expect("Failed to load thresholds.toml")
    }

    fn make_data(symbol: &str, change: f64, funding: f64, oi: f64, vol: f64) -> SymbolData {
        SymbolData {
            symbol: symbol.into(),
            price: 96000.0,
            price_24h_change: change,
            volume_24h: 1_000_000.0,
            volume_ratio: vol,
            funding_rate: funding,
            open_interest: 500_000.0,
            oi_change_pct: oi,
            timestamp: 0,
        }
    }

    #[test]
    fn test_thresholds_load() {
        let cfg = load_test_cfg();
        assert_eq!(cfg.verdict.threshold, 1.5);
        assert_eq!(cfg.verdict.score_clamp, 5.0);
        assert_eq!(cfg.factor.funding_rate.squeeze_threshold, -0.0003);
    }

    #[test]
    fn test_neutral_market_hold() {
        let cfg = load_test_cfg();
        let input = JudgeInput {
            data: make_data("BTCUSDT", 0.5, 0.0001, 1.0, 1.2),
            ..Default::default()
        };
        let v = chief_judge_v2(&input, &cfg);
        assert_eq!(v.decision, Verdict::Hold);
        assert!(v.score.abs() < 1.5, "neutral market should have |score| < threshold");
    }

    #[test]
    fn test_short_squeeze_buy() {
        // Oversold (-6%) + negative funding (-0.001) = strong LONG signal
        let cfg = load_test_cfg();
        let input = JudgeInput {
            data: make_data("BTCUSDT", -6.0, -0.001, 2.0, 1.5),
            ..Default::default()
        };
        let v = chief_judge_v2(&input, &cfg);
        assert_eq!(v.decision, Verdict::Buy);
        assert!(v.score > 0.0, "short squeeze should be positive score");
    }

    #[test]
    fn test_fomo_trap_sell() {
        // Overbought (+7%) + high funding (0.001) = SELL signal
        let cfg = load_test_cfg();
        let input = JudgeInput {
            data: make_data("ETHUSDT", 7.0, 0.001, 3.0, 1.5),
            ..Default::default()
        };
        let v = chief_judge_v2(&input, &cfg);
        assert_eq!(v.decision, Verdict::Sell);
        assert!(v.score < 0.0, "FOMO trap should be negative score");
    }

    #[test]
    fn test_volume_amplification() {
        let cfg = load_test_cfg();
        // Without volume surge
        let input_low = JudgeInput {
            data: make_data("SOLUSDT", -6.0, -0.001, 2.0, 1.5),
            ..Default::default()
        };
        let v_low = chief_judge_v2(&input_low, &cfg);

        // With volume surge (>3x)
        let input_high = JudgeInput {
            data: make_data("SOLUSDT", -6.0, -0.001, 2.0, 4.0),
            ..Default::default()
        };
        let v_high = chief_judge_v2(&input_high, &cfg);

        assert!(v_high.score.abs() > v_low.score.abs(),
            "volume surge should amplify score: {:.2} vs {:.2}", v_high.score, v_low.score);
    }

    #[test]
    fn test_ml_and_macro_factors() {
        let cfg = load_test_cfg();
        let input = JudgeInput {
            data: make_data("BTCUSDT", 0.0, 0.0, 0.0, 1.0),
            ml_fresh: true,
            ml_direction: 1,
            ml_confidence: 0.8,
            macro_fresh: true,
            macro_bias: "BULLISH".into(),
            ..Default::default()
        };
        let v = chief_judge_v2(&input, &cfg);
        // ML: 1 * 0.8 * 1.5 = 1.2 + Macro: 1.0 = 2.2 * gravity 1.5 = 3.3
        assert_eq!(v.decision, Verdict::Buy);
        assert!(v.score > 2.0, "ML + Macro should produce strong buy: {:.2}", v.score);
    }

    #[test]
    fn test_score_clamped() {
        let cfg = load_test_cfg();
        // Stack ALL bullish factors
        let input = JudgeInput {
            data: make_data("BTCUSDT", -8.0, -0.002, -6.0, 5.0),
            bull_argument: "squeeze accumulation bounce oversold reversal".into(),
            alpha_fresh: true,
            alpha_squeeze: true,
            ml_fresh: true,
            ml_direction: 1,
            ml_confidence: 1.0,
            macro_fresh: true,
            macro_bias: "BULLISH".into(),
            ..Default::default()
        };
        let v = chief_judge_v2(&input, &cfg);
        assert!(v.score <= 5.0, "score must be clamped to 5.0, got {:.2}", v.score);
        assert!(v.score >= -5.0, "score must be clamped to -5.0, got {:.2}", v.score);
        assert_eq!(v.decision, Verdict::Buy);
    }
}
