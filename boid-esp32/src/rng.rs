/// Simple pseudo-random number generator using LCG (Linear Congruential Generator)
/// This is a basic RNG suitable for embedded systems where we don't need cryptographic quality
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generate next u32 value
    pub fn next_u32(&mut self) -> u32 {
        // LCG parameters from Numerical Recipes
        const A: u32 = 1664525;
        const C: u32 = 1013904223;

        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }

    /// Generate a float in range [0.0, 1.0)
    pub fn next_f32(&mut self) -> f32 {
        let value = self.next_u32();
        // Convert to float in range [0, 1)
        (value as f32) / (u32::MAX as f32)
    }

    /// Generate a float in a specific range
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_repeatability() {
        let mut rng1 = SimpleRng::new(12345);
        let mut rng2 = SimpleRng::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }

    #[test]
    fn test_f32_range() {
        let mut rng = SimpleRng::new(12345);

        for _ in 0..1000 {
            let val = rng.next_f32();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
}
