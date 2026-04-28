//! User authentication for the EIRA system.
//!
//! Provides deterministic credential management with an immutable login audit log.
//! Zero randomness — passwords are verified using FNV-1a hashing (fully deterministic).
//!
//! # Design constraints
//! - No randomness: FNV-1a is a pure byte-arithmetic hash with no nondeterministic inputs.
//! - Immutable log: every authentication attempt is appended to `login_log`; no deletions.
//! - Type-safe: all errors are returned as `&'static str`; no panics in normal flow.

/// Deterministic FNV-1a (64-bit) hash of `input`.
///
/// Identical input always produces identical output; different inputs produce
/// statistically distinct outputs.
///
/// # ⚠ Demonstration context only
///
/// FNV-1a is **not** a cryptographic password hash. It has no salt, no
/// computational cost, and is trivially brute-forceable. It is used here
/// solely to satisfy the system's zero-randomness constraint in a
/// demonstration environment. **Never use this in a production system.**
/// Production deployments must replace this with a proper password-hashing
/// scheme (e.g. Argon2, bcrypt, or PBKDF2).
fn fnv1a_hash(input: &str) -> u64 {
    const OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const PRIME: u64 = 1_099_511_628_211;
    input.bytes().fold(OFFSET_BASIS, |hash, byte| {
        (hash ^ (byte as u64)).wrapping_mul(PRIME)
    })
}

/// A stored credential entry.
#[derive(Debug, Clone)]
struct Credential {
    username: String,
    password_hash: u64,
}

/// A single login attempt recorded in the immutable audit log.
#[derive(Debug, Clone)]
pub struct LoginRecord {
    /// The username supplied by the caller.
    pub username: String,
    /// Contextual timestamp string provided by the caller.
    pub timestamp: String,
    /// `true` when the credentials matched a registered user.
    pub success: bool,
}

/// Deterministic user registry with an immutable login audit log.
///
/// Every registration and every authentication attempt is recorded.
/// Passwords are never retained in plain text — only their FNV-1a hashes are stored.
pub struct UserRegistry {
    credentials: Vec<Credential>,
    login_log: Vec<LoginRecord>,
}

impl UserRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            credentials: Vec::new(),
            login_log: Vec::new(),
        }
    }

    /// Register a new user.
    ///
    /// Returns `Err` if:
    /// - `username` is empty
    /// - `password` is empty
    /// - `username` is already registered
    pub fn register(&mut self, username: &str, password: &str) -> Result<(), &'static str> {
        if username.is_empty() {
            return Err("username must not be empty");
        }
        if password.is_empty() {
            return Err("password must not be empty");
        }
        if self.credentials.iter().any(|c| c.username == username) {
            return Err("username already registered");
        }
        self.credentials.push(Credential {
            username: username.to_string(),
            password_hash: fnv1a_hash(password),
        });
        Ok(())
    }

    /// Authenticate a user.
    ///
    /// Always appends a `LoginRecord` to the immutable log regardless of outcome.
    /// Returns `true` when the supplied credentials match a registered user.
    pub fn authenticate(
        &mut self,
        username: &str,
        password: &str,
        timestamp: impl Into<String>,
    ) -> bool {
        let success = self
            .credentials
            .iter()
            .any(|c| c.username == username && c.password_hash == fnv1a_hash(password));
        self.login_log.push(LoginRecord {
            username: username.to_string(),
            timestamp: timestamp.into(),
            success,
        });
        success
    }

    /// Return the full, immutable login audit log.
    pub fn login_log(&self) -> &[LoginRecord] {
        &self.login_log
    }

    /// Check whether a username has been registered.
    pub fn is_registered(&self, username: &str) -> bool {
        self.credentials.iter().any(|c| c.username == username)
    }
}

impl Default for UserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_authenticate() {
        let mut registry = UserRegistry::new();
        registry.register("alice", "secret").unwrap();
        assert!(registry.authenticate("alice", "secret", "t0"));
    }

    #[test]
    fn test_wrong_password_fails() {
        let mut registry = UserRegistry::new();
        registry.register("bob", "correct-horse").unwrap();
        assert!(!registry.authenticate("bob", "wrong", "t0"));
    }

    #[test]
    fn test_unknown_user_fails() {
        let mut registry = UserRegistry::new();
        assert!(!registry.authenticate("nobody", "pass", "t0"));
    }

    #[test]
    fn test_duplicate_username_rejected() {
        let mut registry = UserRegistry::new();
        registry.register("carol", "pass1").unwrap();
        assert!(registry.register("carol", "pass2").is_err());
    }

    #[test]
    fn test_empty_username_rejected() {
        let mut registry = UserRegistry::new();
        assert!(registry.register("", "pass").is_err());
    }

    #[test]
    fn test_empty_password_rejected() {
        let mut registry = UserRegistry::new();
        assert!(registry.register("dave", "").is_err());
    }

    #[test]
    fn test_login_log_records_all_attempts() {
        let mut registry = UserRegistry::new();
        registry.register("eve", "pass").unwrap();
        registry.authenticate("eve", "pass", "t1");
        registry.authenticate("eve", "wrong", "t2");
        registry.authenticate("nobody", "x", "t3");
        let log = registry.login_log();
        assert_eq!(log.len(), 3);
        assert!(log[0].success);
        assert!(!log[1].success);
        assert!(!log[2].success);
    }

    #[test]
    fn test_hash_is_deterministic() {
        assert_eq!(fnv1a_hash("password"), fnv1a_hash("password"));
        assert_ne!(fnv1a_hash("password"), fnv1a_hash("PASSWORD"));
    }

    #[test]
    fn test_is_registered() {
        let mut registry = UserRegistry::new();
        assert!(!registry.is_registered("frank"));
        registry.register("frank", "p").unwrap();
        assert!(registry.is_registered("frank"));
    }

    #[test]
    fn test_hash_distinct_values() {
        // Ensure different passwords always hash differently
        assert_ne!(fnv1a_hash("abc"), fnv1a_hash("abd"));
        assert_ne!(fnv1a_hash(""), fnv1a_hash(" "));
    }
}
