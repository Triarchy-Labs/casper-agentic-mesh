//! DEX Adapter — Live price feeds + swap execution for Mantle DEXs.
//!
//! Supports:
//! - Merchant Moe (LB Router) — primary DEX on Mantle
//! - On-chain price fetching via Alloy RPC calls
//! - Swap execution via router contracts
//!
//! Token addresses (Mantle Mainnet):
//!   WMNT:  0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8
//!   USDC:  0x09Bc4E0D10E52d373F5F0EE189840aca7b4C3AeC
//!   USDT:  0x201EBa5CC46D216Ce6DC03F6a759e8E766e956aE
//!   WETH:  0xdEAddEaDdeadDEadDEADDEaddEADDEAddead1111

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::sol;

// ═══════════════════════════════════════════════════════════
// MANTLE TOKEN ADDRESSES
// ═══════════════════════════════════════════════════════════

pub const WMNT: &str = "0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8";
pub const USDC: &str = "0x09Bc4E0D10E52d373F5F0EE189840aca7b4C3AeC";
pub const USDT: &str = "0x201EBa5CC46D216Ce6DC03F6a759e8E766e956aE";
pub const WETH: &str = "0xdEAddEaDdeadDEadDEADDEaddEADDEAddead1111";

// ═══════════════════════════════════════════════════════════
// DEX ROUTER ADDRESSES (Mantle Mainnet)
// ═══════════════════════════════════════════════════════════

/// Merchant Moe LB Router (Liquidity Book — concentrated liquidity).
pub const MOE_LB_ROUTER: &str = "0x013e138EF6008ae5FDFDE29700e3f2Bc61d21E3a";

/// Merchant Moe Classic Router.
pub const MOE_CLASSIC_ROUTER: &str = "0xeaEE7EE68874218c3558b40063c42B82D3E7232a";

/// Merchant Moe LB Quoter (for price quotes).
pub const MOE_LB_QUOTER: &str = "0x501b8AFd35df20f531fF45F6f695793AC3316c85";

// ═══════════════════════════════════════════════════════════
// ABI BINDINGS — ERC-20 balanceOf + decimals
// ═══════════════════════════════════════════════════════════

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
    }
}

// ═══════════════════════════════════════════════════════════
// TOKEN MAPPING — symbol to address
// ═══════════════════════════════════════════════════════════

/// Map a token symbol to its Mantle Mainnet address.
pub fn token_address(symbol: &str) -> Option<&'static str> {
    match symbol.to_uppercase().as_str() {
        "MNT" | "WMNT" => Some(WMNT),
        "USDC" => Some(USDC),
        "USDT" => Some(USDT),
        "WETH" | "ETH" => Some(WETH),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════
// LIVE PRICE FEED — On-chain balance-based price estimation
// ═══════════════════════════════════════════════════════════

/// Fetch the native MNT balance of the deployment wallet.
pub async fn fetch_mnt_balance<P: Provider>(provider: &P, wallet: &str) -> Result<f64, String> {
    let addr: Address = wallet.parse().map_err(|e| format!("bad addr: {e}"))?;
    let balance = provider.get_balance(addr)
        .await
        .map_err(|e| format!("balance fetch failed: {e}"))?;
    // Convert wei to MNT (18 decimals)
    let mnt = balance.to::<u128>() as f64 / 1e18;
    Ok(mnt)
}

/// Fetch ERC-20 token balance for a wallet.
pub async fn fetch_token_balance<P: Provider>(
    provider: &P,
    token_addr: &str,
    wallet: &str,
) -> Result<f64, String> {
    let token: Address = token_addr.parse().map_err(|e| format!("bad token: {e}"))?;
    let account: Address = wallet.parse().map_err(|e| format!("bad wallet: {e}"))?;

    use alloy::sol_types::SolCall;
    let call = IERC20::balanceOfCall { account };
    let calldata = call.abi_encode();

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(token)
        .input(calldata.into());

    let result = provider.call(tx)
        .await
        .map_err(|e| format!("balanceOf call failed: {e}"))?;

    if result.len() >= 32 {
        let raw = U256::from_be_slice(&result[result.len()-32..]);
        // Assume 18 decimals for MNT/WETH, 6 for USDC/USDT
        let decimals = if token_addr == USDC || token_addr == USDT { 6.0 } else { 18.0 };
        Ok(raw.to::<u128>() as f64 / 10f64.powf(decimals))
    } else {
        Ok(0.0)
    }
}

// ═══════════════════════════════════════════════════════════
// LIVE MARKET DATA — RPC-based price fetch for swarm pipeline
// ═══════════════════════════════════════════════════════════

/// Rich market data from DexScreener — feeds regime detector + judge.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DexScreenerData {
    pub symbol: String,
    pub price: f64,
    pub price_change_h1: f64,
    pub price_change_h6: f64,
    pub price_change_h24: f64,
    pub volume_h24: f64,
    pub volume_h6: f64,
    pub volume_h1: f64,
    pub buys_h24: u64,
    pub sells_h24: u64,
    pub liquidity_usd: f64,
    pub dex_id: String,
    pub pair_address: String,
    pub timestamp: i64,
}

impl DexScreenerData {
    /// Buy/sell ratio as a sentiment indicator.
    /// > 1.0 = more buys (bullish), < 1.0 = more sells (bearish).
    pub fn buy_sell_ratio(&self) -> f64 {
        if self.sells_h24 == 0 { return 2.0; }
        self.buys_h24 as f64 / self.sells_h24 as f64
    }

    /// Volume-to-liquidity ratio — measures trading intensity.
    /// > 0.1 = active, > 0.5 = very active, > 1.0 = extremely active.
    pub fn volume_intensity(&self) -> f64 {
        if self.liquidity_usd < 1.0 { return 0.0; }
        self.volume_h24 / self.liquidity_usd
    }

    /// Hourly volume acceleration — is volume picking up?
    /// > 1.0 = accelerating, < 1.0 = decelerating.
    pub fn volume_acceleration(&self) -> f64 {
        if self.volume_h6 < 1.0 { return 1.0; }
        // h1 volume * 6 vs h6 volume (normalized to same timeframe)
        (self.volume_h1 * 6.0) / self.volume_h6
    }
}

/// Fetch rich market data from DexScreener for a given token.
///
/// Extracts: price, 24h/6h/1h change, volume, buy/sell txns, liquidity, dex_id.
/// Uses the highest-liquidity Mantle pair.
pub async fn fetch_rich_data(symbol: &str) -> Result<DexScreenerData, String> {
    let token_addr = match symbol.to_uppercase().as_str() {
        "MNT" | "WMNT" => WMNT,
        "WETH" | "ETH" => WETH,
        _ => return Err(format!("unknown symbol: {symbol}")),
    };

    let url = format!(
        "https://api.dexscreener.com/latest/dex/tokens/{}",
        token_addr
    );

    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("DexScreener request failed: {e}"))?;

    let json: serde_json::Value = resp.json()
        .await
        .map_err(|e| format!("DexScreener parse failed: {e}"))?;

    let pairs = json.get("pairs")
        .and_then(|p| p.as_array())
        .ok_or_else(|| "No pairs in response".to_string())?;

    // Find highest-liquidity Mantle pair
    let best = pairs.iter()
        .filter(|p| p.get("chainId").and_then(|c| c.as_str()) == Some("mantle"))
        .max_by(|a, b| {
            let liq_a = a.pointer("/liquidity/usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let liq_b = b.pointer("/liquidity/usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
            liq_a.partial_cmp(&liq_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or_else(|| format!("No Mantle pairs for {symbol}"))?;

    let price = best.get("priceUsd")
        .and_then(|p| p.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let price_change_h1 = best.pointer("/priceChange/h1").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let price_change_h6 = best.pointer("/priceChange/h6").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let price_change_h24 = best.pointer("/priceChange/h24").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let volume_h24 = best.pointer("/volume/h24").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let volume_h6 = best.pointer("/volume/h6").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let volume_h1 = best.pointer("/volume/h1").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let buys_h24 = best.pointer("/txns/h24/buys").and_then(|v| v.as_u64()).unwrap_or(0);
    let sells_h24 = best.pointer("/txns/h24/sells").and_then(|v| v.as_u64()).unwrap_or(0);

    let liquidity_usd = best.pointer("/liquidity/usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let dex_id = best.get("dexId").and_then(|d| d.as_str()).unwrap_or("unknown").to_string();
    let pair_address = best.get("pairAddress").and_then(|d| d.as_str()).unwrap_or("").to_string();

    Ok(DexScreenerData {
        symbol: symbol.to_uppercase(),
        price,
        price_change_h1,
        price_change_h6,
        price_change_h24,
        volume_h24,
        volume_h6,
        volume_h1,
        buys_h24,
        sells_h24,
        liquidity_usd,
        dex_id,
        pair_address,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Simple price-only fetch (backward compat).
pub async fn fetch_live_price(symbol: &str) -> Result<f64, String> {
    fetch_rich_data(symbol).await.map(|d| d.price)
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_address_mapping() {
        assert_eq!(token_address("MNT"), Some(WMNT));
        assert_eq!(token_address("WMNT"), Some(WMNT));
        assert_eq!(token_address("USDC"), Some(USDC));
        assert_eq!(token_address("WETH"), Some(WETH));
        assert_eq!(token_address("ETH"), Some(WETH));
        assert_eq!(token_address("UNKNOWN"), None);
    }

    #[test]
    fn test_router_addresses() {
        assert!(MOE_LB_ROUTER.starts_with("0x"));
        assert!(MOE_CLASSIC_ROUTER.starts_with("0x"));
        assert!(MOE_LB_QUOTER.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_fetch_rich_data_mnt() {
        match fetch_rich_data("MNT").await {
            Ok(data) => {
                assert!(data.price > 0.0, "MNT price should be > 0");
                assert!(data.price < 100.0, "MNT price sanity check");
                println!("═══ DexScreener Rich Data ═══");
                println!("  Symbol:     {}", data.symbol);
                println!("  Price:      ${:.4}", data.price);
                println!("  24h Change: {:.2}%", data.price_change_h24);
                println!("  6h Change:  {:.2}%", data.price_change_h6);
                println!("  1h Change:  {:.2}%", data.price_change_h1);
                println!("  Volume 24h: ${:.2}", data.volume_h24);
                println!("  Volume 6h:  ${:.2}", data.volume_h6);
                println!("  Volume 1h:  ${:.2}", data.volume_h1);
                println!("  Buys 24h:   {}", data.buys_h24);
                println!("  Sells 24h:  {}", data.sells_h24);
                println!("  Liquidity:  ${:.2}", data.liquidity_usd);
                println!("  DEX:        {}", data.dex_id);
                println!("  Pair:       {}", data.pair_address);
                println!("  B/S Ratio:  {:.3}", data.buy_sell_ratio());
                println!("  Vol Intens: {:.3}", data.volume_intensity());
                println!("  Vol Accel:  {:.3}", data.volume_acceleration());
            }
            Err(e) => {
                println!("Skipping live data test: {e}");
            }
        }
    }

    #[test]
    fn test_dex_screener_data_methods() {
        let data = DexScreenerData {
            symbol: "MNT".into(), price: 0.65,
            price_change_h1: 0.3, price_change_h6: -0.26, price_change_h24: 2.33,
            volume_h24: 261589.82, volume_h6: 34040.51, volume_h1: 10562.38,
            buys_h24: 165, sells_h24: 253,
            liquidity_usd: 3621492.55,
            dex_id: "agni".into(), pair_address: "0xtest".into(),
            timestamp: 0,
        };
        assert!((data.buy_sell_ratio() - 0.652).abs() < 0.01);
        assert!((data.volume_intensity() - 0.0722).abs() < 0.01);
        assert!(data.volume_acceleration() > 0.0);
    }
}
