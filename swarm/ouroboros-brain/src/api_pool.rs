use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiPool {
    keys: Vec<(String, String)>, // (API_KEY, SECRET)
    active_index: Arc<AtomicUsize>,
}

impl ApiPool {
    pub fn new(keys: Vec<(String, String)>) -> Self {
        if keys.is_empty() {
            eprintln!("FATAL: ApiPool — Нет доступных ключей API!");
            std::process::exit(1);
        }
        ApiPool {
            keys,
            active_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_current_keys(&self) -> (String, String) {
        let index = self.active_index.load(Ordering::SeqCst);
        // Защита от выхода за пределы (хотя ротация круговая)
        let safe_index = index % self.keys.len(); 
        self.keys[safe_index].clone()
    }

    pub fn rotate_keys(&self) -> usize {
        // BUG-09 FIX: atomic fetch_add (was load+store TOCTOU)
        let old_index = self.active_index.fetch_add(1, Ordering::SeqCst);
        
        (old_index + 1) % self.keys.len()
    }
    
    pub fn total_keys(&self) -> usize {
        self.keys.len()
    }
    
    #[allow(dead_code)] // Reserved for API pool diagnostics
    pub fn current_index(&self) -> usize {
        self.active_index.load(Ordering::SeqCst)
    }
}
