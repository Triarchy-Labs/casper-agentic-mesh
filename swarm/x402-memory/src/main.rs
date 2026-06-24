mod config;
mod engine;
mod network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Memory Node] GIGANTOMANIA Vector 6 Online.");
    
    let db = sled::open(config::DB_PATH)?;
    println!("[Memory Node] Ultra-Fast Sled HyperGraph Initialized at {}", config::DB_PATH);

    network::run_memory_loop(db).await?;

    Ok(())
}
