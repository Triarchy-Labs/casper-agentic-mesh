//! IPC — JSON output for downstream trading bots.
//! Writes consensus result to file that Titan/bots can read.

use crate::judge::JudgeVerdict;
use crate::agents::debater::DebateResult;
use crate::agents::macro_judge::MacroBiasResult;
use crate::agents::meta_judge::MetaResult;
use crate::state::SymbolData;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct SwarmOutput {
    pub symbol: String,
    pub timestamp: i64,
    pub cycle: u64,

    // Judge verdict
    pub decision: String,
    pub confidence: f64,
    pub score: f64,

    // Debate
    pub bull_argument: String,
    pub bear_argument: String,
    pub bull_model: String,
    pub bear_model: String,

    // Macro bias
    pub macro_bias: String,
    pub macro_reason: String,

    // Meta judge
    pub meta_bias: String,
    pub meta_confidence: i32,
    pub meta_agrees: bool,

    // Raw data
    pub price: f64,
    pub price_change_24h: f64,
    pub funding_rate: f64,
    pub oi_change_pct: f64,
    pub volume_ratio: f64,
}

impl SwarmOutput {
    pub fn build(
        data: &SymbolData,
        cycle: u64,
        verdict: &JudgeVerdict,
        debate: &DebateResult,
        macro_result: &MacroBiasResult,
        meta: &MetaResult,
    ) -> Self {
        Self {
            symbol: data.symbol.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            cycle,

            decision: format!("{}", verdict.decision),
            confidence: verdict.confidence,
            score: verdict.score,

            bull_argument: debate.bull_argument.clone(),
            bear_argument: debate.bear_argument.clone(),
            bull_model: debate.bull_model.clone(),
            bear_model: debate.bear_model.clone(),

            macro_bias: macro_result.bias.clone(),
            macro_reason: macro_result.reason.clone(),

            meta_bias: meta.bias.clone(),
            meta_confidence: meta.confidence,
            meta_agrees: meta.agrees_with_judge,

            price: data.price,
            price_change_24h: data.price_24h_change,
            funding_rate: data.funding_rate,
            oi_change_pct: data.oi_change_pct,
            volume_ratio: data.volume_ratio,
        }
    }
}

/// Write output JSON to both symbol-specific and latest files,
/// plus the shared Titan IPC file (ouroboros_verdicts.json).
pub fn write_output(output: &SwarmOutput, base_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(base_dir)?;

    let json = serde_json::to_string_pretty(output)?;

    // Per-symbol file (e.g., BTCUSDT.json)
    let symbol_path = base_dir.join(format!("{}.json", output.symbol));
    std::fs::write(&symbol_path, &json)?;

    // Latest file (always overwritten — latest cycle for any symbol)
    let latest_path = base_dir.join("latest.json");
    std::fs::write(&latest_path, &json)?;

    // ─── Shared IPC for Titan (brain_feeds.rs reads this) ───
    // Format must match: json[symbol]["verdict"] + json[symbol]["timestamp"] (RFC3339)
    let shared_path = PathBuf::from(
        std::env::var("OUROBOROS_IPC_PATH")
            .unwrap_or_else(|_| "data/ouroboros_verdicts.json".to_string())
    );
    let _ = write_shared_verdicts(output, &shared_path);

    tracing::info!(
        "[{}] 📝 IPC: {} conf={:.0}% score={:.2} → {:?}",
        output.symbol, output.decision, output.confidence,
        output.score, symbol_path
    );

    Ok(())
}

/// Write/merge per-symbol verdict into the shared Titan IPC file.
/// Titan's brain_feeds.rs::read_ouroboros_verdict() expects:
///   { "BTCUSDT": { "verdict": "BUY", "timestamp": "2026-05-08T00:00:00Z", ... } }
fn write_shared_verdicts(output: &SwarmOutput, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read existing file (may contain other symbols)
    let mut verdicts: std::collections::HashMap<String, serde_json::Value> =
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

    // Insert/update this symbol's verdict
    verdicts.insert(output.symbol.clone(), serde_json::json!({
        "verdict": output.decision,
        "confidence": output.confidence,
        "score": output.score,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "macro_bias": output.macro_bias,
        "meta_agrees": output.meta_agrees,
        "cycle": output.cycle,
    }));

    std::fs::write(path, serde_json::to_string_pretty(&verdicts)?)?;
    tracing::debug!("[{}] 📡 Shared IPC updated → {:?}", output.symbol, path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Verdict;

    #[test]
    fn test_build_output() {
        let data = SymbolData {
            symbol: "BTCUSDT".into(),
            price: 96000.0,
            price_24h_change: 2.5,
            volume_24h: 1_000_000.0,
            volume_ratio: 1.5,
            funding_rate: -0.0005,
            open_interest: 500_000.0,
            oi_change_pct: 3.2,
            timestamp: 0,
        };
        let verdict = JudgeVerdict { decision: Verdict::Buy, confidence: 75.0, score: 2.5 };
        let debate = DebateResult {
            bull_argument: "test bull".into(),
            bear_argument: "test bear".into(),
            bull_model: "Hermes".into(),
            bear_model: "Hermes".into(),
        };
        let macro_r = MacroBiasResult {
            bias: "BULLISH".into(), reason: "test".into(), model_used: "GPT".into(),
        };
        let meta = MetaResult {
            bias: "BUY".into(), confidence: 70, reason: "test".into(),
            agrees_with_judge: true, model_used: "Qwen".into(),
        };

        let output = SwarmOutput::build(&data, 1, &verdict, &debate, &macro_r, &meta);
        assert_eq!(output.decision, "BUY");
        assert_eq!(output.score, 2.5);
        assert!(output.meta_agrees);
    }

    #[test]
    fn test_write_output() {
        let output = SwarmOutput {
            symbol: "TEST".into(), timestamp: 0, cycle: 1,
            decision: "HOLD".into(), confidence: 30.0, score: 0.5,
            bull_argument: "t".into(), bear_argument: "t".into(),
            bull_model: "m".into(), bear_model: "m".into(),
            macro_bias: "NEUTRAL".into(), macro_reason: "".into(),
            meta_bias: "HOLD".into(), meta_confidence: 50, meta_agrees: true,
            price: 100.0, price_change_24h: 0.0, funding_rate: 0.0,
            oi_change_pct: 0.0, volume_ratio: 1.0,
        };
        let dir = std::env::temp_dir().join("ouroboros_v2_test");
        write_output(&output, &dir).expect("write failed");

        let read = std::fs::read_to_string(dir.join("TEST.json")).expect("read failed");
        assert!(read.contains("HOLD"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
