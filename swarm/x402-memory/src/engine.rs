use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct HyperEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub sentiment: f64,
    pub timestamp: u64,
}

pub fn create_liquidation_edge(target: String, sentiment: f64, timestamp: u64) -> HyperEdge {
    HyperEdge {
        source: "SwarmX402".to_string(),
        target,
        relation: "LIQUIDATED".to_string(),
        sentiment,
        timestamp,
    }
}

pub fn create_override_edge() -> HyperEdge {
    HyperEdge {
        source: "Creator".to_string(),
        target: "SwarmStrategy".to_string(),
        relation: "OVERRIDE_INJECTED".to_string(),
        sentiment: 1.0,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(std::time::Duration::from_secs(0)).as_secs(),
    }
}

pub fn insert_edge(db: &sled::Db, key_prefix: &str, edge: &HyperEdge) -> Result<(), Box<dyn std::error::Error>> {
    let edge_bytes = bincode::serialize(edge)?;
    let key = format!("{}:{}", key_prefix, edge.timestamp);
    db.insert(key, edge_bytes)?;
    db.flush()?;
    Ok(())
}
