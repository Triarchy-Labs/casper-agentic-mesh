/// Hybrid Recall: vector similarity + OWM fusion.
///
/// ПОРТИРОВАНО ИЗ: tradememory-protocol/src/tradememory/hybrid_recall.py (177 строк)
/// АВТОР ОРИГИНАЛА: mnemox-ai (MIT License)
///
/// Blends cosine similarity (embeddings) with OWM scoring (outcome-weighted).
/// Falls back to pure OWM when embeddings are unavailable.
///
/// Key feature: ensure_negative_balance — guarantees ≥20% negative memories
/// in recall results to prevent survivorship bias.
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════
// Cosine similarity (порт: hybrid_recall.py:21-30)
// ═══════════════════════════════════════════════════════════════

/// Cosine similarity between two vectors. Returns 0.0 on degenerate input.
/// Delegates to turbo::cosine_similarity_fast (SIMD 4x unrolled) for performance.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    crate::turbo::cosine_similarity_fast(a, b)
}

// ═══════════════════════════════════════════════════════════════
// ScoredMemory for hybrid ranking
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct HybridScoredMemory {
    pub memory_id: String,
    pub score: f64,         // blended score
    pub owm_score: f64,     // OWM component
    pub vector_sim: f64,    // vector similarity component
    pub pnl_r: Option<f64>, // for negative balance enforcement
}

// ═══════════════════════════════════════════════════════════════
// Negative Balance Enforcement (порт: hybrid_recall.py:38-102)
// ═══════════════════════════════════════════════════════════════

/// Ensure negative memories (pnl_r < 0) comprise >= min_negative_ratio of results.
///
/// If the ratio is already met, returns unchanged.
/// Otherwise, swaps lowest-scoring positive memories with highest-scoring
/// negative memories from remaining candidates.
///
/// Порт: hybrid_recall.py:38-102 (ensure_negative_balance)
pub fn ensure_negative_balance(
    results: &mut [HybridScoredMemory],
    all_candidates: &[HybridScoredMemory],
    min_negative_ratio: f64,
) {
    if results.is_empty() {
        return;
    }

    let target_count = (results.len() as f64 * min_negative_ratio).ceil().max(1.0) as usize;

    let negative_count = results
        .iter()
        .filter(|r| r.pnl_r.is_some_and(|p| p < 0.0))
        .count();

    if negative_count >= target_count {
        return;
    }

    let need = target_count - negative_count;

    // Find result IDs
    let result_ids: std::collections::HashSet<&str> = results
        .iter()
        .map(|r| r.memory_id.as_str())
        .collect();

    // Spare negatives from candidates not in results
    let mut spare_negatives: Vec<&HybridScoredMemory> = all_candidates
        .iter()
        .filter(|c| !result_ids.contains(c.memory_id.as_str()))
        .filter(|c| c.pnl_r.is_some_and(|p| p < 0.0))
        .collect();
    spare_negatives.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    if spare_negatives.is_empty() {
        return;
    }

    // Find positive memories in results (sorted by score ascending — weakest first)
    let mut positive_indices: Vec<usize> = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.pnl_r.is_none_or(|p| p >= 0.0))
        .map(|(i, _)| i)
        .collect();
    positive_indices.sort_by(|&a, &b| {
        results[a].score.partial_cmp(&results[b].score).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Swap
    let mut swapped = 0;
    for neg in spare_negatives {
        if swapped >= need {
            break;
        }
        if let Some(&idx) = positive_indices.first() {
            results[idx] = neg.clone();
            positive_indices.remove(0);
            swapped += 1;
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
}

// ═══════════════════════════════════════════════════════════════
// Hybrid Recall (порт: hybrid_recall.py:105-176)
// ═══════════════════════════════════════════════════════════════

/// Blend OWM scores with vector similarity.
///
/// * `alpha` — blend weight. 0.0 = pure OWM, 1.0 = pure vector.
/// * Results are sorted by blended score and negative-balance enforced.
pub fn hybrid_blend(
    owm_scores: &[(String, f64, Option<f64>)],  // (id, owm_score, pnl_r)
    query_embedding: Option<&[f64]>,
    memory_embeddings: &[(String, Vec<f64>)],   // (id, embedding)
    alpha: f64,
    limit: usize,
) -> Vec<HybridScoredMemory> {
    if owm_scores.is_empty() {
        return vec![];
    }

    // Build embedding lookup
    let emb_map: std::collections::HashMap<&str, &[f64]> = memory_embeddings
        .iter()
        .map(|(id, emb)| (id.as_str(), emb.as_slice()))
        .collect();

    let use_vector = query_embedding.is_some() && !memory_embeddings.is_empty();

    let mut all_candidates: Vec<HybridScoredMemory> = owm_scores
        .iter()
        .map(|(id, owm, pnl_r)| {
            let vector_sim = if use_vector {
                if let (Some(q_emb), Some(m_emb)) = (query_embedding, emb_map.get(id.as_str())) {
                    cosine_similarity(q_emb, m_emb)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let blended = if use_vector {
                alpha * vector_sim + (1.0 - alpha) * owm
            } else {
                *owm
            };

            HybridScoredMemory {
                memory_id: id.clone(),
                score: blended,
                owm_score: *owm,
                vector_sim,
                pnl_r: *pnl_r,
            }
        })
        .collect();

    all_candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut top: Vec<HybridScoredMemory> = all_candidates.iter().take(limit).cloned().collect();
    ensure_negative_balance(&mut top, &all_candidates, 0.2);
    top
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 0.001, "Identical vectors → sim=1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001, "Orthogonal → sim=0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_opposite() {
        let a = vec![1.0, 2.0];
        let b = vec![-1.0, -2.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.001, "Opposite → sim=-1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_empty() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn test_cosine_length_mismatch() {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_negative_balance_already_met() {
        let mut results = vec![
            HybridScoredMemory { memory_id: "1".into(), score: 0.9, owm_score: 0.9, vector_sim: 0.0, pnl_r: Some(1.0) },
            HybridScoredMemory { memory_id: "2".into(), score: 0.8, owm_score: 0.8, vector_sim: 0.0, pnl_r: Some(-0.5) },
        ];
        let all = results.clone();
        ensure_negative_balance(&mut results, &all, 0.2);
        assert_eq!(results.len(), 2); // unchanged
    }

    #[test]
    fn test_negative_balance_swap() {
        let mut results = vec![
            HybridScoredMemory { memory_id: "1".into(), score: 0.9, owm_score: 0.9, vector_sim: 0.0, pnl_r: Some(1.0) },
            HybridScoredMemory { memory_id: "2".into(), score: 0.8, owm_score: 0.8, vector_sim: 0.0, pnl_r: Some(2.0) },
            HybridScoredMemory { memory_id: "3".into(), score: 0.7, owm_score: 0.7, vector_sim: 0.0, pnl_r: Some(0.5) },
        ];
        let all = vec![
            results[0].clone(), results[1].clone(), results[2].clone(),
            HybridScoredMemory { memory_id: "4".into(), score: 0.6, owm_score: 0.6, vector_sim: 0.0, pnl_r: Some(-1.0) },
        ];
        ensure_negative_balance(&mut results, &all, 0.2);
        let neg_count = results.iter().filter(|r| r.pnl_r.map_or(false, |p| p < 0.0)).count();
        assert!(neg_count >= 1, "Should have swapped in a negative memory");
    }

    #[test]
    fn test_hybrid_blend_pure_owm() {
        let owm = vec![
            ("a".to_string(), 0.9, Some(1.0)),
            ("b".to_string(), 0.5, Some(-0.5)),
            ("c".to_string(), 0.3, Some(0.2)),
        ];
        let result = hybrid_blend(&owm, None, &[], 0.3, 10);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].memory_id, "a"); // highest OWM
    }

    #[test]
    fn test_hybrid_blend_with_vectors() {
        let owm = vec![
            ("a".to_string(), 0.9, Some(1.0)),
            ("b".to_string(), 0.2, Some(-0.5)),
        ];
        let query = vec![1.0, 0.0, 0.0];
        let embeddings = vec![
            ("a".to_string(), vec![0.0, 1.0, 0.0]),  // orthogonal to query
            ("b".to_string(), vec![1.0, 0.0, 0.0]),  // identical to query
        ];
        let result = hybrid_blend(&owm, Some(&query), &embeddings, 0.5, 10);
        assert_eq!(result.len(), 2);
        // "b" has low OWM but high vector sim — blended should rerank
    }

    #[test]
    fn test_hybrid_blend_limit() {
        let owm: Vec<_> = (0..20).map(|i| (format!("{}", i), i as f64 / 20.0, Some(0.5))).collect();
        let result = hybrid_blend(&owm, None, &[], 0.0, 5);
        assert_eq!(result.len(), 5);
    }
}
