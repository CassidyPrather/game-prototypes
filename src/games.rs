//! The prototypes the shell can launch, and the little they have in common.
//!
//! Each prototype is a module here with a `Game` and an `emblem`. The
//! library's [`GameId`] is the list; these two matches are where a new one
//! gets wired in.

pub mod kitchen_garden;
pub mod leitmotif;
pub mod sand_nomad;
pub mod space_trucking;

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

    /// The player has left for the menu. The prototype stays in memory and
    /// will be updated again if they come back, but until then nothing
    /// calls it: this is the moment to stop anything that would otherwise
    /// go on sounding. Most have nothing to do.
    fn leave(&mut self) {}
}

/// Start a prototype.
///
/// Async because a prototype's sound bank is synthesised here and decoded by
/// the browser over several frames; the shell keeps what this returns so
/// that only happens once.
pub async fn load(id: GameId) -> Box<dyn Game> {
    match id {
        GameId::Leitmotif => Box::new(leitmotif::Game::load().await),
        GameId::SandNomad => Box::new(sand_nomad::Game::load().await),
        GameId::SpaceTrucking => Box::new(space_trucking::Game::load().await),
        GameId::KitchenGarden => Box::new(kitchen_garden::Game::load().await),
    }
}

/// Draw a prototype's mark, `width` frame units across and centred on
/// `centre`, without loading it. The menu is a picture book, so every
/// prototype draws its own page.
pub fn emblem(id: GameId, frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    match id {
        GameId::Leitmotif => leitmotif::emblem(frame, centre, width, clock),
        GameId::SandNomad => sand_nomad::emblem(frame, centre, width, clock),
        GameId::SpaceTrucking => space_trucking::emblem(frame, centre, width, clock),
        GameId::KitchenGarden => kitchen_garden::emblem(frame, centre, width, clock),
    }
}
