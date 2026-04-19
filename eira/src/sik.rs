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
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::decision_making::{AutonomousDecisionMaker, DecisionMaker};
    use super::industrial_kernel::IndustrialKernel;

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
        kernel.execute(); // This should output "Decision made by 2"
    }
}