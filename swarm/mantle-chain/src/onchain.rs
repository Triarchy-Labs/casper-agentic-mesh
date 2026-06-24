//! On-Chain Hooks — ERC-8004 Reputation + Verdict Logging for Mantle.
//!
//! Calls the deployed ERC8004Registry contract to:
//! 1. Update agent reputation after each successful trade
//! 2. Log AI verdicts as on-chain events (calldata)
//!
//! Contract: 0x1150f09ae885e6E7BcC0cb38feDd200d7f580008 (Mantle Mainnet)
//! Agent #1: Token ID 1

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::sol;

// ═══════════════════════════════════════════════════════════
// DEPLOYED CONTRACT ADDRESSES (Mantle Mainnet)
// ═══════════════════════════════════════════════════════════

/// ERC8004Registry contract address on Mantle Mainnet.
pub const ERC8004_REGISTRY: &str = "0x1150f09ae885e6E7BcC0cb38feDd200d7f580008";

/// X402 Flash Liquidator contract address on Mantle Mainnet.
pub const FLASH_LIQUIDATOR: &str = "0x30daC056a87D5844Fb5BE47Fb5412A6Bee83072d";

/// Deployment wallet address.
pub const DEPLOYMENT_WALLET: &str = "0xF02332A7d92C86631Ea30d49D9778994B9277c79";

/// Agent #1 NFT Token ID (already minted on-chain).
pub const AGENT_TOKEN_ID: u64 = 1;

// ═══════════════════════════════════════════════════════════
// ABI BINDINGS (generated from Solidity contract)
// ═══════════════════════════════════════════════════════════

sol! {
    #[sol(rpc)]
    interface IERC8004Registry {
        function addReputation(uint256 agentId, uint256 scoreDelta) external;
        function agentReputation(uint256 agentId) external view returns (uint256);
        function agentControllers(uint256 agentId) external view returns (address);

        event ReputationUpdated(uint256 indexed agentId, uint256 newReputation);
        event AgentRegistered(uint256 indexed agentId, address indexed controller);
    }
}

// ═══════════════════════════════════════════════════════════
// ON-CHAIN REPUTATION UPDATE
// ═══════════════════════════════════════════════════════════

/// Encode the `addReputation` calldata for Agent #1.
/// Returns hex-encoded calldata suitable for sending as a transaction.
pub fn encode_add_reputation(score_delta: u64) -> Vec<u8> {
    use alloy::sol_types::SolCall;
    let call = IERC8004Registry::addReputationCall {
        agentId: U256::from(AGENT_TOKEN_ID),
        scoreDelta: U256::from(score_delta),
    };
    call.abi_encode()
}

/// Encode an AI verdict as calldata for on-chain logging.
/// The verdict data is stored as raw bytes in tx input data.
pub fn encode_verdict_log(
    symbol: &str,
    decision: &str,
    score: f64,
    confidence: f64,
    regime: &str,
    cycle: u64,
) -> Vec<u8> {
    // Magic prefix: 0xAI (hex: 0xa100) — marks this as an AI verdict log
    let mut data = vec![0xa1, 0x00];
    let json = serde_json::json!({
        "v": 4,
        "sym": symbol,
        "dec": decision,
        "s": (score * 100.0) as i64,
        "c": (confidence * 10.0) as i64,
        "r": regime,
        "n": cycle,
        "ts": chrono::Utc::now().timestamp(),
    });
    data.extend_from_slice(json.to_string().as_bytes());
    data
}

/// Read current reputation of Agent #1 from on-chain state.
pub async fn read_reputation<P: Provider>(provider: &P) -> Result<u64, String> {
    let addr: Address = ERC8004_REGISTRY.parse().map_err(|e| format!("bad addr: {e}"))?;
    use alloy::sol_types::SolCall;
    let call = IERC8004Registry::agentReputationCall {
        agentId: U256::from(AGENT_TOKEN_ID),
    };
    let calldata = call.abi_encode();

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(addr)
        .input(calldata.into());

    let result = provider.call(tx)
        .await
        .map_err(|e| format!("RPC call failed: {e}"))?;

    // Decode uint256 from ABI response (32 bytes, big-endian)
    if result.len() >= 32 {
        let rep = U256::from_be_slice(&result[result.len()-32..]);
        Ok(rep.to::<u64>())
    } else {
        Ok(0)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_add_reputation() {
        let calldata = encode_add_reputation(10);
        // ABI: function selector (4 bytes) + agentId (32 bytes) + scoreDelta (32 bytes)
        assert_eq!(calldata.len(), 4 + 32 + 32);
    }

    #[test]
    fn test_encode_verdict_log() {
        let data = encode_verdict_log("MNT", "BUY", 2.5, 75.0, "Ranging", 1);
        assert!(data.len() > 2);
        assert_eq!(data[0], 0xa1); // Magic prefix
        assert_eq!(data[1], 0x00);
        let json_str = std::str::from_utf8(&data[2..]).unwrap();
        assert!(json_str.contains("MNT"));
        assert!(json_str.contains("BUY"));
    }

    #[test]
    fn test_contract_addresses() {
        assert!(ERC8004_REGISTRY.starts_with("0x"));
        assert!(FLASH_LIQUIDATOR.starts_with("0x"));
        assert_eq!(AGENT_TOKEN_ID, 1);
    }
}
