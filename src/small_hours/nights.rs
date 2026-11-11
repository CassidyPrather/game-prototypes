//! The nights, in order. Hand-made; nothing is rolled.
//!
//! Each is one thing to notice, then the things noticed so far in a tighter
//! space. `par` is not a guess: the tests below search every night
//! exhaustively and fail if a night can be kept with fewer keepers than it
//! says, or cannot be kept with that many.
//!
//! The maps were found with `examples/small_hours_lab.rs`, which prints a
//! night's par and a way of keeping it; use it before changing one.

use super::night::Level;

/// Every night of the game.
pub const NIGHTS: [Level; 8] = [
    // One shade, and room to walk. It comes for whoever is nearest; be
    // nearest, and be somewhere else by the time it arrives.
    Level {
        rows: &["....s..", ".......", "..1....", ".3F2...", "..4...."],
        hours: 6,
        embers: 1,
        par: 1,
    },
    // Two, from opposite corners. One keeper cannot be in both places, so
    // the hearth goes out, and the night is lived again with two.
    Level {
        rows: &[
            ".......", ".s.....", "...3...", "..2F1..", "...4...", ".....s.", ".......",
        ],
        hours: 6,
        embers: 1,
        par: 2,
    },
    // Three, but from one side: a crowd follows one lantern as readily as
    // one shade does.
    Level {
        rows: &[
            ".......", ".....1.", "...2F..", "....34.", ".......", "s......", "s.s....",
        ],
        hours: 7,
        embers: 1,
        par: 1,
    },
    // A second ember. The hearth can take one shade and stay lit, which
    // makes the question which one to let it have.
    Level {
        rows: &[
            ".......", "......s", "...2...", "s.4F1..", "...3...", "......s", ".......",
        ],
        hours: 7,
        embers: 2,
        par: 1,
    },
    // Trees. A shade walks straight at its light and does not go round;
    // the one behind the tree is no trouble until somebody wakes it.
    Level {
        rows: &[
            "s......", ".......", "...2...", "s#3F1..", "...4...", ".......", ".......",
        ],
        hours: 7,
        embers: 1,
        par: 1,
    },
    // The same, with three more coming from below and less room to lead
    // them in without passing the sleeper.
    Level {
        rows: &[
            ".......", "..#.#..", "....3..", "s#2F4..", "..1....", "s......", "...s...",
        ],
        hours: 8,
        embers: 1,
        par: 1,
    },
    // Five and two embers. Nobody can hold the whole south edge alone.
    Level {
        rows: &[
            ".......", "#.2.3..", "...F...", "s..41..", "....#..", ".......", "s.s.s.s",
        ],
        hours: 8,
        embers: 2,
        par: 2,
    },
    // Six, from everywhere, and a long night.
    Level {
        rows: &[
            ".s.s...", ".......", "s#.....", "#.2F1..", "..34.#s", ".......", ".s..s..",
        ],
        hours: 9,
        embers: 1,
        par: 3,
    },
];

#[cfg(test)]
mod tests {
    use super::super::night::{Ground, MAX_KEEPERS, Step, kept};
    use super::super::solve;
    use super::*;

    /// One way of keeping each night with exactly its par and every
    /// lantern lit: all three marks.
    /// A slash separates the keepers; see [`Step::letter`].
    const WAYS: [&str; NIGHTS.len()] = [
        "...EEE",
        "..ENNN/..WEWE",
        ".SSSENN",
        "...EEWE",
        "...N.EE",
        "SEENEENN",
        "WSWWWNNE/..EESSWE",
        ".NEEWEWEW/.WSWEWENE/E.EWEWEWE",
    ];

    fn tapes(way: &str) -> Vec<Vec<Step>> {
        way.split('/')
            .map(|tape| {
                tape.chars()
                    .map(|c| Step::from_letter(c).expect("a step letter"))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn every_night_is_well_formed() {
        for (i, level) in NIGHTS.iter().enumerate() {
            let ground = Ground::parse(level.rows);
            assert_eq!(ground.beds.len(), MAX_KEEPERS, "night {}", i + 1);
            assert!(!ground.shades.is_empty(), "night {} is empty", i + 1);
            assert!(level.embers >= 1 && level.embers <= 3, "night {}", i + 1);
            assert!(
                (5..=9).contains(&level.hours),
                "night {} is {} hours; nights are short",
                i + 1,
                level.hours
            );
            assert!(
                usize::from(level.par) < MAX_KEEPERS,
                "night {} leaves no bedroll spare",
                i + 1
            );
            assert!(ground.width <= 9 && ground.height <= 7, "night {}", i + 1);
        }
    }

    #[test]
    fn every_night_can_be_kept_at_par() {
        for (i, (level, way)) in NIGHTS.iter().zip(WAYS).enumerate() {
            let tapes = tapes(way);
            assert_eq!(tapes.len(), usize::from(level.par), "night {}", i + 1);
            assert!(kept(level, &tapes), "night {}'s way does not work", i + 1);
            let mut watch = super::super::night::Watch::new(*level);
            for (keeper, tape) in tapes.iter().enumerate() {
                if keeper > 0 {
                    // Earlier tellings need not go well; pad them to dawn.
                    while watch.phase() == super::super::night::Phase::Night {
                        watch.step(Step::Wait);
                    }
                    if !watch.again() {
                        break;
                    }
                }
                for &step in tape {
                    watch.step(step);
                }
            }
            while watch.phase() == super::super::night::Phase::Night {
                watch.step(Step::Wait);
            }
            assert_eq!(
                watch.phase(),
                super::super::night::Phase::Kept { marks: 3 },
                "night {} played one keeper at a time",
                i + 1
            );
        }
    }

    #[test]
    fn no_night_can_be_kept_under_par() {
        for (i, level) in NIGHTS.iter().enumerate() {
            let under = usize::from(level.par) - 1;
            if under > 0 {
                assert!(
                    solve::solve(level, under).is_none(),
                    "night {} can be kept with {under}",
                    i + 1
                );
            }
            assert!(
                solve::solve_as(level, usize::from(level.par), true).is_some(),
                "night {} cannot be kept at par without losing a lantern",
                i + 1
            );
        }
    }

    #[test]
    fn no_night_keeps_itself() {
        // Standing still loses, with as many keepers as the night needs:
        // every night has to be played. (Enough bodies in the way can buy
        // a night by themselves, and that costs every mark but one.)
        for (i, level) in NIGHTS.iter().enumerate() {
            for keepers in 1..=usize::from(level.par) {
                assert!(
                    !kept(level, &vec![Vec::new(); keepers]),
                    "night {} is kept by {keepers} doing nothing",
                    i + 1
                );
            }
        }
    }

    #[test]
    fn the_first_night_forgives_and_the_last_does_not() {
        let first = solve::luck(&NIGHTS[0], 1, 2000, 1);
        assert!(first > 0.25, "stumbling keeps the first night {first}");
        let last = &NIGHTS[NIGHTS.len() - 1];
        let stumbling = solve::luck(last, usize::from(last.par), 2000, 1);
        assert!(
            stumbling < 0.01,
            "stumbling keeps the last night {stumbling}"
        );
    }
}
