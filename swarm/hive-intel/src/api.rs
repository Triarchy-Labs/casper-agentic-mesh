use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use crate::entity::MemoryEntity;
use crate::brain::BrainDiagnostics;
use crate::reward::RewardSignal;
use crate::causal::CausalGraph;
use crate::hive_engine::HiveMindEngine;

/// Application state for the Hive Mind HTTP API.
pub struct AppState {
    pub engine: Arc<HiveMindEngine>,
    pub last_diagnostics: Option<BrainDiagnostics>,
    pub last_reward: Option<RewardSignal>,
    pub causal_snapshot: Option<CausalGraph>,
}

/// V3.1 Shared state — engine (DashMap) + brain diagnostics.
pub type SharedState = Arc<RwLock<AppState>>;

/// GET /api/health — проверка жизни демона.
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "alive",
        "service": "Memory Castle V3.1",
        "version": "3.1.0",
        "modules": 17,
        "engine": "DashMap (lock-free)"
    }))
}

/// GET /api/graph — весь граф памяти (все символы).
async fn get_graph(State(state): State<SharedState>) -> Json<HashMap<String, MemoryEntity>> {
    let s = state.read().await;
    Json(s.engine.snapshot_hashmap())
}

/// GET /api/dna/:symbol — DNA одного символа.
async fn get_dna(
    State(state): State<SharedState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let s = state.read().await;
    let result = if let Some(entity_ref) = s.engine.graph.get(&symbol) {
        let entity = entity_ref.clone();
        drop(entity_ref);
        serde_json::to_value(entity).unwrap_or(serde_json::json!({"error": "serialize"}))
    } else {
        serde_json::json!({"error": "symbol not found", "symbol": symbol})
    };
    Json(result)
}

/// Краткая сводка по символу для overview.
#[derive(Serialize)]
struct SymbolSummary {
    symbol: String,
    trades: i32,
    net_pnl: f64,
    win_rate: f64,
    profit_factor: f64,
    current_streak: String,
}

/// GET /api/overview — краткая таблица всех символов.
async fn get_overview(State(state): State<SharedState>) -> Json<Vec<SymbolSummary>> {
    let s = state.read().await;
    let mut summaries: Vec<SymbolSummary> = s.engine.graph.iter()
        .map(|entry| {
            let symbol = entry.key().clone();
            let e = entry.value();
            let streak = if e.current_win_streak > 0 {
                format!("+{}W", e.current_win_streak)
            } else if e.current_loss_streak > 0 {
                format!("-{}L", e.current_loss_streak)
            } else {
                "0".to_string()
            };
            SymbolSummary {
                symbol,
                trades: e.trade_count,
                net_pnl: e.net_pnl,
                win_rate: e.win_rate,
                profit_factor: e.profit_factor,
                current_streak: streak,
            }
        })
        .collect();
    summaries.sort_by(|a, b| b.net_pnl.partial_cmp(&a.net_pnl).unwrap_or(std::cmp::Ordering::Equal));
    Json(summaries)
}

/// Ранжированная сводка.
#[derive(Serialize)]
struct RankedAsset {
    rank: usize,
    symbol: String,
    net_pnl: f64,
    profit_factor: f64,
    win_rate: f64,
    trades: i32,
    grade: String,
}

/// GET /api/rankings — топ символов с грейдами.
async fn get_rankings(State(state): State<SharedState>) -> Json<Vec<RankedAsset>> {
    let s = state.read().await;
    let mut assets: Vec<(String, f64, f64, f64, i32)> = s.engine.graph.iter()
        .map(|entry| {
            let e = entry.value();
            (entry.key().clone(), e.net_pnl, e.profit_factor, e.win_rate, e.trade_count)
        })
        .collect();
    assets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let rankings: Vec<RankedAsset> = assets.iter().enumerate().map(|(i, (sym, pnl, pf, wr, tc))| {
        let grade = if *pf >= 2.0 && *wr >= 0.55 { "A+" }
            else if *pf >= 1.5 && *wr >= 0.5 { "A" }
            else if *pf >= 1.0 { "B" }
            else if *pf >= 0.5 { "C" }
            else { "D" };
        RankedAsset {
            rank: i + 1,
            symbol: sym.clone(),
            net_pnl: *pnl,
            profit_factor: *pf,
            win_rate: *wr,
            trades: *tc,
            grade: grade.to_string(),
        }
    }).collect();

    Json(rankings)
}

/// GET /api/brain — диагностика когнитивных модулей.
async fn get_brain(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let s = state.read().await;
    match &s.last_diagnostics {
        Some(diag) => Json(serde_json::json!({
            "active": true,
            "last_diagnostics": diag,
        })),
        None => Json(serde_json::json!({
            "active": true,
            "last_diagnostics": null,
            "message": "No trades processed yet"
        })),
    }
}

/// GET /api/sessions/:symbol — PnL по торговым сессиям (D6).
async fn get_sessions(
    State(state): State<SharedState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let s = state.read().await;
    let result = if let Some(entity_ref) = s.engine.graph.get(&symbol) {
        let entity = entity_ref.clone();
        drop(entity_ref);
        let pnl_slice = entity.recent_pnl_slice();
        let trades: Vec<(f64, i64)> = pnl_slice.iter().enumerate().map(|(i, &pnl)| {
            let fake_ts = if entity.last_trade_ts > 0 && !pnl_slice.is_empty() {
                entity.last_trade_ts - ((pnl_slice.len() - 1 - i) as i64 * 60_000)
            } else { 0 };
            (pnl, fake_ts)
        }).collect();
        let analysis = crate::patterns::analyze_sessions(&symbol, &trades);
        serde_json::to_value(analysis).unwrap_or(serde_json::json!({"error": "serialize"}))
    } else {
        serde_json::json!({"error": "symbol not found"})
    };
    Json(result)
}

/// GET /api/reward — последний reward signal (F8).
async fn get_reward(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let s = state.read().await;
    match &s.last_reward {
        Some(reward) => Json(serde_json::json!({
            "active": true,
            "last_reward": reward,
        })),
        None => Json(serde_json::json!({
            "active": true,
            "last_reward": null,
            "message": "No reward signals generated yet"
        })),
    }
}

/// GET /api/correlations — каузальные связи между символами (Phase 3, D7).
async fn get_correlations(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let s = state.read().await;
    match &s.causal_snapshot {
        Some(causal) => {
            let mut edges: Vec<serde_json::Value> = causal.edges.values()
                .filter(|e| e.observations >= 3) // Минимум 3 наблюдения
                .map(|e| serde_json::json!({
                    "cause": e.cause,
                    "effect": e.effect,
                    "strength": format!("{:.3}", e.strength()),
                    "avg_delay_ms": e.avg_delay_ms as i64,
                    "observations": e.observations,
                    "confirmations": e.confirmations,
                    "significant": e.is_significant(),
                }))
                .collect();
            // Сортировать по strength (самые сильные сверху)
            edges.sort_by(|a, b| {
                let sa = a["strength"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let sb = b["strength"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            Json(serde_json::json!({
                "total_edges": causal.edges.len(),
                "shown_edges": edges.len(),
                "correlations": edges,
            }))
        }
        None => Json(serde_json::json!({
            "total_edges": 0,
            "correlations": [],
            "message": "No causal data yet"
        })),
    }
}

/// GET /api/pattern?symbol=BTCUSDT&score=0.0 — Factor 14 HiveMind for Ouroboros.
/// Synthesizes Brain's cognitive modules into a single factor_score.
async fn get_pattern(
    State(state): State<SharedState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let symbol = params.get("symbol").cloned().unwrap_or_default();
    let s = state.read().await;

    // Get entity DNA
    let entity_data = s.engine.graph.get(&symbol);
    let has_data = entity_data.is_some();

    if !has_data || symbol.is_empty() {
        return Json(serde_json::json!({
            "factor_score": 0.0,
            "detail": "no data for symbol",
        }));
    }

    let entity = entity_data.unwrap().clone();

    // Synthesize factor from entity stats
    let mut factor = 0.0_f64;
    let mut details = Vec::new();

    // 1. Win rate signal: >60% = bullish memory, <40% = bearish memory
    if entity.trade_count >= 5 {
        let wr_signal = (entity.win_rate - 0.5) * 2.0; // -1.0 to +1.0
        factor += wr_signal.clamp(-0.5, 0.5);
        details.push(format!("WR:{:.0}%", entity.win_rate * 100.0));
    }

    // 2. Profit factor signal
    if entity.profit_factor > 1.5 {
        factor += 0.3;
        details.push(format!("PF:{:.1}", entity.profit_factor));
    } else if entity.profit_factor < 0.7 && entity.trade_count >= 3 {
        factor -= 0.3;
        details.push(format!("PF:{:.1}⚠", entity.profit_factor));
    }

    // 3. Loss streak penalty
    if entity.current_loss_streak >= 3 {
        factor -= 0.5;
        details.push(format!("-{}L", entity.current_loss_streak));
    }

    // 4. Brain diagnostics (if available)
    if let Some(diag) = &s.last_diagnostics {
        if diag.symbol == symbol {
            // Drift penalty
            if diag.drift_detected {
                factor -= 0.8;
                details.push("DRIFT".into());
            }
            // DQS gate
            if diag.dqs_tier == "skip" {
                factor -= 1.0;
                details.push("DQS:SKIP".into());
            } else if diag.dqs_tier == "caution" {
                factor -= 0.3;
                details.push("DQS:CAUTION".into());
            }
            // OWM recall boost
            if diag.owm_score > 0.5 {
                factor += 0.3;
                details.push(format!("OWM:{:.2}", diag.owm_score));
            }
        }
    }

    // Clamp to ±3.0 (Factor 14 range)
    factor = factor.clamp(-3.0, 3.0);

    Json(serde_json::json!({
        "factor_score": (factor * 100.0).round() / 100.0,
        "detail": details.join(" | "),
        "symbol": symbol,
        "trades": entity.trade_count,
        "net_pnl": entity.net_pnl,
    }))
}

/// Создаёт axum Router V3.2 — 11 endpoints, DashMap + Reward + Correlations + Pattern.
pub fn create_router_v3(shared_state: SharedState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/graph", get(get_graph))
        .route("/api/dna/{symbol}", get(get_dna))
        .route("/api/overview", get(get_overview))
        .route("/api/rankings", get(get_rankings))
        .route("/api/brain", get(get_brain))
        .route("/api/sessions/{symbol}", get(get_sessions))
        .route("/api/reward", get(get_reward))
        .route("/api/correlations", get(get_correlations))
        .route("/api/pattern", get(get_pattern))
        .with_state(shared_state)
}

