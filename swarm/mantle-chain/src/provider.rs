//! Mantle RPC Provider — Alloy-based connection to Mantle L2.

use alloy::providers::ProviderBuilder;

/// Mantle Mainnet chain ID.
pub const MANTLE_CHAIN_ID: u64 = 5000;

/// Mantle Testnet chain ID.
pub const MANTLE_TESTNET_CHAIN_ID: u64 = 5003;

/// Default Mantle Mainnet RPC.
pub const MANTLE_RPC: &str = "https://rpc.mantle.xyz";

/// Default Mantle Testnet RPC.
pub const MANTLE_TESTNET_RPC: &str = "https://rpc.sepolia.mantle.xyz";

/// Create an Alloy HTTP provider for Mantle.
pub fn create_provider(rpc_url: &str) -> impl alloy::providers::Provider {
    ProviderBuilder::new()
        .connect_http(rpc_url.parse().expect("Invalid RPC URL"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let _provider = create_provider(MANTLE_RPC);
    }

    #[test]
    fn test_chain_constants() {
        assert_eq!(MANTLE_CHAIN_ID, 5000);
        assert_eq!(MANTLE_TESTNET_CHAIN_ID, 5003);
    }
}
