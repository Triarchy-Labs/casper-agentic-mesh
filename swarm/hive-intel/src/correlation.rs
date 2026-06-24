/// Cross-Asset Correlation Matrix — портфельный risk awareness.
///
/// Не путать с causal.rs (причинность A→B)!
/// Корреляция = "A и B двигаются ОДНОВРЕМЕННО" (направление + сила).
///
/// Pearson correlation: r ∈ [-1, +1]
/// - r > 0.7:  сильная положительная → удвоение risk при обеих позициях
/// - r < -0.7: сильная отрицательная → хедж
/// - |r| < 0.3: слабая → независимые
///
/// Rolling correlation: пересчитывается на окне последних N трейдов.
use serde::Serialize;
use std::collections::HashMap;

/// Корреляционная пара.
#[derive(Debug, Clone, Serialize)]
pub struct CorrelationPair {
    pub symbol_a: String,
    pub symbol_b: String,
    pub pearson_r: f64,
    pub sample_size: usize,
    pub strength: CorrelationStrength,
}

/// Сила корреляции.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum CorrelationStrength {
    StrongPositive,  // r > 0.7
    ModeratePositive, // 0.3 < r <= 0.7
    Weak,            // -0.3 <= r <= 0.3
    ModerateNegative, // -0.7 <= r < -0.3
    StrongNegative,  // r < -0.7
}

impl CorrelationStrength {
    pub fn from_r(r: f64) -> Self {
        match r {
            r if r > 0.7 => Self::StrongPositive,
            r if r > 0.3 => Self::ModeratePositive,
            r if r >= -0.3 => Self::Weak,
            r if r >= -0.7 => Self::ModerateNegative,
            _ => Self::StrongNegative,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::StrongPositive => "strong_positive",
            Self::ModeratePositive => "moderate_positive",
            Self::Weak => "weak",
            Self::ModerateNegative => "moderate_negative",
            Self::StrongNegative => "strong_negative",
        }
    }
}

/// Pearson correlation coefficient между двумя сериями.
/// r = Σ((xi - x̄)(yi - ȳ)) / √(Σ(xi - x̄)² × Σ(yi - ȳ)²)
pub fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 3 { return None; }

    let x = &x[..n];
    let y = &y[..n];

    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0_f64;
    let mut var_x = 0.0_f64;
    let mut var_y = 0.0_f64;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-10 { return None; } // Одна серия константная

    Some((cov / denom).clamp(-1.0, 1.0))
}

/// Матрица корреляций между всеми символами.
///
/// `pnl_series` — HashMap`<symbol, Vec<pnl>>` (временной ряд PnL).
/// Возвращает все пары с |r| > min_abs_r.
pub fn build_correlation_matrix(
    pnl_series: &HashMap<String, Vec<f64>>,
    min_abs_r: f64,
) -> Vec<CorrelationPair> {
    let symbols: Vec<&String> = pnl_series.keys().collect();
    let mut pairs = Vec::new();

    for i in 0..symbols.len() {
        for j in (i + 1)..symbols.len() {
            let x = &pnl_series[symbols[i]];
            let y = &pnl_series[symbols[j]];

            if let Some(r) = pearson_correlation(x, y) {
                if r.abs() >= min_abs_r {
                    pairs.push(CorrelationPair {
                        symbol_a: symbols[i].clone(),
                        symbol_b: symbols[j].clone(),
                        pearson_r: r,
                        sample_size: x.len().min(y.len()),
                        strength: CorrelationStrength::from_r(r),
                    });
                }
            }
        }
    }

    // Сортировать по абсолютной корреляции (самые сильные сверху)
    pairs.sort_by(|a, b| {
        b.pearson_r.abs().partial_cmp(&a.pearson_r.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    pairs
}

/// Проверяет risk exposure: сколько сильно-коррелированных активов в одном направлении.
pub fn exposure_warning(
    pairs: &[CorrelationPair],
    active_positions: &HashMap<String, f64>, // symbol → size (+ long, - short)
) -> Vec<String> {
    let mut warnings = Vec::new();

    for pair in pairs {
        if let (Some(&size_a), Some(&size_b)) = (
            active_positions.get(&pair.symbol_a),
            active_positions.get(&pair.symbol_b),
        ) {
            // Оба в одном направлении + сильная положительная корреляция
            let same_direction = (size_a > 0.0 && size_b > 0.0) || (size_a < 0.0 && size_b < 0.0);
            if same_direction && pair.pearson_r > 0.7 {
                warnings.push(format!(
                    "⚠️ CORRELATED EXPOSURE: {} + {} (r={:.2}) both {} — DOUBLE RISK!",
                    pair.symbol_a, pair.symbol_b, pair.pearson_r,
                    if size_a > 0.0 { "LONG" } else { "SHORT" }
                ));
            }
            // Противоположные + сильная отрицательная → хедж (хорошо)
            let opposite = (size_a > 0.0 && size_b < 0.0) || (size_a < 0.0 && size_b > 0.0);
            if opposite && pair.pearson_r < -0.7 {
                warnings.push(format!(
                    "⚠️ ANTI-HEDGE: {} and {} (r={:.2}) opposite direction BUT negatively correlated — SAME exposure!",
                    pair.symbol_a, pair.symbol_b, pair.pearson_r
                ));
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_positive_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - 1.0).abs() < 1e-10, "Perfect positive should be 1.0, got {}", r);
    }

    #[test]
    fn test_perfect_negative_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!((r - (-1.0)).abs() < 1e-10, "Perfect negative should be -1.0, got {}", r);
    }

    #[test]
    fn test_no_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![5.0, 2.0, 8.0, 1.0, 7.0, 3.0, 6.0, 4.0];
        let r = pearson_correlation(&x, &y).unwrap();
        assert!(r.abs() < 0.5, "Random data should have low |r|, got {}", r);
    }

    #[test]
    fn test_insufficient_data() {
        assert!(pearson_correlation(&[1.0], &[2.0]).is_none());
        assert!(pearson_correlation(&[1.0, 2.0], &[3.0, 4.0]).is_none());
    }

    #[test]
    fn test_constant_series() {
        let x = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(pearson_correlation(&x, &y).is_none(), "Constant series → None");
    }

    #[test]
    fn test_correlation_matrix_finds_pairs() {
        let mut series = HashMap::new();
        series.insert("BTC".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        series.insert("ETH".to_string(), vec![1.5, 3.0, 4.5, 6.0, 7.5]); // r ≈ 1.0
        series.insert("DOGE".to_string(), vec![5.0, 2.0, 8.0, 1.0, 7.0]); // uncorrelated

        let pairs = build_correlation_matrix(&series, 0.5);
        assert!(!pairs.is_empty(), "Should find BTC-ETH pair");
        assert_eq!(pairs[0].symbol_a.as_str().min(pairs[0].symbol_b.as_str()), "BTC");
    }

    #[test]
    fn test_exposure_warning() {
        let pairs = vec![CorrelationPair {
            symbol_a: "BTC".to_string(),
            symbol_b: "ETH".to_string(),
            pearson_r: 0.9,
            sample_size: 20,
            strength: CorrelationStrength::StrongPositive,
        }];
        let mut positions = HashMap::new();
        positions.insert("BTC".to_string(), 100.0); // Long
        positions.insert("ETH".to_string(), 50.0);  // Long

        let warnings = exposure_warning(&pairs, &positions);
        assert!(!warnings.is_empty(), "Same direction + high r = warning");
        assert!(warnings[0].contains("DOUBLE RISK"));
    }

    #[test]
    fn test_strength_classification() {
        assert_eq!(CorrelationStrength::from_r(0.9), CorrelationStrength::StrongPositive);
        assert_eq!(CorrelationStrength::from_r(0.5), CorrelationStrength::ModeratePositive);
        assert_eq!(CorrelationStrength::from_r(0.0), CorrelationStrength::Weak);
        assert_eq!(CorrelationStrength::from_r(-0.5), CorrelationStrength::ModerateNegative);
        assert_eq!(CorrelationStrength::from_r(-0.9), CorrelationStrength::StrongNegative);
    }
}
