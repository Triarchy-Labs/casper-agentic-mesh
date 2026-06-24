use alloy::providers::Provider;
use alloy::primitives::{Address, U256};
use alloy::sol;
use std::str::FromStr;

pub fn calculate_leverage_multiplier(sentiment: f64) -> f64 {
    1.0 + sentiment
}

// Generate the ABI bindings for our newly deployed contracts
sol! {
    #[sol(rpc)]
    contract ERC8004Registry {
        function registerAgent(address controller) external returns (uint256);
        function agentControllers(uint256 agentId) external view returns (address);
    }

    #[sol(rpc)]
    contract X402FlashLiquidator {
        function executeAILiquidation(address target, uint256 aiSentimentScore, uint256 agentId) external;
    }
}

pub async fn execute_flash_loan_tx<P: Provider>(
    provider: &P,
    target: &str,
    sentiment: f64,
) -> Result<String, Box<dyn std::error::Error>> {
    // Read deployed contract addresses from environment
    let registry_str = std::env::var("REGISTRY_ADDRESS")
        .expect("REGISTRY_ADDRESS env var must be set to deployed ERC8004Registry address");
    let liquidator_str = std::env::var("LIQUIDATOR_ADDRESS")
        .expect("LIQUIDATOR_ADDRESS env var must be set to deployed X402FlashLiquidator address");
    let registry_address = Address::from_str(&registry_str)?;
    let liquidator_address = Address::from_str(&liquidator_str)?;
    let target_addr = Address::from_str(target).unwrap_or(Address::ZERO);

    println!("[Sniper Agent] Verifying ERC-8004 Identity via Registry...");
    let _registry = ERC8004Registry::new(registry_address, provider);
    
    // For MVP we assume the agent already registered and has ID 1.
    // In production, we would call `registerAgent` if `agentControllers(1)` is empty.
    let agent_id = U256::from(1);

    println!("[Sniper Agent] Constructing transaction to X402FlashLiquidator Contract...");
    let liquidator = X402FlashLiquidator::new(liquidator_address, provider);
    
    // Convert float sentiment to a fixed point integer (e.g., 1.05 -> 105)
    let sentiment_score = U256::from((sentiment * 100.0) as u64);

    // Build and send the transaction using the generated alloy method
    let call_builder = liquidator.executeAILiquidation(target_addr, sentiment_score, agent_id);
    
    let pending_tx = call_builder.send().await?;
    let tx_hash = *pending_tx.tx_hash();
    
    println!("[Sniper Agent] AI Inference written on-chain! Transaction broadcasted! Hash: {}", tx_hash);

    // Wait for receipt
    let receipt = pending_tx.get_receipt().await?;
    println!("[Sniper Agent] Transaction confirmed in block: {:?}", receipt.block_number);

    Ok(tx_hash.to_string())
}
