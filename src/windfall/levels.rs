//! The gardens, in order. Each one teaches one thing and every one can be
//! won with pieces to spare; the marks are where the thinking is.
//!
//! Pars are proven by the tests below: `par_pieces` is the fewest pieces
//! that win (nothing smaller does), and `par_gusts` is the least wind of
//! any win within one piece of that (the last garden: of
//! that many pieces exactly, since its search is the slowest). `cargo run --release --example
//! windfall_lab` prints both, with the plans that reach them.

use super::garden::{Basket, Cell, Dir, Drop, Hue, Kit, Level, Piece};

const fn d(tick: u16, x: i8, hue: Hue) -> Drop {
    Drop { tick, x, hue }
}

const fn b(x: i8, hue: Hue) -> Basket {
    Basket { x, hue }
}

const fn c(x: i8, y: i8) -> Cell {
    Cell::new(x, y)
}

use Hue::{Ember as E, Gold as G, Rust as R};

pub const LEVELS: [Level; 11] = [
    // Press play: the leaf is already over its basket.
    Level {
        width: 3,
        height: 3,
        drops: &[d(0, 1, E)],
        baskets: &[b(1, E)],
        fixed: &[],
        stones: &[],
        kit: Kit::Vanes,
        par_pieces: 0,
        par_gusts: 0,
    },
    // One vane.
    Level {
        width: 3,
        height: 3,
        drops: &[d(0, 0, E), d(3, 0, E)],
        baskets: &[b(1, E)],
        fixed: &[],
        stones: &[],
        kit: Kit::Vanes,
        par_pieces: 1,
        par_gusts: 2,
    },
    // A run of vanes, shared by two branches.
    Level {
        width: 4,
        height: 3,
        drops: &[d(0, 0, E), d(2, 1, E), d(4, 0, E)],
        baskets: &[b(3, E)],
        fixed: &[],
        stones: &[],
        kit: Kit::Vanes,
        par_pieces: 3,
        par_gusts: 8,
    },
    // Stones: the easy rows are closed, and left is a direction too.
    Level {
        width: 5,
        height: 4,
        drops: &[d(0, 1, G), d(2, 3, E), d(4, 1, G), d(6, 3, E)],
        baskets: &[b(0, G), b(4, E)],
        fixed: &[],
        stones: &[c(1, 0), c(0, 1), c(3, 0), c(4, 1), c(4, 2), c(0, 3)],
        kit: Kit::Vanes,
        par_pieces: 2,
        par_gusts: 4,
    },
    // Two colours, both going right: the one going farther goes higher.
    Level {
        width: 4,
        height: 3,
        drops: &[d(0, 0, E), d(2, 1, G), d(4, 0, E), d(6, 1, G)],
        baskets: &[b(2, E), b(3, G)],
        fixed: &[],
        stones: &[],
        kit: Kit::Vanes,
        par_pieces: 4,
        par_gusts: 8,
    },
    // The flip vane: one branch, two colours in turn.
    Level {
        width: 3,
        height: 3,
        drops: &[d(0, 1, E), d(2, 1, G), d(4, 1, E), d(6, 1, G)],
        baskets: &[b(0, E), b(2, G)],
        fixed: &[],
        stones: &[],
        kit: Kit::Flips,
        par_pieces: 1,
        par_gusts: 4,
    },
    // Both baskets on one side.
    Level {
        width: 5,
        height: 4,
        drops: &[d(0, 1, G), d(2, 1, E), d(4, 1, G), d(6, 1, E)],
        baskets: &[b(3, E), b(4, G)],
        fixed: &[],
        stones: &[],
        kit: Kit::Flips,
        par_pieces: 4,
        par_gusts: 14,
    },
    // Crossing: what plain vanes cannot do.
    Level {
        width: 4,
        height: 4,
        drops: &[d(0, 0, E), d(2, 3, G), d(4, 0, E), d(6, 3, G)],
        baskets: &[b(1, G), b(2, E)],
        fixed: &[],
        stones: &[],
        kit: Kit::Flips,
        par_pieces: 5,
        par_gusts: 12,
    },
    // Pairs: two of each, in turn.
    Level {
        width: 5,
        height: 4,
        drops: &[
            d(0, 2, E),
            d(2, 2, E),
            d(4, 2, G),
            d(6, 2, G),
            d(8, 2, E),
            d(10, 2, E),
            d(12, 2, G),
            d(14, 2, G),
        ],
        baskets: &[b(0, E), b(4, G)],
        fixed: &[],
        stones: &[],
        kit: Kit::Flips,
        par_pieces: 5,
        par_gusts: 24,
    },
    // Someone else's vanes, already standing.
    Level {
        width: 5,
        height: 5,
        drops: &[d(0, 0, E), d(2, 4, G), d(4, 0, G), d(6, 4, E), d(8, 0, E)],
        baskets: &[b(1, G), b(3, E)],
        fixed: &[
            (c(0, 1), Piece::Vane(Dir::Right)),
            (c(4, 1), Piece::Flip(Dir::Left)),
        ],
        stones: &[c(2, 2)],
        kit: Kit::Flips,
        par_pieces: 5,
        par_gusts: 15,
    },
    // All three colours.
    Level {
        width: 6,
        height: 5,
        drops: &[
            d(0, 1, E),
            d(2, 4, G),
            d(4, 1, R),
            d(6, 4, G),
            d(8, 1, E),
            d(10, 4, R),
        ],
        baskets: &[b(0, R), b(2, G), b(5, E)],
        fixed: &[],
        stones: &[c(3, 0), c(3, 1)],
        kit: Kit::Flips,
        par_pieces: 6,
        par_gusts: 33,
    },
];

#[cfg(test)]
mod tests {
    use super::super::garden::{Kit, Level, Plan, Run};
    use super::super::solve;
    use super::LEVELS;

    /// The last garden's search takes seconds in release and far longer in
    /// a debug build, so its proof is among the ignored ones below.
    const SLOW: usize = LEVELS.len() - 1;

    #[test]
    fn every_garden_is_well_formed() {
        for (n, level) in LEVELS.iter().enumerate() {
            assert!(!level.drops.is_empty(), "garden {n} drops nothing");
            assert!(
                level.drops.windows(2).all(|w| w[0].tick <= w[1].tick),
                "garden {n}'s drops are out of order"
            );
            for drop in level.drops {
                assert!(
                    (0..level.width).contains(&drop.x),
                    "garden {n}: drop off the branch"
                );
                assert!(
                    level.baskets.iter().any(|b| b.hue == drop.hue),
                    "garden {n}: a {:?} leaf with no basket",
                    drop.hue
                );
            }
            for basket in level.baskets {
                assert!(
                    (0..level.width).contains(&basket.x),
                    "garden {n}: basket off the ground"
                );
            }
            for &(at, _) in level.fixed {
                assert!(
                    level.inside(at) && !level.is_stone(at),
                    "garden {n}: fixed piece {at:?}"
                );
            }
            for &at in level.stones {
                assert!(level.inside(at), "garden {n}: stone {at:?}");
            }
        }
    }

    #[test]
    fn the_first_garden_needs_only_a_press_of_play() {
        let mut run = Run::new(&LEVELS[0], &Plan::empty(&LEVELS[0]));
        run.finish();
        assert!(run.all_home());
    }

    #[test]
    fn the_flip_vane_arrives_once_and_stays() {
        let first = LEVELS
            .iter()
            .position(|l| l.kit == Kit::Flips)
            .expect("flips arrive");
        assert!(LEVELS[first..].iter().all(|l| l.kit == Kit::Flips));
        assert_eq!(LEVELS[first].par_pieces, 1, "and is introduced on its own");
    }

    fn proves_pieces(n: usize, level: &Level) {
        let plan = solve::fewest(level, usize::from(level.par_pieces));
        assert_eq!(
            plan.as_ref().map(Plan::count),
            Some(usize::from(level.par_pieces)),
            "garden {n}: par is not the fewest pieces"
        );
        let mut run = Run::new(level, &plan.expect("checked"));
        run.finish();
        assert!(run.all_home(), "garden {n}: the par plan loses a leaf");
    }

    #[test]
    fn every_pieces_par_is_the_fewest_that_wins() {
        for (n, level) in LEVELS.iter().enumerate().take(SLOW) {
            proves_pieces(n, level);
        }
    }

    #[test]
    #[ignore = "seconds in release, minutes in debug: cargo test --release --lib windfall -- --ignored"]
    fn the_last_gardens_pieces_par_is_the_fewest_that_wins() {
        proves_pieces(SLOW, &LEVELS[SLOW]);
    }

    #[test]
    #[ignore = "an exhaustive search per garden: cargo test --release --lib windfall -- --ignored"]
    fn every_wind_par_is_the_least_the_search_finds() {
        for (n, level) in LEVELS.iter().enumerate() {
            let par = usize::from(level.par_pieces);
            let limit = if n == SLOW { par } else { par + 1 };
            let (_, gusts) = solve::gentlest(level, limit).expect("a garden can be won");
            assert_eq!(gusts, level.par_gusts, "garden {n}");
        }
    }

    #[test]
    fn no_garden_is_won_by_chance() {
        // Bar the lessons (one piece or none), a handful of pieces thrown
        // at random should almost never win: the plan is the player's.
        for (n, level) in LEVELS.iter().enumerate().filter(|(_, l)| l.par_pieces > 1) {
            let luck = solve::luck(level, usize::from(level.par_pieces).max(2), 2_000, 11);
            assert!(luck < 0.05, "garden {n} wins by chance {luck}");
        }
    }
}
