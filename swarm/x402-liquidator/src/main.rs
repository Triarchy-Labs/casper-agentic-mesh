mod config;
mod engine;
mod network;

#[tokio::main]
async fn main() {
    println!("[Liquidator Daemon] GIGANTOMANIA Vector 2 Online.");
    println!("[Liquidator Daemon] Scanning Mantle Init Capital for underwater human accounts...");

    network::run_liquidator_loop().await;
}
