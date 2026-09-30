//! The menu's sounds, baked once and played on demand.
//!
//! What they sound like and how loud is the library's
//! [`game_prototypes::shell::sfx`]; this is only the part that needs a live
//! audio context.
//!
//! Nothing plays until the player has pressed something. Browsers hold the
//! audio context suspended until a real gesture and then start everything
//! queued against it at once, so hover ticks from before the first click
//! would all land together the moment it came.

use std::collections::HashMap;

use game_prototypes::shell::sfx::{self, Sfx};
use macroquad::audio::{self, PlaySoundParams, Sound};

pub struct Sounds {
    bank: HashMap<Sfx, Sound>,
    awake: bool,
}

impl Sounds {
    /// Synthesise and decode every menu sound. A handful of short buffers,
    /// so a few frames at startup on the web.
    pub async fn load() -> Self {
        let mut bank = HashMap::new();
        for sfx in Sfx::ALL {
            let sound = audio::load_sound_from_bytes(&sfx::render(sfx))
                .await
                .expect("synth output is a WAV the backend accepts");
            bank.insert(sfx, sound);
        }
        Self { bank, awake: false }
    }

    /// The player pressed something; sounds may play from now on.
    pub const fn wake(&mut self) {
        self.awake = true;
    }

    pub fn play(&self, sfx: Sfx) {
        if !self.awake {
            return;
        }
        if let Some(sound) = self.bank.get(&sfx) {
            audio::play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: sfx.gain(),
                },
            );
        }
    }
}
