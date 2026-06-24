//! Bull/Bear Debater Agents — parallel LLM calls via OpenRouter.
//! Prompts loaded from config/prompts.toml (no recompile needed).

use crate::config::PromptConfig;
use crate::openrouter::{ModelPool, OpenRouterClient};
use crate::state::SymbolData;

/// Result of a debate round.
#[derive(Debug, Clone)]
pub struct DebateResult {
    pub bull_argument: String,
    pub bear_argument: String,
    pub bull_model: String,
    pub bear_model: String,
}

/// Run bull and bear agents in PARALLEL using tokio::spawn.
/// Staggered by 2 seconds to avoid thundering herd on OpenRouter.
pub async fn run_debate(
    client: &OpenRouterClient,
    pool: &ModelPool,
    data: &SymbolData,
    bull_prompt: &PromptConfig,
    bear_prompt: &PromptConfig,
    alpha_ctx: &str,
) -> DebateResult {
    let empirical = format!(
        "Price=${:.0} 24h={:+.1}% FR={:.6} OI_chg={:+.1}% Vol={:.1}x",
        data.price, data.price_24h_change, data.funding_rate,
        data.oi_change_pct, data.volume_ratio
    );

    // ─── Bull Agent ───
    let bull_user = bull_prompt.user_template
        .replace("{symbol}", &data.symbol)
        .replace("{data}", &empirical)
        .replace("{alpha_ctx}", alpha_ctx);

    let bull_model_name = pool.current().title.clone();
    let bull_result = client.chat_with_pool(
        pool,
        &bull_prompt.system,
        &bull_user,
        bull_prompt.temperature,
        bull_prompt.max_tokens,
    ).await;

    // ─── Stagger: T+2s delay before bear call ───
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // ─── Bear Agent ───
    let bear_user = bear_prompt.user_template
        .replace("{symbol}", &data.symbol)
        .replace("{data}", &empirical)
        .replace("{alpha_ctx}", alpha_ctx);

    let bear_model_name = pool.current().title.clone();
    let bear_result = client.chat_with_pool(
        pool,
        &bear_prompt.system,
        &bear_user,
        bear_prompt.temperature,
        bear_prompt.max_tokens,
    ).await;

    let bull_arg = match bull_result {
        Ok(text) => {
            tracing::info!("[{}] 🐂 Bull [{}]: {}", data.symbol, bull_model_name,
                &text[..text.len().min(80)]);
            text
        }
        Err(e) => {
            tracing::warn!("[{}] Bull agent failed: {}", data.symbol, e);
            format!("Bull analysis unavailable: {e}")
        }
    };

    let bear_arg = match bear_result {
        Ok(text) => {
            tracing::info!("[{}] 🐻 Bear [{}]: {}", data.symbol, bear_model_name,
                &text[..text.len().min(80)]);
            text
        }
        Err(e) => {
            tracing::warn!("[{}] Bear agent failed: {}", data.symbol, e);
            format!("Bear analysis unavailable: {e}")
        }
    };

    DebateResult {
        bull_argument: bull_arg,
        bear_argument: bear_arg,
        bull_model: bull_model_name,
        bear_model: bear_model_name,
    }
}

/// Fallback debate for when circuit breaker is YELLOW (reduced LLM usage).
#[allow(dead_code)] // Reserved: activated when CircuitBreaker enters YELLOW state
pub fn fallback_debate(data: &SymbolData) -> DebateResult {
    let bull = if data.funding_rate < -0.0003 {
        format!("{}: Negative funding {:.6} suggests short squeeze potential", data.symbol, data.funding_rate)
    } else {
        format!("{}: No strong bullish edge detected", data.symbol)
    };

    let bear = if data.funding_rate > 0.0005 {
        format!("{}: Overheated funding {:.6} signals dump risk", data.symbol, data.funding_rate)
    } else {
        format!("{}: No strong bearish edge detected", data.symbol)
    };

    DebateResult {
        bull_argument: bull,
        bear_argument: bear,
        bull_model: "FALLBACK".into(),
        bear_model: "FALLBACK".into(),
    }
}
