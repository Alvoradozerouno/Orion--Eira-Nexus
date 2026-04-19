//! Orchestrated Objective Reduction (Orch-OR) simulation.
//!
//! Based on the Penrose-Hameroff model of quantum consciousness.
//! Implementation rules:
//! * Hard deterministic state transitions – no Monte Carlo, no randomness.
//! * Collapse is triggered when the objective reduction threshold is exceeded.
//! * Microtubule dynamics are modelled as discrete state machines.

/// State of a single microtubule quantum unit.
#[derive(Debug, Clone, PartialEq)]
pub enum MicrotubuleState {
    /// Quantum superposition – both classical states coexist.
    Superposition,
    /// Objective reduction in progress – the collapse has been triggered.
    Collapsing,
    /// Fully collapsed into a classical state.
    Collapsed,
}

/// A quantum state subject to Orchestrated Objective Reduction.
#[derive(Debug, Clone)]
pub struct QuantumState {
    /// Current microtubule state.
    pub state: MicrotubuleState,
    /// Energy-difference threshold E_G that triggers objective reduction (J).
    pub reduction_threshold: f64,
    /// Accumulated gravitational self-energy difference (J).
    pub energy_difference: f64,
    /// Number of simulation steps executed.
    pub steps: u64,
}

impl QuantumState {
    /// Construct a new quantum state in superposition.
    pub fn new(reduction_threshold: f64) -> Self {
        QuantumState {
            state: MicrotubuleState::Superposition,
            reduction_threshold,
            energy_difference: 0.0,
            steps: 0,
        }
    }

    /// Advance the simulation by one deterministic step.
    ///
    /// `energy_increment` represents the gravitational self-energy added this
    /// step (positive or negative, in Joules). Collapse is triggered when
    /// |energy_difference| ≥ reduction_threshold.
    pub fn step(&mut self, energy_increment: f64) {
        self.steps += 1;
        self.energy_difference += energy_increment;

        match self.state {
            MicrotubuleState::Superposition => {
                if self.energy_difference.abs() >= self.reduction_threshold {
                    self.state = MicrotubuleState::Collapsing;
                }
            }
            MicrotubuleState::Collapsing => {
                // One additional step to complete the collapse.
                self.state = MicrotubuleState::Collapsed;
            }
            MicrotubuleState::Collapsed => {
                // Terminal state – no further transitions.
            }
        }
    }

    /// Return `true` when the quantum state has fully collapsed.
    pub fn is_collapsed(&self) -> bool {
        self.state == MicrotubuleState::Collapsed
    }
}

/// An Orch-OR simulation containing a collection of microtubule quantum states.
pub struct OrchOrSimulation {
    /// All quantum states tracked by this simulation.
    pub states: Vec<QuantumState>,
    /// Shared energy increment applied to all states per step.
    pub energy_increment_per_step: f64,
}

impl OrchOrSimulation {
    /// Create a simulation with the given number of microtubule units.
    pub fn new(
        unit_count: usize,
        reduction_threshold: f64,
        energy_increment_per_step: f64,
    ) -> Self {
        OrchOrSimulation {
            states: (0..unit_count)
                .map(|_| QuantumState::new(reduction_threshold))
                .collect(),
            energy_increment_per_step,
        }
    }

    /// Advance all units by one deterministic step.
    pub fn step(&mut self) {
        let inc = self.energy_increment_per_step;
        for state in &mut self.states {
            state.step(inc);
        }
    }

    /// Return the number of fully collapsed units.
    pub fn collapsed_count(&self) -> usize {
        self.states.iter().filter(|s| s.is_collapsed()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_deterministic() {
        // Two identical simulations must collapse at exactly the same step.
        let run = |steps: u32| -> MicrotubuleState {
            let mut qs = QuantumState::new(1.0);
            for _ in 0..steps {
                qs.step(0.4);
            }
            qs.state.clone()
        };

        // After 3 steps: energy = 1.2 ≥ 1.0 → Collapsing.
        // After 4 steps: energy = 1.6 → Collapsed.
        assert_eq!(run(3), MicrotubuleState::Collapsing);
        assert_eq!(run(4), MicrotubuleState::Collapsed);
        // Both runs of the same step count must agree.
        assert_eq!(run(4), run(4));
    }

    #[test]
    fn test_no_collapse_below_threshold() {
        let mut qs = QuantumState::new(10.0);
        for _ in 0..5 {
            qs.step(0.1);
        }
        assert_eq!(qs.state, MicrotubuleState::Superposition);
    }

    #[test]
    fn test_collapsed_is_terminal() {
        let mut qs = QuantumState::new(0.5);
        for _ in 0..20 {
            qs.step(1.0);
        }
        assert_eq!(qs.state, MicrotubuleState::Collapsed);
    }

    #[test]
    fn test_simulation_collapsed_count() {
        let mut sim = OrchOrSimulation::new(4, 1.0, 0.4);
        // After 4 steps all 4 units should be Collapsed.
        for _ in 0..4 {
            sim.step();
        }
        assert_eq!(sim.collapsed_count(), 4);
    }
}
