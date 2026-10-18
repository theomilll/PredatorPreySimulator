use glam::Vec2;

pub struct SpatialGrid {
    cell: f32,
    cols: usize,
    rows: usize,
    starts: Vec<u32>,
    cursor: Vec<u32>,
    cells: Vec<u32>,
    items: Vec<u32>,
}

impl SpatialGrid {
    pub fn new(size: Vec2, cell: f32) -> Self {
        let cols = (size.x / cell).ceil() as usize;
        let rows = (size.y / cell).ceil() as usize;
        Self {
            cell,
            cols,
            rows,
            starts: vec![0; cols * rows + 1],
            cursor: Vec::new(),
            cells: Vec::new(),
            items: Vec::new(),
        }
    }

    fn coords(&self, p: Vec2) -> (usize, usize) {
        let x = ((p.x / self.cell).max(0.0) as usize).min(self.cols - 1);
        let y = ((p.y / self.cell).max(0.0) as usize).min(self.rows - 1);
        (x, y)
    }

    pub fn rebuild(&mut self, positions: impl Iterator<Item = Vec2>) {
        let mut cells = std::mem::take(&mut self.cells);
        cells.clear();
        cells.extend(positions.map(|p| {
            let (x, y) = self.coords(p);
            (y * self.cols + x) as u32
        }));
        self.cells = cells;
        self.starts.fill(0);
        for &c in &self.cells {
            self.starts[c as usize + 1] += 1;
        }
        for i in 1..self.starts.len() {
            self.starts[i] += self.starts[i - 1];
        }
        self.cursor.clone_from(&self.starts);
        self.items.resize(self.cells.len(), 0);
        for (i, &c) in self.cells.iter().enumerate() {
            let slot = &mut self.cursor[c as usize];
            self.items[*slot as usize] = i as u32;
            *slot += 1;
        }
    }

    pub fn nearby(&self, center: Vec2, radius: f32) -> impl Iterator<Item = usize> + '_ {
        let (x0, y0) = self.coords(center - radius);
        let (x1, y1) = self.coords(center + radius);
        (y0..=y1)
            .flat_map(move |y| (x0..=x1).map(move |x| y * self.cols + x))
            .flat_map(move |c| {
                self.items[self.starts[c] as usize..self.starts[c + 1] as usize]
                    .iter()
                    .map(|&i| i as usize)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rng;

    #[test]
    fn finds_exactly_what_brute_force_finds() {
        let size = Vec2::new(500.0, 300.0);
        let mut rng = Rng::new(9);
        let points: Vec<Vec2> = (0..2_000)
            .map(|_| Vec2::new(rng.range(0.0, size.x), rng.range(0.0, size.y)))
            .collect();
        let mut grid = SpatialGrid::new(size, 32.0);
        grid.rebuild(points.iter().copied());

        for _ in 0..50 {
            let center = Vec2::new(rng.range(-20.0, size.x + 20.0), rng.range(0.0, size.y));
            let radius = rng.range(5.0, 90.0);
            let mut found: Vec<usize> = grid
                .nearby(center, radius)
                .filter(|&i| points[i].distance(center) <= radius)
                .collect();
            found.sort_unstable();
            let expected: Vec<usize> = (0..points.len())
                .filter(|&i| points[i].distance(center) <= radius)
                .collect();
            assert_eq!(found, expected);
        }
    }
}
