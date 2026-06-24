//! Meta Judge — Qwen3 80B final synthesis of ALL factors.
//! Takes mechanical verdict + all agent data → independent BUY/SELL/HOLD.

use crate::config::{ModelConfig, PromptConfig};
use crate::judge::JudgeVerdict;
use crate::openrouter::OpenRouterClient;
use crate::state::SymbolData;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct MetaResult {
    pub bias: String,        // "BUY", "SELL", "HOLD"
    pub confidence: i32,     // 0-100
    pub reason: String,
    pub agrees_with_judge: bool,
    pub model_used: String,
}

#[derive(Deserialize)]
struct MetaResponse {
    bias: Option<String>,
    conf: Option<i32>,
    reason: Option<String>,
}

/// Run the Meta Judge — final arbitrator.
/// Receives the full data packet including mechanical verdict.
#[allow(clippy::too_many_arguments)]
pub async fn run_meta_judge(
    client: &OpenRouterClient,
    model: &ModelConfig,
    prompt_cfg: &PromptConfig,
    data: &SymbolData,
    mechanical: &JudgeVerdict,
    macro_bias: &str,
    ml_direction: i32,
    ml_confidence: f64,
) -> MetaResult {
    let packet = serde_json::json!({
        "sym": data.symbol,
        "px": (data.price * 100.0).round() / 100.0,
        "chg24h": (data.price_24h_change * 100.0).round() / 100.0,
        "fr": (data.funding_rate * 1_000_000.0).round() / 1_000_000.0,
        "oi_chg": (data.oi_change_pct * 10.0).round() / 10.0,
        "vol_surge": (data.volume_ratio * 10.0).round() / 10.0,
        "f7_ml": {"dir": ml_direction, "conf": (ml_confidence * 100.0).round() / 100.0},
        "f8_macro": macro_bias,
        "mech_score": mechanical.score,
        "mech_verdict": format!("{}", mechanical.decision),
    });

    let user_msg = prompt_cfg.user_template
        .replace("{packet}", &packet.to_string());

    let result = client.chat(
        model,
        &prompt_cfg.system,
        &user_msg,
        prompt_cfg.temperature,
        prompt_cfg.max_tokens,
    ).await;

    match result {
        Ok(text) => parse_meta_response(
            &text,
            &model.title,
            &data.symbol,
            &format!("{}", mechanical.decision),
        ),
        Err(e) => {
            tracing::warn!("[{}] MetaJudge [{}] failed: {}", data.symbol, model.title, e);
            MetaResult {
                bias: format!("{}", mechanical.decision),  // fallback to mechanical
                confidence: 50,
                reason: format!("Meta unavailable, using mechanical: {e}"),
                agrees_with_judge: true,
                model_used: model.title.clone(),
            }
        }
    }
}

fn parse_meta_response(
    text: &str,
    model_title: &str,
    symbol: &str,
    mech_verdict: &str,
) -> MetaResult {
    let re = Regex::new(r"\{[^}]+\}").unwrap();

    if let Some(json_match) = re.find(text) {
        if let Ok(parsed) = serde_json::from_str::<MetaResponse>(json_match.as_str()) {
            let bias = parsed.bias.unwrap_or_else(|| "HOLD".into()).to_uppercase();
            let bias = if ["BUY", "SELL", "HOLD"].contains(&bias.as_str()) {
                bias
            } else {
                "HOLD".into()
            };
            let conf = parsed.conf.unwrap_or(50).clamp(0, 100);
            let reason = parsed.reason.unwrap_or_default();
            let agrees = bias == mech_verdict;

            if !agrees {
                tracing::warn!(
                    "[{symbol}] ⚖️ META DISAGREES: judge={mech_verdict} vs meta={bias} [{model_title}]"
                );
            } else {
                tracing::info!("[{symbol}] ✅ Meta agrees: {bias} conf={conf}% [{model_title}]");
            }

            return MetaResult {
                bias,
                confidence: conf,
                reason: reason[..reason.len().min(200)].to_string(),
                agrees_with_judge: agrees,
                model_used: model_title.into(),
            };
        }
    }

    tracing::warn!("[{symbol}] MetaJudge [{model_title}]: unparseable");
    MetaResult {
        bias: mech_verdict.into(),
        confidence: 50,
        reason: "Unparseable, fallback to mechanical".into(),
        agrees_with_judge: true,
        model_used: model_title.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_buy_response() {
        let text = r#"{"bias":"BUY","conf":75,"reason":"negative funding indicates squeeze"}"#;
        let r = parse_meta_response(text, "Qwen3", "BTCUSDT", "BUY");
        assert_eq!(r.bias, "BUY");
        assert_eq!(r.confidence, 75);
        assert!(r.agrees_with_judge);
    }

    #[test]
    fn test_parse_disagree() {
        let text = r#"{"bias":"SELL","conf":80,"reason":"overbought"}"#;
        let r = parse_meta_response(text, "Qwen3", "ETHUSDT", "BUY");
        assert_eq!(r.bias, "SELL");
        assert!(!r.agrees_with_judge);
    }

    #[test]
    fn test_parse_garbage() {
        let text = "I think the market will go up tomorrow because reasons.";
        let r = parse_meta_response(text, "Qwen3", "SOLUSDT", "HOLD");
        assert_eq!(r.bias, "HOLD"); // fallback
        assert!(r.agrees_with_judge);
    }

    #[test]
    fn test_parse_with_markdown_wrapper() {
        let text = "```json\n{\"bias\":\"BUY\",\"conf\":65,\"reason\":\"squeeze\"}\n```";
        let r = parse_meta_response(text, "Qwen3", "BTCUSDT", "SELL");
        assert_eq!(r.bias, "BUY");
        assert!(!r.agrees_with_judge);
    }
}
