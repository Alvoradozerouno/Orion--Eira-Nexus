//! Policy Gate - Core Safety Layer
//!
//! Deterministic epistemic state management for autonomous decision control.
//! Zero randomness. Immutable audit trail. Type-safe verification.
//!
//! # Epistemic State Machine
//! ```text
//! Uncertain → VerifiedStable (on sufficient evidence)
//! VerifiedStable → Contradiction (on conflicting evidence)
//! Contradiction → terminal (no further decisions)
//! ```

/// The epistemic state of the policy gate.
/// Transitions are one-way: Uncertain → VerifiedStable → Contradiction.
#[derive(Debug, Clone, PartialEq)]
pub enum EpistemicState {
    /// Insufficient evidence to make a decision.
    Uncertain,
    /// Evidence verified and internally consistent.
    VerifiedStable,
    /// Contradictory evidence detected — gate is locked for review.
    Contradiction,
}

/// A proposal submitted to the policy gate for evaluation.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Unique proposal identifier.
    pub id: u64,
    /// Short description of the proposed action.
    pub action: String,
    /// Reasoning chain justifying the action.
    pub reasoning: String,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,
    /// ISO-8601 timestamp string.
    pub timestamp: String,
    /// Additional information items required to be non-empty.
    pub required_info: Vec<String>,
}

impl Proposal {
    /// Create a new proposal.
    pub fn new(
        id: u64,
        action: impl Into<String>,
        reasoning: impl Into<String>,
        confidence: f64,
        timestamp: impl Into<String>,
        required_info: Vec<String>,
    ) -> Self {
        Self {
            id,
            action: action.into(),
            reasoning: reasoning.into(),
            confidence,
            timestamp: timestamp.into(),
            required_info,
        }
    }
}

/// The outcome of a policy gate evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Proposal meets all safety and confidence requirements.
    Approved,
    /// More information is needed before a decision can be made.
    RequestInfo,
    /// Gate deliberately withholds decision (safety abstention).
    Abstain,
    /// Proposal is rejected outright due to safety violation.
    Rejected,
}

/// An immutable audit entry recording a single gate evaluation.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// The evaluated proposal.
    pub proposal: Proposal,
    /// The decision that was reached.
    pub decision: Decision,
    /// The epistemic state at the time of evaluation.
    pub state_at_evaluation: EpistemicState,
}

/// The EIRA Policy Gate.
///
/// Evaluates proposals deterministically based on the current epistemic state,
/// a configurable confidence threshold, and immutable safety rules.
/// Every evaluation is appended to a permanent audit log.
pub struct PolicyGate {
    state: EpistemicState,
    audit_log: Vec<AuditEntry>,
    min_confidence_threshold: f64,
}

impl PolicyGate {
    /// Create a new policy gate with the default confidence threshold (0.85).
    pub fn new() -> Self {
        Self {
            state: EpistemicState::Uncertain,
            audit_log: Vec::new(),
            min_confidence_threshold: 0.85,
        }
    }

    /// Create a policy gate with a custom confidence threshold.
    ///
    /// `threshold` must be in (0.0, 1.0]; panics otherwise.
    pub fn with_threshold(threshold: f64) -> Self {
        assert!(
            threshold > 0.0 && threshold <= 1.0,
            "threshold must be in (0.0, 1.0]"
        );
        Self {
            state: EpistemicState::Uncertain,
            audit_log: Vec::new(),
            min_confidence_threshold: threshold,
        }
    }

    /// Advance the epistemic state based on external evidence.
    ///
    /// - `consistent = true` advances Uncertain → VerifiedStable.
    /// - `consistent = false` records a Contradiction from VerifiedStable.
    pub fn update_state(&mut self, consistent: bool) {
        self.state = match (&self.state, consistent) {
            (EpistemicState::Uncertain, true) => EpistemicState::VerifiedStable,
            (EpistemicState::VerifiedStable, false) => EpistemicState::Contradiction,
            _ => self.state.clone(),
        };
    }

    /// Return the current epistemic state.
    pub fn state(&self) -> &EpistemicState {
        &self.state
    }

    /// Return the full, immutable audit log.
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Evaluate a proposal and append the result to the audit log.
    ///
    /// # Abstention rules ("When NOT to decide")
    /// - Gate is in `Uncertain` state → `Abstain`
    /// - Gate is in `Contradiction` state → `Rejected` (locked)
    /// - Confidence below threshold → `RequestInfo`
    /// - Reasoning chain is empty → `Rejected`
    /// - Any required_info item is empty → `RequestInfo`
    pub fn evaluate(&mut self, proposal: Proposal) -> Decision {
        let decision = self.compute_decision(&proposal);
        self.audit_log.push(AuditEntry {
            proposal,
            decision: decision.clone(),
            state_at_evaluation: self.state.clone(),
        });
        decision
    }

    fn compute_decision(&self, proposal: &Proposal) -> Decision {
        // State-based abstention rules
        match &self.state {
            EpistemicState::Uncertain => return Decision::Abstain,
            EpistemicState::Contradiction => return Decision::Rejected,
            EpistemicState::VerifiedStable => {}
        }

        // Hard minimum confidence threshold (default 0.85)
        if proposal.confidence < self.min_confidence_threshold {
            return Decision::RequestInfo;
        }

        // Reasoning chain must be non-empty
        if proposal.reasoning.is_empty() {
            return Decision::Rejected;
        }

        // All required information items must be provided
        for info in &proposal.required_info {
            if info.is_empty() {
                return Decision::RequestInfo;
            }
        }

        Decision::Approved
    }
}

impl Default for PolicyGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stable_gate() -> PolicyGate {
        let mut gate = PolicyGate::new();
        gate.update_state(true); // Uncertain → VerifiedStable
        gate
    }

    #[test]
    fn test_policy_gate_approval() {
        let mut gate = PolicyGate::with_threshold(0.7);
        gate.update_state(true);
        let proposal = Proposal::new(
            1,
            "Test Action",
            "This action is beneficial",
            0.8,
            "2026-04-19T18:54:38Z",
            vec!["info1".to_string()],
        );
        assert_eq!(gate.evaluate(proposal), Decision::Approved);
    }

    #[test]
    fn test_abstain_when_uncertain() {
        let mut gate = PolicyGate::new(); // stays Uncertain
        let proposal = Proposal::new(
            2,
            "Action",
            "Reasoning",
            0.99,
            "2026-04-19T00:00:00Z",
            vec![],
        );
        assert_eq!(gate.evaluate(proposal), Decision::Abstain);
    }

    #[test]
    fn test_rejected_on_contradiction() {
        let mut gate = stable_gate();
        gate.update_state(false); // VerifiedStable → Contradiction
        let proposal = Proposal::new(
            3,
            "Action",
            "Reasoning",
            0.99,
            "2026-04-19T00:00:00Z",
            vec![],
        );
        assert_eq!(gate.evaluate(proposal), Decision::Rejected);
    }

    #[test]
    fn test_request_info_low_confidence() {
        let mut gate = stable_gate();
        let proposal = Proposal::new(
            4,
            "Risky Action",
            "Some reasoning",
            0.50, // below default 0.85
            "2026-04-19T00:00:00Z",
            vec![],
        );
        assert_eq!(gate.evaluate(proposal), Decision::RequestInfo);
    }

    #[test]
    fn test_rejected_empty_reasoning() {
        let mut gate = stable_gate();
        let proposal = Proposal::new(
            5,
            "Action",
            "", // empty reasoning
            0.90,
            "2026-04-19T00:00:00Z",
            vec![],
        );
        assert_eq!(gate.evaluate(proposal), Decision::Rejected);
    }

    #[test]
    fn test_request_info_empty_required_field() {
        let mut gate = stable_gate();
        let proposal = Proposal::new(
            6,
            "Action",
            "Reasoning",
            0.90,
            "2026-04-19T00:00:00Z",
            vec!["".to_string()], // empty required info
        );
        assert_eq!(gate.evaluate(proposal), Decision::RequestInfo);
    }

    #[test]
    fn test_audit_trail_immutable_growth() {
        let mut gate = stable_gate();
        assert_eq!(gate.audit_log().len(), 0);

        let p1 = Proposal::new(7, "A", "R", 0.95, "t", vec![]);
        gate.evaluate(p1);
        assert_eq!(gate.audit_log().len(), 1);

        let p2 = Proposal::new(8, "B", "R2", 0.90, "t", vec![]);
        gate.evaluate(p2);
        assert_eq!(gate.audit_log().len(), 2);
    }

    #[test]
    fn test_default_confidence_threshold_is_085() {
        let gate = PolicyGate::new();
        assert!((gate.min_confidence_threshold - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_epistemic_state_transitions() {
        let mut gate = PolicyGate::new();
        assert_eq!(gate.state(), &EpistemicState::Uncertain);
        gate.update_state(true);
        assert_eq!(gate.state(), &EpistemicState::VerifiedStable);
        gate.update_state(false);
        assert_eq!(gate.state(), &EpistemicState::Contradiction);
        // Contradiction is terminal — further updates are no-ops
        gate.update_state(true);
        assert_eq!(gate.state(), &EpistemicState::Contradiction);
    }
}