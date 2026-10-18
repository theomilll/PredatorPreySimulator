use glam::Vec2;
use std::f32::consts::TAU;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }

    pub fn range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        let span = u64::from(hi - lo + 1);
        lo + (((self.next_u64() >> 32) * span) >> 32) as u32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    pub fn gauss(&mut self) -> f32 {
        let u = self.unit().max(f32::MIN_POSITIVE);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (TAU * v).cos()
    }

    pub fn direction(&mut self) -> Vec2 {
        Vec2::from_angle(self.range(0.0, TAU))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert!((0..100).all(|_| a.next_u64() == b.next_u64()));
    }

    #[test]
    fn samples_stay_in_range() {
        let mut rng = Rng::new(1);
        for _ in 0..10_000 {
            let u = rng.unit();
            assert!((0.0..1.0).contains(&u));
            assert!((2..=4).contains(&rng.range_u32(2, 4)));
        }
    }

    #[test]
    fn gauss_is_roughly_standard_normal() {
        let mut rng = Rng::new(3);
        let samples: Vec<f32> = (0..20_000).map(|_| rng.gauss()).collect();
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / samples.len() as f32;
        assert!(mean.abs() < 0.05);
        assert!((var - 1.0).abs() < 0.05);
    }
}
