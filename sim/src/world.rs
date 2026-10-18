use crate::{Genes, Grass, Rng, SpatialGrid, Species, Trait};
use glam::Vec2;
use std::f32::consts::TAU;

pub const WORLD_SIZE: Vec2 = Vec2::new(1500.0, 1100.0);
pub const YEAR: u64 = 3600;
const GRASS_CELL: f32 = 8.0;
const GRID_CELL: f32 = 48.0;
const HISTORY_LEN: usize = 480;
const GRASS_ENERGY: f32 = 14.0;
const BITE: f32 = 0.02;
const MOVE_COST: f32 = 0.04;
const VISION_COST: f32 = 0.01;
const STEER: f32 = 0.2;
const WALL: f32 = 24.0;
const FLEE_PACE: f32 = 1.4;
const HUNT_PACE: f32 = 1.6;
const MEAL: f32 = 0.8;
const PREFERENCE: f32 = 1e6;
const COVER: f32 = 0.6;
const SNIFF: f32 = 20.0;
const BREED_ENERGY: f32 = 0.6;
const MIGRANT_FLOOR: usize = 6;
const MIGRANT_CHANCE: f32 = 0.004;
const MIGRANT_GROUP: usize = 4;
const INITIAL: [u32; 4] = [900, 250, 25, 10];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sex {
    Female,
    Male,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Wander,
    Forage,
    Flee,
    Hunt,
    Court,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Predation,
    Starvation,
    OldAge,
}

impl Cause {
    pub const ALL: [Cause; 3] = [Cause::Predation, Cause::Starvation, Cause::OldAge];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Birth { pos: Vec2, species: Species },
    Death { pos: Vec2, species: Species, cause: Cause },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Animal {
    pub id: u32,
    pub species: Species,
    pub sex: Sex,
    pub pos: Vec2,
    pub vel: Vec2,
    pub energy: f32,
    pub age: u32,
    pub lifespan: u32,
    pub genes: Genes,
    pub generation: u32,
    pub cooldown: u32,
    pub state: State,
    pub focus: Option<Vec2>,
}

impl Animal {
    fn new(id: u32, species: Species, pos: Vec2, genes: Genes, generation: u32, rng: &mut Rng) -> Self {
        let lifespan = (species.profile().lifespan as f32 * rng.range(0.8, 1.2)) as u32;
        let sex = if rng.chance(0.5) { Sex::Female } else { Sex::Male };
        let mut animal = Self {
            id,
            species,
            sex,
            pos,
            vel: Vec2::ZERO,
            energy: 0.0,
            age: 0,
            lifespan,
            genes,
            generation,
            cooldown: 0,
            state: State::Wander,
            focus: None,
        };
        animal.energy = animal.max_energy() * 0.5;
        animal
    }

    pub fn max_energy(&self) -> f32 {
        self.species.profile().max_energy * self.genes.get(Trait::Size)
    }

    pub fn vision(&self) -> f32 {
        self.species.profile().vision * self.genes.get(Trait::Vision)
    }

    pub fn top_speed(&self) -> f32 {
        self.species.profile().speed * self.genes.get(Trait::Speed)
    }

    pub fn radius(&self) -> f32 {
        self.species.profile().radius * self.genes.get(Trait::Size).sqrt()
    }

    pub fn maturity(&self) -> u32 {
        (self.species.profile().maturity as f32 * self.genes.get(Trait::Efficiency)) as u32
    }

    pub fn can_breed(&self) -> bool {
        self.age >= self.maturity() && self.cooldown == 0 && self.energy >= self.max_energy() * BREED_ENERGY
    }

    fn is_hungry(&self) -> bool {
        let eating = matches!(self.state, State::Forage | State::Hunt);
        let threshold = match (eating, self.species.is_predator()) {
            (true, _) => 0.95,
            (false, true) => 0.6,
            (false, false) => 0.6,
        };
        self.energy < self.max_energy() * threshold
    }

    fn burn(&self) -> f32 {
        let p = self.species.profile();
        let size = self.genes.get(Trait::Size);
        let pace = self.vel.length() / p.speed;
        let speed = self.genes.get(Trait::Speed);
        p.metabolism * size * speed * speed / self.genes.get(Trait::Efficiency)
            + MOVE_COST * size * pace * pace
            + VISION_COST * self.genes.get(Trait::Vision)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub grass_growth: f32,
    pub seasons: f32,
    pub mutation: f32,
    pub hunt_success: f32,
    pub max_animals: usize,
    pub migration: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            grass_growth: 0.012,
            seasons: 0.6,
            mutation: 0.05,
            hunt_success: 1.0,
            max_animals: 8_000,
            migration: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub tick: u64,
    pub population: [u32; 4],
    pub grass: f32,
    pub traits: [[f32; 4]; 4],
    pub generation: [u32; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Totals {
    pub births: u64,
    pub deaths: [u64; 3],
}

#[derive(Clone, Copy)]
struct Intent {
    state: State,
    desired: Vec2,
    other: Option<usize>,
    focus: Option<Vec2>,
}

#[derive(Default)]
struct Nearest {
    index: Option<usize>,
    d2: f32,
}

impl Nearest {
    fn offer(&mut self, index: usize, d2: f32) {
        if self.index.is_none() || d2 < self.d2 {
            *self = Self { index: Some(index), d2 };
        }
    }
}

pub struct World {
    pub params: Params,
    seed: u64,
    rng: Rng,
    tick: u64,
    next_id: u32,
    animals: Vec<Animal>,
    grass: Grass,
    grid: SpatialGrid,
    intents: Vec<Intent>,
    fates: Vec<Option<Cause>>,
    newborns: Vec<Animal>,
    events: Vec<Event>,
    totals: Totals,
    history: Vec<Sample>,
    sample_every: u64,
}

impl World {
    pub fn empty(seed: u64, params: Params) -> Self {
        let mut rng = Rng::new(seed);
        let grass = Grass::new(WORLD_SIZE, GRASS_CELL, &mut rng);
        let mut world = Self {
            params,
            seed,
            rng,
            tick: 0,
            next_id: 0,
            animals: Vec::new(),
            grass,
            grid: SpatialGrid::new(WORLD_SIZE, GRID_CELL),
            intents: Vec::new(),
            fates: Vec::new(),
            newborns: Vec::new(),
            events: Vec::new(),
            totals: Totals::default(),
            history: Vec::new(),
            sample_every: 10,
        };
        world.record();
        world
    }

    pub fn new(seed: u64, params: Params) -> Self {
        let mut world = Self::empty(seed, params);
        for species in Species::ALL {
            for _ in 0..INITIAL[species.index()] {
                let pos = WORLD_SIZE * Vec2::new(world.rng.unit(), world.rng.unit());
                let id = world.spawn(species, pos);
                let animal = world.animals.iter_mut().find(|a| a.id == id).expect("just spawned");
                animal.age = world.rng.range_u32(0, animal.lifespan / 2);
                animal.energy = animal.max_energy() * world.rng.range(0.5, 0.9);
            }
        }
        world.history.clear();
        world.record();
        world
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn animals(&self) -> &[Animal] {
        &self.animals
    }

    pub fn animal(&self, id: u32) -> Option<&Animal> {
        self.animals.iter().find(|a| a.id == id)
    }

    pub fn grass(&self) -> &Grass {
        &self.grass
    }

    pub fn history(&self) -> &[Sample] {
        &self.history
    }

    pub fn totals(&self) -> &Totals {
        &self.totals
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn population(&self, species: Species) -> usize {
        self.animals.iter().filter(|a| a.species == species).count()
    }

    pub fn season(&self) -> f32 {
        (self.tick % YEAR) as f32 / YEAR as f32
    }

    pub fn growth_factor(&self) -> f32 {
        (1.0 + self.params.seasons * (TAU * self.season()).sin()).max(0.0)
    }

    pub fn spawn(&mut self, species: Species, pos: Vec2) -> u32 {
        let genes = Genes::random(&mut self.rng, 0.05);
        let mut animal = Animal::new(
            self.next_id,
            species,
            pos.clamp(Vec2::ZERO, WORLD_SIZE),
            genes,
            0,
            &mut self.rng,
        );
        animal.age = animal.maturity();
        animal.energy = animal.max_energy() * 0.8;
        self.next_id += 1;
        self.animals.push(animal);
        self.next_id - 1
    }

    pub fn spawn_group(&mut self, species: Species, center: Vec2, count: usize) {
        for _ in 0..count {
            let pos = center + self.rng.direction() * self.rng.range(0.0, 16.0);
            self.spawn(species, pos);
        }
    }

    pub fn fertilize(&mut self, pos: Vec2, radius: f32) {
        self.grass.fertilize(pos, radius);
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.events.clear();
        self.grass.grow(self.params.grass_growth * self.growth_factor());
        self.grid.rebuild(self.animals.iter().map(|a| a.pos));
        self.intents.clear();
        self.intents
            .extend((0..self.animals.len()).map(|i| decide(i, &self.animals, &self.grid, &self.grass, &mut self.rng)));
        self.act();
        self.fates.clear();
        self.fates.resize(self.animals.len(), None);
        self.hunt();
        self.breed();
        self.bury();
        self.animals.append(&mut self.newborns);
        self.migrate();
        self.record();
    }

    fn act(&mut self) {
        for (a, intent) in self.animals.iter_mut().zip(&self.intents) {
            a.vel += (intent.desired - a.vel) * STEER;
            a.pos += a.vel;
            for axis in 0..2 {
                if a.pos[axis] < 0.0 || a.pos[axis] > WORLD_SIZE[axis] {
                    a.pos[axis] = a.pos[axis].clamp(0.0, WORLD_SIZE[axis]);
                    a.vel[axis] = -a.vel[axis];
                }
            }
            a.energy -= a.burn();
            a.age += 1;
            a.cooldown = a.cooldown.saturating_sub(1);
            a.state = intent.state;
            a.focus = intent.focus;
            if a.state == State::Forage {
                let eaten = self.grass.graze(a.pos, BITE * a.genes.get(Trait::Size));
                a.energy = (a.energy + eaten * GRASS_ENERGY).min(a.max_energy());
            }
        }
    }

    fn hunt(&mut self) {
        for i in 0..self.animals.len() {
            let Intent {
                state: State::Hunt,
                other: Some(j),
                ..
            } = self.intents[i]
            else {
                continue;
            };
            if self.fates[i].is_some() || self.fates[j].is_some() {
                continue;
            }
            let (hunter, prey) = (&self.animals[i], &self.animals[j]);
            if hunter.pos.distance(prey.pos) > hunter.radius() + prey.radius() + 1.0 {
                continue;
            }
            let odds = hunter.species.profile().hunt_success
                * self.params.hunt_success
                * (hunter.genes.get(Trait::Speed) / prey.genes.get(Trait::Speed))
                * (hunter.genes.get(Trait::Size) / prey.genes.get(Trait::Size)).sqrt();
            if !self.rng.chance(odds) {
                continue;
            }
            let meal = prey.max_energy() * MEAL;
            self.fates[j] = Some(Cause::Predation);
            let hunter = &mut self.animals[i];
            hunter.energy = (hunter.energy + meal).min(hunter.max_energy());
        }
    }

    fn breed(&mut self) {
        for i in 0..self.animals.len() {
            let Intent {
                state: State::Court,
                other: Some(j),
                ..
            } = self.intents[i]
            else {
                continue;
            };
            let (mother, father) = (&self.animals[i], &self.animals[j]);
            let ready = mother.sex == Sex::Female
                && self.fates[i].is_none()
                && self.fates[j].is_none()
                && mother.can_breed()
                && father.can_breed()
                && mother.pos.distance(father.pos) < (mother.radius() + father.radius()) * 2.0 + 2.0;
            if !ready {
                continue;
            }
            let p = mother.species.profile();
            let litter = self.rng.range_u32(p.litter.0, p.litter.1);
            let dowry = mother.max_energy() * p.breed_cost / litter as f32;
            for _ in 0..litter {
                if self.animals.len() + self.newborns.len() >= self.params.max_animals {
                    break;
                }
                let genes = Genes::inherit(&mother.genes, &father.genes, self.params.mutation, &mut self.rng);
                let pos = (mother.pos + self.rng.direction() * self.rng.range(0.0, 4.0)).clamp(Vec2::ZERO, WORLD_SIZE);
                let generation = mother.generation.max(father.generation) + 1;
                let mut child = Animal::new(self.next_id, mother.species, pos, genes, generation, &mut self.rng);
                child.energy = dowry;
                self.newborns.push(child);
                self.next_id += 1;
                self.totals.births += 1;
                self.events.push(Event::Birth {
                    pos,
                    species: mother.species,
                });
            }
            for (k, cooldown, share) in [(i, p.gestation, 1.0), (j, p.gestation / 2, 0.5)] {
                let parent = &mut self.animals[k];
                parent.energy -= parent.max_energy() * p.breed_cost * share;
                parent.cooldown = cooldown;
            }
        }
    }

    fn bury(&mut self) {
        for (a, fate) in self.animals.iter().zip(self.fates.iter_mut()) {
            if fate.is_none() {
                if a.energy <= 0.0 {
                    *fate = Some(Cause::Starvation);
                } else if a.age >= a.lifespan {
                    *fate = Some(Cause::OldAge);
                }
            }
            if let Some(cause) = *fate {
                self.totals.deaths[cause as usize] += 1;
                self.events.push(Event::Death {
                    pos: a.pos,
                    species: a.species,
                    cause,
                });
            }
        }
        let mut fates = self.fates.iter();
        self.animals.retain(|_| fates.next().is_some_and(Option::is_none));
    }

    fn migrate(&mut self) {
        if !self.params.migration {
            return;
        }
        for species in Species::ALL {
            if self.population(species) >= MIGRANT_FLOOR || !self.rng.chance(MIGRANT_CHANCE) {
                continue;
            }
            let along = self.rng.unit();
            let gate = match self.rng.range_u32(0, 3) {
                0 => Vec2::new(along, 0.0),
                1 => Vec2::new(along, 1.0),
                2 => Vec2::new(0.0, along),
                _ => Vec2::new(1.0, along),
            } * WORLD_SIZE;
            self.spawn_group(species, gate, MIGRANT_GROUP);
        }
    }

    fn record(&mut self) {
        if !self.tick.is_multiple_of(self.sample_every) {
            return;
        }
        let mut population = [0u32; 4];
        let mut sums = [[0.0f32; 4]; 4];
        let mut generation = [0u32; 4];
        for a in &self.animals {
            let s = a.species.index();
            population[s] += 1;
            generation[s] = generation[s].max(a.generation);
            for t in Trait::ALL {
                sums[s][t as usize] += a.genes.get(t);
            }
        }
        let traits = std::array::from_fn(|s| sums[s].map(|sum| sum / population[s].max(1) as f32));
        self.history.push(Sample {
            tick: self.tick,
            population,
            grass: self.grass.mean(),
            traits,
            generation,
        });
        if self.history.len() > HISTORY_LEN {
            self.sample_every *= 2;
            let every = self.sample_every;
            self.history.retain(|s| s.tick.is_multiple_of(every));
        }
    }
}

fn decide(i: usize, animals: &[Animal], grid: &SpatialGrid, grass: &Grass, rng: &mut Rng) -> Intent {
    let me = &animals[i];
    let vision = me.vision();
    let wants_mate = me.can_breed();
    let reach = if wants_mate {
        vision.max(me.species.profile().mate_range)
    } else {
        vision
    };
    let (mut threat, mut prey, mut mate) = (Nearest::default(), Nearest::default(), Nearest::default());
    let (mut separation, mut flock_offset, mut flock_vel, mut flock) = (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0);

    for j in grid.nearby(me.pos, reach) {
        let other = &animals[j];
        let offset = other.pos - me.pos;
        let d2 = offset.length_squared();
        if j == i || d2 > reach * reach {
            continue;
        }
        if wants_mate && other.species == me.species && other.sex != me.sex && other.can_breed() {
            mate.offer(j, d2);
        }
        if d2 > vision * vision {
            continue;
        }
        if other.species.eats(me.species) {
            threat.offer(j, d2);
        } else if let Some(rank) = me.species.profile().prey.iter().position(|&s| s == other.species) {
            let hidden = other.species.profile().hides_in_grass && grass.at(other.pos) > COVER && d2 > SNIFF * SNIFF;
            if !hidden {
                prey.offer(j, d2 + rank as f32 * PREFERENCE);
            }
        } else if other.species == me.species {
            let personal = (me.radius() + other.radius()) * 1.5;
            if d2 < personal * personal {
                separation -= offset / d2.max(0.01).sqrt();
            }
            flock_offset += offset;
            flock_vel += other.vel;
            flock += 1;
        }
    }

    let speed = me.top_speed();
    let toward = |j: usize| (animals[j].pos - me.pos).normalize_or_zero();
    let hungry = me.is_hungry();
    let (state, desired, other) = if let Some(j) = threat.index {
        (State::Flee, -toward(j) * speed * FLEE_PACE, Some(j))
    } else if hungry && let Some(j) = prey.index {
        let lead = animals[j].vel * (me.pos.distance(animals[j].pos) / speed).min(30.0);
        let chase = (animals[j].pos + lead - me.pos).normalize_or_zero();
        (State::Hunt, chase * speed * HUNT_PACE, Some(j))
    } else if hungry && !me.species.is_predator() {
        (State::Forage, forage(me, grass, vision, rng) * speed, None)
    } else if let Some(j) = mate.index {
        (State::Court, toward(j) * speed * 0.8, Some(j))
    } else {
        let herd = if flock > 0 {
            ((flock_offset / flock as f32).normalize_or_zero() * 0.5 + (flock_vel / flock as f32).normalize_or_zero())
                * me.species.profile().herding
        } else {
            Vec2::ZERO
        };
        (
            State::Wander,
            (wander(me, rng) + herd).normalize_or_zero() * speed * 0.5,
            None,
        )
    };

    let wall = WORLD_SIZE - me.pos;
    let avoid = (Vec2::splat(WALL) - me.pos).max(Vec2::ZERO) - (Vec2::splat(WALL) - wall).max(Vec2::ZERO);
    Intent {
        state,
        desired: desired + (separation.clamp_length_max(1.0) + avoid / WALL) * speed,
        other,
        focus: other.map(|j| animals[j].pos),
    }
}

fn wander(me: &Animal, rng: &mut Rng) -> Vec2 {
    let heading = me.vel.try_normalize().unwrap_or_else(|| rng.direction());
    Vec2::from_angle(rng.gauss() * 0.35).rotate(heading)
}

fn forage(me: &Animal, grass: &Grass, vision: f32, rng: &mut Rng) -> Vec2 {
    let here = grass.at(me.pos);
    if here > 0.3 {
        return Vec2::ZERO;
    }
    let offset = rng.range(0.0, TAU);
    let (best, value) = (0..8)
        .map(|k| Vec2::from_angle(offset + k as f32 * TAU / 8.0))
        .map(|dir| (dir, grass.at(me.pos + dir * vision * 0.6)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("eight directions");
    if value > here { best } else { wander(me, rng) }
}
