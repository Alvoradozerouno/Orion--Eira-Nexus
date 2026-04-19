# Orion--Eira-Nexus

> **TRUE NORTH: Deterministic, auditable, and safe autonomous systems — global leading standard.**

[![Build](https://github.com/Alvoradozerouno/Orion--Eira-Nexus/actions/workflows/ci.yml/badge.svg)](https://github.com/Alvoradozerouno/Orion--Eira-Nexus/actions)

---

## True North Manifesto

The Orion-EIRA-Nexus system exists to prove a single thesis:

> **Autonomous AI systems can be made provably safe through determinism, immutable audit trails, and unbreakable epistemic governance — with zero exceptions.**

Every decision, every state transition, every inference — logged, typed, and reproducible.  
No randomness. No shortcuts. No overrides.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ORION-EIRA-NEXUS                             │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              EIRA Policy Gate  (eira/src/policy_gate.rs)     │  │
│  │                                                              │  │
│  │   Uncertain ──────► VerifiedStable ──────► Contradiction    │  │
│  │                                                              │  │
│  │   Proposal → evaluate() → Decision                          │  │
│  │   Decision ∈ { Approved | RequestInfo | Abstain | Rejected } │  │
│  │   Confidence threshold: 0.85 (hard minimum, no bypass)      │  │
│  │   Immutable audit trail — append-only                        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │ gate approval required               │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │          Orion Autonomous Builder  (core/)                   │  │
│  │   • Codebase analysis (ModuleDescriptor → AnalysisReport)   │  │
│  │   • Proposal generation (priority-ordered, deterministic)   │  │
│  │   • EIRA gate submission loop                               │  │
│  │   • Session log (immutable, per-run)                        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌─────────────────────┐  ┌────────────────────────────────────┐  │
│  │  Physics Engine     │  │  Nexus Precausal Buffer            │  │
│  │  (physics/)         │  │  (nexus/)                          │  │
│  │  • Vector3D         │  │  • StateSnapshot ring buffer       │  │
│  │  • Velocity-Verlet  │  │  • Deterministic rule engine       │  │
│  │  • N-body gravity   │  │  • linear_trend / steady_state     │  │
│  │  • Orch-OR sim      │  │  • Inference log (immutable)       │  │
│  └─────────────────────┘  └────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │         Sovereign Industrial Kernel  (eira/src/sik.rs)      │  │
│  │   • 20 W power budget enforcement                           │  │
│  │   • Deterministic processor scheduling                      │  │
│  │   • ExecutionRecord log                                     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │         EIRA Terminal  (terminal/)  ← binary: eira-terminal │  │
│  │   • Interactive CLI / decision workflow display             │  │
│  │   • Audit trail visualisation                               │  │
│  │   • TerminalLine { [INFO] | [ OK ] | [WARN] | [ERR ] }     │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Workspace Structure

| Crate | Path | Purpose |
|---|---|---|
| `eira` | `eira/` | Policy Gate + Sovereign Industrial Kernel |
| `core` | `core/` | Orion Autonomous Builder |
| `physics` | `physics/` | Newtonian engine + Orch-OR simulation |
| `nexus` | `nexus/` | Precausal Buffer + forward-inference |
| `terminal` | `terminal/` | Interactive CLI (`eira-terminal` binary) |

---

## Decision Workflow

```
User / Orion Builder
        │
        ▼
   Proposal { id, action, reasoning, confidence, required_info }
        │
        ▼
   PolicyGate::evaluate(proposal)
        │
        ├── state == Uncertain         → Abstain   (safety abstention)
        ├── state == Contradiction     → Rejected  (gate locked)
        ├── confidence < 0.85          → RequestInfo
        ├── reasoning.is_empty()       → Rejected
        ├── any required_info empty    → RequestInfo
        └── all checks pass            → Approved  ✓
        │
        ▼
   AuditEntry appended to immutable log
        │
        ▼
   Decision returned to caller
```

---

## Safety Guarantees

| Guarantee | Implementation |
|---|---|
| **Zero randomness** | No `rand`, no Monte Carlo anywhere in the system |
| **Immutable audit trail** | `Vec<AuditEntry>` is append-only; no deletion API |
| **Unbreakable gate** | `PolicyGate::evaluate` has no bypass path |
| **Type safety** | Full Rust type coverage; `#[derive(PartialEq)]` on all decisions |
| **Deterministic repeatability** | Same inputs → identical outputs, every run |
| **Power budget** | SIK enforces 20 W ceiling across all processors |
| **Epistemic states are terminal** | Contradiction → no further approvals (no reset API) |

---

## Quick Start

### Prerequisites

- Rust toolchain ≥ 1.70 (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Build

```bash
git clone https://github.com/Alvoradozerouno/Orion--Eira-Nexus.git
cd Orion--Eira-Nexus
cargo build --workspace --release
```

### Run the EIRA Terminal

```bash
cargo run --release --bin eira-terminal
```

Expected output:

```
[INFO] ═══════════════════════════════════════════════
[INFO]  EIRA Terminal — Epistemic Integrity Gate v0.1
[INFO] ═══════════════════════════════════════════════
[INFO] Zero randomness. Immutable audit. Type-safe.
[INFO] Epistemic state → VerifiedStable
[INFO] Proposal #1: "Refactor physics::engine — extract Vector3D arithmetic"  confidence=0.92
[ OK ] #1 → APPROVED
[INFO] Proposal #2: "Enable GPU acceleration"  confidence=0.60
[WARN] #2 → REQUEST_INFO
[INFO] ── Audit Trail (2 entries) ──
...
```

### Run All Tests

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
```

---

## Module Details

### EIRA Policy Gate (`eira/src/policy_gate.rs`)

The single source of truth for all autonomous decisions.

```rust
let mut gate = PolicyGate::new();          // threshold = 0.85
gate.update_state(true);                   // → VerifiedStable
let proposal = Proposal::new(1, "Deploy model", "Benchmarks pass", 0.90, "t", vec![]);
let decision = gate.evaluate(proposal);    // → Approved
assert_eq!(gate.audit_log().len(), 1);     // immutable record
```

### Physics Engine (`physics/src/engine.rs`)

Velocity-Verlet N-body gravitational simulation.

```rust
let sun = Body::new(1, 1.989e30, Vector3D::new(0.0, 0.0, 0.0), Vector3D::ZERO);
let earth = Body::new(2, 5.972e24, Vector3D::new(1.496e11, 0.0, 0.0), Vector3D::new(0.0, 29_783.0, 0.0));
let mut engine = PhysicsEngine::new(vec![sun, earth]);
for _ in 0..365 { engine.step(86_400.0); } // 1 year
```

### Nexus Precausal Buffer (`nexus/src/precausal_buffer.rs`)

Rule-based deterministic forward-inference.

```rust
let mut buf = PrecausalBuffer::new(16);
buf.register_rule(linear_trend_rule(3));
buf.push(StateSnapshot::new(0, vec![1.0, 2.0, 3.0], "t0"));
buf.push(StateSnapshot::new(1, vec![2.0, 4.0, 6.0], "t1"));
let result = buf.infer(); // → Predicted([3.0, 6.0, 9.0])
```

---

## Crate Dependencies

```
terminal ──► eira
core     ──► eira
physics  (no workspace deps)
nexus    (no workspace deps)
eira     (no workspace deps)
```

---

## License

MIT — see [LICENSE](LICENSE).
