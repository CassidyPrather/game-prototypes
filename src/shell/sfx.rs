//! What the menu sounds like: moving, hovering, bumping an end, launching,
//! and coming back.
//!
//! Pure and macroquad-free like the prototypes' synths, so the tests below
//! can check every sound is a WAV the browser will decode and that a player
//! mashing keys cannot clip the output. The binary bakes one buffer per
//! [`Sfx`] and plays it at [`Sfx::gain`].
//!
//! Everything pitched is in E major pentatonic, so no two menu sounds clash
//! however they overlap. Interface sounds should be quick and soft: the
//! cursor ones are over in a tenth of a second, and only the launch lingers,
//! because it has an animation to cover.

use std::f32::consts::TAU;

use crate::leitmotif::synth::{self, SAMPLE_RATE};

/// Loudest a single menu sound plays; the per-sound gains are fractions of
/// this. Tuned by the headroom tests, the same way as Leitmotif's mix.
pub const MASTER: f32 = 0.5;

// The menu's key, E major pentatonic.
const E5: f32 = 659.26;
const GS5: f32 = 830.61;
const B5: f32 = 987.77;
const CS6: f32 = 1108.73;
const E6: f32 = 1318.51;

/// Fixed seed for the launch's noise, so a build always sounds the same.
const NOISE_SEED: u64 = 0x0B5E_55ED;

/// One menu sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    /// The pointer came to rest on a card.
    Hover,
    /// The carousel moved on to the next card.
    Next,
    /// The carousel moved back to the previous card.
    Prev,
    /// A move that had nowhere to go.
    Bump,
    /// A prototype was chosen; covers the dive into it.
    Launch,
    /// Back out of a prototype to the menu.
    Back,
}

impl Sfx {
    /// Every menu sound, for baking.
    pub const ALL: [Self; 6] = [
        Self::Hover,
        Self::Next,
        Self::Prev,
        Self::Bump,
        Self::Launch,
        Self::Back,
    ];

    /// Playback volume, `0..=1`. Lives here rather than in the frontend so
    /// the tests measure what the speakers get.
    #[must_use]
    pub fn gain(self) -> f32 {
        let share = match self {
            Self::Hover => 0.45,
            Self::Next | Self::Prev => 0.6,
            Self::Bump => 0.9,
            Self::Launch => 0.8,
            Self::Back => 0.7,
        };
        share * MASTER
    }
}

/// Synthesise one menu sound as WAV bytes.
#[must_use]
pub fn render(sfx: Sfx) -> Vec<u8> {
    synth::wav(&samples(sfx))
}

fn samples(sfx: Sfx) -> Vec<f32> {
    match sfx {
        Sfx::Hover => blip(B5, 0.07, 60.0),
        Sfx::Next => blip(E6, 0.1, 40.0),
        Sfx::Prev => blip(CS6, 0.1, 40.0),
        Sfx::Bump => bump(),
        Sfx::Launch => launch(),
        Sfx::Back => back(),
    }
}

/// A short, glassy tick: a sine with a quiet octave, bending down into its
/// note the way a key settles.
fn blip(hz: f32, secs: f32, decay: f32) -> Vec<f32> {
    let mut phase = 0.0_f32;
    synth::render(secs, |t| {
        let bend = 0.06_f32.mul_add((-t * 90.0).exp(), 1.0);
        phase += TAU * hz * bend / SAMPLE_RATE as f32;
        0.3_f32.mul_add((phase * 2.0).sin(), phase.sin()) * (-t * decay).exp() * 0.8
    })
}

/// Two soft knocks, the second quieter, low and unpitched enough that they
/// read as "that's the end" rather than as a note.
fn bump() -> Vec<f32> {
    synth::render(0.2, |t| knock(t, 0.0, 1.0) + knock(t, 0.075, 0.6))
}

/// One knock of [`bump`], starting at `start` seconds.
fn knock(t: f32, start: f32, level: f32) -> f32 {
    const FROM: f32 = 260.0;
    const TO: f32 = 150.0;
    const DROP: f32 = 30.0;
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    // Closed-form phase of a pitch falling exponentially from FROM to TO.
    let phase = TAU * TO.mul_add(u, (FROM - TO) / DROP * (1.0 - (-u * DROP).exp()));
    phase.sin() * (-u * 30.0).exp() * level * 0.9
}

/// A struck note at `hz` from `start` seconds: the fundamental and two
/// stretched partials, the shape of Leitmotif's bell but brighter.
fn chime(t: f32, start: f32, hz: f32, decay: f32) -> f32 {
    let since = t - start;
    if since < 0.0 {
        return 0.0;
    }
    let partial = |ratio: f32, damping: f32, level: f32| {
        (TAU * hz * ratio * since).sin() * (-since * decay * damping).exp() * level
    };
    partial(1.0, 1.0, 1.0) + partial(2.01, 1.8, 0.3) + partial(3.02, 3.0, 0.12)
}

/// A rising arpeggio over a swell of air that peaks as the screen closes.
fn launch() -> Vec<f32> {
    const SECS: f32 = 1.0;
    /// When the air is loudest, matching the menu's iris closing.
    const SWELL: f32 = 0.42;
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let notes = chime(t, 0.0, E5, 7.0) * 0.32
            + chime(t, 0.06, GS5, 7.0) * 0.28
            + chime(t, 0.12, B5, 6.0) * 0.28
            + chime(t, 0.18, E6, 5.0) * 0.3;
        let noise = rng.f32().mul_add(2.0, -1.0);
        lp.0 = (noise - lp.0).mul_add(0.18, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.18, lp.1);
        let swell = if t < SWELL {
            (t / SWELL).powi(2)
        } else {
            (-(t - SWELL) * 12.0).exp()
        };
        lp.1.mul_add(swell * 1.6, notes)
    })
}

/// The launch's way home: two notes falling, softer and quicker.
fn back() -> Vec<f32> {
    synth::render(0.5, |t| {
        chime(t, 0.0, B5, 9.0).mul_add(0.4, chime(t, 0.07, E5, 8.0) * 0.45)
    })
}

#[cfg(test)]
// Tests turn positive seconds into sample counts.
#[allow(clippy::cast_sign_loss)]
mod tests {
    use super::*;
    use crate::leitmotif::mix::Mixer;

    /// Loudest any pile-up of menu sounds may get.
    const HEADROOM: f32 = 0.9;

    fn decoded(sfx: Sfx) -> Vec<f32> {
        synth::decode(&render(sfx))
    }

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0.0_f32, |peak, s| peak.max(s.abs()))
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn every_sound_is_a_well_formed_wav() {
        // The browser's decoder hangs rather than failing on a bad header.
        for sfx in Sfx::ALL {
            let bytes = render(sfx);
            assert_eq!(&bytes[0..4], b"RIFF", "{sfx:?}");
            assert_eq!(&bytes[8..12], b"WAVE", "{sfx:?}");
            assert_eq!(u32_at(&bytes, 4) as usize, bytes.len() - 8, "{sfx:?}");
            assert_eq!(u32_at(&bytes, 40) as usize, bytes.len() - 44, "{sfx:?}");
            assert_eq!(u32_at(&bytes, 24), SAMPLE_RATE, "{sfx:?}");
        }
    }

    #[test]
    fn every_sound_is_audible_and_under_the_headroom_alone() {
        for sfx in Sfx::ALL {
            let loudest = peak(&decoded(sfx)) * sfx.gain();
            assert!(loudest > 0.1, "{sfx:?} is nearly silent ({loudest})");
            assert!(loudest < HEADROOM, "{sfx:?} peaks at {loudest}");
        }
    }

    #[test]
    fn every_sound_starts_and_ends_silently() {
        for sfx in Sfx::ALL {
            let samples = decoded(sfx);
            let ends = [samples[0], samples[samples.len() - 1]];
            assert!(ends.iter().all(|s| s.abs() < 1e-3), "{sfx:?} clicks");
        }
    }

    #[test]
    fn cursor_sounds_are_over_quickly() {
        // A held key repeats; a tick that lingers turns repeats into a smear.
        for sfx in [Sfx::Hover, Sfx::Next, Sfx::Prev, Sfx::Bump] {
            let secs = decoded(sfx).len() as f32 / SAMPLE_RATE as f32;
            assert!(secs <= 0.2, "{sfx:?} lasts {secs} s");
        }
        let secs = decoded(Sfx::Launch).len() as f32 / SAMPLE_RATE as f32;
        assert!(secs <= 1.2, "the launch lasts {secs} s");
    }

    #[test]
    fn mashing_the_menu_does_not_clip() {
        // The worst a player can do: a key held on repeat through the whole
        // carousel with the pointer skating over cards, then a launch on top
        // of it and straight back out. A press either moves or bumps, never
        // both, so each shelf size gets its own pile-up.
        let sounds: Vec<(Sfx, Vec<f32>)> = Sfx::ALL.iter().map(|&s| (s, decoded(s))).collect();
        let play = |mixer: &mut Mixer, at: f32, sfx: Sfx| {
            let (_, samples) = sounds.iter().find(|(s, _)| *s == sfx).unwrap();
            mixer.play(at, samples, sfx.gain());
        };
        for (shelf, moves) in [
            ("many prototypes", [Sfx::Next, Sfx::Prev]),
            ("one prototype", [Sfx::Bump, Sfx::Bump]),
        ] {
            let mut mixer = Mixer::new();
            for step in 0..30 {
                let at = step as f32 / 30.0;
                play(&mut mixer, at, moves[step % 2]);
                play(&mut mixer, at + 0.01, Sfx::Hover);
            }
            play(&mut mixer, 0.9, Sfx::Launch);
            play(&mut mixer, 1.0, Sfx::Back);
            let loudest = mixer.peak();
            println!("menu pile-up peak, {shelf}: {loudest:.3}");
            assert!(loudest <= HEADROOM, "{shelf}: a pile-up peaks at {loudest}");
        }
    }

    #[test]
    fn generation_is_reproducible() {
        for sfx in Sfx::ALL {
            assert_eq!(render(sfx), render(sfx), "{sfx:?}");
        }
    }
}
