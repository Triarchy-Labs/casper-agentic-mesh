use memmap2::{MmapMut, MmapOptions};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::path::Path;
use fs2::FileExt;

pub const IPC_FILE: &str = "/tmp/x402_ipc.mmap";
pub const IPC_SIZE: usize = 4096;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentState {
    pub sniper_vote: Option<bool>,
    pub risk_vote: Option<bool>,
    pub consensus_reached: Option<bool>,
    pub liquidation_target: Option<String>,
    pub global_sentiment_modifier: f64,
    pub timestamp: u64,
}

pub struct IpcBridge {
    mmap: MmapMut,
    file: File,
}

impl Default for IpcBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcBridge {
    pub fn new() -> Self {
        let path = Path::new(IPC_FILE);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .unwrap();

        file.set_len(IPC_SIZE as u64).unwrap();

        let mmap = unsafe { MmapOptions::new().map_mut(&file).unwrap() };
        Self { mmap, file }
    }

    fn acquire_exclusive_lock(&self) -> Result<(), String> {
        let mut backoff = 10;
        for _ in 0..5 {
            if self.file.try_lock_exclusive().is_ok() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(backoff));
            backoff *= 2;
        }
        Err("Failed to acquire exclusive IPC lock after 5 retries".into())
    }

    fn acquire_shared_lock(&self) -> Result<(), String> {
        let mut backoff = 10;
        for _ in 0..5 {
            if self.file.try_lock_shared().is_ok() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(backoff));
            backoff *= 2;
        }
        Err("Failed to acquire shared IPC lock after 5 retries".into())
    }

    pub fn write_state(&mut self, state: &AgentState) {
        if let Err(e) = self.acquire_exclusive_lock() {
            eprintln!("[IPC Error] {}", e);
            return;
        }
        
        if let Ok(encoded) = bincode::serialize(state) {
            self.mmap[..].fill(0);
            let len = encoded.len() as u32;
            self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
            self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
            let _ = self.mmap.flush();
        }
        
        let _ = self.file.unlock();
    }

    pub fn read_state(&self) -> Option<AgentState> {
        if let Err(e) = self.acquire_shared_lock() {
            eprintln!("[IPC Error] {}", e);
            return None;
        }
        
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.mmap[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;

        if len == 0 || len > IPC_SIZE - 4 {
            let _ = self.file.unlock();
            return None;
        }

        let decoded: Result<AgentState, _> = bincode::deserialize(&self.mmap[4..4 + len]);
        let _ = self.file.unlock();
        decoded.ok()
    }

    /// Read-Modify-Write atomic update
    pub fn update_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut AgentState),
    {
        if let Err(e) = self.acquire_exclusive_lock() {
            eprintln!("[IPC Error] update_state: {}", e);
            return;
        }
        
        let mut state = {
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&self.mmap[0..4]);
            let len = u32::from_le_bytes(len_bytes) as usize;

            if len == 0 || len > IPC_SIZE - 4 {
                AgentState::default()
            } else {
                match bincode::deserialize(&self.mmap[4..4 + len]) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[IPC Warning] State corruption detected. Not overwriting state from zero. Err: {}", e);
                        let _ = self.file.unlock();
                        return;
                    }
                }
            }
        };

        f(&mut state);

        if let Ok(encoded) = bincode::serialize(&state) {
            self.mmap[..].fill(0);
            let len = encoded.len() as u32;
            self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
            self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
            let _ = self.mmap.flush();
        }
        
        let _ = self.file.unlock();
    }
}
