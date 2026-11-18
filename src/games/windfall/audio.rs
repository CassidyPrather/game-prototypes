//! Windfall's sounds: CC0 recordings, embedded, decoded once, played on
//! cue, and the garden's air looped under everything.
//!
//! Which sample answers which cue, and how loud, is the library's
//! `windfall::sound`; this is only the part that needs a live audio
//! context. Nothing plays until the player has pressed something, because
//! browsers hold audio until a real gesture, and the loop starts at zero
//! volume and eases in so nothing arrives mid-breath.

use std::collections::HashMap;

use game_prototypes::windfall::sound::Sample;
use macroquad::audio::{self, PlaySoundParams, Sound};

/// The bytes of each sample. Every file is credited in `CREDITS.md`, with
/// its processing in `assets/windfall/SOURCES.md`.
const fn bytes(sample: Sample) -> &'static [u8] {
    match sample {
        Sample::Release(0) => include_bytes!("../../../assets/windfall/leaf_release_1.ogg"),
        Sample::Release(_) => include_bytes!("../../../assets/windfall/leaf_release_2.ogg"),
        Sample::Gust(0) => include_bytes!("../../../assets/windfall/gust_1.ogg"),
        Sample::Gust(_) => include_bytes!("../../../assets/windfall/gust_2.ogg"),
        Sample::Flip => include_bytes!("../../../assets/windfall/flip.ogg"),
        Sample::Catch => include_bytes!("../../../assets/windfall/catch.ogg"),
        Sample::Miss => include_bytes!("../../../assets/windfall/miss.ogg"),
        Sample::Place => include_bytes!("../../../assets/windfall/place.ogg"),
        Sample::Remove => include_bytes!("../../../assets/windfall/remove.ogg"),
        Sample::Solved => include_bytes!("../../../assets/windfall/solved.ogg"),
        Sample::Mark => include_bytes!("../../../assets/windfall/mark.ogg"),
        Sample::Ambience => include_bytes!("../../../assets/windfall/ambience.ogg"),
    }
}

pub struct Audio {
    bank: HashMap<Sample, Sound>,
    awake: bool,
    muted: bool,
    /// Sounds waiting their turn: seconds to wait, and what.
    queue: Vec<(f32, Sample)>,
    looping: bool,
    /// The air's level, eased toward full once sound may start.
    air: f32,
}

impl Audio {
    pub async fn load() -> Self {
        let mut bank = HashMap::new();
        for sample in Sample::ALL {
            // A sample the platform cannot decode is left out rather than
            // stopping the game; it simply stays silent.
            if let Ok(sound) = audio::load_sound_from_bytes(bytes(sample)).await {
                bank.insert(sample, sound);
            }
        }
        Self {
            bank,
            awake: false,
            muted: false,
            queue: Vec::new(),
            looping: false,
            air: 0.0,
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

    /// Stop the loop and forget queued sounds, for leaving to the menu.
    pub fn hush(&mut self) {
        if self.looping
            && let Some(sound) = self.bank.get(&Sample::Ambience)
        {
            audio::stop_sound(sound);
        }
        self.looping = false;
        self.air = 0.0;
        self.queue.clear();
    }

    pub fn play(&self, sample: Sample) {
        if !self.awake || self.muted {
            return;
        }
        if let Some(sound) = self.bank.get(&sample) {
            audio::play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: sample.gain(),
                },
            );
        }
    }

    /// Play `sample` after `delay` seconds.
    pub fn later(&mut self, delay: f32, sample: Sample) {
        if delay <= 0.0 {
            self.play(sample);
        } else {
            self.queue.push((delay, sample));
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.looping
            && self.awake
            && !self.muted
            && let Some(sound) = self.bank.get(&Sample::Ambience)
        {
            audio::play_sound(
                sound,
                PlaySoundParams {
                    looped: true,
                    volume: 0.0,
                },
            );
            self.looping = true;
        }
        if self.looping {
            self.air = dt.mul_add(0.4, self.air).min(1.0);
            if let Some(sound) = self.bank.get(&Sample::Ambience) {
                audio::set_sound_volume(sound, Sample::Ambience.gain() * self.air);
            }
        }
        let mut due = Vec::new();
        self.queue.retain_mut(|(wait, sample)| {
            *wait -= dt;
            if *wait <= 0.0 {
                due.push(*sample);
                false
            } else {
                true
            }
        });
        for sample in due {
            self.play(sample);
        }
    }
}
