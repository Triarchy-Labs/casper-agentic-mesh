//! Telemetry Server — Real-time HTTP API for swarm state monitoring.
//!
//! Serves live JSON state at http://0.0.0.0:3402/ for the dashboard frontend.
//! This is the "live-stream/transparency execution" requirement for the hackathon.
//!
//! Endpoints:
//!   GET /           → Full swarm state (all symbols, latest verdicts, pipeline metrics)
//!   GET /health     → Health check + uptime
//!   GET /verdicts   → Latest trade verdicts only
//!   GET /regime     → Current market regime per symbol
//!   GET /paper      → Paper trading PnL + stats

use axum::{routing::get, Json, Router, middleware::{self, Next}, response::Response, http::Request};
use axum::body::Body;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Telemetry port — 3402 (x402 reference).
pub const TELEMETRY_PORT: u16 = 3402;

/// Live swarm telemetry state — updated after each decision cycle.
#[derive(Debug, Clone, Serialize, Default)]
pub struct TelemetryState {
    pub version: &'static str,
    pub uptime_secs: u64,
    pub cycle: u64,
    pub pipeline_stage: u32,
    pub pipeline_total: u32,
    pub live_mode: bool,
    pub symbols: Vec<SymbolTelemetry>,
    pub debates: Vec<DebateTelemetry>,
    pub log_entries: Vec<LogTelemetry>,
    pub tx_hashes: Vec<String>,
    pub paper_stats: Option<PaperStats>,
    pub benchmark: Option<BenchmarkTelemetry>,
    pub pipeline: &'static str,
    pub agent_id: u64,
    pub chain_id: u64,
    pub registry_address: &'static str,
    // New: extended metrics
    pub risk_state: Option<RiskTelemetry>,
    pub ramp_state: Option<RampTelemetry>,
    pub open_positions: Vec<PositionTelemetry>,
    pub pipeline_stages: Vec<PipelineStageTelemetry>,
}

/// Per-symbol telemetry snapshot.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SymbolTelemetry {
    pub symbol: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub regime: String,
    pub regime_confidence: f64,
    pub verdict: String,
    pub score: f64,
    pub confidence: f64,
    pub volume_24h: f64,
    pub buy_sell_ratio: f64,
    pub liquidity_usd: f64,
    pub on_chain_logged: bool,
}

/// Paper trading stats for telemetry.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PaperStats {
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub max_drawdown: f64,
    pub balance: f64,
}

/// AI vs Human benchmark telemetry.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BenchmarkTelemetry {
    pub total_cycles: u64,
    pub agreements: u64,
    pub agreement_rate: f64,
    pub ai_avg_confidence: f64,
}

/// Per-symbol debate telemetry (bull/bear arguments).
#[derive(Debug, Clone, Serialize, Default)]
pub struct DebateTelemetry {
    pub symbol: String,
    pub agent: String,
    pub message: String,
    pub role: String,  // "bull", "bear", "macro"
    pub timestamp: i64,
}

/// Log stream entry for real-time activity feed.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LogTelemetry {
    pub timestamp: i64,
    pub tag: String,
    pub message: String,
    pub level: String,  // "info", "success", "warn"
}

/// Risk management telemetry.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RiskTelemetry {
    pub dynamic_leverage: f64,
    pub atr_estimate: f64,
    pub macro_penalty: f64,
    pub ewma_confidence: f64,
    pub risk_appetite: f64,
    pub pretrade_factor: f64,
    pub circuit_breaker: String, // "GREEN", "YELLOW", "RED"
}

/// AutoRamp capital scaling telemetry.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RampTelemetry {
    pub current_phase: u8,
    pub phase_label: String,
    pub max_position_pct: f64,
    pub daily_loss_kill_pct: f64,
    pub total_promotions: u32,
    pub total_demotions: u32,
}

/// Open position telemetry for paper trading.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PositionTelemetry {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub unrealized_pnl: f64,
    pub hold_duration_secs: u64,
    pub trailing_stop: f64,
    pub unstuck_stage: String,
}

/// Individual pipeline stage telemetry.
#[derive(Debug, Clone, Serialize, Default)]
pub struct PipelineStageTelemetry {
    pub name: String,
    pub status: String, // "pass", "skip", "block", "active"
    pub detail: String,
}

/// Shared telemetry state handle.
pub type TelemetryHandle = Arc<RwLock<TelemetryState>>;

/// Create a new telemetry handle with default state.
pub fn new_handle() -> TelemetryHandle {
    Arc::new(RwLock::new(TelemetryState {
        version: "v5.0-triarchy",
        pipeline: "Data→Correlation→Regime→Debate→ML→Recall→Judge→DQS→PreTrade→Confidence→Patience→Entry→Consensus→Risk→Paper→RiskMatrix→Trailing→Unstuck→AutoRamp→Deallow→Anomaly→Journal→Chain→IPC",
        pipeline_total: 24,
        agent_id: 1,
        chain_id: 5000,
        registry_address: "0x1150f09ae885e6E7BcC0cb38feDd200d7f580008",
        ..Default::default()
    }))
}

/// Spawn the telemetry HTTP server on a background task.
pub fn spawn_server(handle: TelemetryHandle) {
    tokio::spawn(async move {
        let h1 = handle.clone();
        let h2 = handle.clone();
        let h3 = handle.clone();
        let h4 = handle.clone();
        let h5 = handle.clone();
        let h6 = handle.clone();
        let h7 = handle.clone();

        // CORS middleware for cross-origin dashboard access
        async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
            let mut resp = next.run(req).await;
            resp.headers_mut().insert("access-control-allow-origin", "*".parse().unwrap());
            resp.headers_mut().insert("access-control-allow-methods", "GET".parse().unwrap());
            resp
        }

        let app = Router::new()
            .layer(middleware::from_fn(cors_middleware))
            .route("/", get(move || {
                let h = h1.clone();
                async move {
                    let state = h.read().await;
                    Json(state.clone())
                }
            }))
            .route("/health", get(move || {
                let h = h2.clone();
                async move {
                    let state = h.read().await;
                    Json(serde_json::json!({
                        "status": "ok",
                        "version": state.version,
                        "uptime_secs": state.uptime_secs,
                        "cycle": state.cycle,
                        "symbols_tracked": state.symbols.len(),
                    }))
                }
            }))
            .route("/verdicts", get(move || {
                let h = h3.clone();
                async move {
                    let state = h.read().await;
                    let verdicts: Vec<_> = state.symbols.iter().map(|s| {
                        serde_json::json!({
                            "symbol": s.symbol,
                            "verdict": s.verdict,
                            "score": s.score,
                            "confidence": s.confidence,
                            "regime": s.regime,
                        })
                    }).collect();
                    Json(verdicts)
                }
            }))
            .route("/regime", get(move || {
                let h = h4.clone();
                async move {
                    let state = h.read().await;
                    let regimes: Vec<_> = state.symbols.iter().map(|s| {
                        serde_json::json!({
                            "symbol": s.symbol,
                            "regime": s.regime,
                            "confidence": s.regime_confidence,
                            "price": s.price,
                        })
                    }).collect();
                    Json(regimes)
                }
            }))
            .route("/benchmark", get(move || {
                let h = h5.clone();
                async move {
                    let state = h.read().await;
                    Json(state.benchmark.clone().unwrap_or_default())
                }
            }))
            .route("/positions", get(move || {
                let h = h6.clone();
                async move {
                    let state = h.read().await;
                    Json(serde_json::json!({
                        "open": state.open_positions,
                        "paper_stats": state.paper_stats,
                    }))
                }
            }))
            .route("/risk", get(move || {
                let h = h7.clone();
                async move {
                    let state = h.read().await;
                    Json(serde_json::json!({
                        "risk": state.risk_state,
                        "ramp": state.ramp_state,
                        "circuit_breaker": state.risk_state.as_ref().map(|r| r.circuit_breaker.clone()).unwrap_or("UNKNOWN".into()),
                    }))
                }
            }));

        let port = std::env::var("PORT")
            .ok().and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(TELEMETRY_PORT);
        let addr = format!("0.0.0.0:{}", port);
        tracing::info!("📡 Telemetry server: http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await
            .expect("Failed to bind telemetry port");
        axum::serve(listener, app).await
            .expect("Telemetry server crashed");
    });
}
