use ecosim::{Cause, Params, Species, Trait, World};
use std::time::Instant;

fn main() {
    let mut args = std::env::args()
        .skip(1)
        .map(|a| a.parse::<u64>().expect("numeric argument"));
    let seed = args.next().unwrap_or(42);
    let ticks = args.next().unwrap_or(20_000);
    let mut world = World::new(seed, Params::default());
    let start = Instant::now();
    let mut peak = 0;
    for _ in 0..ticks {
        world.step();
        peak = peak.max(world.animals().len());
        if world.tick().is_multiple_of(1_000) {
            let sample = world.history().last().expect("history is never empty");
            let line: Vec<String> = Species::ALL
                .iter()
                .map(|&s| {
                    let speed = sample.traits[s.index()][Trait::Speed as usize];
                    format!("{} {:>5} (speed {speed:.2})", s.profile().name, world.population(s))
                })
                .collect();
            println!(
                "t={:>6} grass {:.2} | {}",
                world.tick(),
                world.grass().mean(),
                line.join(" | ")
            );
        }
    }
    let elapsed = start.elapsed();
    let totals = world.totals();
    println!(
        "{ticks} ticks in {elapsed:.2?} ({:.3} ms/tick), peak {peak} animals, births {}, deaths {:?}",
        elapsed.as_secs_f64() * 1e3 / ticks as f64,
        totals.births,
        Cause::ALL.map(|c| totals.deaths[c as usize]),
    );
}
