//! Small Hours: keep one fire lit through one night, with nobody to help
//! but yourself, again.
//!
//! - [`night`] is the rules: the grid, the keepers, the shades, and the
//!   night lived once per keeper.
//! - [`nights`] is the eight nights, hand-made.
//! - [`solve`] is the exhaustive search that proves each night's par.
//! - [`sfx`] is what it sounds like.
//!
//! Design, reasoning and open questions: `docs/SMALL_HOURS.md`.

pub mod night;
pub mod nights;
pub mod sfx;
pub mod solve;
