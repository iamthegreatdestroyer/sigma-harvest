//! ΣLANG-style log and knowledge compression pipeline.
//!
//! Provides dictionary-based compression for log entries and opportunity records,
//! with HD vector summaries for sub-linear search over compressed history.
//!
//! The compression strategy:
//! 1. Dictionary encoding: map repeated tokens to short codes
//! 2. Run-length encoding for sequential duplicates
//! 3. HD vector summary generation for each compressed batch
//! 4. Packed binary storage

use super::vectors::{HdVector, DEFAULT_DIM};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single log entry before compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

impl LogEntry {
    /// Create a new log entry with the current timestamp.
    pub fn new(source: &str, level: &str, message: &str) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            level: LogLevel::from_str(level),
            source: source.to_string(),
            message: message.to_string(),
        }
    }
}

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Parse a log level from a string (case-insensitive). Defaults to Info.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

/// A compressed batch of log entries with an HD vector summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBatch {
    /// Dictionary: token → code.
    pub dictionary: Vec<String>,
    /// Compressed data: sequence of dictionary indices.
    pub encoded: Vec<u16>,
    /// Run-length encoded form of `encoded` (index, count) pairs.
    pub rle: Vec<(u16, u16)>,
    /// Number of original entries in this batch.
    pub entry_count: usize,
    /// HD vector summary of this batch's content.
    pub summary_vector: HdVector,
    /// Original total bytes before compression.
    pub original_bytes: usize,
    /// Compressed size in bytes.
    pub compressed_bytes: usize,
    /// Timestamp range [first, last].
    pub time_range: (u64, u64),
}

impl CompressedBatch {
    /// Compression ratio (0.0 = maximally compressed, 1.0 = no compression).
    pub fn compression_ratio(&self) -> f64 {
        if self.original_bytes == 0 {
            return 1.0;
        }
        self.compressed_bytes as f64 / self.original_bytes as f64
    }
}

/// The compression pipeline.
#[derive(Clone, Serialize, Deserialize)]
pub struct CompressionPipeline {
    /// Global dictionary for cross-batch token reuse.
    global_dict: HashMap<String, u16>,
    /// Next available dictionary code.
    next_code: u16,
    /// All compressed batches.
    batches: Vec<CompressedBatch>,
    /// Batch size (number of entries per batch).
    batch_size: usize,
    /// Pending entries not yet compressed.
    pending: Vec<LogEntry>,
    /// HD vector dimension.
    dim: usize,
}

impl CompressionPipeline {
    /// Create a new pipeline with default settings.
    pub fn new(batch_size: usize) -> Self {
        Self {
            global_dict: HashMap::new(),
            next_code: 0,
            batches: Vec::new(),
            batch_size,
            pending: Vec::new(),
            dim: DEFAULT_DIM,
        }
    }

    /// Add a log entry. Automatically compresses when batch_size is reached.
    pub fn push(&mut self, entry: LogEntry) {
        self.pending.push(entry);
        if self.pending.len() >= self.batch_size {
            self.flush();
        }
    }

    /// Force-compress all pending entries into a batch.
    pub fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }

        let entries = std::mem::take(&mut self.pending);
        let batch = self.compress_batch(&entries);
        self.batches.push(batch);
    }

    /// Compress a batch of entries.
    fn compress_batch(&mut self, entries: &[LogEntry]) -> CompressedBatch {
        // Calculate original size
        let original_bytes: usize = entries
            .iter()
            .map(|e| 8 + e.level.as_str().len() + e.source.len() + e.message.len())
            .sum();

        // Tokenize all entries
        let mut tokens: Vec<String> = Vec::new();
        for entry in entries {
            tokens.push(entry.level.as_str().to_string());
            tokens.push(entry.source.clone());
            for word in entry.message.split_whitespace() {
                tokens.push(word.to_string());
            }
            tokens.push("||".to_string()); // Entry separator
        }

        // Dictionary encode
        let mut encoded = Vec::with_capacity(tokens.len());
        for token in &tokens {
            let code = if let Some(&code) = self.global_dict.get(token) {
                code
            } else {
                let code = self.next_code;
                self.global_dict.insert(token.clone(), code);
                self.next_code = self.next_code.wrapping_add(1);
                code
            };
            encoded.push(code);
        }

        // Run-length encode
        let rle = run_length_encode(&encoded);

        // Generate HD vector summary (bundle of all token vectors)
        let summary_vector = self.generate_summary(&tokens);

        // Calculate compressed size
        let rle_bytes = rle.len() * 4; // Each (u16, u16) = 4 bytes
        let compressed_bytes = rle_bytes + 32; // RLE data + summary vector (32 bytes packed)

        let time_range = if entries.is_empty() {
            (0, 0)
        } else {
            (
                entries.first().unwrap().timestamp,
                entries.last().unwrap().timestamp,
            )
        };

        CompressedBatch {
            dictionary: self.global_dict.keys().cloned().collect(),
            encoded,
            rle,
            entry_count: entries.len(),
            summary_vector,
            original_bytes,
            compressed_bytes,
            time_range,
        }
    }

    /// Generate an HD vector summary from tokens using deterministic hashing.
    fn generate_summary(&self, tokens: &[String]) -> HdVector {
        if tokens.is_empty() {
            return HdVector::zero(self.dim);
        }

        let mut acc = vec![0i32; self.dim];
        for token in tokens {
            if token == "||" {
                continue;
            }
            let seed = token_seed(token);
            let tv = HdVector::from_seed(self.dim, seed);
            tv.accumulate_into(&mut acc);
        }

        HdVector::from_accumulator(&acc)
    }

    /// Search compressed history for batches most similar to a query string.
    /// Returns batch indices sorted by similarity (descending).
    pub fn search(&self, query: &str, top_k: usize) -> Vec<(usize, f64)> {
        // Encode query as HD vector
        let query_tokens: Vec<String> = query.split_whitespace().map(String::from).collect();
        let query_vec = self.generate_summary(&query_tokens);

        let mut scored: Vec<(usize, f64)> = self
            .batches
            .iter()
            .enumerate()
            .map(|(i, batch)| (i, query_vec.cosine_similarity(&batch.summary_vector)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Total number of compressed batches.
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Total entries across all batches + pending.
    pub fn total_entries(&self) -> usize {
        let batch_entries: usize = self.batches.iter().map(|b| b.entry_count).sum();
        batch_entries + self.pending.len()
    }

    /// Overall compression ratio across all batches.
    pub fn overall_compression_ratio(&self) -> f64 {
        let total_original: usize = self.batches.iter().map(|b| b.original_bytes).sum();
        let total_compressed: usize = self.batches.iter().map(|b| b.compressed_bytes).sum();
        if total_original == 0 {
            return 1.0;
        }
        total_compressed as f64 / total_original as f64
    }

    /// Dictionary size (number of unique tokens).
    pub fn dictionary_size(&self) -> usize {
        self.global_dict.len()
    }

    /// Total memory footprint in bytes (approximate).
    pub fn memory_bytes(&self) -> usize {
        let dict_bytes: usize = self.global_dict.keys().map(|k| k.len() + 2).sum();
        let batch_bytes: usize = self
            .batches
            .iter()
            .map(|b| b.compressed_bytes + b.summary_vector.packed_bytes())
            .sum();
        dict_bytes + batch_bytes
    }
}

/// Simple run-length encoding.
fn run_length_encode(data: &[u16]) -> Vec<(u16, u16)> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current = data[0];
    let mut count: u16 = 1;

    for &item in &data[1..] {
        if item == current && count < u16::MAX {
            count += 1;
        } else {
            result.push((current, count));
            current = item;
            count = 1;
        }
    }
    result.push((current, count));
    result
}

/// Decode run-length encoded data.
#[allow(dead_code)]
fn run_length_decode(rle: &[(u16, u16)]) -> Vec<u16> {
    let mut result = Vec::new();
    for &(value, count) in rle {
        for _ in 0..count {
            result.push(value);
        }
    }
    result
}

/// Deterministic seed from token string.
fn token_seed(token: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037; // FNV-1a offset basis
    for byte in token.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211); // FNV-1a prime
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(msg: &str) -> LogEntry {
        LogEntry {
            timestamp: 1711100000,
            level: LogLevel::Info,
            source: "test".into(),
            message: msg.into(),
        }
    }

    #[test]
    fn rle_basic() {
        let data = vec![1, 1, 1, 2, 2, 3];
        let encoded = run_length_encode(&data);
        assert_eq!(encoded, vec![(1, 3), (2, 2), (3, 1)]);
    }

    #[test]
    fn rle_roundtrip() {
        let data = vec![5, 5, 3, 3, 3, 1, 7, 7];
        let encoded = run_length_encode(&data);
        let decoded = run_length_decode(&encoded);
        assert_eq!(data, decoded);
    }

    #[test]
    fn rle_empty() {
        let encoded = run_length_encode(&[]);
        assert!(encoded.is_empty());
    }

    #[test]
    fn rle_single() {
        let data = vec![42];
        let encoded = run_length_encode(&data);
        assert_eq!(encoded, vec![(42, 1)]);
    }

    #[test]
    fn compression_single_entry() {
        let mut pipeline = CompressionPipeline::new(1);
        pipeline.push(make_entry("hello world"));
        assert_eq!(pipeline.batch_count(), 1);
        assert_eq!(pipeline.total_entries(), 1);
    }

    #[test]
    fn compression_batch_size_respected() {
        let mut pipeline = CompressionPipeline::new(5);
        for i in 0..4 {
            pipeline.push(make_entry(&format!("message {i}")));
        }
        assert_eq!(pipeline.batch_count(), 0); // Not flushed yet
        pipeline.push(make_entry("fifth"));
        assert_eq!(pipeline.batch_count(), 1); // Now flushed
    }

    #[test]
    fn flush_compresses_pending() {
        let mut pipeline = CompressionPipeline::new(100); // Large batch size
        for i in 0..3 {
            pipeline.push(make_entry(&format!("message {i}")));
        }
        pipeline.flush();
        assert_eq!(pipeline.batch_count(), 1);
        assert_eq!(pipeline.total_entries(), 3);
    }

    #[test]
    fn compression_reduces_size() {
        let mut pipeline = CompressionPipeline::new(10);
        // Push entries with repeated tokens (good for compression)
        for _ in 0..10 {
            pipeline.push(make_entry("claimed airdrop on ethereum successfully"));
        }

        let batch = &pipeline.batches[0];
        assert!(
            batch.compression_ratio() < 1.0,
            "compression should reduce size: ratio={}",
            batch.compression_ratio()
        );
    }

    #[test]
    fn dictionary_grows_with_unique_tokens() {
        let mut pipeline = CompressionPipeline::new(5);
        pipeline.push(make_entry("alpha beta gamma"));
        pipeline.push(make_entry("delta epsilon zeta"));
        pipeline.push(make_entry("eta theta iota"));
        pipeline.push(make_entry("kappa lambda mu"));
        pipeline.push(make_entry("nu xi omicron"));

        assert!(pipeline.dictionary_size() > 10);
    }

    #[test]
    fn search_finds_relevant_batch() {
        let mut pipeline = CompressionPipeline::new(3);

        // Batch 1: ethereum airdrops
        pipeline.push(make_entry("claimed ethereum airdrop success"));
        pipeline.push(make_entry("ethereum gas was low today"));
        pipeline.push(make_entry("another ethereum claim completed"));

        // Batch 2: polygon quests
        pipeline.push(make_entry("completed polygon quest reward"));
        pipeline.push(make_entry("polygon network quest finished"));
        pipeline.push(make_entry("polygon reward claimed successfully"));

        let results = pipeline.search("ethereum airdrop", 2);
        assert!(!results.is_empty());

        // First result should be the ethereum batch (index 0)
        assert_eq!(results[0].0, 0, "ethereum batch should rank first");
        assert!(
            results[0].1 > results[1].1,
            "ethereum batch should have higher similarity"
        );
    }

    #[test]
    fn search_empty_pipeline() {
        let pipeline = CompressionPipeline::new(10);
        let results = pipeline.search("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn compressed_batch_has_summary_vector() {
        let mut pipeline = CompressionPipeline::new(2);
        pipeline.push(make_entry("test message"));
        pipeline.push(make_entry("another test"));

        let batch = &pipeline.batches[0];
        assert_eq!(batch.summary_vector.dim(), DEFAULT_DIM);
        assert!(batch.summary_vector.is_bipolar());
    }

    #[test]
    fn time_range_correct() {
        let mut pipeline = CompressionPipeline::new(2);
        pipeline.push(LogEntry {
            timestamp: 100,
            level: LogLevel::Info,
            source: "test".into(),
            message: "first".into(),
        });
        pipeline.push(LogEntry {
            timestamp: 200,
            level: LogLevel::Info,
            source: "test".into(),
            message: "last".into(),
        });

        let batch = &pipeline.batches[0];
        assert_eq!(batch.time_range, (100, 200));
    }

    #[test]
    fn overall_compression_ratio() {
        let mut pipeline = CompressionPipeline::new(5);
        for _ in 0..10 {
            pipeline.push(make_entry("repeated tokens help compression ratio"));
        }
        let ratio = pipeline.overall_compression_ratio();
        assert!(ratio < 1.0 && ratio > 0.0);
    }

    #[test]
    fn log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn token_seed_deterministic() {
        assert_eq!(token_seed("hello"), token_seed("hello"));
        assert_ne!(token_seed("hello"), token_seed("world"));
    }

    #[test]
    fn pipeline_serializable() {
        let mut pipeline = CompressionPipeline::new(2);
        pipeline.push(make_entry("test"));
        pipeline.push(make_entry("data"));

        let json = serde_json::to_string(&pipeline).unwrap();
        let deser: CompressionPipeline = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.batch_count(), 1);
    }

    #[test]
    fn memory_bytes_increases_with_data() {
        let mut pipeline = CompressionPipeline::new(5);
        let bytes_empty = pipeline.memory_bytes();

        for _ in 0..5 {
            pipeline.push(make_entry("data for memory tracking"));
        }

        let bytes_full = pipeline.memory_bytes();
        assert!(bytes_full > bytes_empty);
    }
}
