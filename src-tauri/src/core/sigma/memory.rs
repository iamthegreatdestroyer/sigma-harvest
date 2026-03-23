//! Sub-linear associative memory using HD vectors.
//!
//! Content-addressable storage with O(N) similarity search where N is the number
//! of stored entries. Each entry is a packed binary HD vector (32 bytes for dim=256),
//! making this extremely memory-efficient.
//!
//! The "success attractor" is an evolving HD vector representing the ideal opportunity
//! profile, continuously updated through evolutionary reinforcement.

use super::vectors::{Codebook, HdVector, DEFAULT_DIM};
use serde::{Deserialize, Serialize};

/// A labeled entry in associative memory.
#[derive(Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Human-readable label for this entry.
    pub label: String,
    /// The HD vector representation.
    pub vector: HdVector,
    /// Metadata tags (e.g., chain, type, outcome).
    pub tags: Vec<String>,
    /// Reinforcement count (how many times this pattern was successful).
    pub reinforcement: u32,
    /// Timestamp of last access (epoch seconds).
    pub last_accessed: u64,
}

/// Associative memory store with sub-linear similarity search.
#[derive(Clone, Serialize, Deserialize)]
pub struct AssociativeMemory {
    /// All stored entries.
    entries: Vec<MemoryEntry>,
    /// The "success attractor" — evolving ideal opportunity vector.
    attractor: HdVector,
    /// Accumulator for incremental attractor updates (sum before threshold).
    attractor_acc: Vec<i32>,
    /// Number of reinforcement events applied to the attractor.
    attractor_updates: u64,
    /// HD vector dimension.
    dim: usize,
    /// Shared codebook for encoding.
    pub codebook: Codebook,
}

/// Result of a similarity query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub label: String,
    pub similarity: f64,
    pub tags: Vec<String>,
    pub reinforcement: u32,
}

impl AssociativeMemory {
    /// Create a new empty associative memory.
    pub fn new(dim: usize) -> Self {
        Self {
            entries: Vec::new(),
            attractor: HdVector::random(dim),
            attractor_acc: vec![0i32; dim],
            attractor_updates: 0,
            dim,
            codebook: Codebook::new(dim),
        }
    }

    /// Create with default dimension (256).
    pub fn default_dim() -> Self {
        Self::new(DEFAULT_DIM)
    }

    /// Store a new entry.
    pub fn store(&mut self, label: String, vector: HdVector, tags: Vec<String>) {
        assert_eq!(vector.dim(), self.dim, "vector dimension mismatch");
        self.entries.push(MemoryEntry {
            label,
            vector,
            tags,
            reinforcement: 0,
            last_accessed: current_epoch(),
        });
    }

    /// Find the top-k most similar entries to a query vector.
    pub fn query(&mut self, query: &HdVector, k: usize) -> Vec<SimilarityResult> {
        assert_eq!(query.dim(), self.dim, "query dimension mismatch");

        let mut scored: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (i, query.cosine_similarity(&entry.vector)))
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results: Vec<SimilarityResult> = scored
            .iter()
            .take(k)
            .map(|&(i, sim)| {
                let entry = &self.entries[i];
                SimilarityResult {
                    label: entry.label.clone(),
                    similarity: sim,
                    tags: entry.tags.clone(),
                    reinforcement: entry.reinforcement,
                }
            })
            .collect();

        // Update last_accessed for returned entries
        let now = current_epoch();
        for &(i, _) in scored.iter().take(k) {
            self.entries[i].last_accessed = now;
        }

        results
    }

    /// Find all entries above a similarity threshold.
    pub fn query_threshold(&self, query: &HdVector, threshold: f64) -> Vec<SimilarityResult> {
        assert_eq!(query.dim(), self.dim, "query dimension mismatch");

        self.entries
            .iter()
            .filter_map(|entry| {
                let sim = query.cosine_similarity(&entry.vector);
                if sim >= threshold {
                    Some(SimilarityResult {
                        label: entry.label.clone(),
                        similarity: sim,
                        tags: entry.tags.clone(),
                        reinforcement: entry.reinforcement,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Reinforce an entry (mark as successful). Strengthens the attractor.
    pub fn reinforce(&mut self, label: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.label == label) {
            entry.reinforcement += 1;

            // Update the attractor accumulator with this successful pattern
            entry.vector.accumulate_into(&mut self.attractor_acc);
            self.attractor_updates += 1;

            // Re-threshold the attractor every 10 updates for stability
            if self.attractor_updates % 10 == 0 {
                self.attractor = HdVector::from_accumulator(&self.attractor_acc);
            }
        }
    }

    /// Get the current success attractor vector.
    pub fn attractor(&self) -> &HdVector {
        &self.attractor
    }

    /// Score a candidate opportunity against the success attractor.
    /// Returns similarity in [-1.0, +1.0] where higher = more like past successes.
    pub fn attractor_score(&self, candidate: &HdVector) -> f64 {
        candidate.cosine_similarity(&self.attractor)
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether memory is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total memory footprint in bytes (packed vectors).
    pub fn memory_bytes(&self) -> usize {
        self.entries.len() * self.entries.first().map_or(0, |e| e.vector.packed_bytes())
            + self.attractor.packed_bytes()
    }

    /// Get attractor strength (normalized update count).
    pub fn attractor_strength(&self) -> f64 {
        if self.attractor_updates == 0 {
            return 0.0;
        }
        // Sigmoid-like saturation: approaches 1.0 as updates increase
        1.0 - (-(self.attractor_updates as f64) / 50.0).exp()
    }

    /// Remove entries older than `max_age_secs` that have low reinforcement.
    pub fn evict_stale(&mut self, max_age_secs: u64, min_reinforcement: u32) {
        let now = current_epoch();
        self.entries.retain(|entry| {
            let age = now.saturating_sub(entry.last_accessed);
            age < max_age_secs || entry.reinforcement >= min_reinforcement
        });
    }

    /// Get all entry labels.
    pub fn labels(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.label.as_str()).collect()
    }

    /// Get dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }
}

/// Get current epoch seconds (monotonic-safe).
fn current_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_memory() -> AssociativeMemory {
        let mut mem = AssociativeMemory::new(DEFAULT_DIM);
        let v1 = HdVector::from_seed(DEFAULT_DIM, 1);
        let v2 = HdVector::from_seed(DEFAULT_DIM, 2);
        let v3 = HdVector::from_seed(DEFAULT_DIM, 3);

        mem.store("alpha".into(), v1, vec!["ethereum".into(), "airdrop".into()]);
        mem.store("beta".into(), v2, vec!["arbitrum".into(), "quest".into()]);
        mem.store("gamma".into(), v3, vec!["polygon".into(), "faucet".into()]);
        mem
    }

    #[test]
    fn store_and_count() {
        let mem = make_test_memory();
        assert_eq!(mem.len(), 3);
        assert!(!mem.is_empty());
    }

    #[test]
    fn query_returns_most_similar() {
        let mut mem = make_test_memory();
        let query = HdVector::from_seed(DEFAULT_DIM, 1); // Identical to "alpha"
        let results = mem.query(&query, 3);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].label, "alpha");
        assert!((results[0].similarity - 1.0).abs() < 1e-10, "exact match should have sim=1.0");
    }

    #[test]
    fn query_respects_k() {
        let mut mem = make_test_memory();
        let query = HdVector::random(DEFAULT_DIM);
        let results = mem.query(&query, 1);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn query_threshold_filters() {
        let mem = make_test_memory();
        let query = HdVector::from_seed(DEFAULT_DIM, 1);
        let results = mem.query_threshold(&query, 0.9);

        // Only the exact match should pass a 0.9 threshold
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "alpha");
    }

    #[test]
    fn reinforce_updates_entry() {
        let mut mem = make_test_memory();
        mem.reinforce("alpha");
        mem.reinforce("alpha");

        let results = mem.query_threshold(&HdVector::from_seed(DEFAULT_DIM, 1), 0.9);
        assert_eq!(results[0].reinforcement, 2);
    }

    #[test]
    fn attractor_evolves_with_reinforcement() {
        let mut mem = make_test_memory();
        let initial_attractor = mem.attractor().clone();

        // Reinforce "alpha" many times to shift the attractor
        for _ in 0..20 {
            mem.reinforce("alpha");
        }

        let evolved_attractor = mem.attractor();
        let alpha_vec = HdVector::from_seed(DEFAULT_DIM, 1);

        let sim_before = initial_attractor.cosine_similarity(&alpha_vec);
        let sim_after = evolved_attractor.cosine_similarity(&alpha_vec);

        assert!(
            sim_after > sim_before,
            "attractor should move toward reinforced pattern: before={sim_before} after={sim_after}"
        );
    }

    #[test]
    fn attractor_score_reflects_success_pattern() {
        let mut mem = make_test_memory();

        for _ in 0..20 {
            mem.reinforce("alpha");
        }

        let alpha_vec = HdVector::from_seed(DEFAULT_DIM, 1);
        let gamma_vec = HdVector::from_seed(DEFAULT_DIM, 3);

        let score_alpha = mem.attractor_score(&alpha_vec);
        let score_gamma = mem.attractor_score(&gamma_vec);

        assert!(
            score_alpha > score_gamma,
            "reinforced pattern should score higher: alpha={score_alpha} gamma={score_gamma}"
        );
    }

    #[test]
    fn attractor_strength_starts_at_zero() {
        let mem = AssociativeMemory::new(DEFAULT_DIM);
        assert_eq!(mem.attractor_strength(), 0.0);
    }

    #[test]
    fn attractor_strength_increases() {
        let mut mem = make_test_memory();
        for _ in 0..10 {
            mem.reinforce("alpha");
        }
        assert!(mem.attractor_strength() > 0.0);
    }

    #[test]
    fn memory_bytes_is_compact() {
        let mem = make_test_memory();
        let bytes = mem.memory_bytes();
        // 3 entries × 32 bytes + 1 attractor × 32 bytes = ~128 bytes
        assert!(bytes < 200, "memory should be compact: {bytes} bytes");
    }

    #[test]
    fn evict_stale_removes_old_unreinforced() {
        let mut mem = make_test_memory();
        // Reinforce alpha so it survives eviction
        mem.reinforce("alpha");

        // Force-age all entries by setting last_accessed to 0
        for entry in mem.entries.iter_mut() {
            entry.last_accessed = 0;
        }

        mem.evict_stale(1, 1); // Max age 1 second, min reinforcement 1
        assert_eq!(mem.len(), 1);
        assert_eq!(mem.labels(), vec!["alpha"]);
    }

    #[test]
    fn labels_returns_all_names() {
        let mem = make_test_memory();
        let labels = mem.labels();
        assert_eq!(labels.len(), 3);
        assert!(labels.contains(&"alpha"));
        assert!(labels.contains(&"beta"));
        assert!(labels.contains(&"gamma"));
    }

    #[test]
    fn empty_memory() {
        let mem = AssociativeMemory::new(DEFAULT_DIM);
        assert!(mem.is_empty());
        assert_eq!(mem.len(), 0);
    }

    #[test]
    fn default_dim_constructor() {
        let mem = AssociativeMemory::default_dim();
        assert_eq!(mem.dim(), DEFAULT_DIM);
    }

    #[test]
    fn memory_serializable() {
        let mem = make_test_memory();
        let json = serde_json::to_string(&mem).unwrap();
        let deser: AssociativeMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.len(), 3);
    }
}
