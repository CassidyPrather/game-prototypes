//! Kitchen Garden: one day of cooking from a garden, for whoever comes to
//! the hatch.
//!
//! The demo of the [`cooking`](crate::cooking) module, and the argument for
//! it: the module only knows foods, processes, recipes and stations, and
//! this puts the things a game would around them — somewhere for raw
//! ingredients to come from, at a cost; somewhere for dishes to go, for a
//! price; and never quite enough time, room, fuel, coins or hands.
//!
//! - [`day`] — the rules, as a deterministic state machine the frontend
//!   drives.
//! - [`prices`] — what dishes pay and how long customers wait, derived
//!   from the cooking module's bills rather than set by hand.
//! - [`sfx`] — what it all sounds like, synthesised.

pub mod day;
pub mod prices;
pub mod sfx;
