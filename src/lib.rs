//! # Orion–EIRA–Nexus
//!
//! Root library crate that re-exports the public API of every workspace member.
//!
//! | Module | Crate | Description |
//! |--------|-------|-------------|
//! | `eira` | `eira` | Policy Gate & Sovereign Industrial Kernel |
//! | `core_crate` | `core` | Orion Autonomous Builder |
//! | `physics` | `physics` | Deterministic Physics Engine & Orch-OR |
//! | `nexus` | `nexus` | Precausal Buffer |
//! | `terminal` | `terminal` | Terminal UI |

pub use core_crate as core;
pub use eira;
pub use nexus;
pub use physics;
pub use terminal;
