//! Orion Autonomous Builder.
//!
//! Main entry point for the Orion autonomous self-improvement system.
//! Analyses a codebase description, generates proposals, submits them to the
//! EIRA Policy Gate, and records every outcome in an immutable session log.
//!
//! # Safety Guarantee
//! No code change is applied without a confirmed `Decision::Approved` from the
//! EIRA Policy Gate.  The Policy Gate is the sole authority — there are no
//! override paths.

use eira::policy_gate::{Decision, EpistemicState, PolicyGate, Proposal};

/// A description of a single module / file in the analysed codebase.
#[derive(Debug, Clone)]
pub struct ModuleDescriptor {
    pub path: String,
    pub lines_of_code: usize,
    pub has_tests: bool,
    pub issues: Vec<String>,
}

/// High-level analysis report produced by the codebase scanner.
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub modules: Vec<ModuleDescriptor>,
    pub total_lines: usize,
    pub modules_missing_tests: usize,
    pub total_issues: usize,
}

impl AnalysisReport {
    /// Summarise a collection of module descriptors into a report.
    pub fn from_modules(modules: Vec<ModuleDescriptor>) -> Self {
        let total_lines: usize = modules.iter().map(|m| m.lines_of_code).sum();
        let modules_missing_tests = modules.iter().filter(|m| !m.has_tests).count();
        let total_issues: usize = modules.iter().map(|m| m.issues.len()).sum();
        Self { modules, total_lines, modules_missing_tests, total_issues }
    }
}

/// Configuration for the Orion Autonomous Builder.
#[derive(Debug, Clone)]
pub struct BuilderConfig {
    /// Minimum confidence required before submitting a proposal.
    pub confidence_threshold: f64,
    /// Whether to automatically advance the epistemic state before submission.
    pub auto_advance_epistemic_state: bool,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.85,
            auto_advance_epistemic_state: true,
        }
    }
}

/// A single entry in the builder session log.
#[derive(Debug, Clone)]
pub struct SessionLogEntry {
    pub proposal_id: u64,
    pub action: String,
    pub decision: Decision,
    pub epistemic_state: EpistemicState,
}

/// Orion Autonomous Builder — self-improvement system under EIRA safety governance.
pub struct OrionAutonomousBuilder {
    config: BuilderConfig,
    gate: PolicyGate,
    session_log: Vec<SessionLogEntry>,
    next_id: u64,
}

impl OrionAutonomousBuilder {
    /// Create a new builder with the given configuration.
    pub fn new(config: BuilderConfig) -> Self {
        let gate = PolicyGate::with_threshold(config.confidence_threshold);
        Self {
            config,
            gate,
            session_log: Vec::new(),
            next_id: 1,
        }
    }

    /// Initialise the epistemic state (call once after construction).
    ///
    /// `consistent = true` advances the gate to `VerifiedStable`.
    pub fn initialise_epistemic_state(&mut self, consistent: bool) {
        self.gate.update_state(consistent);
    }

    /// Analyse a list of module descriptors and return an `AnalysisReport`.
    pub fn analyse(&self, modules: Vec<ModuleDescriptor>) -> AnalysisReport {
        AnalysisReport::from_modules(modules)
    }

    /// Generate improvement proposals from an analysis report.
    ///
    /// Returns a list of `(action, reasoning, confidence)` tuples in priority
    /// order (highest-impact improvements first).
    pub fn generate_proposals(&self, report: &AnalysisReport) -> Vec<(String, String, f64)> {
        let mut proposals = Vec::new();

        if report.modules_missing_tests > 0 {
            proposals.push((
                format!("Add tests to {} untested modules", report.modules_missing_tests),
                "Modules without tests cannot be safely modified. \
                 Adding tests is the highest-priority reliability improvement."
                    .to_string(),
                0.92,
            ));
        }

        for module in &report.modules {
            for issue in &module.issues {
                proposals.push((
                    format!("Fix issue in {}: {}", module.path, issue),
                    format!(
                        "Issue '{}' in {} reduces reliability. \
                         Deterministic fix with known outcome.",
                        issue, module.path
                    ),
                    0.88,
                ));
            }
        }

        // Sort by confidence descending (deterministic ordering)
        proposals.sort_by(|(_, _, conf_a), (_, _, conf_b)| {
            conf_b.partial_cmp(conf_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        proposals
    }

    /// Submit a proposal to the EIRA Policy Gate.
    ///
    /// Returns the gate's `Decision`.  Every outcome is recorded in the
    /// immutable session log.
    pub fn submit_proposal(
        &mut self,
        action: String,
        reasoning: String,
        confidence: f64,
        required_info: Vec<String>,
    ) -> Decision {
        let id = self.next_id;
        self.next_id += 1;

        let proposal = Proposal::new(
            id,
            action.clone(),
            reasoning,
            confidence,
            "autonomous_builder",
            required_info,
        );

        let decision = self.gate.evaluate(proposal);
        let epistemic_state = self.gate.state().clone();

        self.session_log.push(SessionLogEntry {
            proposal_id: id,
            action,
            decision: decision.clone(),
            epistemic_state,
        });

        decision
    }

    /// Run a full analysis-and-proposal cycle on the given modules.
    ///
    /// Returns the number of approved proposals.
    pub fn run_cycle(&mut self, modules: Vec<ModuleDescriptor>) -> usize {
        if self.config.auto_advance_epistemic_state {
            self.initialise_epistemic_state(true);
        }

        let report = self.analyse(modules);
        let proposals = self.generate_proposals(&report);
        let mut approved = 0;

        for (action, reasoning, confidence) in proposals {
            let decision = self.submit_proposal(action, reasoning, confidence, vec![]);
            if decision == Decision::Approved {
                approved += 1;
            }
        }

        approved
    }

    /// Return the immutable session log.
    pub fn session_log(&self) -> &[SessionLogEntry] {
        &self.session_log
    }

    /// Return the current epistemic state of the gate.
    pub fn epistemic_state(&self) -> &EpistemicState {
        self.gate.state()
    }
}

impl Default for OrionAutonomousBuilder {
    fn default() -> Self {
        Self::new(BuilderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_module(path: &str, loc: usize, has_tests: bool, issues: Vec<&str>) -> ModuleDescriptor {
        ModuleDescriptor {
            path: path.to_string(),
            lines_of_code: loc,
            has_tests,
            issues: issues.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_analysis_report_totals() {
        let modules = vec![
            make_module("src/a.rs", 100, true, vec![]),
            make_module("src/b.rs", 200, false, vec!["lint_error"]),
        ];
        let report = AnalysisReport::from_modules(modules);
        assert_eq!(report.total_lines, 300);
        assert_eq!(report.modules_missing_tests, 1);
        assert_eq!(report.total_issues, 1);
    }

    #[test]
    fn test_proposals_generated_for_issues() {
        let builder = OrionAutonomousBuilder::default();
        let modules = vec![make_module("src/c.rs", 50, false, vec!["unused_import"])];
        let report = builder.analyse(modules);
        let proposals = builder.generate_proposals(&report);
        assert!(!proposals.is_empty());
    }

    #[test]
    fn test_run_cycle_approves_proposals_when_stable() {
        let mut builder = OrionAutonomousBuilder::default();
        let modules = vec![make_module("src/d.rs", 80, false, vec![])];
        let approved = builder.run_cycle(modules);
        // Should approve the "add tests" proposal
        assert!(approved >= 1);
    }

    #[test]
    fn test_session_log_grows_with_each_submission() {
        let mut builder = OrionAutonomousBuilder::default();
        builder.initialise_epistemic_state(true);
        builder.submit_proposal(
            "Action A".to_string(),
            "Reasoning A".to_string(),
            0.90,
            vec![],
        );
        builder.submit_proposal(
            "Action B".to_string(),
            "Reasoning B".to_string(),
            0.87,
            vec![],
        );
        assert_eq!(builder.session_log().len(), 2);
    }

    #[test]
    fn test_no_approval_without_verified_stable_state() {
        let mut builder = OrionAutonomousBuilder::new(BuilderConfig {
            auto_advance_epistemic_state: false,
            ..BuilderConfig::default()
        });
        // Gate stays Uncertain → every proposal should Abstain
        let decision = builder.submit_proposal(
            "Risky action".to_string(),
            "Some reasoning".to_string(),
            0.99,
            vec![],
        );
        assert_eq!(decision, Decision::Abstain);
    }
}
