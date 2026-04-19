//! Core – Orion Autonomous Builder
//!
//! The main entry point for the Orion autonomous improvement system.
//! Proposes code changes deterministically and submits every proposal
//! through the EIRA Policy Gate before any action is taken.

pub mod orion_autonomous_builder;

pub use orion_autonomous_builder::OrionAutonomousBuilder;
