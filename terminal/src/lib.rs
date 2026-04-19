//! Terminal – Qwen-Code4EIRA integration point and user interface.
//!
//! Provides a text-based interface to the Orion-EIRA-Nexus system:
//! * Submit proposals through the EIRA Policy Gate.
//! * Display decision workflows.
//! * Show immutable audit trails.

pub mod eira_terminal;

pub use eira_terminal::EiraTerminal;
