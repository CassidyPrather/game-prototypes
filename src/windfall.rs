//! Windfall: blow a tree's leaves home, each into the basket of its colour.
//!
//! - [`garden`] is the rules: the grid, the leaves, the vanes, the marks.
//! - [`levels`] is the gardens, hand-made, in order.
//! - [`solve`] is the exhaustive search that proves each garden's par.
//! - [`sound`] is which sample answers which cue, and how loud.
//!
//! Design, reasoning and open questions: `docs/WINDFALL.md`.

pub mod garden;
pub mod levels;
pub mod solve;
pub mod sound;
