// src/modules/safe_io.rs
// ═══════════════════════════════════════════════════════════════
// SAFE I/O — Atomic file writes + crash-resistant persistence
// ═══════════════════════════════════════════════════════════════
// Pattern: write to .tmp → rename to final (atomic on NTFS/ext4).
// If crash during write → old file survives. No corruption possible.

use std::path::{Path, PathBuf};

/// Return the project-local data directory for persistent state files.
///
/// Resolution order:
/// 1. `$SWARM_DATA_DIR` env var (explicit override for deployment)
/// 2. `$CARGO_MANIFEST_DIR/../../data` (workspace-relative, development)
/// 3. `./data` (fallback — works in Docker/Vercel/CI)
///
/// The directory is lazily created on first access.
pub fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("SWARM_DATA_DIR") {
        let p = PathBuf::from(d);
        let _ = std::fs::create_dir_all(&p);
        return p;
    }
    // Try workspace-relative (crates/titan-core/../../data = project-root/data)
    let manifest = option_env!("CARGO_MANIFEST_DIR");
    if let Some(m) = manifest {
        let ws = PathBuf::from(m).join("../../data");
        if let Ok(canonical) = ws.canonicalize() {
            return canonical;
        }
        // Dir doesn't exist yet — create it
        let target = PathBuf::from(m).join("..").join("..").join("data");
        let _ = std::fs::create_dir_all(&target);
        return target;
    }
    // Absolute fallback
    let p = PathBuf::from("./data");
    let _ = std::fs::create_dir_all(&p);
    p
}

/// Resolve a filename within the project data directory.
///
/// Maps legacy Windows paths like `E:\ROXY_SYSTEM\...\foo.json` → `data/foo.json`.
/// Accepts just the filename: `data_file("titan_ramp_state.json")`.
pub fn data_file(name: &str) -> String {
    data_dir().join(name).to_string_lossy().to_string()
}

pub struct SafeIO;

impl SafeIO {
    /// Atomic write: data → path.tmp → rename to path
    /// Returns Ok(()) on success, Err on failure (old file preserved)
    pub fn atomic_write(path: &str, data: &str) -> std::io::Result<()> {
        let tmp_path = format!("{path}.tmp");
        std::fs::write(&tmp_path, data)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Atomic write with JSON pretty-print
    #[allow(dead_code)]
    pub fn atomic_write_json<T: serde::Serialize>(path: &str, value: &T) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Self::atomic_write(path, &json)
    }

    /// Safe read with fallback: if main file corrupted, try .bak
    #[allow(dead_code)]
    pub fn safe_read(path: &str) -> Option<String> {
        // Try main file first
        if let Ok(content) = std::fs::read_to_string(path) {
            if !content.is_empty() {
                return Some(content);
            }
        }
        // Fallback to backup
        let bak_path = format!("{path}.bak");
        if Path::new(&bak_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&bak_path) {
                tracing::warn!("[SAFE-IO] Main file corrupted, loaded backup: {}", path);
                return Some(content);
            }
        }
        None
    }

    /// Atomic write with automatic backup of previous version
    #[allow(dead_code)]
    pub fn atomic_write_with_backup(path: &str, data: &str) -> std::io::Result<()> {
        // Backup current version
        let bak_path = format!("{path}.bak");
        if Path::new(path).exists() {
            let _ = std::fs::copy(path, &bak_path);
        }
        // Atomic write new version
        Self::atomic_write(path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> String {
        std::env::temp_dir().join(name).to_string_lossy().to_string()
    }

    #[test]
    fn test_atomic_write_creates_file() {
        let path = temp_path("titan_test_atomic.json");
        let _ = fs::remove_file(&path);
        
        SafeIO::atomic_write(&path, r#"{"test": true}"#).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("test"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_atomic_write_no_tmp_left() {
        let path = temp_path("titan_test_notmp.json");
        let tmp = format!("{}.tmp", &path);
        
        SafeIO::atomic_write(&path, "data").unwrap();
        
        assert!(!Path::new(&tmp).exists(), ".tmp should be renamed away");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_safe_read_main_file() {
        let path = temp_path("titan_test_read.json");
        fs::write(&path, "hello").unwrap();
        
        let content = SafeIO::safe_read(&path);
        assert_eq!(content, Some("hello".to_string()));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_safe_read_fallback_to_backup() {
        let path = temp_path("titan_test_bak.json");
        let bak = format!("{}.bak", &path);
        let _ = fs::remove_file(&path);
        fs::write(&bak, "backup_data").unwrap();
        
        let content = SafeIO::safe_read(&path);
        assert_eq!(content, Some("backup_data".to_string()));
        let _ = fs::remove_file(&bak);
    }

    #[test]
    fn test_atomic_write_with_backup() {
        let path = temp_path("titan_test_wbak.json");
        let bak = format!("{}.bak", &path);
        
        SafeIO::atomic_write(&path, "v1").unwrap();
        SafeIO::atomic_write_with_backup(&path, "v2").unwrap();
        
        assert_eq!(fs::read_to_string(&path).unwrap(), "v2");
        assert_eq!(fs::read_to_string(&bak).unwrap(), "v1");
        
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&bak);
    }
}
