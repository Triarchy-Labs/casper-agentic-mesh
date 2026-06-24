#![allow(dead_code)] // Pending pipeline integration
mod config;
mod engine;
mod network;

#[tokio::main]
async fn main() {
    println!("[Risk Agent] GIGANTOMANIA Vector 3 Online.");
    
    network::run_risk_loop().await;
}
