//! How few keepers a night needs, found by trying everything.
//!
//! The last telling of a night is the only one that counts, and in it every
//! keeper walks at once, so the question "can `n` keepers hold this night?"
//! is a search over what `n` keepers do together, hour by hour. A snuffed
//! keeper's steps stop mattering, which keeps the search small. This is the
//! balance check: [`super::nights`]' tests use it to prove each night's par.

use std::collections::HashSet;

use super::night::{Ground, Level, Step, World};

/// Steps for `keepers` keepers that keep the hearth lit until dawn, or
/// `None` if there are none.
#[must_use]
pub fn solve(level: &Level, keepers: usize) -> Option<Vec<Vec<Step>>> {
    solve_as(level, keepers, false)
}

/// The same, but when `gently`, only nights in which no lantern goes out
/// count.
#[must_use]
pub fn solve_as(level: &Level, keepers: usize, gently: bool) -> Option<Vec<Vec<Step>>> {
    let ground = Ground::parse(level.rows);
    let world = World::dusk(&ground, level, keepers);
    let mut dead = HashSet::new();
    let mut tapes = vec![Vec::new(); keepers];
    search(&ground, level, &world, gently, &mut dead, &mut tapes).then_some(tapes)
}

/// How often `keepers` keepers stumbling about at random keep the hearth
/// lit, out of `tries`: a rough measure of how forgiving a night is.
#[must_use]
pub fn luck(level: &Level, keepers: usize, tries: u32, seed: u64) -> f32 {
    let ground = Ground::parse(level.rows);
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut scratch = Vec::new();
    let mut kept = 0_u32;
    for _ in 0..tries {
        let mut world = World::dusk(&ground, level, keepers);
        while world.hour < level.hours && world.lit() {
            let steps: Vec<Step> = world
                .keepers
                .iter()
                .map(|keeper| {
                    loop {
                        let step = Step::ALL[rng.usize(..5)];
                        if step == Step::Wait || ground.open(keeper.at.stepped(step)) {
                            break step;
                        }
                    }
                })
                .collect();
            scratch.clear();
            world.advance(&ground, &steps, &mut scratch);
        }
        kept += u32::from(world.lit());
    }
    kept as f32 / tries as f32
}

/// The fewest keepers that can hold the night, up to `most`.
#[must_use]
pub fn fewest(level: &Level, most: usize) -> Option<usize> {
    (1..=most).find(|&keepers| solve(level, keepers).is_some())
}

fn search(
    ground: &Ground,
    level: &Level,
    world: &World,
    gently: bool,
    dead: &mut HashSet<World>,
    tapes: &mut [Vec<Step>],
) -> bool {
    if !world.lit() || (gently && world.keepers.iter().any(|k| !k.lit)) {
        return false;
    }
    if world.hour == level.hours || world.shades.iter().all(Option::is_none) {
        return true;
    }
    if dead.contains(world) {
        return false;
    }
    let keepers = world.keepers.len();
    let mut steps = vec![Step::Wait; keepers];
    let mut scratch = Vec::new();
    let combos = 5_usize.pow(u32::try_from(keepers).expect("few keepers"));
    'combo: for combo in 0..combos {
        let mut rest = combo;
        for (i, step) in steps.iter_mut().enumerate() {
            *step = Step::ALL[rest % 5];
            rest /= 5;
            let keeper = world.keepers[i];
            // A snuffed keeper only waits, and a step into a tree is a wait
            // already tried.
            if *step != Step::Wait && (!keeper.lit || !ground.open(keeper.at.stepped(*step))) {
                continue 'combo;
            }
        }
        let mut next = world.clone();
        scratch.clear();
        next.advance(ground, &steps, &mut scratch);
        for (tape, &step) in tapes.iter_mut().zip(&steps) {
            tape.push(step);
        }
        if search(ground, level, &next, gently, dead, tapes) {
            return true;
        }
        for tape in tapes.iter_mut() {
            tape.pop();
        }
    }
    dead.insert(world.clone());
    false
}
