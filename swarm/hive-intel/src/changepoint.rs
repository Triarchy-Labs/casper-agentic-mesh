/// Bayesian Online Changepoint Detection (Adams & MacKay 2007).
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/owm/changepoint.py (422 строки)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Детектирует СМЕНУ РЕЖИМА в поведении торгового агента.
/// Использует conjugate models:
///   - Beta-Bernoulli для win/loss потока
///   - Normal-Inverse-Gamma для continuous signals (PnL, hold time)
///
/// В отличие от нашего drift.rs (простой CUSUM), это ПОЛНЫЙ Bayesian inference
/// с run-length posterior в log-space.
///
/// CUSUM остаётся как ДОПОЛНИТЕЛЬНЫЙ детектор для gradual shifts.
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Result
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct ChangepointResult {
    pub changepoint_probability: f64, // P(r_t = 0 | x_1:t)
    pub max_run_length: usize,        // argmax of run length posterior
    pub observation_count: u32,
    pub cusum_alert: bool,            // gradual drift detected
    pub cusum_value: f64,
    pub won_posterior_mean: f64,       // P(win) estimate
}

// ═══════════════════════════════════════════════════════════════
// Conjugate model helpers (порт: changepoint.py:32-87)
// ═══════════════════════════════════════════════════════════════

/// Log predictive of Bernoulli observation under Beta prior.
/// P(x=1|α,β) = α/(α+β), P(x=0|α,β) = β/(α+β)
fn beta_bernoulli_logpred(x: f64, alpha: f64, beta: f64) -> f64 {
    if x > 0.5 {
        (alpha / (alpha + beta)).ln()
    } else {
        (beta / (alpha + beta)).ln()
    }
}

/// Update Beta posterior with Bernoulli observation.
fn beta_bernoulli_update(x: f64, alpha: f64, beta: f64) -> (f64, f64) {
    if x > 0.5 {
        (alpha + 1.0, beta)
    } else {
        (alpha, beta + 1.0)
    }
}

/// Log predictive under Normal-Inverse-Gamma prior (Student-t).
/// Порт: changepoint.py:50-75
fn nig_logpred(x: f64, mu: f64, kappa: f64, alpha: f64, beta: f64) -> f64 {
    let nu = 2.0 * alpha;
    let scale_sq = beta * (kappa + 1.0) / (alpha * kappa);
    if scale_sq <= 0.0 { return -50.0; }

    let z = x - mu;
    let numer = z * z / (nu * scale_sq);

    // Student-t log pdf
    lgamma((nu + 1.0) / 2.0)
        - lgamma(nu / 2.0)
        - 0.5 * (nu * std::f64::consts::PI * scale_sq).ln()
        - ((nu + 1.0) / 2.0) * (1.0 + numer).ln()
}

/// Update Normal-Inverse-Gamma posterior.
/// Порт: changepoint.py:78-86
fn nig_update(x: f64, mu: f64, kappa: f64, alpha: f64, beta: f64) -> (f64, f64, f64, f64) {
    let new_kappa = kappa + 1.0;
    let new_mu = (kappa * mu + x) / new_kappa;
    let new_alpha = alpha + 0.5;
    let new_beta = beta + 0.5 * kappa * (x - mu).powi(2) / new_kappa;
    (new_mu, new_kappa, new_alpha, new_beta)
}

/// Numerically stable log-sum-exp.
/// Порт: changepoint.py:413-421
fn logsumexp(log_values: &[f64]) -> f64 {
    if log_values.is_empty() { return f64::NEG_INFINITY; }
    let max_val = log_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if max_val.is_infinite() { return f64::NEG_INFINITY; }
    let total: f64 = log_values.iter().map(|v| (v - max_val).exp()).sum();
    max_val + total.ln()
}

/// Log-gamma function (Stirling approximation — no external deps).
fn lgamma(x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    // Stirling's approximation: ln(Γ(x)) ≈ 0.5*ln(2π/x) + x*(ln(x + 1/(12x - 1/10x)) - 1)
    0.5 * (2.0 * std::f64::consts::PI / x).ln()
        + x * ((x + 1.0 / (12.0 * x - 1.0 / (10.0 * x))).ln() - 1.0)
}

// ═══════════════════════════════════════════════════════════════
// Signal trackers (порт: changepoint.py:93-179)
// ═══════════════════════════════════════════════════════════════

/// Beta-Bernoulli signal (win/loss tracking).
#[derive(Debug, Clone)]
struct BetaBernoulliSignal {
    prior: (f64, f64),
    params: Vec<(f64, f64)>, // per run-length (α, β)
}

impl BetaBernoulliSignal {
    fn new(alpha: f64, beta: f64) -> Self {
        Self {
            prior: (alpha, beta),
            params: vec![(alpha, beta)],
        }
    }

    fn logpred(&self, x: f64) -> Vec<f64> {
        self.params.iter()
            .map(|(a, b)| beta_bernoulli_logpred(x, *a, *b))
            .collect()
    }

    fn update(&mut self, x: f64) {
        self.params = self.params.iter()
            .map(|(a, b)| beta_bernoulli_update(x, *a, *b))
            .collect();
        self.params.insert(0, self.prior); // new run: reset to prior
    }

    fn posterior_mean(&self) -> f64 {
        let (a, b) = self.params.last().unwrap_or(&self.prior);
        a / (a + b)
    }
}

/// Normal-Inverse-Gamma signal (continuous value tracking).
#[derive(Debug, Clone)]
struct NigSignal {
    prior: (f64, f64, f64, f64), // (μ, κ, α, β)
    params: Vec<(f64, f64, f64, f64)>,
}

impl NigSignal {
    fn new(mu: f64, kappa: f64, alpha: f64, beta: f64) -> Self {
        let prior = (mu, kappa, alpha, beta);
        Self {
            prior,
            params: vec![prior],
        }
    }

    fn logpred(&self, x: f64) -> Vec<f64> {
        self.params.iter()
            .map(|(mu, k, a, b)| nig_logpred(x, *mu, *k, *a, *b))
            .collect()
    }

    fn update(&mut self, x: f64) {
        self.params = self.params.iter()
            .map(|(mu, k, a, b)| nig_update(x, *mu, *k, *a, *b))
            .collect();
        self.params.insert(0, self.prior);
    }
}

// ═══════════════════════════════════════════════════════════════
// Main Detector (порт: changepoint.py:185-410)
// ═══════════════════════════════════════════════════════════════

/// Bayesian Online Changepoint Detector.
pub struct BayesianChangepoint {
    #[allow(dead_code)] // Kept for debug inspection
    hazard_lambda: f64,
    truncation_threshold: f64,
    log_h: f64,
    log_1_minus_h: f64,
    log_run_probs: Vec<f64>,

    // Signals
    won_signal: BetaBernoulliSignal,
    pnl_signal: NigSignal,

    observation_count: u32,

    // CUSUM complementary detector
    cusum_s: f64,
    cusum_target_wr: f64,
    cusum_threshold: f64,
    cusum_wins: u32,
    cusum_total: u32,
}

impl BayesianChangepoint {
    /// hazard_lambda: Expected run length between changepoints.
    /// Higher = changepoints less frequent. Default 50.
    pub fn new(hazard_lambda: f64) -> Self {
        Self {
            hazard_lambda,
            truncation_threshold: 1e-6,
            log_h: (1.0 / hazard_lambda).ln(),
            log_1_minus_h: (1.0 - 1.0 / hazard_lambda).ln(),
            log_run_probs: vec![0.0], // log(1.0) = 0.0

            won_signal: BetaBernoulliSignal::new(1.0, 1.0),
            pnl_signal: NigSignal::new(0.0, 1.0, 1.0, 1.0),

            observation_count: 0,

            cusum_s: 0.0,
            cusum_target_wr: 0.5,
            cusum_threshold: 4.0,
            cusum_wins: 0,
            cusum_total: 0,
        }
    }

    /// Process one trade observation.
    /// Порт: changepoint.py:231-359
    pub fn update(&mut self, won: bool, pnl_r: f64) -> ChangepointResult {
        let won_val = if won { 1.0 } else { 0.0 };
        let n = self.log_run_probs.len();

        // Joint log predictive across signals (порт: changepoint.py:246-280)
        let won_lp = self.won_signal.logpred(won_val);
        let pnl_lp = self.pnl_signal.logpred(pnl_r);

        let mut log_pred = vec![0.0; n];
        let mut log_prior_pred = 0.0;

        for i in 0..n {
            log_pred[i] += won_lp[i] + pnl_lp[i];
        }
        log_prior_pred += beta_bernoulli_logpred(won_val, 1.0, 1.0);
        log_prior_pred += nig_logpred(pnl_r, 0.0, 1.0, 1.0, 1.0);

        // Run length update (порт: changepoint.py:283-300)
        // Growth: P(r=r'+1) ∝ P(x|params_r) × (1-h) × P(r')
        let mut new_log_probs: Vec<f64> = (0..n)
            .map(|i| self.log_run_probs[i] + log_pred[i] + self.log_1_minus_h)
            .collect();

        // Changepoint: P(r=0) ∝ π_prior(x) × h × Σ P(r')
        let log_sum_prev = logsumexp(&self.log_run_probs);
        let log_cp = log_prior_pred + self.log_h + log_sum_prev;
        new_log_probs.insert(0, log_cp);

        // Normalize
        let log_total = logsumexp(&new_log_probs);
        new_log_probs.iter_mut().for_each(|lp| *lp -= log_total);

        // Truncate small run lengths (порт: changepoint.py:302-310)
        let keep: Vec<usize> = new_log_probs.iter().enumerate()
            .filter(|(_, lp)| lp.exp() >= self.truncation_threshold)
            .map(|(i, _)| i)
            .collect();

        let keep = if keep.is_empty() { vec![0] } else { keep };

        self.log_run_probs = keep.iter().map(|&i| new_log_probs[i]).collect();

        // Update signals
        self.won_signal.update(won_val);
        self.pnl_signal.update(pnl_r);

        // Truncate signal params to match
        self.won_signal.params = keep.iter()
            .filter_map(|&i| self.won_signal.params.get(i).cloned())
            .collect();
        self.pnl_signal.params = keep.iter()
            .filter_map(|&i| self.pnl_signal.params.get(i).cloned())
            .collect();

        if self.won_signal.params.is_empty() {
            self.won_signal.params.push(self.won_signal.prior);
        }
        if self.pnl_signal.params.is_empty() {
            self.pnl_signal.params.push(self.pnl_signal.prior);
        }

        self.observation_count += 1;

        // Changepoint probability = P(r=0)
        let cp_prob = self.log_run_probs[0].exp();

        // Max run length
        let max_idx = self.log_run_probs.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // CUSUM (порт: changepoint.py:338-347)
        let cusum_x = if won { 1.0 } else { 0.0 };
        self.cusum_total += 1;
        self.cusum_wins += if won { 1 } else { 0 };
        if self.cusum_total >= 20 {
            self.cusum_target_wr = self.cusum_wins as f64 / self.cusum_total as f64;
        }
        self.cusum_s = (self.cusum_s + (self.cusum_target_wr - cusum_x)).max(0.0);
        let cusum_alert = self.cusum_s > self.cusum_threshold;

        ChangepointResult {
            changepoint_probability: cp_prob,
            max_run_length: max_idx,
            observation_count: self.observation_count,
            cusum_alert,
            cusum_value: self.cusum_s,
            won_posterior_mean: self.won_signal.posterior_mean(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let det = BayesianChangepoint::new(50.0);
        assert_eq!(det.observation_count, 0);
        assert_eq!(det.log_run_probs.len(), 1);
    }

    #[test]
    fn test_single_observation() {
        let mut det = BayesianChangepoint::new(50.0);
        let result = det.update(true, 0.5);
        assert_eq!(result.observation_count, 1);
        assert!(result.changepoint_probability >= 0.0 && result.changepoint_probability <= 1.0);
    }

    #[test]
    fn test_stable_regime_low_cp_prob() {
        let mut det = BayesianChangepoint::new(50.0);
        // 20 consistent wins → stable regime
        for _ in 0..20 {
            det.update(true, 1.0);
        }
        let result = det.update(true, 1.0);
        assert!(result.changepoint_probability < 0.1,
            "Stable regime should have low CP prob, got {:.3}", result.changepoint_probability);
        assert!(result.max_run_length > 5, "Run length should be long");
    }

    #[test]
    fn test_regime_change_detection() {
        let mut det = BayesianChangepoint::new(50.0);
        // 30 wins → stable
        for _ in 0..30 {
            det.update(true, 1.0);
        }
        // Sudden shift to losses
        for _ in 0..5 {
            det.update(false, -2.0);
        }
        let result = det.update(false, -2.0);
        // CP probability should increase after regime change
        assert!(result.changepoint_probability > 0.001,
            "Regime change should increase CP prob, got {:.4}", result.changepoint_probability);
    }

    #[test]
    fn test_cusum_alert() {
        let mut det = BayesianChangepoint::new(50.0);
        // Warm up: 20 mixed trades
        for i in 0..20 {
            det.update(i % 2 == 0, if i % 2 == 0 { 1.0 } else { -0.5 });
        }
        // Now pure losses → CUSUM should accumulate
        for _ in 0..10 {
            det.update(false, -1.0);
        }
        let result = det.update(false, -1.0);
        assert!(result.cusum_alert, "Long loss streak should trigger CUSUM alert");
    }

    #[test]
    fn test_won_posterior_updates() {
        let mut det = BayesianChangepoint::new(50.0);
        for _ in 0..50 {
            det.update(true, 0.5);
        }
        let result = det.update(true, 0.5);
        assert!(result.won_posterior_mean > 0.8,
            "50 wins should give high posterior mean, got {:.3}", result.won_posterior_mean);
    }

    #[test]
    fn test_logsumexp_basic() {
        let vals = vec![0.0_f64.ln(), 0.0_f64.ln()]; // ln(0) = -inf
        // Test with valid values
        let vals2 = vec![1.0_f64.ln(), 2.0_f64.ln()]; // ln(1), ln(2)
        let result = logsumexp(&vals2);
        let expected = (1.0 + 2.0_f64).ln();
        assert!((result - expected).abs() < 1e-10,
            "logsumexp([ln1, ln2]) = ln(3), got {:.4} vs {:.4}", result, expected);
    }

    #[test]
    fn test_nig_update_precision() {
        let (mu, kappa, alpha, beta) = nig_update(2.0, 0.0, 1.0, 1.0, 1.0);
        assert_eq!(kappa, 2.0, "kappa should be 1+1=2");
        assert!((mu - 1.0).abs() < 1e-10, "mu should be (1*0+2)/2=1.0");
        assert!((alpha - 1.5).abs() < 1e-10, "alpha should be 1+0.5=1.5");
    }

    #[test]
    fn test_empty_logsumexp() {
        assert!(logsumexp(&[]).is_infinite());
    }
}
