/// Power Law Decay — когнитивная модель забывания.
///
/// Основано на: Wixted & Ebbesen (1991) + ACT-R cognitive architecture.
/// Адаптировано из tradememory-protocol OWM Framework.
///
/// Power Law вместо экспоненты: после 90 дней экспонента оставляет 5% силы,
/// Power Law оставляет ~42% — старые трейды в том же режиме остаются полезными.
/// Сила эпизодической памяти (для отдельных трейдов).
///
/// `S(t) = (1 + t/τ)^(-d) × (1 + 0.3 × ln(1 + n))`
///
/// - `age_days` — возраст воспоминания в днях
/// - `tau` — временна́я константа (30 дней = торговый цикл)
/// - `d` — экспонента затухания (0.5 = power law)
/// - `rehearsal_count` — сколько раз воспоминание было извлечено (testing effect)
pub fn episodic_decay(age_days: f64, tau: f64, d: f64, rehearsal_count: u32) -> f64 {
    debug_assert!(age_days >= 0.0, "age_days must be non-negative");
    debug_assert!(tau > 0.0, "tau must be positive");
    debug_assert!(d >= 0.0, "d must be non-negative");

    let forgetting = (1.0 + age_days / tau).powf(-d);
    let rehearsal_boost = 1.0 + 0.3 * (1.0 + rehearsal_count as f64).ln();
    forgetting * rehearsal_boost
}

/// Сила семантической памяти (для обобщённых знаний).
///
/// `S(t) = (1 + t/τ)^(-d)`
///
/// Семантические воспоминания затухают МЕДЛЕННЕЕ эпизодических:
/// - τ = 180 дней (6 месяцев)
/// - d = 0.3 (более пологий спад)
pub fn semantic_decay(age_days: f64, tau: f64, d: f64) -> f64 {
    debug_assert!(age_days >= 0.0, "age_days must be non-negative");
    debug_assert!(tau > 0.0, "tau must be positive");

    (1.0 + age_days / tau).powf(-d)
}

/// Фактор совпадения режима (для взвешивания релевантности).
///
/// - Режимы совпадают → 1.0 (полная релевантность)
/// - Режимы не совпадают → 0.3 (сильно понижена)
/// - Один из режимов неизвестен → 0.6 (нейтрально)
pub fn regime_match_factor(memory_regime: Option<&str>, current_regime: Option<&str>) -> f64 {
    match (memory_regime, current_regime) {
        (Some(m), Some(c)) if m == c => 1.0,
        (Some(_), Some(_)) => 0.3,
        _ => 0.6,
    }
}

/// Константы по умолчанию (из OWM Framework).
pub mod defaults {
    /// Episodic: τ = 30 дней (торговый цикл)
    pub const EPISODIC_TAU: f64 = 30.0;
    /// Episodic: d = 0.5 (power law)
    pub const EPISODIC_D: f64 = 0.5;
    /// Semantic: τ = 180 дней (6 месяцев)
    pub const SEMANTIC_TAU: f64 = 180.0;
    /// Semantic: d = 0.3 (медленное затухание)
    pub const SEMANTIC_D: f64 = 0.3;
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::defaults::*;

    #[test]
    fn test_episodic_decay_at_zero() {
        // При t=0, n=0: S = 1.0 × (1 + 0.3×ln(1)) = 1.0 × 1.0 = 1.0
        let s = episodic_decay(0.0, EPISODIC_TAU, EPISODIC_D, 0);
        assert!((s - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_episodic_decay_decreases_over_time() {
        let s0 = episodic_decay(0.0, EPISODIC_TAU, EPISODIC_D, 0);
        let s30 = episodic_decay(30.0, EPISODIC_TAU, EPISODIC_D, 0);
        let s90 = episodic_decay(90.0, EPISODIC_TAU, EPISODIC_D, 0);
        assert!(s0 > s30);
        assert!(s30 > s90);
        // Power law: после 90 дней ещё ~42% силы (vs 5% у экспоненты)
        assert!(s90 > 0.3, "Power law should retain >30% at 90 days, got {:.3}", s90);
    }

    #[test]
    fn test_rehearsal_boosts_strength() {
        let s_no = episodic_decay(30.0, EPISODIC_TAU, EPISODIC_D, 0);
        let s_5x = episodic_decay(30.0, EPISODIC_TAU, EPISODIC_D, 5);
        let s_20x = episodic_decay(30.0, EPISODIC_TAU, EPISODIC_D, 20);
        assert!(s_5x > s_no, "5 rehearsals should boost strength");
        assert!(s_20x > s_5x, "20 rehearsals should boost more");
    }

    #[test]
    fn test_semantic_decays_slower() {
        let ep = episodic_decay(90.0, EPISODIC_TAU, EPISODIC_D, 0);
        let sem = semantic_decay(90.0, SEMANTIC_TAU, SEMANTIC_D);
        assert!(sem > ep, "Semantic should decay slower: sem={:.3} ep={:.3}", sem, ep);
    }

    #[test]
    fn test_regime_match() {
        assert_eq!(regime_match_factor(Some("trending"), Some("trending")), 1.0);
        assert_eq!(regime_match_factor(Some("trending"), Some("ranging")), 0.3);
        assert_eq!(regime_match_factor(None, Some("trending")), 0.6);
        assert_eq!(regime_match_factor(None, None), 0.6);
    }
}
