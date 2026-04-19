//! Policy Gate – EIRA Safety Layer
//!
//! Deterministic epistemic state management for autonomous decision control.
//! Zero randomness. Immutable audit trail. Type-safe verification.
//! Confidence thresholds are hard minimums with no exceptions.

/// The epistemic state of the system – determines whether decisions can be made.
#[derive(Debug, Clone, PartialEq)]
pub enum EpistemicState {
    /// Insufficient information to make a decision. Gate will Abstain.
    Uncertain,
    /// Verified stable knowledge base. Gate may Approve valid proposals.
    VerifiedStable,
    /// Internal contradiction detected. Gate will Abstain until resolved.
    Contradiction,
}

/// A proposal submitted to the EIRA Policy Gate for evaluation.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Unique proposal identifier.
    pub id: u32,
    /// The action being proposed.
    pub action: String,
    /// Reasoning chain supporting this proposal.
    pub reasoning: String,
    /// Confidence level in the range [0.0, 1.0].
    pub confidence: f32,
    /// Timestamp of proposal creation.
    pub timestamp: String,
    /// Required information that must be present and non-empty.
    pub required_info: Vec<String>,
}

impl Proposal {
    /// Create a new proposal.
    pub fn new(
        id: u32,
        action: &str,
        reasoning: &str,
        confidence: f32,
        timestamp: &str,
        required_info: Vec<String>,
    ) -> Self {
        Proposal {
            id,
            action: action.to_string(),
            reasoning: reasoning.to_string(),
            confidence,
            timestamp: timestamp.to_string(),
            required_info,
        }
    }
}

/// The decision returned by the EIRA Policy Gate.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Proposal meets all criteria. Action may proceed.
    Approved,
    /// More information is needed before a decision can be made.
    RequestMoreInfo,
    /// Proposal is structurally invalid and must be rejected.
    Rejected,
    /// The system must not decide at this time (epistemic abstention).
    Abstain,
}

/// Immutable record of a single gate evaluation.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// ID of the evaluated proposal.
    pub proposal_id: u32,
    /// Action that was evaluated.
    pub action: String,
    /// Decision that was made.
    pub decision: Decision,
    /// Timestamp of the original proposal.
    pub timestamp: String,
}

/// The EIRA Policy Gate – unbreakable safety layer for autonomous decisions.
///
/// Every autonomous decision passes through this gate. No exceptions. No bypasses.
pub struct PolicyGate {
    /// Current epistemic state of the system.
    pub state: EpistemicState,
    /// Immutable audit trail of all decisions (append-only).
    decision_history: Vec<AuditEntry>,
    /// Hard minimum confidence threshold – no exceptions.
    pub min_confidence_threshold: f32,
}

impl PolicyGate {
    /// Create a new Policy Gate with the given confidence threshold.
    ///
    /// Starts in `Uncertain` state; call `set_state(EpistemicState::VerifiedStable)`
    /// once the knowledge base is verified.
    pub fn new(min_confidence_threshold: f32) -> Self {
        PolicyGate {
            state: EpistemicState::Uncertain,
            decision_history: Vec::new(),
            min_confidence_threshold,
        }
    }

    /// Transition the epistemic state.
    pub fn set_state(&mut self, state: EpistemicState) {
        self.state = state;
    }

    /// Evaluate a proposal and record the decision in the immutable audit trail.
    ///
    /// Rules applied in order (first match wins):
    /// 1. `Uncertain` or `Contradiction` state → `Abstain`
    /// 2. Confidence below hard threshold → `RequestMoreInfo`
    /// 3. Empty reasoning string → `Rejected`
    /// 4. Any empty entry in `required_info` → `RequestMoreInfo`
    /// 5. All checks passed → `Approved`
    pub fn evaluate(&mut self, proposal: &Proposal) -> Decision {
        let decision = self.apply_rules(proposal);
        self.decision_history.push(AuditEntry {
            proposal_id: proposal.id,
            action: proposal.action.clone(),
            decision: decision.clone(),
            timestamp: proposal.timestamp.clone(),
        });
        decision
    }

    fn apply_rules(&self, proposal: &Proposal) -> Decision {
        // Rule 1: Epistemic state gate.
        match self.state {
            EpistemicState::Uncertain | EpistemicState::Contradiction => {
                return Decision::Abstain;
            }
            EpistemicState::VerifiedStable => {}
        }

        // Rule 2: Hard confidence threshold (no exceptions).
        if proposal.confidence < self.min_confidence_threshold {
            return Decision::RequestMoreInfo;
        }

        // Rule 3: Reasoning must be present.
        if proposal.reasoning.is_empty() {
            return Decision::Rejected;
        }

        // Rule 4: All required information must be present and non-empty.
        for info in &proposal.required_info {
            if info.is_empty() {
                return Decision::RequestMoreInfo;
            }
        }

        // Rule 5: All checks passed.
        Decision::Approved
    }

    /// Return an immutable view of the full audit trail.
    pub fn history(&self) -> &[AuditEntry] {
        &self.decision_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified_gate() -> PolicyGate {
        let mut gate = PolicyGate::new(0.7);
        gate.set_state(EpistemicState::VerifiedStable);
        gate
    }

    #[test]
    fn test_approval() {
        let mut gate = verified_gate();
        let proposal = Proposal::new(
            1,
            "Deploy module",
            "Module is verified and deterministic",
            0.9,
            "2026-04-19",
            vec![String::from("spec_v1"), String::from("test_results")],
        );
        assert_eq!(gate.evaluate(&proposal), Decision::Approved);
        assert_eq!(gate.history().len(), 1);
    }

    #[test]
    fn test_low_confidence_requests_more_info() {
        let mut gate = verified_gate();
        let proposal = Proposal::new(2, "Action", "Some reasoning", 0.5, "2026-04-19", vec![]);
        assert_eq!(gate.evaluate(&proposal), Decision::RequestMoreInfo);
    }

    #[test]
    fn test_empty_reasoning_rejected() {
        let mut gate = verified_gate();
        let proposal = Proposal::new(3, "Action", "", 0.9, "2026-04-19", vec![]);
        assert_eq!(gate.evaluate(&proposal), Decision::Rejected);
    }

    #[test]
    fn test_uncertain_state_abstains() {
        let mut gate = PolicyGate::new(0.7);
        // Default state is Uncertain.
        let proposal = Proposal::new(4, "Action", "Reasoning", 0.95, "2026-04-19", vec![]);
        assert_eq!(gate.evaluate(&proposal), Decision::Abstain);
    }

    #[test]
    fn test_contradiction_state_abstains() {
        let mut gate = PolicyGate::new(0.7);
        gate.set_state(EpistemicState::Contradiction);
        let proposal = Proposal::new(5, "Action", "Reasoning", 0.95, "2026-04-19", vec![]);
        assert_eq!(gate.evaluate(&proposal), Decision::Abstain);
    }

    #[test]
    fn test_empty_required_info_requests_more() {
        let mut gate = verified_gate();
        let proposal = Proposal::new(
            6,
            "Action",
            "Reasoning",
            0.9,
            "2026-04-19",
            vec![String::from("")],
        );
        assert_eq!(gate.evaluate(&proposal), Decision::RequestMoreInfo);
    }

    #[test]
    fn test_audit_trail_accumulates() {
        let mut gate = verified_gate();
        let p1 = Proposal::new(1, "A1", "R1", 0.9, "t1", vec![]);
        let p2 = Proposal::new(2, "A2", "", 0.9, "t2", vec![]);
        gate.evaluate(&p1);
        gate.evaluate(&p2);
        let history = gate.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].decision, Decision::Approved);
        assert_eq!(history[1].decision, Decision::Rejected);
    }
}
