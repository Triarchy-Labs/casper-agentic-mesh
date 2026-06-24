mod config;
mod engine;
mod network;

#[tokio::main]
async fn main() {
    println!("[Consensus Node] GIGANTOMANIA Vector 5 Online.");
    
    network::run_consensus_loop().await;
}
