//! Nexus – Precausal Buffer and Time-Shift inference engine.
//!
//! Provides deterministic prediction and precausal state management.
//! No stochastic processes, no Monte Carlo, no randomness.

pub mod precausal_buffer;

pub use precausal_buffer::{NexusPrecausalBuffer, TimeShiftEngine};
