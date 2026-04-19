//! Orion Autonomous Builder
//!
//! Analyzes a codebase, generates deterministic improvement proposals, and
//! submits every proposal to the EIRA Policy Gate.  A change is only applied
//! when the gate returns `Decision::Approved` – no exceptions.

use eira::{Decision, EpistemicState, PolicyGate, Proposal};

/// A code-improvement proposal produced by the Orion builder.
#[derive(Debug, Clone)]
pub struct CodeProposal {
    /// Unique sequential identifier.
    pub id: u32,
    /// Path of the file being modified.
    pub file_path: String,
    /// Human-readable description of the proposed change.
    pub description: String,
    /// Confidence in the proposal [0.0, 1.0].
    pub confidence: f32,
    /// The generated code to apply (if approved).
    pub generated_code: String,
    /// Contextual facts required to evaluate the proposal.
    pub required_info: Vec<String>,
}

impl CodeProposal {
    /// Build a new code proposal.
    pub fn new(
        id: u32,
        file_path: &str,
        description: &str,
        confidence: f32,
        generated_code: &str,
        required_info: Vec<String>,
    ) -> Self {
        CodeProposal {
            id,
            file_path: file_path.to_string(),
            description: description.to_string(),
            confidence,
            generated_code: generated_code.to_string(),
            required_info,
        }
    }
}

/// Orion Autonomous Builder – self-improving system under strict EIRA control.
///
/// All changes are proposed deterministically and gated by EIRA.
/// Nothing is applied unless the Policy Gate explicitly approves.
/// The gate starts in `Uncertain` state; call `verify()` once the
/// knowledge base has been validated.
pub struct OrionAutonomousBuilder {
    gate: PolicyGate,
    proposal_counter: u32,
}

impl OrionAutonomousBuilder {
    /// Create a builder with the given minimum confidence threshold.
    ///
    /// The gate starts in `Uncertain` state (EIRA spec default).
    /// Call `verify()` to transition to `VerifiedStable`.
    pub fn new(min_confidence_threshold: f32) -> Self {
        OrionAutonomousBuilder {
            gate: PolicyGate::new(min_confidence_threshold),
            proposal_counter: 0,
        }
    }

    /// Mark the knowledge base as verified and transition to `VerifiedStable`.
    pub fn verify(&mut self) {
        self.gate.set_state(EpistemicState::VerifiedStable);
    }

    /// Submit a code proposal to the EIRA gate.
    ///
    /// Returns the gate's `Decision`.  The caller is responsible for
    /// applying the change only when `Decision::Approved` is returned.
    pub fn submit(&mut self, proposal: &CodeProposal) -> Decision {
        let eira_proposal = Proposal::new(
            proposal.id,
            &proposal.file_path,
            &proposal.description,
            proposal.confidence,
            "2026-04-19",
            proposal.required_info.clone(),
        );
        let decision = self.gate.evaluate(&eira_proposal);
        if decision == Decision::Approved {
            println!(
                "ORION [APPROVED] Applying change to '{}': {}",
                proposal.file_path, proposal.description
            );
        } else {
            println!(
                "ORION [GATE: {:?}] Proposal for '{}' not applied.",
                decision, proposal.file_path
            );
        }
        decision
    }

    /// Propose a change using raw fields (convenience wrapper).
    pub fn propose(
        &mut self,
        file_path: &str,
        description: &str,
        code: &str,
        confidence: f32,
        required_info: Vec<String>,
    ) -> Decision {
        self.proposal_counter += 1;
        let proposal = CodeProposal::new(
            self.proposal_counter,
            file_path,
            description,
            confidence,
            code,
            required_info,
        );
        self.submit(&proposal)
    }

    /// Return the number of evaluated proposals (audit trail length).
    pub fn audit_trail_len(&self) -> usize {
        self.gate.history().len()
    }

    /// View the full audit trail via the gate.
    pub fn gate(&self) -> &PolicyGate {
        &self.gate
    }

    /// Mutable access to the underlying gate (e.g. to change epistemic state).
    pub fn gate_mut(&mut self) -> &mut PolicyGate {
        &mut self.gate
    }

    /// Submit a pre-built `eira::Proposal` directly to the gate.
    pub fn submit_raw(&mut self, proposal: &Proposal) -> Decision {
        let decision = self.gate.evaluate(proposal);
        if decision == Decision::Approved {
            println!("ORION [APPROVED] Applying change: {}", proposal.action);
        } else {
            println!(
                "ORION [GATE: {:?}] Proposal '{}' not applied.",
                decision, proposal.action
            );
        }
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approved_change() {
        let mut builder = OrionAutonomousBuilder::new(0.75);
        builder.verify();
        let decision = builder.propose(
            "src/engine.rs",
            "Optimise Verlet integration step",
            "// optimised code",
            0.9,
            vec![
                String::from("benchmark_results"),
                String::from("tests_green"),
            ],
        );
        assert_eq!(decision, Decision::Approved);
        assert_eq!(builder.audit_trail_len(), 1);
    }

    #[test]
    fn test_low_confidence_not_applied() {
        let mut builder = OrionAutonomousBuilder::new(0.75);
        builder.verify();
        let decision = builder.propose(
            "src/engine.rs",
            "Speculative refactor",
            "// speculative",
            0.5,
            vec![String::from("analysis")],
        );
        assert_eq!(decision, Decision::RequestMoreInfo);
    }

    #[test]
    fn test_empty_description_rejected() {
        let mut builder = OrionAutonomousBuilder::new(0.75);
        builder.verify();
        let decision = builder.propose("src/lib.rs", "", "// code", 0.9, vec![]);
        assert_eq!(decision, Decision::Rejected);
    }

    #[test]
    fn test_audit_trail_grows() {
        let mut builder = OrionAutonomousBuilder::new(0.75);
        builder.verify();
        builder.propose("a.rs", "desc", "code", 0.9, vec![String::from("info")]);
        builder.propose("b.rs", "desc", "code", 0.9, vec![String::from("info")]);
        assert_eq!(builder.audit_trail_len(), 2);
    }
}
