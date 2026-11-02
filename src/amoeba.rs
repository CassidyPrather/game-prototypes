//! Amoeba Roguelike Remastered: a Rust remaster of the 2021 7DRL roguelike.
//!
//! Play as a giant, constantly evolving amoeba; engulf and digest the humans
//! defending their cities, craft organelles from what is left of them, and
//! break the gates to escape to the surface. Overview and history in
//! `docs/AMOEBA.md`; the complete spec the remaster was built from, quirks
//! kept on purpose included, in `docs/amoeba/PORT_SPEC.md`.
//!
//! The split is the one every prototype here keeps: [`sim`] is the whole
//! game as a pure, deterministic, dependency-light library, resolved all the
//! way down to a glyph and two colours per cell, and the binary's
//! `games::amoeba` is a thin macroquad shell that turns key presses and
//! thumbs into [`sim::Command`]s and a [`sim::RenderView`] into glyphs.
//!
//! [`synth`] is every sound, as WAV bytes computed from arithmetic.
//!
//! It arrived from its own repository (`CassidyPrather/amoeba_rl`) whole and
//! keeps the conventions it grew there; where the remaster departs from the
//! original, the code says so at the point of departure — search for
//! `DELIBERATE CHANGE`.

pub mod sim;
pub mod synth;
