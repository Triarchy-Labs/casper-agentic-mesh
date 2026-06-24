/// OWM Recall — Outcome-Weighted Memory scoring.
///
/// Центральная инновация: Score = Q × Sim × Rec × Conf × Aff
///
/// - Q (Outcome Quality)   — sigmoid(k × pnl_r / σ_r)
/// - Sim (Context Similarity) — взвешенное совпадение контекста
/// - Rec (Recency)          — Power Law decay (из decay.rs)
/// - Conf (Confidence)      — 0.5 + 0.5 × confidence
/// - Aff (Affective)        — модуляция на основе эмоционального состояния
///
/// Портировано из: tradememory-protocol/src/tradememory/owm/recall.py
use serde::Serialize;

// ═══════════════════════════════════════════════════════════
// Структуры
// ═══════════════════════════════════════════════════════════

/// Контекстный вектор рыночной ситуации.
/// 8 измерений (расширяемо до 16D с SIMD в будущем).
#[derive(Debug, Clone, Default)]
pub struct ContextVector {
    pub regime: Option<String>,            // trending_up/down, ranging, volatile
    pub volatility_regime: Option<String>,  // low, normal, high, extreme
    pub session: Option<String>,           // asia, london, newyork, overlap
    pub atr_d1: Option<f64>,
    pub atr_h1: Option<f64>,
    pub spread_as_atr_pct: Option<f64>,
    pub drawdown_pct: Option<f64>,
    pub price: Option<f64>,
}

/// Аффективное состояние агента (эмоциональный фон).
#[derive(Debug, Clone, Default)]
pub struct AffectiveState {
    pub drawdown_state: f64,       // 0.0–1.0
    pub consecutive_losses: u32,
}

/// Воспоминание с рассчитанным OWM score.
#[derive(Debug, Clone, Serialize)]
pub struct ScoredMemory {
    pub memory_id: String,
    pub memory_type: String,
    pub score: f64,
    pub q: f64,
    pub sim: f64,
    pub rec: f64,
    pub conf: f64,
    pub aff: f64,
}

/// "Сырое" воспоминание для recall.
#[derive(Debug, Clone)]
pub struct RawMemory {
    pub id: String,
    pub memory_type: String,       // episodic, semantic, prospective
    pub age_days: f64,
    pub confidence: f64,           // 0.0–1.0
    pub pnl_r: Option<f64>,       // PnL в R-множителях
    pub context: ContextVector,
    pub rehearsal_count: u32,
}

// ═══════════════════════════════════════════════════════════
// Компоненты Score
// ═══════════════════════════════════════════════════════════

/// Стабильный sigmoid: 1 / (1 + exp(-x))
fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let ex = x.exp();
        ex / (1.0 + ex)
    }
}

/// Q(m) — Outcome Quality Score ∈ (0, 1).
/// sigma_r = 1.5, k = 2.0
pub fn outcome_quality(pnl_r: Option<f64>, confidence: f64) -> f64 {
    match pnl_r {
        Some(r) => sigmoid(2.0 * r / 1.5),
        None => confidence.clamp(0.0, 1.0),
    }
}

/// Conf(m) — Confidence Factor ∈ [0.5, 1.0].
pub fn confidence_factor(confidence: f64) -> f64 {
    0.5 + 0.5 * confidence.clamp(0.0, 1.0)
}

/// Aff(m) — Affective Modulation ∈ [0.7, 1.3].
pub fn affective_modulation(pnl_r: Option<f64>, state: &AffectiveState) -> f64 {
    let relevance = if state.drawdown_state > 0.5 {
        match pnl_r {
            Some(r) if r < -1.5 => 0.5,   // Boost loss memories during drawdown
            Some(r) if r > 2.0 => 0.3,    // Also surface big wins
            _ => 0.0,
        }
    } else if state.consecutive_losses >= 3 {
        match pnl_r {
            Some(r) if r > 0.0 => 0.3,    // Surface winners during losing streak
            _ => -0.2,                      // Suppress loss memories
        }
    } else {
        0.0
    };

    let raw: f64 = 1.0 + 0.3 * relevance;
    raw.clamp(0.7, 1.3)
}

// ═══════════════════════════════════════════════════════════
// Context Similarity (weighted categorical + Gaussian numerical)
// ═══════════════════════════════════════════════════════════

/// Similarity между двумя контекстными векторами ∈ [0, 1].
pub fn context_similarity(c1: &ContextVector, c2: &ContextVector) -> f64 {
    let mut score = 0.0_f64;
    let mut total_weight = 0.0_f64;

    // Categorical: exact match
    let cats: [(&Option<String>, &Option<String>, f64); 3] = [
        (&c1.regime, &c2.regime, 0.25),
        (&c1.volatility_regime, &c2.volatility_regime, 0.15),
        (&c1.session, &c2.session, 0.10),
    ];
    for (v1, v2, w) in &cats {
        if let (Some(a), Some(b)) = (v1, v2) {
            total_weight += w;
            if a == b {
                score += w;
            }
        }
    }

    // Numerical: Gaussian kernel
    let nums: [(Option<f64>, Option<f64>, f64, f64); 5] = [
        (c1.atr_d1, c2.atr_d1, 0.15, 0.3),
        (c1.atr_h1, c2.atr_h1, 0.10, 0.3),
        (c1.spread_as_atr_pct, c2.spread_as_atr_pct, 0.05, 0.5),
        (c1.drawdown_pct, c2.drawdown_pct, 0.10, 0.1),
        (c1.price, c2.price, 0.10, 0.2),
    ];
    for (v1, v2, w, bw) in &nums {
        if let (Some(a), Some(b)) = (v1, v2) {
            if *a != 0.0 {
                total_weight += w;
                let ratio = (a - b) / (bw * a.abs());
                let sim = (-0.5 * ratio * ratio).exp();
                score += w * sim;
            }
        }
    }

    if total_weight > 0.0 { score / total_weight } else { 0.5 }
}

/// Score = exp(w_q·ln(Q) + w_s·ln(Sim) + w_r·ln(Rec) + w_c·ln(Conf) + w_a·ln(Aff))
///
/// Log-additive: один слабый компонент НЕ убивает весь score.
/// Floor values: min 0.05 для каждого компонента → никогда не 0.
pub fn outcome_weighted_recall(
    query: &ContextVector,
    memories: &[RawMemory],
    affective: &AffectiveState,
    limit: usize,
) -> Vec<ScoredMemory> {
    // Веса компонентов (нормализованы, сумма = 1.0)
    const W_Q: f64   = 0.30;  // Outcome Quality — самый важный
    const W_SIM: f64  = 0.25;  // Context Similarity
    const W_REC: f64  = 0.20;  // Recency
    const W_CONF: f64 = 0.15;  // Confidence
    const W_AFF: f64  = 0.10;  // Affective modulation
    const FLOOR: f64  = 0.05;  // Минимальный пол (защита от нуля)

    let mut candidates: Vec<ScoredMemory> = memories.iter().map(|m| {
        let (tau, d) = match m.memory_type.as_str() {
            "semantic" => (180.0, 0.3),
            _ => (30.0, 0.5),
        };

        let q = outcome_quality(m.pnl_r, m.confidence).max(FLOOR);
        let sim = context_similarity(&m.context, query).max(FLOOR);
        let rec = crate::decay::episodic_decay(m.age_days, tau, d, m.rehearsal_count).max(FLOOR);
        let conf = confidence_factor(m.confidence).max(FLOOR);
        let aff = affective_modulation(m.pnl_r, affective).max(FLOOR);

        // Log-additive scoring
        let log_score = W_Q * q.ln()
            + W_SIM * sim.ln()
            + W_REC * rec.ln()
            + W_CONF * conf.ln()
            + W_AFF * aff.ln();
        let score = log_score.exp();

        ScoredMemory {
            memory_id: m.id.clone(),
            memory_type: m.memory_type.clone(),
            score, q, sim, rec, conf, aff,
        }
    }).collect();

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(limit);
    candidates
}

// ═══════════════════════════════════════════════════════════
// Тесты
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory(id: &str, pnl_r: f64, age: f64, regime: &str) -> RawMemory {
        RawMemory {
            id: id.to_string(),
            memory_type: "episodic".to_string(),
            age_days: age,
            confidence: 0.7,
            pnl_r: Some(pnl_r),
            context: ContextVector {
                regime: Some(regime.to_string()),
                session: Some("london".to_string()),
                ..Default::default()
            },
            rehearsal_count: 0,
        }
    }

    #[test]
    fn test_sigmoid_symmetry() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!((sigmoid(5.0) + sigmoid(-5.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_outcome_quality_positive() {
        let q = outcome_quality(Some(2.0), 0.5);
        assert!(q > 0.7, "Big winner should have Q > 0.7, got {:.3}", q);
    }

    #[test]
    fn test_outcome_quality_negative() {
        let q = outcome_quality(Some(-2.0), 0.5);
        assert!(q < 0.3, "Big loser should have Q < 0.3, got {:.3}", q);
    }

    #[test]
    fn test_context_similarity_identical() {
        let c = ContextVector {
            regime: Some("trending_up".to_string()),
            session: Some("london".to_string()),
            price: Some(100000.0),
            ..Default::default()
        };
        let sim = context_similarity(&c, &c);
        assert!((sim - 1.0).abs() < 1e-10, "Identical contexts should have sim=1.0");
    }

    #[test]
    fn test_context_similarity_different_regime() {
        let c1 = ContextVector {
            regime: Some("trending_up".to_string()),
            ..Default::default()
        };
        let c2 = ContextVector {
            regime: Some("ranging".to_string()),
            ..Default::default()
        };
        let sim = context_similarity(&c1, &c2);
        assert!(sim < 0.1, "Different regimes should have low similarity, got {:.3}", sim);
    }

    #[test]
    fn test_recall_ranks_by_score() {
        let query = ContextVector {
            regime: Some("trending_up".to_string()),
            session: Some("london".to_string()),
            ..Default::default()
        };

        let memories = vec![
            make_memory("old_loser", -1.5, 60.0, "ranging"),
            make_memory("recent_winner", 2.0, 1.0, "trending_up"),
            make_memory("mid_neutral", 0.0, 15.0, "trending_up"),
        ];

        let affective = AffectiveState::default();
        let results = outcome_weighted_recall(&query, &memories, &affective, 10);

        assert_eq!(results[0].memory_id, "recent_winner");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_affective_during_drawdown() {
        let state = AffectiveState { drawdown_state: 0.8, consecutive_losses: 5 };
        let aff_loss = affective_modulation(Some(-2.0), &state);
        let aff_neutral = affective_modulation(Some(0.5), &state);
        assert!(aff_loss > aff_neutral, "Big loss memories should be boosted during drawdown");
    }
}
