//! Nexus Precausal Buffer – deterministic Time-Shift inference engine.
//!
//! The precausal buffer stores a fixed-size window of past states and
//! provides deterministic prediction of future states.  No randomness is
//! used at any point; given the same history, the prediction is identical
//! on every invocation.

use physics::Vector3D;

// ──────────────────────────────────────────────────────────────────────────────
// Generic ring-buffer
// ──────────────────────────────────────────────────────────────────────────────

/// A fixed-capacity ring buffer that discards the oldest entry when full.
pub struct NexusPrecausalBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
}

impl<T: Clone> NexusPrecausalBuffer<T> {
    /// Create a buffer with the given capacity (must be > 0).
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Buffer capacity must be greater than zero");
        NexusPrecausalBuffer {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a new state into the buffer, evicting the oldest entry if full.
    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.remove(0);
        }
        self.buffer.push(item);
    }

    /// Predict the next state using the simplest deterministic rule:
    /// return the most recent entry (last-value extrapolation).
    ///
    /// Returns `None` if the buffer is empty.
    pub fn predict_next(&self) -> Option<T> {
        self.buffer.last().cloned()
    }

    /// Return the number of states currently stored.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Return `true` when no states are stored.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Immutable view of the stored states (oldest first).
    pub fn as_slice(&self) -> &[T] {
        &self.buffer
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Time-Shift inference engine
// ──────────────────────────────────────────────────────────────────────────────

/// A deterministic time-shift inference engine.
///
/// Given a discrete state and a constant delta, it advances and predicts
/// future states by simple integer arithmetic – no floating-point, no
/// randomness.
#[derive(Debug, Clone)]
pub struct TimeShiftEngine {
    /// Current discrete state counter.
    pub current_state: u64,
    /// Number of advance steps executed.
    pub shift_steps: u64,
    /// Step delta applied on each `advance` call.
    pub delta: u64,
}

impl TimeShiftEngine {
    /// Create a new engine with the given initial state and step delta.
    pub fn new(initial_state: u64, delta: u64) -> Self {
        TimeShiftEngine {
            current_state: initial_state,
            shift_steps: 0,
            delta,
        }
    }

    /// Advance the engine by one step.
    pub fn advance(&mut self) {
        self.current_state = self.current_state.wrapping_add(self.delta);
        self.shift_steps += 1;
    }

    /// Predict the state `steps_ahead` steps in the future (pure computation,
    /// does not modify the engine).
    pub fn infer_future(&self, steps_ahead: u64) -> u64 {
        self.current_state
            .wrapping_add(steps_ahead.wrapping_mul(self.delta))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Orbital precausal buffer (Vector3D specialisation helper)
// ──────────────────────────────────────────────────────────────────────────────

/// Predict the next orbital position using linear extrapolation.
///
/// Given the two most recent positions p₀ (older) and p₁ (newer), the
/// prediction is:  p₂ = p₁ + (p₁ − p₀).
///
/// Returns `None` when fewer than two samples are available.
pub fn predict_next_orbital_position(buffer: &NexusPrecausalBuffer<Vector3D>) -> Option<Vector3D> {
    let s = buffer.as_slice();
    if s.len() < 2 {
        return None;
    }
    let p0 = s[s.len() - 2];
    let p1 = s[s.len() - 1];
    Some(p1 + (p1 - p0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_capacity_eviction() {
        let mut buf: NexusPrecausalBuffer<u32> = NexusPrecausalBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4); // evicts 1
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.as_slice(), &[2, 3, 4]);
    }

    #[test]
    fn test_predict_next_empty() {
        let buf: NexusPrecausalBuffer<u32> = NexusPrecausalBuffer::new(4);
        assert!(buf.predict_next().is_none());
    }

    #[test]
    fn test_predict_next_last_value() {
        let mut buf: NexusPrecausalBuffer<u32> = NexusPrecausalBuffer::new(4);
        buf.push(10);
        buf.push(20);
        assert_eq!(buf.predict_next(), Some(20));
    }

    #[test]
    fn test_time_shift_advance() {
        let mut engine = TimeShiftEngine::new(100, 10);
        engine.advance();
        engine.advance();
        assert_eq!(engine.current_state, 120);
        assert_eq!(engine.shift_steps, 2);
    }

    #[test]
    fn test_time_shift_infer_future() {
        let engine = TimeShiftEngine::new(0, 5);
        assert_eq!(engine.infer_future(4), 20);
    }

    #[test]
    fn test_time_shift_determinism() {
        let e1 = TimeShiftEngine::new(42, 7);
        let e2 = TimeShiftEngine::new(42, 7);
        assert_eq!(e1.infer_future(100), e2.infer_future(100));
    }

    #[test]
    fn test_orbital_prediction() {
        let mut buf: NexusPrecausalBuffer<Vector3D> = NexusPrecausalBuffer::new(4);
        buf.push(Vector3D::new(0.0, 0.0, 0.0));
        buf.push(Vector3D::new(1.0, 2.0, 3.0));
        let predicted = predict_next_orbital_position(&buf).unwrap();
        assert_eq!(predicted, Vector3D::new(2.0, 4.0, 6.0));
    }
}
