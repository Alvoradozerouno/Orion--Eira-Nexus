//! Physics crate — deterministic Newtonian mechanics and Orch-OR simulation.

pub mod engine;
pub mod orch_or;

pub use engine::{PhysicsEngine, Vector3D, Body};
pub use orch_or::{OrchOrSimulator, MicrotubuleState, QuantumCollapseEvent};
