/// Auto-Induction: promote episodic memories to semantic when pattern count ≥ threshold.
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/owm/induction.py (57 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Когда агент накапливает ≥10 эпизодических воспоминаний одного паттерна,
/// система автоматически "кристаллизует" их в семантическое знание:
///   - win_rate, avg_pnl_r, directions, strategies
///
/// Это механизм обучения: единичные трейды → устойчивые правила.
use serde::Serialize;
use std::collections::HashMap;

/// Semantic memory created by auto-induction.
#[derive(Debug, Clone, Serialize)]
pub struct InducedMemory {
    pub pattern_name: String,
    pub sample_size: usize,
    pub win_rate: f64,
    pub avg_pnl_r: f64,
    pub directions: Vec<String>,
    pub strategies: Vec<String>,
}

/// Episodic memory input for induction check.
pub struct EpisodicMemory {
    pub pattern_name: String,
    pub pnl_r: Option<f64>,
    pub direction: Option<String>,
    pub strategy: Option<String>,
}

/// Check if any pattern has accumulated enough episodes to induce semantic memory.
///
/// Порт: induction.py:7-56 (check_auto_induction)
///
/// Groups episodic memories by pattern_name. When a group reaches the threshold,
/// produces an InducedMemory summarizing the pattern.
pub fn check_auto_induction(
    episodes: &[EpisodicMemory],
    threshold: usize,
) -> Vec<InducedMemory> {
    assert!(threshold >= 1, "threshold must be >= 1");

    // Group by pattern_name
    let mut groups: HashMap<&str, Vec<&EpisodicMemory>> = HashMap::new();
    for ep in episodes {
        groups.entry(&ep.pattern_name).or_default().push(ep);
    }

    let mut results = Vec::new();

    for (pattern, memories) in &groups {
        if memories.len() < threshold {
            continue;
        }

        let pnl_rs: Vec<f64> = memories
            .iter()
            .filter_map(|m| m.pnl_r)
            .collect();

        let wins = pnl_rs.iter().filter(|&&p| p > 0.0).count();
        let win_rate = if pnl_rs.is_empty() { 0.0 } else { wins as f64 / pnl_rs.len() as f64 };
        let avg_pnl_r = if pnl_rs.is_empty() { 0.0 } else { pnl_rs.iter().sum::<f64>() / pnl_rs.len() as f64 };

        let mut directions: Vec<String> = memories
            .iter()
            .filter_map(|m| m.direction.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        directions.sort();

        let mut strategies: Vec<String> = memories
            .iter()
            .filter_map(|m| m.strategy.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        strategies.sort();

        results.push(InducedMemory {
            pattern_name: pattern.to_string(),
            sample_size: memories.len(),
            win_rate: (win_rate * 10000.0).round() / 10000.0,
            avg_pnl_r: (avg_pnl_r * 10000.0).round() / 10000.0,
            directions,
            strategies,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ep(pattern: &str, pnl: f64) -> EpisodicMemory {
        EpisodicMemory {
            pattern_name: pattern.to_string(),
            pnl_r: Some(pnl),
            direction: Some("LONG".to_string()),
            strategy: Some("momentum".to_string()),
        }
    }

    #[test]
    fn test_below_threshold() {
        let eps: Vec<EpisodicMemory> = (0..5).map(|_| make_ep("bullish_engulf", 1.0)).collect();
        let result = check_auto_induction(&eps, 10);
        assert!(result.is_empty(), "Below threshold → no induction");
    }

    #[test]
    fn test_at_threshold() {
        let eps: Vec<EpisodicMemory> = (0..10).map(|i| {
            make_ep("bullish_engulf", if i < 7 { 1.5 } else { -0.5 })
        }).collect();
        let result = check_auto_induction(&eps, 10);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern_name, "bullish_engulf");
        assert_eq!(result[0].sample_size, 10);
        assert!((result[0].win_rate - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_multiple_patterns() {
        let mut eps = Vec::new();
        for _ in 0..12 { eps.push(make_ep("pin_bar", 2.0)); }
        for _ in 0..8 { eps.push(make_ep("doji", -0.5)); }
        for _ in 0..15 { eps.push(make_ep("engulfing", 1.0)); }

        let result = check_auto_induction(&eps, 10);
        assert_eq!(result.len(), 2, "pin_bar (12) and engulfing (15) cross threshold, doji (8) doesn't");
    }

    #[test]
    fn test_no_pnl_data() {
        let eps: Vec<EpisodicMemory> = (0..10).map(|_| EpisodicMemory {
            pattern_name: "test".to_string(),
            pnl_r: None,
            direction: None,
            strategy: None,
        }).collect();
        let result = check_auto_induction(&eps, 10);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].win_rate, 0.0);
        assert_eq!(result[0].avg_pnl_r, 0.0);
    }

    #[test]
    #[should_panic(expected = "threshold must be >= 1")]
    fn test_zero_threshold_panics() {
        check_auto_induction(&[], 0);
    }
}
