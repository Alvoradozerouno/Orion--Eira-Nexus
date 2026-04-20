//! Nexus crate — Precausal Buffer for deterministic forward-inference.

pub mod precausal_buffer;

pub use precausal_buffer::{PrecausalBuffer, StateSnapshot, InferenceResult};
