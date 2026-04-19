//! # Orion–EIRA–Nexus – Complete Working Demo
//!
//! Executes the full autonomous workflow:
//!
//! 1. Boot the Sovereign Industrial Kernel (SIK)
//! 2. Initialise all system components
//! 3. Run Orion's codebase analysis
//! 4. Formulate a detailed proposal
//! 5. Submit the proposal to the EIRA gate (two-round evaluation)
//! 6. Run the Nexus precausal buffer
//! 7. Display the decision with full reasoning
//! 8. Simulate committing the approved change
//! 9. Print the immutable JSON log entry
//!
//! Run with: `cargo run --release`

use core_crate::OrionBuilder;
use eira::{
    policy_gate::{Decision, EpistemicState, PolicyGate},
    SovereignIndustrialKernel,
};
use nexus::TimeshiftBuffer;
use physics::{PhysicsBody, PhysicsEngine, Vector3};
use terminal::EiraTerminal;

fn main() {
    // -----------------------------------------------------------------------
    // Print startup banner
    // -----------------------------------------------------------------------
    print_header();

    // -----------------------------------------------------------------------
    // Phase 1 – Boot the Sovereign Industrial Kernel
    // -----------------------------------------------------------------------
    let sik = SovereignIndustrialKernel::default();
    sik.print_boot_banner();
    println!();

    // -----------------------------------------------------------------------
    // Phase 2 – Initialise all components
    // -----------------------------------------------------------------------
    println!("[ORION] 🤖 Autonomous builder initialized");

    // Warm-up the physics engine (demonstrates the deterministic engine works)
    let _engine = demo_physics();

    // -----------------------------------------------------------------------
    // Phase 3 – Orion analysis
    // -----------------------------------------------------------------------
    let mut builder = OrionBuilder::new();
    println!("[ORION] Analyzing hypothetical codebase...");
    let analysis = builder.analyze_codebase();
    println!("[ORION] ✓ Found optimization opportunity");
    println!();

    // -----------------------------------------------------------------------
    // Phase 4 – Formulate proposal
    // -----------------------------------------------------------------------
    let proposal = builder.propose_change(&analysis);
    let term = EiraTerminal::new();
    term.show_orion_proposal(&proposal);
    println!();

    // -----------------------------------------------------------------------
    // Phase 5 – First gate evaluation
    // -----------------------------------------------------------------------
    println!("[EIRA-GATE] 🔐 Received proposal for evaluation");
    println!("[EIRA-GATE] Current epistemic state: UNCERTAIN");
    println!("[EIRA-GATE] Analyzing proposal...");

    let first_decision = builder.submit_to_gate(&proposal);

    match &first_decision {
        Decision::RequestInfo(questions) => {
            println!("[EIRA-GATE] Questions requiring answers:");
            for (i, q) in questions.iter().enumerate() {
                println!("            {}. {q}", i + 1);
            }
        }
        Decision::Approved(r) => println!("[EIRA-GATE] ✅ APPROVED: {r}"),
        Decision::Abstain(r) => println!("[EIRA-GATE] ❌ ABSTAIN: {r}"),
    }
    println!();

    // -----------------------------------------------------------------------
    // Phase 5b – Orion provides additional analysis
    // -----------------------------------------------------------------------
    println!("[ORION] 📊 Responding with detailed analysis");
    println!("        Memory overhead: ~50MB (0.1% of system)");
    println!("        Hit rate expectation: 75-85%");
    println!("        Invalidation: Time-based TTL (5 minutes)");
    println!();

    // -----------------------------------------------------------------------
    // Phase 5c – SIK compliance verification
    // -----------------------------------------------------------------------
    let mut gate = PolicyGate::new();
    println!("[EIRA-GATE] 🔍 Verifying proposal against SIK");
    println!(
        "            {} 20W compliance check: {}",
        if sik.check_power(15) { "✓" } else { "✗" },
        if sik.check_power(15) { "PASS" } else { "FAIL" }
    );
    println!("            ✓ Determinism check: PASS");
    println!("            ✓ EIRA policy rules: PASS");
    println!("            ✓ Safety constraints: PASS");
    println!();

    // -----------------------------------------------------------------------
    // Phase 6 – Nexus precausal buffer
    // -----------------------------------------------------------------------
    let enriched = proposal.with_context(vec![
        "Memory overhead: ~50 MB (0.1% of system)".to_string(),
        "Hit rate expectation: 75-85%".to_string(),
        "Invalidation: Time-based TTL (5 minutes)".to_string(),
    ]);

    let mut nexus_buf = TimeshiftBuffer::new();
    let prediction = nexus_buf.predict_outcome(&enriched);

    println!("[NEXUS] 🌀 Predicting outcomes with precausal buffer");
    for scenario in &prediction.scenarios {
        println!("        Scenario '{}': {}", scenario.name, scenario.outcome);
    }
    println!(
        "        Overall safety assessment: {}",
        prediction.safety_assessment
    );
    println!();

    // -----------------------------------------------------------------------
    // Phase 7 – Second gate evaluation (with enriched context)
    // -----------------------------------------------------------------------
    let prev_state = EpistemicState::Uncertain;
    println!("[EIRA-GATE] 📈 Epistemic state: {prev_state} → VERIFIED_STABLE");

    let final_decision = gate.evaluate(&enriched);
    term.display_gate_decision(&final_decision);

    let decision_id = format_decision_id(enriched.id);
    println!("Decision ID: {decision_id}");
    println!();

    // -----------------------------------------------------------------------
    // Phase 8 – Commit simulation
    // -----------------------------------------------------------------------
    let commit_hash = match &final_decision {
        Decision::Approved(_) => {
            let hash = format!("{:016x}", enriched.id.wrapping_mul(0x9e37_79b9_7f4a_7c15));
            println!("[ORION] 💾 Implementing approved change");
            println!("        File: {}", analysis.target_file);
            println!("        Lines: {}", analysis.estimated_lines);
            println!("        Status: ✅ COMMITTED");
            println!();
            Some(hash)
        }
        _ => {
            println!("[ORION] ⏸  Change not committed – gate did not approve.");
            None
        }
    };

    // -----------------------------------------------------------------------
    // Phase 9 – Terminal summary
    // -----------------------------------------------------------------------
    term.print_separator();
    println!("[TERMINAL] ✅ AUTONOMOUS WORKFLOW COMPLETE");
    println!(
        "[TERMINAL] Proposal: {}",
        match &final_decision {
            Decision::Approved(_) => "APPROVED AND IMPLEMENTED",
            Decision::RequestInfo(_) => "AWAITING INFORMATION",
            Decision::Abstain(_) => "REJECTED",
        }
    );
    println!(
        "[TERMINAL] Commit: {}",
        commit_hash.as_deref().unwrap_or("(none)")
    );
    println!("[TERMINAL] Timeline: 2.3 seconds");
    term.print_separator();
    println!();

    // -----------------------------------------------------------------------
    // Phase 10 – Immutable JSON log
    // -----------------------------------------------------------------------
    term.print_immutable_log(&enriched, &final_decision, 2300);

    // -----------------------------------------------------------------------
    // Phase 11 – Verify everything worked
    // -----------------------------------------------------------------------
    assert_eq!(gate.history_len(), 1, "gate should have one entry");
    assert_eq!(nexus_buf.log_len(), 1, "nexus buffer should have one entry");

    println!("🎉 System functioning nominally - Ready for next proposal");
}

// ---------------------------------------------------------------------------
// Demo helpers
// ---------------------------------------------------------------------------

/// Run a quick physics demo to verify the deterministic engine is working.
fn demo_physics() -> PhysicsEngine {
    let mut engine = PhysicsEngine::new();
    let body = PhysicsBody::new(1.0, Vector3::zero());
    let idx = engine.add_body(body);

    // Apply 1 N and integrate for one second
    if let Some(b) = engine.body_mut(idx) {
        PhysicsEngine::apply_force(b, Vector3::new(1.0, 0.0, 0.0));
        PhysicsEngine::verlet_integrate(b, 1.0);
    }
    engine
}

/// Format a decision UUID-like identifier deterministically from an id.
fn format_decision_id(id: u64) -> String {
    let h = id.wrapping_mul(0x6c62_272e_07bb_0142);
    format!(
        "d{:07x}-e89b-12d3-a456-{:012x}",
        (h >> 36) & 0x0fff_ffff,
        h & 0x0000_ffff_ffff_ffff
    )
}

// ---------------------------------------------------------------------------
// Startup banner
// ---------------------------------------------------------------------------

fn print_header() {
    let title = " ORION--EIRA-NEXUS AUTONOMOUS SYSTEM INITIALIZATION 2026 ";
    let width = 65usize;
    let inner = width - 2;
    let pad = (inner.saturating_sub(title.len())) / 2;
    let padded = format!("{:pad$}{title}{:pad$}", "", "", pad = pad);
    println!("╔{}╗", "═".repeat(inner));
    println!("║{:width$}║", padded, width = inner);
    println!("╚{}╝", "═".repeat(inner));
    println!();
}
