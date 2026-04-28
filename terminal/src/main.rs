//! EIRA Terminal — binary entry point.
//!
//! Launches an interactive session demonstrating the EIRA Policy Gate
//! decision workflow with user authentication and audit trail visualisation.

use terminal::EiraTerminal;

fn main() {
    let mut term = EiraTerminal::new();

    // Print initial banner
    for line in term.flush() {
        println!("{}", line.render());
    }

    // --- Demo workflow ---

    // Step 1: register a user and log in
    // NOTE: these are demonstration-only credentials — do not use in production.
    term.register_user("demo_user", "demo_password");
    term.login("demo_user", "demo_password");
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 2: advance epistemic state to VerifiedStable
    term.set_epistemic_state(true);
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 3: submit a high-confidence proposal
    term.submit(
        "Refactor physics::engine — extract Vector3D arithmetic",
        "The Vector3D helper functions are duplicated in three places. \
         Extracting them into a single impl block reduces maintenance burden \
         with zero functional change.",
        0.92,
        vec!["code_review_complete".to_string()],
    );
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 4: submit a low-confidence proposal → RequestInfo
    term.submit(
        "Enable GPU acceleration",
        "Potential performance improvement.",
        0.60,
        vec![],
    );
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 5: show full proposal audit trail
    term.show_audit_trail();
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 6: show login audit log
    term.show_login_log();
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 7: status summary (includes logged-in user)
    term.show_status();
    for line in term.flush() {
        println!("{}", line.render());
    }

    // Step 8: log out and verify proposals are blocked
    term.logout();
    term.submit(
        "Unauthenticated proposal",
        "This should be rejected.",
        0.95,
        vec![],
    );
    for line in term.flush() {
        println!("{}", line.render());
    }
}
