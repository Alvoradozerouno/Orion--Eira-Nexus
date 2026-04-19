# Orion--Eira-Nexus

> Autonomous ORION agent under strict EIRA policy gate control.

[![CI](https://github.com/Alvoradozerouno/Orion--Eira-Nexus/actions/workflows/ci.yml/badge.svg)](https://github.com/Alvoradozerouno/Orion--Eira-Nexus/actions/workflows/ci.yml)

---

## Overview

**Orion–EIRA–Nexus** is a fully deterministic, 20 W-envelope autonomous system
demonstrating how a self-directing software agent (Orion) can propose and
execute changes while remaining under the strict governance of an immutable
safety layer (EIRA).

Every action that Orion proposes must pass through the EIRA Policy Gate before
it can be committed.  The Nexus Precausal Buffer adds a deterministic temporal
lookahead that predicts outcomes before the gate makes its final decision.

The Physics crate provides a rigorous deterministic simulation engine (Verlet
integration + Penrose–Hameroff Orch-OR model) that grounds the system in
hard physical constraints.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ORION–EIRA–NEXUS SYSTEM                      │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Sovereign Industrial Kernel (SIK)                      │   │
│  │  20W profile • localhost:11434 • determinism ON         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌──────────────┐     propose     ┌──────────────────────────┐ │
│  │   ORION      │ ─────────────►  │   EIRA POLICY GATE       │ │
│  │  (core)      │ ◄─────────────  │   (eira)                 │ │
│  │              │   decision      │                          │ │
│  │  • analyze   │                 │  • EpistemicState        │ │
│  │  • propose   │                 │  • Proposal              │ │
│  │  • commit    │                 │  • Decision              │ │
│  └──────────────┘                 │  • Immutable log         │ │
│         │                         └──────────────────────────┘ │
│         │                                   │                   │
│         ▼                                   ▼                   │
│  ┌──────────────┐                 ┌──────────────────────────┐ │
│  │   NEXUS      │                 │   PHYSICS                │ │
│  │  (nexus)     │                 │   (physics)              │ │
│  │              │                 │                          │ │
│  │  Precausal   │                 │  • Vector3               │ │
│  │  Buffer –    │                 │  • PhysicsEngine         │ │
│  │  deterministic│                │  • Verlet integration    │ │
│  │  lookahead   │                 │  • Orch-OR consciousness │ │
│  └──────────────┘                 └──────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  TERMINAL (terminal)  – pretty-printed workflow UI      │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Workspace Structure

```
Orion--Eira-Nexus/
├── Cargo.toml             # Workspace root + root crate
├── src/
│   ├── lib.rs             # Re-exports all workspace crates
│   └── main.rs            # Complete working demo (~200 lines)
├── eira/                  # Crate 1 – Policy Gate & SIK
│   └── src/
│       ├── lib.rs
│       ├── policy_gate.rs # 250+ lines – Core safety logic
│       └── sik.rs         # 120+ lines – Sovereign Industrial Kernel
├── core/                  # Crate 2 – Orion Autonomous Builder
│   └── src/
│       └── lib.rs         # 250+ lines – Agent logic
├── physics/               # Crate 3 – Deterministic Engine
│   └── src/
│       ├── lib.rs
│       ├── engine.rs      # 200+ lines – Newton / Verlet
│       └── orch_or.rs     # 180+ lines – Penrose-Hameroff Orch-OR
├── nexus/                 # Crate 4 – Precausal Buffer
│   └── src/
│       ├── lib.rs
│       └── precausal_buffer.rs  # 150+ lines
└── terminal/              # Crate 5 – Terminal UI
    └── src/
        ├── lib.rs
        └── eira_terminal.rs     # 150+ lines
```

---

## Core Concepts

### 1. EIRA Policy Gate (`eira`)

The EIRA gate is the **central authority** for every proposed change.  No
action may be executed without an explicit `Decision::Approved` from the gate.

**EpistemicState machine:**

```
Uncertain  →  VerifiedStable  →  Approved
              (after enriched      (final state)
               context supplied)
```

**Policy rules (evaluated in order):**

1. Proposal confidence ≥ 0.75
2. Action description must be non-empty
3. Reasoning must be non-empty
4. If no additional context is provided → `RequestInfo`
5. All rules satisfied → `Approved`

**Example:**

```rust
use eira::policy_gate::{PolicyGate, Proposal};

let mut gate = PolicyGate::new();
let proposal = Proposal::new(1, "Add cache".into(), "Reduces load".into(), 0.95)
    .with_context(vec!["Memory: 50 MB".into()]);
let decision = gate.evaluate(&proposal);
assert!(matches!(decision, Decision::Approved(_)));
```

### 2. Sovereign Industrial Kernel (`eira::sik`)

The SIK encodes hard resource limits:

| Parameter | Default |
|-----------|---------|
| Max power | 20 W |
| Inference endpoint | `localhost:11434` |
| Max memory | 512 MB |
| Determinism mode | `true` |

```rust
use eira::SovereignIndustrialKernel;

let sik = SovereignIndustrialKernel::default();
assert!(sik.check_power(15));   // 15 W ≤ 20 W → OK
assert!(!sik.check_power(25));  // 25 W > 20 W → FAIL
```

### 3. Orion Autonomous Builder (`core`)

Orion drives the full propose → gate → commit lifecycle:

```rust
use core_crate::OrionBuilder;

let mut builder = OrionBuilder::new();
let result = builder.run_full_workflow();
assert_eq!(result.decision, WorkflowDecision::Approved);
```

The full workflow:
1. `analyze_codebase()` – returns a deterministic `CodebaseAnalysis`
2. `propose_change(&analysis)` – creates a `Proposal`
3. `submit_to_gate(&proposal)` – first evaluation (→ `RequestInfo`)
4. Enrich proposal with additional context
5. `submit_to_gate(&enriched)` – second evaluation (→ `Approved`)
6. `commit_if_approved()` – deterministic commit hash

### 4. Deterministic Physics Engine (`physics::engine`)

Implements Newton's laws with the symplectic **velocity-Verlet** integrator:

```
a(t)    = F(t) / m
x(t+dt) = x(t) + v(t)·dt + ½·a(t)·dt²
v(t+dt) = v(t) + ½·(a(t-dt) + a(t))·dt
```

```rust
use physics::engine::{PhysicsEngine, PhysicsBody, Vector3};

let mut body = PhysicsBody::new(1.0, Vector3::zero());
PhysicsEngine::apply_force(&mut body, Vector3::new(1.0, 0.0, 0.0));
PhysicsEngine::verlet_integrate(&mut body, 1.0);
assert!((body.position.x - 0.5).abs() < 1e-10);
```

### 5. Orch-OR Consciousness Model (`physics::orch_or`)

A deterministic analogue of the Penrose–Hameroff Orchestrated Objective
Reduction hypothesis.  Microtubules accumulate coherence until a gravitational
self-energy threshold triggers an objective reduction (collapse).

```rust
use physics::orch_or::OrchOR;

let mut orch = OrchOR::new(8);
for _ in 0..200 { orch.orchestrate(0.01); }
assert!(orch.collapse_count() > 0);
```

### 6. Nexus Precausal Buffer (`nexus`)

Provides a **deterministic temporal lookahead** by simulating three scenarios
for every proposal before it reaches the gate:

| Scenario | Condition | Result |
|----------|-----------|--------|
| Primary path (cache hit) | confidence ≥ 0.80 | DeterministicSuccess |
| Fallback path (cache miss) | always | DeterministicSuccess |
| Rollback path | always | DeterministicSuccess |

If any scenario predicts `DeterministicFailure`, the buffer's
`abstention_check` returns `false` and the proposal is flagged.

### 7. Terminal UI (`terminal`)

Formats and prints every workflow event:

```rust
use terminal::EiraTerminal;

let term = EiraTerminal::new();
term.show_approval_workflow(); // runs the full pipeline and prints it
```

---

## Getting Started

### Prerequisites

* Rust 1.75+ (stable)
* Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone & Build

```bash
git clone https://github.com/Alvoradozerouno/Orion--Eira-Nexus.git
cd Orion--Eira-Nexus
cargo build --release
```

### Run the Demo

```bash
cargo run --release
```

### Run Tests

```bash
cargo test --workspace
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
```

---

## Example Demo Output

```
╔═══════════════════════════════════════════════════════════════╗
║    ORION--EIRA-NEXUS AUTONOMOUS SYSTEM INITIALIZATION 2026    ║
╚═══════════════════════════════════════════════════════════════╝

[SIK] 🔧 Sovereign Industrial Kernel activated
      → 20W profile: ACTIVE
      → localhost:11434 ready
      → Resource constraints: ENFORCED

[ORION] 🤖 Autonomous builder initialized
[ORION] Analyzing hypothetical codebase...
[ORION] ✓ Found optimization opportunity

[ORION] 📋 PROPOSAL #1
        Action: "Implement result caching layer"
        Confidence: 0.95

[EIRA-GATE] 🔐 Received proposal for evaluation
[EIRA-GATE] Questions requiring answers:
            1. What is the estimated memory overhead?
            2. What is the expected cache hit rate?
            3. How is cache invalidation handled?

[EIRA-GATE] 🔍 Verifying proposal against SIK
            ✓ 20W compliance check: PASS
            ✓ Determinism check: PASS
            ✓ EIRA policy rules: PASS
            ✓ Safety constraints: PASS

[NEXUS] 🌀 Predicting outcomes with precausal buffer
        Scenario 'Primary execution path (cache hit)': DETERMINISTIC SUCCESS
        Overall safety assessment: SAFE TO PROCEED

[EIRA-GATE] 📈 Epistemic state: UNCERTAIN → VERIFIED_STABLE
[EIRA-GATE] ✅ DECISION: APPROVED

[TERMINAL] ✅ AUTONOMOUS WORKFLOW COMPLETE
[TERMINAL] Proposal: APPROVED AND IMPLEMENTED

🎉 System functioning nominally - Ready for next proposal
```

---

## The 5 Unbreakable Rules

1. **No randomness** – every decision is a pure deterministic function of its
   inputs.  Given the same sequence of calls the system always produces the
   same outputs.

2. **Gate first** – Orion may never commit a change that has not received an
   explicit `Decision::Approved` from the EIRA Policy Gate.

3. **Immutable history** – decision logs are append-only.  No entry may ever be
   modified or deleted.

4. **20 W envelope** – all components must satisfy the SIK power and memory
   constraints.

5. **Type safety** – every interface boundary is expressed through Rust's type
   system.  There are no implicit coercions or `unsafe` blocks in the public
   API.

---

## Security Considerations

* No network calls are made at runtime; the `localhost:11434` endpoint is a
  configuration value only.
* All external inputs (proposal fields) are validated by the gate before any
  action is taken.
* The `unsafe` keyword does not appear anywhere in this codebase.

---

## Contributing

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes, ensuring `cargo clippy -- -D warnings` passes.
4. Add or update unit tests for any new logic.
5. Submit a pull request describing your changes.

All contributions must respect the 5 unbreakable rules above.

---

## License

MIT – see [LICENSE](LICENSE).
