// src/modules/logger.rs
// Модуль логирования. UDP + File + Telegram.
use std::io::Write;
use crate::safe_io::data_file;
use std::net::UdpSocket;
use std::sync::Mutex;
use chrono::Local;

pub struct TitanLogger;

// BUG-25 FIX: Mutex to prevent file corruption from concurrent writes
static LOG_MUTEX: Mutex<()> = Mutex::new(());

impl TitanLogger {
    /// Основной лог: консоль + UDP:4444 + файл + Telegram при критических событиях
    pub fn log(head: &str, msg: &str) {
        let now = Local::now().format("%H:%M:%S");
        tracing::info!(head = %head, time = %now, "{}", msg);
        
        // UDP to Dashboard
        if let Ok(s) = UdpSocket::bind("0.0.0.0:0") {
            let _ = s.send_to(format!("[TITAN] [{head}] {msg}").as_bytes(), "127.0.0.1:4444");
        }
        
        // File log with rotation (BUG-25: under mutex)
        if let Ok(_guard) = LOG_MUTEX.lock() {
            let log_path = data_file("swarm_feed.log");
            if let Ok(meta) = std::fs::metadata(&log_path) {
                if meta.len() > 5_000_000 {
                    if let Ok(content) = std::fs::read_to_string(&log_path) {
                        let mut cut = content.len().saturating_sub(2_000_000);
                        while cut < content.len() && !content.is_char_boundary(cut) { cut += 1; }
                        let _ = std::fs::write(&log_path, &content[cut..]);
                    }
                }
            }
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(f, "[{head}] {msg}");
            }
        }
        
        // Telegram for critical events
        let up = msg.to_uppercase();
        if up.contains("ОРДЕР") || up.contains("АВАРИЙН") || up.contains("ТЕЙК") || up.contains("STRIKE") {
            let _ = std::process::Command::new("python3")
                .arg(data_file("send_telepathic_tg.py"))
                .arg(format!("TITAN: {msg}")).arg("--target").arg("swarm")
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null()).spawn();
        }
    }

    /// CSV Trade Logger
    pub fn log_trade(head: &str, action: &str, symbol: &str, side: &str, qty: f64, price: f64, note: &str) {
        let now = Local::now().format("%Y-%m-%dT%H:%M:%S");
        let log_path = data_file("trades_log.csv");
        let needs_header = !std::path::Path::new(&log_path).exists();
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            if needs_header { let _ = writeln!(f, "timestamp,bot,action,symbol,side,qty,price,note"); }
            let _ = writeln!(f, "{now},{head},{action},{symbol},{side},{qty:.6},{price:.6},{note}");
        }
    }

    /// V10 Hive Mind Memory Reporter V2 (UDP 8888) — sends full DNA payload
    pub fn report_to_v10(symbol: &str, pnl: f64, side: &str, entry_time_ms: i64) {
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let hold_duration_ms = if entry_time_ms > 0 { now_ms - entry_time_ms } else { 0 };
            let msg = serde_json::json!({
                "symbol": symbol,
                "pnl": pnl,
                "side": side,
                "hold_duration_ms": hold_duration_ms,
                "timestamp_ms": now_ms
            });
            let _ = sock.send_to(msg.to_string().as_bytes(), "127.0.0.1:8888");
        }
    }

    /// Treasury Reporter (UDP 8766)
    pub fn report_to_treasury(symbol: &str, action: &str, pnl: f64) {
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            let msg = serde_json::json!({"bot":"TITAN","ticker":symbol,"action":action,"pnl":pnl,"timestamp":chrono::Utc::now().to_rfc3339()});
            let _ = sock.send_to(msg.to_string().as_bytes(), "127.0.0.1:8766");
        }
    }

    /// Ouroboros PnL Feedback — writes outcome JSON for decision_memory.ingest_outcomes()
    pub fn report_to_ouroboros(symbol: &str, side: &str, entry_price: f64, exit_price: f64, pnl: f64) {
        let outcomes_dir = data_file("outcomes");
        let _ = std::fs::create_dir_all(&outcomes_dir);
        let now_ms = chrono::Utc::now().timestamp_millis();
        let outcome = serde_json::json!({
            "symbol": symbol,
            "side": side,
            "entry_price": entry_price,
            "exit_price": exit_price,
            "pnl": pnl,
            "timestamp_ms": now_ms,
        });
        let path = format!("{outcomes_dir}/{symbol}_{now_ms}.json");
        match std::fs::write(&path, serde_json::to_string_pretty(&outcome).unwrap_or_default()) {
            Ok(_) => tracing::info!("[OUROBOROS-FB] {} outcome written: PnL=${:.2}", symbol, pnl),
            Err(e) => tracing::warn!("[OUROBOROS-FB] Failed to write outcome: {}", e),
        }
    }
}
