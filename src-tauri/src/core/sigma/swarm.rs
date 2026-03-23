//! Evolutionary sub-agent swarm with HD vector personas.
//!
//! Each agent has:
//! - An HD vector "persona" encoding its strategy preferences
//! - A performance history (claim success/failure counts)
//! - An assigned chain or domain
//!
//! The swarm evolves through:
//! - Reinforcement: successful claims strengthen an agent's persona
//! - Mutation: random perturbation for exploration of new strategies
//! - Consensus: agents "vote" via vector bundle before executing claims

use super::vectors::{HdVector, DEFAULT_DIM};
use serde::{Deserialize, Serialize};

/// A single sub-agent in the evolutionary swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgent {
    /// Unique agent identifier.
    pub id: String,
    /// Assigned domain (e.g., chain name or "social", "onchain").
    pub domain: String,
    /// HD vector persona encoding strategy preferences.
    pub persona: HdVector,
    /// Total successful claims.
    pub successes: u64,
    /// Total failed claims.
    pub failures: u64,
    /// Generation number (increments on mutation).
    pub generation: u32,
    /// Whether this agent is currently active.
    pub active: bool,
}

impl SwarmAgent {
    /// Create a new agent with a random persona.
    pub fn new(id: String, domain: String, dim: usize) -> Self {
        Self {
            id,
            domain,
            persona: HdVector::random(dim),
            successes: 0,
            failures: 0,
            generation: 0,
            active: true,
        }
    }

    /// Create an agent with a specific persona (e.g., cloned from a successful parent).
    pub fn with_persona(id: String, domain: String, persona: HdVector) -> Self {
        Self {
            id,
            domain,
            persona,
            successes: 0,
            failures: 0,
            generation: 0,
            active: true,
        }
    }

    /// Success rate (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        let total = self.successes + self.failures;
        if total == 0 {
            return 0.0;
        }
        self.successes as f64 / total as f64
    }

    /// Total operations (claims attempted).
    pub fn total_ops(&self) -> u64 {
        self.successes + self.failures
    }

    /// Record a successful claim. Optionally reinforce persona toward the opportunity vector.
    pub fn record_success(&mut self, opportunity_vector: Option<&HdVector>) {
        self.successes += 1;
        if let Some(opp) = opportunity_vector {
            // Shift persona toward the successful opportunity vector.
            // Bundle with equal weight so the persona actually evolves.
            self.persona = HdVector::bundle(&[&self.persona, opp]);
        }
    }

    /// Record a failed claim.
    pub fn record_failure(&mut self) {
        self.failures += 1;
    }

    /// Mutate the persona by flipping a fraction of bits.
    /// `mutation_rate` is the probability of flipping each component (0.0 to 1.0).
    pub fn mutate(&mut self, mutation_rate: f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for c in self.persona.components.iter_mut() {
            if rng.gen_bool(mutation_rate.clamp(0.0, 1.0)) {
                *c = -*c; // Flip bipolar component
            }
        }
        self.generation += 1;
    }

    /// How similar this agent's persona is to a candidate opportunity.
    pub fn affinity(&self, opportunity: &HdVector) -> f64 {
        self.persona.cosine_similarity(opportunity)
    }
}

/// The evolutionary swarm — a collection of agents that collaborate via vector consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swarm {
    /// All agents in the swarm.
    pub agents: Vec<SwarmAgent>,
    /// HD vector dimension.
    dim: usize,
    /// Mutation rate for evolutionary exploration (0.0 to 1.0).
    pub mutation_rate: f64,
    /// Minimum consensus threshold for proceeding with a claim.
    pub consensus_threshold: f64,
}

impl Swarm {
    /// Create a new swarm with one agent per domain.
    pub fn new(domains: &[&str], dim: usize) -> Self {
        let agents = domains
            .iter()
            .enumerate()
            .map(|(i, domain)| SwarmAgent::new(format!("agent-{i}"), domain.to_string(), dim))
            .collect();

        Self {
            agents,
            dim,
            mutation_rate: 0.02, // 2% default mutation rate
            consensus_threshold: 0.3,
        }
    }

    /// Create the default ΣHARVEST swarm (one agent per chain + social + onchain).
    pub fn default_harvest() -> Self {
        Self::new(
            &[
                "ethereum", "arbitrum", "optimism", "base", "polygon", "zksync", "social",
                "onchain",
            ],
            DEFAULT_DIM,
        )
    }

    /// Vector consensus: agents vote on whether to proceed with an opportunity.
    /// Returns the consensus score in [-1.0, +1.0] where higher = stronger agreement.
    ///
    /// The vote is computed by bundling all active agents' affinity-weighted personas,
    /// then measuring similarity to the opportunity vector.
    pub fn consensus_vote(&self, opportunity: &HdVector) -> ConsensusResult {
        let active_agents: Vec<&SwarmAgent> = self.agents.iter().filter(|a| a.active).collect();

        if active_agents.is_empty() {
            return ConsensusResult {
                score: 0.0,
                votes_for: 0,
                votes_against: 0,
                abstentions: 0,
                proceed: false,
            };
        }

        let mut votes_for = 0u32;
        let mut votes_against = 0u32;
        let mut affinities = Vec::new();

        for agent in &active_agents {
            let affinity = agent.affinity(opportunity);
            affinities.push(affinity);
            if affinity > 0.0 {
                votes_for += 1;
            } else {
                votes_against += 1;
            }
        }

        // Weighted consensus: average affinity
        let score = affinities.iter().sum::<f64>() / affinities.len() as f64;

        ConsensusResult {
            score,
            votes_for,
            votes_against,
            abstentions: 0,
            proceed: score >= self.consensus_threshold,
        }
    }

    /// Find the best agent for a given opportunity (highest affinity).
    pub fn best_agent_for(&self, opportunity: &HdVector) -> Option<&SwarmAgent> {
        self.agents
            .iter()
            .filter(|a| a.active)
            .max_by(|a, b| {
                a.affinity(opportunity)
                    .partial_cmp(&b.affinity(opportunity))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Find the best agent index for a given opportunity.
    pub fn best_agent_index(&self, opportunity: &HdVector) -> Option<usize> {
        self.agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.active)
            .max_by(|(_, a), (_, b)| {
                a.affinity(opportunity)
                    .partial_cmp(&b.affinity(opportunity))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    /// Evolve the swarm: mutate underperforming agents, clone successful ones.
    pub fn evolve(&mut self) {
        // Find agents with enough history to evaluate
        let experienced: Vec<usize> = self
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.total_ops() >= 5)
            .map(|(i, _)| i)
            .collect();

        if experienced.len() < 2 {
            return; // Not enough data to evolve
        }

        // Find best and worst performing agents
        let best_idx = *experienced
            .iter()
            .max_by(|&&a, &&b| {
                self.agents[a]
                    .success_rate()
                    .partial_cmp(&self.agents[b].success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let worst_idx = *experienced
            .iter()
            .min_by(|&&a, &&b| {
                self.agents[a]
                    .success_rate()
                    .partial_cmp(&self.agents[b].success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        if best_idx != worst_idx {
            // Replace worst agent's persona with a mutated copy of the best
            let best_persona = self.agents[best_idx].persona.clone();
            let worst_domain = self.agents[worst_idx].domain.clone();
            let worst_gen = self.agents[worst_idx].generation;

            self.agents[worst_idx].persona = best_persona;
            self.agents[worst_idx].generation = worst_gen + 1;
            self.agents[worst_idx].successes = 0;
            self.agents[worst_idx].failures = 0;
            self.agents[worst_idx].mutate(self.mutation_rate);

            tracing::info!(
                "Evolved: {} (gen {}) cloned from {} persona, domain={}",
                self.agents[worst_idx].id,
                self.agents[worst_idx].generation,
                self.agents[best_idx].id,
                worst_domain
            );
        }
    }

    /// Number of active agents.
    pub fn active_count(&self) -> usize {
        self.agents.iter().filter(|a| a.active).count()
    }

    /// Total agents.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Whether the swarm is empty.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Get a summary of swarm performance.
    pub fn performance_summary(&self) -> SwarmSummary {
        let total_successes: u64 = self.agents.iter().map(|a| a.successes).sum();
        let total_failures: u64 = self.agents.iter().map(|a| a.failures).sum();
        let total = total_successes + total_failures;
        let avg_generation =
            self.agents.iter().map(|a| a.generation as f64).sum::<f64>() / self.agents.len().max(1) as f64;

        SwarmSummary {
            total_agents: self.agents.len(),
            active_agents: self.active_count(),
            total_successes,
            total_failures,
            overall_success_rate: if total > 0 {
                total_successes as f64 / total as f64
            } else {
                0.0
            },
            avg_generation,
        }
    }
}

/// Result of a consensus vote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Weighted consensus score in [-1.0, +1.0].
    pub score: f64,
    /// Number of agents voting for.
    pub votes_for: u32,
    /// Number of agents voting against.
    pub votes_against: u32,
    /// Number of abstaining agents.
    pub abstentions: u32,
    /// Whether the consensus meets the threshold to proceed.
    pub proceed: bool,
}

/// Performance summary of the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSummary {
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_successes: u64,
    pub total_failures: u64,
    pub overall_success_rate: f64,
    pub avg_generation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_agent_has_zero_stats() {
        let agent = SwarmAgent::new("test".into(), "ethereum".into(), DEFAULT_DIM);
        assert_eq!(agent.successes, 0);
        assert_eq!(agent.failures, 0);
        assert_eq!(agent.generation, 0);
        assert!(agent.active);
    }

    #[test]
    fn success_rate_empty() {
        let agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        assert_eq!(agent.success_rate(), 0.0);
    }

    #[test]
    fn success_rate_calculation() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        agent.successes = 3;
        agent.failures = 1;
        assert!((agent.success_rate() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn record_success_increments() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        agent.record_success(None);
        agent.record_success(None);
        assert_eq!(agent.successes, 2);
    }

    #[test]
    fn record_success_with_reinforcement() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        let opp = HdVector::random(DEFAULT_DIM);
        let persona_before = agent.persona.clone();
        agent.record_success(Some(&opp));
        // Persona should be modified by reinforcement
        assert_ne!(agent.persona, persona_before);
    }

    #[test]
    fn record_failure_increments() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        agent.record_failure();
        assert_eq!(agent.failures, 1);
    }

    #[test]
    fn mutation_changes_persona() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        let before = agent.persona.clone();
        agent.mutate(0.5); // 50% mutation rate → should change significantly
        assert_ne!(agent.persona, before);
        assert_eq!(agent.generation, 1);
    }

    #[test]
    fn mutation_zero_rate_is_identity() {
        let mut agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        let before = agent.persona.clone();
        agent.mutate(0.0);
        assert_eq!(agent.persona, before);
        assert_eq!(agent.generation, 1); // Generation still increments
    }

    #[test]
    fn affinity_returns_valid_range() {
        let agent = SwarmAgent::new("test".into(), "eth".into(), DEFAULT_DIM);
        let opp = HdVector::random(DEFAULT_DIM);
        let aff = agent.affinity(&opp);
        assert!(aff >= -1.0 && aff <= 1.0);
    }

    // ── Swarm tests ─────────────────────────────────────

    #[test]
    fn default_harvest_swarm_has_eight_agents() {
        let swarm = Swarm::default_harvest();
        assert_eq!(swarm.len(), 8);
        assert_eq!(swarm.active_count(), 8);
    }

    #[test]
    fn swarm_domains_correct() {
        let swarm = Swarm::default_harvest();
        let domains: Vec<&str> = swarm.agents.iter().map(|a| a.domain.as_str()).collect();
        assert!(domains.contains(&"ethereum"));
        assert!(domains.contains(&"arbitrum"));
        assert!(domains.contains(&"social"));
        assert!(domains.contains(&"onchain"));
    }

    #[test]
    fn consensus_vote_with_active_agents() {
        let swarm = Swarm::default_harvest();
        let opp = HdVector::random(DEFAULT_DIM);
        let result = swarm.consensus_vote(&opp);
        assert!(result.score >= -1.0 && result.score <= 1.0);
        assert_eq!(
            result.votes_for + result.votes_against + result.abstentions,
            swarm.active_count() as u32
        );
    }

    #[test]
    fn consensus_empty_swarm_returns_zero() {
        let swarm = Swarm::new(&[], DEFAULT_DIM);
        let result = swarm.consensus_vote(&HdVector::random(DEFAULT_DIM));
        assert_eq!(result.score, 0.0);
        assert!(!result.proceed);
    }

    #[test]
    fn best_agent_for_returns_some() {
        let swarm = Swarm::default_harvest();
        let opp = HdVector::random(DEFAULT_DIM);
        assert!(swarm.best_agent_for(&opp).is_some());
    }

    #[test]
    fn best_agent_index_consistent() {
        let swarm = Swarm::default_harvest();
        let opp = HdVector::random(DEFAULT_DIM);
        let best = swarm.best_agent_for(&opp).unwrap();
        let best_idx = swarm.best_agent_index(&opp).unwrap();
        assert_eq!(swarm.agents[best_idx].id, best.id);
    }

    #[test]
    fn evolve_requires_experience() {
        let mut swarm = Swarm::default_harvest();
        // No agents have enough operations yet — evolve should be a no-op
        swarm.evolve();
        // All agents should still be generation 0
        assert!(swarm.agents.iter().all(|a| a.generation == 0));
    }

    #[test]
    fn evolve_replaces_worst_with_best() {
        let mut swarm = Swarm::new(&["a", "b"], DEFAULT_DIM);

        // Agent 0: 5/5 success
        for _ in 0..5 {
            swarm.agents[0].record_success(None);
        }
        // Agent 1: 0/5 success
        for _ in 0..5 {
            swarm.agents[1].record_failure();
        }

        let best_persona_before = swarm.agents[0].persona.clone();
        swarm.evolve();

        // Agent 1 should have been evolved (generation > 0, reset stats)
        assert!(swarm.agents[1].generation > 0);
        assert_eq!(swarm.agents[1].successes, 0);
        assert_eq!(swarm.agents[1].failures, 0);
    }

    #[test]
    fn performance_summary_correct() {
        let mut swarm = Swarm::new(&["a", "b"], DEFAULT_DIM);
        swarm.agents[0].successes = 10;
        swarm.agents[0].failures = 2;
        swarm.agents[1].successes = 5;
        swarm.agents[1].failures = 3;

        let summary = swarm.performance_summary();
        assert_eq!(summary.total_agents, 2);
        assert_eq!(summary.total_successes, 15);
        assert_eq!(summary.total_failures, 5);
        assert!((summary.overall_success_rate - 0.75).abs() < 1e-10);
    }

    #[test]
    fn swarm_serializable() {
        let swarm = Swarm::default_harvest();
        let json = serde_json::to_string(&swarm).unwrap();
        let deser: Swarm = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.len(), 8);
    }

    #[test]
    fn consensus_result_serializable() {
        let result = ConsensusResult {
            score: 0.42,
            votes_for: 5,
            votes_against: 3,
            abstentions: 0,
            proceed: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deser: ConsensusResult = serde_json::from_str(&json).unwrap();
        assert!((deser.score - 0.42).abs() < 1e-10);
    }

    #[test]
    fn swarm_mutation_rate_adjustable() {
        let mut swarm = Swarm::default_harvest();
        swarm.mutation_rate = 0.1;
        assert!((swarm.mutation_rate - 0.1).abs() < 1e-10);
    }
}
