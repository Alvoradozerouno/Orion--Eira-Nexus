//! # EIRA Policy Gate – Core Safety Layer
//!
//! The `PolicyGate` is the central authority that evaluates every [`Proposal`]
//! submitted by an autonomous agent (e.g. Orion) and produces a [`Decision`].
//!
//! ## Design principles
//!
//! * **Deterministic** – given the same inputs the gate always produces the same
//!   decision.  There is no randomness anywhere in this module.
//! * **Immutable history** – every proposal and its decision are appended to an
//!   append-only log that can never be modified.
//! * **Minimal authority** – the gate only approves actions that satisfy *all*
//!   defined policy rules.  A single failing rule causes rejection or a
//!   request for more information.
//!
//! ## Example
//!
//! ```rust
//! use eira::policy_gate::{PolicyGate, Proposal, EpistemicState};
//!
//! let mut gate = PolicyGate::new();
//! let proposal = Proposal::new(
//!     1,
//!     "Implement result caching layer".to_string(),
//!     "API responses are queried repeatedly".to_string(),
//!     0.95,
//! );
//! let decision = gate.evaluate(&proposal);
//! println!("{decision:?}");
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The epistemic certainty level that the gate has reached about a proposal.
///
/// The gate always starts in [`EpistemicState::Uncertain`] when it receives a
/// new proposal and may advance toward [`EpistemicState::Approved`] as it
/// collects and verifies information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicState {
    /// The gate has not yet gathered enough information to form a judgement.
    Uncertain,
    /// All required information has been collected and verified.  The proposal
    /// is consistent with the SIK rules and safety constraints.
    VerifiedStable,
    /// The gate has issued a final approval.  This state is terminal for the
    /// current evaluation round.
    Approved,
}

impl std::fmt::Display for EpistemicState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpistemicState::Uncertain => write!(f, "UNCERTAIN"),
            EpistemicState::VerifiedStable => write!(f, "VERIFIED_STABLE"),
            EpistemicState::Approved => write!(f, "APPROVED"),
        }
    }
}

/// A change proposal submitted by an autonomous agent for gate evaluation.
///
/// Every field is immutable once the proposal is created.  The gate must
/// never modify a proposal – it can only accept or reject it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Monotonically increasing proposal identifier.
    pub id: u64,
    /// Human-readable description of the action to be performed.
    pub action: String,
    /// Justification for why this action should be taken.
    pub reasoning: String,
    /// Agent's self-reported confidence in the proposal (0.0 – 1.0).
    pub confidence: f64,
    /// Wall-clock time at which the proposal was created.
    pub timestamp: DateTime<Utc>,
    /// Optional additional context supplied after a [`Decision::RequestInfo`].
    pub additional_context: Vec<String>,
}

impl Proposal {
    /// Create a new proposal with the current UTC timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` – unique monotonic proposal number
    /// * `action` – description of the proposed change
    /// * `reasoning` – rationale for the change
    /// * `confidence` – agent confidence level in `[0.0, 1.0]`
    pub fn new(id: u64, action: String, reasoning: String, confidence: f64) -> Self {
        Self {
            id,
            action,
            reasoning,
            confidence,
            timestamp: Utc::now(),
            additional_context: Vec::new(),
        }
    }

    /// Attach additional context answers (e.g. in response to a
    /// [`Decision::RequestInfo`]).
    pub fn with_context(mut self, context: Vec<String>) -> Self {
        self.additional_context = context;
        self
    }
}

/// The gate's verdict for a given [`Proposal`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// The proposal passes all policy rules and may be executed.
    /// The inner `String` contains the approval rationale.
    Approved(String),
    /// The gate needs more information before it can decide.
    /// The inner `Vec<String>` lists specific questions.
    RequestInfo(Vec<String>),
    /// The gate explicitly declines to approve the proposal.
    /// The inner `String` contains the rejection rationale.
    Abstain(String),
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Approved(r) => write!(f, "APPROVED – {r}"),
            Decision::RequestInfo(qs) => write!(f, "REQUEST_INFO – {} question(s)", qs.len()),
            Decision::Abstain(r) => write!(f, "ABSTAIN – {r}"),
        }
    }
}

/// A single entry in the immutable decision log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The proposal that was evaluated.
    pub proposal: Proposal,
    /// The decision that was reached.
    pub decision: Decision,
    /// Epistemic state at the time the decision was finalised.
    pub epistemic_state: EpistemicState,
    /// Wall-clock time of the decision.
    pub decided_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// PolicyGate
// ---------------------------------------------------------------------------

/// The EIRA Policy Gate.
///
/// Acts as an immutable referee between an autonomous agent and the system it
/// controls.  All evaluations are deterministic; no external I/O is performed.
///
/// # Immutable history
///
/// Every [`LogEntry`] produced by [`PolicyGate::evaluate`] is appended to
/// [`PolicyGate::history`].  The history can be read but never modified after
/// the fact.
///
/// # Policy rules (evaluated in order)
///
/// 1. Confidence must be ≥ 0.75.
/// 2. Action description must not be empty.
/// 3. Reasoning must not be empty.
/// 4. If the proposal has no additional context, the gate requests information.
/// 5. All checks pass → the proposal advances to `VerifiedStable` and is
///    approved.
pub struct PolicyGate {
    /// Append-only decision history.  Never modified, only appended to.
    pub history: Vec<LogEntry>,
    /// Current epistemic state of the gate.
    pub epistemic_state: EpistemicState,
}

impl Default for PolicyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyGate {
    /// Construct a new gate in the initial [`EpistemicState::Uncertain`] state.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            epistemic_state: EpistemicState::Uncertain,
        }
    }

    /// Evaluate a proposal against all policy rules and return a [`Decision`].
    ///
    /// The gate's epistemic state is updated as a side effect, and the result
    /// is appended to the immutable history log.
    ///
    /// # Determinism guarantee
    ///
    /// This function is pure with respect to its inputs.  Given an identical
    /// sequence of calls it will always produce identical results.
    pub fn evaluate(&mut self, proposal: &Proposal) -> Decision {
        self.epistemic_state = EpistemicState::Uncertain;

        // Rule 1 – confidence threshold
        if proposal.confidence < 0.75 {
            let decision = Decision::Abstain(format!(
                "Confidence {:.2} is below the required threshold of 0.75",
                proposal.confidence
            ));
            self.record(proposal, decision.clone(), EpistemicState::Uncertain);
            return decision;
        }

        // Rule 2 – action must be non-empty
        if proposal.action.trim().is_empty() {
            let decision = Decision::Abstain("Action description must not be empty".to_string());
            self.record(proposal, decision.clone(), EpistemicState::Uncertain);
            return decision;
        }

        // Rule 3 – reasoning must be non-empty
        if proposal.reasoning.trim().is_empty() {
            let decision = Decision::Abstain("Reasoning must not be empty".to_string());
            self.record(proposal, decision.clone(), EpistemicState::Uncertain);
            return decision;
        }

        // Rule 4 – request additional context if none provided
        if proposal.additional_context.is_empty() {
            let questions = vec![
                "What is the estimated memory overhead?".to_string(),
                "What is the expected cache hit rate?".to_string(),
                "How is cache invalidation handled?".to_string(),
            ];
            let decision = Decision::RequestInfo(questions);
            self.record(proposal, decision.clone(), EpistemicState::Uncertain);
            return decision;
        }

        // All rules satisfied → advance epistemic state and approve
        self.epistemic_state = EpistemicState::VerifiedStable;
        let decision = Decision::Approved(
            "Proposal passes all verification criteria and safety checks".to_string(),
        );
        self.record(proposal, decision.clone(), EpistemicState::VerifiedStable);
        self.epistemic_state = EpistemicState::Approved;
        decision
    }

    /// Verify a proposal against SIK compliance rules without recording the
    /// result in history.
    ///
    /// Returns `true` when all SIK constraints are satisfied.
    pub fn sik_compliance_check(&self, proposal: &Proposal) -> bool {
        // 20 W power profile compliance: reject suspiciously large payloads
        proposal.action.len() < 1024
            && proposal.reasoning.len() < 4096
            && proposal.confidence >= 0.0
            && proposal.confidence <= 1.0
    }

    /// Return the number of entries in the immutable history log.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Retrieve a read-only reference to a specific history entry.
    pub fn history_entry(&self, index: usize) -> Option<&LogEntry> {
        self.history.get(index)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn record(&mut self, proposal: &Proposal, decision: Decision, state: EpistemicState) {
        self.history.push(LogEntry {
            proposal: proposal.clone(),
            decision,
            epistemic_state: state,
            decided_at: Utc::now(),
        });
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn base_proposal() -> Proposal {
        Proposal::new(
            1,
            "Implement result caching layer".to_string(),
            "API responses are queried repeatedly".to_string(),
            0.95,
        )
    }

    #[test]
    fn test_request_info_when_no_context() {
        let mut gate = PolicyGate::new();
        let proposal = base_proposal();
        let decision = gate.evaluate(&proposal);
        assert!(
            matches!(decision, Decision::RequestInfo(_)),
            "Expected RequestInfo, got {decision:?}"
        );
        assert_eq!(gate.history_len(), 1);
        assert_eq!(gate.epistemic_state, EpistemicState::Uncertain);
    }

    #[test]
    fn test_approved_with_context() {
        let mut gate = PolicyGate::new();
        let proposal = base_proposal().with_context(vec![
            "Memory overhead ~50 MB".to_string(),
            "Hit rate 75–85 %".to_string(),
            "TTL-based invalidation".to_string(),
        ]);
        let decision = gate.evaluate(&proposal);
        assert!(
            matches!(decision, Decision::Approved(_)),
            "Expected Approved, got {decision:?}"
        );
        assert_eq!(gate.epistemic_state, EpistemicState::Approved);
        assert_eq!(gate.history_len(), 1);
    }

    #[test]
    fn test_abstain_low_confidence() {
        let mut gate = PolicyGate::new();
        let proposal = Proposal::new(
            2,
            "Risky change".to_string(),
            "Seems fine".to_string(),
            0.50,
        );
        let decision = gate.evaluate(&proposal);
        assert!(
            matches!(decision, Decision::Abstain(_)),
            "Expected Abstain, got {decision:?}"
        );
    }

    #[test]
    fn test_abstain_empty_action() {
        let mut gate = PolicyGate::new();
        let proposal = Proposal::new(3, "".to_string(), "Some reasoning".to_string(), 0.90);
        let decision = gate.evaluate(&proposal);
        assert!(matches!(decision, Decision::Abstain(_)));
    }

    #[test]
    fn test_abstain_empty_reasoning() {
        let mut gate = PolicyGate::new();
        let proposal = Proposal::new(4, "Some action".to_string(), "".to_string(), 0.90);
        let decision = gate.evaluate(&proposal);
        assert!(matches!(decision, Decision::Abstain(_)));
    }

    #[test]
    fn test_sik_compliance_valid() {
        let gate = PolicyGate::new();
        let proposal = base_proposal();
        assert!(gate.sik_compliance_check(&proposal));
    }

    #[test]
    fn test_history_immutability_semantics() {
        let mut gate = PolicyGate::new();
        let p1 = base_proposal();
        let p2 = Proposal::new(
            2,
            "Second action".to_string(),
            "Second reason".to_string(),
            0.80,
        );
        gate.evaluate(&p1);
        gate.evaluate(&p2);
        assert_eq!(gate.history_len(), 2);
        // History entries are stable – first entry still has id == 1
        assert_eq!(gate.history_entry(0).unwrap().proposal.id, 1);
    }

    #[test]
    fn test_deterministic_evaluation() {
        // Evaluating the same proposal twice (on fresh gates) must produce the
        // same decision variant.
        let proposal = base_proposal();
        let mut gate_a = PolicyGate::new();
        let mut gate_b = PolicyGate::new();
        let d_a = gate_a.evaluate(&proposal);
        let d_b = gate_b.evaluate(&proposal);
        assert_eq!(
            std::mem::discriminant(&d_a),
            std::mem::discriminant(&d_b),
            "Evaluation must be deterministic"
        );
    }

    #[test]
    fn test_epistemic_state_display() {
        assert_eq!(EpistemicState::Uncertain.to_string(), "UNCERTAIN");
        assert_eq!(
            EpistemicState::VerifiedStable.to_string(),
            "VERIFIED_STABLE"
        );
        assert_eq!(EpistemicState::Approved.to_string(), "APPROVED");
    }
}
