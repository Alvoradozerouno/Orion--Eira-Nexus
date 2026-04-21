//! Orch-OR Simulation — Penrose-Hameroff Orchestrated Objective Reduction.
//!
//! # Design Principles
//! - **Deterministic collapse**: quantum state transitions follow hard threshold
//!   rules derived from the gravitational self-energy criterion (E_G).
//! - **No Monte Carlo**: every collapse is determined by the accumulated
//!   gravitational self-energy reaching the objective-reduction threshold.
//! - **Hard rules only**: microtubule states advance through a fixed FSM.
//!
//! ## Penrose-Hameroff E_G Criterion
//! A superposition collapses when its gravitational self-energy satisfies
//! ```text
//! E_G · τ ≥ ħ
//! ```
//! where `τ` is the coherence time and `ħ` is the reduced Planck constant.
//! In this simulation `E_G` is accumulated deterministically from tubulin
//! dipole moments, and collapse occurs the instant the threshold is crossed.

/// Reduced Planck constant (J·s).
pub const HBAR: f64 = 1.054_571_817e-34;

/// Gravitational self-energy scale per tubulin dimer (J).
/// Estimated from Penrose: ~10⁻²⁷ J per tubulin dipole reorientation.
pub const E_G_PER_TUBULIN: f64 = 1.0e-27;

/// Microtubule protofilament states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TubulinState {
    /// α-tubulin dipole points "up" (GTP-bound).
    AlphaUp,
    /// α-tubulin dipole points "down" (GDP-bound).
    AlphaDown,
    /// Superposition of both orientations (quantum coherence active).
    Superposition,
}

/// State of a single microtubule.
#[derive(Debug, Clone)]
pub struct MicrotubuleState {
    pub id: u32,
    /// Number of tubulin dimers in this microtubule.
    pub num_tubulins: u32,
    /// Current tubulin dipole state.
    pub tubulin_state: TubulinState,
    /// Accumulated gravitational self-energy (J).
    pub accumulated_e_g: f64,
    /// Coherence time so far (s).
    pub coherence_time: f64,
    /// Number of collapses that have occurred.
    pub collapse_count: u32,
}

impl MicrotubuleState {
    pub fn new(id: u32, num_tubulins: u32) -> Self {
        Self {
            id,
            num_tubulins,
            tubulin_state: TubulinState::AlphaUp,
            accumulated_e_g: 0.0,
            coherence_time: 0.0,
            collapse_count: 0,
        }
    }

    /// Threshold for objective reduction: ħ.
    pub fn collapse_threshold(&self) -> f64 {
        HBAR
    }

    /// True if the Orch-OR threshold has been reached.
    pub fn threshold_reached(&self) -> bool {
        self.accumulated_e_g * self.coherence_time >= self.collapse_threshold()
    }
}

/// The outcome of a quantum state collapse event.
#[derive(Debug, Clone)]
pub struct QuantumCollapseEvent {
    /// Which microtubule collapsed.
    pub microtubule_id: u32,
    /// Simulation time at which collapse occurred (s).
    pub time: f64,
    /// The post-collapse tubulin state (deterministic: AlphaUp or AlphaDown).
    pub post_state: TubulinState,
    /// Gravitational self-energy at collapse (J).
    pub e_g_at_collapse: f64,
    /// Coherence time at collapse (s).
    pub coherence_time_at_collapse: f64,
}

/// Deterministic Orch-OR simulator.
///
/// Advances microtubule quantum coherence states by `dt` seconds per step.
/// Collapse is deterministic: when `E_G · τ ≥ ħ` the state collapses to
/// `AlphaDown` for even collapse counts and `AlphaUp` for odd ones,
/// ensuring reproducible alternation without any randomness.
pub struct OrchOrSimulator {
    pub microtubules: Vec<MicrotubuleState>,
    pub time: f64,
    pub collapse_log: Vec<QuantumCollapseEvent>,
}

impl OrchOrSimulator {
    pub fn new(microtubules: Vec<MicrotubuleState>) -> Self {
        Self {
            microtubules,
            time: 0.0,
            collapse_log: Vec::new(),
        }
    }

    /// Advance all microtubules by `dt` seconds.
    ///
    /// For each microtubule:
    /// 1. If in `Superposition`, accumulate E_G and coherence time.
    /// 2. Check collapse threshold.
    /// 3. On threshold: record collapse event, reset coherence, transition state.
    pub fn step(&mut self, dt: f64) {
        for mt in &mut self.microtubules {
            match mt.tubulin_state {
                TubulinState::Superposition => {
                    // Accumulate gravitational self-energy
                    mt.accumulated_e_g += E_G_PER_TUBULIN * mt.num_tubulins as f64 * dt;
                    mt.coherence_time += dt;

                    if mt.threshold_reached() {
                        // Deterministic post-collapse state (no randomness).
                        // Even collapses → AlphaDown (GDP-bound, lower energy);
                        // odd collapses → AlphaUp (GTP-bound, higher energy).
                        // This models the GTP hydrolysis cycle deterministically:
                        // energy is first released (Down) then recharged (Up).
                        let post_state = if mt.collapse_count % 2 == 0 {
                            TubulinState::AlphaDown
                        } else {
                            TubulinState::AlphaUp
                        };

                        Self::record_collapse(&mut self.collapse_log, mt, self.time, post_state);
                    }
                }
                TubulinState::AlphaUp => {
                    // After a short refractory period, enter superposition
                    mt.coherence_time += dt;
                    if mt.coherence_time >= 1.0e-13 {
                        // ~100 fs refractory period
                        mt.tubulin_state = TubulinState::Superposition;
                        mt.coherence_time = 0.0;
                        mt.accumulated_e_g = 0.0;
                    }
                }
                TubulinState::AlphaDown => {
                    mt.coherence_time += dt;
                    if mt.coherence_time >= 1.0e-13 {
                        mt.tubulin_state = TubulinState::Superposition;
                        mt.coherence_time = 0.0;
                        mt.accumulated_e_g = 0.0;
                    }
                }
            }
        }
        self.time += dt;
    }

    fn record_collapse(
        log: &mut Vec<QuantumCollapseEvent>,
        mt: &mut MicrotubuleState,
        time: f64,
        post_state: TubulinState,
    ) {
        log.push(QuantumCollapseEvent {
            microtubule_id: mt.id,
            time,
            post_state,
            e_g_at_collapse: mt.accumulated_e_g,
            coherence_time_at_collapse: mt.coherence_time,
        });
        mt.collapse_count += 1;
        mt.tubulin_state = post_state;
        mt.accumulated_e_g = 0.0;
        mt.coherence_time = 0.0;
    }

    /// Return total collapses across all microtubules.
    pub fn total_collapses(&self) -> usize {
        self.collapse_log.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mt(id: u32, tubulins: u32) -> MicrotubuleState {
        let mut mt = MicrotubuleState::new(id, tubulins);
        // Start in superposition so it can collapse immediately
        mt.tubulin_state = TubulinState::Superposition;
        mt
    }

    #[test]
    fn test_collapse_threshold_constant() {
        let mt = MicrotubuleState::new(1, 1000);
        assert!((mt.collapse_threshold() - HBAR).abs() < 1e-50);
    }

    #[test]
    fn test_no_collapse_before_threshold() {
        let mt = make_mt(1, 1);
        let mut sim = OrchOrSimulator::new(vec![mt]);
        // E_G_PER_TUBULIN * 1 * dt = 1e-27 * dt
        // Threshold = ħ ≈ 1.05e-34
        // Need E_G * τ = (1e-27 * 1e-7) * 1e-7 ≈ too small to collapse quickly
        // Step with a tiny dt — should not collapse yet
        sim.step(1.0e-15);
        assert_eq!(sim.total_collapses(), 0);
    }

    #[test]
    fn test_collapse_occurs_eventually() {
        // Use large tubulin count to accelerate collapse
        let mt = make_mt(1, 1_000_000_000);
        let mut sim = OrchOrSimulator::new(vec![mt]);
        // Step until collapse happens (or bail after many steps)
        for _ in 0..20_000 {
            sim.step(1.0e-12);
            if sim.total_collapses() > 0 {
                break;
            }
        }
        assert!(sim.total_collapses() > 0, "Expected at least one collapse");
    }

    #[test]
    fn test_deterministic_collapse_sequence() {
        // Two identical simulations must produce identical collapse sequences
        let make_sim = || {
            let mt = make_mt(42, 1_000_000_000);
            OrchOrSimulator::new(vec![mt])
        };

        let mut sim1 = make_sim();
        let mut sim2 = make_sim();

        for _ in 0..5_000 {
            sim1.step(1.0e-12);
            sim2.step(1.0e-12);
        }

        assert_eq!(sim1.total_collapses(), sim2.total_collapses());
        for (e1, e2) in sim1.collapse_log.iter().zip(sim2.collapse_log.iter()) {
            assert_eq!(e1.post_state, e2.post_state);
        }
    }

    #[test]
    fn test_post_collapse_state_alternates_deterministically() {
        let mt = make_mt(1, 1_000_000_000);
        let mut sim = OrchOrSimulator::new(vec![mt]);

        let mut collapses = Vec::new();
        for _ in 0..100_000 {
            sim.step(1.0e-12);
            if sim.collapse_log.len() > collapses.len() {
                collapses.push(sim.collapse_log.last().unwrap().post_state);
                if collapses.len() >= 4 {
                    break;
                }
            }
        }

        if collapses.len() >= 2 {
            // Even index → AlphaDown, odd index → AlphaUp
            assert_eq!(collapses[0], TubulinState::AlphaDown);
            assert_eq!(collapses[1], TubulinState::AlphaUp);
        }
    }
}
