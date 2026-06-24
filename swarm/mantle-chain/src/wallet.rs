//! Wallet Signer — Alloy-based transaction signing and broadcast for Mantle.
//!
//! Provides signed transaction construction and broadcast to Mantle Mainnet.
//! Uses the deployment wallet from Phase 1 (D2166).
//!
//! WARNING: Private key loaded from MANTLE_PRIVATE_KEY env var.
//! Never hardcode keys. Use .env + .gitignore.

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;

use crate::onchain::{encode_add_reputation, encode_verdict_log, ERC8004_REGISTRY, AGENT_TOKEN_ID};

/// Create a signed provider (can send transactions) for Mantle.
///
/// Requires MANTLE_PRIVATE_KEY env var.
/// Returns a provider with EthereumWallet signer attached.
pub fn create_signed_provider(
    rpc_url: &str,
) -> Result<impl Provider, String> {
    let key_hex = std::env::var("MANTLE_PRIVATE_KEY")
        .map_err(|_| "MANTLE_PRIVATE_KEY not set".to_string())?;

    let signer: PrivateKeySigner = key_hex.parse()
        .map_err(|e| format!("Invalid private key: {e}"))?;

    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(rpc_url.parse().map_err(|e| format!("Invalid RPC URL: {e}"))?);

    Ok(provider)
}

/// Broadcast an `addReputation` transaction to the ERC8004Registry.
///
/// This is called after each successful AI trade to record the agent's
/// decision quality on-chain as a reputation score delta.
pub async fn broadcast_reputation<P: Provider>(
    provider: &P,
    score_delta: u64,
) -> Result<String, String> {
    let registry_addr: Address = ERC8004_REGISTRY.parse()
        .map_err(|e| format!("bad registry addr: {e}"))?;

    let calldata = encode_add_reputation(score_delta);

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(registry_addr)
        .input(calldata.into())
        .value(U256::ZERO);

    let pending = provider.send_transaction(tx)
        .await
        .map_err(|e| format!("send_transaction failed: {e}"))?;

    let tx_hash = format!("{:?}", pending.tx_hash());
    tracing::info!("⛓️  TX SENT: addReputation(agent={}, delta={}) → {}",
        AGENT_TOKEN_ID, score_delta, tx_hash);

    Ok(tx_hash)
}

/// Broadcast a verdict log as a self-addressed transaction with calldata.
///
/// This records the AI's decision as immutable on-chain data.
/// The transaction is sent to the agent's own wallet (self-transfer with data).
pub async fn broadcast_verdict<P: Provider>(
    provider: &P,
    wallet_addr: &str,
    symbol: &str,
    decision: &str,
    score: f64,
    confidence: f64,
    regime: &str,
    cycle: u64,
) -> Result<String, String> {
    let to_addr: Address = wallet_addr.parse()
        .map_err(|e| format!("bad wallet addr: {e}"))?;

    let calldata = encode_verdict_log(symbol, decision, score, confidence, regime, cycle);

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(to_addr)
        .input(calldata.into())
        .value(U256::ZERO);

    let pending = provider.send_transaction(tx)
        .await
        .map_err(|e| format!("send_transaction failed: {e}"))?;

    let tx_hash = format!("{:?}", pending.tx_hash());
    tracing::info!("⛓️  TX SENT: verdict_log({} {} s={:.2} c={:.1}%) → {}",
        symbol, decision, score, confidence, tx_hash);

    Ok(tx_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_signed_provider_missing_key() {
        // Should fail without env var
        std::env::remove_var("MANTLE_PRIVATE_KEY");
        let result = create_signed_provider("https://rpc.mantle.xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_signed_provider_with_key() {
        // Use a throwaway test key (never use in production)
        std::env::set_var("MANTLE_PRIVATE_KEY",
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        let result = create_signed_provider("https://rpc.mantle.xyz");
        assert!(result.is_ok());
        std::env::remove_var("MANTLE_PRIVATE_KEY");
    }
}
