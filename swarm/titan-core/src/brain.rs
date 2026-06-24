// src/brain.rs — Exchange-Agnostic NeuralBrain
use crate::brain_feeds::BrainFeeds;
use crate::indicators;
use crate::confidence::ConfidenceEngine;

/// Data provider trait — implement for any exchange (Mantle DEX, Bybit, mock).
pub trait DataProvider: Send + Sync {
    fn funding_rate(&self, symbol: &str) -> Option<f64>;
    fn oi_delta(&self, symbol: &str) -> Option<f64>;
    fn price_change_24h(&self, symbol: &str) -> Option<f64>;
}

/// Главный вычислительный центр. Собирает данные из модулей и генерирует Торговый Сигнал (Score).
pub struct NeuralBrain;

impl NeuralBrain {
    /// Возвращает (score, verdict, breakdown_log)
    pub fn calculate_entry_score(
        data: &dyn DataProvider,
        symbol: &str,
        btc_health_score: f64,
        last_closes: &[f64],
        last_vols: &[f64],
    ) -> (f64, String, String) {

        // 🚨 DEAD MAN'S SWITCH
        if !BrainFeeds::is_swarm_heartbeat_alive() {
            return (-999.0, "SWARM_DEAD".to_string(), "DEAD_MAN_SWITCH".to_string());
        }

        // SMART-4: API BLINDNESS GUARD — если API не ответил, НЕ торгуем
        let funding_rate = match data.funding_rate(symbol) {
            Some(v) => v, None => return (0.0, "NONE".to_string(), "API_BLIND: funding_rate=None".to_string()),
        };
        let oi_delta = match data.oi_delta(symbol) {
            Some(v) => v, None => return (0.0, "NONE".to_string(), "API_BLIND: oi_delta=None".to_string()),
        };
        let change_24h = match data.price_change_24h(symbol) {
            Some(v) => v, None => return (0.0, "NONE".to_string(), "API_BLIND: change_24h=None".to_string()),
        };

        let macro_bias = BrainFeeds::read_macro_bias();
        let ouroboros_verdict = BrainFeeds::read_ouroboros_verdict(symbol);
        
        let mut score: f64 = 0.0;
        let mut breakdown: Vec<String> = Vec::new();

        // Pre-compute candle color (used by VolSurge + RSI)
        let green_candle = indicators::is_green_candle(last_closes);

        // 0a. VOLUME SURGE (FORENSIC-14: direction-aware via candle color)
        let (vol_surge, vol_ratio) = indicators::calc_volume_surge(last_vols);
        if vol_surge {
            if green_candle {
                score += 1.0; breakdown.push(format!("VolSurge+1.0(r{vol_ratio:.1})"));
            } else {
                score -= 1.0; breakdown.push(format!("VolSurge-1.0(r{vol_ratio:.1})"));
            }
        }

        // 0b. RSI WILDER'S 14 (plug-in из indicators.rs)
        let rsi = indicators::calc_rsi_wilders(last_closes, 14);
        if rsi > 55.0 && rsi < 75.0 && green_candle { score += 1.0; breakdown.push(format!("RSI+1.0({rsi:.0})")); }
        else if rsi >= 75.0 && !green_candle { score -= 1.0; breakdown.push(format!("RSI-1.0(OB{rsi:.0})")); } // BUG-18: overbought SHORT
        else if rsi < 45.0 && rsi > 25.0 && !green_candle { score -= 1.0; breakdown.push(format!("RSI-1.0({rsi:.0})")); }
        else if rsi <= 25.0 && green_candle { score += 1.0; breakdown.push(format!("RSI+1.0(OS{rsi:.0})")); } // BUG-18: oversold LONG
        
        // 1. Квантовый Фандинг (Continuous Scaling - Vector Dogma 2)
        let funding_score = (funding_rate * -1000.0).clamp(-2.0, 3.0);
        score += funding_score;
        if funding_score.abs() > 0.1 { breakdown.push(format!("FR{funding_score:+.1}")); }
        
        // 2. Векторная Кросс-Валидация OI (Vector Dogma 1: No Naked Vectors)
        if oi_delta > 3.0 {
            if change_24h > 0.0 {
                score += 1.0; breakdown.push("OI+1.0(momentum)".to_string());
            } else {
                score -= 1.5; breakdown.push("OI-1.5(knife)".to_string());
            }
        } else if oi_delta < -3.0 {
            score += 0.5; breakdown.push("OI+0.5(flush)".to_string());
        }

        // 3. Читаем Радары из Альфы для контекста
        let liq_bias = BrainFeeds::read_liq_magnet(symbol);
        score += liq_bias;
        if liq_bias.abs() > 0.1 { breakdown.push(format!("Liq{liq_bias:+.1}")); }

        let whale_bias = BrainFeeds::read_whale_radar(symbol);
        score += whale_bias;
        if whale_bias.abs() > 0.1 { breakdown.push(format!("Whale{whale_bias:+.1}")); }

        let oi_tracker_bias = BrainFeeds::read_oi_tracker(symbol);
        score += oi_tracker_bias;
        if oi_tracker_bias.abs() > 0.1 { breakdown.push(format!("OIT{oi_tracker_bias:+.1}")); }
        
        // 4. Истинный Моментум против FOMO (Vector Dogma 3)
        if change_24h > 5.0 {
            if oi_tracker_bias < 0.0 {
                score -= 2.0; breakdown.push("FOMO-2.0(trap)".to_string());
            } else {
                score += 0.5; breakdown.push("Momentum+0.5".to_string());
            }
        } else if change_24h < -5.0 {
            if oi_tracker_bias > 0.0 {
                score += 2.0; breakdown.push("Bottom+2.0".to_string());
            } else {
                score -= 0.5; breakdown.push("Dump-0.5".to_string());
            }
        }
        
        // 5. Macro Bias LLM
        match macro_bias.as_str() {
            "LONG" => { score += 1.0; breakdown.push("Macro+1.0".to_string()); }
            "SHORT" => { score -= 1.0; breakdown.push("Macro-1.0".to_string()); }
            _ => {}
        }
        
        // 6. Ouroboros AI Override (ML Predictor Factor 7)
        match ouroboros_verdict.as_str() {
            "BUY" => { score += 3.0; breakdown.push("Ouro+3.0".to_string()); }
            "SELL" => { score -= 3.0; breakdown.push("Ouro-3.0".to_string()); }
            _ => {} 
        }

        // 7. ПРАВОЕ ПОЛУШАРИЕ: Alpha Boost (PREDATOR-03: direction-aware amplification)
        let (_alpha_multiplier, alpha_score_bonus) = BrainFeeds::read_alpha_boost(symbol);
        if alpha_score_bonus > 0.0 {
            if score > 0.0 {
                score += alpha_score_bonus;
                breakdown.push(format!("Alpha+{alpha_score_bonus:.0}"));
            } else if score < 0.0 {
                score -= alpha_score_bonus; // Amplify SHORT direction
                breakdown.push(format!("Alpha-{alpha_score_bonus:.0}(short)"));
            }
        }

        // 8. Volume Anomaly Amplifier (BUG-19 FIX: only amplifies positive score)
        if vol_ratio > 3.0 && score > 0.0 {
            let pre_amp = score;
            score *= 1.3;
            breakdown.push(format!("VolAmp×1.3({pre_amp:.1}→{score:.1})"));
        }

        // 9. CONFIDENCE SCORE (Vector 3: DNA Memory)
        let confidence = ConfidenceEngine::calculate(symbol);
        score += confidence;
        if confidence.abs() > 0.1 { breakdown.push(format!("Conf{confidence:+.1}")); }

        // 11. HOUR BIAS (Vector 6: мягкий модификатор, НЕ блокирует)
        let hour_bias = BrainFeeds::read_hour_bias();
        score += hour_bias;
        if hour_bias.abs() > 0.1 { breakdown.push(format!("Hour{hour_bias:+.1}")); }


        // V11: OVERCONFIDENCE GUARD (Swarmbots-inspired)
        // Extreme conviction (all factors aligned) often = crowded trade / trap
        if score.abs() > 12.0 {
            let original = score;
            score = score.signum() * 12.0;
            breakdown.push(format!("OvConf({original:.1}→{score:.1})"));
        }

        // V11.0.1 Q5: TRANSACTION COST DRAG
        // Bybit taker: 0.055% × 2 (entry+exit) = 0.11% round-trip
        // At avg 5x leverage → 0.55% of margin. Penalize marginal trades.
        // -0.3 score ≈ filters out trades where fees > expected edge
        let fee_penalty = Self::fee_drag_penalty(score);
        if fee_penalty.abs() > 0.01 {
            score += fee_penalty;
            breakdown.push(format!("Fee{fee_penalty:+.1}"));
        }

        // V11.1 Q4: MARKET REGIME DETECTION
        // Efficiency Ratio classifies market → adjust score for regime fitness
        let (regime, regime_mod) = indicators::detect_market_regime(last_closes);
        if regime_mod.abs() > 0.01 {
            score += regime_mod;
            breakdown.push(format!("Regime({regime}{regime_mod:+.1})"));
        }

        // Порог входа: Calibration → BTC dynamic → default
        let mut threshold = {
            // Cross-platform: check env var or use default
            let cal_threshold = std::env::var("TITAN_CALIBRATION_PATH").ok()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                .and_then(|j| j["brain_threshold"].as_f64());
            match cal_threshold {
                Some(t) => t,
                None => if btc_health_score.abs() >= 3.0 { 2.5 } else { 3.5 },
            }
        };
        
        if ouroboros_verdict == "HOLD" || ouroboros_verdict == "NEUTRAL" {
            threshold = 5.0;
        }

        // Определяем предварительный вердикт
        let mut verdict = "NONE".to_string();
        if score >= threshold && change_24h >= -10.0 {
            verdict = "LONG".to_string();
        } else if score <= -threshold && change_24h <= 10.0 {
            verdict = "SHORT".to_string();
        }

        // 10. DIRECTIONAL BIAS (Vector 2: блок противоположного направления)
        if verdict != "NONE" {
            if let Some(best_side) = ConfidenceEngine::get_directional_bias(symbol) {
                let side_match = (verdict == "LONG" && best_side == "Buy") || 
                                 (verdict == "SHORT" && best_side == "Sell");
                if !side_match {
                    breakdown.push(format!("DirBlock(best={best_side})"));
                    verdict = "NONE".to_string(); // Блокируем противоречивое направление
                }
            }
        }

        let breakdown_str = format!("Score={:.1} [{}] thr={:.1}", score, breakdown.join(" "), threshold);
        (score, verdict, breakdown_str)
    }

    // ═══ PURE HELPERS (testable without async/API) ═══

    /// Overconfidence guard: clamps extreme scores to ±12
    #[allow(dead_code)]
    pub fn apply_overconfidence_guard(score: f64) -> f64 {
        if score.abs() > 12.0 { score.signum() * 12.0 } else { score }
    }

    /// Determine verdict from score vs threshold + 24h crash guard
    #[allow(dead_code)]
    pub fn determine_verdict(score: f64, threshold: f64, change_24h: f64) -> String {
        if score >= threshold && change_24h >= -10.0 {
            "LONG".to_string()
        } else if score <= -threshold && change_24h <= 10.0 {
            "SHORT".to_string()
        } else {
            "NONE".to_string()
        }
    }

    /// Calculate dynamic threshold based on BTC health
    #[allow(dead_code)]
    pub fn calculate_threshold(btc_health_score: f64, calibrated: Option<f64>) -> f64 {
        match calibrated {
            Some(t) => t,
            None => if btc_health_score.abs() >= 3.0 { 2.5 } else { 3.5 },
        }
    }

    /// Funding rate → score contribution (continuous scaling, clamped)
    #[allow(dead_code)]
    pub fn funding_score(funding_rate: f64) -> f64 {
        (funding_rate * -1000.0).clamp(-2.0, 3.0)
    }

    /// Q5: Transaction cost drag — reduces score magnitude by fee friction
    /// Applied sign-aware: positive score gets negative penalty, negative gets positive
    /// Only applies when |score| > 1.0 (below that, threshold blocks anyway)
    pub fn fee_drag_penalty(score: f64) -> f64 {
        const FEE_DRAG: f64 = 0.3; // ~0.11% round-trip × scaling factor
        if score.abs() < 1.0 {
            0.0 // Don't bother with sub-threshold scores
        } else {
            -score.signum() * FEE_DRAG
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══ OVERCONFIDENCE GUARD ═══

    #[test]
    fn test_overconfidence_clamp_high() {
        assert_eq!(NeuralBrain::apply_overconfidence_guard(15.0), 12.0);
    }

    #[test]
    fn test_overconfidence_clamp_low() {
        assert_eq!(NeuralBrain::apply_overconfidence_guard(-15.0), -12.0);
    }

    #[test]
    fn test_overconfidence_no_clamp() {
        assert_eq!(NeuralBrain::apply_overconfidence_guard(8.5), 8.5);
        assert_eq!(NeuralBrain::apply_overconfidence_guard(-8.5), -8.5);
    }

    // ═══ VERDICT GENERATION ═══

    #[test]
    fn test_verdict_long() {
        let v = NeuralBrain::determine_verdict(5.0, 3.5, 2.0);
        assert_eq!(v, "LONG");
    }

    #[test]
    fn test_verdict_short() {
        let v = NeuralBrain::determine_verdict(-5.0, 3.5, -2.0);
        assert_eq!(v, "SHORT");
    }

    #[test]
    fn test_verdict_none_below_threshold() {
        let v = NeuralBrain::determine_verdict(2.0, 3.5, 0.0);
        assert_eq!(v, "NONE");
    }

    #[test]
    fn test_verdict_crash_guard_blocks_long() {
        // Score says LONG but 24h crash >10% → blocked
        let v = NeuralBrain::determine_verdict(5.0, 3.5, -12.0);
        assert_eq!(v, "NONE", "Crash guard should block LONG during -12% dump");
    }

    #[test]
    fn test_verdict_pump_guard_blocks_short() {
        // Score says SHORT but 24h pump >10% → blocked
        let v = NeuralBrain::determine_verdict(-5.0, 3.5, 12.0);
        assert_eq!(v, "NONE", "Pump guard should block SHORT during +12% pump");
    }

    // ═══ THRESHOLD LOGIC ═══

    #[test]
    fn test_threshold_btc_trending() {
        let t = NeuralBrain::calculate_threshold(4.0, None);
        assert_eq!(t, 2.5, "BTC trending → lower threshold");
    }

    #[test]
    fn test_threshold_btc_flat() {
        let t = NeuralBrain::calculate_threshold(1.0, None);
        assert_eq!(t, 3.5, "BTC flat → higher threshold (caution)");
    }

    #[test]
    fn test_threshold_calibrated_override() {
        let t = NeuralBrain::calculate_threshold(4.0, Some(4.0));
        assert_eq!(t, 4.0, "Calibrated value should override BTC-based");
    }

    // ═══ FUNDING SCORE ═══

    #[test]
    fn test_funding_score_negative_rate() {
        // Negative funding → positive score (shorts are paying)
        let s = NeuralBrain::funding_score(-0.003);
        assert!(s > 0.0, "Negative funding should produce positive score");
        assert!(s <= 3.0, "Should be clamped to 3.0 max");
    }

    #[test]
    fn test_funding_score_positive_rate() {
        let s = NeuralBrain::funding_score(0.003);
        assert!(s < 0.0, "Positive funding should produce negative score");
        assert!(s >= -2.0, "Should be clamped to -2.0 min");
    }

    // ═══ FEE DRAG (Q5) ═══

    #[test]
    fn test_fee_drag_positive_score() {
        let p = NeuralBrain::fee_drag_penalty(5.0);
        assert!(p < 0.0, "Positive score should get negative fee drag");
        assert!((p - (-0.3)).abs() < 0.01, "Should be -0.3, got {}", p);
    }

    #[test]
    fn test_fee_drag_negative_score() {
        let p = NeuralBrain::fee_drag_penalty(-5.0);
        assert!(p > 0.0, "Negative score should get positive fee drag");
        assert!((p - 0.3).abs() < 0.01, "Should be +0.3, got {}", p);
    }

    #[test]
    fn test_fee_drag_sub_threshold_noop() {
        let p = NeuralBrain::fee_drag_penalty(0.5);
        assert_eq!(p, 0.0, "Sub-threshold score should have zero fee drag");
    }
}
