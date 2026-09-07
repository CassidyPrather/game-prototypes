//! What a cue sounds like, and an offline mixer to prove the mix fits.
//!
//! The runtime audio backend (quad-snd natively, Web Audio in the browser)
//! sums every playing voice, scaled by its volume, and hands the total to
//! the device. Nothing in between limits it: a sum over full scale clips at
//! the DAC, and it clips worst exactly when the arena is busiest. So the
//! policy that maps a [`Cue`] to a voice and a gain lives here, in the
//! library, where a test can render the entire song through the same policy
//! and measure the peak. The frontend's `audio` module uses [`voice_for`]
//! and [`all_voices`] verbatim; if the tests pass, the speakers see a mix
//! that was measured.

use crate::sim::Cue;
use crate::song::{self, Chord, HAT, Instrument, KICK, SNARE};
use crate::synth;

/// Everything is scaled by this. Tuned so the busiest tick of the song, with
/// a pickup and a hit on top, still sits under full scale — see the tests.
pub const MASTER: f32 = 0.42;

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
const HEAL_GAIN: f32 = 0.7;
const HIT_GAIN: f32 = 0.9;
const HOME_GAIN: f32 = 0.8;
const UI_GAIN: f32 = 0.25;

/// One baked buffer. The frontend keeps a `Sound` per voice; the tests keep
/// samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Voice {
    /// A drum, by its MIDI note.
    Drum(u8),
    /// A bass note, by MIDI number.
    Bass(u8),
    /// A lead note, by MIDI number.
    Lead(u8),
    /// A pad chord.
    Pad(Chord),
    /// Pickups and reaching home.
    Chime,
    /// Taking a hit.
    Thump,
    /// Unpausing, unhushing, restarting.
    BlipUp,
    /// Pausing, hushing.
    BlipDown,
}

/// The policy: which voice a cue plays, and how loud. `None` for cues that
/// make no sound of their own — the music marks its own section changes.
#[must_use]
pub fn voice_for(cue: Cue) -> Option<(Voice, f32)> {
    let (voice, gain) = match cue {
        Cue::Note {
            instrument,
            pitch,
            velocity,
        } => {
            let (voice, gain) = match instrument {
                Instrument::Drums => (
                    Voice::Drum(pitch),
                    match pitch {
                        KICK => KICK_GAIN,
                        SNARE => SNARE_GAIN,
                        HAT => HAT_GAIN,
                        _ => return None,
                    },
                ),
                Instrument::Bass => (Voice::Bass(pitch), BASS_GAIN),
                Instrument::Lead => (Voice::Lead(pitch), LEAD_GAIN),
                // Pad notes arrive as chords, not as single pitches.
                Instrument::Pad => return None,
            };
            (voice, gain * loudness(velocity))
        }
        Cue::Chord(chord) => (Voice::Pad(chord), PAD_GAIN),
        Cue::Pickup { healed: false } => (Voice::Chime, PICKUP_GAIN),
        Cue::Pickup { healed: true } => (Voice::Chime, HEAL_GAIN),
        Cue::Hit { .. } => (Voice::Thump, HIT_GAIN),
        Cue::Home => (Voice::Chime, HOME_GAIN),
        Cue::Layer { on: true, .. } | Cue::Pause { paused: false } | Cue::Restart => {
            (Voice::BlipUp, UI_GAIN)
        }
        Cue::Layer { on: false, .. } | Cue::Pause { paused: true } => (Voice::BlipDown, UI_GAIN),
        Cue::Section(_) => return None,
    };
    Some((voice, gain * MASTER))
}

/// Every voice any cue can ask for, so the frontend can bake them all up
/// front. The song says which pitches and chords; the rest is fixed.
#[must_use]
pub fn all_voices() -> Vec<Voice> {
    let mut voices = vec![Voice::Chime, Voice::Thump, Voice::BlipUp, Voice::BlipDown];
    voices.extend(
        song::pitches(Instrument::Drums)
            .into_iter()
            .map(Voice::Drum),
    );
    voices.extend(song::pitches(Instrument::Bass).into_iter().map(Voice::Bass));
    voices.extend(song::pitches(Instrument::Lead).into_iter().map(Voice::Lead));
    voices.extend(song::chords().into_iter().map(Voice::Pad));
    voices
}

/// Render one voice to WAV bytes, exactly what the frontend hands the
/// backend.
#[must_use]
pub fn render(voice: Voice) -> Vec<u8> {
    match voice {
        Voice::Drum(KICK) => synth::kick(),
        Voice::Drum(SNARE) => synth::snare(),
        Voice::Drum(_) => synth::hat(),
        Voice::Bass(pitch) => synth::bass(song::hertz(pitch)),
        Voice::Lead(pitch) => synth::pluck(song::hertz(pitch)),
        Voice::Pad(chord) => {
            let hz: Vec<f32> = chord.notes.iter().map(|&n| song::hertz(n)).collect();
            synth::pad(&hz)
        }
        Voice::Chime => synth::chime(),
        Voice::Thump => synth::thump(),
        Voice::BlipUp => synth::blip(true),
        Voice::BlipDown => synth::blip(false),
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

/// An offline copy of what the backend does: sum scaled voices at the times
/// they start. Sample rate is [`synth::SAMPLE_RATE`].
#[derive(Clone, Debug, Default)]
pub struct Mixer {
    out: Vec<f32>,
}

impl Mixer {
    /// An empty mix.
    #[must_use]
    pub const fn new() -> Self {
        Self { out: Vec::new() }
    }

    /// Start `samples` at `at` seconds, scaled by `gain`.
    pub fn play(&mut self, at: f32, samples: &[f32], gain: f32) {
        // Start times are non-negative by construction.
        #[allow(clippy::cast_sign_loss)]
        let start = (at.max(0.0) * synth::SAMPLE_RATE as f32).round() as usize;
        let end = start + samples.len();
        if self.out.len() < end {
            self.out.resize(end, 0.0);
        }
        for (slot, sample) in self.out[start..end].iter_mut().zip(samples) {
            *slot = sample.mul_add(gain, *slot);
        }
    }

    /// The mix so far.
    #[must_use]
    pub fn samples(&self) -> &[f32] {
        &self.out
    }

    /// Loudest absolute sample, `0` for silence. Over `1` clips.
    #[must_use]
    pub fn peak(&self) -> f32 {
        self.out.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()))
    }
}

/// A peak as decibels below full scale, for reports.
#[must_use]
pub fn dbfs(peak: f32) -> f32 {
    20.0 * peak.max(1e-9).log10()
}

#[cfg(test)]
// Tests turn positive seconds into tick and sample counts.
#[allow(clippy::cast_sign_loss)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::sim::{InputFrame, Phase, Sim, TICK_DT, Vec2};
    use crate::song::{BARS_PER_SECTION, SONG, STEPS_PER_BAR};

    /// Loudest the mix may get. Below full scale by enough that a frame of
    /// runtime jitter cannot line peaks up that the offline render did not.
    const HEADROOM: f32 = 0.9;

    /// Quietest the loudest moment may be: headroom is for using.
    const FLOOR: f32 = 0.4;

    /// Every voice, decoded once.
    fn bank() -> HashMap<Voice, Vec<f32>> {
        all_voices()
            .into_iter()
            .map(|voice| (voice, synth::decode(&render(voice))))
            .collect()
    }

    /// Mix every cue of one sim frame at `at` seconds.
    fn mix_cues(mixer: &mut Mixer, bank: &HashMap<Voice, Vec<f32>>, at: f32, cues: &[Cue]) {
        for &cue in cues {
            if let Some((voice, gain)) = voice_for(cue) {
                let samples = bank.get(&voice).expect("every cue's voice is baked");
                mixer.play(at, samples, gain);
            }
        }
    }

    /// Seconds in one loop of the song.
    fn song_secs() -> f32 {
        SONG.iter()
            .map(|s| s.motif.bar_secs() * BARS_PER_SECTION as f32)
            .sum()
    }

    #[test]
    fn every_voice_bakes_and_every_cue_has_one() {
        let bank = bank();
        assert!(bank.len() > 20, "suspiciously few voices: {}", bank.len());
        for samples in bank.values() {
            assert!(!samples.is_empty());
        }

        // Play the whole song with everything on and check every cue finds
        // a baked voice, or is a cue that deliberately makes no sound.
        let mut sim = Sim::new(0xA0D10);
        sim.advance(
            0.0,
            &InputFrame {
                start: true,
                ..InputFrame::default()
            },
        );
        let ticks = ((song_secs() + 1.0) / TICK_DT) as u32;
        for _ in 0..ticks {
            sim.advance(TICK_DT, &InputFrame::default());
            for &cue in sim.cues() {
                match voice_for(cue) {
                    Some((voice, gain)) => {
                        assert!(bank.contains_key(&voice), "{cue:?} wants unbaked {voice:?}");
                        assert!(gain > 0.0 && gain <= 1.0, "{cue:?} at gain {gain}");
                    }
                    None => assert!(
                        matches!(cue, Cue::Section(_)),
                        "{cue:?} is silent by accident"
                    ),
                }
            }
        }
    }

    /// The whole song, every layer on, with a pickup and a hit forced onto
    /// every downbeat: the loudest the game can get in ordinary play, and
    /// then some.
    #[test]
    fn the_whole_song_fits_under_full_scale() {
        let bank = bank();
        let mut mixer = Mixer::new();
        let mut sim = Sim::new(0xA0D10);
        sim.advance(
            0.0,
            &InputFrame {
                start: true,
                ..InputFrame::default()
            },
        );
        let ticks = ((song_secs() + 2.0) / TICK_DT) as u32;
        let mut healed = false;
        for tick in 0..ticks {
            let at = tick as f32 * TICK_DT;
            sim.shield_player();
            sim.advance(TICK_DT, &InputFrame::default());
            mix_cues(&mut mixer, &bank, at, sim.cues());
            let music = sim.music();
            if music.step == 0 && music.step_phase < TICK_DT / SONG[0].motif.step_secs() {
                // A downbeat: pile on the loudest game sounds too.
                healed = !healed;
                mix_cues(
                    &mut mixer,
                    &bank,
                    at,
                    &[Cue::Pickup { healed }, Cue::Hit { fatal: false }],
                );
            }
        }
        assert_eq!(sim.phase(), Phase::Playing);
        let peak = mixer.peak();
        println!(
            "whole song peak: {peak:.3} ({:.1} dBFS), {} s rendered",
            dbfs(peak),
            mixer.samples().len() / synth::SAMPLE_RATE as usize
        );
        assert!(
            peak <= HEADROOM,
            "song peaks at {peak}, over the {HEADROOM} headroom"
        );
        assert!(
            peak >= FLOOR,
            "song peaks at only {peak}; MASTER is wasting headroom"
        );
    }

    /// Everything that can ever coincide on one tick, at full velocity, on
    /// top of the previous step's notes still ringing: an adversarial tick
    /// the song itself never quite reaches.
    #[test]
    fn the_busiest_possible_tick_fits() {
        let bank = bank();
        let mut mixer = Mixer::new();
        let loudest_pitch = |instrument: Instrument| {
            song::pitches(instrument)
                .into_iter()
                .max_by(|&a, &b| {
                    let peak = |p: u8| {
                        let voice = match instrument {
                            Instrument::Bass => Voice::Bass(p),
                            _ => Voice::Lead(p),
                        };
                        bank[&voice].iter().fold(0.0_f32, |m, s| m.max(s.abs()))
                    };
                    peak(a).total_cmp(&peak(b))
                })
                .unwrap()
        };
        let chord = song::chords()[0];
        let full = |instrument, pitch| Cue::Note {
            instrument,
            pitch,
            velocity: 1.0,
        };
        let everything = [
            full(Instrument::Drums, KICK),
            full(Instrument::Drums, SNARE),
            full(Instrument::Drums, HAT),
            full(Instrument::Bass, loudest_pitch(Instrument::Bass)),
            full(Instrument::Lead, loudest_pitch(Instrument::Lead)),
            Cue::Chord(chord),
            Cue::Pickup { healed: true },
            Cue::Hit { fatal: true },
            Cue::Layer {
                instrument: Instrument::Pad,
                on: false,
            },
        ];
        // The step before, still ringing.
        let step = SONG[0].motif.step_secs();
        mix_cues(&mut mixer, &bank, 0.0, &everything[..5]);
        mix_cues(&mut mixer, &bank, step, &everything);
        let peak = mixer.peak();
        println!("busiest tick peak: {peak:.3} ({:.1} dBFS)", dbfs(peak));
        assert!(peak <= 1.0, "an adversarial tick clips at {peak}");
    }

    #[test]
    fn hushed_layers_leave_no_trace_in_the_mix() {
        let bank = bank();
        let mut mixer = Mixer::new();
        let mut sim = Sim::new(3);
        sim.advance(
            0.0,
            &InputFrame {
                start: true,
                ..InputFrame::default()
            },
        );
        // Hush everything for a second (well within the pool, even at two
        // instruments' worth of drain) and mix only the music cues, ignoring
        // the blips the hushing itself makes.
        let hold = InputFrame {
            hold_layer: [true; 4],
            ..InputFrame::default()
        };
        let ticks = (1.0 / TICK_DT) as u32;
        for tick in 0..ticks {
            sim.advance(TICK_DT, &hold);
            let music: Vec<Cue> = sim
                .cues()
                .iter()
                .copied()
                .filter(|c| matches!(c, Cue::Note { .. } | Cue::Chord(_)))
                .collect();
            mix_cues(&mut mixer, &bank, tick as f32 * TICK_DT, &music);
        }
        assert!(
            mixer.peak() < 1e-6,
            "hushed music still sounds: {}",
            mixer.peak()
        );
    }

    #[test]
    fn the_mix_is_the_sum_of_its_voices() {
        let bank = bank();
        let kick = &bank[&Voice::Drum(KICK)];
        let chime = &bank[&Voice::Chime];
        let mut both = Mixer::new();
        both.play(0.0, kick, 0.5);
        both.play(0.01, chime, 0.3);
        let mut only_kick = Mixer::new();
        only_kick.play(0.0, kick, 0.5);
        let mut only_chime = Mixer::new();
        only_chime.play(0.01, chime, 0.3);
        for (i, sample) in both.samples().iter().enumerate() {
            let a = only_kick.samples().get(i).copied().unwrap_or(0.0);
            let b = only_chime.samples().get(i).copied().unwrap_or(0.0);
            assert!((sample - (a + b)).abs() < 1e-6, "sample {i} is not the sum");
        }
        // And a mix that lands exactly on a note's own peak reports it.
        let peak = only_kick.peak();
        let expected = kick.iter().fold(0.0_f32, |m, s| m.max(s.abs())) * 0.5;
        assert!((peak - expected).abs() < 1e-6);
    }

    #[test]
    fn notes_land_where_the_sequencer_put_them() {
        // Mix a bar of the intro and check the mix is loud right after each
        // note and that a downbeat is louder than the quietest sixteenth.
        let bank = bank();
        let mut mixer = Mixer::new();
        let mut sim = Sim::new(3);
        sim.advance(
            0.0,
            &InputFrame {
                start: true,
                ..InputFrame::default()
            },
        );
        sim.place_player(Vec2::new(-5000.0, 0.0));
        let motif = SONG[0].motif;
        let mut onsets = Vec::new();
        let ticks = (motif.bar_secs() / TICK_DT) as u32 + 1;
        for tick in 0..ticks {
            let at = tick as f32 * TICK_DT;
            sim.shield_player();
            sim.advance(TICK_DT, &InputFrame::default());
            if sim.cues().iter().any(|c| matches!(c, Cue::Note { .. })) {
                onsets.push(at);
            }
            mix_cues(&mut mixer, &bank, at, sim.cues());
        }
        assert!(onsets.len() >= 4, "too few notes in a bar: {onsets:?}");
        let out = mixer.samples();
        let rms = |from: f32, secs: f32| {
            let start = (from * synth::SAMPLE_RATE as f32) as usize;
            let end = ((from + secs) * synth::SAMPLE_RATE as f32) as usize;
            let window = &out[start.min(out.len())..end.min(out.len())];
            (window.iter().map(|s| s * s).sum::<f32>() / window.len().max(1) as f32).sqrt()
        };
        let step = motif.step_secs();
        for &at in &onsets {
            assert!(
                rms(at, step * 0.5) > 1e-3,
                "silence right after a note at {at}s"
            );
        }
        // Somewhere in the bar there is a step with nothing starting on it.
        let quiet = (0..STEPS_PER_BAR)
            .map(|s| s as f32 * step)
            .filter(|t| onsets.iter().all(|o| (o - t).abs() > step * 0.5))
            .map(|t| rms(t, step * 0.5))
            .fold(f32::MAX, f32::min);
        assert!(
            quiet < rms(onsets[0], step * 0.5),
            "no dynamics between steps"
        );
    }
}
