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
    pub fn try_new() -> Result<Self, std::io::Error> {
        let path = Path::new(IPC_FILE);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        file.set_len(IPC_SIZE as u64)?;

        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self { mmap, file })
    }

    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|_| {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/null")
                .or_else(|_| File::open("/dev/null"))
                .or_else(|_| File::create("/tmp/x402_ipc_fallback.mmap"))
                .unwrap_or_else(|_| panic!("Failed to open fallback file"));
            let mmap = MmapMut::map_anon(IPC_SIZE).unwrap_or_else(|_| panic!("Failed to map memory"));
            Self { mmap, file }
        })
    }

    pub fn write_state(&mut self, state: &AgentState) {
        if self.file.lock_exclusive().is_ok() {
            if let Ok(encoded) = bincode::serialize(state) {
                if encoded.len() + 4 <= IPC_SIZE {
                    self.mmap[..].fill(0);
                    let len = encoded.len() as u32;
                    self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
                    self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
                    let _ = self.mmap.flush();
                }
            }
            let _ = self.file.unlock();
        }
    }

    pub fn read_state(&self) -> Option<AgentState> {
        if self.file.lock_shared().is_err() {
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
}
