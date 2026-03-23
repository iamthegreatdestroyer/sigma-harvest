//! ΣCORE: Hyperdimensional Vector Symbolic Architecture.
//!
//! Provides sub-linear associative memory, non-linear dynamics-based prediction,
//! evolutionary swarm agents, and compressed knowledge storage.
//! This is the "nervous system" of ΣHARVEST.

pub mod compression;
pub mod dynamics;
pub mod memory;
pub mod swarm;
pub mod vectors;

use serde::{Deserialize, Serialize};

/// Global ΣCORE status summary, exposed via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaCoreStatus {
    pub memory_entries: usize,
    pub memory_bytes: usize,
    pub active_agents: usize,
    pub attractor_strength: f64,
    pub compression_ratio: f64,
    pub dynamics_enabled: bool,
}

impl Default for SigmaCoreStatus {
    fn default() -> Self {
        Self {
            memory_entries: 0,
            memory_bytes: 0,
            active_agents: 0,
            attractor_strength: 0.0,
            compression_ratio: 1.0,
            dynamics_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigma_core_status_default() {
        let status = SigmaCoreStatus::default();
        assert_eq!(status.memory_entries, 0);
        assert_eq!(status.active_agents, 0);
    }

    #[test]
    fn sigma_core_status_serializable() {
        let status = SigmaCoreStatus {
            memory_entries: 42,
            memory_bytes: 1024,
            active_agents: 6,
            attractor_strength: 0.85,
            compression_ratio: 0.3,
            dynamics_enabled: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        let deser: SigmaCoreStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.memory_entries, 42);
        assert_eq!(deser.active_agents, 6);
        assert!((deser.attractor_strength - 0.85).abs() < f64::EPSILON);
    }
}
