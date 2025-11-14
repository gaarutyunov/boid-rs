#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use rand::Rng;

/// A 2D vector used for position and velocity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f32 {
        #[cfg(feature = "std")]
        {
            (self.x * self.x + self.y * self.y).sqrt()
        }
        #[cfg(not(feature = "std"))]
        {
            libm::sqrtf(self.x * self.x + self.y * self.y)
        }
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            Self::zero()
        }
    }

    pub fn limit(&self, max: f32) -> Self {
        let mag = self.magnitude();
        if mag > max {
            let normalized = self.normalize();
            Self {
                x: normalized.x * max,
                y: normalized.y * max,
            }
        } else {
            *self
        }
    }

    pub fn distance(&self, other: &Vector2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        #[cfg(feature = "std")]
        {
            (dx * dx + dy * dy).sqrt()
        }
        #[cfg(not(feature = "std"))]
        {
            libm::sqrtf(dx * dx + dy * dy)
        }
    }
}

impl core::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl core::ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl core::ops::Mul<f32> for Vector2D {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl core::ops::Div<f32> for Vector2D {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl core::ops::AddAssign for Vector2D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

/// A single boid entity
#[derive(Debug, Clone)]
pub struct Boid {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub acceleration: Vector2D,
    pub wander_angle: f32,
}

impl Boid {
    pub fn new(position: Vector2D, velocity: Vector2D) -> Self {
        Self {
            position,
            velocity,
            acceleration: Vector2D::zero(),
            wander_angle: 0.0,
        }
    }

    #[cfg(feature = "std")]
    pub fn random(width: f32, height: f32) -> Self {
        let mut rng = rand::thread_rng();
        let position = Vector2D::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        let velocity = Vector2D::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0));
        Self::new(position, velocity)
    }

    pub fn apply_force(&mut self, force: Vector2D) {
        self.acceleration += force;
    }

    pub fn update(&mut self, max_speed: f32, _max_force: f32) {
        self.velocity += self.acceleration;
        self.velocity = self.velocity.limit(max_speed);
        self.position += self.velocity;
        self.acceleration = Vector2D::zero();
    }

    pub fn wrap_edges(&mut self, width: f32, height: f32) {
        if self.position.x < 0.0 {
            self.position.x = width;
        } else if self.position.x > width {
            self.position.x = 0.0;
        }

        if self.position.y < 0.0 {
            self.position.y = height;
        } else if self.position.y > height {
            self.position.y = 0.0;
        }
    }

    pub fn contain_within_bounds(&mut self, width: f32, height: f32) {
        let margin = 10.0;

        // Bounce off edges by reversing velocity component
        if self.position.x < margin {
            self.position.x = margin;
            self.velocity.x = self.velocity.x.abs();
        } else if self.position.x > width - margin {
            self.position.x = width - margin;
            self.velocity.x = -self.velocity.x.abs();
        }

        if self.position.y < margin {
            self.position.y = margin;
            self.velocity.y = self.velocity.y.abs();
        } else if self.position.y > height - margin {
            self.position.y = height - margin;
            self.velocity.y = -self.velocity.y.abs();
        }
    }
}

/// Configuration for the boid simulation
#[derive(Debug, Clone, Copy)]
pub struct BoidConfig {
    pub max_speed: f32,
    pub max_force: f32,
    pub separation_distance: f32,
    pub alignment_distance: f32,
    pub cohesion_distance: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub seek_weight: f32,
    pub wander_radius: f32,
}

impl Default for BoidConfig {
    fn default() -> Self {
        Self {
            max_speed: 2.0,
            max_force: 0.05,
            separation_distance: 15.0,
            alignment_distance: 25.0,
            cohesion_distance: 25.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            seek_weight: 8.0,
            wander_radius: 2.0,
        }
    }
}

/// Trait for flock behavior
pub trait FlockBehavior {
    fn separation(&self, boid: &Boid, config: &BoidConfig) -> Vector2D;
    fn alignment(&self, boid: &Boid, config: &BoidConfig) -> Vector2D;
    fn cohesion(&self, boid: &Boid, config: &BoidConfig) -> Vector2D;
    fn seek(&self, boid: &Boid, target: Vector2D, config: &BoidConfig) -> Vector2D;
}

/// Helper functions for boid behavior
pub mod behavior {
    use super::*;

    pub fn separation<'a, I>(boid: &Boid, others: I, config: &BoidConfig) -> Vector2D
    where
        I: Iterator<Item = &'a Boid>,
    {
        let mut steering = Vector2D::zero();
        let mut count = 0;

        for other in others {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < config.separation_distance {
                let mut diff = boid.position - other.position;
                diff = diff.normalize();
                diff = diff / distance;
                steering += diff;
                count += 1;
            }
        }

        if count > 0 {
            steering = steering / count as f32;
        }

        if steering.magnitude() > 0.0 {
            steering = steering.normalize();
            steering = steering * config.max_speed;
            steering = steering - boid.velocity;
            steering = steering.limit(config.max_force);
        }

        steering
    }

    pub fn alignment<'a, I>(boid: &Boid, others: I, config: &BoidConfig) -> Vector2D
    where
        I: Iterator<Item = &'a Boid>,
    {
        let mut sum = Vector2D::zero();
        let mut count = 0;

        for other in others {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < config.alignment_distance {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f32;
            sum = sum.normalize();
            sum = sum * config.max_speed;
            let steering = sum - boid.velocity;
            steering.limit(config.max_force)
        } else {
            Vector2D::zero()
        }
    }

    pub fn cohesion<'a, I>(boid: &Boid, others: I, config: &BoidConfig) -> Vector2D
    where
        I: Iterator<Item = &'a Boid>,
    {
        let mut sum = Vector2D::zero();
        let mut count = 0;

        for other in others {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < config.cohesion_distance {
                sum += other.position;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f32;
            seek(boid, sum, config)
        } else {
            Vector2D::zero()
        }
    }

    pub fn seek(boid: &Boid, target: Vector2D, config: &BoidConfig) -> Vector2D {
        let mut desired = target - boid.position;
        desired = desired.normalize();
        desired = desired * config.max_speed;
        let steering = desired - boid.velocity;
        steering.limit(config.max_force)
    }

    #[cfg(feature = "std")]
    pub fn wander(boid: &mut Boid, config: &BoidConfig) -> Vector2D {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Update wander angle with small random change
        boid.wander_angle += rng.gen_range(-0.05..0.05);

        // Convert angle to vector
        #[cfg(feature = "std")]
        let (sin, cos) = (boid.wander_angle.sin(), boid.wander_angle.cos());

        #[cfg(not(feature = "std"))]
        let (sin, cos) = (libm::sinf(boid.wander_angle), libm::cosf(boid.wander_angle));

        let mut wander_force = Vector2D::new(cos, sin);
        wander_force = wander_force.normalize();
        wander_force = wander_force * config.wander_radius;

        wander_force
    }
}

/// A collection of boids for embedded (no_std) environments
pub struct Flock<const N: usize> {
    pub boids: heapless::Vec<Boid, N>,
    pub config: BoidConfig,
    pub width: f32,
    pub height: f32,
}

impl<const N: usize> Flock<N> {
    pub fn new(width: f32, height: f32, config: BoidConfig) -> Self {
        Self {
            boids: heapless::Vec::new(),
            config,
            width,
            height,
        }
    }

    pub fn add_boid(&mut self, boid: Boid) -> Result<(), Boid> {
        self.boids.push(boid)
    }

    pub fn update(&mut self) {
        // Calculate forces for all boids
        let mut forces = heapless::Vec::<Vector2D, N>::new();

        for boid in self.boids.iter() {
            let sep = behavior::separation(boid, self.boids.iter(), &self.config)
                * self.config.separation_weight;
            let ali = behavior::alignment(boid, self.boids.iter(), &self.config)
                * self.config.alignment_weight;
            let coh = behavior::cohesion(boid, self.boids.iter(), &self.config)
                * self.config.cohesion_weight;
            let _ = forces.push(sep + ali + coh);
        }

        // Apply forces and update boids
        for (boid, force) in self.boids.iter_mut().zip(forces.iter()) {
            boid.apply_force(*force);
            boid.update(self.config.max_speed, self.config.max_force);
            boid.wrap_edges(self.width, self.height);
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

/// A collection of boids for std environments
#[cfg(feature = "std")]
pub struct FlockStd {
    pub boids: Vec<Boid>,
    pub config: BoidConfig,
    pub width: f32,
    pub height: f32,
}

#[cfg(feature = "std")]
impl FlockStd {
    pub fn new(width: f32, height: f32, count: usize) -> Self {
        let boids = (0..count).map(|_| Boid::random(width, height)).collect();

        Self {
            boids,
            config: BoidConfig::default(),
            width,
            height,
        }
    }

    pub fn new_with_config(width: f32, height: f32, count: usize, config: BoidConfig) -> Self {
        let boids = (0..count).map(|_| Boid::random(width, height)).collect();

        Self {
            boids,
            config,
            width,
            height,
        }
    }

    pub fn update(&mut self) {
        self.update_with_target(None);
    }

    pub fn update_with_target(&mut self, target: Option<Vector2D>) {
        // First update wander angles if seeking
        if target.is_some() {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            for boid in self.boids.iter_mut() {
                boid.wander_angle += rng.gen_range(-0.05..0.05);
            }
        }

        // Calculate forces for all boids
        let forces: Vec<Vector2D> = self
            .boids
            .iter()
            .map(|boid| {
                let sep = behavior::separation(boid, self.boids.iter(), &self.config)
                    * self.config.separation_weight;
                let ali = behavior::alignment(boid, self.boids.iter(), &self.config)
                    * self.config.alignment_weight;
                let coh = behavior::cohesion(boid, self.boids.iter(), &self.config)
                    * self.config.cohesion_weight;

                // Add seek and wander behaviors if target is present
                let (seek_force, wander_force) = if let Some(target_pos) = target {
                    let seek =
                        behavior::seek(boid, target_pos, &self.config) * self.config.seek_weight;

                    // Calculate wander using the updated angle
                    let (sin, cos) = (boid.wander_angle.sin(), boid.wander_angle.cos());
                    let mut wander = Vector2D::new(cos, sin);
                    wander = wander.normalize();
                    wander = wander * self.config.wander_radius;

                    (seek, wander)
                } else {
                    (Vector2D::zero(), Vector2D::zero())
                };

                sep + ali + coh + seek_force + wander_force
            })
            .collect();

        // Apply forces and update boids
        for (boid, force) in self.boids.iter_mut().zip(forces.iter()) {
            boid.apply_force(*force);
            boid.update(self.config.max_speed, self.config.max_force);

            // Keep boids within canvas bounds
            boid.contain_within_bounds(self.width, self.height);
        }
    }

    pub fn add_boid(&mut self, boid: Boid) {
        self.boids.push(boid);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2d_new() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_vector2d_magnitude() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.magnitude(), 5.0);
    }

    #[test]
    fn test_vector2d_normalize() {
        let v = Vector2D::new(3.0, 4.0);
        let normalized = v.normalize();
        assert!((normalized.magnitude() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_vector2d_operations() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);

        let sum = v1 + v2;
        assert_eq!(sum.x, 4.0);
        assert_eq!(sum.y, 6.0);

        let diff = v2 - v1;
        assert_eq!(diff.x, 2.0);
        assert_eq!(diff.y, 2.0);

        let scaled = v1 * 2.0;
        assert_eq!(scaled.x, 2.0);
        assert_eq!(scaled.y, 4.0);
    }

    #[test]
    fn test_boid_creation() {
        let pos = Vector2D::new(10.0, 20.0);
        let vel = Vector2D::new(1.0, 1.0);
        let boid = Boid::new(pos, vel);

        assert_eq!(boid.position.x, 10.0);
        assert_eq!(boid.position.y, 20.0);
        assert_eq!(boid.velocity.x, 1.0);
        assert_eq!(boid.velocity.y, 1.0);
    }

    #[test]
    fn test_boid_update() {
        let pos = Vector2D::new(0.0, 0.0);
        let vel = Vector2D::new(1.0, 1.0);
        let mut boid = Boid::new(pos, vel);

        boid.update(10.0, 1.0);

        assert_eq!(boid.position.x, 1.0);
        assert_eq!(boid.position.y, 1.0);
    }

    #[test]
    fn test_boid_wrap_edges() {
        let pos = Vector2D::new(-1.0, -1.0);
        let vel = Vector2D::new(0.0, 0.0);
        let mut boid = Boid::new(pos, vel);

        boid.wrap_edges(100.0, 100.0);

        assert_eq!(boid.position.x, 100.0);
        assert_eq!(boid.position.y, 100.0);
    }

    #[test]
    fn test_flock_creation() {
        let flock = FlockStd::new(800.0, 600.0, 50);
        assert_eq!(flock.boids.len(), 50);
        assert_eq!(flock.width, 800.0);
        assert_eq!(flock.height, 600.0);
    }

    #[test]
    fn test_flock_update() {
        let mut flock = FlockStd::new(800.0, 600.0, 10);
        let initial_positions: Vec<_> = flock.boids.iter().map(|b| b.position).collect();

        flock.update();

        // Positions should change after update
        let changed = flock
            .boids
            .iter()
            .zip(initial_positions.iter())
            .any(|(b, &initial)| b.position.x != initial.x || b.position.y != initial.y);

        assert!(changed);
    }

    #[test]
    fn test_flock_add_boid() {
        let mut flock = FlockStd::new(800.0, 600.0, 10);
        let initial_count = flock.boids.len();

        let boid = Boid::random(800.0, 600.0);
        flock.add_boid(boid);

        assert_eq!(flock.boids.len(), initial_count + 1);
    }
}
