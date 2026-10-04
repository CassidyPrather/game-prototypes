//! Kitchen Garden's sounds: baked once, played on cue, and two loops — the
//! garden outside, and the hearth whenever it is lit.
//!
//! What they sound like and how loud is the library's
//! `kitchen_garden::sfx`; this is only the part that needs a live audio
//! context. Nothing plays until the player has clicked, because browsers
//! hold audio until a real gesture and then release everything queued at
//! once, and the loops start at zero volume so nothing arrives mid-note.

use std::collections::HashMap;

use game_prototypes::kitchen_garden::sfx::{self, Sfx};
use macroquad::audio::{self, PlaySoundParams, Sound};

const LOOPS: [Sfx; 2] = [Sfx::Garden, Sfx::Fire];

pub struct Audio {
    bank: HashMap<Sfx, Sound>,
    awake: bool,
    muted: bool,
    /// Sounds waiting their turn: seconds to wait, and what.
    queue: Vec<(f32, Sfx)>,
    /// Whether the loops are playing.
    looping: bool,
    /// The hearth loop's level, eased toward the fire.
    fire: f32,
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
            looping: false,
            fire: 0.0,
        }
    }

    /// The player pressed something; sound may start.
    pub const fn wake(&mut self) {
        self.awake = true;
    }

    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.stop_loops();
        }
    }

    /// Stop the loops and forget queued sounds, for leaving to the menu.
    /// Nothing sets the loops' volume while the game is not updated, so
    /// left alone they would play on under the menu.
    pub fn hush(&mut self) {
        self.stop_loops();
        self.queue.clear();
    }

    fn stop_loops(&mut self) {
        if self.looping {
            for sfx in LOOPS {
                if let Some(sound) = self.bank.get(&sfx) {
                    audio::stop_sound(sound);
                }
            }
        }
        self.looping = false;
    }

    fn start_loops(&mut self) {
        if self.looping || self.muted || !self.awake {
            return;
        }
        for sfx in LOOPS {
            if let Some(sound) = self.bank.get(&sfx) {
                audio::play_sound(
                    sound,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.0,
                    },
                );
            }
        }
        self.looping = true;
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

    /// Let queued sounds whose time has come play, and set the loops: the
    /// garden always, the hearth as hot as `fire` (`0..=1`).
    pub fn update(&mut self, dt: f32, fire: f32, title: bool) {
        self.start_loops();
        let want = if fire > 0.0 { 1.0 } else { 0.0 };
        self.fire = (want - self.fire).mul_add((dt * 2.5).min(1.0), self.fire);
        if self.looping {
            let garden = if title { 0.6 } else { 1.0 };
            if let Some(sound) = self.bank.get(&Sfx::Garden) {
                audio::set_sound_volume(sound, Sfx::Garden.gain() * garden);
            }
            if let Some(sound) = self.bank.get(&Sfx::Fire) {
                audio::set_sound_volume(sound, Sfx::Fire.gain() * self.fire);
            }
        }
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
