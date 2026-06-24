mod config;
mod engine;
mod network;
pub mod bounty;

#[tokio::main]
async fn main() {
    println!("[Sniper Agent] Panopticon Node 1 Online.");
    println!("[Sniper Agent] Listening to L0 IPC Mmap Bridge at 0-latency...");

    network::run_sniper_loop().await;
}
