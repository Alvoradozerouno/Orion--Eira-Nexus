//! EIRA Terminal — Interactive CLI for the Orion-EIRA-Nexus system.
//!
//! Provides a deterministic, text-based interface for:
//! - Submitting proposals to the EIRA Policy Gate
//! - Inspecting the current epistemic state
//! - Viewing the complete audit trail
//! - Visualising the decision workflow

use eira::policy_gate::{Decision, EpistemicState, PolicyGate, Proposal};

/// A single line of terminal output.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalLine {
    pub prefix: &'static str,
    pub content: String,
}

impl TerminalLine {
    pub fn info(content: impl Into<String>) -> Self {
        Self {
            prefix: "[INFO]",
            content: content.into(),
        }
    }
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            prefix: "[ OK ]",
            content: content.into(),
        }
    }
    pub fn warn(content: impl Into<String>) -> Self {
        Self {
            prefix: "[WARN]",
            content: content.into(),
        }
    }
    pub fn err(content: impl Into<String>) -> Self {
        Self {
            prefix: "[ERR ]",
            content: content.into(),
        }
    }

    pub fn render(&self) -> String {
        format!("{} {}", self.prefix, self.content)
    }
}

/// The interactive EIRA terminal session.
pub struct EiraTerminal {
    gate: PolicyGate,
    output_buffer: Vec<TerminalLine>,
    proposal_counter: u64,
}

impl EiraTerminal {
    /// Create a new terminal session.
    pub fn new() -> Self {
        let mut t = Self {
            gate: PolicyGate::new(),
            output_buffer: Vec::new(),
            proposal_counter: 1,
        };
        t.emit_banner();
        t
    }

    fn emit_banner(&mut self) {
        self.output_buffer.push(TerminalLine::info(
            "═══════════════════════════════════════════════",
        ));
        self.output_buffer.push(TerminalLine::info(
            " EIRA Terminal — Epistemic Integrity Gate v0.1 ",
        ));
        self.output_buffer.push(TerminalLine::info(
            "═══════════════════════════════════════════════",
        ));
        self.output_buffer.push(TerminalLine::info(
            "Zero randomness. Immutable audit. Type-safe.",
        ));
    }

    /// Advance the epistemic state.
    ///
    /// `consistent = true` → Uncertain → VerifiedStable
    /// `consistent = false` → VerifiedStable → Contradiction
    pub fn set_epistemic_state(&mut self, consistent: bool) {
        self.gate.update_state(consistent);
        let label = match self.gate.state() {
            EpistemicState::Uncertain => "Uncertain",
            EpistemicState::VerifiedStable => "VerifiedStable",
            EpistemicState::Contradiction => "Contradiction",
        };
        self.output_buffer
            .push(TerminalLine::info(format!("Epistemic state → {}", label)));
    }

    /// Submit a proposal and display the decision.
    pub fn submit(
        &mut self,
        action: impl Into<String>,
        reasoning: impl Into<String>,
        confidence: f64,
        required_info: Vec<String>,
    ) -> Decision {
        let id = self.proposal_counter;
        self.proposal_counter += 1;

        let action_str = action.into();
        let reasoning_str = reasoning.into();

        self.output_buffer.push(TerminalLine::info(format!(
            "Proposal #{}: \"{}\"  confidence={:.2}",
            id, action_str, confidence
        )));

        let proposal = Proposal::new(
            id,
            action_str.clone(),
            reasoning_str,
            confidence,
            "terminal",
            required_info,
        );

        let decision = self.gate.evaluate(proposal);

        let line = match &decision {
            Decision::Approved => TerminalLine::ok(format!("#{} → APPROVED", id)),
            Decision::RequestInfo => TerminalLine::warn(format!("#{} → REQUEST_INFO", id)),
            Decision::Abstain => TerminalLine::warn(format!("#{} → ABSTAIN (gate uncertain)", id)),
            Decision::Rejected => TerminalLine::err(format!("#{} → REJECTED", id)),
        };
        self.output_buffer.push(line);
        decision
    }

    /// Print a formatted ASCII audit trail.
    pub fn show_audit_trail(&mut self) {
        let log = self.gate.audit_log();
        self.output_buffer.push(TerminalLine::info(format!(
            "── Audit Trail ({} entries) ──",
            log.len()
        )));
        for entry in log {
            let decision_label = match &entry.decision {
                Decision::Approved => "APPROVED",
                Decision::RequestInfo => "REQUEST_INFO",
                Decision::Abstain => "ABSTAIN",
                Decision::Rejected => "REJECTED",
            };
            let state_label = match &entry.state_at_evaluation {
                EpistemicState::Uncertain => "Uncertain",
                EpistemicState::VerifiedStable => "VerifiedStable",
                EpistemicState::Contradiction => "Contradiction",
            };
            self.output_buffer.push(TerminalLine::info(format!(
                "  #{:<4} | {:12} | state={} | \"{}\"",
                entry.proposal.id, decision_label, state_label, entry.proposal.action,
            )));
        }
    }

    /// Display the current epistemic state.
    pub fn show_status(&mut self) {
        let state_label = match self.gate.state() {
            EpistemicState::Uncertain => "Uncertain",
            EpistemicState::VerifiedStable => "VerifiedStable ✓",
            EpistemicState::Contradiction => "Contradiction ✗",
        };
        self.output_buffer
            .push(TerminalLine::info(format!("Gate state: {}", state_label)));
        self.output_buffer.push(TerminalLine::info(format!(
            "Audit entries: {}",
            self.gate.audit_log().len()
        )));
    }

    /// Flush and return all buffered output lines, clearing the buffer.
    pub fn flush(&mut self) -> Vec<TerminalLine> {
        std::mem::take(&mut self.output_buffer)
    }

    /// Return all buffered output without clearing.
    pub fn peek_output(&self) -> &[TerminalLine] {
        &self.output_buffer
    }
}

impl Default for EiraTerminal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_banner_on_creation() {
        let t = EiraTerminal::new();
        let output = t.peek_output();
        assert!(!output.is_empty());
        let joined: String = output
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("EIRA Terminal"));
    }

    #[test]
    fn test_abstain_before_epistemic_state_set() {
        let mut t = EiraTerminal::new();
        let d = t.submit("Test action", "Reasoning", 0.99, vec![]);
        assert_eq!(d, Decision::Abstain);
    }

    #[test]
    fn test_approval_after_verified_stable() {
        let mut t = EiraTerminal::new();
        t.set_epistemic_state(true); // → VerifiedStable
        let d = t.submit("Safe action", "Well-reasoned", 0.90, vec![]);
        assert_eq!(d, Decision::Approved);
    }

    #[test]
    fn test_show_audit_trail_output() {
        let mut t = EiraTerminal::new();
        t.set_epistemic_state(true);
        t.submit("Action", "Reasoning", 0.90, vec![]);
        t.flush(); // clear buffer
        t.show_audit_trail();
        let out = t.peek_output();
        assert!(!out.is_empty());
        let joined: String = out
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("Audit Trail"));
    }

    #[test]
    fn test_show_status_output() {
        let mut t = EiraTerminal::new();
        t.flush();
        t.show_status();
        let out = t.peek_output();
        assert!(out.iter().any(|l| l.content.contains("Gate state")));
    }

    #[test]
    fn test_terminal_line_render() {
        let l = TerminalLine::ok("All good");
        assert_eq!(l.render(), "[ OK ] All good");
    }

    #[test]
    fn test_flush_clears_buffer() {
        let mut t = EiraTerminal::new();
        assert!(!t.peek_output().is_empty());
        let flushed = t.flush();
        assert!(!flushed.is_empty());
        assert!(t.peek_output().is_empty());
    }
}
