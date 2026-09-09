//! The prototypes the shell can launch, and the little they have in common.
//!
//! Each prototype is a module here with a `Game` and an `emblem`. The
//! library's [`GameId`] is the list; these two matches are where a new one
//! gets wired in.

pub mod leitmotif;

use game_prototypes::shell::GameId;

use crate::ui::{Frame, MqVec2};

/// What the shell needs of a prototype.
///
/// Deliberately thin: a prototype gathers its own input and owns its own
/// camera, palette and sound. The shell only decides *when* it runs and
/// hands it the frame to paint in.
pub trait Game {
    /// One frame of play, `dt` seconds long.
    fn update(&mut self, dt: f32);

    /// Paint inside the shell's letterboxed frame.
    fn draw(&self, frame: &Frame);
}

/// Start a prototype.
///
/// Async because a prototype's sound bank is synthesised here and decoded by
/// the browser over several frames; the shell keeps what this returns so
/// that only happens once.
pub async fn load(id: GameId) -> Box<dyn Game> {
    match id {
        GameId::Leitmotif => Box::new(leitmotif::Game::load().await),
    }
}

/// Draw a prototype's mark, `width` frame units across and centred on
/// `centre`, without loading it. The menu is a picture book, so every
/// prototype draws its own page.
pub fn emblem(id: GameId, frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    match id {
        GameId::Leitmotif => leitmotif::emblem(frame, centre, width, clock),
    }
}
