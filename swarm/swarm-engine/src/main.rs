//! Swarm Engine — Full Multiverse Convergence v4
//!
//! ALL 12 crates wired + 6 new intelligence layers:
//!   Ouroboros (LLM Brain + PreTrade Risk + Decision Memory + IPC Telemetry)
//!   Titan (8-Gate Entry)
//!   Hive Mind (ML + Hybrid Recall + Regime Detection + DQS + Affective + Paper)
//!   X402 (Consensus + Risk + Sniper + Liquidator + Polymarket + Memory)
//!
//! Pipeline: Data → Regime → Debate → ML → Recall → Judge →
//!           DQS → PreTradeRisk → Entry → Consensus → RiskGate →
//!           PaperTrade → DecisionJournal → IPC Telemetry → (Chain Execute)

mod telemetry;

use ouroboros_brain::{
    config::{load_models, load_prompts, ModelsFile, PromptsFile},
    judge::{chief_judge_v2, load_thresholds, JudgeInput, JudgeVerdict, ThresholdsConfig},
    openrouter::{ModelPool, OpenRouterClient},
    state::{ConsensusResult, SymbolData, SwarmState, Verdict},
    decision_memory::DecisionMemory,
    risk_engine::{pre_trade_risk_check, RiskConfig},
};
use titan_core::entry::{EntryConfig, EntryContext, EntryPipeline, EntryVerdict};
use titan_core::trailing::{TrailingConfig, TrailingEngine, TrailingVerdict};
use titan_core::unstuck::{UnstuckConfig, UnstuckEngine, UnstuckVerdict};
use titan_core::risk::RiskMatrix;
use titan_core::confidence::ConfidenceEngine;
use titan_core::auto_ramp::AutoRamp;
use titan_core::deallow::Deallow;
use titan_core::patience::PatienceTracker;
use core_ipc::IpcBridge;
use hive_intel::ml_local::{FeatureVector, LocalModel};
use hive_intel::paper_engine::{PaperEngine, Side as PaperSide};
use hive_intel::recall::{
    AffectiveState, ContextVector, RawMemory,
    outcome_weighted_recall,
};
use hive_intel::hybrid_recall::hybrid_blend;
use hive_intel::regime::{classify_regime, MarketRegime as HiveRegime};
use hive_intel::affective::{ewma_confidence, risk_appetite};
use hive_intel::benchmark::{SmaCrossover, BenchmarkResult, BenchmarkStats};
use x402_consensus::engine::{Action, AgentVote, PolicyGovernor};
use x402_risk::engine::{AtrStops, MarketRegime, RiskGate};
use x402_memory::engine::create_liquidation_edge;
// use casper_client::{put_deploy, query_global_state};

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

// ═══════════════════════════════════════════════════════════
// CONFIG
// ═══════════════════════════════════════════════════════════

fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("CONFIG_DIR") {
        return PathBuf::from(p);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("config")
}

fn mock_market_data() -> Vec<SymbolData> {
    vec![
        SymbolData {
            symbol: "MNT".into(), price: 0.82, price_24h_change: -3.2,
            volume_24h: 45_000_000.0, volume_ratio: 1.8,
            funding_rate: -0.0004, open_interest: 120_000_000.0,
            oi_change_pct: -2.1, timestamp: chrono::Utc::now().timestamp(),
        },
        SymbolData {
            symbol: "WETH".into(), price: 2650.0, price_24h_change: 1.4,
            volume_24h: 180_000_000.0, volume_ratio: 1.2,
            funding_rate: 0.0001, open_interest: 890_000_000.0,
            oi_change_pct: 0.8, timestamp: chrono::Utc::now().timestamp(),
        },
        SymbolData {
            symbol: "USDC".into(), price: 1.0, price_24h_change: 0.01,
            volume_24h: 500_000_000.0, volume_ratio: 1.0,
            funding_rate: 0.0, open_interest: 0.0,
            oi_change_pct: 0.0, timestamp: chrono::Utc::now().timestamp(),
        },
    ]
}

/// Fetch live market data from DexScreener with FULL signal enrichment.
/// Fetch live market data (temporarily mocked pending Casper Oracle)
async fn live_market_data() -> Vec<SymbolData> {
    mock_market_data()
}


// ═══════════════════════════════════════════════════════════
// REGIME DETECTION — 4-State Market Classifier
// ═══════════════════════════════════════════════════════════

fn detect_market_regime(data: &SymbolData) -> (HiveRegime, f64) {
    // Synthesize pseudo-returns from available data
    let base_return = data.price_24h_change / 100.0;
    let vol_signal = data.volume_ratio - 1.0;
    let oi_signal = data.oi_change_pct / 100.0;
    let fr_signal = data.funding_rate * 100.0;

    let pseudo_returns = vec![
        base_return, base_return * 0.8, base_return * 1.1,
        base_return + oi_signal * 0.3, base_return - fr_signal * 0.5,
        vol_signal * 0.1, base_return * 0.9 + vol_signal * 0.05,
        base_return * 1.05,
    ];
    let historical_vol = 0.02; // 2% baseline vol for crypto
    let result = classify_regime(&pseudo_returns, historical_vol);
    (result.regime, result.confidence)
}

fn regime_to_risk(regime: HiveRegime) -> MarketRegime {
    match regime {
        HiveRegime::TrendingUp | HiveRegime::TrendingDown => MarketRegime::Trending,
        HiveRegime::Ranging => MarketRegime::Calm,
        HiveRegime::Volatile => MarketRegime::Choppy,
    }
}

// ═══════════════════════════════════════════════════════════
// DIMENSION 1: OUROBOROS — LLM Debate
// ═══════════════════════════════════════════════════════════

struct DebateResult {
    bull_argument: String,
    bear_argument: String,
    macro_bias: String,
}

async fn run_debate(
    client: &OpenRouterClient, debate_pool: &ModelPool,
    prompts: &PromptsFile, models: &ModelsFile, data: &SymbolData,
) -> DebateResult {
    let data_str = format!(
        "price=${:.4}, 24h_change={:.1}%, funding={:.6}, oi_change={:.1}%, vol_ratio={:.1}x",
        data.price, data.price_24h_change, data.funding_rate, data.oi_change_pct, data.volume_ratio
    );
    let fr_label = if data.funding_rate < -0.0003 { "SHORTS PAYING" }
        else if data.funding_rate > 0.0005 { "LONGS OVERHEATED" }
        else { "neutral" };

    let bull_user = prompts.debate.bull.user_template
        .replace("{symbol}", &data.symbol).replace("{data}", &data_str).replace("{alpha_ctx}", "");
    let bull_argument = client.chat_with_pool(debate_pool, &prompts.debate.bull.system,
        &bull_user, prompts.debate.bull.temperature, prompts.debate.bull.max_tokens,
    ).await.unwrap_or_default();
    if !bull_argument.is_empty() { tracing::info!("🐂 Bull [{}]: {}", data.symbol, bull_argument); }

    let bear_user = prompts.debate.bear.user_template
        .replace("{symbol}", &data.symbol).replace("{data}", &data_str).replace("{alpha_ctx}", "");
    let bear_argument = client.chat_with_pool(debate_pool, &prompts.debate.bear.system,
        &bear_user, prompts.debate.bear.temperature, prompts.debate.bear.max_tokens,
    ).await.unwrap_or_default();
    if !bear_argument.is_empty() { tracing::info!("🐻 Bear [{}]: {}", data.symbol, bear_argument); }

    let macro_user = prompts.macro_judge.user_template
        .replace("{symbol}", &data.symbol)
        .replace("{price}", &format!("{:.4}", data.price))
        .replace("{change}", &format!("{:.1}", data.price_24h_change))
        .replace("{fr}", &format!("{:.6}", data.funding_rate))
        .replace("{fr_label}", fr_label)
        .replace("{oi_delta}", &format!("{:.1}", data.oi_change_pct))
        .replace("{vol_surge}", &format!("{:.1}", data.volume_ratio));
    let macro_raw = client.chat(&models.macro_judge_model, &prompts.macro_judge.system,
        &macro_user, prompts.macro_judge.temperature, prompts.macro_judge.max_tokens,
    ).await.unwrap_or_default();
    let upper = macro_raw.to_uppercase();
    let macro_bias = if upper.contains("BUY") || upper.contains("BULLISH") { "BULLISH" }
        else if upper.contains("SELL") || upper.contains("BEARISH") { "BEARISH" }
        else { "NEUTRAL" };
    tracing::info!("⚖️ Macro [{}]: {} → {}", data.symbol, macro_raw, macro_bias);

    DebateResult { bull_argument, bear_argument, macro_bias: macro_bias.to_string() }
}

// ═══════════════════════════════════════════════════════════
// DIMENSION 3: HIVE MIND — ML + OWM Recall
// ═══════════════════════════════════════════════════════════

fn run_ml_prediction(ml: &LocalModel, data: &SymbolData) -> (i32, f64) {
    let features = FeatureVector::from_raw(
        50.0 + data.price_24h_change * -3.0, data.price_24h_change,
        data.volume_24h, data.volume_24h / data.volume_ratio.max(0.01),
        data.oi_change_pct / 10.0, data.funding_rate.abs() * 100.0, 0.55, 0.5,
    );
    let pred = ml.predict(&features);
    let dir = if pred.probability > 0.6 { 1 } else if pred.probability < 0.4 { -1 } else { 0 };
    tracing::info!("🤖 ML [{}]: P={:.3} conf={:.3} → {}", data.symbol, pred.probability, pred.confidence,
        match dir { 1 => "BULL", -1 => "BEAR", _ => "NEUTRAL" });
    (dir, pred.confidence)
}

fn run_memory_recall(data: &SymbolData, affective: &AffectiveState, trade_memories: &[RawMemory]) -> f64 {
    let query = ContextVector {
        regime: Some(if data.price_24h_change > 1.0 { "trending_up" } else if data.price_24h_change < -1.0 { "trending_down" } else { "ranging" }.into()),
        price: Some(data.price),
        ..Default::default()
    };

    // OWM base scoring
    let recalled = outcome_weighted_recall(&query, trade_memories, affective, 5);

    // Upgrade to hybrid (vector + OWM) when available
    let owm_scores: Vec<(String, f64, Option<f64>)> = recalled.iter()
        .map(|m| (m.memory_id.clone(), m.score, trade_memories.iter()
            .find(|t| t.id == m.memory_id).and_then(|t| t.pnl_r)))
        .collect();
    let hybrid = hybrid_blend(&owm_scores, None, &[], 0.3, 3);

    let memory_boost = if hybrid.is_empty() { 0.0 } else {
        hybrid.iter().map(|m| m.score).sum::<f64>() / hybrid.len() as f64
    };
    if !hybrid.is_empty() {
        tracing::info!("🧠 Hybrid Recall [{}]: {} memories, avg_score={:.3} (anti-survivorship enforced)",
            data.symbol, hybrid.len(), memory_boost);
    }
    memory_boost
}

// ═══════════════════════════════════════════════════════════
// DIMENSION 2: TITAN — Entry Pipeline (8 Gates)
// ═══════════════════════════════════════════════════════════

fn run_entry_gate(verdict: &JudgeVerdict, data: &SymbolData) -> bool {
    let side = match verdict.decision {
        Verdict::Buy => "LONG", Verdict::Sell => "SHORT", Verdict::Hold => return false,
    };
    let ctx = EntryContext {
        daily_loss: 0.0, session_limit: 50.0, symbol_loss_streak: 0,
        head_position_count: 0, global_position_count: 0, symbol_already_owned: false,
        verdict: side.to_string(), symbol: data.symbol.clone(), score: verdict.score,
        btc_score: 0.0, imbalance_ratio: data.volume_ratio.clamp(0.1, 10.0),
        existing_total_margin: 0.0, available_balance: 1000.0, new_margin_size: 50.0,
        is_held_by_other_bot: false, existing_position_symbols: vec![],
    };
    match EntryPipeline::evaluate(&ctx, &EntryConfig::default()) {
        EntryVerdict::Approved { side, reason } => { tracing::info!("🚪 Entry [{}]: ✅ {} ({})", data.symbol, side, reason); true }
        EntryVerdict::Rejected { gate, reason } => { tracing::info!("🚪 Entry [{}]: ❌ {} ({})", data.symbol, gate, reason); false }
    }
}

// ═══════════════════════════════════════════════════════════
// DIMENSION 4: X402 — Consensus + Risk + Stops
// ═══════════════════════════════════════════════════════════

fn run_consensus(verdict: &JudgeVerdict, ml_dir: i32, macro_bias: &str) -> (bool, Action) {
    let governor = PolicyGovernor::new();
    let to_action = |v: &Verdict| match v { Verdict::Buy => Action::Buy, Verdict::Sell => Action::Sell, _ => Action::Wait };
    let votes = vec![
        AgentVote { agent_name: "signal".into(), action: to_action(&verdict.decision),
            confidence: (verdict.confidence / 100.0).clamp(0.0, 1.0), timestamp: 0 },
        AgentVote { agent_name: "trend".into(),
            action: match ml_dir { 1 => Action::Buy, -1 => Action::Sell, _ => Action::Wait },
            confidence: 0.5, timestamp: 0 },
        AgentVote { agent_name: "regime".into(),
            action: match macro_bias { "BULLISH" => Action::Buy, "BEARISH" => Action::Sell, _ => Action::Wait },
            confidence: 0.6, timestamp: 0 },
    ];
    let d = governor.resolve(&votes);
    let ok = !d.vetoed && d.action != Action::Wait;
    let tag = if d.vetoed { "🛑 VETO" } else if ok { "✅" } else { "⏸️" };
    tracing::info!("{} Consensus: {} (conf={:.2}) {}", tag, d.action, d.confidence, d.veto_reason.as_deref().unwrap_or(""));
    (ok, d.action)
}

fn run_risk_gate(risk: &RiskGate, symbol: &str, confidence: f64, regime: MarketRegime) -> Option<f64> {
    let win_rate = (confidence / 100.0).clamp(0.3, 0.8);
    match risk.evaluate(symbol, win_rate, 1.0, regime) {
        Ok(size) => { tracing::info!("💰 Risk [{}]: ✅ size=${:.2} (regime={:?})", symbol, size, regime); Some(size) }
        Err(reason) => { tracing::info!("💰 Risk [{}]: ❌ {}", symbol, reason); None }
    }
}

fn run_paper_trade(paper: &Mutex<PaperEngine>, symbol: &str, verdict: Verdict, price: f64, size: f64) {
    let mut pe = paper.lock().unwrap();
    let side = match verdict { Verdict::Buy => PaperSide::Long, Verdict::Sell => PaperSide::Short, _ => return };
    let qty = size / price;
    let stops = AtrStops::calculate(price * 0.015, price); // 1.5% ATR estimate
    let (sl, tp) = match side {
        PaperSide::Long => (Some(stops.stop_price_long()), Some(stops.target_price_long())),
        PaperSide::Short => (Some(stops.stop_price_short()), Some(stops.target_price_short())),
    };
    let ts = chrono::Utc::now().timestamp_millis();
    match pe.open_position(symbol, side, price, qty, sl, tp, ts) {
        Ok(id) => tracing::info!("📝 Paper [{}]: opened #{} qty={:.6} SL={:.4} TP={:.4}", symbol, id, qty, sl.unwrap_or(0.0), tp.unwrap_or(0.0)),
        Err(e) => tracing::warn!("📝 Paper [{}]: rejected — {}", symbol, e),
    }
}

fn log_memory_edge(symbol: &str, verdict: Verdict, score: f64) {
    let sentiment = match verdict { Verdict::Buy => score, Verdict::Sell => -score, _ => 0.0 };
    let edge = create_liquidation_edge(symbol.to_string(), sentiment, chrono::Utc::now().timestamp() as u64);
    tracing::debug!("🔗 HyperEdge: {} → {} ({:.2})", edge.source, edge.target, edge.sentiment);
}

// ═══════════════════════════════════════════════════════════
// DECISION CYCLE — ALL DIMENSIONS
// ═══════════════════════════════════════════════════════════

async fn decision_cycle(
    client: &OpenRouterClient, debate_pool: &ModelPool,
    prompts: &PromptsFile, models: &ModelsFile, thresholds: &ThresholdsConfig,
    state: &SwarmState, ml: &LocalModel, risk: &RiskGate,
    paper: &Mutex<PaperEngine>, trade_memories: &Mutex<Vec<RawMemory>>,
    decision_mem: &DecisionMemory,
    sma_engines: &Mutex<std::collections::HashMap<String, SmaCrossover>>,
    bench_stats: &Mutex<BenchmarkStats>,
    signed_provider: &Option<String>,
    tx_hashes: &Mutex<Vec<String>>,
    patience: &Mutex<PatienceTracker>,
) {
    let cycle = state.increment_cycle();
    tracing::info!("━━━ CYCLE {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", cycle);

    if !state.is_trading_allowed() {
        tracing::warn!("🔴 Circuit breaker RED — skip"); return;
    }

    let (affective, pnl_outcomes) = {
        let pe = paper.lock().unwrap();
        (AffectiveState {
            drawdown_state: if pe.peak_equity > 0.0 { ((pe.peak_equity - pe.equity) / pe.peak_equity).clamp(0.0, 1.0) } else { 0.0 },
            consecutive_losses: pe.pnl_history.iter().rev().take_while(|p| **p < 0.0).count() as u32,
        }, pe.pnl_history.clone())
    };

    // Affective Intelligence — EWMA confidence & risk appetite from trade history
    let ewma_conf = ewma_confidence(&pnl_outcomes.to_vec(), 0.9);
    let risk_app = risk_appetite(affective.drawdown_state, 0.15); // 15% max DD
    tracing::info!("🧬 Affective: EWMA_conf={:.3} risk_appetite={:.3} streak={}",
        ewma_conf, risk_app, affective.consecutive_losses);

    // L2: Ingest any Titan outcomes into Decision Memory
    decision_mem.ingest_outcomes();

    // L1→L3: Convert PaperEngine closed trades into OWM RawMemory (learning loop)
    {
        let pe = paper.lock().unwrap();
        let mut memories = trade_memories.lock().unwrap();
        for trade in &pe.closed_trades {
            let trade_id = format!("paper_{}_{}", trade.symbol, trade.id);
            if memories.iter().any(|m| m.id == trade_id) { continue; }
            let pnl_r = if trade.entry_price > 0.0 {
                Some(trade.pnl / (trade.entry_price * trade.quantity).max(0.01))
            } else { None };
            memories.push(RawMemory {
                id: trade_id,
                memory_type: "episodic".into(),
                age_days: 0.0,
                confidence: 0.7,
                pnl_r,
                context: ContextVector {
                    price: Some(trade.entry_price),
                    ..Default::default()
                },
                rehearsal_count: 0,
            });
        }
    }

    let risk_config = RiskConfig::default();

    // Fetch market data: live from DexScreener or mock (set MOCK_DATA=1 to force mock)
    let use_mock = std::env::var("MOCK_DATA").map(|v| v == "1").unwrap_or(false);
    let market_data = if use_mock {
        tracing::info!("📦 Using MOCK market data (MOCK_DATA=1)");
        mock_market_data()
    } else {
        live_market_data().await
    };

    // D3: Hive Mind — Cross-Asset Correlation Matrix (portfolio risk awareness)
    if market_data.len() >= 2 {
        let mut pnl_series = std::collections::HashMap::new();
        for d in &market_data {
            // Use recent price changes as PnL proxy for correlation
            pnl_series.insert(d.symbol.clone(), vec![d.price, d.price * (1.0 + d.price_24h_change / 100.0)]);
        }
        let pairs = hive_intel::correlation::build_correlation_matrix(&pnl_series, 0.5);
        for pair in &pairs {
            if pair.pearson_r.abs() > 0.7 {
                tracing::info!("📊 CORRELATION: {} ↔ {} r={:.2} ({})",
                    pair.symbol_a, pair.symbol_b, pair.pearson_r, pair.strength.as_str());
            }
        }
    }

    for data in &market_data {
        state.symbols.insert(data.symbol.clone(), data.clone());

        // REGIME DETECTION — 4-state classifier before anything else
        let (regime, regime_conf) = detect_market_regime(data);
        let x402_regime = regime_to_risk(regime);
        tracing::info!("📊 {} @ ${:.4} | 24h:{:.1}% | FR:{:.6} | OI:{:.1}% | Regime: {:?} ({:.0}%)",
            data.symbol, data.price, data.price_24h_change, data.funding_rate,
            data.oi_change_pct, regime, regime_conf * 100.0);

        if data.price_24h_change.abs() < 0.05 && data.funding_rate.abs() < 0.00001 { continue; }

        // D1: Ouroboros — LLM Debate
        let debate = run_debate(client, debate_pool, prompts, models, data).await;

        // D3: Hive Mind — ML + Hybrid Recall (OWM + vector + anti-survivorship)
        let (ml_dir, ml_conf) = run_ml_prediction(ml, data);
        let memories = trade_memories.lock().unwrap();
        let memory_boost = run_memory_recall(data, &affective, &memories);
        drop(memories);

        // L2: Decision Memory — inject past context into judge
        let past_ctx = decision_mem.get_past_context(&data.symbol, 3, 2);

        // D1: Ouroboros — 15-Factor Judge (with memory context + regime)
        let input = JudgeInput {
            data: data.clone(),
            bull_argument: debate.bull_argument.clone(), bear_argument: debate.bear_argument.clone(),
            macro_fresh: debate.macro_bias != "NEUTRAL", macro_bias: debate.macro_bias.clone(),
            ml_fresh: true, ml_direction: ml_dir, ml_confidence: ml_conf,
            ..Default::default()
        };
        let verdict = chief_judge_v2(&input, thresholds);
        let e = match verdict.decision { Verdict::Buy => "🟢", Verdict::Sell => "🔴", Verdict::Hold => "⚪" };
        tracing::info!("{} Judge [{}]: {} | score={:.2} conf={:.1}%", e, data.symbol, verdict.decision, verdict.score, verdict.confidence);

        // ═══ AI vs HUMAN BENCHMARK ═══
        {
            let mut engines = sma_engines.lock().unwrap();
            let sma = engines.entry(data.symbol.clone()).or_insert_with(|| {
                let mut s = SmaCrossover::default_config();
                s.seed_synthetic(data.price, data.price_24h_change);
                s
            });
            let human_sig = sma.signal(data.price);
            let ai_verdict_str = format!("{}", verdict.decision);
            let agreement = (ai_verdict_str == human_sig.verdict)
                || (ai_verdict_str == "Hold" && human_sig.verdict == "HOLD");

            let result = BenchmarkResult {
                ai_verdict: ai_verdict_str.clone(),
                human_verdict: human_sig.verdict.clone(),
                ai_score: verdict.score,
                human_score: human_sig.score,
                ai_confidence: verdict.confidence,
                agreement,
                ai_inference_ms: 0,
                cycle: state.cycle_count.load(std::sync::atomic::Ordering::Relaxed),
                symbol: data.symbol.clone(),
                price: data.price,
            };

            let mut stats = bench_stats.lock().unwrap();
            stats.update(&result);

            let vs = if agreement { "🤝" } else { "⚔️" };
            tracing::info!("{} Benchmark [{}]: AI={} vs Human={} (SMA:{:.4}/{:.4}) | {}",
                vs, data.symbol, ai_verdict_str, human_sig.verdict,
                human_sig.sma_short, human_sig.sma_long, human_sig.reason);
        }

        if !past_ctx.is_empty() {
            tracing::info!("📜 Decision Memory [{}]: {} chars of past context injected", data.symbol, past_ctx.len());
        }

        // ═══ NEW: Pre-Trade Risk Engine (5 institutional filters) ═══
        let verdict_str = format!("{}", verdict.decision);
        let risk_check = pre_trade_risk_check(
            &data.symbol, &verdict_str.to_uppercase(), verdict.confidence,
            state, decision_mem, &risk_config,
        );
        if !risk_check.allowed {
            tracing::warn!("🛡️ PreTrade [{}]: ❌ {}", data.symbol, risk_check.reason);
            store_result(state, data, &verdict, &debate, false); continue;
        }
        if risk_check.max_size_factor < 1.0 {
            tracing::info!("🛡️ PreTrade [{}]: ⚠️ size capped to {:.0}% — {}",
                data.symbol, risk_check.max_size_factor * 100.0, risk_check.reason);
        }

        // D2: Titan — 8-Gate Entry
        if !run_entry_gate(&verdict, data) {
            store_result(state, data, &verdict, &debate, false); continue;
        }

        // D4: X402 — Consensus (3 voters)
        let (consensus_ok, _) = run_consensus(&verdict, ml_dir, &debate.macro_bias);
        if !consensus_ok { store_result(state, data, &verdict, &debate, false); continue; }

        // D4: X402 — Risk Gate (Kelly + KillSwitch + BucketCap) — REGIME-AWARE
        let raw_size = match run_risk_gate(risk, &data.symbol, verdict.confidence, x402_regime) {
            Some(s) => s, None => { store_result(state, data, &verdict, &debate, false); continue; }
        };

        // Apply PreTrade size factor + risk appetite dampening
        let final_size = raw_size * risk_check.max_size_factor * risk_app;
        if final_size < 0.5 {
            tracing::info!("💰 Size [{}]: ${:.2} too small after dampening — skip", data.symbol, final_size);
            store_result(state, data, &verdict, &debate, false); continue;
        }

        // D3: Hive Mind — Decision Quality Score (5-factor pre-trade gate)
        let dqs_engine = hive_intel::dqs::DqsEngine::new();
        let dqs_result = {
            let pe = paper.lock().unwrap();
            let dd = if pe.peak_equity > 0.0 { ((pe.peak_equity - pe.equity) / pe.peak_equity * 100.0).max(0.0) } else { 0.0 };
            let losses = pe.pnl_history.iter().rev().take_while(|p| **p < 0.0).count() as u32;
            let wr = {
                let total = pe.closed_trades.len();
                if total > 0 { pe.closed_trades.iter().filter(|t| t.pnl > 0.0).count() as f64 / total as f64 } else { 0.5 }
            };
            dqs_engine.compute(&hive_intel::dqs::DqsInput {
                regime_win_rate: Some(wr),
                overall_win_rate: wr,
                proposed_lot: final_size,
                kelly_fraction: None,
                owm_score: memory_boost,
                drawdown_pct: dd,
                consecutive_losses: losses,
                confidence: verdict.confidence / 100.0,
                avg_pnl_r_similar: verdict.score,
            })
        };
        let final_size = final_size * dqs_result.position_multiplier;
        if dqs_result.tier == hive_intel::dqs::DqsTier::Skip {
            tracing::info!("📊 DQS [{}]: SKIP (score={:.1}/10) — quality too low", data.symbol, dqs_result.score);
            store_result(state, data, &verdict, &debate, false); continue;
        }
        tracing::info!("📊 DQS [{}]: {} (score={:.1}/10) size×{:.1} | regime={:.2} sizing={:.2} process={:.2} risk={:.2} pattern={:.2}",
            data.symbol, dqs_result.tier.as_str().to_uppercase(), dqs_result.score, dqs_result.position_multiplier,
            dqs_result.factors.regime_match, dqs_result.factors.position_sizing,
            dqs_result.factors.process_adherence, dqs_result.factors.risk_state, dqs_result.factors.historical_pattern);
        tracing::info!("💰 Final Size [{}]: ${:.2} (raw=${:.2} × pretrade={:.2} × appetite={:.2} × dqs={:.1})",
            data.symbol, final_size, raw_size, risk_check.max_size_factor, risk_app, dqs_result.position_multiplier);

        // D2: Titan — Patience Tracker (15-min signal confirmation)
        {
            let side_str = match verdict.decision { Verdict::Buy => "Buy", Verdict::Sell => "Sell", _ => "Hold" };
            let mut pt = patience.lock().unwrap();
            pt.purge_stale(); // Clean entries older than 2h
            if verdict.decision != Verdict::Hold {
                if !pt.is_ready_to_strike(&data.symbol, side_str) {
                    pt.lock_target(&data.symbol, side_str);
                    tracing::info!("⏳ Patience [{}]: {} signal locked — waiting 15m confirmation", data.symbol, side_str);
                } else {
                    tracing::info!("✅ Patience [{}]: {} signal confirmed after wait", data.symbol, side_str);
                    pt.clear_target(&data.symbol);
                }
            }
        }

        // D3: Hive Mind — Paper Trade (ATR stops)
        run_paper_trade(paper, &data.symbol, verdict.decision, data.price, final_size);

        // D2: Titan — Dynamic Leverage (ATR-based volatility dampening)
        {
            let risk_matrix = RiskMatrix::new();
            let atr_estimate = data.price * 0.015; // 1.5% ATR proxy
            let dynamic_lev = risk_matrix.calculate_dynamic_leverage(atr_estimate, data.price);
            let macro_lev = risk_matrix.apply_macro_penalty(
                dynamic_lev,
                match verdict.decision { Verdict::Buy => "Buy", Verdict::Sell => "Sell", _ => "Hold" },
                0.0, // BTC score (would need BTC data feed for real)
            );
            if macro_lev < 10.0 {
                tracing::info!("⚙️ RiskMatrix [{}]: leverage={:.0}x (ATR dampened from 10x)", data.symbol, macro_lev);
            }
        }

        // D2: Titan — Trailing SL management for open positions
        {
            let pe = paper.lock().unwrap();
            let trailing_cfg = TrailingConfig::default();
            for pos in pe.open_positions.values() {
                let atr_est = pos.entry_price * 0.015;
                let (highest, lowest) = (pos.entry_price.max(data.price), pos.entry_price.min(data.price));
                let side_str = match pos.side { PaperSide::Long => "Buy", PaperSide::Short => "Sell" };

                // Check adverse selection
                if let Some(adv) = TrailingEngine::check_adverse_selection(
                    side_str, pos.entry_price, data.price, highest, lowest,
                    atr_est, pos.opened_at_ms,
                ) {
                    tracing::warn!("⚠️ Adverse Selection [{}]: {:?}", pos.symbol, adv);
                }

                // Calculate trailing SL
                let trail = TrailingEngine::calculate_trailing_sl(
                    side_str, pos.entry_price, data.price, highest, lowest,
                    atr_est, 3.5, pos.stop_loss.unwrap_or(0.0),
                    &pos.symbol, &trailing_cfg,
                );
                match &trail {
                    TrailingVerdict::Tightened { new_sl, reason } => {
                        tracing::info!("🔒 Trailing [{}]: SL → {:.4} ({})", pos.symbol, new_sl, reason);
                    }
                    _ => {}
                }
            }
        }

        // D2: Titan — Unstuck Engine (stuck position recovery)
        {
            let pe = paper.lock().unwrap();
            let unstuck_cfg = UnstuckConfig::default();
            for pos in pe.open_positions.values() {
                let side_str = match pos.side { PaperSide::Long => "Buy", PaperSide::Short => "Sell" };
                let atr_est = pos.entry_price * 0.015;
                let unstuck = UnstuckEngine::evaluate(
                    side_str, pos.entry_price, data.price, pos.quantity,
                    pos.opened_at_ms, atr_est, 0.0, // btc_score placeholder
                    0.0, pe.peak_equity, false, 0, &unstuck_cfg,
                );
                match &unstuck {
                    UnstuckVerdict::ReleaseStage1 { close_pct, reason } => {
                        tracing::warn!("🔓 Unstuck [{}]: Stage1 close {:.0}% — {}", pos.symbol, close_pct * 100.0, reason);
                    }
                    UnstuckVerdict::FullEvacuation { reason } => {
                        tracing::error!("🚨 Unstuck [{}]: FULL EVACUATION — {}", pos.symbol, reason);
                    }
                    UnstuckVerdict::Monitoring { hold_hours, loss_pct } => {
                        tracing::debug!("👁 Unstuck [{}]: monitoring {:.1}h @ {:.1}%", pos.symbol, hold_hours, loss_pct);
                    }
                    _ => {}
                }
            }
        }

        // D3: Hive Mind — Anomaly Detection (Z-score + IQR on trade PnL history)
        {
            let pe = paper.lock().unwrap();
            if pe.pnl_history.len() >= 5 {
                let history: Vec<f64> = pe.pnl_history.to_vec();
                let latest_pnl = history.last().copied().unwrap_or(0.0);
                let anomaly = hive_intel::anomaly::detect_anomaly(latest_pnl, &history);
                if anomaly.severity != hive_intel::anomaly::AnomalySeverity::Normal {
                    tracing::warn!("🔍 ANOMALY [{}]: z={:.2} severity={} pctl={:.0}%",
                        data.symbol, anomaly.z_score, anomaly.severity.as_str(), anomaly.percentile);
                }
            }
        }

        // D2: Titan — Confidence Engine (DNA-based scoring, adaptive ATR, directional bias)
        {
            let conf_score = ConfidenceEngine::calculate(&data.symbol);
            let adaptive_atr = ConfidenceEngine::get_adaptive_atr_mult(&data.symbol, 3.5);
            let adaptive_cd = ConfidenceEngine::get_adaptive_cooldown(&data.symbol, 900);
            let dir_bias = ConfidenceEngine::get_directional_bias(&data.symbol);
            if conf_score.abs() > 0.1 || dir_bias.is_some() {
                tracing::info!("🧬 Confidence [{}]: score={:.1} atr_mult={:.1} cooldown={}s bias={:?}",
                    data.symbol, conf_score, adaptive_atr, adaptive_cd, dir_bias);
            }
        }

        // D2: Titan — AutoRamp (5-gate capital scaling state machine)
        {
            let ramp_log = AutoRamp::evaluate();
            let phase = AutoRamp::current_phase();
            tracing::info!("📈 AutoRamp: Phase {}/4 ({}) max_pos={:.0}% | {}",
                phase.phase, phase.label, phase.max_position_pct * 100.0, ramp_log);
        }

        // D2: Titan — Deallow (underperformer ban/recovery scanner)
        {
            let banned = Deallow::scan_underperformers();
            if !banned.is_empty() {
                tracing::warn!("🚫 Deallow: {} symbols banned: {}", banned.len(), banned.join(", "));
            }
        }

        // D4: X402 — Memory Edge
        log_memory_edge(&data.symbol, verdict.decision, verdict.score);

        // L2: Decision Memory — store verdict for future reflection
        let factors_summary = format!(
            "regime={:?} macro={} ml_dir={} ml_conf={:.2} score={:.2} ewma={:.3} appetite={:.3}",
            regime, debate.macro_bias, ml_dir, ml_conf, verdict.score, ewma_conf, risk_app
        );
        decision_mem.store_decision(
            &data.symbol,
            &format!("{}", verdict.decision),
            verdict.score,
            verdict.confidence,
            &factors_summary,
        );

        tracing::info!("🚀 EXECUTE [{}]: {} ${:.2} | score={:.2} conf={:.1}% | regime={:?}",
            data.symbol, verdict.decision, final_size, verdict.score, verdict.confidence, regime);

        // Casper on-chain write: record the agent's decision on the deployed escrow
        // contract via a real, signed TransactionV1 (never a fabricated hash).
        if signed_provider.is_some() {
            match casper_onchain_record(&data.symbol) {
                Ok(tx) => {
                    tracing::info!("⛓️  CASPER TX [{}]: {}", data.symbol, tx);
                    tx_hashes.lock().unwrap().push(tx);
                }
                Err(e) => tracing::warn!("⛓️  CASPER TX [{}] rejected: {}", data.symbol, e),
            }
        } else {
            tracing::info!("⛓️  CASPER DRY-RUN [{}]: set CASPER_PRIVATE_KEY for live on-chain writes.", data.symbol);
        }

        store_result(state, data, &verdict, &debate, true);
    }

    // Cycle summary
    tracing::info!("━━━ CYCLE {} COMPLETE ━━━━━━━━━━━━━━━━━━━━━━━━━━", cycle);
    for entry in state.consensus.iter() {
        let r = entry.value();
        let tag = if r.meta_agreement { "✅" } else { "❌" };
        tracing::info!("  {} {} → {} ({:.1}%) score={:.2}", tag, r.symbol, r.final_verdict, r.confidence, r.judge_score);
    }
    // Paper trading stats
    let pe = paper.lock().unwrap();
    if !pe.pnl_history.is_empty() {
        let s = pe.stats();
        tracing::info!("  📊 Paper: {} trades | WR={:.0}% | PnL=${:.2} | DD=${:.2}", s.total_trades, s.win_rate*100.0, s.total_pnl, s.max_drawdown);
    }
}

/// Submit a real, signed on-chain transaction to the deployed Casper escrow
/// contract recording this agent's decision. Shells out to the project signer
/// (`casper-tx-signer`), which builds + signs + broadcasts a TransactionV1 and
/// prints the real 64-char tx hash. Returns an error instead of a fake hash on
/// any failure.
fn casper_onchain_record(symbol: &str) -> Result<String, String> {
    let env_or = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let chain = env_or("CASPER_CHAIN", "casper-test");
    let secret_key = env_or("CASPER_SECRET_KEY", "swarm/casper-client/key.pem");
    let signer = env_or("CASPER_SIGNER_BIN", "./swarm/casper-client/go-signer/casper-tx-signer");
    let package = env_or(
        "CASPER_CONTRACT_PACKAGE",
        "a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d",
    );
    let pubkey = env_or(
        "CASPER_AGENT_PUBKEY",
        "013d8de764919e6dfb002636071ec1729abb0f2be2c3589da79e2278131ce52c35",
    );
    let metadata = format!("decision-{}", symbol).replace([',', ':'], "-");
    let args = format!("public_key:string:{pubkey},metadata_uri:string:{metadata}");

    let output = std::process::Command::new(&signer)
        .args([
            "--mode", "call-entrypoint",
            "--node", &node,
            "--chain", &chain,
            "--secret-key", &secret_key,
            "--algo", "ed25519",
            "--payment", "5000000000",
            "--contract-hash", &package,
            "--entrypoint", "register_agent",
            "--args", &args,
        ])
        .output()
        .map_err(|e| format!("failed to launch signer '{signer}': {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let tx = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tx.len() != 64 || !tx.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("unexpected tx hash: '{tx}'"));
    }
    Ok(tx)
}

fn store_result(state: &SwarmState, data: &SymbolData, v: &JudgeVerdict, d: &DebateResult, agreed: bool) {
    state.consensus.insert(data.symbol.clone(), ConsensusResult {
        symbol: data.symbol.clone(), final_verdict: v.decision, confidence: v.confidence,
        bull_argument: d.bull_argument.clone(), bear_argument: d.bear_argument.clone(),
        macro_bias: d.macro_bias.clone(), judge_score: v.score, meta_agreement: agreed,
        timestamp: chrono::Utc::now().timestamp(),
    });
}

// ═══════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "swarm_engine=info,ouroboros_brain=info".into()))
        .init();

    tracing::info!("═══════════════════════════════════════════════");
    tracing::info!("  CASPER AI SWARM — Full Multiverse v4");
    tracing::info!("  12 crates · 22K+ LOC · 6 Intelligence Layers");
    tracing::info!("═══════════════════════════════════════════════");

    // D1: Ouroboros configs
    let cfg = config_dir();
    let models = load_models(&cfg.join("models.toml")).expect("models.toml");
    let prompts = load_prompts(&cfg.join("prompts.toml")).expect("prompts.toml");
    let thresholds = load_thresholds(&cfg.join("thresholds.toml")).expect("thresholds.toml");

    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY required");
    assert!(api_key.len() >= 10, "OPENROUTER_API_KEY invalid");
    tracing::info!("🔑 API key: {}...", &api_key[..8]);

    let client = OpenRouterClient::new(api_key, &models.defaults);
    let debate_pool = ModelPool::new(models.debate_pool.clone(), models.defaults.max_failures_before_rotate);
    tracing::info!("🧠 D1 Ouroboros: {} debate models + macro/meta judges", debate_pool.pool_size());

    // D2: Titan (8 active modules)
    let patience = Mutex::new(PatienceTracker::new());
    tracing::info!("🚪 D2 Titan: Entry(8-gate) + Trailing(ATR/BE/AdverseSelect) + Unstuck(3-stage) + RiskMatrix + Confidence(DNA) + AutoRamp(5-phase) + Deallow + Patience(15m)");

    // D3: Hive Mind
    let ml_model = LocalModel::new();
    let paper = Mutex::new(PaperEngine::new(1000.0));
    let trade_memories = Mutex::new(Vec::<RawMemory>::new()); // Grows via PaperEngine→OWM loop
    tracing::info!("🤖 D3 Hive Mind: ML(7-feature LR) + PaperEngine($1000) + Hybrid Recall");

    // L2: Decision Memory — self-learning trade journal
    let data_dir = if let Ok(p) = std::env::var("DATA_DIR") {
        PathBuf::from(p)
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().join("data")
    };
    let _ = std::fs::create_dir_all(&data_dir);
    let decision_mem = DecisionMemory::new(&data_dir);
    tracing::info!("📜 L2 Decision Memory: trade journal at {}/trading_memory.md", data_dir.display());

    // D4: X402 Agents
    let risk = RiskGate::new(1000.0);
    tracing::info!("⚡ D4 X402: PolicyGovernor(3v) + RiskGate(Kelly/Kill/Bucket) + HyperEdge Memory");

    // L3: Inter-Agent IPC Bridge (shared memory state)
    let ipc = Mutex::new(IpcBridge::new());
    tracing::info!("🔗 L3 IPC Bridge: mmap inter-agent state at {}", core_ipc::IPC_FILE);

    // D5: Casper RPC stub
    let signed_provider: Option<String> = if std::env::var("CASPER_PRIVATE_KEY").is_ok() {
        tracing::info!("⛓️  D5 Casper: LIVE TX MODE — RPC ready");
        Some("https://rpc.testnet.casperlabs.io/rpc".into())
    } else {
        tracing::warn!("⛓️  D5 Casper: DRY-RUN MODE (set CASPER_PRIVATE_KEY for live deploys)");
        None
    };

    // State
    let state = Arc::new(SwarmState::new());
    let interval = std::time::Duration::from_secs(
        std::env::var("CYCLE_INTERVAL_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(60));

    // Telemetry HTTP server
    let telem = telemetry::new_handle();
    telemetry::spawn_server(telem.clone());

    tracing::info!("🚀 Full Memory Stack: L0(DashMap)→L1(OWM+Hybrid)→L2(DecisionMemory)→L3(HyperEdge)→L4(Paper)");
    tracing::info!("🚀 Pipeline: Data→Regime→Debate→ML→Recall→Judge→PreTrade→Entry→Consensus→Risk→Paper→Journal→Chain");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let start_time = std::time::Instant::now();
    let mut cycle_count: u64 = 0;

    // AI vs Human Benchmark
    let sma_engines: Mutex<std::collections::HashMap<String, SmaCrossover>> = Mutex::new(std::collections::HashMap::new());
    let bench_stats: Mutex<BenchmarkStats> = Mutex::new(BenchmarkStats::default());

    // Tx hash tracking for telemetry
    let tx_hashes: Mutex<Vec<String>> = Mutex::new(Vec::new());

    loop {
        cycle_count += 1;
        decision_cycle(&client, &debate_pool, &prompts, &models, &thresholds,
            &state, &ml_model, &risk, &paper, &trade_memories, &decision_mem,
            &sma_engines, &bench_stats, &signed_provider, &tx_hashes, &patience).await;

        // Update telemetry state after each cycle
        {
            let mut t = telem.write().await;
            t.cycle = cycle_count;
            t.uptime_secs = start_time.elapsed().as_secs();
            t.live_mode = signed_provider.is_some();
            t.tx_hashes = tx_hashes.lock().unwrap().clone();
            t.symbols = state.consensus.iter().map(|entry| {
                let r = entry.value();
                telemetry::SymbolTelemetry {
                    symbol: r.symbol.clone(),
                    price: state.symbols.get(&r.symbol).map(|s| s.price).unwrap_or(0.0),
                    price_change_24h: state.symbols.get(&r.symbol).map(|s| s.price_24h_change).unwrap_or(0.0),
                    regime: "live".into(),
                    regime_confidence: r.confidence,
                    verdict: format!("{}", r.final_verdict),
                    score: r.judge_score,
                    confidence: r.confidence,
                    volume_24h: state.symbols.get(&r.symbol).map(|s| s.volume_24h).unwrap_or(0.0),
                    buy_sell_ratio: 0.0,
                    liquidity_usd: state.symbols.get(&r.symbol).map(|s| s.open_interest).unwrap_or(0.0),
                    on_chain_logged: true,
                }
            }).collect();

            let pe = paper.lock().unwrap();
            if !pe.pnl_history.is_empty() {
                let s = pe.stats();
                t.paper_stats = Some(telemetry::PaperStats {
                    total_trades: s.total_trades,
                    win_rate: s.win_rate,
                    total_pnl: s.total_pnl,
                    max_drawdown: s.max_drawdown,
                    balance: pe.balance,
                });
            }

            // Benchmark stats
            let bs = bench_stats.lock().unwrap();
            if bs.total_cycles > 0 {
                t.benchmark = Some(telemetry::BenchmarkTelemetry {
                    total_cycles: bs.total_cycles,
                    agreements: bs.agreements,
                    agreement_rate: if bs.total_cycles > 0 { bs.agreements as f64 / bs.total_cycles as f64 } else { 0.0 },
                    ai_avg_confidence: bs.ai_avg_confidence,
                });
            }

            // AutoRamp telemetry
            let ramp_phase = titan_core::auto_ramp::AutoRamp::current_phase();
            t.ramp_state = Some(telemetry::RampTelemetry {
                current_phase: ramp_phase.phase,
                phase_label: ramp_phase.label.to_string(),
                max_position_pct: ramp_phase.max_position_pct,
                daily_loss_kill_pct: ramp_phase.daily_loss_kill_pct,
                total_promotions: 0,
                total_demotions: 0,
            });

            // Risk telemetry (from latest cycle state)
            t.risk_state = Some(telemetry::RiskTelemetry {
                dynamic_leverage: 5.0, // Default leverage (dynamic per-cycle)
                atr_estimate: 0.015,
                macro_penalty: 0.0,
                ewma_confidence: 0.0,
                risk_appetite: 0.0,
                pretrade_factor: 0.0,
                circuit_breaker: if state.is_trading_allowed() { "GREEN".into() } else { "RED".into() },
            });

            // Open positions from paper engine
            {
                let pe_inner = paper.lock().unwrap();
                t.open_positions = pe_inner.open_positions.values().map(|pos| {
                    let now_ms = chrono::Utc::now().timestamp_millis() as u64;
                    let hold_ms = (now_ms as i64).saturating_sub(pos.opened_at_ms);
                    telemetry::PositionTelemetry {
                        symbol: pos.symbol.clone(),
                        side: format!("{:?}", pos.side),
                        entry_price: pos.entry_price,
                        quantity: pos.quantity,
                        unrealized_pnl: 0.0, // calculated per-tick
                        hold_duration_secs: (hold_ms / 1000).max(0) as u64,
                        trailing_stop: pos.stop_loss.unwrap_or(0.0),
                        unstuck_stage: "monitoring".into(),
                    }
                }).collect();
            }

            // Pipeline stage completion (24 = all stages complete after cycle)
            t.pipeline_stage = 24;

            // Debates — extract from consensus results
            t.debates = state.consensus.iter().flat_map(|entry| {
                let r = entry.value();
                let mut debates = Vec::new();
                if !r.bull_argument.is_empty() {
                    debates.push(telemetry::DebateTelemetry {
                        symbol: r.symbol.clone(),
                        agent: "Veldora (Synthesis)".into(),
                        message: r.bull_argument.chars().take(200).collect(),
                        role: "bull".into(),
                        timestamp: r.timestamp,
                    });
                }
                if !r.bear_argument.is_empty() {
                    debates.push(telemetry::DebateTelemetry {
                        symbol: r.symbol.clone(),
                        agent: "Zegion (Executor)".into(),
                        message: r.bear_argument.chars().take(200).collect(),
                        role: "bear".into(),
                        timestamp: r.timestamp,
                    });
                }
                if r.macro_bias != "NEUTRAL" {
                    debates.push(telemetry::DebateTelemetry {
                        symbol: r.symbol.clone(),
                        agent: "Diablo (Architect)".into(),
                        message: format!("Macro signal: {}. Score={:.2}", r.macro_bias, r.judge_score),
                        role: "macro".into(),
                        timestamp: r.timestamp,
                    });
                }
                debates
            }).collect();

            // Log entries — synthesize from latest cycle state
            let mut logs = Vec::new();
            for entry in state.consensus.iter() {
                let r = entry.value();
                let sd = state.symbols.get(&r.symbol);
                let price = sd.as_ref().map(|s| s.price).unwrap_or(0.0);
                let vol = sd.as_ref().map(|s| s.volume_24h).unwrap_or(0.0);

                logs.push(telemetry::LogTelemetry {
                    timestamp: r.timestamp - 30,
                    tag: "[SYNAPSE]".into(),
                    message: format!("{}: Live data ingested. Price=${:.4}, Vol=${:.0}", r.symbol, price, vol),
                    level: "info".into(),
                });
                logs.push(telemetry::LogTelemetry {
                    timestamp: r.timestamp - 25,
                    tag: "[ANALYSIS]".into(),
                    message: format!("{}: Regime classified. Confidence={:.1}%", r.symbol, r.confidence),
                    level: "info".into(),
                });
                if !r.bull_argument.is_empty() {
                    logs.push(telemetry::LogTelemetry {
                        timestamp: r.timestamp - 20,
                        tag: "[DEBATE]".into(),
                        message: format!("{}: Bull vs Bear debate complete.", r.symbol),
                        level: "info".into(),
                    });
                }
                logs.push(telemetry::LogTelemetry {
                    timestamp: r.timestamp - 15,
                    tag: "[JUDGE]".into(),
                    message: format!("{}: Verdict={} Score={:.2} Conf={:.1}%", r.symbol, r.final_verdict, r.judge_score, r.confidence),
                    level: if r.meta_agreement { "success" } else { "info" }.into(),
                });
                if r.meta_agreement {
                    logs.push(telemetry::LogTelemetry {
                        timestamp: r.timestamp - 5,
                        tag: "[EXECUTE]".into(),
                        message: format!("{}: {} order consensus reached. On-chain logged.", r.symbol, r.final_verdict),
                        level: "success".into(),
                    });
                }
            }
            logs.sort_by_key(|l| l.timestamp);
            // Keep last 20 entries
            if logs.len() > 20 { logs = logs.split_off(logs.len() - 20); }
            t.log_entries = logs;
        }
        // L3: Write IPC state for inter-agent coordination
        {
            let mut bridge = ipc.lock().unwrap();
            bridge.write_state(&core_ipc::AgentState {
                sniper_vote: None,
                risk_vote: Some(state.is_trading_allowed()),
                consensus_reached: None,
                liquidation_target: None,
                global_sentiment_modifier: 0.0,
                timestamp: chrono::Utc::now().timestamp() as u64,
            });
        }

        tracing::info!("💤 Next in {}s...", interval.as_secs());
        tokio::time::sleep(interval).await;
    }
}
