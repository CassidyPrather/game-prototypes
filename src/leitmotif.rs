//! Leitmotif: a game whose rules are set by its music.
//!
//! [`sim`] is the whole game as a pure, deterministic library, and the
//! binary's `games::leitmotif` is a thin macroquad shell that turns input
//! into [`sim::InputFrame`]s and state into shapes and sound. [`song`] is
//! the score and the sequencer that walks it — the sim's source of events.
//! [`mix`] is the policy for what a cue sounds like, kept here so a test can
//! render the whole song and prove it never clips.

pub mod mix;
pub mod sim;
pub mod song;
pub mod synth;
