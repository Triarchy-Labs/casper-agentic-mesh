use core_ipc::IpcBridge;
use notify::{Watcher, RecursiveMode, EventKind};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use crate::config;
use crate::engine;

pub async fn run_memory_loop(db: sled::Db) -> Result<(), Box<dyn std::error::Error>> {
    let mut ipc = IpcBridge::new();
    
    std::fs::create_dir_all(config::MEMORY_DIR)?;

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new(config::MEMORY_DIR), RecursiveMode::Recursive)?;

    println!("[Memory Node] Obsidian-Vault Watcher listening on {}...", config::MEMORY_DIR);

    let mut last_timestamp = 0;

    loop {
        // 1. Process L0 IPC Experience (Liquidations)
        if let Some(state) = ipc.read_state()
            && state.timestamp > last_timestamp && state.liquidation_target.is_some() {
                last_timestamp = state.timestamp;
                let target = state.liquidation_target.clone().unwrap();
                
                let edge = engine::create_liquidation_edge(target.clone(), state.global_sentiment_modifier, state.timestamp);
                let key_prefix = format!("edge:{target}");
                
                if engine::insert_edge(&db, &key_prefix, &edge).is_ok() {
                    println!("[Memory Node] 🧠 Experience Crystallized in Sled HyperGraph: SwarmX402 -> LIQUIDATED -> {target}");
                }
            }

        // 2. Process Artificial Injection (File Drops)
        if let Ok(Ok(event)) = rx.recv_timeout(Duration::from_millis(100)) {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("md") {
                            println!("[Memory Node] 📥 New Knowledge Tome Detected: {}", path.file_name().unwrap().to_string_lossy());
                            if let Ok(_content) = std::fs::read_to_string(&path) {
                                // Inject into HyperGraph
                                let edge = engine::create_override_edge();
                                if engine::insert_edge(&db, "tome", &edge).is_ok() {
                                    println!("[Memory Node] 🧬 Strategy Override Synthesized from Markdown into HyperGraph!");
                                    
                                    // Push to L0 IPC
                                    let mut state = ipc.read_state().unwrap_or_default();
                                    state.global_sentiment_modifier += 0.5; 
                                    state.timestamp = edge.timestamp;
                                    ipc.write_state(&state);
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
