/// Brain V3 — Центральный оркестратор когнитивных модулей.
///
/// Diamond-verified: ВСЕ модули подключены включая OWM Recall.
/// Bloom rotation, semantic/causal pruning, NaN-safe.
use serde::{Serialize, Deserialize};
use crate::entity::MemoryEntity;
use crate::semantic::SemanticStore;
use crate::drift;
use crate::regime;
use crate::bloom::{BloomFilter, context_hash};
use crate::affective;
use crate::causal::CausalGraph;
use crate::adaptive::AdaptiveWeightStore;
use crate::recall::{self, ContextVector, AffectiveState, RawMemory};
use crate::dqs::{DqsEngine, DqsInput, DqsTier};
use crate::changepoint::BayesianChangepoint;
use crate::orderbook_imbalance::ImbalanceEngine;

/// Диагностика мозга на один трейд.
#[derive(Debug, Clone, Serialize)]
pub struct BrainDiagnostics {
    pub symbol: String,
    pub regime: String,
    pub regime_confidence: f64,
    pub ewma_confidence: f64,
    pub risk_appetite: f64,
    pub drift_detected: bool,
    pub drift_cusum: f64,
    pub disposition_detected: bool,
    pub disposition_severity: String,
    pub is_novel_context: bool,
    pub belief_confidence: Option<f64>,
    pub causal_predictions: Vec<String>,
    pub owm_score: f64,
    // ═══ Surgical Port: DQS + Changepoint + Risk Sizing ═══
    pub dqs_score: f64,
    pub dqs_tier: String,
    pub dqs_position_multiplier: f64,
    pub changepoint_probability: f64,
    pub kelly_fraction: f64,
    // ═══ F15: Order Book Imbalance ═══
    pub obi_ratio: f64,
    pub obi_bias: String,
    pub obi_confidence: f64,
}

/// Persistent state для save/load.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrainPersist {
    semantic: SemanticStore,
    causal: CausalGraph,
    bloom: BloomFilter,
    adaptive: AdaptiveWeightStore,
    peak_equity: std::collections::HashMap<String, f64>,
    last_events: Vec<(String, i64)>,
}

/// Центральный мозг Memory Castle.
pub struct Brain {
    pub semantic: SemanticStore,
    pub causal: CausalGraph,
    pub bloom: BloomFilter,
    pub adaptive: AdaptiveWeightStore,
    pub last_diagnostics: Option<BrainDiagnostics>,
    pub peak_equity: std::collections::HashMap<String, f64>,
    pub last_events: Vec<(String, i64)>,
    pub bloom_insertions: usize,
    // ═══ Surgical Port modules ═══
    pub dqs_engine: DqsEngine,
    pub changepoint: BayesianChangepoint,
    // ═══ F15: Order Book Imbalance Engine ═══
    pub obi_engine: ImbalanceEngine,
}

/// Максимум bloom insertions до ротации (поддерживает FP < 0.1%).
const BLOOM_ROTATION_THRESHOLD: usize = 8_000;
/// Максимум semantic beliefs (пруним слабые).
const MAX_BELIEFS: usize = 500;
/// Максимум causal edges (пруним слабые).
const MAX_CAUSAL_EDGES: usize = 1000;

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}

impl Brain {
    pub fn new() -> Self {
        Self {
            semantic: SemanticStore::new(),
            causal: CausalGraph::new(),
            bloom: BloomFilter::optimal(10_000, 0.001),
            adaptive: AdaptiveWeightStore::new(),
            last_diagnostics: None,
            peak_equity: std::collections::HashMap::new(),
            last_events: Vec::new(),
            bloom_insertions: 0,
            dqs_engine: DqsEngine::new(),
            changepoint: BayesianChangepoint::new(100.0),
            obi_engine: ImbalanceEngine::new(),
        }
    }

    /// Загрузить мозг из файла или создать новый.
    pub fn load_or_new(path: &str) -> Self {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(state) = serde_json::from_str::<BrainPersist>(&data) {
                println!("[BRAIN] Loaded: {} beliefs, {} causal edges, {} adaptive weights, {} bloom items",
                    state.semantic.beliefs.len(),
                    state.causal.edges.len(),
                    state.adaptive.weights.len(),
                    state.bloom.len());
                return Self {
                    semantic: state.semantic,
                    causal: state.causal,
                    bloom: state.bloom,
                    adaptive: state.adaptive,
                    last_diagnostics: None,
                    peak_equity: state.peak_equity,
                    last_events: state.last_events,
                    bloom_insertions: 0,
                    dqs_engine: DqsEngine::new(),
                    changepoint: BayesianChangepoint::new(100.0),
                    obi_engine: ImbalanceEngine::new(),
                };
            }
            println!("[BRAIN] Corrupted brain file, starting fresh.");
        }
        println!("[BRAIN] No brain file found, starting fresh.");
        Self::new()
    }

    /// Сохранить мозг на диск (JSON).
    pub fn save_brain(&self, path: &str) {
        let state = BrainPersist {
            semantic: self.semantic.clone(),
            causal: self.causal.clone(),
            bloom: self.bloom.clone(),
            adaptive: self.adaptive.clone(),
            peak_equity: self.peak_equity.clone(),
            last_events: self.last_events.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Конвертирует PnL историю в returns (процентные изменения).
    fn pnl_to_returns(pnl_history: &[f64]) -> Vec<f64> {
        if pnl_history.len() < 2 { return vec![]; }
        let mut returns = Vec::with_capacity(pnl_history.len() - 1);
        let mut equity = 100.0; // Базовый equity
        for &pnl in pnl_history {
            let ret = pnl / equity;
            returns.push(ret);
            equity += pnl;
            if equity < 1.0 { equity = 1.0; } // Защита от деления на 0
        }
        returns
    }

    /// Главный метод: обработать новый трейд всеми когнитивными модулями.
    pub fn process_trade(&mut self, entity: &MemoryEntity, strategy: &str) -> BrainDiagnostics {
        let symbol = &entity.entity_id;

        let pnl_slice = entity.recent_pnl_slice();

        // ═══ 1. REGIME: классификация на RETURNS, не PnL ═══
        let returns = Self::pnl_to_returns(&pnl_slice);
        let regime_result = regime::classify_regime(
            &returns,
            0.01, // historical vol baseline
        );

        // ═══ 2. SEMANTIC: обновить Bayesian belief ═══
        let last_pnl = pnl_slice.last().copied().unwrap_or(0.0);
        self.semantic.record_outcome(strategy, regime_result.regime.as_str(), last_pnl);
        let belief_confidence = self.semantic.query_confidence(strategy, regime_result.regime.as_str());

        // ═══ 3. DRIFT: проверить деградацию win rate ═══
        let drift_result = if pnl_slice.len() >= 10 {
            drift::cusum_degradation(&pnl_slice, 0.5, 4.0)
        } else {
            drift::CusumResult { drift_detected: false, drift_point: None, final_cusum: 0.0 }
        };

        // ═══ 4. DISPOSITION: проверить эмоциональный перекос ═══
        let disposition = if pnl_slice.len() >= 4 && entity.recent_hold_ms.len() >= 4 {
            let hold_vec = entity.recent_hold_ms_vec();
            let trades: Vec<(f64, i64)> = pnl_slice.iter()
                .zip(hold_vec.iter())
                .map(|(&pnl, &hold)| (pnl, hold))
                .collect();
            drift::detect_disposition(&trades)
        } else {
            drift::DispositionAnalysis {
                avg_win_hold_ms: 0.0, avg_loss_hold_ms: 0.0,
                ratio: 1.0, disposition_detected: false,
                severity: "none".to_string(),
            }
        };

        // ═══ 5. AFFECTIVE: EWMA + РЕАЛЬНЫЙ drawdown от peak equity ═══
        let ewma_conf = affective::ewma_confidence(&pnl_slice, 0.9);
        
        // Track peak equity per symbol
        let peak = self.peak_equity.entry(symbol.clone()).or_insert(0.0);
        if entity.net_pnl > *peak {
            *peak = entity.net_pnl;
        }
        let drawdown = (*peak - entity.net_pnl).max(0.0);
        let risk_app = affective::risk_appetite(drawdown, 50.0);

        // ═══ 6. BLOOM: novelty check + ROTATION ═══
        let ctx_hash = context_hash(symbol, regime_result.regime.as_str(), "");
        let is_novel = !self.bloom.maybe_contains(ctx_hash);
        if is_novel {
            self.bloom.insert(ctx_hash);
            self.bloom_insertions += 1;
            // Ротация bloom filter когда FP rate растёт
            if self.bloom_insertions >= BLOOM_ROTATION_THRESHOLD {
                println!("  🔄 [BRAIN] Bloom filter rotated after {} insertions", self.bloom_insertions);
                self.bloom = BloomFilter::optimal(10_000, 0.001);
                self.bloom_insertions = 0;
            }
        }

        // ═══ 7. CAUSAL: МЕЖСИМВОЛЬНЫЕ события ═══
        let event_name = format!("{}_{}", symbol,
            if last_pnl > 0.0 { "win" } else { "loss" });
        let event_ts = entity.last_trade_ts;

        // Записать каузальные связи с ПРЕДЫДУЩИМИ событиями ДРУГИХ символов
        let max_lag_ms: i64 = 3_600_000; // 1 час окно
        for (prev_event, prev_ts) in &self.last_events {
            let lag = event_ts - prev_ts;
            if lag > 0 && lag <= max_lag_ms && !prev_event.starts_with(symbol.as_str()) {
                self.causal.observe(
                    prev_event,
                    &event_name,
                    last_pnl > 0.0,
                    lag as f64,
                );
            }
        }

        // Добавить текущее событие в историю (ring buffer 100 событий)
        self.last_events.push((event_name.clone(), event_ts));
        if self.last_events.len() > 100 {
            self.last_events.remove(0);
        }

        // Получить предсказания каузального графа
        let causal_predictions: Vec<String> = self.causal.predict(&event_name)
            .iter()
            .map(|(effect, strength, delay)| {
                format!("{effect} (P={strength:.2}, lag={delay:.0}ms)")
            })
            .collect();

        // ═══ 8. ADAPTIVE: обновить веса на основе результата ═══
        let components = [
            entity.win_rate,                                    // Q proxy
            regime_result.confidence,                            // Sim proxy
            1.0 / (1.0 + entity.trade_count as f64 * 0.01),    // Rec proxy
            belief_confidence.unwrap_or(0.5),                    // Conf proxy
            ewma_conf,                                           // Aff proxy
        ];
        let outcome = last_pnl > 0.0;
        self.adaptive.update(symbol, &components, outcome, 0.05);

        // ═══ 9. OWM RECALL: Score = Q × Sim × Rec × Conf × Aff ═══
        let current_context = ContextVector {
            regime: Some(regime_result.regime.as_str().to_string()),
            volatility_regime: if regime_result.confidence > 0.7 {
                Some("high".to_string())
            } else {
                Some("normal".to_string())
            },
            session: Some(crate::patterns::TradingSession::from_timestamp_ms(entity.last_trade_ts).name().to_string()),
            atr_d1: None,  // TODO: получать из Titan/Ouroboros
            atr_h1: None,
            spread_as_atr_pct: None,
            drawdown_pct: Some(drawdown),
            price: None,   // TODO: получать из payload
        };

        // Построить RawMemory из текущего entity для OWM scoring
        let age_days = if entity.last_trade_ts > 0 {
            // Примерный возраст в днях от первого трейда
            (entity.trade_count as f64 * 0.01).max(0.01) // ~1 day per 100 trades
        } else { 0.01 };
        let pnl_r = if entity.trade_count > 0 {
            // PnL в R-множителях: avg_pnl / avg_loss_size
            let avg_loss = entity.avg_loss_size.abs().max(0.01);
            Some(last_pnl / avg_loss)
        } else { None };

        let memory = RawMemory {
            id: symbol.clone(),
            memory_type: "episodic".to_string(),
            age_days,
            confidence: belief_confidence.unwrap_or(0.5),
            pnl_r,
            context: current_context.clone(),
            rehearsal_count: entity.trade_count.max(0) as u32,
        };

        let aff_state = AffectiveState {
            drawdown_state: (drawdown / 50.0).min(1.0),
            consecutive_losses: entity.current_loss_streak.max(0) as u32,
        };

        let scored = recall::outcome_weighted_recall(
            &current_context, &[memory], &aff_state, 1
        );
        let owm_score = scored.first().map(|s| s.score).unwrap_or(0.0);

        // ═══ 9.5. DQS PRE-TRADE GATE (Surgical Port) ═══
        let kelly_f = crate::risk_sizing::half_kelly(
            entity.win_rate,
            entity.avg_win_size.max(0.01),
            entity.avg_loss_size.abs().max(0.01),
        );

        let dqs_input = DqsInput {
            regime_win_rate: self.semantic.query_confidence(strategy, regime_result.regime.as_str()),
            overall_win_rate: entity.win_rate,
            proposed_lot: 0.1, // TODO: получать от Titan
            kelly_fraction: if kelly_f > 0.0 { Some(kelly_f) } else { None },
            owm_score,
            drawdown_pct: if *peak > 0.0 { (drawdown / *peak) * 100.0 } else { 0.0 },
            consecutive_losses: entity.current_loss_streak.max(0) as u32,
            confidence: belief_confidence.unwrap_or(0.5),
            avg_pnl_r_similar: owm_score, // Use OWM as similarity proxy
        };
        let dqs_result = self.dqs_engine.compute(&dqs_input);

        // ═══ 9.6. CHANGEPOINT DETECTION (Surgical Port) ═══
        let cp_result = self.changepoint.update(last_pnl > 0.0, last_pnl);

        // DQS + Changepoint logging
        match dqs_result.tier {
            DqsTier::Skip => println!("  🚫 [DQS] {} SKIP — score {:.2}, too risky", symbol, dqs_result.score),
            DqsTier::Caution => println!("  ⚠️ [DQS] {} CAUTION — score {:.2}, half size", symbol, dqs_result.score),
            DqsTier::Go => {}
        }
        if cp_result.changepoint_probability > 0.1 {
            println!("  🔄 [CHANGEPOINT] {} regime shift P={:.3}, run_length={}",
                symbol, cp_result.changepoint_probability, cp_result.max_run_length);
        }

        // ═══ 10. ORDER BOOK IMBALANCE (F15) ═══
        // OBI запускается когда brain получает данные orderbook через отдельный метод
        // process_orderbook(). Здесь считываем ПОСЛЕДНИЙ результат.
        let obi_diag = self.obi_engine.history.back().copied().unwrap_or(0.0);
        let obi_bias_str = if obi_diag > 0.3 {
            "bullish"
        } else if obi_diag < -0.3 {
            "bearish"
        } else {
            "neutral"
        };
        let obi_confidence = obi_diag.abs().min(1.0);

        // ═══ 11. PRUNING: держать semantic/causal в границах ═══
        if self.semantic.beliefs.len() > MAX_BELIEFS {
            self.semantic.beliefs.sort_by_key(|b| std::cmp::Reverse(b.evidence_count));
            self.semantic.beliefs.truncate(MAX_BELIEFS);
            println!("  🧹 [BRAIN] Pruned beliefs to {MAX_BELIEFS}");
        }
        if self.causal.edges.len() > MAX_CAUSAL_EDGES {
            let mut edges_vec: Vec<_> = self.causal.edges.drain().collect();
            edges_vec.sort_by_key(|(_, e)| std::cmp::Reverse(e.observations));
            edges_vec.truncate(MAX_CAUSAL_EDGES);
            self.causal.edges = edges_vec.into_iter().collect();
            println!("  🧹 [BRAIN] Pruned causal edges to {MAX_CAUSAL_EDGES}");
        }

        // ═══ 11. DIAGNOSTICS ═══
        if drift_result.drift_detected {
            println!("  ⚠️ [BRAIN] {} DRIFT DETECTED (CUSUM={:.1}) — win rate degrading!", 
                symbol, drift_result.final_cusum);
        }
        if disposition.disposition_detected {
            println!("  ⚠️ [BRAIN] {} DISPOSITION EFFECT ({}) — cutting winners too early!", 
                symbol, disposition.severity);
        }
        if is_novel {
            println!("  🆕 [BRAIN] {} NOVEL CONTEXT — first time in {} regime", 
                symbol, regime_result.regime.as_str());
        }
        if drawdown > 10.0 {
            println!("  🔴 [BRAIN] {} DRAWDOWN ${:.2} from peak ${:.2} — risk appetite {:.2}", 
                symbol, drawdown, *peak, risk_app);
        }
        if !causal_predictions.is_empty() {
            println!("  🔮 [BRAIN] {symbol} CAUSAL: {causal_predictions:?}");
        }

        let diag = BrainDiagnostics {
            symbol: symbol.clone(),
            regime: regime_result.regime.as_str().to_string(),
            regime_confidence: regime_result.confidence,
            ewma_confidence: ewma_conf,
            risk_appetite: risk_app,
            drift_detected: drift_result.drift_detected,
            drift_cusum: drift_result.final_cusum,
            disposition_detected: disposition.disposition_detected,
            disposition_severity: disposition.severity,
            is_novel_context: is_novel,
            belief_confidence,
            causal_predictions,
            owm_score,
            dqs_score: dqs_result.score,
            dqs_tier: dqs_result.tier.as_str().to_string(),
            dqs_position_multiplier: dqs_result.position_multiplier,
            changepoint_probability: cp_result.changepoint_probability,
            kelly_fraction: kelly_f,
            obi_ratio: obi_diag,
            obi_bias: obi_bias_str.to_string(),
            obi_confidence,
        };

        self.last_diagnostics = Some(diag.clone());
        diag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::MemoryEntity;

    fn make_entity_with_history(symbol: &str, pnls: &[f64]) -> MemoryEntity {
        let mut e = MemoryEntity::new(symbol);
        let mut ts = 1000_i64;
        for &pnl in pnls {
            e.trade_count += 1;
            e.net_pnl += pnl;
            e.push_pnl(pnl);
            e.push_hold_ms(5000);
            e.last_trade_ts = ts;
            ts += 60000; // 1 min apart
            if pnl > 0.0 { e.win_count += 1; } else { e.loss_count += 1; }
        }
        e.recalculate_derived();
        e
    }

    #[test]
    fn test_pnl_to_returns() {
        let pnl = vec![10.0, -5.0, 8.0];
        let returns = Brain::pnl_to_returns(&pnl);
        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.1).abs() < 1e-10, "10/100 = 0.1");
    }

    #[test]
    fn test_brain_processes_trade() {
        let mut brain = Brain::new();
        let entity = make_entity_with_history("BTCUSDT", &[10.0, -5.0, 8.0, -3.0, 12.0]);
        let diag = brain.process_trade(&entity, "VolBreakout");
        
        assert_eq!(diag.symbol, "BTCUSDT");
        assert!(!diag.regime.is_empty());
        assert!(diag.ewma_confidence > 0.0);
    }

    #[test]
    fn test_brain_detects_drift() {
        let mut brain = Brain::new();
        let entity = make_entity_with_history("BADCOIN", 
            &[-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]);
        let diag = brain.process_trade(&entity, "TestStrat");
        
        assert!(diag.drift_detected, "12 consecutive losses should trigger drift");
    }

    #[test]
    fn test_brain_novelty_detection() {
        let mut brain = Brain::new();
        let entity = make_entity_with_history("NEWCOIN", &[5.0]);
        
        let diag1 = brain.process_trade(&entity, "Strat");
        assert!(diag1.is_novel_context, "First time should be novel");
        
        let diag2 = brain.process_trade(&entity, "Strat");
        assert!(!diag2.is_novel_context, "Second time should NOT be novel");
    }

    #[test]
    fn test_brain_drawdown_tracking() {
        let mut brain = Brain::new();
        
        // First: entity at +20 → peak = 20
        let mut e1 = make_entity_with_history("BTC", &[20.0]);
        brain.process_trade(&e1, "S");
        
        // Then: entity drops to +10 → drawdown = 10
        e1.net_pnl = 10.0;
        e1.push_pnl(-10.0);
        e1.trade_count += 1;
        let diag = brain.process_trade(&e1, "S");
        
        assert!(diag.risk_appetite < 1.0, "Should reduce risk appetite during drawdown");
    }

    #[test]
    fn test_brain_cross_symbol_causal() {
        let mut brain = Brain::new();
        
        // BTC win at t=1000
        let btc = make_entity_with_history("BTC", &[10.0]);
        brain.process_trade(&btc, "S");
        
        // ETH loss at t=1060000 (1 min later)
        let mut eth = MemoryEntity::new("ETH");
        eth.trade_count = 1;
        eth.net_pnl = -5.0;
        eth.push_pnl(-5.0);
        eth.push_hold_ms(3000);
        eth.last_trade_ts = 1060000;
        eth.loss_count = 1;
        brain.process_trade(&eth, "S");
        
        // Check that causal edge BTC_win → ETH_loss exists
        let edges = brain.causal.effects_of("BTC_win");
        assert!(!edges.is_empty(), "Should have cross-symbol causal edge");
    }

    #[test]
    fn test_brain_adaptive_updates() {
        let mut brain = Brain::new();
        let entity = make_entity_with_history("BTC", &[10.0, 5.0, -2.0]);
        brain.process_trade(&entity, "Strat");
        
        let w = brain.adaptive.get_weights("BTC");
        assert!(w.updates > 0, "Adaptive weights should be updated");
    }

    #[test]
    fn test_brain_persistence_roundtrip() {
        let mut brain = Brain::new();
        
        // Process some trades to populate brain state
        let btc = make_entity_with_history("BTC", &[10.0, -5.0, 8.0]);
        brain.process_trade(&btc, "VolBreakout");
        let eth = make_entity_with_history("ETH", &[-3.0, 7.0]);
        brain.process_trade(&eth, "Scalping");
        
        // Save
        let tmp = std::env::temp_dir().join("test_brain_persist.json");
        let path = tmp.to_str().unwrap();
        brain.save_brain(path);
        
        // Load
        let loaded = Brain::load_or_new(path);
        
        // Verify semantic beliefs survived
        assert!(!loaded.semantic.beliefs.is_empty(), "Beliefs should persist");
        // Check that VolBreakout has a belief (regime may vary based on data)
        let has_volbreakout = loaded.semantic.beliefs.iter()
            .any(|b| b.domain_strategy == "VolBreakout");
        assert!(has_volbreakout, "VolBreakout belief should persist");
        
        // Verify adaptive weights survived
        let w = loaded.adaptive.get_weights("BTC");
        assert!(w.updates > 0, "Adaptive weights should persist");
        
        // Verify peak equity survived
        assert!(loaded.peak_equity.contains_key("BTC"), "Peak equity should persist");
        
        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_brain_obi_integration() {
        use crate::orderbook_imbalance::{OrderBookSnapshot, BookLevel};

        let mut brain = Brain::new();

        // Feed bullish orderbook data into OBI engine
        let book = OrderBookSnapshot {
            symbol: "BTCUSDT".to_string(),
            bids: vec![
                BookLevel { price: 100.0, quantity: 50.0 },
                BookLevel { price: 99.0, quantity: 40.0 },
            ],
            asks: vec![
                BookLevel { price: 101.0, quantity: 5.0 },
                BookLevel { price: 102.0, quantity: 5.0 },
            ],
            timestamp_ms: 1000,
        };
        brain.obi_engine.analyze(&book);

        // Now process trade — should pick up OBI data
        let entity = make_entity_with_history("BTCUSDT", &[10.0, -5.0, 8.0]);
        let diag = brain.process_trade(&entity, "Strat");

        assert!(diag.obi_ratio > 0.3, "OBI should detect bullish pressure: {}", diag.obi_ratio);
        assert_eq!(diag.obi_bias, "bullish");
        assert!(diag.obi_confidence > 0.0);
    }

    #[test]
    fn test_brain_obi_neutral_without_data() {
        let mut brain = Brain::new();
        let entity = make_entity_with_history("ETH", &[5.0]);
        let diag = brain.process_trade(&entity, "Strat");

        // No orderbook data fed → OBI should be neutral
        assert_eq!(diag.obi_ratio, 0.0);
        assert_eq!(diag.obi_bias, "neutral");
        assert_eq!(diag.obi_confidence, 0.0);
    }
}
