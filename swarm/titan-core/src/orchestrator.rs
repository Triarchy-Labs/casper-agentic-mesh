// src/modules/orchestrator.rs
// ═══════════════════════════════════════════════════════════════
// ORCHESTRATOR — Boot Sequence + Head Spawning + Market Weather
// ═══════════════════════════════════════════════════════════════
// Вынесено из main.rs:534-795 в отдельный модуль.
// V10.5: tracing structured logging + WebSocket price cache
//
// Ответственность:
//   - API key loading + time sync
//   - Shared state initialization
//   - Anti-Amnesia crash recovery (position restore)
//   - Head spawning (MEDIUM-3m, MONSTER-5m)
//   - Market weather loop (BTC Gravity, Hype Radar, Balance refresh)
//   - Auto-Recalibration (6h cycle: hour_analyzer + backtester)
//   - Time resync (BUG-15), Patience cleanup (FORENSIC-11)

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use reqwest::Client;
use chrono::Local;
use serde_json::Value;
use std::fs;

use crate::api_pool::ApiPool;
use crate::risk::RiskMatrix;
use crate::alpha_head::AlphaGhostHead;
use crate::patience::PatienceTracker;
use crate::network::BybitNetwork;
use crate::scanner::MarketScanner;
use crate::logger::TitanLogger;
use crate::types::{ActivePosition, get_current_session};
use crate::ws_feed::{WsFeed, SharedPriceCache};
use crate::analyze_symbol;



/// Всё shared state Титана в одной структуре
pub struct TitanState {
    pub api_pool: Arc<ApiPool>,
    pub time_offset: Arc<RwLock<i64>>,
    pub positions: Arc<RwLock<HashMap<String, ActivePosition>>>,
    pub cooldowns: Arc<RwLock<HashMap<String, i64>>>,
    pub hype_list: Arc<RwLock<Vec<String>>>,
    pub btc_score: Arc<RwLock<f64>>,
    pub loss_streak: Arc<RwLock<HashMap<String, u32>>>,
    pub daily_loss: Arc<RwLock<f64>>,
    pub session: Arc<RwLock<String>>,
    pub ghost_head: Arc<RwLock<AlphaGhostHead>>,
    pub patience: Arc<RwLock<PatienceTracker>>,
    pub available_balance: Arc<RwLock<f64>>,
    pub price_cache: SharedPriceCache,
}

impl TitanState {
    /// Cheap clone (all Arc — just reference count bump)
    pub fn clone_state(&self) -> Self {
        Self {
            api_pool: self.api_pool.clone(),
            time_offset: self.time_offset.clone(),
            positions: self.positions.clone(),
            cooldowns: self.cooldowns.clone(),
            hype_list: self.hype_list.clone(),
            btc_score: self.btc_score.clone(),
            loss_streak: self.loss_streak.clone(),
            daily_loss: self.daily_loss.clone(),
            session: self.session.clone(),
            ghost_head: self.ghost_head.clone(),
            patience: self.patience.clone(),
            available_balance: self.available_balance.clone(),
            price_cache: self.price_cache.clone(),
        }
    }
}

/// Конфигурация одной "головы"
#[derive(Clone)]
pub struct HeadConfig {
    pub name: &'static str,
    pub timeframe: &'static str,
    pub atr_multiplier: f64,
    pub cooldown_time: i64,
    pub sleep_between_symbols_ms: u64,
    pub sleep_after_cycle_ms: u64,
}

pub struct Orchestrator;

impl Orchestrator {
    /// Boot sequence: загрузка ключей, синхронизация времени, shared state
    pub async fn boot() -> (TitanState, Client) {
        // V10.5: Initialize structured tracing
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("bot_v4_titan=info"))
            )
            .with_target(false)
            .compact()
            .init();

        tracing::info!("════════════════════════════════════════════════");
        tracing::info!("TITAN V11.1 — INSTITUTIONAL HARDENING ENGINE");
        tracing::info!(build = %Local::now().format("%Y-%m-%d %H:%M"), "BUILD: V11.1");
        tracing::info!("MODULES: 25 | TESTS: 119 | WARNINGS: 0");
        tracing::info!("MEMORY: V10 Hive Mind (UDP 8888) + tilt_lock + alpha_boost");
        tracing::info!("════════════════════════════════════════════════");

        // Load API keys
        let env_content = fs::read_to_string(r"E:\ROXY_SYSTEM\Projects\Antigravity-Swarm\bot_config.env").unwrap_or_default();
        let mut api_pairs: Vec<(String, String)> = Vec::new();
        for i in 1..=5 {
            let mut k = String::new(); let mut s = String::new();
            for line in env_content.lines() {
                if line.starts_with(&format!("BYBIT_API_KEY_{i}=")) { k = line.split('=').nth(1).unwrap_or("").replace('"',"").trim().to_string(); }
                if line.starts_with(&format!("BYBIT_SECRET_KEY_{i}=")) { s = line.split('=').nth(1).unwrap_or("").replace('"',"").trim().to_string(); }
            }
            if !k.is_empty() && !s.is_empty() && !k.contains("placeholder") { api_pairs.push((k, s)); }
        }
        if api_pairs.is_empty() { eprintln!("FATAL: No API keys in bot_config.env!"); std::process::exit(1); }
        let api_pool = Arc::new(ApiPool::new(api_pairs));
        tracing::info!(keys = api_pool.total_keys(), "[BOOT] API Pool loaded");

        // Time sync
        let main_client = Client::builder().timeout(Duration::from_secs(10)).build().expect("FATAL: HTTP client init failed");
        let server_time = BybitNetwork::fetch_bybit_server_time(&main_client).await.unwrap_or(0);
        let time_offset = Arc::new(RwLock::new(server_time - chrono::Utc::now().timestamp_millis()));
        tracing::info!(offset_ms = *time_offset.read().await, "[BOOT] Time synced");

        // Shared state
        let positions: Arc<RwLock<HashMap<String, ActivePosition>>> = Arc::new(RwLock::new(HashMap::new()));
        let cooldowns: Arc<RwLock<HashMap<String, i64>>> = Arc::new(RwLock::new(HashMap::new()));
        let hype_list: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let btc_score: Arc<RwLock<f64>> = Arc::new(RwLock::new(0.0));
        let loss_streak: Arc<RwLock<HashMap<String, u32>>> = Arc::new(RwLock::new(HashMap::new()));
        let daily_loss: Arc<RwLock<f64>> = Arc::new(RwLock::new(0.0));
        let session: Arc<RwLock<String>> = Arc::new(RwLock::new(get_current_session().to_string()));
        let ghost_head: Arc<RwLock<AlphaGhostHead>> = Arc::new(RwLock::new(AlphaGhostHead::new()));
        let patience: Arc<RwLock<PatienceTracker>> = Arc::new(RwLock::new(PatienceTracker::new()));

        // REAL BALANCE from Bybit API
        let init_balance = BybitNetwork::fetch_wallet_balance(&main_client, &api_pool, &time_offset).await;
        let available_balance: Arc<RwLock<f64>> = Arc::new(RwLock::new(if init_balance > 0.0 { init_balance } else { 50.0 }));
        tracing::info!(balance = format!("{:.2}", *available_balance.read().await).as_str(), "[BOOT] 💰 Available balance");

        // Restore daily_loss from state file
        if let Ok(content) = fs::read_to_string(r"E:\ROXY_SYSTEM\Projects\Antigravity-Swarm\Swarm_Kingdoms\V4_Titan\titan_state.json") {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if json["date"].as_str() == Some(&Local::now().format("%Y-%m-%d").to_string()) {
                    *daily_loss.write().await = json["daily_loss"].as_f64().unwrap_or(0.0);
                }
            }
        }

        // Anti-Amnesia: restore positions from Bybit API
        tracing::info!("[BOOT] Anti-Amnesia: syncing positions with exchange...");
        let saved_ownership: HashMap<String, Value> = fs::read_to_string(
            r"E:\ROXY_SYSTEM\Projects\Antigravity-Swarm\Swarm_Kingdoms\V4_Titan\titan_ownership.json")
            .ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default();

        let (key, secret) = api_pool.get_current_keys();
        let qs = "category=linear&settleCoin=USDT";
        let ts = (chrono::Utc::now().timestamp_millis() - 3000).to_string();
        // V11.0.1 P3 FIX: delegate to BybitSigning (was inline duplicate)
        let sig = super::network::BybitSigning::generate_signature(&ts, "5000", qs, &key, &secret);

        if let Ok(res) = main_client.get(format!("https://api.bybit.com/v5/position/list?{qs}"))
            .header("X-BAPI-API-KEY", &key).header("X-BAPI-TIMESTAMP", &ts)
            .header("X-BAPI-SIGN", &sig).header("X-BAPI-RECV-WINDOW", "5000").send().await {
            if let Ok(json) = res.json::<Value>().await {
                if let Some(arr) = json["result"]["list"].as_array() {
                    for item in arr {
                        let sym = item["symbol"].as_str().unwrap_or("").to_string();
                        let amount: f64 = item["size"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        let side = item["side"].as_str().unwrap_or("Buy").to_string();
                        let entry: f64 = item["avgPrice"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                        if amount > 0.0 && sym.ends_with("USDT") {
                            let (owner, hp, lp, sl) = if let Some(val) = saved_ownership.get(&sym) {
                                if val.is_string() {
                                    (val.as_str().unwrap_or("5").to_string(), entry, entry, 0.0)
                                } else {
                                    (
                                        val["tf"].as_str().unwrap_or("5").to_string(),
                                        val["hp"].as_f64().unwrap_or(entry),
                                        val["lp"].as_f64().unwrap_or(entry),
                                        val["sl"].as_f64().unwrap_or(0.0),
                                    )
                                }
                            } else {
                                ("5".to_string(), entry, entry, 0.0)
                            };
                            positions.write().await.insert(sym.clone(), ActivePosition {
                                symbol: sym.clone(), side: side.clone(), amount, buy_price: entry,
                                highest_price: hp, lowest_price: lp,
                                owner_timeframe: owner.clone(), last_pushed_sl: sl,
                                entry_time_ms: chrono::Utc::now().timestamp_millis(),
                                unstuck_stage1_done: false,
                                unstuck_stage1_time: 0,
                                pending_reentry_price: None,
                                pending_reentry_qty: None,
                            });
                            tracing::info!(symbol = %sym, side = %side, qty = amount, entry = entry, head = %owner, hp = format!("{hp:.2}").as_str(), sl = format!("{sl:.4}").as_str(), "[BOOT] Position restored");
                        }
                    }
                }
            }
        }

        tracing::info!(count = positions.read().await.len(), "[BOOT] Active positions");
        tracing::info!("════════════════════════════════════════════════");
        tracing::info!("HEADS: MEDIUM (3m) + MONSTER (5m) [MICRO OFF]");
        tracing::info!("PROTECTION: $18/session | 2-loss ban | Dead Man's Switch");
        tracing::info!("════════════════════════════════════════════════");

        // V10.5: WebSocket price cache + spawn
        let price_cache = WsFeed::new_cache();
        {
            // Collect initial symbols for WS subscription
            let pos_syms: Vec<String> = positions.read().await.keys().cloned().collect();
            let hype_syms: Vec<String> = hype_list.read().await.clone();
            let mut ws_symbols: Vec<String> = Vec::new();
            ws_symbols.push("BTCUSDT".to_string()); // Always subscribe to BTC
            ws_symbols.extend(pos_syms);
            ws_symbols.extend(hype_syms);
            ws_symbols.sort();
            ws_symbols.dedup();
            WsFeed::spawn(price_cache.clone(), ws_symbols);
            tracing::info!("🔌 [BOOT] WebSocket price feed spawned");
        }

        let state = TitanState {
            api_pool, time_offset, positions, cooldowns, hype_list,
            btc_score, loss_streak, daily_loss, session, ghost_head,
            patience, available_balance, price_cache,
        };

        (state, main_client)
    }

    /// Spawn a trading head (MEDIUM-3m, MONSTER-5m, etc.)
    pub fn spawn_head(state: &TitanState, head: HeadConfig) {
        let c = Client::builder().timeout(Duration::from_secs(10)).build().expect("FATAL: HTTP client init failed");
        let s = state.clone_state();
        let rm = RiskMatrix::new();

        tokio::spawn(async move {
            loop {
                let mut syms: Vec<String> = Vec::new();
                { let pp = s.positions.read().await; let h = s.hype_list.read().await;
                  for k in pp.keys() { syms.push(k.clone()); }
                  for sym in h.iter() { syms.push(sym.clone()); } }
                let mut seen = HashSet::new();
                syms.retain(|x| seen.insert(x.clone()));
                let score = *s.btc_score.read().await;
                for sym in syms {
                    analyze_symbol(&c, &s, &sym, head.name, head.timeframe,
                        head.atr_multiplier, head.cooldown_time,
                        score, &rm).await;
                    sleep(Duration::from_millis(head.sleep_between_symbols_ms)).await;
                }
                sleep(Duration::from_millis(head.sleep_after_cycle_ms)).await;
            }
        });
    }

    /// Market weather loop (BTC gravity, hype scan, balance refresh, auto-calibration)
    /// V11.0.1: Includes graceful shutdown handler (T4)
    pub async fn run_weather_loop(state: &TitanState, client: &Client) {
        let mut last_recalibration = std::time::Instant::now();
        let recalibration_interval = std::time::Duration::from_secs(6 * 3600);
        let mut time_resync_counter: u32 = 0;

        // T4: Spawn shutdown signal listener
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let shutdown_clone = shutdown.clone();
        let state_clone = state.clone_state();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::warn!("⚠️ [SHUTDOWN] CTRL-C received — initiating graceful shutdown...");
            
            // Snapshot positions to disk before exit
            let positions = state_clone.positions.read().await;
            if !positions.is_empty() {
                let ownership: HashMap<String, serde_json::Value> = positions.iter().map(|(k, v)| (k.clone(), serde_json::json!({
                    "side": v.side, "amount": v.amount, "buy_price": v.buy_price,
                    "tf": v.owner_timeframe, "hp": v.highest_price, "lp": v.lowest_price,
                    "sl": v.last_pushed_sl
                }))).collect();
                let _ = crate::safe_io::SafeIO::atomic_write_with_backup(
                    r"E:\ROXY_SYSTEM\Projects\Antigravity-Swarm\Swarm_Kingdoms\V4_Titan\titan_ownership.json",
                    &serde_json::to_string(&ownership).unwrap_or_default()
                );
                tracing::info!(count = positions.len(), "[SHUTDOWN] 💾 Positions snapshot saved with backup");
            }
            
            // Save daily state
            let dl = *state_clone.daily_loss.read().await;
            let today = Local::now().format("%Y-%m-%d").to_string();
            let j = serde_json::json!({"date": today, "daily_loss": dl, "session": crate::types::get_current_session()});
            let _ = crate::safe_io::SafeIO::atomic_write(
                r"E:\ROXY_SYSTEM\Projects\Antigravity-Swarm\Swarm_Kingdoms\V4_Titan\titan_state.json",
                &j.to_string()
            );
            tracing::info!("[SHUTDOWN] 💾 Daily state saved (loss={:.2})", dl);
            
            tracing::warn!("[SHUTDOWN] ✅ Graceful shutdown complete. Positions preserved on exchange.");
            shutdown_clone.notify_one();
        });

        loop {
            // T4: Check shutdown flag
            tokio::select! {
                _ = shutdown.notified() => {
                    tracing::info!("[SHUTDOWN] Weather loop terminated cleanly.");
                    std::process::exit(0);
                }
                _ = Self::weather_cycle(state, client, &mut last_recalibration, recalibration_interval, &mut time_resync_counter) => {}
            }
        }
    }

    /// Single weather cycle (extracted for tokio::select! compatibility)
    async fn weather_cycle(
        state: &TitanState, client: &Client,
        last_recalibration: &mut std::time::Instant,
        recalibration_interval: std::time::Duration,
        time_resync_counter: &mut u32,
    ) {
            // BTC Gravity
            let score = MarketScanner::check_btc_gravity(client).await;
            *state.btc_score.write().await = score;
            if score < 0.0 { TitanLogger::log("SERVER", &format!("BTC GRAVITY RED ({score:.2}%). Short mode.")); }

            // Hype Radar
            let hype = MarketScanner::get_hype_coins(client).await;
            if !hype.is_empty() {
                // V11: Auto-Deallow filter — remove underperforming symbols
                let (filtered_hype, removed) = crate::deallow::Deallow::filter_hype_list(&hype);
                if !removed.is_empty() {
                    TitanLogger::log("DEALLOW", &format!("Banned: {}", removed.join(", ")));
                }
                *state.hype_list.write().await = filtered_hype.clone();
                TitanLogger::log("SERVER", &format!("RADAR: {}", filtered_hype.iter().take(6).cloned().collect::<Vec<_>>().join(", ")));

                // V11: Check if any removed symbols have recovered → log reallow eligibility
                for sym in &removed {
                    if crate::deallow::Deallow::check_reallow(sym) {
                        TitanLogger::log("DEALLOW", &format!("♻️ {sym} eligible for reallow (WR≥55%)"));
                    }
                }
            }

            // Refresh balance every cycle
            let fresh_balance = BybitNetwork::fetch_wallet_balance(client, &state.api_pool, &state.time_offset).await;
            if fresh_balance > 0.0 {
                *state.available_balance.write().await = fresh_balance;
            }

            // V11: Auto-Ramp evaluation (5-Gate Capital Scaling)
            let ramp_log = crate::auto_ramp::AutoRamp::evaluate();
            TitanLogger::log("RAMP", &ramp_log);

            // BUG-15 FIX: periodic time resync (~10 min)
            *time_resync_counter += 1;
            if *time_resync_counter >= 13 {
                *time_resync_counter = 0;
                if let Some(server_ms) = BybitNetwork::fetch_bybit_server_time(client).await {
                    let new_offset = server_ms - chrono::Utc::now().timestamp_millis();
                    *state.time_offset.write().await = new_offset;
                    TitanLogger::log("TIME", &format!("Resync: offset={new_offset}ms"));
                }
            }

            // ═══ AUTO-RECALIBRATION (every 6 hours) ═══
            if last_recalibration.elapsed() >= recalibration_interval {
                *last_recalibration = std::time::Instant::now();
                crate::calibration::Calibration::run_if_needed();
            }

            // FORENSIC-11 FIX: periodic patience cleanup (V11.0.1: delegated to purge_stale)
            {
                state.patience.write().await.purge_stale();
            }

            // V10.5: Cleanup stale WS price cache entries (>60s)
            {
                let now_ms = chrono::Utc::now().timestamp_millis();
                state.price_cache.write().await.retain(|_, v| now_ms - v.timestamp_ms < 60_000);
            }

            // Heartbeat (M4: reduced from 9×5s=45s to 4×3s=12s blind spot)
            for _ in 0..4 {
                sleep(Duration::from_secs(3)).await;
                let n = state.positions.read().await.len();
                let bal = *state.available_balance.read().await;
                let (cb_fails, cb_open) = crate::network::CircuitBreaker::status();
                let cb_tag = if cb_open { "⚡OPEN" } else if cb_fails > 0 { &format!("⚠{cb_fails}f") } else { "✅" };
                TitanLogger::log("HEARTBEAT", &format!("Scanning... {n}/9 active | ${bal:.2} bal | API:{cb_tag}"));
            }
    }
}
