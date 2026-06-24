/// Anomaly Detection — статистическое обнаружение аномальных трейдов.
///
/// Не просто "видели ли мы это?" (bloom) →
/// "НАСКОЛЬКО аномален этот трейд по сравнению с историей?"
///
/// Методы:
/// - Z-Score: отклонение от среднего в стандартных отклонениях
/// - IQR (Interquartile Range): robust к выбросам
/// - Modified Z-Score (MAD): ещё более robust
///
/// Если трейд = -50$ при avg_loss = -5$ → это 10σ → КРИК.
use serde::Serialize;

/// Результат анализа аномалии.
#[derive(Debug, Clone, Serialize)]
pub struct AnomalyResult {
    pub value: f64,
    pub z_score: f64,
    pub iqr_anomaly: bool,
    pub severity: AnomalySeverity,
    pub percentile: f64,
}

/// Уровень аномалии.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum AnomalySeverity {
    /// Нормальный трейд (< 2σ)
    Normal,
    /// Необычный (2-3σ)
    Unusual,
    /// Аномальный (3-4σ)
    Anomalous,
    /// Экстремальный (> 4σ) — чёрный лебедь
    Extreme,
}

impl AnomalySeverity {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Normal => "normal",
            Self::Unusual => "unusual",
            Self::Anomalous => "anomalous",
            Self::Extreme => "extreme",
        }
    }
}

/// Z-Score: (x - μ) / σ
fn z_score(value: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev < 1e-10 {
        return 0.0; // Нет вариации → всё "нормально"
    }
    (value - mean) / std_dev
}

/// Среднее арифметическое.
/// Делегирует в turbo::batch_stats (Welford, один проход).
#[allow(dead_code)] // Utility: available for future ML/risk modules
pub(crate) fn mean(data: &[f64]) -> f64 {
    crate::turbo::batch_stats(data).mean
}

/// Стандартное отклонение (population).
/// Делегирует в turbo::batch_stats (Welford, один проход, numerically stable).
#[allow(dead_code)] // Utility: available for future ML/risk modules
pub(crate) fn std_dev(data: &[f64]) -> f64 {
    crate::turbo::batch_stats(data).std_dev
}

/// Медиана (для IQR и MAD).
#[allow(dead_code)] // Utility: available for future ML/risk modules
pub(crate) fn median(data: &mut [f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = data.len() / 2;
    if data.len().is_multiple_of(2) {
        (data[mid - 1] + data[mid]) / 2.0
    } else {
        data[mid]
    }
}

/// Percentile rank: какой % значений меньше данного.
fn percentile_rank(data: &[f64], value: f64) -> f64 {
    if data.is_empty() { return 50.0; }
    let below = data.iter().filter(|&&x| x < value).count();
    (below as f64 / data.len() as f64) * 100.0
}

/// IQR-based anomaly detection (robust).
fn iqr_is_anomaly(data: &mut [f64], value: f64) -> bool {
    if data.len() < 4 { return false; }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1_idx = data.len() / 4;
    let q3_idx = 3 * data.len() / 4;
    let q1 = data[q1_idx];
    let q3 = data[q3_idx];
    let iqr = q3 - q1;
    let lower = q1 - 1.5 * iqr;
    let upper = q3 + 1.5 * iqr;
    value < lower || value > upper
}

/// Главная функция: проанализировать аномальность значения.
///
/// `value` — текущее значение (PnL трейда).
/// `history` — исторические значения для сравнения.
pub fn detect_anomaly(value: f64, history: &[f64]) -> AnomalyResult {
    if history.len() < 3 {
        return AnomalyResult {
            value,
            z_score: 0.0,
            iqr_anomaly: false,
            severity: AnomalySeverity::Normal,
            percentile: 50.0,
        };
    }

    // ГИПЕРЗВУК: один проход Welford вместо двух (mean + std_dev)
    let stats = crate::turbo::batch_stats(history);
    let z = z_score(value, stats.mean, stats.std_dev);
    let pct = percentile_rank(history, value);
    
    let mut hist_copy: Vec<f64> = history.to_vec();
    let iqr_flag = iqr_is_anomaly(&mut hist_copy, value);

    let severity = match z.abs() {
        z if z >= 4.0 => AnomalySeverity::Extreme,
        z if z >= 3.0 => AnomalySeverity::Anomalous,
        z if z >= 2.0 => AnomalySeverity::Unusual,
        _ => AnomalySeverity::Normal,
    };

    AnomalyResult {
        value,
        z_score: z,
        iqr_anomaly: iqr_flag,
        severity,
        percentile: pct,
    }
}

/// Пакетный анализ: найти все аномалии в серии.
pub fn find_anomalies(data: &[f64], min_severity: AnomalySeverity) -> Vec<(usize, AnomalyResult)> {
    if data.len() < 5 { return vec![]; }
    
    let mut results = Vec::new();
    // Скользящее окно: каждое значение сравниваем со ВСЕМИ остальными
    for i in 0..data.len() {
        let history: Vec<f64> = data.iter().enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, &v)| v)
            .collect();
        let result = detect_anomaly(data[i], &history);
        
        let dominated = matches!((min_severity, result.severity),
            (AnomalySeverity::Normal, _)
            | (AnomalySeverity::Unusual, AnomalySeverity::Unusual | AnomalySeverity::Anomalous | AnomalySeverity::Extreme)
            | (AnomalySeverity::Anomalous, AnomalySeverity::Anomalous | AnomalySeverity::Extreme)
            | (AnomalySeverity::Extreme, AnomalySeverity::Extreme)
        );
        
        if dominated {
            results.push((i, result));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_trade() {
        let history = vec![5.0, -3.0, 7.0, -2.0, 4.0, -1.0, 6.0, -4.0, 3.0, -2.0];
        let result = detect_anomaly(4.0, &history);
        assert_eq!(result.severity, AnomalySeverity::Normal);
    }

    #[test]
    fn test_extreme_loss() {
        let history = vec![5.0, -3.0, 7.0, -2.0, 4.0, -1.0, 6.0, -4.0, 3.0, -2.0];
        let result = detect_anomaly(-50.0, &history);
        assert!(result.z_score < -3.0, "Extreme loss should have z < -3, got {}", result.z_score);
        assert!(matches!(result.severity, AnomalySeverity::Anomalous | AnomalySeverity::Extreme));
    }

    #[test]
    fn test_extreme_win() {
        let history = vec![1.0, 2.0, -1.0, 3.0, -2.0, 1.5, -0.5, 2.5, -1.5, 0.5];
        let result = detect_anomaly(100.0, &history);
        assert!(result.z_score > 3.0, "Extreme win z should be > 3");
        assert!(matches!(result.severity, AnomalySeverity::Anomalous | AnomalySeverity::Extreme));
    }

    #[test]
    fn test_iqr_flags_outlier() {
        let history = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = detect_anomaly(50.0, &history);
        assert!(result.iqr_anomaly, "50 should be IQR outlier in [1..8]");
    }

    #[test]
    fn test_percentile_calculation() {
        let history = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = detect_anomaly(5.0, &history);
        assert!((result.percentile - 40.0).abs() < 1.0, "5 should be ~40th percentile, got {}", result.percentile);
    }

    #[test]
    fn test_insufficient_history() {
        let result = detect_anomaly(100.0, &[1.0, 2.0]);
        assert_eq!(result.severity, AnomalySeverity::Normal, "< 3 data points = always normal");
    }

    #[test]
    fn test_find_anomalies_batch() {
        let data = vec![1.0, 2.0, -1.0, 3.0, -50.0, 2.0, 1.0, -1.5, 100.0, 0.5];
        let anomalies = find_anomalies(&data, AnomalySeverity::Unusual);
        assert!(!anomalies.is_empty(), "Should find at least 1 anomaly");
        // -50 and 100 should be flagged
        let indices: Vec<usize> = anomalies.iter().map(|(i, _)| *i).collect();
        assert!(indices.contains(&4) || indices.contains(&8), "Should flag -50 or 100");
    }

    #[test]
    fn test_z_score_zero_variance() {
        let history = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let result = detect_anomaly(5.0, &history);
        assert_eq!(result.z_score, 0.0, "Zero variance should give z=0");
    }
}
