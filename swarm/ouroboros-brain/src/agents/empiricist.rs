//! Empiricist Agent — fetches ENRICHED market data from Bybit API.
//! 3 parallel API calls per symbol: ticker + funding history + OI history.
//! Public endpoints — no API key needed.

use crate::state::SymbolData;
use reqwest::Client;
use serde::Deserialize;

// ═══════════════════════════════════════════════════════════
// BYBIT API RESPONSE TYPES
// ═══════════════════════════════════════════════════════════

#[derive(Deserialize)]
struct BybitResponse<T> {
    #[serde(rename = "retCode")]
    ret_code: i32,
    result: Option<T>,
}

// ─── Ticker ───
#[derive(Deserialize)]
struct TickerResult {
    list: Vec<TickerItem>,
}

#[derive(Deserialize)]
struct TickerItem {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "price24hPcnt")]
    price_24h_pcnt: String,
    #[serde(rename = "volume24h")]
    volume_24h: String,
    #[serde(rename = "turnover24h")]
    turnover_24h: String,
    #[serde(rename = "fundingRate")]
    funding_rate: String,
    #[serde(rename = "openInterest")]
    open_interest: String,
}

// ─── Funding Rate History ───
#[derive(Deserialize)]
struct FundingResult {
    list: Vec<FundingItem>,
}

#[derive(Deserialize)]
struct FundingItem {
    #[serde(rename = "fundingRate")]
    funding_rate: String,
}

// ─── Open Interest History ───
#[derive(Deserialize)]
struct OiResult {
    list: Vec<OiItem>,
}

#[derive(Deserialize)]
struct OiItem {
    #[serde(rename = "openInterest")]
    open_interest: String,
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

// ═══════════════════════════════════════════════════════════
// ENRICHED FETCH — 3 concurrent API calls
// ═══════════════════════════════════════════════════════════

/// Fetch FULL enriched market data: ticker + funding trend + OI delta.
/// Mirrors Python's `fetch_enriched_market_data()` exactly.
pub async fn fetch_market_data(
    client: &Client,
    symbol: &str,
    prev_turnover: f64,
) -> Result<SymbolData, Box<dyn std::error::Error + Send + Sync>> {
    // ─── 3 parallel API calls ───
    let client1 = client.clone();
    let client2 = client.clone();
    let client3 = client.clone();
    let sym1 = symbol.to_string();
    let sym2 = symbol.to_string();
    let sym3 = symbol.to_string();

    let ticker_fut = tokio::spawn(async move {
        let url = format!(
            "https://api.bybit.com/v5/market/tickers?category=linear&symbol={sym1}"
        );
        client1.get(&url).timeout(TIMEOUT).send().await?.json::<BybitResponse<TickerResult>>().await
    });

    let funding_fut = tokio::spawn(async move {
        let url = format!(
            "https://api.bybit.com/v5/market/funding/history?category=linear&symbol={sym2}&limit=2"
        );
        client2.get(&url).timeout(TIMEOUT).send().await?.json::<BybitResponse<FundingResult>>().await
    });

    let oi_fut = tokio::spawn(async move {
        let url = format!(
            "https://api.bybit.com/v5/market/open-interest?category=linear&symbol={sym3}&intervalTime=1h&limit=2"
        );
        client3.get(&url).timeout(TIMEOUT).send().await?.json::<BybitResponse<OiResult>>().await
    });

    // ─── Await all ───
    let ticker_resp = ticker_fut.await??;
    let funding_resp = funding_fut.await.ok();
    let oi_resp = oi_fut.await.ok();

    // ─── Parse ticker (required) ───
    if ticker_resp.ret_code != 0 {
        return Err(format!("Bybit ticker error: retCode={}", ticker_resp.ret_code).into());
    }
    let result = ticker_resp.result.ok_or("Empty ticker result")?;
    let ticker = result.list.first().ok_or("No ticker data")?;

    let price: f64 = ticker.last_price.parse().unwrap_or(0.0);
    let change: f64 = ticker.price_24h_pcnt.parse::<f64>().unwrap_or(0.0) * 100.0;
    let _volume: f64 = ticker.volume_24h.parse().unwrap_or(0.0);
    let turnover: f64 = ticker.turnover_24h.parse().unwrap_or(0.0);
    let funding: f64 = ticker.funding_rate.parse().unwrap_or(0.0);
    let oi_current: f64 = ticker.open_interest.parse().unwrap_or(0.0);

    // ─── Parse funding history (optional, graceful) ───
    let funding_final = if let Some(Ok(fr)) = funding_resp {
        if let Some(ref res) = fr.result {
            if let Some(first) = res.list.first() {
                first.funding_rate.parse::<f64>().unwrap_or(funding)
            } else { funding }
        } else { funding }
    } else { funding };

    // ─── Parse OI delta (optional, graceful) ───
    let oi_change_pct = if let Some(Ok(oi_r)) = oi_resp {
        if let Some(ref res) = oi_r.result {
            if res.list.len() >= 2 {
                let oi_now: f64 = res.list[0].open_interest.parse().unwrap_or(0.0);
                let oi_prev: f64 = res.list[1].open_interest.parse().unwrap_or(0.0);
                if oi_prev > 0.0 {
                    ((oi_now - oi_prev) / oi_prev) * 100.0
                } else { 0.0 }
            } else { 0.0 }
        } else { 0.0 }
    } else { 0.0 };

    // ─── Volume surge: current turnover vs previous cycle ───
    let volume_ratio = if turnover > 0.0 && prev_turnover > 0.0 {
        (turnover / prev_turnover).clamp(0.1, 10.0) // cap extreme ratios
    } else {
        1.0 // first cycle or missing data → neutral
    };

    Ok(SymbolData {
        symbol: ticker.symbol.clone(),
        price,
        price_24h_change: change,
        volume_24h: turnover, // Use turnover (USD) not coin volume
        volume_ratio,
        funding_rate: funding_final,
        open_interest: oi_current,
        oi_change_pct,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Fetch data for multiple symbols concurrently.
/// Uses SwarmState to compute volume_ratio vs previous cycle.
pub async fn fetch_all_symbols(
    client: &Client,
    symbols: &[String],
    prev_volumes: &std::collections::HashMap<String, f64>,
) -> Vec<SymbolData> {
    let mut handles = Vec::new();

    for symbol in symbols {
        let client = client.clone();
        let sym = symbol.clone();
        let prev_vol = prev_volumes.get(symbol).copied().unwrap_or(0.0);
        handles.push(tokio::spawn(async move {
            match fetch_market_data(&client, &sym, prev_vol).await {
                Ok(data) => {
                    tracing::info!(
                        "[{}] 📊 ${:.0} {:+.1}% FR={:.6} OI={:.0} OI_Δ={:+.1}%",
                        data.symbol, data.price, data.price_24h_change,
                        data.funding_rate, data.open_interest, data.oi_change_pct
                    );
                    Some(data)
                }
                Err(e) => {
                    tracing::error!("[{sym}] Bybit fetch failed: {e}");
                    None
                }
            }
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Some(data)) = handle.await {
            results.push(data);
        }
    }
    results
}
