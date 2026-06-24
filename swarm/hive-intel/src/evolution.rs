/// Strategy Evolution Engine — tournament selection + statistical validation.
///
/// ПОРТИРОВАНО ИЗ:
///   - tradememory-protocol/src/tradememory/evolution/engine.py (265 строк)
/// - tradememory-protocol/src/tradememory/evolution/selector.py (210 строк)
/// - tradememory-protocol/src/tradememory/evolution/statistical_gates.py (170 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Pipeline per generation:
///   1. Generate strategy parameters (mutate existing or explore random)
///   2. Evaluate IS (in-sample) fitness
///   3. Rank by IS fitness → top N
///   4. Validate OOS (out-of-sample) for top N
///   5. Graduated / Graveyard → feed back to next generation
///
/// Statistical Gates:
///   - Deflated Sharpe Ratio (Bailey & Lopez de Prado 2014)
///   - Benjamini-Hochberg FDR correction
///   - Minimum Backtest Length calculator
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Strategy Hypothesis
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct StrategyParams {
    pub name: String,
    pub params: Vec<f64>,       // numerical parameters to tune
    pub generation: u32,
    pub parent_id: Option<u32>, // mutation parent
}

#[derive(Debug, Clone, Serialize)]
pub struct Fitness {
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    pub win_rate: f64,
    pub max_drawdown_pct: f64,
    pub trade_count: usize,
    pub total_pnl: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum HypothesisStatus {
    Pending,
    Backtesting,
    SurvivedIS,
    Graduated,
    Eliminated,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hypothesis {
    pub id: u32,
    pub params: StrategyParams,
    pub status: HypothesisStatus,
    pub fitness_is: Option<Fitness>,
    pub fitness_oos: Option<Fitness>,
    pub elimination_reason: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Selection Config (порт: selector.py:24-43)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SelectionConfig {
    pub top_n: usize,                // how many proceed to OOS
    pub min_is_trade_count: usize,   // IS minimum trades
    pub min_is_sharpe: f64,          // IS minimum Sharpe
    pub min_oos_sharpe: f64,         // OOS Sharpe threshold
    pub min_oos_trade_count: usize,  // OOS minimum trades
    pub max_oos_drawdown_pct: f64,   // max allowed DD
    pub min_oos_profit_factor: f64,  // min PF
    pub min_oos_win_rate: f64,       // min WR
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            top_n: 10,
            min_is_trade_count: 10,
            min_is_sharpe: 0.0,
            min_oos_sharpe: 1.0,
            min_oos_trade_count: 30,
            max_oos_drawdown_pct: 20.0,
            min_oos_profit_factor: 1.2,
            min_oos_win_rate: 0.4,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Rank & Select (порт: selector.py:63-194)
// ═══════════════════════════════════════════════════════════════

/// Rank hypotheses by IS fitness, filter weak ones, return top N.
/// Порт: selector.py:63-98
pub fn rank_by_is_fitness(
    hypotheses: &mut [Hypothesis],
    config: &SelectionConfig,
) -> Vec<usize> {
    // Filter: has IS fitness + meets minimum thresholds
    let mut viable: Vec<(usize, f64)> = hypotheses.iter().enumerate()
        .filter_map(|(i, h)| {
            h.fitness_is.as_ref().and_then(|f| {
                if f.trade_count >= config.min_is_trade_count && f.sharpe_ratio >= config.min_is_sharpe {
                    Some((i, f.sharpe_ratio))
                } else {
                    None
                }
            })
        })
        .collect();

    // Sort by Sharpe (descending)
    viable.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top N indices
    viable.iter().take(config.top_n).map(|(i, _)| *i).collect()
}

/// Validate hypothesis against OOS thresholds.
/// Порт: selector.py:101-136
pub fn validate_oos(h: &Hypothesis, config: &SelectionConfig) -> (bool, String) {
    match &h.fitness_oos {
        None => (false, "no OOS fitness data".to_string()),
        Some(f) => {
            if f.trade_count < config.min_oos_trade_count {
                return (false, format!("OOS trades={} < {}", f.trade_count, config.min_oos_trade_count));
            }
            if f.sharpe_ratio < config.min_oos_sharpe {
                return (false, format!("OOS Sharpe={:.2} < {:.1}", f.sharpe_ratio, config.min_oos_sharpe));
            }
            if f.max_drawdown_pct > config.max_oos_drawdown_pct {
                return (false, format!("OOS DD={:.1}% > {:.1}%", f.max_drawdown_pct, config.max_oos_drawdown_pct));
            }
            if f.profit_factor < config.min_oos_profit_factor {
                return (false, format!("OOS PF={:.2} < {:.1}", f.profit_factor, config.min_oos_profit_factor));
            }
            if f.win_rate < config.min_oos_win_rate {
                return (false, format!("OOS WR={:.2} < {:.1}", f.win_rate, config.min_oos_win_rate));
            }
            (true, String::new())
        }
    }
}

/// Full selection & elimination pipeline.
/// Порт: selector.py:139-194
pub fn select_and_eliminate(
    hypotheses: &mut [Hypothesis],
    config: &SelectionConfig,
) -> SelectionResult {
    let ranked = rank_by_is_fitness(hypotheses, config);
    let ranked_set: std::collections::HashSet<usize> = ranked.iter().cloned().collect();

    let mut result = SelectionResult::default();

    // Mark non-ranked as eliminated (IS stage)
    for (i, h) in hypotheses.iter_mut().enumerate() {
        if !ranked_set.contains(&i) && h.fitness_is.is_some() {
            h.status = HypothesisStatus::Eliminated;
            h.elimination_reason = Some("ranked below top N".to_string());
            result.eliminated_count += 1;
        }
    }

    // OOS validation for ranked
    for &idx in &ranked {
        let (passed, reason) = validate_oos(&hypotheses[idx], config);
        if passed {
            hypotheses[idx].status = HypothesisStatus::Graduated;
            result.graduated_count += 1;
            result.graduated_ids.push(idx);
        } else {
            hypotheses[idx].status = HypothesisStatus::Eliminated;
            hypotheses[idx].elimination_reason = Some(reason);
            result.eliminated_count += 1;
        }
    }

    result
}

#[derive(Debug, Default, Serialize)]
pub struct SelectionResult {
    pub graduated_count: usize,
    pub eliminated_count: usize,
    pub graduated_ids: Vec<usize>,
}

// ═══════════════════════════════════════════════════════════════
// Gaussian Mutation (порт: generator.py concept)
// ═══════════════════════════════════════════════════════════════

/// Mutate strategy parameters with Gaussian perturbation.
/// scale controls mutation intensity (0.0 = no change, 1.0 = 100% std).
pub fn mutate_params(params: &[f64], scale: f64, rng_seed: u64) -> Vec<f64> {
    // Simple LCG PRNG for no-dependency Gaussian approximation
    let mut state = rng_seed;
    params.iter().map(|&p| {
        // Box-Muller transform for Gaussian from uniform
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u1 = (state >> 33) as f64 / (1u64 << 31) as f64;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u2 = (state >> 33) as f64 / (1u64 << 31) as f64;

        let u1 = u1.max(1e-10); // avoid ln(0)
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        p + z * scale * p.abs().max(0.01) // scale relative to parameter magnitude
    }).collect()
}

// ═══════════════════════════════════════════════════════════════
// Deflated Sharpe Ratio (порт: statistical_gates.py:21-79)
// Bailey & Lopez de Prado (2014)
// ═══════════════════════════════════════════════════════════════

/// Deflated Sharpe Ratio — adjusts for number of trials tested.
/// Порт: statistical_gates.py:21-79
///
/// Returns (dsr, p_value).
/// DSR > 0 means strategy likely has real alpha after trial correction.
/// p_value < 0.05 means statistically significant.
pub fn deflated_sharpe_ratio(
    observed_sr: f64,
    num_trials: usize,
    num_obs: usize,
    skewness: f64,
    kurtosis: f64,
) -> (f64, f64) {
    if num_trials < 1 || num_obs < 2 {
        return (0.0, 1.0);
    }

    // Expected max SR under null (Gumbel approximation)
    // Порт: statistical_gates.py:48-61
    let sr_max = if num_trials <= 1 {
        0.0
    } else {
        let ln_m = (num_trials as f64).ln();
        let z = (2.0 * ln_m).sqrt();
        let e_z_max = z - (ln_m.ln() + (4.0 * std::f64::consts::PI).ln()) / (2.0 * z);
        e_z_max * (1.0 / (num_obs as f64 - 1.0)).sqrt()
    };

    // SE of Sharpe ratio (non-normality adjusted)
    // Порт: statistical_gates.py:64-71
    let sr = observed_sr;
    let se_sr = ((1.0 - skewness * sr + ((kurtosis - 1.0) / 4.0) * sr * sr)
        / (num_obs as f64 - 1.0))
        .max(1e-12)
        .sqrt();

    // DSR = (SR_observed - SR_max) / SE(SR)
    let dsr = (sr - sr_max) / se_sr;
    let p_value = 1.0 - norm_cdf(dsr);

    (round6(dsr), round6(p_value))
}

/// Minimum backtest length for target Sharpe at significance level.
/// Порт: statistical_gates.py:82-123 (binary search)
pub fn min_backtest_length(
    target_sr: f64,
    num_trials: usize,
    alpha: f64,
    skewness: f64,
    kurtosis: f64,
) -> usize {
    if target_sr <= 0.0 || num_trials < 1 {
        return 0;
    }

    let (mut lo, mut hi): (usize, usize) = (2, 100_000);

    let (_, p) = deflated_sharpe_ratio(target_sr, num_trials, hi, skewness, kurtosis);
    if p >= alpha { return hi; }

    while lo < hi {
        let mid = (lo + hi) / 2;
        let (_, p) = deflated_sharpe_ratio(target_sr, num_trials, mid, skewness, kurtosis);
        if p < alpha {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// Benjamini-Hochberg FDR correction for multiple testing.
/// Порт: statistical_gates.py:126-164
///
/// Returns Vec of (original_index, p_value, significant).
pub fn benjamini_hochberg(p_values: &[f64], alpha: f64) -> Vec<(usize, f64, bool)> {
    if p_values.is_empty() { return vec![]; }

    let m = p_values.len();

    // Sort by p-value, keep indices
    let mut indexed: Vec<(usize, f64)> = p_values.iter().cloned().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut significant = vec![false; m];
    let mut max_significant_rank: Option<usize> = None;

    for (rank, &(_, p)) in indexed.iter().enumerate() {
        let k = rank + 1; // 1-indexed
        let threshold = (k as f64 / m as f64) * alpha;
        if p <= threshold {
            max_significant_rank = Some(rank);
        }
    }

    if let Some(max_rank) = max_significant_rank {
        for item in indexed.iter().take(max_rank + 1) {
            significant[item.0] = true;
        }
    }

    (0..m).map(|i| (i, p_values[i], significant[i])).collect()
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Standard normal CDF. Порт: statistical_gates.py:167-169
fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Error function (Abramowitz and Stegun approximation).
fn erf(x: f64) -> f64 {
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741
        + t * (-1.453152027 + t * 1.061405429))));
    sign * (1.0 - poly * (-x * x).exp())
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hypothesis(id: u32, is_sharpe: f64, is_trades: usize) -> Hypothesis {
        Hypothesis {
            id,
            params: StrategyParams {
                name: format!("strat_{}", id),
                params: vec![0.1, 0.5],
                generation: 0,
                parent_id: None,
            },
            status: HypothesisStatus::Pending,
            fitness_is: Some(Fitness {
                sharpe_ratio: is_sharpe,
                profit_factor: 1.5,
                win_rate: 0.55,
                max_drawdown_pct: 10.0,
                trade_count: is_trades,
                total_pnl: 100.0,
            }),
            fitness_oos: None,
            elimination_reason: None,
        }
    }

    #[test]
    fn test_rank_filters_low_sharpe() {
        let config = SelectionConfig::default();
        let mut hyps = vec![
            make_hypothesis(1, 2.0, 20),
            make_hypothesis(2, -0.5, 20), // below min_is_sharpe
            make_hypothesis(3, 1.5, 20),
        ];
        let ranked = rank_by_is_fitness(&mut hyps, &config);
        assert_eq!(ranked.len(), 2, "Only 2 should pass IS filter");
        assert_eq!(ranked[0], 0, "Highest Sharpe first");
        assert_eq!(ranked[1], 2, "Second highest");
    }

    #[test]
    fn test_rank_filters_low_trade_count() {
        let config = SelectionConfig::default();
        let mut hyps = vec![
            make_hypothesis(1, 2.0, 5), // below min_is_trade_count (10)
            make_hypothesis(2, 1.0, 20),
        ];
        let ranked = rank_by_is_fitness(&mut hyps, &config);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0], 1);
    }

    #[test]
    fn test_validate_oos_pass() {
        let config = SelectionConfig::default();
        let mut h = make_hypothesis(1, 2.0, 50);
        h.fitness_oos = Some(Fitness {
            sharpe_ratio: 1.5,
            profit_factor: 1.5,
            win_rate: 0.55,
            max_drawdown_pct: 10.0,
            trade_count: 50,
            total_pnl: 200.0,
        });
        let (passed, _) = validate_oos(&h, &config);
        assert!(passed, "Should pass OOS with good metrics");
    }

    #[test]
    fn test_validate_oos_fail_sharpe() {
        let config = SelectionConfig::default();
        let mut h = make_hypothesis(1, 2.0, 50);
        h.fitness_oos = Some(Fitness {
            sharpe_ratio: 0.5, // below 1.0
            profit_factor: 1.5,
            win_rate: 0.55,
            max_drawdown_pct: 10.0,
            trade_count: 50,
            total_pnl: 50.0,
        });
        let (passed, reason) = validate_oos(&h, &config);
        assert!(!passed);
        assert!(reason.contains("Sharpe"), "Reason: {}", reason);
    }

    #[test]
    fn test_validate_oos_fail_drawdown() {
        let config = SelectionConfig::default();
        let mut h = make_hypothesis(1, 2.0, 50);
        h.fitness_oos = Some(Fitness {
            sharpe_ratio: 1.5,
            profit_factor: 1.5,
            win_rate: 0.55,
            max_drawdown_pct: 25.0, // above 20%
            trade_count: 50,
            total_pnl: 100.0,
        });
        let (passed, reason) = validate_oos(&h, &config);
        assert!(!passed);
        assert!(reason.contains("DD"), "Reason: {}", reason);
    }

    // Deflated Sharpe Ratio tests

    #[test]
    fn test_dsr_single_trial() {
        let (dsr, _p) = deflated_sharpe_ratio(2.0, 1, 252, 0.0, 3.0);
        // Single trial: no deflation, SR should be positive
        assert!(dsr > 0.0, "Single trial DSR should be positive, got {:.4}", dsr);
    }

    #[test]
    fn test_dsr_many_trials_deflates() {
        let (dsr_1, _) = deflated_sharpe_ratio(1.0, 1, 252, 0.0, 3.0);
        let (dsr_100, _) = deflated_sharpe_ratio(1.0, 100, 252, 0.0, 3.0);
        assert!(dsr_100 < dsr_1,
            "100 trials should deflate more: dsr_1={:.3}, dsr_100={:.3}", dsr_1, dsr_100);
    }

    #[test]
    fn test_dsr_insufficient_data() {
        let (dsr, p) = deflated_sharpe_ratio(2.0, 0, 10, 0.0, 3.0);
        assert_eq!(dsr, 0.0);
        assert_eq!(p, 1.0);
    }

    #[test]
    fn test_min_backtest_length() {
        let t = min_backtest_length(1.5, 10, 0.05, 0.0, 3.0);
        assert!(t >= 2, "Need at least 2 observations, got {}", t);
        assert!(t < 100_000, "Should not need insane amount, got {}", t);
    }

    #[test]
    fn test_bh_no_significant() {
        let p_vals = vec![0.3, 0.5, 0.8];
        let result = benjamini_hochberg(&p_vals, 0.05);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|(_, _, sig)| !sig), "None should be significant");
    }

    #[test]
    fn test_bh_some_significant() {
        let p_vals = vec![0.001, 0.01, 0.03, 0.5, 0.9];
        let result = benjamini_hochberg(&p_vals, 0.05);
        let sig_count = result.iter().filter(|(_, _, s)| *s).count();
        assert!(sig_count >= 2, "At least 2 should be significant, got {}", sig_count);
    }

    #[test]
    fn test_bh_empty() {
        let result = benjamini_hochberg(&[], 0.05);
        assert!(result.is_empty());
    }

    // Mutation tests

    #[test]
    fn test_mutate_params_deterministic() {
        let params = vec![1.0, 2.0, 3.0];
        let m1 = mutate_params(&params, 0.1, 42);
        let m2 = mutate_params(&params, 0.1, 42);
        assert_eq!(m1, m2, "Same seed should produce same mutations");
    }

    #[test]
    fn test_mutate_params_changes_values() {
        let params = vec![1.0, 2.0, 3.0];
        let mutated = mutate_params(&params, 0.5, 12345);
        assert_ne!(params, mutated, "Mutation should change values");
        // Values should be close to originals with small scale
        for (o, m) in params.iter().zip(mutated.iter()) {
            assert!((o - m).abs() < 5.0, "Mutation too large: {} -> {}", o, m);
        }
    }

    #[test]
    fn test_mutate_zero_scale() {
        let params = vec![1.0, 2.0, 3.0];
        let mutated = mutate_params(&params, 0.0, 42);
        assert_eq!(params, mutated, "Zero scale = no mutation");
    }

    // Helper tests

    #[test]
    fn test_norm_cdf_symmetry() {
        let c1 = norm_cdf(1.0);
        let c2 = norm_cdf(-1.0);
        assert!((c1 + c2 - 1.0).abs() < 1e-6, "CDF(-x) + CDF(x) = 1");
    }

    #[test]
    fn test_norm_cdf_zero() {
        assert!((norm_cdf(0.0) - 0.5).abs() < 1e-6, "CDF(0) = 0.5");
    }

    #[test]
    fn test_erf_bounds() {
        assert!(erf(0.0).abs() < 1e-6, "erf(0) ≈ 0, got {:.10}", erf(0.0));
        assert!(erf(3.0) > 0.999, "erf(3) ≈ 1");
        assert!(erf(-3.0) < -0.999, "erf(-3) ≈ -1");
    }
}
