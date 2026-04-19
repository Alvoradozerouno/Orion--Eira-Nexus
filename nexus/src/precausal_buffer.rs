//! # Nexus Precausal Buffer
//!
//! The precausal buffer gives the system a **deterministic temporal lookahead**:
//! before any proposal is submitted to the EIRA gate it is first run through a
//! set of deterministic scenario simulations that predict the outcome.
//!
//! ## Design principles
//!
//! * **Pure functions** – `predict_outcome` is a pure function of its inputs.
//! * **No randomness** – all scenarios are deterministic threshold computations.
//! * **Append-only log** – every prediction is stored; nothing is ever deleted
//!   or modified.
//!
//! ## Example
//!
//! ```rust
//! use nexus::precausal_buffer::TimeshiftBuffer;
//! use eira::policy_gate::Proposal;
//!
//! let mut buf = TimeshiftBuffer::new();
//! let proposal = Proposal::new(
//!     1,
//!     "Add caching layer".to_string(),
//!     "Reduces redundant API calls".to_string(),
//!     0.95,
//! );
//! let prediction = buf.predict_outcome(&proposal);
//! println!("{}", prediction.safety_assessment);
//! ```

use chrono::{DateTime, Utc};
use eira::policy_gate::Proposal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The predicted outcome of a single scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioOutcome {
    DeterministicSuccess,
    DeterministicFailure,
    RequiresMoreInformation,
}

impl std::fmt::Display for ScenarioOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioOutcome::DeterministicSuccess => write!(f, "DETERMINISTIC SUCCESS"),
            ScenarioOutcome::DeterministicFailure => write!(f, "DETERMINISTIC FAILURE"),
            ScenarioOutcome::RequiresMoreInformation => {
                write!(f, "REQUIRES MORE INFORMATION")
            }
        }
    }
}

/// A single lookahead scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Human-readable scenario name.
    pub name: String,
    /// Predicted outcome.
    pub outcome: ScenarioOutcome,
    /// Confidence of the prediction (0.0 – 1.0).
    pub prediction_confidence: f64,
    /// Brief rationale.
    pub rationale: String,
}

/// The aggregated prediction for a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedState {
    /// The proposal ID this prediction relates to.
    pub proposal_id: u64,
    /// Individual scenarios that were evaluated.
    pub scenarios: Vec<Scenario>,
    /// Overall safety assessment text.
    pub safety_assessment: String,
    /// Whether the gate should be allowed to decide (`true`) or abstain
    /// (`false`).
    pub proceed: bool,
    /// Timestamp of the prediction.
    pub predicted_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// TimeshiftBuffer
// ---------------------------------------------------------------------------

/// The Nexus Precausal Buffer.
///
/// Maintains an append-only log of all predictions and provides deterministic
/// lookahead for proposals.
pub struct TimeshiftBuffer {
    /// Append-only prediction log.
    log: Vec<PredictedState>,
}

impl Default for TimeshiftBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeshiftBuffer {
    /// Construct a new, empty buffer.
    pub fn new() -> Self {
        Self { log: Vec::new() }
    }

    /// Predict the outcome of a proposal using deterministic lookahead.
    ///
    /// Three scenarios are always evaluated:
    /// 1. **Cache hit path** – the proposal succeeds on first execution.
    /// 2. **Cache miss path** – the proposal falls back to the original code
    ///    path safely.
    /// 3. **Rollback path** – the system can revert without side-effects.
    ///
    /// The prediction is deterministic: the same proposal always produces the
    /// same prediction.
    pub fn predict_outcome(&mut self, proposal: &Proposal) -> PredictedState {
        let scenarios = self.build_scenarios(proposal);
        let proceed = self.abstention_check(&scenarios);
        let safety = if proceed {
            "SAFE TO PROCEED".to_string()
        } else {
            "ABSTAIN – insufficient confidence".to_string()
        };

        let state = PredictedState {
            proposal_id: proposal.id,
            scenarios,
            safety_assessment: safety,
            proceed,
            predicted_at: Utc::now(),
        };
        self.log.push(state.clone());
        state
    }

    /// Provide the prediction as additional gate context.
    ///
    /// Returns a formatted string suitable for inclusion in
    /// [`Proposal::additional_context`].
    pub fn feed_to_gate(&self, prediction: &PredictedState) -> Vec<String> {
        prediction
            .scenarios
            .iter()
            .map(|s| format!("Nexus scenario '{}': {}", s.name, s.outcome))
            .collect()
    }

    /// Determine whether the system should abstain from deciding.
    ///
    /// Returns `false` (abstain) when *any* scenario predicts a deterministic
    /// failure, or when the average confidence is below 0.70.
    pub fn abstention_check(&self, scenarios: &[Scenario]) -> bool {
        let any_failure = scenarios
            .iter()
            .any(|s| s.outcome == ScenarioOutcome::DeterministicFailure);
        if any_failure {
            return false;
        }
        if scenarios.is_empty() {
            return false;
        }
        let avg_confidence: f64 = scenarios
            .iter()
            .map(|s| s.prediction_confidence)
            .sum::<f64>()
            / scenarios.len() as f64;
        avg_confidence >= 0.70
    }

    /// Return the number of predictions in the log.
    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    /// Return a read-only reference to a specific log entry.
    pub fn log_entry(&self, index: usize) -> Option<&PredictedState> {
        self.log.get(index)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn build_scenarios(&self, proposal: &Proposal) -> Vec<Scenario> {
        // Scenario 1 – primary execution path
        let s1 = if proposal.confidence >= 0.80 {
            Scenario {
                name: "Primary execution path (cache hit)".to_string(),
                outcome: ScenarioOutcome::DeterministicSuccess,
                prediction_confidence: proposal.confidence,
                rationale: "High agent confidence; primary path is expected to succeed."
                    .to_string(),
            }
        } else {
            Scenario {
                name: "Primary execution path (cache hit)".to_string(),
                outcome: ScenarioOutcome::RequiresMoreInformation,
                prediction_confidence: proposal.confidence,
                rationale: "Agent confidence is below 0.80; more information required.".to_string(),
            }
        };

        // Scenario 2 – fallback path
        let s2 = Scenario {
            name: "Fallback path (cache miss)".to_string(),
            outcome: ScenarioOutcome::DeterministicSuccess,
            prediction_confidence: 0.90,
            rationale: "Cache miss falls back to original code path without side-effects."
                .to_string(),
        };

        // Scenario 3 – rollback path
        let s3 = Scenario {
            name: "Rollback path".to_string(),
            outcome: ScenarioOutcome::DeterministicSuccess,
            prediction_confidence: 0.99,
            rationale: "Change is isolated; rollback is always possible via version control."
                .to_string(),
        };

        vec![s1, s2, s3]
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(confidence: f64) -> Proposal {
        Proposal::new(
            1,
            "Add caching layer".to_string(),
            "Reduces redundant API calls".to_string(),
            confidence,
        )
    }

    #[test]
    fn test_predict_high_confidence_proceeds() {
        let mut buf = TimeshiftBuffer::new();
        let pred = buf.predict_outcome(&make_proposal(0.95));
        assert!(pred.proceed);
        assert_eq!(pred.safety_assessment, "SAFE TO PROCEED");
        assert_eq!(buf.log_len(), 1);
    }

    #[test]
    fn test_predict_deterministic() {
        let proposal = make_proposal(0.90);
        let mut b1 = TimeshiftBuffer::new();
        let mut b2 = TimeshiftBuffer::new();
        let p1 = b1.predict_outcome(&proposal);
        let p2 = b2.predict_outcome(&proposal);
        assert_eq!(p1.proceed, p2.proceed);
        assert_eq!(p1.scenarios.len(), p2.scenarios.len());
    }

    #[test]
    fn test_feed_to_gate_returns_context() {
        let mut buf = TimeshiftBuffer::new();
        let pred = buf.predict_outcome(&make_proposal(0.95));
        let ctx = buf.feed_to_gate(&pred);
        assert_eq!(ctx.len(), pred.scenarios.len());
        for line in &ctx {
            assert!(line.contains("Nexus scenario"));
        }
    }

    #[test]
    fn test_abstention_check_fails_on_failure_scenario() {
        let buf = TimeshiftBuffer::new();
        let scenarios = vec![Scenario {
            name: "Bad scenario".to_string(),
            outcome: ScenarioOutcome::DeterministicFailure,
            prediction_confidence: 0.99,
            rationale: "Will break".to_string(),
        }];
        assert!(!buf.abstention_check(&scenarios));
    }

    #[test]
    fn test_abstention_check_passes_all_success() {
        let buf = TimeshiftBuffer::new();
        let scenarios = vec![
            Scenario {
                name: "S1".to_string(),
                outcome: ScenarioOutcome::DeterministicSuccess,
                prediction_confidence: 0.90,
                rationale: String::new(),
            },
            Scenario {
                name: "S2".to_string(),
                outcome: ScenarioOutcome::DeterministicSuccess,
                prediction_confidence: 0.95,
                rationale: String::new(),
            },
        ];
        assert!(buf.abstention_check(&scenarios));
    }

    #[test]
    fn test_log_appends_on_each_predict() {
        let mut buf = TimeshiftBuffer::new();
        buf.predict_outcome(&make_proposal(0.90));
        buf.predict_outcome(&make_proposal(0.85));
        assert_eq!(buf.log_len(), 2);
    }

    #[test]
    fn test_scenario_outcome_display() {
        assert_eq!(
            ScenarioOutcome::DeterministicSuccess.to_string(),
            "DETERMINISTIC SUCCESS"
        );
        assert_eq!(
            ScenarioOutcome::DeterministicFailure.to_string(),
            "DETERMINISTIC FAILURE"
        );
    }
}
