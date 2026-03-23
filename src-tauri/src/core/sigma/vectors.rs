//! Hyperdimensional (HD) Vector Symbolic Architecture.
//!
//! Binary HD vectors with bind (⊗), bundle (+), permute (ρ), and cosine similarity.
//! Uses bipolar representation internally ({-1, +1} stored as `i8`) for clean algebra,
//! but exposes a simple API. Default dimension: 256 (compact yet expressive).
//!
//! This is the foundational data structure for all ΣCORE operations:
//! - Opportunity encoding
//! - Success attractor evolution
//! - Swarm agent personas
//! - Compressed memory indexing

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Default hypervector dimension. 256 gives 2^256 near-orthogonal directions.
pub const DEFAULT_DIM: usize = 256;

/// A hyperdimensional vector — the atomic unit of ΣCORE.
/// Stored as bipolar components ({-1, +1}).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdVector {
    /// Bipolar components: each element is -1 or +1.
    pub components: Vec<i8>,
}

impl HdVector {
    /// Create a random HD vector with the given dimension.
    pub fn random(dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let components: Vec<i8> = (0..dim).map(|_| if rng.gen_bool(0.5) { 1 } else { -1 }).collect();
        Self { components }
    }

    /// Create a random HD vector with deterministic seed (for reproducible codebooks).
    pub fn from_seed(dim: usize, seed: u64) -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let components: Vec<i8> = (0..dim).map(|_| if rng.gen_bool(0.5) { 1 } else { -1 }).collect();
        Self { components }
    }

    /// Create a zero vector (neutral for bundling).
    pub fn zero(dim: usize) -> Self {
        Self {
            components: vec![0; dim],
        }
    }

    /// Create an all-ones vector.
    pub fn ones(dim: usize) -> Self {
        Self {
            components: vec![1; dim],
        }
    }

    /// Dimension of this vector.
    pub fn dim(&self) -> usize {
        self.components.len()
    }

    /// Bind operation (⊗): element-wise multiply. Preserves similarity with neither operand.
    /// Used to create "role-filler" pairs (e.g., chain ⊗ ethereum).
    pub fn bind(&self, other: &HdVector) -> HdVector {
        assert_eq!(self.dim(), other.dim(), "dimension mismatch for bind");
        let components = self
            .components
            .iter()
            .zip(other.components.iter())
            .map(|(&a, &b)| a * b)
            .collect();
        HdVector { components }
    }

    /// Bundle operation (+): element-wise sum then threshold to bipolar.
    /// Used to create superpositions (e.g., combine multiple features).
    /// Breaks ties randomly (for even number of vectors).
    pub fn bundle(vectors: &[&HdVector]) -> HdVector {
        assert!(!vectors.is_empty(), "cannot bundle empty list");
        let dim = vectors[0].dim();
        let mut sums: Vec<i32> = vec![0; dim];

        for v in vectors {
            assert_eq!(v.dim(), dim, "dimension mismatch for bundle");
            for (i, &c) in v.components.iter().enumerate() {
                sums[i] += c as i32;
            }
        }

        let mut rng = rand::thread_rng();
        let components = sums
            .iter()
            .map(|&s| {
                if s > 0 {
                    1
                } else if s < 0 {
                    -1
                } else {
                    // Tie-break randomly
                    if rng.gen_bool(0.5) { 1 } else { -1 }
                }
            })
            .collect();

        HdVector { components }
    }

    /// Bundle with accumulator (for incremental bundling).
    /// Adds this vector's components to an i32 accumulator.
    pub fn accumulate_into(&self, acc: &mut [i32]) {
        assert_eq!(self.dim(), acc.len(), "accumulator dimension mismatch");
        for (i, &c) in self.components.iter().enumerate() {
            acc[i] += c as i32;
        }
    }

    /// Threshold an i32 accumulator into a bipolar HD vector.
    pub fn from_accumulator(acc: &[i32]) -> Self {
        let mut rng = rand::thread_rng();
        let components = acc
            .iter()
            .map(|&s| {
                if s > 0 {
                    1
                } else if s < 0 {
                    -1
                } else {
                    if rng.gen_bool(0.5) { 1 } else { -1 }
                }
            })
            .collect();
        Self { components }
    }

    /// Permute (ρ): circular left-shift by `n` positions.
    /// Used to encode sequence/order information.
    pub fn permute(&self, n: usize) -> HdVector {
        let dim = self.dim();
        if dim == 0 {
            return self.clone();
        }
        let shift = n % dim;
        let mut components = Vec::with_capacity(dim);
        components.extend_from_slice(&self.components[shift..]);
        components.extend_from_slice(&self.components[..shift]);
        HdVector { components }
    }

    /// Cosine similarity in [-1.0, +1.0].
    /// +1.0 = identical, 0.0 = orthogonal, -1.0 = opposite.
    pub fn cosine_similarity(&self, other: &HdVector) -> f64 {
        assert_eq!(self.dim(), other.dim(), "dimension mismatch for similarity");
        let dot: i64 = self
            .components
            .iter()
            .zip(other.components.iter())
            .map(|(&a, &b)| (a as i64) * (b as i64))
            .sum();

        let norm_a: f64 = self.components.iter().map(|&c| (c as f64).powi(2)).sum::<f64>().sqrt();
        let norm_b: f64 = other
            .components
            .iter()
            .map(|&c| (c as f64).powi(2))
            .sum::<f64>()
            .sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot as f64 / (norm_a * norm_b)
    }

    /// Hamming distance (number of positions that differ).
    pub fn hamming_distance(&self, other: &HdVector) -> usize {
        assert_eq!(self.dim(), other.dim(), "dimension mismatch");
        self.components
            .iter()
            .zip(other.components.iter())
            .filter(|(&a, &b)| a != b)
            .count()
    }

    /// Negate (flip all components).
    pub fn negate(&self) -> HdVector {
        let components = self.components.iter().map(|&c| -c).collect();
        HdVector { components }
    }

    /// Pack into bytes (1 bit per component: +1 → 1, -1 → 0).
    /// Returns a compact byte representation for storage.
    pub fn pack(&self) -> Vec<u8> {
        let num_bytes = (self.dim() + 7) / 8;
        let mut bytes = vec![0u8; num_bytes];
        for (i, &c) in self.components.iter().enumerate() {
            if c > 0 {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }
        bytes
    }

    /// Unpack from bytes back to bipolar HD vector.
    pub fn unpack(bytes: &[u8], dim: usize) -> Self {
        let mut components = Vec::with_capacity(dim);
        for i in 0..dim {
            let bit = (bytes[i / 8] >> (i % 8)) & 1;
            components.push(if bit == 1 { 1 } else { -1 });
        }
        Self { components }
    }

    /// Memory footprint in bytes (packed representation).
    pub fn packed_bytes(&self) -> usize {
        (self.dim() + 7) / 8
    }

    /// Check if this is a valid bipolar vector (all components are -1 or +1).
    pub fn is_bipolar(&self) -> bool {
        self.components.iter().all(|&c| c == 1 || c == -1)
    }
}

impl fmt::Debug for HdVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HdVector(dim={}, packed={}B)",
            self.dim(),
            self.packed_bytes()
        )
    }
}

/// A deterministic codebook that maps symbol names to HD vectors.
/// Used to encode categorical features (chain names, opportunity types, risk levels).
#[derive(Clone, Serialize, Deserialize)]
pub struct Codebook {
    dim: usize,
    /// Maps symbol name → deterministic seed → HD vector.
    entries: Vec<(String, HdVector)>,
}

impl Codebook {
    /// Create a new codebook with given dimension.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            entries: Vec::new(),
        }
    }

    /// Get or create a vector for a symbol. Uses deterministic hashing.
    pub fn get_or_create(&mut self, symbol: &str) -> &HdVector {
        if let Some(pos) = self.entries.iter().position(|(name, _)| name == symbol) {
            return &self.entries[pos].1;
        }

        // Deterministic seed from symbol name
        let seed = symbol_to_seed(symbol);
        let vec = HdVector::from_seed(self.dim, seed);
        self.entries.push((symbol.to_string(), vec));
        &self.entries.last().unwrap().1
    }

    /// Look up a symbol without creating it.
    pub fn get(&self, symbol: &str) -> Option<&HdVector> {
        self.entries
            .iter()
            .find(|(name, _)| name == symbol)
            .map(|(_, v)| v)
    }

    /// Number of symbols in the codebook.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the codebook is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// List all symbol names.
    pub fn symbols(&self) -> Vec<&str> {
        self.entries.iter().map(|(name, _)| name.as_str()).collect()
    }
}

/// Encode an opportunity into an HD vector using bind/bundle.
/// Structure: (chain ⊗ chain_value) + (type ⊗ type_value) + (risk ⊗ risk_value)
pub fn encode_opportunity(
    codebook: &mut Codebook,
    chain: &str,
    opportunity_type: &str,
    risk_level: &str,
) -> HdVector {
    let chain_role = codebook.get_or_create("role:chain").clone();
    let type_role = codebook.get_or_create("role:type").clone();
    let risk_role = codebook.get_or_create("role:risk").clone();

    let chain_val = codebook.get_or_create(chain).clone();
    let type_val = codebook.get_or_create(opportunity_type).clone();
    let risk_val = codebook.get_or_create(risk_level).clone();

    let bound_chain = chain_role.bind(&chain_val);
    let bound_type = type_role.bind(&type_val);
    let bound_risk = risk_role.bind(&risk_val);

    HdVector::bundle(&[&bound_chain, &bound_type, &bound_risk])
}

/// Deterministic seed from a symbol string (simple hash).
fn symbol_to_seed(symbol: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in symbol.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_vector_has_correct_dimension() {
        let v = HdVector::random(DEFAULT_DIM);
        assert_eq!(v.dim(), DEFAULT_DIM);
    }

    #[test]
    fn random_vector_is_bipolar() {
        let v = HdVector::random(DEFAULT_DIM);
        assert!(v.is_bipolar());
    }

    #[test]
    fn seeded_vectors_are_deterministic() {
        let a = HdVector::from_seed(DEFAULT_DIM, 42);
        let b = HdVector::from_seed(DEFAULT_DIM, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_vectors() {
        let a = HdVector::from_seed(DEFAULT_DIM, 42);
        let b = HdVector::from_seed(DEFAULT_DIM, 99);
        assert_ne!(a, b);
    }

    #[test]
    fn self_similarity_is_one() {
        let v = HdVector::random(DEFAULT_DIM);
        let sim = v.cosine_similarity(&v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn opposite_similarity_is_negative_one() {
        let v = HdVector::random(DEFAULT_DIM);
        let neg = v.negate();
        let sim = v.cosine_similarity(&neg);
        assert!((sim + 1.0).abs() < 1e-10);
    }

    #[test]
    fn random_vectors_are_near_orthogonal() {
        // With dimension 256, random vectors should have similarity near 0
        let a = HdVector::random(DEFAULT_DIM);
        let b = HdVector::random(DEFAULT_DIM);
        let sim = a.cosine_similarity(&b);
        assert!(sim.abs() < 0.3, "expected near-orthogonal, got {sim}");
    }

    #[test]
    fn bind_is_self_inverse() {
        let a = HdVector::random(DEFAULT_DIM);
        let b = HdVector::random(DEFAULT_DIM);
        let bound = a.bind(&b);
        let recovered = bound.bind(&b);
        assert_eq!(recovered, a, "bind should be self-inverse");
    }

    #[test]
    fn bind_produces_dissimilar_to_operands() {
        let a = HdVector::random(DEFAULT_DIM);
        let b = HdVector::random(DEFAULT_DIM);
        let bound = a.bind(&b);
        let sim_a = bound.cosine_similarity(&a);
        let sim_b = bound.cosine_similarity(&b);
        assert!(sim_a.abs() < 0.3, "bind result should be dissimilar to a");
        assert!(sim_b.abs() < 0.3, "bind result should be dissimilar to b");
    }

    #[test]
    fn bundle_recovers_components() {
        let a = HdVector::random(DEFAULT_DIM);
        let b = HdVector::random(DEFAULT_DIM);
        let c = HdVector::random(DEFAULT_DIM);
        let bundled = HdVector::bundle(&[&a, &b, &c]);

        // Each component should be somewhat similar to the bundle
        let sim_a = bundled.cosine_similarity(&a);
        let sim_b = bundled.cosine_similarity(&b);
        let sim_c = bundled.cosine_similarity(&c);

        assert!(sim_a > 0.2, "bundle should be similar to component a: {sim_a}");
        assert!(sim_b > 0.2, "bundle should be similar to component b: {sim_b}");
        assert!(sim_c > 0.2, "bundle should be similar to component c: {sim_c}");
    }

    #[test]
    fn permute_changes_vector() {
        let v = HdVector::random(DEFAULT_DIM);
        let p = v.permute(1);
        assert_ne!(v, p);
    }

    #[test]
    fn permute_by_zero_is_identity() {
        let v = HdVector::random(DEFAULT_DIM);
        let p = v.permute(0);
        assert_eq!(v, p);
    }

    #[test]
    fn permute_by_dim_is_identity() {
        let v = HdVector::random(DEFAULT_DIM);
        let p = v.permute(DEFAULT_DIM);
        assert_eq!(v, p);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let v = HdVector::random(DEFAULT_DIM);
        let packed = v.pack();
        let unpacked = HdVector::unpack(&packed, DEFAULT_DIM);
        assert_eq!(v, unpacked);
    }

    #[test]
    fn packed_size_is_compact() {
        let v = HdVector::random(DEFAULT_DIM);
        assert_eq!(v.packed_bytes(), 32); // 256 bits = 32 bytes
    }

    #[test]
    fn hamming_distance_self_is_zero() {
        let v = HdVector::random(DEFAULT_DIM);
        assert_eq!(v.hamming_distance(&v), 0);
    }

    #[test]
    fn hamming_distance_opposite_is_dim() {
        let v = HdVector::random(DEFAULT_DIM);
        let neg = v.negate();
        assert_eq!(v.hamming_distance(&neg), DEFAULT_DIM);
    }

    #[test]
    fn zero_vector_cosine_is_zero() {
        let v = HdVector::random(DEFAULT_DIM);
        let z = HdVector::zero(DEFAULT_DIM);
        assert_eq!(v.cosine_similarity(&z), 0.0);
    }

    #[test]
    fn accumulate_and_threshold() {
        let a = HdVector::random(DEFAULT_DIM);
        let b = HdVector::random(DEFAULT_DIM);
        let mut acc = vec![0i32; DEFAULT_DIM];
        a.accumulate_into(&mut acc);
        b.accumulate_into(&mut acc);
        let result = HdVector::from_accumulator(&acc);
        assert_eq!(result.dim(), DEFAULT_DIM);
        assert!(result.is_bipolar());
    }

    // ── Codebook tests ──────────────────────────────────

    #[test]
    fn codebook_creates_symbols() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        cb.get_or_create("ethereum");
        cb.get_or_create("arbitrum");
        assert_eq!(cb.len(), 2);
    }

    #[test]
    fn codebook_is_deterministic() {
        let mut cb1 = Codebook::new(DEFAULT_DIM);
        let mut cb2 = Codebook::new(DEFAULT_DIM);
        let v1 = cb1.get_or_create("ethereum").clone();
        let v2 = cb2.get_or_create("ethereum").clone();
        assert_eq!(v1, v2);
    }

    #[test]
    fn codebook_different_symbols_are_dissimilar() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        let eth = cb.get_or_create("ethereum").clone();
        let arb = cb.get_or_create("arbitrum").clone();
        let sim = eth.cosine_similarity(&arb);
        assert!(sim.abs() < 0.3, "different symbols should be near-orthogonal");
    }

    #[test]
    fn codebook_get_returns_none_for_unknown() {
        let cb = Codebook::new(DEFAULT_DIM);
        assert!(cb.get("solana").is_none());
    }

    #[test]
    fn codebook_symbols_list() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        cb.get_or_create("a");
        cb.get_or_create("b");
        let syms = cb.symbols();
        assert_eq!(syms.len(), 2);
        assert!(syms.contains(&"a"));
        assert!(syms.contains(&"b"));
    }

    // ── Opportunity encoding tests ──────────────────────

    #[test]
    fn encode_opportunity_produces_bipolar() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        let vec = encode_opportunity(&mut cb, "ethereum", "airdrop", "low");
        assert!(vec.is_bipolar());
        assert_eq!(vec.dim(), DEFAULT_DIM);
    }

    #[test]
    fn same_opportunity_encodes_similarly() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        let v1 = encode_opportunity(&mut cb, "ethereum", "airdrop", "low");
        let v2 = encode_opportunity(&mut cb, "ethereum", "airdrop", "low");
        let sim = v1.cosine_similarity(&v2);
        // Should be very similar (may not be identical due to bundle tie-breaking)
        assert!(sim > 0.5, "same encoding should be similar: {sim}");
    }

    #[test]
    fn different_opportunities_encode_differently() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        let v1 = encode_opportunity(&mut cb, "ethereum", "airdrop", "low");
        let v2 = encode_opportunity(&mut cb, "polygon", "quest", "high");
        let sim = v1.cosine_similarity(&v2);
        assert!(sim < 0.5, "different encodings should be dissimilar: {sim}");
    }

    #[test]
    fn opportunity_partially_similar() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        let v1 = encode_opportunity(&mut cb, "ethereum", "airdrop", "low");
        let v2 = encode_opportunity(&mut cb, "ethereum", "airdrop", "high");
        let v3 = encode_opportunity(&mut cb, "polygon", "quest", "high");
        let sim_close = v1.cosine_similarity(&v2);
        let sim_far = v1.cosine_similarity(&v3);
        // v2 shares 2/3 features with v1, v3 shares 0/3
        assert!(
            sim_close > sim_far,
            "partially matching should be more similar: close={sim_close} far={sim_far}"
        );
    }

    #[test]
    fn symbol_to_seed_deterministic() {
        let a = symbol_to_seed("ethereum");
        let b = symbol_to_seed("ethereum");
        assert_eq!(a, b);
    }

    #[test]
    fn symbol_to_seed_different_for_different_strings() {
        let a = symbol_to_seed("ethereum");
        let b = symbol_to_seed("arbitrum");
        assert_ne!(a, b);
    }

    #[test]
    fn hdvector_debug_format() {
        let v = HdVector::random(DEFAULT_DIM);
        let debug = format!("{:?}", v);
        assert!(debug.contains("HdVector"));
        assert!(debug.contains("256"));
    }

    #[test]
    fn codebook_serializable() {
        let mut cb = Codebook::new(DEFAULT_DIM);
        cb.get_or_create("test");
        let json = serde_json::to_string(&cb).unwrap();
        let deser: Codebook = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.len(), 1);
    }
}
