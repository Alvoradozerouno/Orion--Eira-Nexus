//! EIRA – Policy Gate & Sovereign Industrial Kernel
//!
//! EIRA is the immutable safety layer that every autonomous action must pass
//! through before it can be executed.  No change reaches the system without
//! receiving an explicit [`Decision::Approved`] from the gate.

pub mod policy_gate;
pub mod sik;

pub use policy_gate::{Decision, EpistemicState, PolicyGate, Proposal};
pub use sik::{SikConfig, SovereignIndustrialKernel};
