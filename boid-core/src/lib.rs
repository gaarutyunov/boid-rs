use rand::Rng;

/// A 2D vector used for position and velocity
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
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

    pub fn limit(&self, max: f64) -> Self {
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

    pub fn distance(&self, other: &Vector2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl std::ops::Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f64> for Vector2D {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f64> for Vector2D {
    type Output = Self;

    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl std::ops::AddAssign for Vector2D {
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
}

impl Boid {
    pub fn new(position: Vector2D, velocity: Vector2D) -> Self {
        Self {
            position,
            velocity,
            acceleration: Vector2D::zero(),
        }
    }

    pub fn random(width: f64, height: f64) -> Self {
        let mut rng = rand::thread_rng();
        let position = Vector2D::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));
        let velocity = Vector2D::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0));
        Self::new(position, velocity)
    }

    pub fn apply_force(&mut self, force: Vector2D) {
        self.acceleration += force;
    }

    pub fn update(&mut self, max_speed: f64, _max_force: f64) {
        self.velocity += self.acceleration;
        self.velocity = self.velocity.limit(max_speed);
        self.position += self.velocity;
        self.acceleration = Vector2D::zero();
    }

    pub fn wrap_edges(&mut self, width: f64, height: f64) {
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
}

/// Configuration for the boid simulation
#[derive(Debug, Clone)]
pub struct BoidConfig {
    pub max_speed: f64,
    pub max_force: f64,
    pub separation_distance: f64,
    pub alignment_distance: f64,
    pub cohesion_distance: f64,
    pub separation_weight: f64,
    pub alignment_weight: f64,
    pub cohesion_weight: f64,
}

impl Default for BoidConfig {
    fn default() -> Self {
        Self {
            max_speed: 4.0,
            max_force: 0.1,
            separation_distance: 25.0,
            alignment_distance: 50.0,
            cohesion_distance: 50.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
        }
    }
}

/// A collection of boids
pub struct Flock {
    pub boids: Vec<Boid>,
    pub config: BoidConfig,
    pub width: f64,
    pub height: f64,
}

impl Flock {
    pub fn new(width: f64, height: f64, count: usize) -> Self {
        let boids = (0..count).map(|_| Boid::random(width, height)).collect();

        Self {
            boids,
            config: BoidConfig::default(),
            width,
            height,
        }
    }

    pub fn new_with_config(width: f64, height: f64, count: usize, config: BoidConfig) -> Self {
        let boids = (0..count).map(|_| Boid::random(width, height)).collect();

        Self {
            boids,
            config,
            width,
            height,
        }
    }

    /// Separation: steer to avoid crowding local flockmates
    fn separation(&self, boid: &Boid) -> Vector2D {
        let mut steering = Vector2D::zero();
        let mut count = 0;

        for other in &self.boids {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < self.config.separation_distance {
                let mut diff = boid.position - other.position;
                diff = diff.normalize();
                diff = diff / distance; // Weight by distance
                steering += diff;
                count += 1;
            }
        }

        if count > 0 {
            steering = steering / count as f64;
        }

        if steering.magnitude() > 0.0 {
            steering = steering.normalize();
            steering = steering * self.config.max_speed;
            steering = steering - boid.velocity;
            steering = steering.limit(self.config.max_force);
        }

        steering
    }

    /// Alignment: steer towards the average heading of local flockmates
    fn alignment(&self, boid: &Boid) -> Vector2D {
        let mut sum = Vector2D::zero();
        let mut count = 0;

        for other in &self.boids {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < self.config.alignment_distance {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f64;
            sum = sum.normalize();
            sum = sum * self.config.max_speed;
            let steering = sum - boid.velocity;
            steering.limit(self.config.max_force)
        } else {
            Vector2D::zero()
        }
    }

    /// Cohesion: steer to move toward the average position of local flockmates
    fn cohesion(&self, boid: &Boid) -> Vector2D {
        let mut sum = Vector2D::zero();
        let mut count = 0;

        for other in &self.boids {
            let distance = boid.position.distance(&other.position);
            if distance > 0.0 && distance < self.config.cohesion_distance {
                sum += other.position;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f64;
            self.seek(boid, sum)
        } else {
            Vector2D::zero()
        }
    }

    /// Seek a target position
    fn seek(&self, boid: &Boid, target: Vector2D) -> Vector2D {
        let mut desired = target - boid.position;
        desired = desired.normalize();
        desired = desired * self.config.max_speed;
        let steering = desired - boid.velocity;
        steering.limit(self.config.max_force)
    }

    /// Apply all boid rules and update positions
    pub fn update(&mut self) {
        // Calculate forces for all boids
        let forces: Vec<Vector2D> = self
            .boids
            .iter()
            .map(|boid| {
                let sep = self.separation(boid) * self.config.separation_weight;
                let ali = self.alignment(boid) * self.config.alignment_weight;
                let coh = self.cohesion(boid) * self.config.cohesion_weight;
                sep + ali + coh
            })
            .collect();

        // Apply forces and update boids
        for (boid, force) in self.boids.iter_mut().zip(forces.iter()) {
            boid.apply_force(*force);
            boid.update(self.config.max_speed, self.config.max_force);
            boid.wrap_edges(self.width, self.height);
        }
    }

    pub fn add_boid(&mut self, boid: Boid) {
        self.boids.push(boid);
    }

    pub fn resize(&mut self, width: f64, height: f64) {
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
        let flock = Flock::new(800.0, 600.0, 50);
        assert_eq!(flock.boids.len(), 50);
        assert_eq!(flock.width, 800.0);
        assert_eq!(flock.height, 600.0);
    }

    #[test]
    fn test_flock_update() {
        let mut flock = Flock::new(800.0, 600.0, 10);
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
        let mut flock = Flock::new(800.0, 600.0, 10);
        let initial_count = flock.boids.len();

        let boid = Boid::random(800.0, 600.0);
        flock.add_boid(boid);

        assert_eq!(flock.boids.len(), initial_count + 1);
    }
}
