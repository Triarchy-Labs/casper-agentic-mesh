// src/modules/risk.rs
use crate::brain_feeds::BrainFeeds;

/// Модуль Динамической Матрицы Рисков
/// Никогда не возвращает статическое 5x плечо, если рынок слишком волатилен.
pub struct RiskMatrix {
    pub base_leverage: f64,
    pub max_leverage: f64,
}

impl Default for RiskMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskMatrix {
    pub fn new() -> Self {
        RiskMatrix {
            base_leverage: 5.0, // Дефолт в 5х для защиты (Токсичный рынок)
            max_leverage: 10.0, // Clean Trend (по протоколу Casino Heist)
        }
    }

    /// Рассчитывает безопасное плечо на основе волатильности (True Range)
    /// Протокол: Снижение плеча в 2 раза увеличивает ширину дыхания монеты ровно в 2 раза.
    pub fn calculate_dynamic_leverage(&self, atr: f64, current_price: f64) -> f64 {
        // Вычисляем процент хода свечи
        let atr_pct = (atr / current_price) * 100.0;

        let mut output_leverage = self.max_leverage; // Начинаем с 10x

        // Мясорубка / Сквизы (Toxic Volatility) -> Режем плечо до 5x для ширины дыхания
        if atr_pct > 3.0 {
            output_leverage = self.base_leverage; // 5x
        } 
        
        // Экстремальный сквиз -> Аппаратный дефенс
        if atr_pct > 5.0 {
            output_leverage = 2.0; 
        }

        output_leverage
    }

    /// Корректировка макро-гравитации: Если заходим в лонг на красном BTC, режем плечо.
    pub fn apply_macro_penalty(&self, base_calculated_lev: f64, side: &str, btc_score: f64) -> f64 {
        if side == "Buy" && btc_score < -1.0 {
            return base_calculated_lev * 0.5; // Режем в 2 раза за контртренд
        } else if side == "Sell" && btc_score > 1.0 {
            return base_calculated_lev * 0.5; 
        }
        base_calculated_lev
    }

    /// Рассчитывает динамический размер позиции на основе баланса, скора и волатильности
    /// V11: Kelly Hybrid — математически оптимальный sizing из DNA + fallback ступенчатый
    pub fn calculate_position_size(&self, available_balance: f64, bot_score: f64, atr_pct: f64, symbol: &str) -> f64 {
        // 1. Иерархия размеров из Мастер-Казначея (absolute $)
        let treasury_size = BrainFeeds::read_treasury_size(bot_score);

        // 1.1. V11 KELLY HYBRID: DNA-based sizing with step-function fallback
        let pct_based = Self::kelly_hybrid_size(available_balance, bot_score, symbol);

        // 1.2. Smart merge: use whichever is LARGER (Treasury absolute vs Kelly/% of balance)
        // This ensures: small deposit → Treasury floors apply; large deposit → scales up
        let mut target_size_usdt = treasury_size.max(pct_based);

        // 1.5. ПРАВОЕ ПОЛУШАРИЕ: Если монета S-TIER, масштабируем размер
        let (alpha_multiplier, _) = BrainFeeds::read_alpha_boost(symbol);
        if alpha_multiplier > 1.0 {
            target_size_usdt *= alpha_multiplier;
            tracing::info!(symbol = %symbol, multiplier = format!("{alpha_multiplier:.1}").as_str(), size = format!("{target_size_usdt:.2}").as_str(), "[PREDATOR MODE] Size boosted");
        }

        // 1.7. SWARM IDLE BOOST: если другие боты спят, забираем их ликвидность
        let idle_boost = BrainFeeds::read_swarm_idle_boost();
        if idle_boost > 1.0 {
            target_size_usdt *= idle_boost;
        }

        // 2. Волатильность (Toxic Market Penalty)
        if atr_pct > 3.0 {
            target_size_usdt *= 0.5;
        }
        
        // V11: Dynamic cap from AutoRamp phase (was FORENSIC-01 static 30%)
        // Seed=10%, Sprout=15%, Growth=20%, Mature=25%, Apex=30%
        let phase_cap = crate::auto_ramp::AutoRamp::max_position_pct();
        if target_size_usdt > available_balance * phase_cap {
            target_size_usdt = available_balance * phase_cap;
        }

        // BUG-17 FIX: absolute minimum — prevent dust trades
        if target_size_usdt < 1.0 {
            target_size_usdt = 1.0;
        }

        target_size_usdt
    }

    /// V11.1 Q1: Institutional Kelly Criterion — proper formula + Half-Kelly + shrinkage
    /// f* = (p × b - q) / b  where p=win_rate, q=1-p, b=avg_win/avg_loss
    /// Then: Half-Kelly = f*/2 (industry standard for variance reduction)
    /// Then: Bayesian shrinkage toward prior (0.5 WR) when sample N < 50
    fn kelly_hybrid_size(available_balance: f64, bot_score: f64, symbol: &str) -> f64 {
        let snapshot_path = crate::safe_io::data_file("hive_mind_snapshot.json");
        
        if let Ok(data) = std::fs::read_to_string(snapshot_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(entity) = json.get(symbol) {
                    let trade_count = entity["trade_count"].as_i64().unwrap_or(0);
                    let win_rate = entity["win_rate"].as_f64().unwrap_or(0.5);
                    let avg_loss = entity["avg_loss_size"].as_f64().unwrap_or(0.0).abs();
                    let avg_win = entity["avg_win_size"].as_f64().unwrap_or(0.0);

                    // Need ≥10 trades for any Kelly estimate
                    if trade_count >= 10 && avg_loss > 0.0 && avg_win > 0.0 {
                        let kelly = Self::kelly_fraction(
                            win_rate, avg_win, avg_loss, trade_count as u64
                        );
                        
                        tracing::info!(
                            symbol = %symbol,
                            wr = format!("{:.0}%", win_rate * 100.0).as_str(),
                            kelly = format!("{:.1}%", kelly * 100.0).as_str(),
                            trades = trade_count,
                            "📊 [KELLY] Institutional Half-Kelly sizing"
                        );
                        
                        return available_balance * kelly;
                    }
                }
            }
        }

        // Fallback: original step-function (no DNA data)
        if bot_score >= 10.0 {
            available_balance * 0.25
        } else if bot_score >= 7.0 {
            available_balance * 0.15
        } else if bot_score >= 5.0 {
            available_balance * 0.10
        } else {
            available_balance * 0.05
        }
    }

    /// Pure Kelly fraction calculator (testable without file I/O)
    /// Returns position size as fraction of bankroll [0.03, 0.40]
    pub fn kelly_fraction(win_rate: f64, avg_win: f64, avg_loss: f64, sample_n: u64) -> f64 {
        // Step 1: Bayesian shrinkage — pull win_rate toward prior (0.5) when N is small
        // shrinkage_factor = N / (N + K), where K=50 is equivalent sample size of prior
        let prior_wr = 0.5;
        let k = 50.0; // Equivalent prior sample size
        let shrink = sample_n as f64 / (sample_n as f64 + k);
        let adj_wr = prior_wr + shrink * (win_rate - prior_wr);
        
        // Step 2: Proper Kelly formula
        // f* = (p × b - q) / b
        // where p = adjusted win rate, q = 1 - p, b = avg_win / avg_loss
        let p = adj_wr.clamp(0.01, 0.99);
        let q = 1.0 - p;
        let b = (avg_win / avg_loss.max(0.001)).max(0.01);
        let kelly_full = ((p * b) - q) / b;
        
        // Step 3: Half-Kelly (industry standard for variance reduction)
        let kelly_half = kelly_full * 0.5;

        // Step 4: Edge floor — if Kelly < 3%, fees likely > edge → minimum sizing
        // Clamp to [0.03, 0.40]
        kelly_half.clamp(0.03, 0.40)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn rm() -> RiskMatrix { RiskMatrix::new() }

    // ═══ DYNAMIC LEVERAGE ═══

    #[test]
    fn test_leverage_normal_vol() {
        // ATR 1% of price → max leverage 10x
        let lev = rm().calculate_dynamic_leverage(1.0, 100.0); // 1%
        assert_eq!(lev, 10.0, "Normal vol should give max 10x");
    }

    #[test]
    fn test_leverage_toxic_vol() {
        // ATR 4% of price → base leverage 5x
        let lev = rm().calculate_dynamic_leverage(4.0, 100.0); // 4%
        assert_eq!(lev, 5.0, "Toxic vol should cut to 5x");
    }

    #[test]
    fn test_leverage_extreme_vol() {
        // ATR 6% of price → emergency 2x
        let lev = rm().calculate_dynamic_leverage(6.0, 100.0); // 6%
        assert_eq!(lev, 2.0, "Extreme vol should force 2x defense");
    }

    // ═══ MACRO PENALTY ═══

    #[test]
    fn test_macro_penalty_long_in_bear() {
        // Buying when BTC bearish → 50% cut
        let lev = rm().apply_macro_penalty(10.0, "Buy", -2.0);
        assert_eq!(lev, 5.0, "Long in bear should halve leverage");
    }

    #[test]
    fn test_macro_penalty_short_in_bull() {
        let lev = rm().apply_macro_penalty(10.0, "Sell", 2.0);
        assert_eq!(lev, 5.0, "Short in bull should halve leverage");
    }

    #[test]
    fn test_macro_penalty_aligned() {
        // Long when BTC bullish → no penalty
        let lev = rm().apply_macro_penalty(10.0, "Buy", 2.0);
        assert_eq!(lev, 10.0, "Aligned trade should keep full leverage");
    }

    #[test]
    fn test_macro_penalty_neutral_btc() {
        // BTC score between -1 and 1 → no penalty either way
        let lev_long = rm().apply_macro_penalty(10.0, "Buy", -0.5);
        let lev_short = rm().apply_macro_penalty(10.0, "Sell", 0.5);
        assert_eq!(lev_long, 10.0);
        assert_eq!(lev_short, 10.0);
    }

    // ═══ KELLY FALLBACK (step-function, no DNA file) ═══

    #[test]
    fn test_kelly_fallback_high_score() {
        // Score ≥10 → 25% of balance
        let size = RiskMatrix::kelly_hybrid_size(1000.0, 10.0, "NONEXISTENT_SYMBOL");
        assert!((size - 250.0).abs() < 0.01, "Score≥10 should give 25%, got {}", size);
    }

    #[test]
    fn test_kelly_fallback_medium_score() {
        let size = RiskMatrix::kelly_hybrid_size(1000.0, 7.0, "NONEXISTENT_SYMBOL");
        assert!((size - 150.0).abs() < 0.01, "Score≥7 should give 15%, got {}", size);
    }

    #[test]
    fn test_kelly_fallback_low_score() {
        let size = RiskMatrix::kelly_hybrid_size(1000.0, 5.0, "NONEXISTENT_SYMBOL");
        assert!((size - 100.0).abs() < 0.01, "Score≥5 should give 10%, got {}", size);
    }

    #[test]
    fn test_kelly_fallback_minimum_score() {
        let size = RiskMatrix::kelly_hybrid_size(1000.0, 2.0, "NONEXISTENT_SYMBOL");
        assert!((size - 50.0).abs() < 0.01, "Low score should give 5%, got {}", size);
    }

    #[test]
    fn test_leverage_bounds() {
        let rm = rm();
        // Test all ATR ranges produce valid leverage
        for atr_pct_x100 in 0..1000 {
            let atr = atr_pct_x100 as f64 / 100.0; // 0.00 to 10.00
            let lev = rm.calculate_dynamic_leverage(atr, 100.0);
            assert!(lev >= 2.0 && lev <= 10.0, "Leverage {} out of [2,10] at ATR {}", lev, atr);
        }
    }

    // ═══ KELLY FRACTION (Q1: Institutional Formula) ═══

    #[test]
    fn test_kelly_strong_edge() {
        // 65% WR, 1.5:1 R:R, 200 trades → strong sizing
        let k = RiskMatrix::kelly_fraction(0.65, 1.5, 1.0, 200);
        assert!(k > 0.15, "Strong edge should size >15%, got {:.1}%", k * 100.0);
        assert!(k <= 0.40, "Should be capped at 40%");
    }

    #[test]
    fn test_kelly_weak_edge() {
        // 52% WR, 1:1 R:R, 200 trades → small sizing
        let k = RiskMatrix::kelly_fraction(0.52, 1.0, 1.0, 200);
        assert!(k >= 0.03, "Weak edge should still be ≥3% floor");
        assert!(k < 0.10, "Weak edge should be <10%, got {:.1}%", k * 100.0);
    }

    #[test]
    fn test_kelly_small_sample_shrinkage() {
        // 70% WR but only 15 trades → should shrink toward 50% prior
        let k_small = RiskMatrix::kelly_fraction(0.70, 1.5, 1.0, 15);
        let k_large = RiskMatrix::kelly_fraction(0.70, 1.5, 1.0, 200);
        assert!(k_small < k_large, "Small sample should shrink: {:.1}% < {:.1}%", 
            k_small * 100.0, k_large * 100.0);
    }

    #[test]
    fn test_kelly_losing_strategy_floor() {
        // 40% WR, 1:1 R:R → negative edge, should clamp to 3% floor
        let k = RiskMatrix::kelly_fraction(0.40, 1.0, 1.0, 200);
        assert!((k - 0.03).abs() < 0.01, "Losing strategy should hit 3% floor, got {:.1}%", k * 100.0);
    }

    #[test]
    fn test_kelly_output_bounds() {
        // Verify bounds across all reasonable inputs
        for wr in [0.30, 0.40, 0.50, 0.55, 0.60, 0.70, 0.80] {
            for rr in [0.5, 1.0, 1.5, 2.0, 3.0] {
                for n in [10, 25, 50, 100, 500] {
                    let k = RiskMatrix::kelly_fraction(wr, rr, 1.0, n);
                    assert!(k >= 0.03 && k <= 0.40, 
                        "Kelly {:.1}% out of [3,40] at WR={}, RR={}, N={}", k * 100.0, wr, rr, n);
                }
            }
        }
    }
}
