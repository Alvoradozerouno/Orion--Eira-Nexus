//! EIRA Terminal — Interactive CLI for the Orion-EIRA-Nexus system.
//!
//! Provides a deterministic, text-based interface for:
//! - Registering users and authenticating via the EIRA login system
//! - Submitting proposals to the EIRA Policy Gate (requires authentication)
//! - Inspecting the current epistemic state and login status
//! - Viewing the complete proposal and login audit trails
//! - Visualising the decision workflow

use eira::auth::UserRegistry;
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
    user_registry: UserRegistry,
    logged_in_user: Option<String>,
    output_buffer: Vec<TerminalLine>,
    proposal_counter: u64,
}

impl EiraTerminal {
    /// Create a new terminal session.
    pub fn new() -> Self {
        let mut t = Self {
            gate: PolicyGate::new(),
            user_registry: UserRegistry::new(),
            logged_in_user: None,
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

    // ── Authentication ────────────────────────────────────────────────────────

    /// Register a new user credential.
    ///
    /// Returns `true` on success; emits an `[ERR ]` line and returns `false` on failure.
    pub fn register_user(&mut self, username: &str, password: &str) -> bool {
        match self.user_registry.register(username, password) {
            Ok(()) => {
                self.output_buffer
                    .push(TerminalLine::ok(format!("User '{}' registered.", username)));
                true
            }
            Err(e) => {
                self.output_buffer
                    .push(TerminalLine::err(format!("Registration failed: {}", e)));
                false
            }
        }
    }

    /// Authenticate a user and open a session.
    ///
    /// Returns `true` and emits `[ OK ]` on success.
    /// Returns `false` and emits `[ERR ]` on failure; no session is opened.
    pub fn login(&mut self, username: &str, password: &str) -> bool {
        let success = self
            .user_registry
            .authenticate(username, password, "terminal");
        if success {
            self.logged_in_user = Some(username.to_string());
            self.output_buffer.push(TerminalLine::ok(format!(
                "User '{}' authenticated.",
                username
            )));
        } else {
            self.output_buffer.push(TerminalLine::err(format!(
                "Authentication failed for '{}'.",
                username
            )));
        }
        success
    }

    /// Close the current session.
    ///
    /// Emits an `[INFO]` line confirming the logout; no-op if no session is open.
    pub fn logout(&mut self) {
        if let Some(user) = self.logged_in_user.take() {
            self.output_buffer
                .push(TerminalLine::info(format!("User '{}' logged out.", user)));
        }
    }

    /// Return the username of the currently authenticated user, if any.
    pub fn current_user(&self) -> Option<&str> {
        self.logged_in_user.as_deref()
    }

    /// Return `true` when a user session is active.
    pub fn is_authenticated(&self) -> bool {
        self.logged_in_user.is_some()
    }

    // ── Epistemic state ───────────────────────────────────────────────────────

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

    // ── Proposal submission ───────────────────────────────────────────────────

    /// Submit a proposal to the EIRA Policy Gate.
    ///
    /// **Authentication is required.** If no user session is active this method
    /// emits an `[ERR ]` line and returns `Decision::Rejected` immediately,
    /// without consulting the gate.
    pub fn submit(
        &mut self,
        action: impl Into<String>,
        reasoning: impl Into<String>,
        confidence: f64,
        required_info: Vec<String>,
    ) -> Decision {
        let id = self.proposal_counter;
        self.proposal_counter += 1;

        // Authentication guard — must be logged in before submitting proposals.
        if self.logged_in_user.is_none() {
            self.output_buffer.push(TerminalLine::err(format!(
                "Proposal #{}: authentication required — login first",
                id
            )));
            return Decision::Rejected;
        }

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

    // ── Audit trail & status ──────────────────────────────────────────────────

    /// Print a formatted ASCII proposal audit trail.
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

    /// Print the login audit log.
    pub fn show_login_log(&mut self) {
        let log = self.user_registry.login_log();
        self.output_buffer.push(TerminalLine::info(format!(
            "── Login Log ({} entries) ──",
            log.len()
        )));
        for record in log {
            let status = if record.success { "SUCCESS" } else { "FAILURE" };
            self.output_buffer.push(TerminalLine::info(format!(
                "  {} | {} | ts={}",
                record.username, status, record.timestamp,
            )));
        }
    }

    /// Display the current epistemic state and authentication status.
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
        let user_label = self
            .logged_in_user
            .as_deref()
            .unwrap_or("<not authenticated>");
        self.output_buffer.push(TerminalLine::info(format!(
            "Logged-in user: {}",
            user_label
        )));
        self.output_buffer.push(TerminalLine::info(format!(
            "Login attempts: {}",
            self.user_registry.login_log().len()
        )));
    }

    // ── Buffer helpers ────────────────────────────────────────────────────────

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

    /// Helper: create a terminal with a pre-registered and logged-in test user.
    fn authenticated_terminal() -> EiraTerminal {
        let mut t = EiraTerminal::new();
        t.register_user("test_user", "test_pass");
        t.login("test_user", "test_pass");
        t
    }

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
    fn test_rejected_when_not_logged_in() {
        let mut t = EiraTerminal::new();
        let d = t.submit("Test action", "Reasoning", 0.99, vec![]);
        assert_eq!(d, Decision::Rejected);
        let joined: String = t
            .peek_output()
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("authentication required"));
    }

    #[test]
    fn test_abstain_before_epistemic_state_set() {
        let mut t = authenticated_terminal();
        let d = t.submit("Test action", "Reasoning", 0.99, vec![]);
        assert_eq!(d, Decision::Abstain);
    }

    #[test]
    fn test_approval_after_verified_stable() {
        let mut t = authenticated_terminal();
        t.set_epistemic_state(true); // → VerifiedStable
        let d = t.submit("Safe action", "Well-reasoned", 0.90, vec![]);
        assert_eq!(d, Decision::Approved);
    }

    #[test]
    fn test_show_audit_trail_output() {
        let mut t = authenticated_terminal();
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

    // ── Login-specific tests ─────────────────────────────────────────────────

    #[test]
    fn test_register_user_success() {
        let mut t = EiraTerminal::new();
        let ok = t.register_user("alice", "hunter2");
        assert!(ok);
        let joined: String = t
            .peek_output()
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("registered"));
    }

    #[test]
    fn test_register_duplicate_user_fails() {
        let mut t = EiraTerminal::new();
        t.register_user("alice", "hunter2");
        let ok = t.register_user("alice", "other");
        assert!(!ok);
    }

    #[test]
    fn test_login_success() {
        let mut t = EiraTerminal::new();
        t.register_user("bob", "pass");
        let ok = t.login("bob", "pass");
        assert!(ok);
        assert_eq!(t.current_user(), Some("bob"));
        assert!(t.is_authenticated());
    }

    #[test]
    fn test_login_wrong_password_fails() {
        let mut t = EiraTerminal::new();
        t.register_user("carol", "correct");
        let ok = t.login("carol", "wrong");
        assert!(!ok);
        assert!(!t.is_authenticated());
        assert_eq!(t.current_user(), None);
    }

    #[test]
    fn test_login_unknown_user_fails() {
        let mut t = EiraTerminal::new();
        let ok = t.login("ghost", "x");
        assert!(!ok);
        assert!(!t.is_authenticated());
    }

    #[test]
    fn test_logout_clears_session() {
        let mut t = EiraTerminal::new();
        t.register_user("dave", "pw");
        t.login("dave", "pw");
        assert!(t.is_authenticated());
        t.logout();
        assert!(!t.is_authenticated());
        assert_eq!(t.current_user(), None);
    }

    #[test]
    fn test_submit_rejected_after_logout() {
        let mut t = authenticated_terminal();
        t.set_epistemic_state(true);
        t.logout();
        let d = t.submit("Action", "Reasoning", 0.90, vec![]);
        assert_eq!(d, Decision::Rejected);
    }

    #[test]
    fn test_show_login_log() {
        let mut t = EiraTerminal::new();
        t.register_user("eve", "p");
        t.login("eve", "p");
        t.login("eve", "wrong");
        t.flush();
        t.show_login_log();
        let out = t.peek_output();
        let joined: String = out
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("Login Log"));
        assert!(joined.contains("SUCCESS"));
        assert!(joined.contains("FAILURE"));
    }

    #[test]
    fn test_show_status_includes_user_info() {
        let mut t = EiraTerminal::new();
        t.register_user("frank", "pw");
        t.login("frank", "pw");
        t.flush();
        t.show_status();
        let out = t.peek_output();
        let joined: String = out
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("frank"));
        assert!(joined.contains("Logged-in user"));
    }

    #[test]
    fn test_show_status_not_authenticated() {
        let mut t = EiraTerminal::new();
        t.flush();
        t.show_status();
        let out = t.peek_output();
        let joined: String = out
            .iter()
            .map(|l| l.render())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("not authenticated"));
    }
}
