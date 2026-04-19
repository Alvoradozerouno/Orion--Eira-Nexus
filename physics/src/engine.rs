//! # Deterministic Physics Engine
//!
//! Provides a minimal, **deterministic** physics simulation suitable for
//! orbital-mechanics calculations and trajectory prediction inside the Nexus
//! precausal buffer.
//!
//! ## Design principles
//!
//! * **No randomness** – every calculation is a pure function of its inputs.
//! * **Fixed-precision arithmetic** – all floating-point operations follow
//!   standard IEEE 754 double precision with no platform-specific extensions.
//! * **Verlet integration** – the velocity-Verlet integrator is used because it
//!   is symplectic (it conserves energy over long simulations far better than
//!   explicit Euler).
//!
//! ## Example
//!
//! ```rust
//! use physics::engine::{PhysicsEngine, PhysicsBody, Vector3};
//!
//! let mut engine = PhysicsEngine::new();
//! let mut body = PhysicsBody::new(1.0, Vector3::new(0.0, 0.0, 0.0));
//! PhysicsEngine::apply_force(&mut body, Vector3::new(1.0, 0.0, 0.0));
//! PhysicsEngine::verlet_integrate(&mut body, 1.0);
//! assert!(body.position.x > 0.0);
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Vector3
// ---------------------------------------------------------------------------

/// A three-dimensional vector with `f64` components.
///
/// All operations are implemented as pure functions to guarantee determinism.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Construct a new vector.
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// The zero vector.
    #[inline]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Scalar multiplication.
    #[inline]
    pub fn scale(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    /// Dot product.
    #[inline]
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Euclidean magnitude.
    #[inline]
    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Unit vector (normalised).  Returns the zero vector if magnitude is zero.
    pub fn normalise(self) -> Self {
        let m = self.magnitude();
        if m == 0.0 {
            Self::zero()
        } else {
            self.scale(1.0 / m)
        }
    }

    /// Cross product.
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

impl std::ops::Add for Vector3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::AddAssign for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Sub for Vector3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::SubAssign for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::Neg for Vector3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.6}, {:.6}, {:.6})", self.x, self.y, self.z)
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::zero()
    }
}

// ---------------------------------------------------------------------------
// PhysicsBody
// ---------------------------------------------------------------------------

/// A rigid body in the physics simulation.
///
/// Position and velocity are maintained using the velocity-Verlet scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    /// Current position (m).
    pub position: Vector3,
    /// Current velocity (m/s).
    pub velocity: Vector3,
    /// Mass (kg) – must be positive.
    pub mass: f64,
    /// Accumulated force (N) for the current time step.
    /// Reset to zero after each integration step.
    pub force: Vector3,
    /// Acceleration computed in the previous integration step.
    prev_acceleration: Vector3,
}

impl PhysicsBody {
    /// Create a new body at the given position with zero velocity.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `mass` is not strictly positive.
    pub fn new(mass: f64, position: Vector3) -> Self {
        debug_assert!(mass > 0.0, "mass must be positive");
        Self {
            position,
            velocity: Vector3::zero(),
            mass,
            force: Vector3::zero(),
            prev_acceleration: Vector3::zero(),
        }
    }

    /// Kinetic energy of the body: ½mv².
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.dot(self.velocity)
    }

    /// Linear momentum: mv.
    pub fn momentum(&self) -> Vector3 {
        self.velocity.scale(self.mass)
    }
}

// ---------------------------------------------------------------------------
// PhysicsEngine
// ---------------------------------------------------------------------------

/// The deterministic physics engine.
///
/// Manages a collection of bodies and provides integration + diagnostic
/// utilities.
pub struct PhysicsEngine {
    /// All bodies currently in the simulation.
    bodies: Vec<PhysicsBody>,
    /// Simulation time elapsed (s).
    pub time: f64,
    /// Gravitational constant (m³ kg⁻¹ s⁻²).
    pub gravitational_constant: f64,
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsEngine {
    /// Construct a new engine with no bodies and standard gravitational
    /// constant (6.674 × 10⁻¹¹ m³ kg⁻¹ s⁻²).
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            time: 0.0,
            gravitational_constant: 6.674e-11,
        }
    }

    /// Add a body to the simulation and return its index.
    pub fn add_body(&mut self, body: PhysicsBody) -> usize {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    /// Return a read-only reference to a body by index.
    pub fn body(&self, index: usize) -> Option<&PhysicsBody> {
        self.bodies.get(index)
    }

    /// Return a mutable reference to a body by index.
    pub fn body_mut(&mut self, index: usize) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(index)
    }

    /// Apply an instantaneous force to a body.
    ///
    /// Implements Newton's second law: F = ma, so the accumulated force is
    /// stored and will be converted to acceleration during integration.
    pub fn apply_force(body: &mut PhysicsBody, force: Vector3) {
        body.force += force;
    }

    /// Advance the body's state by time step `dt` using the **velocity-Verlet**
    /// integrator.
    ///
    /// The velocity-Verlet scheme:
    /// ```text
    /// a(t)    = F(t) / m
    /// x(t+dt) = x(t) + v(t)·dt + ½·a(t)·dt²
    /// a(t+dt) = F(t+dt) / m   (computed in the next call)
    /// v(t+dt) = v(t) + ½·(a(t-dt) + a(t))·dt
    /// ```
    ///
    /// On the first call `prev_acceleration` is zero, so the position update
    /// correctly uses the force accumulated via [`PhysicsEngine::apply_force`].
    /// Subsequent calls use the previous acceleration for the velocity average.
    pub fn verlet_integrate(body: &mut PhysicsBody, dt: f64) {
        // Current acceleration derived from the accumulated force
        let a_current = body.force.scale(1.0 / body.mass);
        // Update position using current acceleration
        let half_dt2 = 0.5 * dt * dt;
        body.position = body.position + body.velocity.scale(dt) + a_current.scale(half_dt2);
        // Update velocity with average of previous and current acceleration
        body.velocity += (body.prev_acceleration + a_current).scale(0.5 * dt);
        // Prepare for next step
        body.prev_acceleration = a_current;
        body.force = Vector3::zero(); // reset accumulated force
    }

    /// Compute the gravitational force that body `j` exerts on body `i`.
    ///
    /// Uses Newton's law of gravitation:
    /// ```text
    /// F = G · m_i · m_j / r²  (in the direction from i to j)
    /// ```
    pub fn gravitational_force(&self, attractor: &PhysicsBody, subject: &PhysicsBody) -> Vector3 {
        let delta = attractor.position - subject.position;
        let r = delta.magnitude();
        if r == 0.0 {
            return Vector3::zero();
        }
        let magnitude = self.gravitational_constant * attractor.mass * subject.mass / (r * r);
        delta.normalise().scale(magnitude)
    }

    /// Advance the entire simulation by one time step `dt`.
    ///
    /// Applies mutual gravitational forces between all pairs of bodies, then
    /// integrates each body.
    pub fn step(&mut self, dt: f64) {
        // Compute forces (read-only pass)
        let n = self.bodies.len();
        let mut forces = vec![Vector3::zero(); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let f = {
                    let bi = &self.bodies[i];
                    let bj = &self.bodies[j];
                    self.gravitational_force(bi, bj)
                };
                forces[i] += f;
                forces[j] -= f; // Newton's third law
            }
        }
        // Apply forces and integrate
        for (body, force) in self.bodies.iter_mut().zip(forces.iter()) {
            PhysicsEngine::apply_force(body, *force);
            PhysicsEngine::verlet_integrate(body, dt);
        }
        self.time += dt;
    }

    /// Check conservation of total linear momentum.
    ///
    /// Returns the total momentum vector; in a closed system this should be
    /// constant between steps.
    pub fn total_momentum(&self) -> Vector3 {
        self.bodies
            .iter()
            .fold(Vector3::zero(), |acc, b| acc + b.momentum())
    }

    /// Check conservation of total kinetic energy.
    pub fn total_kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }

    /// Verify that the simulation conserves momentum to within a tolerance.
    ///
    /// Returns `true` when the change in total momentum magnitude is below
    /// `tolerance`.
    pub fn conservation_check(&self, baseline_momentum: Vector3, tolerance: f64) -> bool {
        let current = self.total_momentum();
        let delta = (current - baseline_momentum).magnitude();
        delta < tolerance
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    #[test]
    fn test_vector_add() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);
        let c = a + b;
        assert!((c.x - 5.0).abs() < EPS);
        assert!((c.y - 7.0).abs() < EPS);
        assert!((c.z - 9.0).abs() < EPS);
    }

    #[test]
    fn test_vector_dot() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(1.0, 0.0, 0.0);
        assert!((a.dot(b) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_vector_magnitude() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < EPS);
    }

    #[test]
    fn test_vector_normalise() {
        let v = Vector3::new(0.0, 5.0, 0.0);
        let n = v.normalise();
        assert!((n.magnitude() - 1.0).abs() < EPS);
    }

    #[test]
    fn test_verlet_integrates_constant_force() {
        let mut body = PhysicsBody::new(1.0, Vector3::zero());
        // Apply 1 N in x-direction
        PhysicsEngine::apply_force(&mut body, Vector3::new(1.0, 0.0, 0.0));
        PhysicsEngine::verlet_integrate(&mut body, 1.0);
        // x(t=1) should equal ½·a·t² = 0.5 m  (starting from rest)
        assert!(
            (body.position.x - 0.5).abs() < EPS,
            "pos.x = {}",
            body.position.x
        );
    }

    #[test]
    fn test_verlet_deterministic() {
        let run = |mut body: PhysicsBody| {
            PhysicsEngine::apply_force(&mut body, Vector3::new(2.0, 0.0, 0.0));
            PhysicsEngine::verlet_integrate(&mut body, 0.5);
            body.position
        };
        let p1 = run(PhysicsBody::new(2.0, Vector3::zero()));
        let p2 = run(PhysicsBody::new(2.0, Vector3::zero()));
        assert_eq!(p1, p2, "verlet_integrate must be deterministic");
    }

    #[test]
    fn test_kinetic_energy() {
        let body = PhysicsBody {
            position: Vector3::zero(),
            velocity: Vector3::new(2.0, 0.0, 0.0),
            mass: 3.0,
            force: Vector3::zero(),
            prev_acceleration: Vector3::zero(),
        };
        // KE = ½ · 3 · 4 = 6
        assert!((body.kinetic_energy() - 6.0).abs() < EPS);
    }

    #[test]
    fn test_gravitational_force_direction() {
        let engine = PhysicsEngine::new();
        let attractor = PhysicsBody::new(1e12, Vector3::new(10.0, 0.0, 0.0));
        let subject = PhysicsBody::new(1.0, Vector3::zero());
        let f = engine.gravitational_force(&attractor, &subject);
        // Force should point in the +x direction
        assert!(f.x > 0.0);
        assert!(f.y.abs() < EPS);
        assert!(f.z.abs() < EPS);
    }

    #[test]
    fn test_cross_product() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 1.0, 0.0);
        let c = a.cross(b);
        assert!((c.z - 1.0).abs() < EPS);
        assert!(c.x.abs() < EPS);
        assert!(c.y.abs() < EPS);
    }
}
