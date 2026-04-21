//! EIRA - Epistemic Integrity Reasoning Architecture
//! Policy Gate - Safety Layer for Autonomous Systems
//!
//! The core safety mechanism that controls all autonomous decisions
//! through deterministic epistemic state management and verification.

pub mod policy_gate;
pub mod sik;

pub use policy_gate::{Decision, EpistemicState, PolicyGate, Proposal};
pub use sik::SovereignIndustrialKernel;
