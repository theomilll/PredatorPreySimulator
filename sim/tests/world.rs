use ecosim::{Cause, Event, Params, Species, State, WORLD_SIZE, World};
use glam::Vec2;

fn quiet() -> Params {
    Params {
        migration: false,
        ..Params::default()
    }
}

fn center() -> Vec2 {
    WORLD_SIZE / 2.0
}

#[test]
fn same_seed_replays_identically() {
    let mut a = World::new(7, Params::default());
    let mut b = World::new(7, Params::default());
    for _ in 0..400 {
        a.step();
        b.step();
    }
    assert_eq!(a.animals(), b.animals());
    assert_eq!(a.history(), b.history());
    assert_eq!(a.totals(), b.totals());
}

#[test]
fn different_seeds_diverge() {
    let mut a = World::new(1, Params::default());
    let mut b = World::new(2, Params::default());
    for _ in 0..50 {
        a.step();
        b.step();
    }
    assert_ne!(a.animals(), b.animals());
}

#[test]
fn predators_without_prey_starve() {
    let mut world = World::empty(3, quiet());
    world.spawn_group(Species::Fox, center(), 6);
    let mut starved = 0;
    for _ in 0..3_000 {
        world.step();
        starved += world
            .events()
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::Death {
                        cause: Cause::Starvation,
                        species: Species::Fox,
                        ..
                    }
                )
            })
            .count();
        if world.animals().is_empty() {
            break;
        }
    }
    assert!(world.animals().is_empty());
    assert!(starved >= 6);
    assert_eq!(world.totals().deaths[Cause::Predation as usize], 0);
}

#[test]
fn prey_runs_from_a_nearby_predator() {
    let mut world = World::empty(4, quiet());
    world.spawn(Species::Rabbit, center());
    world.spawn(Species::Fox, center() + Vec2::new(20.0, 0.0));
    for _ in 0..5 {
        world.step();
    }
    let rabbit = world.animals().iter().find(|a| a.species == Species::Rabbit).unwrap();
    let fox = world.animals().iter().find(|a| a.species == Species::Fox).unwrap();
    assert_eq!(rabbit.state, State::Flee);
    assert!(rabbit.vel.dot(rabbit.pos - fox.pos) > 0.0);
}

#[test]
fn hungry_predators_eat_prey() {
    let mut world = World::empty(5, quiet());
    world.spawn_group(Species::Rabbit, center(), 40);
    world.spawn_group(Species::Fox, center(), 4);
    for _ in 0..2_000 {
        world.step();
    }
    assert!(world.totals().deaths[Cause::Predation as usize] > 0);
}

#[test]
fn a_lone_animal_never_breeds() {
    let mut world = World::empty(6, quiet());
    world.spawn(Species::Rabbit, center());
    for _ in 0..1_500 {
        world.step();
    }
    assert_eq!(world.totals().births, 0);
}

#[test]
fn offspring_belong_to_a_later_generation() {
    let mut world = World::empty(8, quiet());
    world.fertilize(center(), 200.0);
    world.spawn_group(Species::Rabbit, center(), 30);
    for _ in 0..800 {
        world.step();
    }
    assert!(world.totals().births > 0);
    let young: Vec<_> = world.animals().iter().filter(|a| a.generation > 0).collect();
    assert!(!young.is_empty());
    assert!(young.iter().all(|a| a.species == Species::Rabbit));
}

#[test]
fn population_cap_is_respected() {
    let params = Params {
        max_animals: 60,
        ..quiet()
    };
    let mut world = World::empty(9, params);
    world.fertilize(center(), 300.0);
    world.spawn_group(Species::Rabbit, center(), 50);
    for _ in 0..2_000 {
        world.step();
        assert!(world.animals().len() <= 60);
    }
}

#[test]
fn animals_die_of_old_age() {
    let mut world = World::empty(10, quiet());
    world.fertilize(center(), 400.0);
    world.spawn(Species::Rabbit, center());
    for _ in 0..3_000 {
        world.step();
    }
    assert_eq!(world.totals().deaths, [0, 0, 1]);
}

#[test]
fn migrants_recolonise_an_empty_world() {
    let mut world = World::empty(11, Params::default());
    let mut arrived = [false; 4];
    for _ in 0..3_000 {
        world.step();
        world.animals().iter().for_each(|a| arrived[a.species.index()] = true);
    }
    assert_eq!(arrived, [true; 4]);
}

#[test]
fn history_stays_bounded_and_ordered() {
    let mut world = World::empty(12, quiet());
    for _ in 0..12_000 {
        world.step();
    }
    let history = world.history();
    assert!(history.len() <= 480);
    assert!(history.windows(2).all(|w| w[0].tick < w[1].tick));
}
