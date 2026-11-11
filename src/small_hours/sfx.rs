//! What the night sounds like: a fire, the dark around it, and a few soft
//! notes for everything that moves.
//!
//! Pure and macroquad-free like the other synths here, so the tests below
//! check every sound is a WAV the browser will decode and that the worst
//! pile-up a night can cause stays under full scale. The frontend bakes one
//! buffer per [`Sfx`] and plays it at [`Sfx::gain`].
//!
//! Everything pitched is in C sharp minor pentatonic, which is the menu's E
//! major pentatonic started from its dark end, so dawn can resolve to E
//! without changing a note. Each keeper walks on a note of its own; four of
//! them stepping in the same hour is a chord.

use std::f32::consts::TAU;

use crate::leitmotif::synth::{self, SAMPLE_RATE};

/// Loudest a single sound plays; the per-sound gains are fractions of it.
pub const MASTER: f32 = 0.5;

/// Fixed noise seed, so a build always sounds the same.
const SEED: u64 = 0x5A11_4025;

const CS2: f32 = 69.30;
const GS2: f32 = 103.83;
const CS3: f32 = 138.59;
const E3: f32 = 164.81;
const GS3: f32 = 207.65;
const B3: f32 = 246.94;
const CS4: f32 = 277.18;
const E4: f32 = 329.63;
const GS4: f32 = 415.30;
const B4: f32 = 493.88;
const CS5: f32 = 554.37;
const E5: f32 = 659.26;
const GS5: f32 = 830.61;
const B5: f32 = 987.77;
const E6: f32 = 1318.51;

/// The note each keeper walks on, in the order they wake.
const KEEPER_NOTES: [f32; 4] = [E4, GS4, B4, CS5];

/// One sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    /// Keeper `0..4` takes a step.
    Step(u8),
    /// A keeper walks into something that will not move.
    Balk,
    /// The shades move. One per hour, however many there are.
    Creep,
    /// A shade turns toward a keeper.
    Notice,
    /// A shade turns back toward the hearth, or loses interest.
    Drift,
    /// A lantern goes out.
    Snuff,
    /// A shade reaches the hearth and takes an ember, but not the last.
    Fed,
    /// The hearth goes out.
    Out,
    /// Dawn on a lit hearth.
    Dawn,
    /// Dawn on a cold one.
    Cold,
    /// The night runs backward.
    Rewind,
    /// Keeper `0..4` wakes.
    Wake(u8),
    /// An hour taken back.
    Undo,
    /// One of the three marks, `0..3`, rising.
    Mark(u8),
    /// The hearth, looped.
    Fire,
    /// The dark around it, looped: wind in the trees, and crickets.
    Night,
}

impl Sfx {
    /// Every sound, for baking.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all = vec![
            Self::Balk,
            Self::Creep,
            Self::Notice,
            Self::Drift,
            Self::Snuff,
            Self::Fed,
            Self::Out,
            Self::Dawn,
            Self::Cold,
            Self::Rewind,
            Self::Undo,
            Self::Fire,
            Self::Night,
        ];
        for i in 0..4 {
            all.push(Self::Step(i));
            all.push(Self::Wake(i));
        }
        for i in 0..3 {
            all.push(Self::Mark(i));
        }
        all
    }

    /// Playback volume, `0..=1`. Lives here so the tests measure what the
    /// speakers get.
    #[must_use]
    pub const fn gain(self) -> f32 {
        let share = match self {
            Self::Undo => 0.25,
            Self::Step(_) | Self::Night => 0.3,
            Self::Balk | Self::Creep | Self::Drift => 0.4,
            Self::Fire => 0.45,
            Self::Wake(_) | Self::Fed => 0.5,
            Self::Notice | Self::Rewind => 0.55,
            Self::Snuff | Self::Mark(_) => 0.6,
            Self::Out | Self::Cold => 0.7,
            Self::Dawn => 0.75,
        };
        share * MASTER
    }
}

/// Synthesise one sound as WAV bytes.
#[must_use]
pub fn render(sfx: Sfx) -> Vec<u8> {
    synth::wav(&tame(samples(sfx)))
}

/// Scale a buffer down if it ran hot, so nothing is squashed at the rail;
/// the gains then set the loudness.
fn tame(mut samples: Vec<f32>) -> Vec<f32> {
    const CEILING: f32 = 0.9;
    let peak = samples.iter().fold(0.0_f32, |p, s| p.max(s.abs()));
    if peak > CEILING {
        for s in &mut samples {
            *s *= CEILING / peak;
        }
    }
    samples
}

fn keeper_note(i: u8) -> f32 {
    KEEPER_NOTES[usize::from(i).min(3)]
}

fn mark_note(i: u8) -> f32 {
    [GS5, B5, E6][usize::from(i).min(2)]
}

fn samples(sfx: Sfx) -> Vec<f32> {
    match sfx {
        Sfx::Step(i) => {
            let hz = keeper_note(i);
            synth::render(0.3, |t| {
                mallet(t, 0.0, hz, 16.0).mul_add(0.6, thud(t, 0.0, 120.0, 0.25))
            })
        }
        Sfx::Balk => synth::render(0.18, |t| {
            thud(t, 0.0, 150.0, 0.8) + thud(t, 0.05, 110.0, 0.4)
        }),
        Sfx::Creep => creep(),
        Sfx::Notice => synth::render(0.7, |t| {
            // A falling minor third with a shiver on it: something looked up.
            let shiver = (TAU * 11.0 * t).sin().mul_add(0.25, 0.75);
            glass(t, 0.11, GS4, 5.0).mul_add(0.6, glass(t, 0.0, B4, 7.0) * 0.5) * shiver
        }),
        Sfx::Drift => synth::render(0.5, |t| glass(t, 0.0, CS4, 7.0) * 0.6),
        Sfx::Snuff => snuff(),
        Sfx::Fed => hiss(0.5, 0.5, GS2),
        Sfx::Out => hiss(1.3, 1.0, CS2),
        Sfx::Dawn => synth::render(2.6, |t| {
            let rise: f32 = [E4, GS4, B4, E5, GS5]
                .iter()
                .enumerate()
                .map(|(i, &hz)| mallet(t, i as f32 * 0.14, hz, 2.2) * 0.3)
                .sum();
            let swell = (t / 0.9).min(1.0) * (-(t - 0.9).max(0.0) * 1.6).exp();
            let pad = (TAU * B3 * t).sin().mul_add(0.7, (TAU * E3 * t).sin()) * swell * 0.22;
            rise + pad
        }),
        Sfx::Cold => synth::render(2.2, |t| {
            glass(t, 0.0, CS3, 1.6).mul_add(0.6, glass(t, 0.35, GS3, 2.0) * 0.35)
                + thud(t, 0.0, 70.0, 0.4)
        }),
        Sfx::Rewind => rewind(),
        Sfx::Wake(i) => {
            let hz = keeper_note(i);
            synth::render(0.9, |t| {
                mallet(t, 0.0, hz, 4.0).mul_add(0.5, mallet(t, 0.09, hz * 2.0, 5.0) * 0.3)
            })
        }
        Sfx::Undo => synth::render(0.06, |t| (TAU * E6 * t).sin() * (-t * 90.0).exp()),
        Sfx::Mark(i) => {
            let hz = mark_note(i);
            synth::render(1.3, |t| {
                mallet(t, 0.0, hz, 3.0).mul_add(0.5, mallet(t, 0.0, hz * 0.5, 2.5) * 0.3)
            })
        }
        Sfx::Fire => fire(),
        Sfx::Night => night(),
    }
}

/// White noise, `-1..1`, from a seeded generator.
fn noise(rng: &mut fastrand::Rng) -> f32 {
    rng.f32().mul_add(2.0, -1.0)
}

/// A soft struck note, a felt mallet on wood: the fundamental and a little
/// of its octave, dying at `decay` per second.
fn mallet(t: f32, start: f32, hz: f32, decay: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let attack = (u * 400.0).min(1.0);
    let body = (TAU * hz * u).sin() * (-u * decay).exp();
    let over = (TAU * hz * 2.0 * u).sin() * (-u * decay * 2.5).exp() * 0.25;
    (body + over) * attack
}

/// A thinner, glassier note with a slow attack, for the shades.
fn glass(t: f32, start: f32, hz: f32, decay: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let attack = (u * 40.0).min(1.0);
    let body = (TAU * hz * 1.005 * u)
        .sin()
        .mul_add(0.6, (TAU * hz * u).sin());
    let fifth = (TAU * hz * 3.0 * u).sin() * 0.12;
    (body + fifth) * attack * (-u * decay).exp() * 0.6
}

/// A dull low knock sliding down from `hz`.
fn thud(t: f32, start: f32, hz: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let phase = TAU * (hz * 0.6).mul_add(u, hz * 0.4 / 30.0 * (1.0 - (-u * 30.0).exp()));
    phase.sin() * (-u * 30.0).exp() * level
}

/// The shades moving: a breath of dark air and a low note under it.
fn creep() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0xC4);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(0.42, |t| {
        let swell = (TAU * t / 0.84).sin().max(0.0).powi(2);
        lp.0 = (noise(&mut rng) - lp.0).mul_add(0.04, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.04, lp.1);
        let low = (TAU * GS2 * t)
            .sin()
            .mul_add(0.2, (TAU * CS3 * t).sin() * 0.25);
        lp.1.mul_add(5.0, low) * swell
    })
}

/// A lantern going out: a puff, and its note falling away.
fn snuff() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0x5F);
    let mut lp = 0.0_f32;
    synth::render(0.9, |t| {
        let puff_env = (t * 200.0).min(1.0) * (-t * 14.0).exp();
        lp = (noise(&mut rng) - lp).mul_add(0.12, lp);
        // B3 falling an octave and a bit as it fades.
        let hz = B3 * (-t * 1.4).exp().mul_add(0.55, 0.45);
        let fall = (TAU * hz * t).sin() * (-t * 4.5).exp() * 0.5;
        (lp * puff_env).mul_add(2.2, fall)
    })
}

/// Embers hissing as something cold lands on them, `secs` long, over a low
/// note at `hz`.
fn hiss(secs: f32, weight: f32, hz: f32) -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0x415);
    let mut lp = 0.0_f32;
    let mut hp = 0.0_f32;
    synth::render(secs, |t| {
        let n = noise(&mut rng);
        lp = (n - lp).mul_add(0.5, lp);
        hp = (lp - hp).mul_add(0.05, hp);
        let steam = (lp - hp) * (t * 300.0).min(1.0) * (-t * 5.0 / secs).exp() * 0.5;
        let drop = (TAU * hz * t).sin() * (-t * 3.0 / secs).exp() * 0.6 * weight;
        steam + drop + thud(t, 0.0, 90.0, 0.5 * weight)
    })
}

/// The night running backward: a swell that rises to a stop, like a tape
/// rewound to its head.
fn rewind() -> Vec<f32> {
    const SECS: f32 = 0.9;
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0x2E);
    let mut lp = 0.0_f32;
    synth::render(SECS, |t| {
        let x = t / SECS;
        let env = x * x * (1.0 - x).sqrt() * 2.6;
        let cutoff = x.mul_add(0.25, 0.02);
        lp = (noise(&mut rng) - lp).mul_add(cutoff, lp);
        // E3 sliding up two octaves; this is that sweep's phase.
        let phase = TAU * E3 * SECS / 1.386 * (1.386 * x).exp_m1();
        let flutter = (TAU * 23.0 * t).sin().mul_add(0.3, 0.7);
        lp.mul_add(0.8, phase.sin() * 0.3) * env * flutter
    })
}

/// Render `secs` of a looping voice: `fold` seconds more are rendered and
/// laid back over the head, so the end runs straight into the start. Any
/// periodic modulation in `voice` must make whole cycles over `secs`.
fn looped(secs: f32, fold: f32, mut voice: impl FnMut(f32) -> f32) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (body, fold) = (
        (secs * SAMPLE_RATE as f32) as usize,
        (fold * SAMPLE_RATE as f32) as usize,
    );
    let raw: Vec<f32> = (0..body + fold)
        .map(|i| voice(i as f32 / SAMPLE_RATE as f32))
        .collect();
    let mut out: Vec<f32> = raw[..body].to_vec();
    for (i, slot) in out.iter_mut().take(fold).enumerate() {
        let w = i as f32 / fold as f32;
        *slot = raw[i].mul_add(w, raw[body + i] * (1.0 - w));
    }
    out
}

/// The hearth: a low breathing rumble with the odd log popping over it.
fn fire() -> Vec<f32> {
    const SECS: f32 = 6.0;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0xF1);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut snap = 0.0_f32;
    let mut crackle = 0.0_f32;
    looped(SECS, 1.0, |t| {
        let n = noise(&mut rng);
        let breath = (TAU * 2.0 * t / SECS).sin().mul_add(0.2, 0.8)
            * (TAU * 5.0 * t / SECS).sin().mul_add(0.15, 0.85);
        lp.0 = (n - lp.0).mul_add(0.02, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.02, lp.1);
        let rumble = lp.1 * 9.0 * breath;
        let at_seam = (t - SECS).abs() < 0.02;
        if pops.f32() < 0.000_12 && !at_seam {
            snap = pops.f32().powi(2).mul_add(0.7, 0.15);
        }
        snap *= 0.985;
        crackle = n.mul_add(snap, -crackle).mul_add(0.5, crackle);
        crackle.mul_add(1.1, rumble)
    })
}

/// The dark: wind in the trees, and crickets keeping time, never across
/// the seam.
fn night() -> Vec<f32> {
    const SECS: f32 = 8.0;
    let mut rng = fastrand::Rng::with_seed(SEED ^ 0x91);
    let mut lp = (0.0_f32, 0.0_f32);
    looped(SECS, 1.5, |t| {
        let gust = (TAU * t / SECS).sin().mul_add(0.4, 0.6)
            * (TAU * 3.0 * t / SECS).sin().mul_add(0.2, 0.8);
        let cutoff = gust.mul_add(0.02, 0.006);
        lp.0 = (noise(&mut rng) - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let wind = lp.1 * gust * 5.0;
        // Crickets: bursts of a fast trill, four to the loop, each three
        // pulses, high and thin.
        let beat = (t * 4.0 / SECS).fract() * SECS / 4.0;
        let cricket: f32 = (0..3)
            .map(|i| {
                let u = (i as f32).mul_add(-0.09, beat - 0.5);
                if (0.0..0.06).contains(&u) {
                    let trill = (TAU * 55.0 * u).sin().max(0.0);
                    (TAU * 4434.9 * u).sin() * trill * (TAU * u / 0.12).sin() * 0.05
                } else {
                    0.0
                }
            })
            .sum();
        wind + cricket
    })
}

#[cfg(test)]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod tests {
    use super::*;
    use crate::leitmotif::mix::Mixer;

    /// Loudest any pile-up may get.
    const HEADROOM: f32 = 0.9;

    const LOOPS: [Sfx; 2] = [Sfx::Fire, Sfx::Night];

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
    fn every_value_is_listed_once() {
        let all = Sfx::all();
        for (i, a) in all.iter().enumerate() {
            assert!(!all[i + 1..].contains(a), "{a:?} is listed twice");
        }
        assert_eq!(all.len(), 13 + 8 + 3);
    }

    #[test]
    fn every_sound_is_a_well_formed_wav() {
        for sfx in Sfx::all() {
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
        for sfx in Sfx::all() {
            let samples = decoded(sfx);
            let loudest = peak(&samples) * sfx.gain();
            assert!(loudest > 0.02, "{sfx:?} is nearly silent ({loudest})");
            assert!(loudest < HEADROOM, "{sfx:?} peaks at {loudest}");
            let pinned = samples.iter().filter(|s| s.abs() > 0.999).count();
            assert!(pinned * 200 < samples.len(), "{sfx:?} clips in its buffer");
        }
    }

    #[test]
    fn every_one_shot_starts_and_ends_silently() {
        for sfx in Sfx::all().into_iter().filter(|s| !LOOPS.contains(s)) {
            let samples = decoded(sfx);
            let ends = [samples[0], samples[samples.len() - 1]];
            assert!(
                ends.iter().all(|s| s.abs() < 2e-3),
                "{sfx:?} clicks: {ends:?}"
            );
        }
    }

    #[test]
    fn the_loops_have_no_seam() {
        for sfx in LOOPS {
            let samples = decoded(sfx);
            let (first, last) = (samples[0], samples[samples.len() - 1]);
            assert!(
                (first - last).abs() < 0.02,
                "{sfx:?} jumps {first} -> {last}"
            );
            let secs = samples.len() as f32 / SAMPLE_RATE as f32;
            assert!((4.0..=8.0).contains(&secs), "{sfx:?} loops every {secs} s");
        }
    }

    #[test]
    fn steps_are_quick_and_each_keeper_has_its_own_note() {
        for i in 0..4 {
            let secs = decoded(Sfx::Step(i)).len() as f32 / SAMPLE_RATE as f32;
            assert!(secs <= 0.3, "Step({i}) lasts {secs} s");
        }
        assert!(KEEPER_NOTES.windows(2).all(|w| w[0] < w[1]));
        assert_ne!(render(Sfx::Step(0)), render(Sfx::Step(3)));
    }

    #[test]
    fn marks_rise() {
        let marks: Vec<f32> = (0..3).map(mark_note).collect();
        assert!(marks.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn the_worst_hour_does_not_clip() {
        // Both loops going, and a player holding a key through the worst
        // hours a night has: four keepers stepping at once, the shades
        // moving, two of them turning, a lantern going out, the hearth
        // going out, and dawn landing on top of it with its marks.
        let sounds: Vec<(Sfx, Vec<f32>)> =
            Sfx::all().into_iter().map(|s| (s, decoded(s))).collect();
        let play = |mixer: &mut Mixer, at: f32, sfx: Sfx| {
            let (_, samples) = sounds.iter().find(|(s, _)| *s == sfx).unwrap();
            mixer.play(at, samples, sfx.gain());
        };
        let mut mixer = Mixer::new();
        play(&mut mixer, 0.0, Sfx::Fire);
        play(&mut mixer, 0.0, Sfx::Night);
        for hour in 0..6 {
            let at = hour as f32 * 0.12;
            for keeper in 0..4 {
                play(&mut mixer, at, Sfx::Step(keeper));
            }
            play(&mut mixer, at + 0.08, Sfx::Creep);
            play(&mut mixer, at + 0.1, Sfx::Notice);
        }
        play(&mut mixer, 0.6, Sfx::Snuff);
        play(&mut mixer, 0.62, Sfx::Out);
        play(&mut mixer, 0.7, Sfx::Dawn);
        for mark in 0..3 {
            play(
                &mut mixer,
                f32::from(mark).mul_add(0.25, 0.9),
                Sfx::Mark(mark),
            );
        }
        let loudest = mixer.peak();
        println!("small hours pile-up peak: {loudest:.3}");
        assert!(loudest <= HEADROOM, "the worst hour peaks at {loudest}");
    }

    #[test]
    fn generation_is_reproducible() {
        for sfx in [Sfx::Fire, Sfx::Night, Sfx::Snuff, Sfx::Rewind] {
            assert_eq!(render(sfx), render(sfx), "{sfx:?}");
        }
    }
}
