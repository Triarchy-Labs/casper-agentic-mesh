//! Macro Judge — GPT-OSS 120B independent bias assessment.
//! Architecturally separated from debate pool (different model = no weight-bias).

use crate::config::{ModelConfig, PromptConfig};
use crate::openrouter::OpenRouterClient;
use crate::state::SymbolData;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct MacroBiasResult {
    pub bias: String,       // "BULLISH", "BEARISH", "NEUTRAL"
    pub reason: String,
    pub model_used: String,
}

#[derive(Deserialize)]
struct BiasResponse {
    bias: Option<String>,
    reason: Option<String>,
}

/// Query the independent macro judge for bias assessment.
/// Uses a DEDICATED model (not from debate pool) to avoid weight-bias.
pub async fn fetch_macro_bias(
    client: &OpenRouterClient,
    model: &ModelConfig,
    prompt_cfg: &PromptConfig,
    data: &SymbolData,
) -> MacroBiasResult {
    let fr_label = if data.funding_rate < -0.0003 {
        "shorts paying heavily"
    } else if data.funding_rate > 0.0005 {
        "longs overheated"
    } else {
        "neutral"
    };

    let user_msg = prompt_cfg.user_template
        .replace("{symbol}", &data.symbol)
        .replace("{price}", &format!("{:.0}", data.price))
        .replace("{change}", &format!("{:.1}", data.price_24h_change))
        .replace("{fr}", &format!("{:.6}", data.funding_rate))
        .replace("{fr_label}", fr_label)
        .replace("{oi_delta}", &format!("{:.1}", data.oi_change_pct))
        .replace("{vol_surge}", &format!("{:.1}", data.volume_ratio));

    let result = client.chat(
        model,
        &prompt_cfg.system,
        &user_msg,
        prompt_cfg.temperature,
        prompt_cfg.max_tokens,
    ).await;

    match result {
        Ok(text) => parse_bias_response(&text, &model.title, &data.symbol),
        Err(e) => {
            tracing::warn!("[{}] MacroJudge [{}] failed: {}", data.symbol, model.title, e);
            MacroBiasResult {
                bias: "NEUTRAL".into(),
                reason: format!("Judge unavailable: {e}"),
                model_used: model.title.clone(),
            }
        }
    }
}

fn parse_bias_response(text: &str, model_title: &str, symbol: &str) -> MacroBiasResult {
    let re = Regex::new(r"\{[^}]+\}").unwrap();

    if let Some(json_match) = re.find(text) {
        if let Ok(parsed) = serde_json::from_str::<BiasResponse>(json_match.as_str()) {
            let bias = parsed.bias.unwrap_or_else(|| "NEUTRAL".into()).to_uppercase();
            let bias = if ["BULLISH", "BEARISH", "NEUTRAL"].contains(&bias.as_str()) {
                bias
            } else {
                "NEUTRAL".into()
            };
            let reason = parsed.reason.unwrap_or_default();
            tracing::info!("[{symbol}] MacroJudge [{model_title}]: {bias} | {reason}");

            return MacroBiasResult {
                bias,
                reason: reason[..reason.len().min(200)].to_string(),
                model_used: model_title.into(),
            };
        }
    }

    tracing::warn!("[{symbol}] MacroJudge [{model_title}]: unparseable: {}", &text[..text.len().min(100)]);
    MacroBiasResult {
        bias: "NEUTRAL".into(),
        reason: "Unparseable response".into(),
        model_used: model_title.into(),
    }
}
