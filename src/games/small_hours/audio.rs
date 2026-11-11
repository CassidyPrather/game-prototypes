//! Small Hours' sounds: baked once, played on cue, and two loops — the
//! dark, and the hearth while it has embers.
//!
//! What they sound like and how loud is the library's `small_hours::sfx`;
//! this is only the part that needs a live audio context. Nothing plays
//! until the player has pressed something, because browsers hold audio
//! until a real gesture and then release everything queued at once, and the
//! loops start at zero volume so nothing arrives mid-note.

use std::collections::HashMap;

use game_prototypes::small_hours::sfx::{self, Sfx};
use macroquad::audio::{self, PlaySoundParams, Sound};

const LOOPS: [Sfx; 2] = [Sfx::Night, Sfx::Fire];

pub struct Audio {
    bank: HashMap<Sfx, Sound>,
    awake: bool,
    muted: bool,
    /// Sounds waiting their turn: seconds to wait, and what.
    queue: Vec<(f32, Sfx)>,
    looping: bool,
    /// The hearth loop's level, eased toward the fire.
    fire: f32,
    /// The dark's level, eased down as the sky lightens.
    dark: f32,
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
            dark: 0.0,
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
            self.hush();
        }
    }

    /// Stop the loops and forget queued sounds, for leaving to the menu.
    pub fn hush(&mut self) {
        if self.looping {
            for sfx in LOOPS {
                if let Some(sound) = self.bank.get(&sfx) {
                    audio::stop_sound(sound);
                }
            }
        }
        self.looping = false;
        self.queue.clear();
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
        if delay <= 0.0 {
            self.play(sfx);
        } else {
            self.queue.push((delay, sfx));
        }
    }

    /// Let queued sounds whose time has come play, and set the loops: the
    /// hearth as high as `fire` and the dark as deep as `dark`, both
    /// `0..=1`.
    pub fn update(&mut self, dt: f32, fire: f32, dark: f32) {
        self.start_loops();
        let ease = (dt * 3.0).min(1.0);
        self.fire = (fire - self.fire).mul_add(ease, self.fire);
        self.dark = (dark - self.dark).mul_add(ease, self.dark);
        if self.looping {
            for (sfx, level) in [(Sfx::Night, self.dark), (Sfx::Fire, self.fire)] {
                if let Some(sound) = self.bank.get(&sfx) {
                    audio::set_sound_volume(sound, sfx.gain() * level);
                }
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
