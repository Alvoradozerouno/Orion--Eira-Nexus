//! Deterministic Newtonian physics engine with Verlet integration.
//!
//! Rules:
//! * No randomness – every operation is 100 % reproducible given the same input.
//! * Velocity-Verlet integration for accurate orbital simulation.
//! * All state is explicit (no hidden global state).

/// A 3-dimensional vector used for positions, velocities and forces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    /// The zero vector.
    pub const ZERO: Vector3D = Vector3D {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Construct a new vector.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3D { x, y, z }
    }

    /// Scalar multiplication.
    pub fn scale(self, s: f64) -> Vector3D {
        Vector3D::new(self.x * s, self.y * s, self.z * s)
    }

    /// Squared Euclidean magnitude (no square root – exact).
    pub fn magnitude_sq(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Euclidean magnitude.
    pub fn magnitude(self) -> f64 {
        self.magnitude_sq().sqrt()
    }

    /// Dot product.
    pub fn dot(self, other: Vector3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl std::ops::Add for Vector3D {
    type Output = Vector3D;
    fn add(self, other: Vector3D) -> Vector3D {
        Vector3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Vector3D;
    fn sub(self, other: Vector3D) -> Vector3D {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Vector3D;
    fn neg(self) -> Vector3D {
        Vector3D::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::AddAssign for Vector3D {
    fn add_assign(&mut self, other: Vector3D) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

/// A point-mass body in the simulation.
#[derive(Debug, Clone)]
pub struct Body {
    /// Current position (m).
    pub position: Vector3D,
    /// Current velocity (m/s).
    pub velocity: Vector3D,
    /// Mass (kg).
    pub mass: f64,
    /// Acceleration computed during the previous step (m/s²).
    pub acceleration: Vector3D,
}

impl Body {
    /// Create a body at rest.
    pub fn new(position: Vector3D, mass: f64) -> Self {
        Body {
            position,
            velocity: Vector3D::ZERO,
            mass,
            acceleration: Vector3D::ZERO,
        }
    }
}

/// Deterministic Newtonian physics engine using velocity-Verlet integration.
///
/// The engine holds a list of bodies and advances the simulation one step at
/// a time. Given identical initial conditions, results are identical on every
/// run (no stochastic terms).
pub struct PhysicsEngine {
    /// All bodies in the simulation.
    pub bodies: Vec<Body>,
    /// Gravitational constant G (m³ kg⁻¹ s⁻²).
    pub gravitational_constant: f64,
    /// Integration time step (s).
    pub time_step: f64,
    /// Total elapsed simulation time (s).
    pub elapsed: f64,
}

impl PhysicsEngine {
    /// Create a new engine.
    pub fn new(gravitational_constant: f64, time_step: f64) -> Self {
        PhysicsEngine {
            bodies: Vec::new(),
            gravitational_constant,
            time_step,
            elapsed: 0.0,
        }
    }

    /// Add a body to the simulation.
    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
    }

    /// Advance the simulation by one time step using velocity-Verlet integration.
    ///
    /// Algorithm:
    /// 1. Update position: x(t+dt) = x(t) + v(t)·dt + ½·a(t)·dt²
    /// 2. Compute new accelerations a(t+dt) from gravitational forces.
    /// 3. Update velocity: v(t+dt) = v(t) + ½·(a(t) + a(t+dt))·dt
    pub fn step(&mut self) {
        let dt = self.time_step;

        // Step 1: Update positions using current velocity and acceleration.
        for body in &mut self.bodies {
            let half_acc_dt2 = body.acceleration.scale(0.5 * dt * dt);
            body.position = body.position + body.velocity.scale(dt) + half_acc_dt2;
        }

        // Step 2: Compute new accelerations from pairwise gravitational forces.
        let new_accels = self.compute_accelerations();

        // Step 3: Update velocities using average of old and new accelerations.
        for (body, new_acc) in self.bodies.iter_mut().zip(new_accels.into_iter()) {
            let avg_acc = (body.acceleration + new_acc).scale(0.5 * dt);
            body.velocity += avg_acc;
            body.acceleration = new_acc;
        }

        self.elapsed += dt;
    }

    /// Compute gravitational accelerations for all bodies.
    fn compute_accelerations(&self) -> Vec<Vector3D> {
        let n = self.bodies.len();
        let mut accels = vec![Vector3D::ZERO; n];
        let g = self.gravitational_constant;

        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.bodies[j].position - self.bodies[i].position;
                let dist_sq = r.magnitude_sq();
                if dist_sq == 0.0 {
                    continue;
                }
                let dist = dist_sq.sqrt();
                let force_mag = g * self.bodies[i].mass * self.bodies[j].mass / dist_sq;
                let direction = r.scale(1.0 / dist);
                accels[i] += direction.scale(force_mag / self.bodies[i].mass);
                accels[j] += -(direction.scale(force_mag / self.bodies[j].mass));
            }
        }
        accels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_add() {
        let a = Vector3D::new(1.0, 2.0, 3.0);
        let b = Vector3D::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vector3D::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_vector_magnitude() {
        let v = Vector3D::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_engine_determinism() {
        let run = |steps: u32| -> Vector3D {
            let mut engine = PhysicsEngine::new(6.674e-11, 1.0);
            engine.add_body(Body::new(Vector3D::new(0.0, 0.0, 0.0), 1.989e30));
            engine.add_body(Body::new(Vector3D::new(1.496e11, 0.0, 0.0), 5.972e24));
            // Give the second body an initial orbital velocity.
            engine.bodies[1].velocity = Vector3D::new(0.0, 29_783.0, 0.0);
            for _ in 0..steps {
                engine.step();
            }
            engine.bodies[1].position
        };

        // Same initial conditions must yield exactly the same result every time.
        let pos_a = run(10);
        let pos_b = run(10);
        assert_eq!(pos_a, pos_b);
    }

    #[test]
    fn test_elapsed_time_advances() {
        let mut engine = PhysicsEngine::new(6.674e-11, 0.5);
        engine.add_body(Body::new(Vector3D::ZERO, 1.0));
        engine.step();
        engine.step();
        assert!((engine.elapsed - 1.0).abs() < 1e-12);
    }
}
