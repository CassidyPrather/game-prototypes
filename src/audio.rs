//! Cues in, sound out. The audio half of the frontend.
//!
//! This is the counterpart to `draw`: [`leitmotif::sim`] says *what*
//! happened, and everything here decides what that sounds like. The
//! waveforms come from [`leitmotif::synth`], which stays macroquad-free so
//! it can be unit-tested; this file is the part that needs a live audio
//! context.
//!
//! The music is the interesting part. macroquad cannot pitch a sound, so at
//! load this bakes one buffer per pitch each instrument uses anywhere in the
//! song — [`leitmotif::song::pitches`] and [`leitmotif::song::chords`] say
//! which — and a note cue is a lookup plus a `play_sound`. Notes fire on the
//! frame the sim crosses a step, so timing carries up to a frame of jitter;
//! that is the accepted cost of keeping the sequencer inside the
//! deterministic sim rather than on an audio thread.
//!
//! ## The autoplay problem
//!
//! Browsers start every page's audio context suspended and only resume it
//! inside a real user gesture. quad-snd's `audio.js` hooks the first
//! mousedown or keydown to do the resuming, and the sim independently waits
//! on the title screen for a first press before the song starts — so by the
//! time there is anything to play, the context is awake.

use std::collections::HashMap;

use leitmotif::sim::{Cue, InputFrame};
use leitmotif::song::{self, Chord, HAT, Instrument, KICK, SNARE};
use leitmotif::synth;
use macroquad::audio::{self, PlaySoundParams, Sound};

/// Peak gain per drum, at full velocity.
const KICK_GAIN: f32 = 0.85;
const SNARE_GAIN: f32 = 0.55;
const HAT_GAIN: f32 = 0.28;

/// Peak gain for the pitched layers, at full velocity.
const BASS_GAIN: f32 = 0.6;
const LEAD_GAIN: f32 = 0.5;
const PAD_GAIN: f32 = 0.55;

/// Gain for the game's own sounds.
const PICKUP_GAIN: f32 = 0.45;
const HIT_GAIN: f32 = 0.9;
const UI_GAIN: f32 = 0.35;

/// The loaded sound bank plus the state that shapes playback.
pub struct Audio {
    drums: HashMap<u8, Sound>,
    bass: HashMap<u8, Sound>,
    lead: HashMap<u8, Sound>,
    pads: HashMap<Chord, Sound>,
    chime: Sound,
    thump: Sound,
    blip_up: Sound,
    blip_down: Sound,
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
        let mut drums = HashMap::new();
        for pitch in song::pitches(Instrument::Drums) {
            let wav = match pitch {
                KICK => synth::kick(),
                SNARE => synth::snare(),
                _ => synth::hat(),
            };
            drums.insert(pitch, bake(&wav).await);
        }
        let mut bass = HashMap::new();
        for pitch in song::pitches(Instrument::Bass) {
            bass.insert(pitch, bake(&synth::bass(song::hertz(pitch))).await);
        }
        let mut lead = HashMap::new();
        for pitch in song::pitches(Instrument::Lead) {
            lead.insert(pitch, bake(&synth::pluck(song::hertz(pitch))).await);
        }
        let mut pads = HashMap::new();
        for chord in song::chords() {
            let hz: Vec<f32> = chord.notes.iter().map(|&n| song::hertz(n)).collect();
            pads.insert(chord, bake(&synth::pad(&hz)).await);
        }
        Self {
            drums,
            bass,
            lead,
            pads,
            chime: bake(&synth::chime()).await,
            thump: bake(&synth::thump()).await,
            blip_up: bake(&synth::blip(true)).await,
            blip_down: bake(&synth::blip(false)).await,
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
        for cue in cues {
            self.play(*cue);
        }
    }

    /// Whether we are still waiting on a first press, which is the one
    /// audio state worth nagging the user about.
    #[must_use]
    pub const fn needs_gesture(&self) -> bool {
        !self.awake
    }

    /// HUD text for the current state.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        if self.muted {
            "muted - M to unmute"
        } else if self.awake {
            "on - M to mute"
        } else {
            "press or click to start"
        }
    }

    /// Fire one cue.
    fn play(&self, cue: Cue) {
        if self.muted {
            return;
        }
        let (sound, volume) = match cue {
            Cue::Note {
                instrument,
                pitch,
                velocity,
            } => {
                let Some((sound, gain)) = self.voice(instrument, pitch) else {
                    return;
                };
                (sound, gain * loudness(velocity))
            }
            Cue::Chord(chord) => {
                let Some(sound) = self.pads.get(&chord) else {
                    return;
                };
                (sound, PAD_GAIN)
            }
            Cue::Pickup { value } => {
                // A little louder the more it was worth, within reason.
                let bonus = (value as f32 * 0.03).min(0.25);
                (&self.chime, PICKUP_GAIN + bonus)
            }
            Cue::Hit { .. } => (&self.thump, HIT_GAIN),
            Cue::Layer { on: true, .. } | Cue::Pause { paused: false } | Cue::Restart => {
                (&self.blip_up, UI_GAIN)
            }
            Cue::Layer { on: false, .. } | Cue::Pause { paused: true } => {
                (&self.blip_down, UI_GAIN)
            }
            // The music marks its own section changes.
            Cue::Section(_) => return,
        };
        audio::play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }

    /// The buffer for a note, and the gain its instrument sits at.
    fn voice(&self, instrument: Instrument, pitch: u8) -> Option<(&Sound, f32)> {
        match instrument {
            Instrument::Drums => {
                let gain = match pitch {
                    KICK => KICK_GAIN,
                    SNARE => SNARE_GAIN,
                    HAT => HAT_GAIN,
                    _ => return None,
                };
                self.drums.get(&pitch).map(|s| (s, gain))
            }
            Instrument::Bass => self.bass.get(&pitch).map(|s| (s, BASS_GAIN)),
            Instrument::Lead => self.lead.get(&pitch).map(|s| (s, LEAD_GAIN)),
            // Pad notes arrive as chords, not as single pitches.
            Instrument::Pad => None,
        }
    }
}

/// Map a cue velocity onto a gain multiplier.
///
/// Amplitude and perceived loudness are not the same thing: halving amplitude
/// is nothing like halving loudness. The square root pulls quiet events up
/// where they can still be heard, and the floor keeps the smallest ones from
/// vanishing entirely.
fn loudness(velocity: f32) -> f32 {
    velocity.clamp(0.0, 1.0).sqrt().mul_add(0.75, 0.25)
}

/// Hand one synthesised WAV to the audio backend.
async fn bake(wav: &[u8]) -> Sound {
    audio::load_sound_from_bytes(wav)
        .await
        .expect("synth output is a WAV the backend just accepted")
}
