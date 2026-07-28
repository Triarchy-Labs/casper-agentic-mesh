use core_ipc::IpcBridge;
use notify::{Watcher, RecursiveMode, EventKind};
use std::path::Path;
// std::sync::mpsc removed for async
use std::time::Duration;
use crate::config;
use crate::engine;

pub async fn run_memory_loop(db: sled::Db) -> Result<(), Box<dyn std::error::Error>> {
    let mut ipc = IpcBridge::new();
    
    std::fs::create_dir_all(config::MEMORY_DIR)?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.blocking_send(res);
    })?;
    watcher.watch(Path::new(config::MEMORY_DIR), RecursiveMode::Recursive)?;

    println!("[Memory Node] Obsidian-Vault Watcher listening on {}...", config::MEMORY_DIR);

    let mut last_timestamp = 0;

    loop {
        // 1. Process L0 IPC Experience (Liquidations)
        if let Some(state) = ipc.read_state().ok().flatten()
            && state.timestamp > last_timestamp && let Some(target) = state.liquidation_target.clone() {
                last_timestamp = state.timestamp;
                
                let edge = engine::create_liquidation_edge(target.clone(), state.global_sentiment_modifier, state.timestamp);
                let key_prefix = format!("edge:{target}");
                
                if engine::insert_edge(&db, &key_prefix, &edge).is_ok() {
                    println!("[Memory Node] 🧠 Experience Crystallized in Sled HyperGraph: SwarmX402 -> LIQUIDATED -> {target}");
                }
            }

        // 2. Process Artificial Injection (File Drops)
        if let Ok(Some(Ok(event))) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("md") {
                            let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                            println!("[Memory Node] 📥 New Knowledge Tome Detected: {file_name}");
                            if let Ok(_content) = tokio::fs::read_to_string(&path).await {
                                // Inject into HyperGraph
                                let edge = engine::create_override_edge();
                                if engine::insert_edge(&db, "tome", &edge).is_ok() {
                                    println!("[Memory Node] 🧬 Strategy Override Synthesized from Markdown into HyperGraph!");
                                    
                                    // Push to L0 IPC
                                    let mut state = ipc.read_state().ok().flatten().unwrap_or_default();
                                    state.global_sentiment_modifier += 0.5; 
                                    state.timestamp = edge.timestamp;
                                    if let Err(e) = ipc.write_state(&state) {
                                        eprintln!("[Memory Node] IPC write failed: {e}");
                                    }
                                    println!("[Memory Node] ⚡ L0 IPC State Mutated via Memory Injection.");
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
