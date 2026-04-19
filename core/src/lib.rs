//! # CORE – Orion Autonomous Builder
//!
//! The `core` crate implements the autonomous agent that drives the entire
//! Orion–EIRA–Nexus system.  Orion analyses a (conceptual) codebase, formulates
//! [`Proposal`]s, submits them to the EIRA [`PolicyGate`], and executes only
//! those changes that receive an explicit approval.
//!
//! ## Design principles
//!
//! * **Gate-first** – every proposed change is evaluated by EIRA before it can
//!   be committed.  Orion has no bypass path.
//! * **Immutable log** – the builder records every proposal and its outcome in
//!   an append-only history.
//! * **Deterministic** – given the same inputs the builder always produces the
//!   same proposals and takes the same decisions.
//!
//! ## Example
//!
//! ```rust,ignore
//! use core_crate::OrionBuilder;
//!
//! let mut builder = OrionBuilder::new();
//! let commit = builder.run_full_workflow();
//! println!("{commit:?}");
//! ```

use chrono::{DateTime, Utc};
use eira::policy_gate::{Decision, PolicyGate, Proposal};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The result of a full Orion workflow cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// The proposal that was submitted to the gate.
    pub proposal_id: u64,
    /// Final gate decision.
    pub decision: WorkflowDecision,
    /// Commit hash if the change was implemented.
    pub commit_hash: Option<String>,
    /// Total duration in milliseconds (deterministic placeholder).
    pub duration_ms: u64,
    /// Timestamp at which the workflow completed.
    pub completed_at: DateTime<Utc>,
}

/// Simplified decision variant stored in [`WorkflowResult`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowDecision {
    Approved,
    RequestedMoreInfo,
    Rejected,
}

impl std::fmt::Display for WorkflowDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowDecision::Approved => write!(f, "APPROVED"),
            WorkflowDecision::RequestedMoreInfo => write!(f, "REQUESTED_MORE_INFO"),
            WorkflowDecision::Rejected => write!(f, "REJECTED"),
        }
    }
}

/// A record of a completed workflow cycle, stored in the builder's log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEntry {
    pub result: WorkflowResult,
    pub recorded_at: DateTime<Utc>,
}

/// Analysis of a hypothetical codebase section.
///
/// In a real deployment this would be populated by static-analysis tooling.
/// Here it is a pure data container.
#[derive(Debug, Clone)]
pub struct CodebaseAnalysis {
    /// Short description of the opportunity found.
    pub opportunity: String,
    /// Path of the file that would be affected.
    pub target_file: String,
    /// Estimated number of lines to be added or modified.
    pub estimated_lines: u32,
    /// Agent's confidence that the change would be beneficial.
    pub confidence: f64,
    /// Detailed reasoning.
    pub reasoning: String,
}

// ---------------------------------------------------------------------------
// OrionBuilder
// ---------------------------------------------------------------------------

/// The Orion Autonomous Builder.
///
/// Orchestrates the full proposal lifecycle:
/// `analyse → propose → gate → commit`.
pub struct OrionBuilder {
    /// Monotonic counter for proposal IDs.
    next_proposal_id: u64,
    /// The EIRA gate instance owned by this builder.
    gate: PolicyGate,
    /// Append-only workflow history.
    log: Vec<WorkflowEntry>,
}

impl Default for OrionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrionBuilder {
    /// Construct a new builder with a fresh gate and empty log.
    pub fn new() -> Self {
        Self {
            next_proposal_id: 1,
            gate: PolicyGate::new(),
            log: Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // Core pipeline steps
    // ------------------------------------------------------------------

    /// Analyse a hypothetical codebase and return an [`CodebaseAnalysis`].
    ///
    /// In production this would invoke static-analysis tools.  Here it returns
    /// a deterministic, hard-coded finding to demonstrate the pipeline.
    pub fn analyze_codebase(&self) -> CodebaseAnalysis {
        CodebaseAnalysis {
            opportunity: "Implement result caching layer".to_string(),
            target_file: "src/caching.rs".to_string(),
            estimated_lines: 45,
            confidence: 0.95,
            reasoning: "API responses are queried repeatedly with identical parameters; \
                        introducing a time-bounded cache would reduce latency and CPU load."
                .to_string(),
        }
    }

    /// Convert a [`CodebaseAnalysis`] into a gate-ready [`Proposal`].
    pub fn propose_change(&mut self, analysis: &CodebaseAnalysis) -> Proposal {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        Proposal::new(
            id,
            analysis.opportunity.clone(),
            analysis.reasoning.clone(),
            analysis.confidence,
        )
    }

    /// Submit a [`Proposal`] to the EIRA gate and return its [`Decision`].
    pub fn submit_to_gate(&mut self, proposal: &Proposal) -> Decision {
        self.gate.evaluate(proposal)
    }

    /// Simulate committing an approved change and return a deterministic hash.
    ///
    /// The hash is derived from the proposal id so that it is stable across
    /// runs.
    pub fn commit_if_approved(
        &mut self,
        proposal: &Proposal,
        decision: &Decision,
    ) -> Option<String> {
        match decision {
            Decision::Approved(_) => {
                // Deterministic "hash" derived from proposal id
                let hash = format!("{:016x}", proposal.id.wrapping_mul(0x9e37_79b9_7f4a_7c15));
                Some(hash)
            }
            _ => None,
        }
    }

    // ------------------------------------------------------------------
    // Full workflow
    // ------------------------------------------------------------------

    /// Run the complete Orion workflow cycle and return a [`WorkflowResult`].
    ///
    /// The cycle is:
    /// 1. Analyse the codebase.
    /// 2. Formulate a proposal.
    /// 3. Submit to the EIRA gate (first round – no context yet).
    /// 4. If the gate requests more information, enrich the proposal and
    ///    resubmit.
    /// 5. Simulate committing the change if approved.
    /// 6. Record the outcome in the immutable log.
    pub fn run_full_workflow(&mut self) -> WorkflowResult {
        let analysis = self.analyze_codebase();
        let proposal = self.propose_change(&analysis);

        // First submission
        let first_decision = self.submit_to_gate(&proposal);

        // If the gate requests more information, enrich and resubmit
        let (final_proposal, final_decision) = match first_decision {
            Decision::RequestInfo(_) => {
                let enriched = proposal.with_context(vec![
                    "Memory overhead: ~50 MB (0.1 % of system)".to_string(),
                    "Hit rate expectation: 75–85 %".to_string(),
                    "Invalidation: Time-based TTL (5 minutes)".to_string(),
                ]);
                let decision = self.submit_to_gate(&enriched);
                (enriched, decision)
            }
            other => (proposal, other),
        };

        let commit_hash = self.commit_if_approved(&final_proposal, &final_decision);
        let workflow_decision = match &final_decision {
            Decision::Approved(_) => WorkflowDecision::Approved,
            Decision::RequestInfo(_) => WorkflowDecision::RequestedMoreInfo,
            Decision::Abstain(_) => WorkflowDecision::Rejected,
        };

        let result = WorkflowResult {
            proposal_id: final_proposal.id,
            decision: workflow_decision,
            commit_hash,
            duration_ms: 2300, // deterministic placeholder
            completed_at: Utc::now(),
        };

        self.log.push(WorkflowEntry {
            result: result.clone(),
            recorded_at: Utc::now(),
        });

        result
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    /// Return a read-only reference to the EIRA gate.
    pub fn gate(&self) -> &PolicyGate {
        &self.gate
    }

    /// Return the number of entries in the immutable workflow log.
    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    /// Return a read-only reference to a specific log entry.
    pub fn log_entry(&self, index: usize) -> Option<&WorkflowEntry> {
        self.log.get(index)
    }

    /// Pretty-print a workflow result in the canonical terminal format.
    pub fn format_workflow_result(&self, result: &WorkflowResult) -> String {
        let commit = result.commit_hash.as_deref().unwrap_or("(none)");
        format!(
            "[ORION] 💾 Workflow complete\n        Proposal #{} → {}\n        Commit: {}\n        Duration: {} ms",
            result.proposal_id, result.decision, commit, result.duration_ms
        )
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_returns_deterministic_result() {
        let builder = OrionBuilder::new();
        let a1 = builder.analyze_codebase();
        let a2 = builder.analyze_codebase();
        assert_eq!(a1.opportunity, a2.opportunity);
        assert_eq!(a1.confidence, a2.confidence);
    }

    #[test]
    fn test_propose_increments_id() {
        let mut builder = OrionBuilder::new();
        let analysis = builder.analyze_codebase();
        let p1 = builder.propose_change(&analysis);
        let p2 = builder.propose_change(&analysis);
        assert_eq!(p1.id, 1);
        assert_eq!(p2.id, 2);
    }

    #[test]
    fn test_full_workflow_approved() {
        let mut builder = OrionBuilder::new();
        let result = builder.run_full_workflow();
        assert_eq!(result.decision, WorkflowDecision::Approved);
        assert!(result.commit_hash.is_some());
        assert_eq!(builder.log_len(), 1);
    }

    #[test]
    fn test_commit_hash_deterministic() {
        let mut b1 = OrionBuilder::new();
        let mut b2 = OrionBuilder::new();
        let r1 = b1.run_full_workflow();
        let r2 = b2.run_full_workflow();
        assert_eq!(r1.commit_hash, r2.commit_hash);
    }

    #[test]
    fn test_log_records_each_workflow() {
        let mut builder = OrionBuilder::new();
        builder.run_full_workflow();
        builder.run_full_workflow();
        assert_eq!(builder.log_len(), 2);
    }

    #[test]
    fn test_gate_history_grows_with_workflow() {
        let mut builder = OrionBuilder::new();
        builder.run_full_workflow();
        // Two gate evaluations per workflow: initial (RequestInfo) + enriched (Approved)
        assert!(builder.gate().history_len() >= 2);
    }

    #[test]
    fn test_workflow_decision_display() {
        assert_eq!(WorkflowDecision::Approved.to_string(), "APPROVED");
        assert_eq!(
            WorkflowDecision::RequestedMoreInfo.to_string(),
            "REQUESTED_MORE_INFO"
        );
        assert_eq!(WorkflowDecision::Rejected.to_string(), "REJECTED");
    }
}
