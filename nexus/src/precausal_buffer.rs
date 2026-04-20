//! Nexus Precausal Buffer — Deterministic Forward Inference.
//!
//! The Precausal Buffer maintains a rolling window of system `StateSnapshot`s
//! and performs deterministic rule-based forward-inference to predict the most
//! probable next state — without any stochastic methods.
//!
//! # Time-Shift Inference
//! Given the last `N` snapshots the buffer projects forward by applying
//! each registered `InferenceRule` in priority order.  The first rule whose
//! precondition matches determines the predicted next state.

use std::collections::VecDeque;

/// A timestamped snapshot of an arbitrary system state.
#[derive(Debug, Clone, PartialEq)]
pub struct StateSnapshot {
    /// Monotonically increasing logical clock (integer ticks, not wall-time).
    pub tick: u64,
    /// Numeric state vector — interpretation is domain-specific.
    pub values: Vec<f64>,
    /// Free-form label for debugging / audit purposes.
    pub label: String,
}

impl StateSnapshot {
    pub fn new(tick: u64, values: Vec<f64>, label: impl Into<String>) -> Self {
        Self { tick, values, label: label.into() }
    }
}

/// Outcome of a forward-inference pass.
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceResult {
    /// A prediction was produced.
    Predicted(StateSnapshot),
    /// The buffer does not yet contain enough history.
    InsufficientHistory,
    /// No inference rule matched the current history.
    NoRuleMatched,
}

/// Precondition function type for inference rules.
pub type PreconditionFn = Box<dyn Fn(&[StateSnapshot]) -> bool + Send + Sync>;

/// Projection function type for inference rules.
pub type ProjectionFn = Box<dyn Fn(&[StateSnapshot]) -> StateSnapshot + Send + Sync>;

/// A single deterministic inference rule.
///
/// `precondition` returns `true` when the rule is applicable to the given
/// history slice.  `apply` produces the predicted next snapshot.
pub struct InferenceRule {
    /// Human-readable name.
    pub name: String,
    /// Minimum history length required.
    pub min_history: usize,
    /// Precondition function.
    pub precondition: PreconditionFn,
    /// Projection function — must be deterministic (no randomness).
    pub apply: ProjectionFn,
}

impl InferenceRule {
    pub fn new(
        name: impl Into<String>,
        min_history: usize,
        precondition: impl Fn(&[StateSnapshot]) -> bool + Send + Sync + 'static,
        apply: impl Fn(&[StateSnapshot]) -> StateSnapshot + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            min_history,
            precondition: Box::new(precondition),
            apply: Box::new(apply),
        }
    }
}

/// Nexus Precausal Buffer.
///
/// Stores up to `capacity` snapshots and runs registered inference rules in
/// priority order (rules are evaluated in the order they were registered).
/// Uses a `VecDeque` ring buffer for O(1) oldest-entry eviction.
pub struct PrecausalBuffer {
    capacity: usize,
    history: VecDeque<StateSnapshot>,
    rules: Vec<InferenceRule>,
    inference_log: Vec<(u64, String, InferenceResult)>,
}

impl PrecausalBuffer {
    /// Create a new buffer with the given history capacity.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        Self {
            capacity,
            history: VecDeque::with_capacity(capacity),
            rules: Vec::new(),
            inference_log: Vec::new(),
        }
    }

    /// Append a snapshot to the buffer, evicting the oldest if at capacity.
    /// Eviction is O(1) via `VecDeque::pop_front`.
    pub fn push(&mut self, snapshot: StateSnapshot) {
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(snapshot);
    }

    /// Register an inference rule.
    pub fn register_rule(&mut self, rule: InferenceRule) {
        self.rules.push(rule);
    }

    /// Run forward-inference over the current history.
    ///
    /// Returns the first matching rule's prediction (priority order).
    pub fn infer(&mut self) -> InferenceResult {
        // Collect into a contiguous slice for rule closures.
        let history_vec: Vec<StateSnapshot> = self.history.iter().cloned().collect();
        let result = self.compute_inference(&history_vec);
        let tick = history_vec.last().map(|s| s.tick).unwrap_or(0);
        let rule_name = match &result {
            InferenceResult::Predicted(_) => "matched".to_string(),
            InferenceResult::InsufficientHistory => "insufficient_history".to_string(),
            InferenceResult::NoRuleMatched => "no_rule".to_string(),
        };
        self.inference_log.push((tick, rule_name, result.clone()));
        result
    }

    fn compute_inference(&self, history: &[StateSnapshot]) -> InferenceResult {
        if history.is_empty() {
            return InferenceResult::InsufficientHistory;
        }
        for rule in &self.rules {
            if history.len() < rule.min_history {
                continue;
            }
            if (rule.precondition)(history) {
                let prediction = (rule.apply)(history);
                return InferenceResult::Predicted(prediction);
            }
        }
        if self.rules.iter().any(|r| history.len() < r.min_history) {
            InferenceResult::InsufficientHistory
        } else {
            InferenceResult::NoRuleMatched
        }
    }

    /// Return the full inference log.
    pub fn inference_log(&self) -> &[(u64, String, InferenceResult)] {
        &self.inference_log
    }

    /// Return the current history as a slice.
    ///
    /// Note: allocates a temporary `Vec` to produce a contiguous view.
    pub fn history(&self) -> Vec<StateSnapshot> {
        self.history.iter().cloned().collect()
    }
}

// ── Built-in rules ──────────────────────────────────────────────────────────

/// Create the standard linear-trend rule.
///
/// Predicts the next snapshot by computing the average first-order difference
/// across all values and projecting forward by that delta.
pub fn linear_trend_rule(dim: usize) -> InferenceRule {
    InferenceRule::new(
        "linear_trend",
        2,
        move |history| history.last().map(|s| s.values.len() == dim).unwrap_or(false),
        move |history| {
            let n = history.len();
            let last = &history[n - 1];
            let prev = &history[n - 2];
            let next_values: Vec<f64> = (0..dim)
                .map(|i| last.values[i] + (last.values[i] - prev.values[i]))
                .collect();
            StateSnapshot::new(last.tick + 1, next_values, "linear_trend_prediction")
        },
    )
}

/// Create the steady-state rule (all deltas < epsilon → predict same values).
pub fn steady_state_rule(dim: usize, epsilon: f64) -> InferenceRule {
    InferenceRule::new(
        "steady_state",
        2,
        move |history| {
            let n = history.len();
            if history[n - 1].values.len() != dim {
                return false;
            }
            let last = &history[n - 1];
            let prev = &history[n - 2];
            (0..dim).all(|i| (last.values[i] - prev.values[i]).abs() < epsilon)
        },
        move |history| {
            let last = &history[history.len() - 1];
            StateSnapshot::new(
                last.tick + 1,
                last.values.clone(),
                "steady_state_prediction",
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_history_empty_buffer() {
        let mut buf = PrecausalBuffer::new(10);
        buf.register_rule(linear_trend_rule(1));
        assert_eq!(buf.infer(), InferenceResult::InsufficientHistory);
    }

    #[test]
    fn test_insufficient_history_one_snapshot() {
        let mut buf = PrecausalBuffer::new(10);
        buf.register_rule(linear_trend_rule(1));
        buf.push(StateSnapshot::new(0, vec![1.0], "s0"));
        assert_eq!(buf.infer(), InferenceResult::InsufficientHistory);
    }

    #[test]
    fn test_linear_trend_prediction() {
        let mut buf = PrecausalBuffer::new(10);
        buf.register_rule(linear_trend_rule(1));
        buf.push(StateSnapshot::new(0, vec![0.0], "s0"));
        buf.push(StateSnapshot::new(1, vec![1.0], "s1"));
        match buf.infer() {
            InferenceResult::Predicted(s) => {
                assert!((s.values[0] - 2.0).abs() < 1e-12);
                assert_eq!(s.tick, 2);
            }
            other => panic!("Expected Predicted, got {:?}", other),
        }
    }

    #[test]
    fn test_steady_state_prediction() {
        let mut buf = PrecausalBuffer::new(10);
        buf.register_rule(steady_state_rule(2, 1e-9));
        buf.push(StateSnapshot::new(0, vec![5.0, 3.0], "s0"));
        buf.push(StateSnapshot::new(1, vec![5.0, 3.0], "s1")); // no change
        match buf.infer() {
            InferenceResult::Predicted(s) => {
                assert!((s.values[0] - 5.0).abs() < 1e-12);
                assert!((s.values[1] - 3.0).abs() < 1e-12);
            }
            other => panic!("Expected Predicted, got {:?}", other),
        }
    }

    #[test]
    fn test_buffer_capacity_eviction() {
        let mut buf = PrecausalBuffer::new(3);
        for i in 0..5_u64 {
            buf.push(StateSnapshot::new(i, vec![i as f64], "s"));
        }
        // Only last 3 snapshots should be retained
        assert_eq!(buf.history().len(), 3);
        assert_eq!(buf.history()[0].tick, 2);
        assert_eq!(buf.history()[2].tick, 4);
    }

    #[test]
    fn test_inference_log_grows() {
        let mut buf = PrecausalBuffer::new(10);
        buf.register_rule(linear_trend_rule(1));
        buf.infer(); // empty → InsufficientHistory
        buf.push(StateSnapshot::new(0, vec![1.0], "s0"));
        buf.infer();
        assert_eq!(buf.inference_log().len(), 2);
    }

    #[test]
    fn test_deterministic_repeated_inference() {
        let make_buf = || {
            let mut buf = PrecausalBuffer::new(10);
            buf.register_rule(linear_trend_rule(2));
            buf.push(StateSnapshot::new(0, vec![1.0, 2.0], "s0"));
            buf.push(StateSnapshot::new(1, vec![3.0, 5.0], "s1"));
            buf
        };
        let mut b1 = make_buf();
        let mut b2 = make_buf();
        let r1 = b1.infer();
        let r2 = b2.infer();
        assert_eq!(r1, r2);
    }
}
