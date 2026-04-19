//! # Sovereign Industrial Kernel (SIK)
//!
//! The SIK is the lowest-level runtime contract for every component in the
//! Orion–EIRA–Nexus system.  It encodes hard resource limits (20 W power
//! profile), network endpoint configuration, and operational constraints in a
//! type-safe, purely-descriptive data structure.
//!
//! No component may exceed the limits stated in the active [`SikConfig`].
//! Enforcement is performed by the [`SovereignIndustrialKernel`] wrapper.
//!
//! ## Example
//!
//! ```rust
//! use eira::sik::SovereignIndustrialKernel;
//!
//! let sik = SovereignIndustrialKernel::default();
//! assert!(sik.is_active());
//! println!("{}", sik.status_line());
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// Power-profile budget expressed in watts.
///
/// The system is designed to run within a 20 W envelope, which is sufficient
/// for edge and embedded deployments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerProfile {
    /// Maximum sustained power draw in watts.
    pub max_watts: u32,
    /// Whether the power constraint is actively enforced.
    pub enforced: bool,
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self {
            max_watts: 20,
            enforced: true,
        }
    }
}

impl std::fmt::Display for PowerProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.enforced { "ACTIVE" } else { "INACTIVE" };
        write!(f, "{}W profile: {}", self.max_watts, status)
    }
}

/// Network endpoint configuration for the local LLM inference server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkEndpoint {
    /// Hostname (default: `localhost`).
    pub host: String,
    /// Port (default: `11434` – the Ollama default).
    pub port: u16,
}

impl Default for NetworkEndpoint {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
        }
    }
}

impl std::fmt::Display for NetworkEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

/// Memory resource constraint in megabytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryConstraint {
    /// Maximum resident set size in megabytes.
    pub max_mb: u32,
    /// Whether the constraint is actively enforced.
    pub enforced: bool,
}

impl Default for MemoryConstraint {
    fn default() -> Self {
        Self {
            max_mb: 512,
            enforced: true,
        }
    }
}

/// Complete Sovereign Industrial Kernel configuration.
///
/// All fields have sensible defaults that satisfy the 20 W design envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SikConfig {
    /// Power budget.
    pub power: PowerProfile,
    /// Inference server endpoint.
    pub endpoint: NetworkEndpoint,
    /// Memory constraint.
    pub memory: MemoryConstraint,
    /// Maximum number of concurrent proposals the gate may evaluate.
    pub max_concurrent_proposals: u32,
    /// Whether determinism mode is engaged (disables all non-deterministic
    /// code paths).
    pub determinism_mode: bool,
}

impl Default for SikConfig {
    fn default() -> Self {
        Self {
            power: PowerProfile::default(),
            endpoint: NetworkEndpoint::default(),
            memory: MemoryConstraint::default(),
            max_concurrent_proposals: 1,
            determinism_mode: true,
        }
    }
}

// ---------------------------------------------------------------------------
// SovereignIndustrialKernel
// ---------------------------------------------------------------------------

/// The runtime handle for the Sovereign Industrial Kernel.
///
/// Holds an immutable [`SikConfig`] snapshot and provides query methods used
/// by all other crates to check compliance.
#[derive(Debug, Clone)]
pub struct SovereignIndustrialKernel {
    config: SikConfig,
    active: bool,
}

impl Default for SovereignIndustrialKernel {
    fn default() -> Self {
        Self::new(SikConfig::default())
    }
}

impl SovereignIndustrialKernel {
    /// Initialise the kernel with a specific configuration.
    pub fn new(config: SikConfig) -> Self {
        Self {
            config,
            active: true,
        }
    }

    /// Return `true` if the kernel is running.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Return a read-only reference to the current configuration.
    pub fn config(&self) -> &SikConfig {
        &self.config
    }

    /// Human-readable one-line status summary suitable for log output.
    pub fn status_line(&self) -> String {
        format!(
            "SIK active | {} | endpoint: {} | memory: {}MB | determinism: {}",
            self.config.power,
            self.config.endpoint,
            self.config.memory.max_mb,
            if self.config.determinism_mode {
                "ON"
            } else {
                "OFF"
            }
        )
    }

    /// Check whether a proposed memory allocation (in MB) fits within the
    /// configured constraint.
    pub fn check_memory(&self, requested_mb: u32) -> bool {
        !self.config.memory.enforced || requested_mb <= self.config.memory.max_mb
    }

    /// Check whether the configured power profile allows a given wattage.
    pub fn check_power(&self, requested_watts: u32) -> bool {
        !self.config.power.enforced || requested_watts <= self.config.power.max_watts
    }

    /// Print a structured boot banner to stdout.
    pub fn print_boot_banner(&self) {
        println!("[SIK] 🔧 Sovereign Industrial Kernel activated");
        println!("      → {}", self.config.power);
        println!("      → {} ready", self.config.endpoint);
        println!(
            "      → Resource constraints: {}",
            if self.config.memory.enforced && self.config.power.enforced {
                "ENFORCED"
            } else {
                "RELAXED"
            }
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let sik = SovereignIndustrialKernel::default();
        assert!(sik.is_active());
        assert_eq!(sik.config().power.max_watts, 20);
        assert_eq!(sik.config().endpoint.port, 11434);
        assert_eq!(sik.config().endpoint.host, "localhost");
        assert!(sik.config().determinism_mode);
    }

    #[test]
    fn test_memory_check_within_limit() {
        let sik = SovereignIndustrialKernel::default();
        assert!(sik.check_memory(256));
    }

    #[test]
    fn test_memory_check_exceeds_limit() {
        let sik = SovereignIndustrialKernel::default();
        assert!(!sik.check_memory(1024));
    }

    #[test]
    fn test_power_check_within_limit() {
        let sik = SovereignIndustrialKernel::default();
        assert!(sik.check_power(15));
    }

    #[test]
    fn test_power_check_exceeds_limit() {
        let sik = SovereignIndustrialKernel::default();
        assert!(!sik.check_power(25));
    }

    #[test]
    fn test_status_line_contains_key_fields() {
        let sik = SovereignIndustrialKernel::default();
        let line = sik.status_line();
        assert!(line.contains("20W"));
        assert!(line.contains("localhost:11434"));
        assert!(line.contains("determinism: ON"));
    }

    #[test]
    fn test_power_profile_display() {
        let p = PowerProfile::default();
        assert_eq!(p.to_string(), "20W profile: ACTIVE");
    }

    #[test]
    fn test_endpoint_display() {
        let e = NetworkEndpoint::default();
        assert_eq!(e.to_string(), "localhost:11434");
    }

    #[test]
    fn test_custom_config() {
        let cfg = SikConfig {
            power: PowerProfile {
                max_watts: 10,
                enforced: true,
            },
            ..SikConfig::default()
        };
        let sik = SovereignIndustrialKernel::new(cfg);
        assert!(!sik.check_power(15));
        assert!(sik.check_power(5));
    }
}
