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

pub mod leitmotif;
pub mod shell;

/// `git describe` version, embedded by `build.rs`.
pub const VERSION: &str = env!("GIT_DESCRIBE_VERSION");
