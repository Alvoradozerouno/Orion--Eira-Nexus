//! Sovereign Industrial Kernel (SIK)
//!
//! Deterministic execution processor with a 20 W power profile.
//! Zero randomness. Local-only autonomous processing.
//! All state transitions are logged and reproducible.

/// Power budget constants (in milliwatts).
pub const POWER_BUDGET_MW: u32 = 20_000; // 20 W

/// Processor states — strictly ordered, no backwards transitions.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorState {
    Idle,
    Active,
    Suspended,
    Error(String),
}

/// A single execution record appended to the kernel log.
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub processor_id: u32,
    pub decision: String,
    pub power_used_mw: u32,
    pub state: ProcessorState,
}

pub mod decision_making {
    /// Struct representing an autonomous decision-making entity.
    pub struct AutonomousDecisionMaker {
        pub id: u32,
        pub state: String,
    }

    impl AutonomousDecisionMaker {
        /// Creates a new decision maker.
        pub fn new(id: u32, state: &str) -> Self {
            AutonomousDecisionMaker {
                id,
                state: state.to_string(),
            }
        }

        /// Executes a decision-making process.
        pub fn make_decision(&self) -> String {
            format!("Decision made by {}", self.id)
        }
    }

    /// Trait for decision-making strategies.
    pub trait DecisionMaker {
        fn decide(&self) -> String;
    }

    impl DecisionMaker for AutonomousDecisionMaker {
        fn decide(&self) -> String {
            self.make_decision()
        }
    }
}

pub mod industrial_kernel {
    use super::decision_making::{AutonomousDecisionMaker, DecisionMaker};

    /// Struct representing the Industrial Kernel.
    pub struct IndustrialKernel {
        pub processors: Vec<AutonomousDecisionMaker>,
    }

    impl IndustrialKernel {
        /// Creates a new industrial kernel.
        pub fn new() -> Self {
            IndustrialKernel {
                processors: Vec::new(),
            }
        }

        /// Adds a decision maker to the kernel.
        pub fn add_processor(&mut self, decision_maker: AutonomousDecisionMaker) {
            self.processors.push(decision_maker);
        }

        /// Executes all decision makers.
        pub fn execute(&self) {
            for processor in &self.processors {
                println!("{}", processor.decide());
            }
        }
    }

    impl Default for IndustrialKernel {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Sovereign Industrial Kernel — the top-level deterministic execution manager.
///
/// Enforces a 20 W total power budget across all registered processors.
/// Each execution cycle is recorded in an immutable log.
pub struct SovereignIndustrialKernel {
    processors: Vec<decision_making::AutonomousDecisionMaker>,
    execution_log: Vec<ExecutionRecord>,
    total_power_used_mw: u32,
}

impl SovereignIndustrialKernel {
    /// Create a new sovereign kernel.
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            execution_log: Vec::new(),
            total_power_used_mw: 0,
        }
    }

    /// Register a processor with the kernel.
    pub fn add_processor(&mut self, processor: decision_making::AutonomousDecisionMaker) {
        self.processors.push(processor);
    }

    /// Execute all registered processors, subject to the 20 W power budget.
    ///
    /// Returns the number of processors successfully executed.
    pub fn execute(&mut self) -> usize {
        let per_processor_mw = if self.processors.is_empty() {
            0
        } else {
            POWER_BUDGET_MW / self.processors.len() as u32
        };

        let mut executed = 0;
        for processor in &self.processors {
            if self.total_power_used_mw + per_processor_mw > POWER_BUDGET_MW {
                break;
            }
            use decision_making::DecisionMaker;
            let decision = processor.decide();
            self.total_power_used_mw += per_processor_mw;
            self.execution_log.push(ExecutionRecord {
                processor_id: processor.id,
                decision,
                power_used_mw: per_processor_mw,
                state: ProcessorState::Active,
            });
            executed += 1;
        }
        executed
    }

    /// Return the immutable execution log.
    pub fn execution_log(&self) -> &[ExecutionRecord] {
        &self.execution_log
    }

    /// Return total power consumed so far (in milliwatts).
    pub fn total_power_used_mw(&self) -> u32 {
        self.total_power_used_mw
    }
}

impl Default for SovereignIndustrialKernel {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::decision_making::{AutonomousDecisionMaker, DecisionMaker};
    use super::industrial_kernel::IndustrialKernel;
    use super::{SovereignIndustrialKernel, POWER_BUDGET_MW};

    #[test]
    fn test_decision_maker() {
        let decision_maker = AutonomousDecisionMaker::new(1, "active");
        assert_eq!(decision_maker.decide(), "Decision made by 1");
    }

    #[test]
    fn test_industrial_kernel() {
        let mut kernel = IndustrialKernel::new();
        let decision_maker = AutonomousDecisionMaker::new(2, "idle");
        kernel.add_processor(decision_maker);
        kernel.execute();
    }

    #[test]
    fn test_sovereign_kernel_executes_within_budget() {
        let mut sik = SovereignIndustrialKernel::new();
        sik.add_processor(AutonomousDecisionMaker::new(10, "active"));
        sik.add_processor(AutonomousDecisionMaker::new(11, "active"));
        let count = sik.execute();
        assert_eq!(count, 2);
        assert!(sik.total_power_used_mw() <= POWER_BUDGET_MW);
    }

    #[test]
    fn test_execution_log_grows() {
        let mut sik = SovereignIndustrialKernel::new();
        sik.add_processor(AutonomousDecisionMaker::new(20, "active"));
        sik.execute();
        assert_eq!(sik.execution_log().len(), 1);
    }
}