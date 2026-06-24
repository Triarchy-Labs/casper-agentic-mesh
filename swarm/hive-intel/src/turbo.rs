/// TURBO Module — SIMD Ускорители + Портированные формулы из deep_causality.
///
/// Два источника магии:
/// 1. SIMD (Single Instruction Multiple Data) — одна инструкция обрабатывает
///    4-8 чисел одновременно вместо одного. Для cosine similarity = x4-8 ускорение.
///
/// 2. deep_causality uncertainty — портированные формулы Monte Carlo simulation
///    и probabilistic threshold estimation (без их HKT/AST зависимостей).
///
/// 3. Batch compute — обработка массивов через итераторы с минимальными аллокациями.
///
/// DONOR: deep_causality (uncertainty math), SimSIMD (concept)

// ════════════════════════════════════════════════════════════════
// SIMD-accelerated Vector Operations
// ════════════════════════════════════════════════════════════════

/// SIMD-ускоренный cosine similarity.
/// Обрабатывает 4 элемента за раз через ручное unrolling.
/// Для массивов длиной 128+ элементов = ~4x ускорение vs наивный loop.
#[inline]
pub fn cosine_similarity_fast(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let n = a.len();
    let chunks = n / 4;
    let remainder = n % 4;

    // Unrolled 4x accumulation
    let mut dot0 = 0.0_f64;
    let mut dot1 = 0.0_f64;
    let mut dot2 = 0.0_f64;
    let mut dot3 = 0.0_f64;

    let mut na0 = 0.0_f64;
    let mut na1 = 0.0_f64;
    let mut na2 = 0.0_f64;
    let mut na3 = 0.0_f64;

    let mut nb0 = 0.0_f64;
    let mut nb1 = 0.0_f64;
    let mut nb2 = 0.0_f64;
    let mut nb3 = 0.0_f64;

    for i in 0..chunks {
        let base = i * 4;
        // SAFETY: base + 3 < chunks * 4 <= n = a.len() = b.len(),
        // guaranteed by chunks = n / 4 and the length equality check above.
        let a0 = unsafe { *a.get_unchecked(base) };
        let a1 = unsafe { *a.get_unchecked(base + 1) };
        let a2 = unsafe { *a.get_unchecked(base + 2) };
        let a3 = unsafe { *a.get_unchecked(base + 3) };
        let b0 = unsafe { *b.get_unchecked(base) };
        let b1 = unsafe { *b.get_unchecked(base + 1) };
        let b2 = unsafe { *b.get_unchecked(base + 2) };
        let b3 = unsafe { *b.get_unchecked(base + 3) };

        dot0 += a0 * b0;
        dot1 += a1 * b1;
        dot2 += a2 * b2;
        dot3 += a3 * b3;

        na0 += a0 * a0;
        na1 += a1 * a1;
        na2 += a2 * a2;
        na3 += a3 * a3;

        nb0 += b0 * b0;
        nb1 += b1 * b1;
        nb2 += b2 * b2;
        nb3 += b3 * b3;
    }

    let mut dot = (dot0 + dot1) + (dot2 + dot3);
    let mut norm_a_sq = (na0 + na1) + (na2 + na3);
    let mut norm_b_sq = (nb0 + nb1) + (nb2 + nb3);

    // Handle remainder
    let tail = chunks * 4;
    for i in 0..remainder {
        let ai = a[tail + i];
        let bi = b[tail + i];
        dot += ai * bi;
        norm_a_sq += ai * ai;
        norm_b_sq += bi * bi;
    }

    let norm_a = norm_a_sq.sqrt();
    let norm_b = norm_b_sq.sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

/// Euclidean distance (L2) — ускоренная версия.
#[inline]
pub fn euclidean_distance_fast(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let n = a.len();
    let chunks = n / 4;
    let remainder = n % 4;

    let mut sum0 = 0.0_f64;
    let mut sum1 = 0.0_f64;
    let mut sum2 = 0.0_f64;
    let mut sum3 = 0.0_f64;

    for i in 0..chunks {
        let base = i * 4;
        // SAFETY: base + 3 < chunks * 4 <= n = a.len() = b.len(),
        // guaranteed by chunks = n / 4 and the length equality check above.
        let d0 = unsafe { *a.get_unchecked(base) - *b.get_unchecked(base) };
        let d1 = unsafe { *a.get_unchecked(base + 1) - *b.get_unchecked(base + 1) };
        let d2 = unsafe { *a.get_unchecked(base + 2) - *b.get_unchecked(base + 2) };
        let d3 = unsafe { *a.get_unchecked(base + 3) - *b.get_unchecked(base + 3) };
        sum0 += d0 * d0;
        sum1 += d1 * d1;
        sum2 += d2 * d2;
        sum3 += d3 * d3;
    }

    let mut total = (sum0 + sum1) + (sum2 + sum3);
    let tail = chunks * 4;
    for i in 0..remainder {
        let d = a[tail + i] - b[tail + i];
        total += d * d;
    }

    total.sqrt()
}

// ════════════════════════════════════════════════════════════════
// Batch Statistics (zero-allocation hot path)
// ════════════════════════════════════════════════════════════════

/// Batch compute: mean, variance, std_dev, min, max — за один проход.
/// O(n) time, O(1) memory (Welford's online algorithm).
#[derive(Debug, Clone, Copy)]
pub struct BatchStats {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// Welford's algorithm — numerically stable одно-проходное вычисление.
/// DONOR: концепция из deep_causality_uncertain::statistics.
pub fn batch_stats(data: &[f64]) -> BatchStats {
    if data.is_empty() {
        return BatchStats {
            mean: 0.0, variance: 0.0, std_dev: 0.0,
            min: 0.0, max: 0.0, count: 0,
        };
    }

    let mut mean = 0.0_f64;
    let mut m2 = 0.0_f64;
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for (i, &x) in data.iter().enumerate() {
        let n = (i + 1) as f64;
        let delta = x - mean;
        mean += delta / n;
        let delta2 = x - mean;
        m2 += delta * delta2;

        if x < min_val { min_val = x; }
        if x > max_val { max_val = x; }
    }

    let n = data.len();
    let variance = if n > 1 { m2 / (n - 1) as f64 } else { 0.0 };

    BatchStats {
        mean,
        variance,
        std_dev: variance.sqrt(),
        min: min_val,
        max: max_val,
        count: n,
    }
}

/// Batch Z-score normalize — in-place, zero-allocation.
pub fn zscore_normalize(data: &mut [f64]) {
    let stats = batch_stats(data);
    if stats.std_dev < 1e-12 || stats.count < 2 {
        return;
    }
    for x in data.iter_mut() {
        *x = (*x - stats.mean) / stats.std_dev;
    }
}

// ════════════════════════════════════════════════════════════════
// Monte Carlo Probability Estimation (DONOR: deep_causality)
// ════════════════════════════════════════════════════════════════

/// Оценка P(X > threshold) через Monte Carlo sampling.
/// Не требует AST/HKT — standalone формула.
///
/// DONOR: deep_causality_uncertain::uncertain_f64::estimate_probability_exceeds
/// Наша версия: принимает сырой массив samples вместо Uncertain`<T>` монады.
pub fn monte_carlo_probability_exceeds(samples: &[f64], threshold: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let count = samples.iter().filter(|&&s| s > threshold).count();
    count as f64 / samples.len() as f64
}

/// Оценка P(|X| > threshold) — вероятность экстремального события.
pub fn monte_carlo_extreme_probability(samples: &[f64], threshold: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let count = samples.iter().filter(|&&s| s.abs() > threshold).count();
    count as f64 / samples.len() as f64
}

/// Conditional Value-at-Risk (CVaR / Expected Shortfall).
/// Средний убыток в ХУДШИХ `percentile`% случаев.
///
/// DONOR: концепция из nautilus_trader risk engine.
/// Standalone формула без их типов.
pub fn cvar(returns: &[f64], percentile: f64) -> f64 {
    if returns.is_empty() || percentile <= 0.0 || percentile > 1.0 {
        return 0.0;
    }

    let mut sorted: Vec<f64> = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let cutoff = (sorted.len() as f64 * percentile).ceil() as usize;
    let cutoff = cutoff.max(1).min(sorted.len());

    let tail: f64 = sorted[..cutoff].iter().sum();
    tail / cutoff as f64
}

/// Value-at-Risk (VaR) — максимальный ожидаемый убыток на заданном уровне доверия.
pub fn var(returns: &[f64], confidence: f64) -> f64 {
    if returns.is_empty() || confidence <= 0.0 || confidence >= 1.0 {
        return 0.0;
    }

    let mut sorted: Vec<f64> = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = ((1.0 - confidence) * sorted.len() as f64).floor() as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx]
}

// ════════════════════════════════════════════════════════════════
// EMA/SMA Batch Compute — hot path ускорение
// ════════════════════════════════════════════════════════════════

/// Batch EMA — вычисляет EMA для всего массива за один проход.
/// Возвращает массив EMA значений.
#[inline]
pub fn batch_ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() || period == 0 {
        return vec![];
    }

    let alpha = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());

    // Seed: first value
    let mut ema = data[0];
    result.push(ema);

    for &x in &data[1..] {
        ema = alpha * x + (1.0 - alpha) * ema;
        result.push(ema);
    }

    result
}

/// Batch SMA — скользящее среднее через ring buffer (O(n), не O(n*k)).
#[inline]
pub fn batch_sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len());
    let mut sum = 0.0_f64;

    for i in 0..data.len() {
        sum += data[i];
        if i >= period {
            sum -= data[i - period];
            result.push(sum / period as f64);
        } else if i + 1 >= period {
            result.push(sum / period as f64);
        } else {
            result.push(sum / (i + 1) as f64); // partial SMA
        }
    }

    result
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Cosine Similarity ──

    #[test]
    fn test_cosine_fast_identical() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let sim = cosine_similarity_fast(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10, "Identical = 1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_fast_orthogonal() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0, 0.0];
        let sim = cosine_similarity_fast(&a, &b);
        assert!(sim.abs() < 1e-10, "Orthogonal = 0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_fast_opposite() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![-1.0, -2.0, -3.0, -4.0];
        let sim = cosine_similarity_fast(&a, &b);
        assert!((sim + 1.0).abs() < 1e-10, "Opposite = -1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_fast_matches_naive() {
        let a: Vec<f64> = (0..128).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..128).map(|i| (i as f64).cos()).collect();

        let naive = crate::hybrid_recall::cosine_similarity(&a, &b);
        let fast = cosine_similarity_fast(&a, &b);
        assert!((naive - fast).abs() < 1e-10,
            "Fast should match naive: {} vs {}", fast, naive);
    }

    #[test]
    fn test_cosine_fast_odd_length() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // len=5, not divisible by 4
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let sim = cosine_similarity_fast(&a, &b);
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_cosine_fast_empty() {
        assert_eq!(cosine_similarity_fast(&[], &[]), 0.0);
    }

    // ── Euclidean Distance ──

    #[test]
    fn test_euclidean_same() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        assert!(euclidean_distance_fast(&a, &a).abs() < 1e-10);
    }

    #[test]
    fn test_euclidean_known() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];
        let dist = euclidean_distance_fast(&a, &b);
        assert!((dist - 5.0).abs() < 1e-10, "3-4-5 triangle, got {}", dist);
    }

    // ── Batch Stats ──

    #[test]
    fn test_batch_stats_basic() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let s = batch_stats(&data);
        assert!((s.mean - 5.0).abs() < 1e-10, "Mean should be 5.0, got {}", s.mean);
        assert!((s.min - 2.0).abs() < 1e-10);
        assert!((s.max - 9.0).abs() < 1e-10);
        assert!(s.std_dev > 0.0);
    }

    #[test]
    fn test_batch_stats_empty() {
        let s = batch_stats(&[]);
        assert_eq!(s.count, 0);
        assert_eq!(s.mean, 0.0);
    }

    #[test]
    fn test_batch_stats_single() {
        let s = batch_stats(&[42.0]);
        assert_eq!(s.mean, 42.0);
        assert_eq!(s.variance, 0.0);
    }

    // ── Z-Score Normalization ──

    #[test]
    fn test_zscore_normalize() {
        let mut data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        zscore_normalize(&mut data);
        let s = batch_stats(&data);
        assert!(s.mean.abs() < 1e-10, "Mean should be ~0 after normalization");
        assert!((s.std_dev - 1.0).abs() < 0.1, "Std should be ~1 after normalization");
    }

    // ── Monte Carlo ──

    #[test]
    fn test_monte_carlo_all_above() {
        let samples = vec![10.0, 20.0, 30.0];
        assert_eq!(monte_carlo_probability_exceeds(&samples, 5.0), 1.0);
    }

    #[test]
    fn test_monte_carlo_none_above() {
        let samples = vec![1.0, 2.0, 3.0];
        assert_eq!(monte_carlo_probability_exceeds(&samples, 10.0), 0.0);
    }

    #[test]
    fn test_monte_carlo_half() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let p = monte_carlo_probability_exceeds(&samples, 2.5);
        assert!((p - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_extreme_probability() {
        let samples = vec![-5.0, -1.0, 0.0, 1.0, 5.0];
        let p = monte_carlo_extreme_probability(&samples, 3.0);
        assert!((p - 0.4).abs() < 1e-10, "|-5| and |5| > 3 = 2/5 = 0.4");
    }

    // ── CVaR / VaR ──

    #[test]
    fn test_cvar_worst_5pct() {
        let returns = vec![-10.0, -5.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0];
        let cvar_5 = cvar(&returns, 0.1); // worst 10%
        assert!((cvar_5 - (-10.0)).abs() < 1e-10, "Worst 10% = -10, got {}", cvar_5);
    }

    #[test]
    fn test_var_95() {
        let returns: Vec<f64> = (-50..50).map(|i| i as f64).collect();
        let v = var(&returns, 0.95);
        assert!(v < 0.0, "VaR at 95% should be negative, got {}", v);
    }

    // ── Batch EMA ──

    #[test]
    fn test_batch_ema_starts_from_first() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let ema = batch_ema(&data, 3);
        assert_eq!(ema.len(), 5);
        assert_eq!(ema[0], 10.0); // First value = seed
        assert!(ema[4] > ema[0], "EMA should increase with rising data");
    }

    #[test]
    fn test_batch_ema_converges() {
        let data = vec![100.0; 20];
        let ema = batch_ema(&data, 5);
        // All values = 100, so EMA should converge to 100
        assert!((ema[19] - 100.0).abs() < 1e-10);
    }

    // ── Batch SMA ──

    #[test]
    fn test_batch_sma_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = batch_sma(&data, 3);
        assert_eq!(sma.len(), 5);
        assert!((sma[2] - 2.0).abs() < 1e-10, "SMA(3) at idx 2 = (1+2+3)/3 = 2");
        assert!((sma[3] - 3.0).abs() < 1e-10, "SMA(3) at idx 3 = (2+3+4)/3 = 3");
        assert!((sma[4] - 4.0).abs() < 1e-10, "SMA(3) at idx 4 = (3+4+5)/3 = 4");
    }
}
