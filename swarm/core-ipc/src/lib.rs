use memmap2::{MmapMut, MmapOptions};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::path::Path;
use fs2::FileExt;

pub const IPC_FILE: &str = "/tmp/x402_ipc.mmap";
pub const IPC_SIZE: usize = 4096;

#[derive(Debug)]
pub enum IpcError {
    Io(std::io::Error),
    Lock(String),
    Serialize(String),
    Overflow { encoded: usize, capacity: usize },
    Deserialize(String),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Io(e) => write!(f, "IPC I/O error: {e}"),
            IpcError::Lock(ctx) => write!(f, "IPC file lock failed: {ctx}"),
            IpcError::Serialize(e) => write!(f, "IPC serialize error: {e}"),
            IpcError::Overflow { encoded, capacity } => {
                write!(f, "IPC state too large: {encoded} bytes > {capacity} capacity")
            }
            IpcError::Deserialize(e) => write!(f, "IPC deserialize error: {e}"),
        }
    }
}

impl std::error::Error for IpcError {}

impl From<std::io::Error> for IpcError {
    fn from(e: std::io::Error) -> Self {
        IpcError::Io(e)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentState {
    pub sniper_vote: Option<bool>,
    pub risk_vote: Option<bool>,
    pub liquidation_target: Option<String>,
    pub global_sentiment_modifier: f64,
    pub timestamp: u64,
}

/// RAII guard that ensures file unlock on drop — prevents deadlocks
/// even when early returns or panics occur inside locked sections.
struct FileGuard<'a> {
    file: &'a File,
}

impl<'a> FileGuard<'a> {
    fn exclusive(file: &'a File) -> Result<Self, IpcError> {
        file.lock_exclusive().map_err(|e| IpcError::Lock(format!("exclusive: {e}")))?;
        Ok(Self { file })
    }

    fn shared(file: &'a File) -> Result<Self, IpcError> {
        file.lock_shared().map_err(|e| IpcError::Lock(format!("shared: {e}")))?;
        Ok(Self { file })
    }
}

impl<'a> Drop for FileGuard<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            tracing::error!("IPC file unlock failed during drop: {e}");
        }
    }
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
    /// Create a new IPC bridge. Returns an error if the mmap file
    /// cannot be created or memory-mapped — the caller decides how to handle it.
    pub fn try_new() -> Result<Self, IpcError> {
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

    /// Convenience constructor that logs the error and panics if IPC
    /// cannot be initialized — because an agent without IPC is useless.
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|e| {
            panic!("[core-ipc] FATAL: cannot initialize IPC bridge at {IPC_FILE}: {e}")
        })
    }

    /// Write agent state to the shared memory region.
    /// Returns `Ok(())` on success, or a descriptive `IpcError` on failure.
    pub fn write_state(&mut self, state: &AgentState) -> Result<(), IpcError> {
        let _guard = FileGuard::exclusive(&self.file)?;

        let encoded = bincode::serialize(state)
            .map_err(|e| IpcError::Serialize(e.to_string()))?;

        if encoded.len() + 4 > IPC_SIZE {
            return Err(IpcError::Overflow {
                encoded: encoded.len() + 4,
                capacity: IPC_SIZE,
            });
        }

        self.mmap[..].fill(0);
        let len = encoded.len() as u32;
        self.mmap[0..4].copy_from_slice(&len.to_le_bytes());
        self.mmap[4..4 + encoded.len()].copy_from_slice(&encoded);
        self.mmap.flush()?;
        Ok(())
        // _guard dropped here → file.unlock() guaranteed
    }

    /// Read agent state from the shared memory region.
    /// Returns `None` if the region is empty, `Err` if corrupted or locked.
    pub fn read_state(&self) -> Result<Option<AgentState>, IpcError> {
        let _guard = FileGuard::shared(&self.file)?;

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.mmap[0..4]);
        let len = u32::from_le_bytes(len_bytes) as usize;

        if len == 0 {
            return Ok(None); // empty region, not an error
        }

        if len > IPC_SIZE - 4 {
            return Err(IpcError::Deserialize(format!(
                "corrupt length header: {len} exceeds buffer capacity {}", IPC_SIZE - 4
            )));
        }

        let state: AgentState = bincode::deserialize(&self.mmap[4..4 + len])
            .map_err(|e| IpcError::Deserialize(e.to_string()))?;
        Ok(Some(state))
        // _guard dropped here → file.unlock() guaranteed
    }
}
