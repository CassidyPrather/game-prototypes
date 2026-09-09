//! Cues in, sound out. The audio half of the frontend.
//!
//! This is the counterpart to `draw`: the library's `leitmotif::sim` says
//! *what* happened, `leitmotif::mix` says what that sounds like and how loud,
//! and this file is the part that needs a live audio context. It bakes one
//! buffer per [`Voice`] up front — macroquad cannot pitch a sound, so every
//! pitch the song uses is its own buffer — and a cue becomes a lookup and a
//! `play_sound`.
//!
//! Notes fire on the frame the sim crosses a step, so timing carries up to
//! a frame of jitter. That is the accepted cost of keeping the sequencer
//! inside the deterministic sim rather than on an audio thread; the offline
//! mix tests leave headroom for it.
//!
//! ## The autoplay problem
//!
//! Browsers start every page's audio context suspended and only resume it
//! inside a real user gesture. quad-snd's `audio.js` hooks the first
//! mousedown or keydown to do the resuming, and the sim independently waits
//! on the title screen for a first press before the song starts — so by the
//! time there is anything to play, the context is awake.

use std::collections::HashMap;

use game_prototypes::leitmotif::mix::{self, Voice};
use game_prototypes::leitmotif::sim::{Cue, InputFrame};
use macroquad::audio::{self, PlaySoundParams, Sound};

/// The loaded sound bank plus the state that shapes playback.
pub struct Audio {
    bank: HashMap<Voice, Sound>,
    /// Whether the user has pressed something yet.
    awake: bool,
    muted: bool,
}

impl Audio {
    /// Synthesise and decode the whole bank.
    ///
    /// Async because on the web each buffer goes to the browser to decode and
    /// macroquad waits frames for the result. A few dozen buffers means a
    /// few dozen frames at startup.
    pub async fn load() -> Self {
        let mut bank = HashMap::new();
        for voice in mix::all_voices() {
            let sound = audio::load_sound_from_bytes(&mix::render(voice))
                .await
                .expect("synth output is a WAV the backend just accepted");
            bank.insert(voice, sound);
        }
        Self {
            bank,
            awake: false,
            muted: false,
        }
    }

    /// One frame: handle the mute key and play everything the sim cued.
    pub fn update(&mut self, cues: &[Cue], input: &InputFrame, toggle_mute: bool) {
        if toggle_mute {
            self.muted = !self.muted;
        }
        // Only presses count; a browser will not resume audio for mouse
        // movement, and neither will we.
        if input.start || toggle_mute || input.restart.is_some() {
            self.awake = true;
        }
        if self.muted {
            return;
        }
        for &cue in cues {
            let Some((voice, volume)) = mix::voice_for(cue) else {
                continue;
            };
            let Some(sound) = self.bank.get(&voice) else {
                continue;
            };
            audio::play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume,
                },
            );
        }
    }

    /// Whether we are still waiting on a first press.
    #[must_use]
    pub const fn needs_gesture(&self) -> bool {
        !self.awake
    }

    /// Whether the player has silenced everything.
    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }
}
