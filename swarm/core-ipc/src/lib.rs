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

    pub fn write_state(&mut self, state: &AgentState) {
        self.file.lock_exclusive().unwrap();
        let encoded = bincode::serialize(state).unwrap();
        // Zero out the buffer
        self.mmap[..].fill(0);
        // Write the new data length as a u32, followed by the data
        let len = encoded.len() as u32;
        self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
        self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
        self.mmap.flush().unwrap();
        self.file.unlock().unwrap();
    }

    pub fn read_state(&self) -> Option<AgentState> {
        self.file.lock_shared().unwrap();
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.mmap[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;

        if len == 0 || len > IPC_SIZE - 4 {
            self.file.unlock().unwrap();
            return None;
        }

        let decoded: Result<AgentState, _> = bincode::deserialize(&self.mmap[4..4 + len]);
        self.file.unlock().unwrap();
        decoded.ok()
    }

    /// Read-Modify-Write atomic update
    pub fn update_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut AgentState),
    {
        self.file.lock_exclusive().unwrap();
        
        let mut state = {
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&self.mmap[0..4]);
            let len = u32::from_le_bytes(len_bytes) as usize;

            if len == 0 || len > IPC_SIZE - 4 {
                AgentState::default()
            } else {
                bincode::deserialize(&self.mmap[4..4 + len]).unwrap_or_default()
            }
        };

        f(&mut state);

        let encoded = bincode::serialize(&state).unwrap();
        self.mmap[..].fill(0);
        let len = encoded.len() as u32;
        self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
        self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
        self.mmap.flush().unwrap();
        
        self.file.unlock().unwrap();
    }
}
