use crate::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trait {
    Speed,
    Vision,
    Size,
    Efficiency,
}

impl Trait {
    pub const ALL: [Trait; 4] = [Trait::Speed, Trait::Vision, Trait::Size, Trait::Efficiency];

    pub fn name(self) -> &'static str {
        match self {
            Trait::Speed => "Speed",
            Trait::Vision => "Vision",
            Trait::Size => "Size",
            Trait::Efficiency => "Efficiency",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Genes([f32; 4]);

impl Genes {
    pub const MIN: f32 = 0.5;
    pub const MAX: f32 = 2.0;
    pub const BASE: Genes = Genes([1.0; 4]);

    pub fn get(&self, t: Trait) -> f32 {
        self.0[t as usize]
    }

    pub fn with(mut self, t: Trait, value: f32) -> Self {
        self.0[t as usize] = value.clamp(Self::MIN, Self::MAX);
        self
    }

    pub fn random(rng: &mut Rng, spread: f32) -> Self {
        Genes(std::array::from_fn(|_| {
            (1.0 + rng.gauss() * spread).clamp(Self::MIN, Self::MAX)
        }))
    }

    pub fn inherit(mother: &Genes, father: &Genes, mutation: f32, rng: &mut Rng) -> Self {
        Genes(std::array::from_fn(|i| {
            let parent = if rng.chance(0.5) { mother.0[i] } else { father.0[i] };
            (parent + rng.gauss() * mutation).clamp(Self::MIN, Self::MAX)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_mutation_each_gene_comes_from_a_parent() {
        let mut rng = Rng::new(11);
        let mother = Genes([0.6, 0.7, 0.8, 0.9]);
        let father = Genes([1.6, 1.7, 1.8, 1.9]);
        for _ in 0..200 {
            let child = Genes::inherit(&mother, &father, 0.0, &mut rng);
            for t in Trait::ALL {
                assert!(child.get(t) == mother.get(t) || child.get(t) == father.get(t));
            }
        }
    }

    #[test]
    fn mutation_never_leaves_bounds() {
        let mut rng = Rng::new(5);
        let extreme = Genes([Genes::MAX; 4]);
        for _ in 0..1_000 {
            let child = Genes::inherit(&extreme, &Genes::BASE, 0.5, &mut rng);
            assert!(
                Trait::ALL
                    .iter()
                    .all(|&t| (Genes::MIN..=Genes::MAX).contains(&child.get(t)))
            );
        }
    }
}
