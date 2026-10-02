//! Sand Nomad's sounds: baked once, played on cue, and the wind.
//!
//! What they sound like and how loud is the library's
//! `sand_nomad::sfx`; this is only the part that needs a live audio
//! context. Nothing plays until the player has clicked, because browsers
//! hold audio until a real gesture and then release everything queued at
//! once.

use std::collections::HashMap;

use game_prototypes::sand_nomad::sfx::{self, Sfx};
use macroquad::audio::{self, PlaySoundParams, Sound};

pub struct Audio {
    bank: HashMap<Sfx, Sound>,
    awake: bool,
    muted: bool,
    /// Sounds waiting their turn: seconds to wait, and what.
    queue: Vec<(f32, Sfx)>,
    wind_on: bool,
}

impl Audio {
    pub async fn load() -> Self {
        let mut bank = HashMap::new();
        for sfx in Sfx::all() {
            let sound = audio::load_sound_from_bytes(&sfx::render(sfx))
                .await
                .expect("synth output is a WAV the backend accepts");
            bank.insert(sfx, sound);
        }
        Self {
            bank,
            awake: false,
            muted: false,
            queue: Vec::new(),
            wind_on: false,
        }
    }

    /// The player pressed something; sound may start.
    pub fn wake(&mut self) {
        if !self.awake {
            self.awake = true;
            self.start_wind();
        }
    }

    /// Stop the wind and forget sounds still waiting their turn, for the
    /// player leaving to the menu. Nothing sets the wind's volume while the
    /// game is not being updated, so left alone it would blow on under the
    /// menu, and mute could not reach it. [`Audio::update`] starts it again
    /// when the player is back.
    pub fn hush(&mut self) {
        if let (true, Some(wind)) = (self.wind_on, self.bank.get(&Sfx::Wind)) {
            audio::stop_sound(wind);
        }
        self.wind_on = false;
        self.queue.clear();
    }

    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            if let Some(wind) = self.bank.get(&Sfx::Wind) {
                audio::stop_sound(wind);
            }
            self.wind_on = false;
        } else {
            self.start_wind();
        }
    }

    fn start_wind(&mut self) {
        if self.wind_on || self.muted || !self.awake {
            return;
        }
        if let Some(wind) = self.bank.get(&Sfx::Wind) {
            audio::play_sound(
                wind,
                PlaySoundParams {
                    looped: true,
                    volume: Sfx::Wind.gain(),
                },
            );
            self.wind_on = true;
        }
    }

    /// How hard the wind blows, `0..=1` of its gain: stronger under sail.
    pub fn wind(&self, strength: f32) {
        if let (true, Some(wind)) = (self.wind_on, self.bank.get(&Sfx::Wind)) {
            audio::set_sound_volume(wind, Sfx::Wind.gain() * strength.clamp(0.0, 1.0));
        }
    }

    pub fn play(&self, sfx: Sfx) {
        if !self.awake || self.muted {
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

    /// Play `sfx` after `delay` seconds.
    pub fn later(&mut self, delay: f32, sfx: Sfx) {
        self.queue.push((delay, sfx));
    }

    /// Let queued sounds whose time has come play.
    pub fn update(&mut self, dt: f32) {
        // A no-op unless the wind has been stopped by `hush`: it is already
        // blowing, or muted, or audio has not woken.
        self.start_wind();
        let mut due = Vec::new();
        self.queue.retain_mut(|(wait, sfx)| {
            *wait -= dt;
            if *wait <= 0.0 {
                due.push(*sfx);
                false
            } else {
                true
            }
        });
        for sfx in due {
            self.play(sfx);
        }
    }
}
