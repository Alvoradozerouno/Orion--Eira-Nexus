//! Physics crate — deterministic Newtonian mechanics and Orch-OR simulation.

pub mod engine;
pub mod orch_or;

pub use engine::{Body, PhysicsEngine, Vector3D};
pub use orch_or::{MicrotubuleState, OrchOrSimulator, QuantumCollapseEvent};
