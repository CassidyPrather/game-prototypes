//! Sand Nomad: a circuit of the Grand Basin, carrying as little as you can.
//!
//! The basin is the dry bed of the Azure Steppe's lost ocean, sailed by
//! sand ships. Its nomads believe that anything someone grows attached to
//! gathers *weight*, and that a thing with too much of it wakes up, which
//! is a nuisance. So they offload heavy things on anyone who will take
//! them, burn the worst to their ships, and trade with the Crimson Harbor,
//! whose people do not believe in any of it.
//!
//! You walk the circuit: every waystone in the basin wants an offering of
//! its motive, and then you sail home. The fewer days you sail and the
//! lighter you travel, the less it all weighs on you.
//!
//! Like every prototype here the whole game is a pure, deterministic
//! library, and the binary's `games::sand_nomad` is a thin macroquad
//! shell.
//!
//! - [`art`] — the pixel art, as data.
//! - [`motive`] — the six motives and their three opposing axes.
//! - [`item`] — things, their weight, and what it does to them.
//! - [`grid`] — the hold: packing, and how neighbours feed or quarrel.
//! - [`barter`] — the basin's cultures and how each prices a thing.
//! - [`world`] — the hand-made basin: places, waystones, rugs.
//! - [`history`] — the era's grand events, going on without you.
//! - [`journey`] — the rules, as a state machine the frontend drives.
//! - [`sfx`] — what it all sounds like, synthesised.

pub mod art;
pub mod barter;
pub mod grid;
pub mod history;
pub mod item;
pub mod journey;
pub mod motive;
pub mod sfx;
pub mod world;
