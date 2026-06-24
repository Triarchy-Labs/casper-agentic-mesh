use std::collections::HashMap;
use std::io::Read;
use crate::entity::MemoryEntity;

/// Загружает снапшот памяти.
/// Приоритет: bincode+zstd (быстрый) → JSON (fallback для backward compat).
pub fn load_snapshot(json_path: &str) -> HashMap<String, MemoryEntity> {
    let bin_path = json_path.replace(".json", ".bin");
    
    // 1. Попытка загрузить бинарный снапшот (быстро)
    if let Ok(compressed) = std::fs::read(&bin_path) {
        if let Ok(mut decoder) = zstd::Decoder::new(compressed.as_slice()) {
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                if let Ok(graph) = bincode::deserialize::<HashMap<String, MemoryEntity>>(&decompressed) {
                    println!("[SNAPSHOT RECOVERY] Binary snapshot loaded: {} assets ({} KB compressed).",
                        graph.len(), compressed.len() / 1024);
                    return graph;
                }
            }
        }
        println!("[SNAPSHOT WARN] Binary snapshot corrupted, falling back to JSON.");
    }

    // 2. Fallback: JSON (совместимость со старыми версиями)
    if let Ok(content) = std::fs::read_to_string(json_path) {
        if let Ok(graph) = serde_json::from_str::<HashMap<String, MemoryEntity>>(&content) {
            println!("[SNAPSHOT RECOVERY] JSON snapshot loaded: {} assets.", graph.len());
            return graph;
        }
    }

    println!("[GENESIS] No snapshot found. Starting with clean slate.");
    HashMap::new()
}

/// Сохраняет снапшот памяти в ОБА формата:
/// - JSON (для backward compat + человеко-читаемый)
/// - bincode+zstd (для быстрого старта)
pub fn save_snapshot(graph: &HashMap<String, MemoryEntity>, json_path: &str) {
    let bin_path = json_path.replace(".json", ".bin");

    // 1. JSON (backward compat)
    if let Ok(json_str) = serde_json::to_string_pretty(graph) {
        let _ = std::fs::write(json_path, &json_str);
    }

    // 2. bincode + zstd (быстрый бинарный)
    if let Ok(encoded) = bincode::serialize(graph) {
        // zstd compression level 3 (быстро, хорошее сжатие)
        if let Ok(compressed) = zstd::encode_all(encoded.as_slice(), 3) {
            let _ = std::fs::write(&bin_path, &compressed);
            let ratio = if !compressed.is_empty() {
                encoded.len() as f64 / compressed.len() as f64
            } else { 1.0 };
            println!("[SNAPSHOT] Saved: {} assets | JSON: {} KB | Binary: {} KB (ratio {:.1}x)",
                graph.len(),
                std::fs::metadata(json_path).map(|m| m.len() / 1024).unwrap_or(0),
                compressed.len() / 1024,
                ratio);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::MemoryEntity;
    use std::collections::HashMap;

    #[test]
    fn test_snapshot_roundtrip_json() {
        let mut graph = HashMap::new();
        let mut entity = MemoryEntity::new("BTCUSDT");
        entity.trade_count = 10;
        entity.net_pnl = 42.5;
        entity.profit_factor = 2.1;
        graph.insert("BTCUSDT".to_string(), entity);

        let tmp = std::env::temp_dir().join("test_hive_snapshot.json");
        let path = tmp.to_str().unwrap();
        
        save_snapshot(&graph, path);
        let loaded = load_snapshot(path);
        
        assert_eq!(loaded.len(), 1);
        let bt = loaded.get("BTCUSDT").unwrap();
        assert_eq!(bt.trade_count, 10);
        assert!((bt.net_pnl - 42.5).abs() < 1e-10);
        assert!((bt.profit_factor - 2.1).abs() < 1e-10);
        
        // Cleanup
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.replace(".json", ".bin"));
    }

    #[test]
    fn test_binary_smaller_than_json() {
        let mut graph = HashMap::new();
        for i in 0..20 {
            let mut entity = MemoryEntity::new(&format!("SYM{}USDT", i));
            entity.trade_count = i as i32 * 5;
            entity.net_pnl = (i as f64) * 3.14;
            graph.insert(entity.entity_id.clone(), entity);
        }

        let tmp = std::env::temp_dir().join("test_size_compare.json");
        let path = tmp.to_str().unwrap();
        
        save_snapshot(&graph, path);
        
        let json_size = std::fs::metadata(path).unwrap().len();
        let bin_size = std::fs::metadata(path.replace(".json", ".bin")).unwrap().len();
        
        assert!(bin_size < json_size, 
            "Binary ({} bytes) should be smaller than JSON ({} bytes)", 
            bin_size, json_size);
        
        // Cleanup
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.replace(".json", ".bin"));
    }

    #[test]
    fn test_load_empty_returns_empty() {
        let loaded = load_snapshot("/nonexistent/path/no_file.json");
        assert!(loaded.is_empty());
    }
}
