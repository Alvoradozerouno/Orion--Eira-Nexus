//! # Orch-OR Consciousness Model
//!
//! A **purely conceptual**, deterministic simulation of the Penrose–Hameroff
//! Orchestrated Objective Reduction (Orch-OR) hypothesis.
//!
//! This module does **not** claim to implement real quantum mechanics.  It
//! provides a computational analogue that captures the *structure* of Orch-OR:
//!
//! * Microtubules accumulate quantum superposition over time.
//! * When the accumulated "gravitational self-energy" exceeds a threshold, an
//!   objective reduction collapses the superposition.
//! * The collapsed state feeds into the next orchestration cycle.
//!
//! ## Determinism guarantee
//!
//! All collapse rules are deterministic threshold comparisons.  There is no
//! random number generation anywhere in this module.
//!
//! ## Example
//!
//! ```rust
//! use physics::orch_or::{OrchOR, Microtubule};
//!
//! let mut orch = OrchOR::new(8);
//! for _ in 0..100 {
//!     orch.orchestrate(0.01);
//! }
//! println!("Collapse count: {}", orch.collapse_count());
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Microtubule
// ---------------------------------------------------------------------------

/// A single microtubule participating in an Orch-OR cycle.
///
/// Each tubulin dimer inside the microtubule is modelled as a two-state
/// quantum bit (qubit analogue) with a real-valued bias `coherence`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microtubule {
    /// Unique identifier.
    pub id: u32,
    /// Number of tubulin dimers in this microtubule.
    pub dimer_count: u32,
    /// Accumulated quantum coherence (0.0 – 1.0).
    /// Increases deterministically as orchestration proceeds.
    pub coherence: f64,
    /// Gravitational self-energy proxy (dimensionless, proportional to mass
    /// separation of superposed states).
    pub gravitational_self_energy: f64,
    /// Whether this microtubule is currently in a collapsed (classical) state.
    pub collapsed: bool,
}

impl Microtubule {
    /// Construct a new microtubule with `dimer_count` dimers and zero initial
    /// coherence.
    pub fn new(id: u32, dimer_count: u32) -> Self {
        Self {
            id,
            dimer_count,
            coherence: 0.0,
            gravitational_self_energy: 0.0,
            collapsed: false,
        }
    }

    /// Accumulate coherence and gravitational self-energy over time step `dt`.
    ///
    /// The accumulation rate is proportional to the number of dimers (more
    /// dimers → faster decoherence).
    pub fn accumulate(&mut self, dt: f64) {
        if !self.collapsed {
            let rate = self.dimer_count as f64 * 0.01;
            self.coherence = (self.coherence + rate * dt).min(1.0);
            self.gravitational_self_energy += rate * dt * self.coherence;
        }
    }

    /// Reset to the post-collapse ground state.
    pub fn reset(&mut self) {
        self.coherence = 0.0;
        self.gravitational_self_energy = 0.0;
        self.collapsed = false;
    }
}

// ---------------------------------------------------------------------------
// ObjectiveReduction
// ---------------------------------------------------------------------------

/// The result of a single objective-reduction event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveReduction {
    /// IDs of the microtubules that collapsed in this event.
    pub collapsed_ids: Vec<u32>,
    /// Total gravitational self-energy released.
    pub total_energy_released: f64,
    /// Cumulative simulation time at which the collapse occurred.
    pub time: f64,
    /// Orchestration cycle index.
    pub cycle: u64,
}

// ---------------------------------------------------------------------------
// OrchOR
// ---------------------------------------------------------------------------

/// Orchestrated Objective Reduction engine.
///
/// Manages a population of [`Microtubule`]s and drives them through
/// orchestration → coherence-accumulation → collapse cycles.
pub struct OrchOR {
    /// All microtubules in this neural bundle.
    microtubules: Vec<Microtubule>,
    /// Gravitational self-energy threshold that triggers a collapse.
    collapse_threshold: f64,
    /// Current simulation time (arbitrary units).
    pub time: f64,
    /// Number of completed orchestration cycles.
    cycle: u64,
    /// History of all collapse events (append-only).
    collapse_history: Vec<ObjectiveReduction>,
}

impl OrchOR {
    /// Create a new Orch-OR engine with `n` microtubules.
    ///
    /// Microtubule dimer counts are assigned deterministically based on the
    /// index, cycling through a fixed set of values.
    pub fn new(n: u32) -> Self {
        let dimer_counts = [8u32, 13, 21, 34, 55, 89, 144, 233];
        let microtubules = (0..n)
            .map(|i| Microtubule::new(i, dimer_counts[(i as usize) % dimer_counts.len()]))
            .collect();
        Self {
            microtubules,
            collapse_threshold: 0.5,
            time: 0.0,
            cycle: 0,
            collapse_history: Vec::new(),
        }
    }

    /// Set the gravitational self-energy collapse threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.collapse_threshold = threshold;
        self
    }

    /// Run one orchestration step of duration `dt`.
    ///
    /// Each microtubule accumulates coherence; any that exceed the collapse
    /// threshold undergo objective reduction and are reset.
    pub fn orchestrate(&mut self, dt: f64) {
        // Accumulate coherence in all microtubules
        for mt in &mut self.microtubules {
            mt.accumulate(dt);
        }

        // Identify microtubules ready to collapse
        let collapsed_ids: Vec<u32> = self
            .microtubules
            .iter()
            .filter(|mt| mt.gravitational_self_energy >= self.collapse_threshold)
            .map(|mt| mt.id)
            .collect();

        if !collapsed_ids.is_empty() {
            let total_energy: f64 = self
                .microtubules
                .iter()
                .filter(|mt| collapsed_ids.contains(&mt.id))
                .map(|mt| mt.gravitational_self_energy)
                .sum();

            let event = ObjectiveReduction {
                collapsed_ids: collapsed_ids.clone(),
                total_energy_released: total_energy,
                time: self.time + dt,
                cycle: self.cycle,
            };
            self.collapse_history.push(event);

            // Reset collapsed microtubules
            for mt in &mut self.microtubules {
                if collapsed_ids.contains(&mt.id) {
                    mt.objective_reduce();
                }
            }
        }

        self.time += dt;
        self.cycle += 1;
    }

    /// Total number of collapse events across all cycles.
    pub fn collapse_count(&self) -> usize {
        self.collapse_history.len()
    }

    /// Return a read-only reference to the collapse history.
    pub fn collapse_history(&self) -> &[ObjectiveReduction] {
        &self.collapse_history
    }

    /// Return a read-only slice of all microtubules.
    pub fn microtubules(&self) -> &[Microtubule] {
        &self.microtubules
    }

    /// Check whether *any* microtubule is currently in a superposed
    /// (non-collapsed) state.
    pub fn any_superposed(&self) -> bool {
        self.microtubules.iter().any(|mt| !mt.collapsed)
    }

    /// Compute the mean coherence across all microtubules.
    pub fn mean_coherence(&self) -> f64 {
        if self.microtubules.is_empty() {
            return 0.0;
        }
        let total: f64 = self.microtubules.iter().map(|mt| mt.coherence).sum();
        total / self.microtubules.len() as f64
    }
}

// Extend Microtubule with the collapse method
impl Microtubule {
    /// Perform objective reduction: mark the microtubule as collapsed and
    /// prepare it for the next orchestration cycle.
    pub fn objective_reduce(&mut self) {
        self.collapsed = true;
        // Coherence and energy are preserved in the record; the microtubule is
        // reset on the *next* orchestration call once it leaves the collapsed
        // state.
        self.coherence = 0.0;
        self.gravitational_self_energy = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_microtubule_zero_coherence() {
        let mt = Microtubule::new(0, 13);
        assert_eq!(mt.coherence, 0.0);
        assert_eq!(mt.gravitational_self_energy, 0.0);
        assert!(!mt.collapsed);
    }

    #[test]
    fn test_accumulate_increases_coherence() {
        let mut mt = Microtubule::new(0, 13);
        mt.accumulate(0.1);
        assert!(mt.coherence > 0.0, "coherence should increase");
        assert!(mt.gravitational_self_energy > 0.0);
    }

    #[test]
    fn test_coherence_capped_at_one() {
        let mut mt = Microtubule::new(0, 233);
        for _ in 0..1000 {
            mt.accumulate(0.1);
        }
        assert!(mt.coherence <= 1.0 + f64::EPSILON);
    }

    #[test]
    fn test_objective_reduce_resets_state() {
        let mut mt = Microtubule::new(0, 13);
        mt.accumulate(0.5);
        mt.objective_reduce();
        assert_eq!(mt.coherence, 0.0);
        assert!(mt.collapsed);
    }

    #[test]
    fn test_orch_or_collapses_after_threshold() {
        let mut orch = OrchOR::new(4).with_threshold(0.1);
        for _ in 0..200 {
            orch.orchestrate(0.01);
        }
        assert!(
            orch.collapse_count() > 0,
            "should have at least one collapse"
        );
    }

    #[test]
    fn test_orch_or_deterministic() {
        let run = || {
            let mut orch = OrchOR::new(4).with_threshold(0.1);
            for _ in 0..100 {
                orch.orchestrate(0.01);
            }
            orch.collapse_count()
        };
        assert_eq!(run(), run(), "OrchOR must be deterministic");
    }

    #[test]
    fn test_mean_coherence_bounded() {
        let mut orch = OrchOR::new(8);
        for _ in 0..50 {
            orch.orchestrate(0.01);
        }
        let mc = orch.mean_coherence();
        assert!(mc >= 0.0 && mc <= 1.0);
    }

    #[test]
    fn test_collapse_history_grows() {
        let mut orch = OrchOR::new(4).with_threshold(0.05);
        for _ in 0..500 {
            orch.orchestrate(0.01);
        }
        let hist = orch.collapse_history();
        assert!(!hist.is_empty());
        // History is append-only: timestamps should be non-decreasing
        for w in hist.windows(2) {
            assert!(w[1].time >= w[0].time);
        }
    }
}
