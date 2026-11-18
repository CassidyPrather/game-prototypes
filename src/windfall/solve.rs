//! How few pieces a garden needs, and how little wind, found by trying
//! everything that could matter.
//!
//! The search leans on one fact: a run only ever reads the cells its leaves
//! pass through. So if a plan loses a leaf, any plan that saves it must put
//! something different in one of the cells the leaves crossed before that
//! loss ([`Run::deciding_cells`]). Branching only on those cells keeps the
//! search small enough to be exhaustive, which is what lets the levels'
//! tests call a par proven.

use std::collections::HashSet;

use super::garden::{Level, Plan, Run};

/// The fewest pieces that bring every leaf home, as a plan, if any plan
/// of at most `limit` pieces does.
#[must_use]
pub fn fewest(level: &Level, limit: usize) -> Option<Plan> {
    (0..=limit).find_map(|n| {
        let mut seen = HashSet::new();
        let mut plan = Plan::empty(level);
        first(level, &mut plan, n, &mut seen)
    })
}

fn first(level: &Level, plan: &mut Plan, limit: usize, seen: &mut HashSet<Plan>) -> Option<Plan> {
    if !seen.insert(plan.clone()) {
        return None;
    }
    let (cells, lost) = Run::deciding_cells(level, plan);
    if lost.is_none() {
        return Some(plan.clone());
    }
    if plan.count() >= limit {
        return None;
    }
    for at in cells {
        if plan.get(at).is_some() || !level.open(at) {
            continue;
        }
        for piece in level.kit.pieces() {
            plan.set(level, at, Some(piece));
            let found = first(level, plan, limit, seen);
            plan.set(level, at, None);
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

/// The gentlest solution with at most `limit` pieces: the least wind, and
/// among those the fewest pieces. `None` if there is no solution that small.
#[must_use]
pub fn gentlest(level: &Level, limit: usize) -> Option<(Plan, u16)> {
    let mut seen = HashSet::new();
    let mut best = None;
    let mut plan = Plan::empty(level);
    every(level, &mut plan, limit, &mut seen, &mut best);
    best
}

fn every(
    level: &Level,
    plan: &mut Plan,
    limit: usize,
    seen: &mut HashSet<Plan>,
    best: &mut Option<(Plan, u16)>,
) {
    if !seen.insert(plan.clone()) {
        return;
    }
    let (cells, lost) = Run::deciding_cells(level, plan);
    if lost.is_none() {
        let mut run = Run::new(level, plan);
        run.finish();
        let better = best.as_ref().is_none_or(|(p, g)| {
            run.gusts() < *g || (run.gusts() == *g && plan.count() < p.count())
        });
        if better {
            *best = Some((plan.clone(), run.gusts()));
        }
    }
    if plan.count() >= limit {
        return;
    }
    for at in cells {
        if plan.get(at).is_some() || !level.open(at) {
            continue;
        }
        for piece in level.kit.pieces() {
            plan.set(level, at, Some(piece));
            every(level, plan, limit, seen, best);
            plan.set(level, at, None);
        }
    }
}

/// How many plans of exactly `pieces` pieces, placed at random on open
/// cells, bring every leaf home, out of `tries`: a rough measure of how
/// forgiving a garden is.
#[must_use]
pub fn luck(level: &Level, pieces: usize, tries: u32, seed: u64) -> f32 {
    let mut rng = fastrand::Rng::with_seed(seed);
    let open: Vec<_> = (0..level.height)
        .flat_map(|y| (0..level.width).map(move |x| super::garden::Cell::new(x, y)))
        .filter(|&c| level.open(c))
        .collect();
    let kit: Vec<_> = level.kit.pieces().collect();
    let mut won = 0_u32;
    for _ in 0..tries {
        let mut plan = Plan::empty(level);
        while plan.count() < pieces.min(open.len()) {
            let at = open[rng.usize(..open.len())];
            plan.set(level, at, Some(kit[rng.usize(..kit.len())]));
        }
        let mut run = Run::new(level, &plan);
        run.finish();
        won += u32::from(run.all_home());
    }
    won as f32 / tries as f32
}
