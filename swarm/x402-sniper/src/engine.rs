pub fn calculate_leverage_multiplier(sentiment: f64) -> f64 {
    1.0 + sentiment
}

pub async fn execute_flash_loan_tx(
    target: &str,
    sentiment: f64,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("[Sniper Agent] Constructing transaction to Casper Escrow Contract for target {}...", target);
    
    let sentiment_score = (sentiment * 100.0) as u64;
    println!("[Sniper Agent] Using sentiment score {} for execution.", sentiment_score);

    // Simulated Casper transaction execution
    let tx_hash = "fake_casper_tx_hash_0xabcdef";
    println!("[Sniper Agent] AI Inference written on-chain! Transaction broadcasted! Hash: {}", tx_hash);

    Ok(tx_hash.to_string())
}
