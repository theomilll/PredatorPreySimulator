use crate::Rng;
use glam::Vec2;

pub struct Grass {
    cols: usize,
    rows: usize,
    cell: f32,
    biomass: Vec<f32>,
    capacity: Vec<f32>,
    scratch: Vec<f32>,
}

impl Grass {
    pub fn new(size: Vec2, cell: f32, rng: &mut Rng) -> Self {
        let cols = (size.x / cell).ceil() as usize;
        let rows = (size.y / cell).ceil() as usize;
        let coarse = noise(rng, cols, rows, 28.0);
        let fine = noise(rng, cols, rows, 9.0);
        let capacity: Vec<f32> = coarse
            .iter()
            .zip(&fine)
            .map(|(c, f)| 0.08 + 0.92 * smoothstep(0.3, 0.7, 0.7 * c + 0.3 * f))
            .collect();
        let biomass = capacity.iter().map(|k| k * rng.range(0.5, 1.0)).collect();
        Self {
            cols,
            rows,
            cell,
            biomass,
            scratch: vec![0.0; capacity.len()],
            capacity,
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cell(&self) -> f32 {
        self.cell
    }

    pub fn biomass(&self) -> &[f32] {
        &self.biomass
    }

    pub fn capacity(&self) -> &[f32] {
        &self.capacity
    }

    pub fn mean(&self) -> f32 {
        self.biomass.iter().sum::<f32>() / self.biomass.len() as f32
    }

    fn index(&self, pos: Vec2) -> usize {
        let x = ((pos.x / self.cell).max(0.0) as usize).min(self.cols - 1);
        let y = ((pos.y / self.cell).max(0.0) as usize).min(self.rows - 1);
        y * self.cols + x
    }

    pub fn at(&self, pos: Vec2) -> f32 {
        self.biomass[self.index(pos)]
    }

    pub fn graze(&mut self, pos: Vec2, amount: f32) -> f32 {
        let i = self.index(pos);
        let eaten = amount.min(self.biomass[i]);
        self.biomass[i] -= eaten;
        eaten
    }

    pub fn fertilize(&mut self, pos: Vec2, radius: f32) {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let center = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * self.cell;
                let falloff = 1.0 - center.distance(pos) / radius;
                if falloff > 0.0 {
                    let i = y * self.cols + x;
                    self.capacity[i] = self.capacity[i].max((falloff * 2.0).min(1.0));
                    self.biomass[i] = self.biomass[i].max(self.capacity[i]);
                }
            }
        }
    }

    pub fn grow(&mut self, rate: f32) {
        let (cols, rows) = (self.cols, self.rows);
        let b = &self.biomass;
        for y in 0..rows {
            let (up, down) = (y.saturating_sub(1) * cols, (y + 1).min(rows - 1) * cols);
            for x in 0..cols {
                let i = y * cols + x;
                let (left, right) = (x.saturating_sub(1), (x + 1).min(cols - 1));
                let neighbours = (b[y * cols + left] + b[y * cols + right] + b[up + x] + b[down + x]) * 0.25;
                let growth = rate * (b[i] + 0.5 * neighbours + 0.01) * (self.capacity[i] - b[i]);
                self.scratch[i] = (b[i] + growth).max(0.0);
            }
        }
        std::mem::swap(&mut self.biomass, &mut self.scratch);
    }
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn noise(rng: &mut Rng, cols: usize, rows: usize, period: f32) -> Vec<f32> {
    let gw = (cols as f32 / period).ceil() as usize + 2;
    let gh = (rows as f32 / period).ceil() as usize + 2;
    let lattice: Vec<f32> = (0..gw * gh).map(|_| rng.unit()).collect();
    let at = |x: usize, y: usize| lattice[y * gw + x];
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    (0..rows * cols)
        .map(|i| {
            let (fx, fy) = ((i % cols) as f32 / period, (i / cols) as f32 / period);
            let (x, y) = (fx as usize, fy as usize);
            let (tx, ty) = (smoothstep(0.0, 1.0, fx.fract()), smoothstep(0.0, 1.0, fy.fract()));
            lerp(
                lerp(at(x, y), at(x + 1, y), tx),
                lerp(at(x, y + 1), at(x + 1, y + 1), tx),
                ty,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> Grass {
        Grass::new(Vec2::new(160.0, 80.0), 8.0, &mut Rng::new(2))
    }

    #[test]
    fn grazing_removes_at_most_what_is_there() {
        let mut grass = field();
        let pos = Vec2::new(40.0, 40.0);
        let before = grass.at(pos);
        assert_eq!(grass.graze(pos, 10.0), before);
        assert_eq!(grass.at(pos), 0.0);
        assert_eq!(grass.graze(pos, 1.0), 0.0);
    }

    #[test]
    fn grazed_land_regrows_toward_capacity_without_exceeding_it() {
        let mut grass = field();
        for y in 0..10 {
            for x in 0..20 {
                grass.graze(Vec2::new(x as f32 * 8.0, y as f32 * 8.0), 1.0);
            }
        }
        assert_eq!(grass.mean(), 0.0);
        for _ in 0..3_000 {
            grass.grow(0.02);
        }
        let ratio: Vec<f32> = grass
            .biomass()
            .iter()
            .zip(grass.capacity())
            .map(|(b, k)| b / k)
            .collect();
        assert!(ratio.iter().all(|&r| r > 0.9 && r <= 1.0 + 1e-6));
    }

    #[test]
    fn fertilizing_raises_capacity_locally() {
        let mut grass = field();
        let center = Vec2::new(80.0, 40.0);
        grass.fertilize(center, 24.0);
        assert!(grass.at(center) > 0.9);
        assert!(grass.capacity()[grass.index(center)] > 0.9);
    }
}
