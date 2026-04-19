# Orion–EIRA–Nexus

> **TRUE NORTH: The world's first fully deterministic, safety-gated autonomous system.**

---

## True North Manifesto

Every autonomous system that acts in the world carries a moral obligation: it must
*know what it knows*, *know what it doesn't know*, and *never act when it should not*.

**Orion–EIRA–Nexus** was built on three unbreakable pillars:

1. **Epistemic Integrity** – The system tracks its own certainty.  
   If it is Uncertain or in Contradiction, it does *not* act.  
   Abstention is wisdom, not failure.

2. **Zero Randomness** – Every computation is 100 % reproducible.  
   Given the same input, the output is identical on every machine, every run, forever.  
   No Monte Carlo. No stochastic processes. No probability distributions.

3. **Immutable Audit Trail** – Every decision is recorded and cannot be altered.  
   Full proof chain from proposal to outcome.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Orion–EIRA–Nexus                            │
│                                                                 │
│  ┌────────────┐   propose    ┌──────────────────────────────┐   │
│  │            │ ──────────▶  │   EIRA Policy Gate           │   │
│  │   Orion    │              │                              │   │
│  │ Autonomous │  ◀────────── │  EpistemicState              │   │
│  │  Builder   │   Decision   │   Uncertain ──▶ VerifiedStable│  │
│  │  (core)    │              │   Contradiction (must Abstain)│  │
│  └────────────┘              │                              │   │
│        │                     │  Confidence threshold (hard) │   │
│        │ approved only       │  Reasoning required          │   │
│        ▼                     │  Immutable audit trail       │   │
│  ┌──────────┐                └──────────────────────────────┘   │
│  │  Apply   │                                                    │
│  │  Change  │                                                    │
│  └──────────┘                                                    │
│                                                                 │
│  ┌──────────────┐  ┌────────────────────┐  ┌─────────────────┐  │
│  │   physics    │  │      nexus         │  │    terminal     │  │
│  │              │  │                   │  │                 │  │
│  │ Newtonian    │  │  PrecausalBuffer   │  │  EiraTerminal   │  │
│  │ Vector3D     │  │  TimeShiftEngine   │  │  (Qwen bridge)  │  │
│  │ Verlet       │  │  Orbital predict   │  │  Audit display  │  │
│  │ Orch-OR      │  │                   │  │                 │  │
│  └──────────────┘  └────────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Crate Map

| Crate | Purpose |
|-------|---------|
| `eira` | **Safety layer.** Policy Gate, Sovereign Industrial Kernel |
| `core` | **Orion builder.** Proposes and submits changes under EIRA control |
| `physics` | **Deterministic physics.** Newtonian mechanics, Verlet integration, Orch-OR |
| `nexus` | **Precausal buffer.** Time-shift inference, orbital position prediction |
| `terminal` | **Interface.** Qwen-Code4EIRA bridge, audit trail display |

---

## Decision Workflow

```
User / Qwen-Code4EIRA
        │
        │  propose_code_change("src/engine.rs",
        │                      "Optimise Verlet step",
        │                      confidence=0.92)
        ▼
  EiraTerminal::propose_code_change()
        │
        ▼
  OrionAutonomousBuilder::propose()
        │
        │  builds Proposal { id, action, reasoning, confidence, required_info }
        ▼
  PolicyGate::evaluate()
        │
        ├─ state == Uncertain      ──▶  Abstain
        ├─ state == Contradiction  ──▶  Abstain
        ├─ confidence < threshold  ──▶  RequestMoreInfo
        ├─ reasoning.is_empty()    ──▶  Rejected
        ├─ required_info empty     ──▶  RequestMoreInfo
        └─ all checks passed       ──▶  Approved ──▶ change applied
        │
        ▼
  AuditEntry appended to immutable history
```

### Example output

```
┌─── EIRA Policy Gate ────────────────────┐
│  Proposal #1    Optimise Verlet step    │
│  Decision : Approved                    │
└─────────────────────────────────────────┘
ORION [APPROVED] Applying change to 'src/engine.rs': Optimise Verlet step

╔══════════════════════════════════════════╗
║         EIRA IMMUTABLE AUDIT TRAIL       ║
╠══════════════════════════════════════════╣
║  [2026-04-19] #   1 src/engine.rs         Approved
╚══════════════════════════════════════════╝
```

---

## Safety Guarantees

| Rule | Guarantee |
|------|-----------|
| Epistemic gate | Gate **always** checks state first; Uncertain/Contradiction → Abstain |
| Confidence threshold | Hard minimum; no exceptions, no overrides |
| Reasoning required | Empty reasoning → Rejected |
| Immutable history | Audit entries are append-only; no deletion, no mutation |
| Zero randomness | Every module tested for deterministic reproducibility |
| No bypasses | There is no API path that skips the Policy Gate |

---

## Quick Start

```bash
# Build everything
cargo build --workspace --release

# Run all tests (31 tests across 5 crates)
cargo test --workspace

# Lint (zero warnings enforced)
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all
```

### Minimal code example

```rust
use core::OrionAutonomousBuilder;
use eira::Decision;

fn main() {
    let mut builder = OrionAutonomousBuilder::new(0.75);
    builder.verify(); // knowledge base validated

    let decision = builder.propose(
        "src/engine.rs",
        "Optimise Verlet integration step",
        "// optimised code here",
        0.92,
        vec!["benchmark_results".into(), "tests_green".into()],
    );

    match decision {
        Decision::Approved      => println!("Change applied."),
        Decision::RequestMoreInfo => println!("Need more data."),
        Decision::Rejected      => println!("Proposal rejected."),
        Decision::Abstain       => println!("System abstained."),
    }
}
```

---

## Module Details

### EIRA Policy Gate (`eira/src/policy_gate.rs`)

- `EpistemicState`: `Uncertain` | `VerifiedStable` | `Contradiction`
- `Decision`: `Approved` | `RequestMoreInfo` | `Rejected` | `Abstain`
- Rules applied in strict order; first match wins
- History is `Vec<AuditEntry>` – push-only, no removal

### Sovereign Industrial Kernel (`eira/src/sik.rs`)

- `SovereignIndustrialKernel` (alias for `IndustrialKernel`)
- Deterministic processor registration and execution
- Local 20 W execution profile (no network, no GPU requirement)

### Physics Engine (`physics/src/engine.rs`)

- `Vector3D` with `Add`, `Sub`, `Neg`, `AddAssign` operator traits
- Velocity-Verlet integration (energy-conserving)
- `PhysicsEngine::step()` is 100 % deterministic

### Orch-OR Simulation (`physics/src/orch_or.rs`)

- Penrose-Hameroff model: `Superposition → Collapsing → Collapsed`
- Collapse triggered when |energy_difference| ≥ reduction_threshold
- No Monte Carlo – deterministic threshold comparison only

### Nexus Precausal Buffer (`nexus/src/precausal_buffer.rs`)

- `NexusPrecausalBuffer<T>`: fixed-capacity ring buffer
- `TimeShiftEngine`: integer arithmetic future-state inference
- `predict_next_orbital_position`: linear extrapolation (no randomness)

### EIRA Terminal (`terminal/src/eira_terminal.rs`)

- `EiraTerminal`: single integration point for external agents
- `propose_code_change()` / `submit_proposal()` / `show_audit_trail()`
- Qwen-Code4EIRA bridge (language-model integration point)

---

*Orion–EIRA–Nexus – Global Leading Autonomous System*  
*License: see [LICENSE](LICENSE)*
