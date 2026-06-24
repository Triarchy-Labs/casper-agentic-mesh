//! ERC-8004 Registry — on-chain agent identity interaction.
//!
//! Deployed on Mantle Mainnet:
//!   Registry: 0x1150f09ae885e6E7BcC0cb38feDd200d7f580008
//!   Liquidator: 0x30daC056a87D5844Fb5BE47Fb5412A6Bee83072d

use alloy::sol;

// Generate Rust bindings from Solidity ABI
sol! {
    #[sol(rpc)]
    interface IERC8004Registry {
        function registerAgent(address controller) external returns (uint256 agentId);
        function addReputation(uint256 agentId, uint256 scoreDelta) external;
        function agentReputation(uint256 agentId) external view returns (uint256);
        function agentControllers(uint256 agentId) external view returns (address);

        event AgentRegistered(uint256 indexed agentId, address indexed controller);
        event ReputationUpdated(uint256 indexed agentId, uint256 newReputation);
    }

    #[sol(rpc)]
    interface IX402FlashLiquidator {
        function executeAILiquidation(address target, uint256 aiSentimentScore, uint256 agentId) external;

        event LiquidationExecuted(
            uint256 indexed agentId,
            address indexed target,
            uint256 aiSentimentScore,
            bool success
        );
    }
}

/// Deployed contract addresses on Mantle Mainnet.
pub mod addresses {
    /// ERC-8004 Registry — agent identity NFTs.
    pub const ERC8004_REGISTRY: &str = "0x1150f09ae885e6E7BcC0cb38feDd200d7f580008";

    /// X402 Flash Liquidator — AI execution engine.
    pub const FLASH_LIQUIDATOR: &str = "0x30daC056a87D5844Fb5BE47Fb5412A6Bee83072d";

    /// Deployment wallet.
    pub const DEPLOYER: &str = "0xF02332A7d92C86631Ea30d49D9778994B9277c79";

    /// Agent NFT Token ID.
    pub const AGENT_TOKEN_ID: u64 = 1;
}

#[cfg(test)]
mod tests {
    use super::addresses::*;

    #[test]
    fn test_addresses_valid_hex() {
        assert!(ERC8004_REGISTRY.starts_with("0x"));
        assert_eq!(ERC8004_REGISTRY.len(), 42);
        assert!(FLASH_LIQUIDATOR.starts_with("0x"));
        assert_eq!(FLASH_LIQUIDATOR.len(), 42);
        assert!(DEPLOYER.starts_with("0x"));
        assert_eq!(DEPLOYER.len(), 42);
    }

    #[test]
    fn test_agent_token_id() {
        assert_eq!(AGENT_TOKEN_ID, 1);
    }
}
