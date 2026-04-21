//! Physics Engine — Newtonian mechanics with Velocity-Verlet integration.
//!
//! # Design Principles
//! - **Exact arithmetic**: all quantities are `f64`; no random noise injected.
//! - **Deterministic repeatability**: given the same initial conditions and
//!   time-step the simulation produces identical results on every run.
//! - **Verlet integration**: symplectic, energy-conserving, second-order.

/// A three-dimensional vector with `f64` components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    /// The zero vector.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Euclidean magnitude.
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Squared magnitude (avoids the square-root, useful for comparisons).
    pub fn magnitude_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Return a normalised copy, or the zero vector if magnitude is zero.
    pub fn normalise(&self) -> Self {
        let m = self.magnitude();
        if m == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.x / m, self.y / m, self.z / m)
        }
    }

    pub fn add(&self, rhs: &Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    pub fn scale(&self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

impl std::ops::Add for Vector3D {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vector3D::add(&self, &rhs)
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vector3D::sub(&self, &rhs)
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        self.scale(s)
    }
}

/// A rigid body participating in the simulation.
#[derive(Debug, Clone)]
pub struct Body {
    /// Unique identifier.
    pub id: u64,
    /// Mass in kilograms.
    pub mass: f64,
    /// Position in metres.
    pub position: Vector3D,
    /// Velocity in metres per second.
    pub velocity: Vector3D,
    /// Accumulated force for the current time step (cleared after integration).
    acceleration: Vector3D,
}

impl Body {
    pub fn new(id: u64, mass: f64, position: Vector3D, velocity: Vector3D) -> Self {
        Self {
            id,
            mass,
            position,
            velocity,
            acceleration: Vector3D::ZERO,
        }
    }

    /// Kinetic energy: ½ m v².
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.magnitude_sq()
    }
}

/// Newtonian gravitational constant (m³ kg⁻¹ s⁻²).
pub const G: f64 = 6.674_30e-11;

/// Deterministic physics engine using Velocity-Verlet integration.
///
/// Each call to [`step`] advances all bodies by `dt` seconds.
/// The integration is symplectic and second-order accurate.
pub struct PhysicsEngine {
    pub bodies: Vec<Body>,
    /// Current simulation time in seconds.
    pub time: f64,
    /// Number of steps completed.
    pub steps: u64,
}

impl PhysicsEngine {
    /// Create a new engine with the given bodies.
    pub fn new(bodies: Vec<Body>) -> Self {
        Self {
            bodies,
            time: 0.0,
            steps: 0,
        }
    }

    /// Advance the simulation by one time step `dt` (seconds).
    ///
    /// Uses the Velocity-Verlet algorithm:
    /// ```text
    /// x(t+dt) = x(t) + v(t)·dt + ½·a(t)·dt²
    /// a(t+dt) = F(x(t+dt)) / m
    /// v(t+dt) = v(t) + ½·(a(t) + a(t+dt))·dt
    /// ```
    pub fn step(&mut self, dt: f64) {
        let n = self.bodies.len();

        // --- Half-step velocity update and full position update ---
        for body in &mut self.bodies {
            let half_dv = body.acceleration.scale(0.5 * dt);
            body.velocity = body.velocity + half_dv;
            body.position = body.position + body.velocity.scale(dt);
        }

        // --- Compute new gravitational accelerations ---
        let mut new_acc: Vec<Vector3D> = vec![Vector3D::ZERO; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.bodies[j].position - self.bodies[i].position;
                let dist_sq = r.magnitude_sq();
                if dist_sq == 0.0 {
                    continue; // avoid divide-by-zero
                }
                let dist = dist_sq.sqrt();
                // |F| = G m_i m_j / r²
                let f_mag = G * self.bodies[i].mass * self.bodies[j].mass / dist_sq;
                let f_dir = r.scale(1.0 / dist); // unit vector i→j
                let f = f_dir.scale(f_mag);

                new_acc[i] = new_acc[i] + f.scale(1.0 / self.bodies[i].mass);
                new_acc[j] = new_acc[j] + f.scale(-1.0 / self.bodies[j].mass);
            }
        }

        // --- Second half-step velocity update ---
        for (body, acc) in self.bodies.iter_mut().zip(new_acc.iter()) {
            let half_dv = acc.scale(0.5 * dt);
            body.velocity = body.velocity + half_dv;
            body.acceleration = *acc;
        }

        self.time += dt;
        self.steps += 1;
    }

    /// Total kinetic energy of the system.
    pub fn total_kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }

    /// Total gravitational potential energy: U = -G m_i m_j / r_{ij}.
    pub fn total_potential_energy(&self) -> f64 {
        let mut u = 0.0;
        let n = self.bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let r = (self.bodies[j].position - self.bodies[i].position).magnitude();
                if r > 0.0 {
                    u -= G * self.bodies[i].mass * self.bodies[j].mass / r;
                }
            }
        }
        u
    }

    /// Total mechanical energy (kinetic + potential).
    pub fn total_energy(&self) -> f64 {
        self.total_kinetic_energy() + self.total_potential_energy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3d_magnitude() {
        let v = Vector3D::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_vector3d_normalise() {
        let v = Vector3D::new(0.0, 3.0, 4.0);
        let n = v.normalise();
        assert!((n.magnitude() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_vector3d_cross() {
        let x = Vector3D::new(1.0, 0.0, 0.0);
        let y = Vector3D::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert!((z.z - 1.0).abs() < 1e-12);
        assert!(z.x.abs() < 1e-12);
        assert!(z.y.abs() < 1e-12);
    }

    #[test]
    fn test_free_body_moves_linearly() {
        // A single body with no other bodies — no gravitational force.
        // It should move in a straight line at constant velocity.
        let body = Body::new(
            1,
            1.0,
            Vector3D::new(0.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        );
        let mut engine = PhysicsEngine::new(vec![body]);
        engine.step(1.0);
        assert!((engine.bodies[0].position.x - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_two_body_energy_conservation() {
        // Two equal masses starting at rest separated by 1 AU.
        // Over a short simulation the total energy should be approximately constant.
        let au = 1.496e11_f64; // 1 AU in metres
        let m_sun = 1.989e30_f64;
        let b1 = Body::new(1, m_sun, Vector3D::new(-au / 2.0, 0.0, 0.0), Vector3D::ZERO);
        let b2 = Body::new(2, m_sun, Vector3D::new(au / 2.0, 0.0, 0.0), Vector3D::ZERO);
        let mut engine = PhysicsEngine::new(vec![b1, b2]);
        let e0 = engine.total_energy();
        let dt = 3600.0; // 1 hour
        for _ in 0..10 {
            engine.step(dt);
        }
        let e1 = engine.total_energy();
        // Energy should be conserved to within 0.1% over 10 hours
        let rel_err = (e1 - e0).abs() / e0.abs();
        assert!(rel_err < 0.001, "Energy drift too large: {:.2e}", rel_err);
    }

    #[test]
    fn test_deterministic_repeatability() {
        let make_engine = || {
            let b = Body::new(
                1,
                1.0e24,
                Vector3D::new(0.0, 0.0, 0.0),
                Vector3D::new(1000.0, 0.0, 0.0),
            );
            PhysicsEngine::new(vec![b])
        };
        let mut e1 = make_engine();
        let mut e2 = make_engine();
        for _ in 0..50 {
            e1.step(10.0);
            e2.step(10.0);
        }
        assert_eq!(e1.bodies[0].position.x, e2.bodies[0].position.x);
        assert_eq!(e1.bodies[0].position.y, e2.bodies[0].position.y);
    }
}
