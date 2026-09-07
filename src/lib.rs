//! Leitmotif: a small game whose rules are set by its music.
//!
//! The split the template exists to demonstrate: [`sim`] is the whole game
//! as a pure, deterministic, dependency-light library, and the binary is a
//! thin macroquad shell that turns input into [`sim::InputFrame`]s and state
//! into circles and sound. [`song`] is the score and the sequencer that
//! walks it — the sim's source of events, and the frontend's list of what
//! to bake. No macroquad types appear anywhere in here, so the interesting
//! half runs headless in `cargo test` and `cargo bench` at whatever speed
//! the CPU allows.

pub mod sim;
pub mod song;
pub mod synth;

/// `git describe` version, embedded by `build.rs`.
pub const VERSION: &str = env!("GIT_DESCRIBE_VERSION");
