/// Strategy Validator — Institutional statistical validation engine.
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/strategy_validator.py (1038 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Three-layer verification:
///   1. CPCV (Combinatorial Purged Cross-Validation) — de Prado methodology
///   2. Walk-Forward Validation — IS/OOS sliding windows
///   3. Raw Sharpe + Max Drawdown utilities
///
/// Все формулы портированы formula-in-formula из Python оригинала.
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Statistical Utilities (порт: strategy_validator.py:943-1037)
// ═══════════════════════════════════════════════════════════════

/// Raw Sharpe ratio (NOT annualized).
/// Порт: strategy_validator.py:943-950
pub fn raw_sharpe(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = variance.sqrt();
    if std > 0.0 { mean / std } else { 0.0 }
}

/// Sample standard deviation.
/// Порт: strategy_validator.py:953-959
pub fn sample_std(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    variance.sqrt()
}

/// Max drawdown from PnL series. Returns (absolute, percent).
/// Порт: strategy_validator.py:962-982
pub fn max_drawdown(daily_values: &[f64]) -> (f64, f64) {
    if daily_values.is_empty() {
        return (0.0, 0.0);
    }
    let mut running = 0.0;
    let mut peak = 0.0;
    let mut max_dd = 0.0;

    for &v in daily_values {
        running += v;
        if running > peak {
            peak = running;
        }
        let dd = peak - running;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    let max_dd_pct = if peak > 0.0 { max_dd / peak * 100.0 } else { 0.0 };
    (max_dd, max_dd_pct)
}

/// Approximate standard normal CDF (Abramowitz & Stegun).
/// Порт: strategy_validator.py:1026-1037
pub fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 { return 0.0; }
    if x > 8.0 { return 1.0; }
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.398_942_280_401_432_7; // 1/sqrt(2*pi)
    let p = d * (-x * x / 2.0).exp()
        * (t * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937
                    + t * (-1.821_255_978 + t * 1.330_274_429)))));
    if x > 0.0 { 1.0 - p } else { p }
}

/// Find contiguous blocks in sorted index list.
/// Порт: strategy_validator.py:1010-1023
fn find_contiguous_blocks(sorted_indices: &[usize]) -> Vec<(usize, usize)> {
    if sorted_indices.is_empty() {
        return vec![];
    }
    let mut blocks = Vec::new();
    let mut block_start = sorted_indices[0];
    let mut prev = sorted_indices[0];

    for &i in &sorted_indices[1..] {
        if i > prev + 1 {
            blocks.push((block_start, prev));
            block_start = i;
        }
        prev = i;
    }
    blocks.push((block_start, prev));
    blocks
}

// ═══════════════════════════════════════════════════════════════
// CPCV — Combinatorial Purged Cross-Validation
// Порт: strategy_validator.py:643-737
// ═══════════════════════════════════════════════════════════════

/// CPCV result.
#[derive(Debug, Clone, Serialize)]
pub struct CpcvResult {
    pub verdict: ValidationVerdict,
    pub n_folds: usize,
    pub n_groups: usize,
    pub n_test_groups: usize,
    pub purge_window: usize,
    pub embargo_window: usize,
    pub mean_sharpe: f64,
    pub std_sharpe: f64,
    pub min_sharpe: f64,
    pub max_sharpe: f64,
    pub consistency: f64,   // fraction of folds with positive Sharpe
    pub positive_folds: usize,
    pub error: Option<String>,
}

/// CPCV on daily returns — cross-validated Sharpe distribution.
///
/// Unlike ML-based CPCV, this computes Sharpe on each OOS fold directly,
/// measuring how stable the strategy's edge is across time periods.
///
/// Порт: strategy_validator.py:643-737 (cpcv_sharpe)
///
/// * `daily_returns` — daily return values
/// * `n_groups` — sequential groups (N), default 10
/// * `n_test_groups` — groups per test set (k), folds = C(N,k)
/// * `purge_window` — bars removed at train/test boundary
/// * `embargo_window` — additional bars skipped after test block
pub fn cpcv_sharpe(
    daily_returns: &[f64],
    n_groups: usize,
    n_test_groups: usize,
    purge_window: usize,
    embargo_window: usize,
) -> CpcvResult {
    let n = daily_returns.len();
    let min_required = n_groups * (purge_window + embargo_window + 5);

    if n < min_required {
        return CpcvResult {
            verdict: ValidationVerdict::InsufficientData,
            n_folds: 0,
            n_groups,
            n_test_groups,
            purge_window,
            embargo_window,
            mean_sharpe: 0.0,
            std_sharpe: 0.0,
            min_sharpe: 0.0,
            max_sharpe: 0.0,
            consistency: 0.0,
            positive_folds: 0,
            error: Some(format!("Need >= {min_required} observations, got {n}")),
        };
    }

    let group_size = n / n_groups;

    // Build groups: Vec of index ranges
    let groups: Vec<Vec<usize>> = (0..n_groups)
        .map(|i| {
            let start = i * group_size;
            let end = if i < n_groups - 1 { (i + 1) * group_size } else { n };
            (start..end).collect()
        })
        .collect();

    // Generate all C(n_groups, n_test_groups) combinations
    let combos = combinations(n_groups, n_test_groups);

    let mut fold_sharpes = Vec::with_capacity(combos.len());

    for test_group_ids in &combos {
        // Collect test indices
        let mut test_indices: Vec<usize> = Vec::new();
        for &gid in test_group_ids {
            test_indices.extend_from_slice(&groups[gid]);
        }
        test_indices.sort_unstable();

        // Find contiguous blocks for purge/embargo
        let blocks = find_contiguous_blocks(&test_indices);

        // Build exclude set
        let mut exclude = std::collections::HashSet::new();
        for &(block_start, block_end) in &blocks {
            // Purge before block
            let purge_start = block_start.saturating_sub(purge_window);
            for i in purge_start..block_start {
                exclude.insert(i);
            }
            // Embargo after block
            for i in (block_end + 1)..((block_end + 1 + embargo_window).min(n)) {
                exclude.insert(i);
            }
        }

        // Test returns (OOS, excluding purged/embargoed)
        let test_returns: Vec<f64> = test_indices
            .iter()
            .filter(|&&i| !exclude.contains(&i))
            .map(|&i| daily_returns[i])
            .collect();

        let sharpe = if test_returns.is_empty() { 0.0 } else { raw_sharpe(&test_returns) };
        fold_sharpes.push(sharpe);
    }

    if fold_sharpes.is_empty() {
        return CpcvResult {
            verdict: ValidationVerdict::Fail,
            n_folds: 0,
            n_groups,
            n_test_groups,
            purge_window,
            embargo_window,
            mean_sharpe: 0.0,
            std_sharpe: 0.0,
            min_sharpe: 0.0,
            max_sharpe: 0.0,
            consistency: 0.0,
            positive_folds: 0,
            error: Some("No valid folds".to_string()),
        };
    }

    let mean = fold_sharpes.iter().sum::<f64>() / fold_sharpes.len() as f64;
    let std = sample_std(&fold_sharpes);
    let min = fold_sharpes.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = fold_sharpes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let positive = fold_sharpes.iter().filter(|&&s| s > 0.0).count();
    let consistency = positive as f64 / fold_sharpes.len() as f64;

    // Verdict: порт strategy_validator.py:717-722
    let verdict = if consistency >= 0.70 && mean > 0.0 {
        ValidationVerdict::Pass
    } else if consistency >= 0.55 && mean > 0.0 {
        ValidationVerdict::Caution
    } else {
        ValidationVerdict::Fail
    };

    CpcvResult {
        verdict,
        n_folds: fold_sharpes.len(),
        n_groups,
        n_test_groups,
        purge_window,
        embargo_window,
        mean_sharpe: mean,
        std_sharpe: std,
        min_sharpe: min,
        max_sharpe: max,
        consistency,
        positive_folds: positive,
        error: None,
    }
}

/// Generate all C(n, k) combinations of indices.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    let mut combo = vec![0usize; k];
    combinations_recursive(n, k, 0, 0, &mut combo, &mut result);
    result
}

fn combinations_recursive(
    n: usize,
    k: usize,
    start: usize,
    depth: usize,
    combo: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if depth == k {
        result.push(combo.clone());
        return;
    }
    for i in start..=(n - k + depth) {
        combo[depth] = i;
        combinations_recursive(n, k, i + 1, depth + 1, combo, result);
    }
}

// ═══════════════════════════════════════════════════════════════
// Walk-Forward Validation
// Порт: strategy_validator.py:404-547
// ═══════════════════════════════════════════════════════════════

/// Walk-Forward window result.
#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardWindow {
    pub label: String,
    pub is_sharpe: f64,
    pub oos_sharpe: f64,
    pub oos_win_rate: f64,
    pub is_count: usize,
    pub oos_count: usize,
    pub verdict: ValidationVerdict,
}

/// Walk-Forward validation result.
#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardResult {
    pub windows: Vec<WalkForwardWindow>,
    pub windows_passed: usize,
    pub windows_total: usize,
    pub verdict: ValidationVerdict,
}

/// Walk-Forward on daily returns with year-based windows.
///
/// Порт: strategy_validator.py:482-547 (walk_forward_returns)
///
/// * `returns` — (year, daily_return) pairs
/// * `is_years` — in-sample window (default 3)
/// * `oos_years` — out-of-sample window (default 1)
pub fn walk_forward_returns(
    returns: &[(u16, f64)],  // (year, return)
    is_years: u16,
    oos_years: u16,
) -> WalkForwardResult {
    if returns.is_empty() {
        return WalkForwardResult {
            windows: vec![],
            windows_passed: 0,
            windows_total: 0,
            verdict: ValidationVerdict::InsufficientData,
        };
    }

    let first_year = returns.first().unwrap().0;
    let last_year = returns.last().unwrap().0;

    let mut windows = Vec::new();
    let mut is_start = first_year;

    while is_start + is_years + oos_years - 1 <= last_year {
        let is_end = is_start + is_years - 1;
        let oos_year = is_end + 1;

        let is_vals: Vec<f64> = returns.iter()
            .filter(|(y, _)| *y >= is_start && *y <= is_end)
            .map(|(_, r)| *r)
            .collect();

        let oos_vals: Vec<f64> = returns.iter()
            .filter(|(y, _)| *y == oos_year)
            .map(|(_, r)| *r)
            .collect();

        let is_sharpe = raw_sharpe(&is_vals);
        let oos_sharpe = raw_sharpe(&oos_vals);
        let oos_wr = if oos_vals.is_empty() {
            0.0
        } else {
            oos_vals.iter().filter(|&&v| v > 0.0).count() as f64 / oos_vals.len() as f64 * 100.0
        };

        // Verdict per window: порт strategy_validator.py:511-518
        let verdict = if oos_vals.is_empty() {
            ValidationVerdict::InsufficientData
        } else if oos_sharpe > 0.05 {
            ValidationVerdict::Pass
        } else if oos_sharpe > -0.02 {
            ValidationVerdict::Caution
        } else {
            ValidationVerdict::Fail
        };

        windows.push(WalkForwardWindow {
            label: format!("{is_start}-{is_end} -> {oos_year}"),
            is_sharpe,
            oos_sharpe,
            oos_win_rate: oos_wr,
            is_count: is_vals.len(),
            oos_count: oos_vals.len(),
            verdict,
        });

        is_start += oos_years;
    }

    let passed = windows.iter().filter(|w| w.verdict == ValidationVerdict::Pass).count();
    let total = windows.iter().filter(|w| w.verdict != ValidationVerdict::InsufficientData).count();

    // Overall: порт strategy_validator.py:533-540
    let overall = if total == 0 {
        ValidationVerdict::InsufficientData
    } else if passed as f64 / total as f64 >= 0.67 {
        ValidationVerdict::Pass
    } else if passed as f64 / total as f64 >= 0.50 {
        ValidationVerdict::Caution
    } else {
        ValidationVerdict::Fail
    };

    WalkForwardResult {
        windows,
        windows_passed: passed,
        windows_total: total,
        verdict: overall,
    }
}

// ═══════════════════════════════════════════════════════════════
// Unified Verdict
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ValidationVerdict {
    Pass,
    Caution,
    Fail,
    InsufficientData,
}

impl ValidationVerdict {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pass => "PASS",
            Self::Caution => "CAUTION",
            Self::Fail => "FAIL",
            Self::InsufficientData => "INSUFFICIENT_DATA",
        }
    }
}

/// Full validation pipeline result.
#[derive(Debug, Clone, Serialize)]
pub struct FullValidationResult {
    pub cpcv: CpcvResult,
    pub walk_forward: WalkForwardResult,
    pub raw_sharpe: f64,
    pub max_drawdown_abs: f64,
    pub max_drawdown_pct: f64,
    pub verdict: ValidationVerdict,
}

/// Run full validation pipeline on daily returns.
/// Порт: strategy_validator.py:813-875 (validate_from_returns)
pub fn validate_returns(
    daily_returns: &[f64],
    yearly_returns: &[(u16, f64)],
) -> FullValidationResult {
    let sharpe = raw_sharpe(daily_returns);
    let (dd_abs, dd_pct) = max_drawdown(daily_returns);
    let cpcv = cpcv_sharpe(daily_returns, 10, 2, 5, 10);
    let wf = walk_forward_returns(yearly_returns, 3, 1);

    // Combine verdicts: порт strategy_validator.py:849-861
    let verdicts = [cpcv.verdict, wf.verdict];
    let pass_count = verdicts.iter().filter(|&&v| v == ValidationVerdict::Pass).count();
    let fail_count = verdicts.iter().filter(|&&v| v == ValidationVerdict::Fail).count();

    let overall = if fail_count >= 2 {
        ValidationVerdict::Fail
    } else if pass_count >= 2 && fail_count == 0 {
        ValidationVerdict::Pass
    } else {
        ValidationVerdict::Caution
    };

    FullValidationResult {
        cpcv,
        walk_forward: wf,
        raw_sharpe: sharpe,
        max_drawdown_abs: dd_abs,
        max_drawdown_pct: dd_pct,
        verdict: overall,
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // --- Utility tests ---

    #[test]
    fn test_raw_sharpe_positive() {
        let vals = vec![0.01, 0.02, 0.015, 0.005, 0.01, 0.02, 0.01];
        let s = raw_sharpe(&vals);
        assert!(s > 0.0, "Positive returns → positive Sharpe: {}", s);
    }

    #[test]
    fn test_raw_sharpe_negative() {
        let vals = vec![-0.01, -0.02, -0.015, -0.005, -0.01];
        let s = raw_sharpe(&vals);
        assert!(s < 0.0, "Negative returns → negative Sharpe: {}", s);
    }

    #[test]
    fn test_raw_sharpe_insufficient() {
        assert_eq!(raw_sharpe(&[0.01]), 0.0);
        assert_eq!(raw_sharpe(&[]), 0.0);
    }

    #[test]
    fn test_max_drawdown() {
        let vals = vec![10.0, 5.0, -20.0, 3.0, 8.0];
        let (dd_abs, _dd_pct) = max_drawdown(&vals);
        assert!(dd_abs > 0.0, "Should have drawdown: {}", dd_abs);
    }

    #[test]
    fn test_max_drawdown_empty() {
        let (a, p) = max_drawdown(&[]);
        assert_eq!(a, 0.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_normal_cdf_symmetry() {
        let a = normal_cdf(1.0);
        let b = normal_cdf(-1.0);
        assert!((a + b - 1.0).abs() < 0.001, "CDF should be symmetric: {} + {} ≠ 1", a, b);
    }

    #[test]
    fn test_normal_cdf_zero() {
        let v = normal_cdf(0.0);
        assert!((v - 0.5).abs() < 0.001, "CDF(0) should be ~0.5: {}", v);
    }

    // --- Combinations tests ---

    #[test]
    fn test_combinations_c5_2() {
        let c = combinations(5, 2);
        assert_eq!(c.len(), 10, "C(5,2) = 10, got {}", c.len());
    }

    #[test]
    fn test_combinations_c4_1() {
        let c = combinations(4, 1);
        assert_eq!(c.len(), 4, "C(4,1) = 4");
    }

    #[test]
    fn test_combinations_c6_3() {
        let c = combinations(6, 3);
        assert_eq!(c.len(), 20, "C(6,3) = 20, got {}", c.len());
    }

    // --- Contiguous blocks ---

    #[test]
    fn test_contiguous_blocks() {
        let indices = vec![0, 1, 2, 5, 6, 10, 11, 12];
        let blocks = find_contiguous_blocks(&indices);
        assert_eq!(blocks, vec![(0, 2), (5, 6), (10, 12)]);
    }

    #[test]
    fn test_contiguous_blocks_single() {
        let indices = vec![3];
        let blocks = find_contiguous_blocks(&indices);
        assert_eq!(blocks, vec![(3, 3)]);
    }

    // --- CPCV tests ---

    #[test]
    fn test_cpcv_insufficient_data() {
        let returns = vec![0.01; 10]; // Too few
        let result = cpcv_sharpe(&returns, 10, 2, 5, 10);
        assert_eq!(result.verdict, ValidationVerdict::InsufficientData);
    }

    #[test]
    fn test_cpcv_consistent_positive() {
        // 500 days of consistently positive returns
        let returns: Vec<f64> = (0..500)
            .map(|i| 0.001 + 0.0005 * ((i as f64 * 0.1).sin()))
            .collect();
        let result = cpcv_sharpe(&returns, 5, 2, 3, 5);
        assert!(result.n_folds > 0, "Should have folds");
        assert!(result.consistency > 0.5, "Consistent positive → high consistency: {}", result.consistency);
    }

    #[test]
    fn test_cpcv_negative_returns() {
        let returns: Vec<f64> = (0..500)
            .map(|i| -0.001 - 0.0005 * ((i as f64 * 0.1).sin()))
            .collect();
        let result = cpcv_sharpe(&returns, 5, 2, 3, 5);
        assert!(result.mean_sharpe < 0.0, "Negative returns → negative mean Sharpe");
        assert_eq!(result.verdict, ValidationVerdict::Fail);
    }

    // --- Walk-Forward tests ---

    #[test]
    fn test_walk_forward_empty() {
        let result = walk_forward_returns(&[], 3, 1);
        assert_eq!(result.verdict, ValidationVerdict::InsufficientData);
    }

    #[test]
    fn test_walk_forward_basic() {
        // 5 years of returns (2020-2024)
        let mut returns = Vec::new();
        for year in 2020..=2024u16 {
            for _ in 0..252 { // ~252 trading days
                returns.push((year, 0.001));
            }
        }
        let result = walk_forward_returns(&returns, 3, 1);
        assert!(!result.windows.is_empty(), "Should have windows");
    }

    #[test]
    fn test_walk_forward_insufficient_years() {
        let returns: Vec<(u16, f64)> = (0..100).map(|_| (2024, 0.001)).collect();
        let result = walk_forward_returns(&returns, 3, 1);
        assert!(result.windows.is_empty(), "Single year → no windows");
    }

    // --- Full validation tests ---

    #[test]
    fn test_full_validation() {
        let daily: Vec<f64> = (0..500).map(|i| 0.001 + 0.0005 * ((i as f64 * 0.1).sin())).collect();
        let yearly: Vec<(u16, f64)> = (0..500)
            .map(|i| (2020 + (i / 252) as u16, daily[i]))
            .collect();
        let result = validate_returns(&daily, &yearly);
        assert!(result.raw_sharpe > 0.0);
    }

    #[test]
    fn test_sample_std_basic() {
        let vals = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let s = sample_std(&vals);
        assert!((s - 2.138).abs() < 0.01, "Expected ~2.138, got {}", s);
    }
}
