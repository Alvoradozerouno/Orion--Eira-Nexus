//! Physics – deterministic Newtonian mechanics and Orch-OR simulation.
//!
//! Zero randomness. Every computation is exact and 100 % reproducible.

pub mod engine;
pub mod orch_or;

pub use engine::{Body, PhysicsEngine, Vector3D};
pub use orch_or::{MicrotubuleState, OrchOrSimulation, QuantumState};
