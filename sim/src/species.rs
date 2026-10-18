#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Species {
    Rabbit,
    Deer,
    Fox,
    Wolf,
}

#[derive(Debug)]
pub struct Profile {
    pub name: &'static str,
    pub speed: f32,
    pub vision: f32,
    pub mate_range: f32,
    pub radius: f32,
    pub max_energy: f32,
    pub metabolism: f32,
    pub lifespan: u32,
    pub maturity: u32,
    pub gestation: u32,
    pub litter: (u32, u32),
    pub breed_cost: f32,
    pub herding: f32,
    pub prey: &'static [Species],
    pub hunt_success: f32,
    pub hides_in_grass: bool,
}

const RABBIT: Profile = Profile {
    name: "Rabbit",
    speed: 1.0,
    vision: 45.0,
    mate_range: 90.0,
    radius: 2.6,
    max_energy: 60.0,
    metabolism: 0.05,
    lifespan: 2400,
    maturity: 400,
    gestation: 700,
    litter: (1, 3),
    breed_cost: 0.5,
    herding: 0.2,
    prey: &[],
    hunt_success: 0.0,
    hides_in_grass: true,
};

const DEER: Profile = Profile {
    name: "Deer",
    speed: 1.15,
    vision: 70.0,
    mate_range: 140.0,
    radius: 4.0,
    max_energy: 120.0,
    metabolism: 0.08,
    lifespan: 4800,
    maturity: 700,
    gestation: 900,
    litter: (1, 1),
    breed_cost: 0.5,
    herding: 0.6,
    prey: &[],
    hunt_success: 0.0,
    hides_in_grass: false,
};

const FOX: Profile = Profile {
    name: "Fox",
    speed: 1.2,
    vision: 60.0,
    mate_range: 260.0,
    radius: 3.2,
    max_energy: 90.0,
    metabolism: 0.09,
    lifespan: 3000,
    maturity: 350,
    gestation: 400,
    litter: (1, 2),
    breed_cost: 0.5,
    herding: 0.0,
    prey: &[Species::Rabbit],
    hunt_success: 0.35,
    hides_in_grass: false,
};

const WOLF: Profile = Profile {
    name: "Wolf",
    speed: 1.3,
    vision: 85.0,
    mate_range: 300.0,
    radius: 4.6,
    max_energy: 160.0,
    metabolism: 0.08,
    lifespan: 4200,
    maturity: 500,
    gestation: 600,
    litter: (2, 3),
    breed_cost: 0.5,
    herding: 0.5,
    prey: &[Species::Deer, Species::Rabbit],
    hunt_success: 0.7,
    hides_in_grass: false,
};

impl Species {
    pub const ALL: [Species; 4] = [Species::Rabbit, Species::Deer, Species::Fox, Species::Wolf];

    pub fn profile(self) -> &'static Profile {
        match self {
            Species::Rabbit => &RABBIT,
            Species::Deer => &DEER,
            Species::Fox => &FOX,
            Species::Wolf => &WOLF,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn is_predator(self) -> bool {
        !self.profile().prey.is_empty()
    }

    pub fn eats(self, other: Species) -> bool {
        self.profile().prey.contains(&other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_chain() {
        assert!(Species::Fox.eats(Species::Rabbit));
        assert!(!Species::Fox.eats(Species::Deer));
        assert!(Species::Wolf.eats(Species::Deer) && Species::Wolf.eats(Species::Rabbit));
        assert!(!Species::Wolf.eats(Species::Fox));
        assert!(!Species::Rabbit.is_predator() && !Species::Deer.is_predator());
        assert!(Species::Fox.is_predator() && Species::Wolf.is_predator());
    }
}
