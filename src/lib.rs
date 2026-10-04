//! Cassidy's game prototypes: a menu and the toys behind it.
//!
//! The split every prototype here keeps: the *simulation* is a pure,
//! deterministic, dependency-light library, and the binary is a thin
//! macroquad shell that turns input into a frame struct and state into
//! shapes and sound. No macroquad types appear anywhere in this library, so
//! the interesting half of each toy runs headless in `cargo test` and
//! `cargo bench` at whatever speed the CPU allows.
//!
//! - [`shell`] is the menu's own state: which prototype it is pointing at.
//! - [`leitmotif`] is the first prototype, a game whose rules are set by its
//!   music.
//! - [`sand_nomad`] is a trading circuit of a dry ocean, where everything
//!   you grow attached to weighs on you.
//! - [`space_trucking`] is an ambient game of hauling cargo across the solar
//!   system, built to be played in the background.
//! - [`cooking`] is not a prototype but a reusable, copy-pasteable cooking
//!   system: foods, processes, recipes and a station state machine.
//! - [`kitchen_garden`] is its demo: a day of cooking from a garden for
//!   whoever comes to the hatch.

pub mod cooking;
pub mod kitchen_garden;
pub mod leitmotif;
pub mod sand_nomad;
pub mod shell;
pub mod space_trucking;

/// `git describe` version, embedded by `build.rs`.
pub const VERSION: &str = env!("GIT_DESCRIBE_VERSION");
