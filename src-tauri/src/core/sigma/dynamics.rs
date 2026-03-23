//! Non-linear dynamics engine for opportunity prediction.
//!
//! Inspired by Lotka-Volterra predator-prey dynamics and chaotic attractors.
//! Models opportunity ecosystems as interacting populations:
//! - Gas pressure (resource cost)
//! - Community signal (social momentum)
//! - Deadline urgency (time pressure)
//! - Value estimate (expected reward)
//!
//! These interact with non-linear feedback loops to predict "waves" of
//! related airdrops before they hit mainstream discovery channels.
//!
//! Also provides Hurst exponent estimation for self-similarity detection
//! across chains (fractal analysis of opportunity time series).

use serde::{Deserialize, Serialize};

/// State vector for a single opportunity ecosystem.
/// Each component represents a normalized signal in [0.0, 1.0].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemState {
    /// Gas cost pressure (higher = more expensive to claim).
    pub gas_pressure: f64,
    /// Community/social signal strength.
    pub community_signal: f64,
    /// Deadline urgency (approaches 1.0 as deadline nears).
    pub deadline_urgency: f64,
    /// Estimated value of opportunity.
    pub value_estimate: f64,
}

impl EcosystemState {
    /// Create from raw values, clamping to [0, 1].
    pub fn new(gas: f64, community: f64, urgency: f64, value: f64) -> Self {
        Self {
            gas_pressure: gas.clamp(0.0, 1.0),
            community_signal: community.clamp(0.0, 1.0),
            deadline_urgency: urgency.clamp(0.0, 1.0),
            value_estimate: value.clamp(0.0, 1.0),
        }
    }

    /// Convert to array for integration.
    fn as_array(&self) -> [f64; 4] {
        [
            self.gas_pressure,
            self.community_signal,
            self.deadline_urgency,
            self.value_estimate,
        ]
    }

    /// Create from array.
    fn from_array(arr: [f64; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// Euclidean distance to another state.
    pub fn distance(&self, other: &EcosystemState) -> f64 {
        let a = self.as_array();
        let b = other.as_array();
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Parameters for the non-linear dynamics model.
/// These control the interaction strengths between ecosystem components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsParams {
    /// Gas pressure growth rate (how fast gas rises with community activity).
    pub gas_growth: f64,
    /// Community signal decay rate (natural cooling).
    pub community_decay: f64,
    /// Urgency acceleration (exponential ramp near deadline).
    pub urgency_accel: f64,
    /// Value-community coupling (community boosts perceived value).
    pub value_community_coupling: f64,
    /// Gas-value inhibition (high gas suppresses effective value).
    pub gas_value_inhibition: f64,
    /// Community-urgency amplification (urgency drives community FOMO).
    pub community_urgency_amp: f64,
}

impl Default for DynamicsParams {
    fn default() -> Self {
        Self {
            gas_growth: 0.3,
            community_decay: 0.1,
            urgency_accel: 0.5,
            value_community_coupling: 0.4,
            gas_value_inhibition: 0.2,
            community_urgency_amp: 0.3,
        }
    }
}

/// The non-linear dynamics engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsEngine {
    pub params: DynamicsParams,
}

impl DynamicsEngine {
    /// Create with default parameters.
    pub fn new() -> Self {
        Self {
            params: DynamicsParams::default(),
        }
    }

    /// Create with custom parameters.
    pub fn with_params(params: DynamicsParams) -> Self {
        Self { params }
    }

    /// Compute the derivative (rate of change) for each state variable.
    /// This is the "heart" of the non-linear model:
    ///
    /// dG/dt = gas_growth × C × U - gas_decay × G
    /// dC/dt = -community_decay × C + community_urgency_amp × U × (1 - C)
    /// dU/dt = urgency_accel × U × (1 - U)  [logistic growth toward deadline]
    /// dV/dt = value_community_coupling × C - gas_value_inhibition × G × V
    fn derivatives(&self, state: &EcosystemState) -> [f64; 4] {
        let p = &self.params;
        let g = state.gas_pressure;
        let c = state.community_signal;
        let u = state.deadline_urgency;
        let v = state.value_estimate;

        let dg = p.gas_growth * c * u - 0.15 * g; // Gas rises with community+urgency, decays naturally
        let dc = -p.community_decay * c + p.community_urgency_amp * u * (1.0 - c); // Community: FOMO + natural decay
        let du = p.urgency_accel * u * (1.0 - u); // Urgency: logistic growth (S-curve toward 1.0)
        let dv = p.value_community_coupling * c * (1.0 - v) - p.gas_value_inhibition * g * v; // Value: community boosts, gas suppresses

        [dg, dc, du, dv]
    }

    /// Euler integration: advance the state by `dt` time units.
    pub fn step(&self, state: &EcosystemState, dt: f64) -> EcosystemState {
        let derivs = self.derivatives(state);
        let current = state.as_array();
        let mut next = [0.0f64; 4];

        for i in 0..4 {
            next[i] = (current[i] + derivs[i] * dt).clamp(0.0, 1.0);
        }

        EcosystemState::from_array(next)
    }

    /// Simulate a trajectory for `steps` iterations with time step `dt`.
    /// Returns the full trajectory (including initial state).
    pub fn simulate(
        &self,
        initial: &EcosystemState,
        steps: usize,
        dt: f64,
    ) -> Vec<EcosystemState> {
        let mut trajectory = Vec::with_capacity(steps + 1);
        trajectory.push(initial.clone());

        let mut current = initial.clone();
        for _ in 0..steps {
            current = self.step(&current, dt);
            trajectory.push(current.clone());
        }

        trajectory
    }

    /// Predict the "wave score" — likelihood this opportunity is entering
    /// a high-value phase. Based on trajectory gradient analysis.
    /// Returns a score in [0.0, 1.0] where higher = more promising dynamics.
    pub fn wave_score(&self, state: &EcosystemState) -> f64 {
        let derivs = self.derivatives(state);

        // Value is rising AND community is rising → bullish wave
        let value_momentum = derivs[3].max(0.0);
        let community_momentum = derivs[1].max(0.0);

        // Gas is NOT rising → favorable conditions
        let gas_favorability = (1.0 - derivs[0].max(0.0)).max(0.0);

        // Combined wave score with weighted factors
        let raw = 0.4 * value_momentum + 0.3 * community_momentum + 0.3 * gas_favorability;
        raw.clamp(0.0, 1.0)
    }

    /// Compare a current state trajectory against a reference trajectory
    /// to detect self-similar patterns. Returns similarity in [0.0, 1.0].
    pub fn trajectory_similarity(
        traj_a: &[EcosystemState],
        traj_b: &[EcosystemState],
    ) -> f64 {
        let len = traj_a.len().min(traj_b.len());
        if len == 0 {
            return 0.0;
        }

        let total_distance: f64 = traj_a
            .iter()
            .zip(traj_b.iter())
            .take(len)
            .map(|(a, b)| a.distance(b))
            .sum();

        let avg_distance = total_distance / len as f64;
        // Max distance in 4D unit cube is 2.0
        (1.0 - avg_distance / 2.0).max(0.0)
    }
}

impl Default for DynamicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate the Hurst exponent of a time series using Rescaled Range (R/S) analysis.
/// H ≈ 0.5 → random walk (no memory)
/// H > 0.5 → persistent (trending, self-similar)
/// H < 0.5 → anti-persistent (mean-reverting)
///
/// Returns the estimated Hurst exponent, or None if insufficient data.
pub fn hurst_exponent(series: &[f64]) -> Option<f64> {
    let n = series.len();
    if n < 20 {
        return None; // Need sufficient data
    }

    let mut log_rs = Vec::new();
    let mut log_n = Vec::new();

    // Try different window sizes
    let mut window_size = 10;
    while window_size <= n / 2 {
        let mut rs_values = Vec::new();

        for chunk in series.chunks(window_size) {
            if chunk.len() < window_size {
                break;
            }

            let mean: f64 = chunk.iter().sum::<f64>() / chunk.len() as f64;

            // Cumulative deviations from mean
            let mut cumsum = Vec::with_capacity(chunk.len());
            let mut running = 0.0;
            for &x in chunk {
                running += x - mean;
                cumsum.push(running);
            }

            let range = cumsum.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - cumsum.iter().cloned().fold(f64::INFINITY, f64::min);

            let variance: f64 =
                chunk.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / chunk.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev > 1e-10 {
                rs_values.push(range / std_dev);
            }
        }

        if !rs_values.is_empty() {
            let avg_rs: f64 = rs_values.iter().sum::<f64>() / rs_values.len() as f64;
            if avg_rs > 0.0 {
                log_rs.push(avg_rs.ln());
                log_n.push((window_size as f64).ln());
            }
        }

        window_size = (window_size as f64 * 1.5) as usize;
    }

    if log_rs.len() < 2 {
        return None;
    }

    // Linear regression: log(R/S) = H * log(n) + c
    Some(linear_regression_slope(&log_n, &log_rs))
}

/// Simple linear regression slope (least squares).
fn linear_regression_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_xx: f64 = x.iter().map(|a| a * a).sum();

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-10 {
        return 0.5; // Default to random walk if degenerate
    }

    (n * sum_xy - sum_x * sum_y) / denom
}

/// Classify a Hurst exponent into a human-readable regime.
pub fn hurst_regime(h: f64) -> &'static str {
    if h < 0.4 {
        "anti-persistent (mean-reverting)"
    } else if h < 0.6 {
        "random walk (no memory)"
    } else {
        "persistent (trending)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_state_clamped() {
        let state = EcosystemState::new(-0.5, 1.5, 0.5, 0.3);
        assert_eq!(state.gas_pressure, 0.0);
        assert_eq!(state.community_signal, 1.0);
    }

    #[test]
    fn ecosystem_distance_self_is_zero() {
        let state = EcosystemState::new(0.5, 0.5, 0.5, 0.5);
        assert!((state.distance(&state)).abs() < 1e-10);
    }

    #[test]
    fn ecosystem_distance_max() {
        let a = EcosystemState::new(0.0, 0.0, 0.0, 0.0);
        let b = EcosystemState::new(1.0, 1.0, 1.0, 1.0);
        let d = a.distance(&b);
        assert!((d - 2.0).abs() < 1e-10, "max distance should be 2.0: {d}");
    }

    #[test]
    fn dynamics_engine_default() {
        let engine = DynamicsEngine::new();
        assert!((engine.params.gas_growth - 0.3).abs() < 1e-10);
    }

    #[test]
    fn step_stays_in_bounds() {
        let engine = DynamicsEngine::new();
        let state = EcosystemState::new(0.1, 0.8, 0.9, 0.5);
        let next = engine.step(&state, 0.1);

        assert!(next.gas_pressure >= 0.0 && next.gas_pressure <= 1.0);
        assert!(next.community_signal >= 0.0 && next.community_signal <= 1.0);
        assert!(next.deadline_urgency >= 0.0 && next.deadline_urgency <= 1.0);
        assert!(next.value_estimate >= 0.0 && next.value_estimate <= 1.0);
    }

    #[test]
    fn simulate_produces_correct_length() {
        let engine = DynamicsEngine::new();
        let initial = EcosystemState::new(0.1, 0.3, 0.2, 0.5);
        let trajectory = engine.simulate(&initial, 100, 0.01);
        assert_eq!(trajectory.len(), 101); // initial + 100 steps
    }

    #[test]
    fn urgency_increases_monotonically() {
        let engine = DynamicsEngine::new();
        let initial = EcosystemState::new(0.1, 0.1, 0.1, 0.5);
        let trajectory = engine.simulate(&initial, 50, 0.1);

        // Urgency should generally increase (logistic growth)
        let first_urgency = trajectory[0].deadline_urgency;
        let last_urgency = trajectory.last().unwrap().deadline_urgency;
        assert!(
            last_urgency > first_urgency,
            "urgency should increase: {first_urgency} → {last_urgency}"
        );
    }

    #[test]
    fn wave_score_in_range() {
        let engine = DynamicsEngine::new();
        let state = EcosystemState::new(0.2, 0.6, 0.7, 0.5);
        let score = engine.wave_score(&state);
        assert!(score >= 0.0 && score <= 1.0, "wave score out of range: {score}");
    }

    #[test]
    fn high_community_urgency_gives_high_wave_score() {
        let engine = DynamicsEngine::new();
        let bullish = EcosystemState::new(0.1, 0.8, 0.9, 0.3);
        let bearish = EcosystemState::new(0.9, 0.1, 0.1, 0.3);

        let score_bull = engine.wave_score(&bullish);
        let score_bear = engine.wave_score(&bearish);

        assert!(
            score_bull > score_bear,
            "bullish should score higher: bull={score_bull} bear={score_bear}"
        );
    }

    #[test]
    fn trajectory_similarity_self_is_one() {
        let engine = DynamicsEngine::new();
        let initial = EcosystemState::new(0.1, 0.3, 0.2, 0.5);
        let traj = engine.simulate(&initial, 20, 0.1);
        let sim = DynamicsEngine::trajectory_similarity(&traj, &traj);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trajectory_similarity_different_is_low() {
        let engine = DynamicsEngine::new();
        let traj_a = engine.simulate(&EcosystemState::new(0.1, 0.1, 0.1, 0.9), 20, 0.1);
        let traj_b = engine.simulate(&EcosystemState::new(0.9, 0.9, 0.9, 0.1), 20, 0.1);
        let sim = DynamicsEngine::trajectory_similarity(&traj_a, &traj_b);
        assert!(sim < 0.8, "divergent trajectories should be dissimilar: {sim}");
    }

    #[test]
    fn trajectory_similarity_empty() {
        let sim = DynamicsEngine::trajectory_similarity(&[], &[]);
        assert_eq!(sim, 0.0);
    }

    // ── Hurst exponent tests ─────────────────────────────

    #[test]
    fn hurst_needs_minimum_data() {
        let short = vec![1.0; 10];
        assert!(hurst_exponent(&short).is_none());
    }

    #[test]
    fn hurst_white_noise_near_half() {
        // White noise (independent increments) should have H ≈ 0.5
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let noise: Vec<f64> = (0..2000).map(|_| rng.gen_range(-1.0..1.0)).collect();

        let h = hurst_exponent(&noise).unwrap();
        assert!(
            h > 0.2 && h < 0.8,
            "white noise Hurst should be near 0.5: {h}"
        );
    }

    #[test]
    fn hurst_random_walk_is_persistent() {
        // A cumulative random walk (integrated noise) should have H > 0.5
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut walk = Vec::with_capacity(2000);
        let mut val = 0.0;
        for _ in 0..2000 {
            val += rng.gen_range(-1.0..1.0);
            walk.push(val);
        }

        let h = hurst_exponent(&walk).unwrap();
        assert!(h > 0.5, "random walk levels should have H > 0.5: {h}");
    }

    #[test]
    fn hurst_persistent_series() {
        // Generate a trending/persistent series
        let series: Vec<f64> = (0..200).map(|i| (i as f64) * 0.1 + (i as f64 * 0.1).sin()).collect();
        let h = hurst_exponent(&series).unwrap();
        assert!(h > 0.5, "persistent series should have H > 0.5: {h}");
    }

    #[test]
    fn hurst_regime_classification() {
        assert_eq!(hurst_regime(0.3), "anti-persistent (mean-reverting)");
        assert_eq!(hurst_regime(0.5), "random walk (no memory)");
        assert_eq!(hurst_regime(0.7), "persistent (trending)");
    }

    #[test]
    fn linear_regression_slope_exact() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![2.0, 4.0, 6.0, 8.0]; // y = 2x
        let slope = linear_regression_slope(&x, &y);
        assert!((slope - 2.0).abs() < 1e-10, "slope should be 2.0: {slope}");
    }

    #[test]
    fn dynamics_engine_serializable() {
        let engine = DynamicsEngine::new();
        let json = serde_json::to_string(&engine).unwrap();
        let deser: DynamicsEngine = serde_json::from_str(&json).unwrap();
        assert!((deser.params.gas_growth - 0.3).abs() < 1e-10);
    }

    #[test]
    fn ecosystem_state_serializable() {
        let state = EcosystemState::new(0.5, 0.6, 0.7, 0.8);
        let json = serde_json::to_string(&state).unwrap();
        let deser: EcosystemState = serde_json::from_str(&json).unwrap();
        assert!((deser.gas_pressure - 0.5).abs() < 1e-10);
    }
}
