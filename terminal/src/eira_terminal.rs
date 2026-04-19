//! EIRA Terminal – Qwen-Code4EIRA integration and interactive interface.
//!
//! Connects the user (or an upstream model such as Qwen-Code) to the full
//! Orion-EIRA-Nexus stack:
//! * Forwards proposals through the EIRA Policy Gate.
//! * Renders decision workflows as human-readable output.
//! * Displays the immutable audit trail on demand.

use core_lib::OrionAutonomousBuilder;
use eira::{Decision, EpistemicState, Proposal};

/// EIRA terminal interface.
///
/// Acts as the single integration point between external agents (users,
/// language models, CI pipelines) and the autonomous Orion builder.
pub struct EiraTerminal {
    builder: OrionAutonomousBuilder,
}

impl EiraTerminal {
    /// Create a new terminal backed by an `OrionAutonomousBuilder` with the
    /// given confidence threshold.
    pub fn new(min_confidence_threshold: f32) -> Self {
        EiraTerminal {
            builder: OrionAutonomousBuilder::new(min_confidence_threshold),
        }
    }

    /// Set the epistemic state of the underlying EIRA gate.
    pub fn set_epistemic_state(&mut self, state: EpistemicState) {
        self.builder.gate_mut().set_state(state);
    }

    /// Mark the knowledge base as verified (convenience wrapper).
    pub fn verify(&mut self) {
        self.builder.verify();
    }

    /// Submit a raw `eira::Proposal` directly to the gate.
    ///
    /// Renders the decision to stdout and returns it.
    pub fn submit_proposal(&mut self, proposal: &Proposal) -> Decision {
        let decision = self.builder.submit_raw(proposal);
        self.render_decision(proposal.id, &proposal.action, &decision);
        decision
    }

    /// Submit a code change proposal through the Orion builder.
    pub fn propose_code_change(
        &mut self,
        file_path: &str,
        description: &str,
        code: &str,
        confidence: f32,
        required_info: Vec<String>,
    ) -> Decision {
        let decision =
            self.builder
                .propose(file_path, description, code, confidence, required_info);
        println!(
            "  → File: '{}' | Confidence: {:.2} | Gate: {:?}",
            file_path, confidence, decision
        );
        decision
    }

    /// Print the full audit trail to stdout.
    pub fn show_audit_trail(&self) {
        println!();
        println!("╔══════════════════════════════════════════╗");
        println!("║         EIRA IMMUTABLE AUDIT TRAIL       ║");
        println!("╠══════════════════════════════════════════╣");
        for entry in self.builder.gate().history() {
            println!(
                "║  [{}] #{:>4} {:30} {:?}",
                entry.timestamp,
                entry.proposal_id,
                truncate(&entry.action, 30),
                entry.decision,
            );
        }
        println!("╚══════════════════════════════════════════╝");
    }

    /// Return the number of proposals evaluated so far.
    pub fn audit_trail_len(&self) -> usize {
        self.builder.audit_trail_len()
    }

    fn render_decision(&self, id: u32, action: &str, decision: &Decision) {
        println!();
        println!("┌─── EIRA Policy Gate ────────────────────┐");
        println!("│  Proposal #{:<5}  {}", id, truncate(action, 32));
        println!("│  Decision : {:?}", decision);
        println!("└─────────────────────────────────────────┘");
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_approved_proposal() {
        let mut term = EiraTerminal::new(0.7);
        term.verify();
        let decision = term.propose_code_change(
            "src/main.rs",
            "Add deterministic hash function",
            "fn hash(x: u64) -> u64 { x ^ (x >> 33) }",
            0.9,
            vec![String::from("spec"), String::from("tests_green")],
        );
        assert_eq!(decision, Decision::Approved);
    }

    #[test]
    fn test_terminal_abstains_when_uncertain() {
        let mut term = EiraTerminal::new(0.7);
        // Default state is Uncertain – gate must abstain.
        let decision = term.propose_code_change(
            "src/main.rs",
            "Some change",
            "// code",
            0.95,
            vec![String::from("info")],
        );
        assert_eq!(decision, Decision::Abstain);
    }

    #[test]
    fn test_audit_trail_visible() {
        let mut term = EiraTerminal::new(0.7);
        term.verify();
        term.propose_code_change("a.rs", "desc", "code", 0.9, vec![String::from("x")]);
        // Audit trail has one entry.
        assert_eq!(term.audit_trail_len(), 1);
    }
}
