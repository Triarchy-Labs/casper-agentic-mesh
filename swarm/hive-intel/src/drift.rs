/// Drift Detection — обнаружение изменений в поведении и рынке.
///
/// 1. CUSUM: детектирует сдвиг win-rate (стратегия сломалась?)
/// 2. Disposition Effect: режем прибыль рано, держим убыток долго?
///
/// Портировано из: tradememory-protocol/src/tradememory/owm/drift.py
use serde::Serialize;

// ═══════════════════════════════════════════════════════════
// CUSUM Drift Detector
// ═══════════════════════════════════════════════════════════

/// Результат CUSUM детекции.
#[derive(Debug, Clone, Serialize)]
pub struct CusumResult {
    pub drift_detected: bool,
    pub drift_point: Option<usize>,
    pub final_cusum: f64,
}

/// One-sided CUSUM для обнаружения сдвига win rate.
///
/// S_i = max(0, S_{i-1} + (x_i - target_wr))
///
/// - `pnl_values`: список PnL (>0 = win, ≤0 = loss)
/// - `target_wr`: ожидаемый win rate (0.5 по умолчанию)
/// - `threshold`: порог CUSUM для срабатывания (4.0 по умолчанию)
pub fn cusum_drift(pnl_values: &[f64], target_wr: f64, threshold: f64) -> CusumResult {
    debug_assert!(threshold > 0.0);
    debug_assert!((0.0..=1.0).contains(&target_wr));

    let mut s = 0.0_f64;
    let mut drift_point = None;

    for (i, &pnl) in pnl_values.iter().enumerate() {
        let x = if pnl > 0.0 { 1.0 } else { 0.0 };
        s = (s + (x - target_wr)).max(0.0);
        if drift_point.is_none() && s > threshold {
            drift_point = Some(i);
        }
    }

    CusumResult {
        drift_detected: drift_point.is_some(),
        drift_point,
        final_cusum: s,
    }
}

/// CUSUM вниз — обнаружение ДЕГРАДАЦИИ win rate.
pub fn cusum_degradation(pnl_values: &[f64], target_wr: f64, threshold: f64) -> CusumResult {
    debug_assert!(threshold > 0.0);

    let mut s = 0.0_f64;
    let mut drift_point = None;

    for (i, &pnl) in pnl_values.iter().enumerate() {
        let x = if pnl > 0.0 { 1.0 } else { 0.0 };
        s = (s + (target_wr - x)).max(0.0); // Инвертировано!
        if drift_point.is_none() && s > threshold {
            drift_point = Some(i);
        }
    }

    CusumResult {
        drift_detected: drift_point.is_some(),
        drift_point,
        final_cusum: s,
    }
}

// ═══════════════════════════════════════════════════════════
// Disposition Effect Detector
// ═══════════════════════════════════════════════════════════

/// Результат анализа disposition effect.
#[derive(Debug, Clone, Serialize)]
pub struct DispositionAnalysis {
    pub avg_win_hold_ms: f64,
    pub avg_loss_hold_ms: f64,
    pub ratio: f64,                  // win_hold / loss_hold
    pub disposition_detected: bool,  // ratio < 0.7 = режем прибыль рано
    pub severity: String,            // "none", "mild", "severe"
}

/// Анализ disposition effect: сравнивает время удержания winners vs losers.
///
/// Если avg_win_hold << avg_loss_hold → классический disposition effect:
/// - Режем winners рано (страх потерять прибыль)
/// - Держим losers долго (надежда на разворот)
///
/// - `trades`: slice of (pnl, hold_duration_ms)
pub fn detect_disposition(trades: &[(f64, i64)]) -> DispositionAnalysis {
    let wins: Vec<_> = trades.iter().filter(|(pnl, _)| *pnl > 0.0).collect();
    let losses: Vec<_> = trades.iter().filter(|(pnl, _)| *pnl < 0.0).collect();

    if wins.is_empty() || losses.is_empty() {
        return DispositionAnalysis {
            avg_win_hold_ms: 0.0,
            avg_loss_hold_ms: 0.0,
            ratio: 1.0,
            disposition_detected: false,
            severity: "none".to_string(),
        };
    }

    let avg_win: f64 = wins.iter().map(|(_, h)| *h as f64).sum::<f64>() / wins.len() as f64;
    let avg_loss: f64 = losses.iter().map(|(_, h)| *h as f64).sum::<f64>() / losses.len() as f64;

    let ratio = if avg_loss > 0.0 { avg_win / avg_loss } else { 1.0 };

    let (detected, severity) = if ratio < 0.5 {
        (true, "severe")
    } else if ratio < 0.7 {
        (true, "mild")
    } else {
        (false, "none")
    };

    DispositionAnalysis {
        avg_win_hold_ms: avg_win,
        avg_loss_hold_ms: avg_loss,
        ratio,
        disposition_detected: detected,
        severity: severity.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cusum_no_drift() {
        let balanced = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let result = cusum_drift(&balanced, 0.5, 4.0);
        assert!(!result.drift_detected);
    }

    #[test]
    fn test_cusum_detects_improvement() {
        let improving = vec![-1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let result = cusum_drift(&improving, 0.5, 4.0);
        assert!(result.drift_detected, "Should detect upward drift");
    }

    #[test]
    fn test_cusum_degradation() {
        let degrading = vec![1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0];
        let result = cusum_degradation(&degrading, 0.5, 4.0);
        assert!(result.drift_detected, "Should detect degradation");
    }

    #[test]
    fn test_disposition_normal() {
        let trades = vec![
            (10.0, 60000_i64),   // Win held 60s
            (-5.0, 30000),       // Loss held 30s  
            (15.0, 50000),       // Win held 50s
            (-8.0, 25000),       // Loss held 25s
        ];
        let result = detect_disposition(&trades);
        assert!(!result.disposition_detected, "Normal: winners held LONGER");
        assert!(result.ratio > 1.0);
    }

    #[test]
    fn test_disposition_severe() {
        let trades = vec![
            (10.0, 10000_i64),   // Win cut at 10s (early!)
            (-5.0, 60000),       // Loss held 60s (hoping!)
            (15.0, 8000),        // Win cut at 8s
            (-8.0, 55000),       // Loss held 55s
        ];
        let result = detect_disposition(&trades);
        assert!(result.disposition_detected, "Should detect disposition effect");
        assert_eq!(result.severity, "severe");
    }
}
