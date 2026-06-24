//! Decision Memory — Self-Learning Trade Journal
//! Ported from TradingAgents (71K★) `memory.py` → Rust
//!
//! Architecture:
//! - Phase A: After each cycle, append verdict to `trading_memory.md` (pending)
//! - Phase B: On next cycle for same symbol, fetch real PnL from Bybit,
//!            generate LLM reflection, and update the entry
//! - Phase C: Inject last N reflections into Meta Judge prompt

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════
// DECISION ENTRY
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub timestamp: String,
    pub symbol: String,
    pub verdict: String,       // "BUY", "SELL", "HOLD"
    pub score: f64,
    pub confidence: f64,
    pub factors_summary: String, // compact factor snapshot
    pub pending: bool,
    pub raw_return: Option<f64>,
    pub reflection: Option<String>,
}

// ═══════════════════════════════════════════════════════════
// DECISION MEMORY LOG
// ═══════════════════════════════════════════════════════════

const SEPARATOR: &str = "\n\n<!-- ENTRY_END -->\n\n";
const MAX_ENTRIES: usize = 50;

pub struct DecisionMemory {
    log_path: PathBuf,
    outcomes_dir: PathBuf,
}

impl DecisionMemory {
    pub fn new(data_dir: &Path) -> Self {
        let log_path = data_dir.join("trading_memory.md");
        let outcomes_dir = data_dir.join("outcomes");
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::create_dir_all(&outcomes_dir);
        Self { log_path, outcomes_dir }
    }

    // ═══ Phase A: Store Decision (no LLM call) ═══

    pub fn store_decision(
        &self,
        symbol: &str,
        verdict: &str,
        score: f64,
        confidence: f64,
        factors: &str,
    ) {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Idempotency guard: check if same symbol+timestamp already pending
        if self.log_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.log_path) {
                let tag_prefix = format!("[{} | {} |", &now[..10], symbol);
                if content.lines().any(|l| l.starts_with(&tag_prefix) && l.ends_with("| pending]")) {
                    tracing::debug!("⏭️ Decision already logged for {} today", symbol);
                    return;
                }
            }
        }

        let tag = format!(
            "[{now} | {symbol} | {verdict} | score:{score:.2} | conf:{confidence:.1}% | pending]"
        );
        let entry = format!(
            "{tag}\n\nFACTORS:\n{factors}\n\nDECISION:\n{verdict} {symbol} @ score={score:.2} confidence={confidence:.1}%{SEPARATOR}"
        );

        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(entry.as_bytes())
            })
        {
            tracing::error!("❌ Failed to write decision memory: {}", e);
        } else {
            tracing::info!("📝 Decision stored: {} {} (score={:.2})", verdict, symbol, score);
        }
    }

    // ═══ Phase A.5: Ingest Outcomes from Titan ═══
    // Titan writes JSON files to outcomes/ when closing positions.
    // Format: {"symbol": "BTCUSDT", "pnl": 12.5, "entry_price": 95000, "exit_price": 95500}

    pub fn ingest_outcomes(&self) {
        let entries = match std::fs::read_dir(&self.outcomes_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut processed = 0u32;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(j) => j,
                Err(_) => {
                    tracing::warn!("⚠️ Invalid outcome JSON: {:?}", path);
                    continue;
                }
            };

            let symbol = json["symbol"].as_str().unwrap_or("");
            let pnl = json["pnl"].as_f64().unwrap_or(0.0);
            let entry_price = json["entry_price"].as_f64().unwrap_or(0.0);
            let exit_price = json["exit_price"].as_f64().unwrap_or(0.0);

            if symbol.is_empty() || entry_price == 0.0 {
                tracing::warn!("⚠️ Outcome missing symbol/entry_price: {:?}", path);
                continue;
            }

            // Calculate raw return as fraction
            let raw_return = if entry_price > 0.0 {
                (exit_price - entry_price) / entry_price
            } else {
                0.0
            };

            // Auto-generate a simple reflection (no LLM needed)
            let reflection = format!(
                "Trade closed: {} entry={:.2} exit={:.2} PnL=${:.2} ({:+.2}%). {}",
                symbol, entry_price, exit_price, pnl, raw_return * 100.0,
                if pnl > 0.0 { "The directional call was correct." }
                else { "The directional call was incorrect — review conditions." }
            );

            self.update_with_outcome(symbol, raw_return, &reflection);

            // Delete processed file
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::warn!("⚠️ Failed to delete outcome file {:?}: {}", path, e);
            } else {
                processed += 1;
            }
        }

        if processed > 0 {
            tracing::info!("📊 Ingested {} trade outcomes from Titan", processed);
        }
    }

    // ═══ Phase B: Update with real PnL outcome ═══

    pub fn update_with_outcome(
        &self,
        symbol: &str,
        raw_return: f64,
        reflection: &str,
    ) {
        if !self.log_path.exists() {
            return;
        }

        let Ok(text) = std::fs::read_to_string(&self.log_path) else { return };
        let blocks: Vec<&str> = text.split(SEPARATOR).collect();

        let pending_marker = format!("| {symbol} |");
        let mut new_blocks: Vec<String> = Vec::new();
        let mut updated = false;

        for block in &blocks {
            let stripped = block.trim();
            if stripped.is_empty() {
                new_blocks.push(block.to_string());
                continue;
            }

            let first_line = stripped.lines().next().unwrap_or("");

            if !updated
                && first_line.contains(&pending_marker)
                && first_line.ends_with("| pending]")
            {
                // Replace pending tag with outcome
                let new_tag = first_line
                    .replace("| pending]", &format!("| return:{:+.2}%]", raw_return * 100.0));
                let rest: String = stripped
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n");

                new_blocks.push(format!(
                    "{new_tag}\n{rest}\n\nREFLECTION:\n{reflection}"
                ));
                updated = true;
            } else {
                new_blocks.push(block.to_string());
            }
        }

        if !updated {
            tracing::warn!("⚠️ No pending entry found for {}", symbol);
            return;
        }

        // Apply rotation: keep only MAX_ENTRIES resolved entries
        let rotated = Self::apply_rotation(&new_blocks, MAX_ENTRIES);
        let new_text = rotated.join(SEPARATOR);

        // Atomic write: tmp → replace
        let tmp_path = self.log_path.with_extension("tmp");
        if std::fs::write(&tmp_path, &new_text).is_ok() {
            let _ = std::fs::rename(&tmp_path, &self.log_path);
            tracing::info!("✅ Decision updated with outcome: {} → {:+.2}%", symbol, raw_return * 100.0);
        }
    }

    // ═══ Phase C: Get past context for prompt injection ═══

    pub fn get_past_context(&self, symbol: &str, n_same: usize, n_cross: usize) -> String {
        let entries = self.load_resolved_entries();
        if entries.is_empty() {
            return String::new();
        }

        let mut same: Vec<&ResolvedEntry> = Vec::new();
        let mut cross: Vec<&ResolvedEntry> = Vec::new();

        for entry in entries.iter().rev() {
            if same.len() >= n_same && cross.len() >= n_cross {
                break;
            }
            if entry.symbol == symbol && same.len() < n_same {
                same.push(entry);
            } else if entry.symbol != symbol && cross.len() < n_cross {
                cross.push(entry);
            }
        }

        if same.is_empty() && cross.is_empty() {
            return String::new();
        }

        let mut parts: Vec<String> = Vec::new();

        if !same.is_empty() {
            parts.push(format!("Past analyses of {symbol} (most recent first):"));
            for e in &same {
                parts.push(format!(
                    "[{} | {} | {} | {:+.2}%]\nREFLECTION: {}",
                    e.date, e.symbol, e.verdict, e.raw_return * 100.0, e.reflection
                ));
            }
        }

        if !cross.is_empty() {
            parts.push("Recent cross-symbol lessons:".into());
            for e in &cross {
                parts.push(format!(
                    "[{} | {}] {}",
                    e.date, e.symbol,
                    if e.reflection.len() > 200 { &e.reflection[..200] } else { &e.reflection }
                ));
            }
        }

        parts.join("\n\n")
    }

    // ═══ Get pending entries for a symbol ═══

    pub fn get_pending_symbols(&self) -> Vec<String> {
        if !self.log_path.exists() {
            return Vec::new();
        }
        let Ok(text) = std::fs::read_to_string(&self.log_path) else { return Vec::new() };
        let mut symbols = Vec::new();

        for line in text.lines() {
            if line.ends_with("| pending]") && line.starts_with('[') {
                // Extract symbol from tag: [date | SYMBOL | ...]
                let fields: Vec<&str> = line.trim_matches(|c| c == '[' || c == ']')
                    .split('|')
                    .map(str::trim)
                    .collect();
                if fields.len() >= 2 {
                    symbols.push(fields[1].to_string());
                }
            }
        }
        symbols.dedup();
        symbols
    }

    // ═══ Internal helpers ═══

    fn load_resolved_entries(&self) -> Vec<ResolvedEntry> {
        if !self.log_path.exists() {
            return Vec::new();
        }
        let Ok(text) = std::fs::read_to_string(&self.log_path) else { return Vec::new() };
        let blocks: Vec<&str> = text.split(SEPARATOR).collect();

        let mut entries = Vec::new();
        for block in blocks {
            let stripped = block.trim();
            if stripped.is_empty() { continue; }

            let first_line = stripped.lines().next().unwrap_or("");
            if first_line.ends_with("| pending]") || !first_line.starts_with('[') {
                continue;
            }

            // Parse tag: [date | symbol | verdict | return:+X.XX%]
            let fields: Vec<&str> = first_line
                .trim_matches(|c| c == '[' || c == ']')
                .split('|')
                .map(str::trim)
                .collect();

            if fields.len() < 4 { continue; }

            let raw_return = fields.iter()
                .find(|f| f.starts_with("return:"))
                .and_then(|f| f.trim_start_matches("return:").trim_end_matches('%').parse::<f64>().ok())
                .unwrap_or(0.0) / 100.0;

            // Extract REFLECTION section
            let reflection = if let Some(pos) = stripped.find("REFLECTION:\n") {
                stripped[pos + 12..].trim().to_string()
            } else {
                String::new()
            };

            entries.push(ResolvedEntry {
                date: fields[0].to_string(),
                symbol: fields[1].to_string(),
                verdict: fields[2].to_string(),
                raw_return,
                reflection,
            });
        }
        entries
    }

    fn apply_rotation(blocks: &[String], max_entries: usize) -> Vec<String> {
        let resolved_count = blocks.iter()
            .filter(|b| {
                let t = b.trim();
                !t.is_empty()
                    && t.lines().next().is_some_and(|l| l.starts_with('[') && !l.ends_with("| pending]"))
            })
            .count();

        if resolved_count <= max_entries {
            return blocks.to_vec();
        }

        let mut to_drop = resolved_count - max_entries;
        let mut kept = Vec::new();
        for block in blocks {
            let t = block.trim();
            let is_resolved = !t.is_empty()
                && t.lines().next().is_some_and(|l| l.starts_with('[') && !l.ends_with("| pending]"));
            if is_resolved && to_drop > 0 {
                to_drop -= 1;
                continue;
            }
            kept.push(block.clone());
        }
        kept
    }
}

#[derive(Debug)]
struct ResolvedEntry {
    date: String,
    symbol: String,
    verdict: String,
    raw_return: f64,
    reflection: String,
}

// ═══════════════════════════════════════════════════════════
// REFLECTION PROMPT (for LLM-generated self-reflection)
// ═══════════════════════════════════════════════════════════

/// Returns the system prompt for trade reflection (matches TradingAgents)
pub fn reflection_system_prompt() -> &'static str {
    "You are a crypto trading analyst reviewing your own past decision now that the outcome is known.\n\
     Write exactly 2-4 sentences of plain prose (no bullets, no headers, no markdown).\n\n\
     Cover in order:\n\
     1. Was the directional call correct? (cite the return figure)\n\
     2. Which part of the investment thesis held or failed?\n\
     3. One concrete lesson to apply to the next similar analysis.\n\n\
     Be specific and terse. Your output will be stored verbatim in a decision log \
     and re-read by future analysts, so every word must earn its place."
}

/// Builds the user message for reflection
pub fn reflection_user_message(
    verdict: &str,
    symbol: &str,
    score: f64,
    raw_return: f64,
) -> String {
    format!(
        "Raw return: {:+.2}%\n\n\
         Original Decision:\n{} {} @ score={:.2}\n\n\
         What worked? What failed? What lesson for next time?",
        raw_return * 100.0, verdict, symbol, score
    )
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_load() {
        let tmp = TempDir::new().unwrap();
        let mem = DecisionMemory::new(tmp.path());

        mem.store_decision("BTCUSDT", "BUY", 3.5, 78.0, "F1:+2 F2:+1.5 F9:+0.5");
        assert_eq!(mem.get_pending_symbols(), vec!["BTCUSDT"]);
    }

    #[test]
    fn test_update_with_outcome() {
        let tmp = TempDir::new().unwrap();
        let mem = DecisionMemory::new(tmp.path());

        mem.store_decision("ETHUSDT", "SELL", -2.1, 65.0, "F1:-1 F2:-1.5");

        mem.update_with_outcome(
            "ETHUSDT",
            0.035,  // +3.5% return
            "The sell call was correct as ETH dropped 3.5%. Funding rate squeeze signal was the key driver.",
        );

        // Should no longer be pending
        assert!(mem.get_pending_symbols().is_empty());

        // Should have context
        let ctx = mem.get_past_context("ETHUSDT", 5, 3);
        assert!(ctx.contains("ETHUSDT"));
        assert!(ctx.contains("REFLECTION"));
    }

    #[test]
    fn test_cross_symbol_context() {
        let tmp = TempDir::new().unwrap();
        let mem = DecisionMemory::new(tmp.path());

        // Store + resolve BTC
        mem.store_decision("BTCUSDT", "BUY", 2.0, 70.0, "factors");
        mem.update_with_outcome("BTCUSDT", 0.02, "BTC call was correct.");

        // Store + resolve ETH
        mem.store_decision("ETHUSDT", "SELL", -1.5, 60.0, "factors");
        mem.update_with_outcome("ETHUSDT", -0.01, "ETH sell was wrong.");

        // Get context for SOL — should include cross-symbol lessons
        let ctx = mem.get_past_context("SOLUSDT", 5, 3);
        assert!(ctx.contains("cross-symbol"));
    }

    #[test]
    fn test_rotation() {
        let blocks: Vec<String> = (0..60)
            .map(|i| format!("[2026-05-{:02} | BTC | BUY | return:+1.00%]\nDECISION:\ntest", i % 28 + 1))
            .collect();

        let rotated = DecisionMemory::apply_rotation(&blocks, 50);
        assert!(rotated.len() <= 50, "rotation failed: {} blocks", rotated.len());
    }

    #[test]
    fn test_reflection_prompt() {
        let prompt = reflection_system_prompt();
        assert!(prompt.contains("2-4 sentences"));

        let msg = reflection_user_message("BUY", "BTCUSDT", 3.5, 0.02);
        assert!(msg.contains("+2.00%"));
        assert!(msg.contains("BTCUSDT"));
    }

    #[test]
    fn test_ingest_outcomes() {
        let tmp = TempDir::new().unwrap();
        let mem = DecisionMemory::new(tmp.path());

        // Store a pending decision
        mem.store_decision("BTCUSDT", "BUY", 3.0, 80.0, "F1:+2 F9:+1");
        assert_eq!(mem.get_pending_symbols(), vec!["BTCUSDT"]);

        // Simulate Titan writing an outcome file
        let outcome = serde_json::json!({
            "symbol": "BTCUSDT",
            "entry_price": 95000.0,
            "exit_price": 95500.0,
            "pnl": 12.5,
        });
        let outcome_path = tmp.path().join("outcomes").join("BTCUSDT__1715100000.json");
        std::fs::write(&outcome_path, serde_json::to_string(&outcome).unwrap()).unwrap();

        // Ingest outcomes
        mem.ingest_outcomes();

        // Decision should no longer be pending
        assert!(mem.get_pending_symbols().is_empty(), "should have resolved pending");

        // Outcome file should be deleted
        assert!(!outcome_path.exists(), "outcome file should be deleted");

        // Context should contain reflection
        let ctx = mem.get_past_context("BTCUSDT", 5, 3);
        assert!(ctx.contains("BTCUSDT"), "context should mention symbol");
    }
}
