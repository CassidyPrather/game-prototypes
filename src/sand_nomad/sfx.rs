//! What the basin sounds like. No music: wind, wood, stone, water, fire,
//! and the small noises of packing a hold.
//!
//! Pure and macroquad-free like the other synths here, so the tests below
//! check every sound is a WAV the browser will decode and that the worst
//! pile-up a player can cause stays under full scale. The frontend bakes
//! one buffer per [`Sfx`] and plays it at [`Sfx::gain`].
//!
//! Handling a thing sounds like what it is made of: lifting and setting
//! down wood knocks, metal rings, clay clinks, cloth thumps. Everything
//! pitched belongs to one D major pentatonic, and each motive has its own
//! note in it, so a waystone taking an offering, a rune being read and a
//! trader's face all agree about which motive is which.

use std::f32::consts::TAU;

use crate::leitmotif::synth::{self, SAMPLE_RATE};
use crate::sand_nomad::barter::Emote;
use crate::sand_nomad::item::Material;
use crate::sand_nomad::motive::Motive;

/// Loudest a single sound plays; the per-sound gains are fractions of it.
pub const MASTER: f32 = 0.55;

/// Fixed noise seed, so a build always sounds the same.
const SEED: u64 = 0x5A4D_0B0A;

/// The basin's scale, D major pentatonic, low to high.
const D4: f32 = 293.66;
const E4: f32 = 329.63;
const FS4: f32 = 369.99;
const A4: f32 = 440.0;
const B4: f32 = 493.88;
const D5: f32 = 587.33;
const A3: f32 = 220.0;
const D3: f32 = 146.83;

/// A motive's own note.
#[must_use]
pub const fn note(motive: Motive) -> f32 {
    match motive {
        Motive::Bliss => D5,
        Motive::Repose => A4,
        Motive::Love => FS4,
        Motive::Pain => E4,
        Motive::Zeal => B4,
        Motive::Hate => D4,
    }
}

/// What a thing sounds like when handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stuff {
    Wood,
    Cloth,
    Metal,
    Clay,
    Bone,
    Paper,
    Stone,
}

impl Stuff {
    pub const ALL: [Self; 7] = [
        Self::Wood,
        Self::Cloth,
        Self::Metal,
        Self::Clay,
        Self::Bone,
        Self::Paper,
        Self::Stone,
    ];

    /// How a material sounds.
    #[must_use]
    pub const fn of(material: Material) -> Self {
        match material {
            Material::Wood => Self::Wood,
            Material::Cloth => Self::Cloth,
            Material::Metal | Material::Brass => Self::Metal,
            Material::Ceramic | Material::Glass => Self::Clay,
            Material::Bone => Self::Bone,
            Material::Paper => Self::Paper,
            Material::Salt | Material::Relic => Self::Stone,
        }
    }
}

/// One sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    /// Picking a thing up.
    Lift(Stuff),
    /// Setting a thing down in a cell.
    Set(Stuff),
    /// It will not go there.
    Nope,
    /// Turning a thing a quarter.
    Turn,
    /// The pointer settling on something.
    Tick,
    /// A thing set on a brass pan.
    Pan,
    /// The scale's beam swinging to a new reading.
    Beam,
    /// A bargain struck.
    Deal,
    /// A trader's face.
    Face(Emote),
    /// Starting to sit with a thing.
    Ponder,
    /// One dot of a rune being read, by position, `0..6`.
    Dot(u8),
    /// A thing grew heavier.
    Heavier,
    /// A thing woke up.
    Wake,
    /// A woken thing moved itself.
    Skitter,
    /// The crew drinking.
    Sip,
    /// Nothing to drink.
    Thirst,
    /// A new day.
    Dawn,
    /// Casting off.
    Sail,
    /// Mooring.
    Moor,
    /// A waystone taking its offering, in its motive's note.
    Stone(Motive),
    /// A waystone turning something away.
    Clunk,
    /// Burning a thing.
    Burn,
    /// The Theseans' hammers.
    Hammer,
    /// Filling jars.
    Pour,
    /// Leaving a thing in the sand.
    Drop,
    /// The circuit walked.
    Circuit,
    /// Home.
    Home,
    /// The convoy's guns, far off.
    Guns,
    /// A semaphore tower clacking, far off.
    Semaphore,
    /// Fireworks, far off.
    Fireworks,
    /// The dirigible's engines passing over.
    Drone,
    /// Wind over the basin, looped.
    Wind,
}

impl Sfx {
    /// Every sound, for baking.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all = Vec::new();
        for stuff in Stuff::ALL {
            all.push(Self::Lift(stuff));
            all.push(Self::Set(stuff));
        }
        for emote in [
            Emote::Love,
            Emote::Bliss,
            Emote::Repose,
            Emote::Zeal,
            Emote::Hate,
            Emote::Pain,
            Emote::Anima,
        ] {
            all.push(Self::Face(emote));
        }
        for dot in 0..6 {
            all.push(Self::Dot(dot));
        }
        for motive in Motive::ALL {
            all.push(Self::Stone(motive));
        }
        all.extend([
            Self::Nope,
            Self::Turn,
            Self::Tick,
            Self::Pan,
            Self::Beam,
            Self::Deal,
            Self::Ponder,
            Self::Heavier,
            Self::Wake,
            Self::Skitter,
            Self::Sip,
            Self::Thirst,
            Self::Dawn,
            Self::Sail,
            Self::Moor,
            Self::Clunk,
            Self::Burn,
            Self::Hammer,
            Self::Pour,
            Self::Drop,
            Self::Circuit,
            Self::Home,
            Self::Guns,
            Self::Semaphore,
            Self::Fireworks,
            Self::Drone,
            Self::Wind,
        ]);
        all
    }

    /// Playback volume, `0..=1`. Lives here so the tests measure what the
    /// speakers get.
    #[must_use]
    pub const fn gain(self) -> f32 {
        let share = match self {
            Self::Tick => 0.25,
            Self::Lift(_) | Self::Turn | Self::Beam => 0.5,
            Self::Set(_) | Self::Pan | Self::Skitter | Self::Sip | Self::Drop => 0.6,
            Self::Dot(_) | Self::Dawn | Self::Semaphore | Self::Drone => 0.45,
            Self::Face(_) | Self::Nope | Self::Clunk | Self::Thirst | Self::Pour => 0.55,
            Self::Wind => 0.35,
            Self::Heavier | Self::Ponder | Self::Fireworks | Self::Guns => 0.7,
            Self::Deal
            | Self::Wake
            | Self::Sail
            | Self::Moor
            | Self::Stone(_)
            | Self::Burn
            | Self::Hammer => 0.75,
            Self::Circuit | Self::Home => 0.8,
        };
        share * MASTER
    }
}

/// Synthesise one sound as WAV bytes.
#[must_use]
pub fn render(sfx: Sfx) -> Vec<u8> {
    synth::wav(&tame(samples(sfx)))
}

/// Scale a buffer down if its noise ran hot, so nothing is squashed at the
/// rail; the gains then set the loudness.
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

fn samples(sfx: Sfx) -> Vec<f32> {
    match sfx {
        Sfx::Lift(stuff) => handle(stuff, true),
        Sfx::Set(stuff) => handle(stuff, false),
        Sfx::Nope => synth::render(0.18, |t| {
            knock(t, 0.0, 140.0, 95.0, 1.0) + knock(t, 0.07, 120.0, 85.0, 0.7)
        }),
        Sfx::Turn => swish(0.09, 0.5),
        Sfx::Tick => synth::render(0.03, |t| {
            (TAU * 2400.0 * t).sin() * (-t * 180.0).exp() * 0.6
        }),
        Sfx::Pan => pan(),
        Sfx::Beam => beam(),
        Sfx::Deal => deal(),
        Sfx::Face(emote) => face(emote),
        Sfx::Ponder => ponder(),
        Sfx::Dot(i) => {
            let notes = [D4, E4, FS4, A4, B4, D5];
            let hz = notes[usize::from(i).min(5)] * 2.0;
            synth::render(0.35, |t| chime(t, 0.0, hz, 9.0) * 0.6)
        }
        Sfx::Heavier => heavier(),
        Sfx::Wake => wake(),
        Sfx::Skitter => skitter(),
        Sfx::Sip => sip(),
        Sfx::Thirst => thirst(),
        Sfx::Dawn => synth::render(1.2, |t| {
            chime(t, 0.0, A4 * 2.0, 3.5).mul_add(0.3, chime(t, 0.12, D5 * 2.0, 3.0) * 0.25)
        }),
        Sfx::Sail => sail(),
        Sfx::Moor => moor(),
        Sfx::Stone(motive) => stone(note(motive), 2.2),
        Sfx::Clunk => synth::render(0.3, |t| {
            knock(t, 0.0, 90.0, 60.0, 1.0) + noise_burst(t, 0.0, 0.03, 0.3)
        }),
        Sfx::Burn => burn(),
        Sfx::Hammer => synth::render(0.75, |t| {
            [0.0, 0.22, 0.44]
                .iter()
                .map(|&at| wood(t - at, 1.0))
                .sum::<f32>()
        }),
        Sfx::Pour => pour(),
        Sfx::Drop => synth::render(0.35, |t| sand(t, 0.25) * 0.9),
        Sfx::Circuit => synth::render(2.6, |t| {
            [D4, FS4, A4, B4, D5]
                .iter()
                .enumerate()
                .map(|(i, &hz)| stone_at(t, i as f32 * 0.14, hz, 2.0) * 0.28)
                .sum()
        }),
        Sfx::Home => synth::render(3.5, |t| {
            [D3, A3, D4, FS4, A4]
                .iter()
                .map(|&hz| stone_at(t, 0.0, hz, 1.2) * 0.24)
                .sum::<f32>()
                + stone_at(t, 0.5, D5, 1.4) * 0.22
        }),
        Sfx::Guns => guns(),
        Sfx::Semaphore => synth::render(0.6, |t| {
            [0.0, 0.16, 0.24, 0.42]
                .iter()
                .map(|&at| wood(t - at, 0.35) * 0.5)
                .sum()
        }),
        Sfx::Fireworks => fireworks(),
        Sfx::Drone => drone(),
        Sfx::Wind => wind(),
    }
}

/// White noise, `-1..1`, from a seeded generator.
fn noise(rng: &mut fastrand::Rng) -> f32 {
    rng.f32().mul_add(2.0, -1.0)
}

/// A decaying click of noise from `start`, lasting about `len` seconds.
fn noise_burst(t: f32, start: f32, len: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    // Cheap deterministic hash noise, so this can be evaluated at any `t`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = {
        let i = (t * SAMPLE_RATE as f32) as u32;
        let h = i.wrapping_mul(0x9E37_79B9) ^ (i >> 7).wrapping_mul(0x85EB_CA6B);
        (h >> 8) as f32 / (1u32 << 24) as f32
    };
    n.mul_add(2.0, -1.0) * (-u / len * 5.0).exp() * level
}

/// A pitch falling exponentially from `from` to `to` Hz, struck at `start`.
fn knock(t: f32, start: f32, from: f32, to: f32, level: f32) -> f32 {
    const DROP: f32 = 30.0;
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let phase = TAU * to.mul_add(u, (from - to) / DROP * (1.0 - (-u * DROP).exp()));
    phase.sin() * (-u * 28.0).exp() * level
}

/// A struck note with two stretched partials.
fn chime(t: f32, start: f32, hz: f32, decay: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let partial = |ratio: f32, damping: f32, level: f32| {
        (TAU * hz * ratio * u).sin() * (-u * decay * damping).exp() * level
    };
    partial(1.0, 1.0, 1.0) + partial(2.01, 1.8, 0.3) + partial(3.02, 3.0, 0.12)
}

/// A singing stone: a low, pure tone with a fifth and an inharmonic ring,
/// slow to die.
fn stone_at(t: f32, start: f32, hz: f32, decay: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let attack = (u / 0.01).min(1.0);
    let a = (TAU * hz * u).sin();
    let fifth = (TAU * hz * 1.5 * u).sin() * 0.35 * (-u * 1.5).exp();
    let ring = (TAU * hz * 2.76 * u).sin() * 0.2 * (-u * 4.0).exp();
    (a + fifth + ring) * (-u * decay).exp() * attack
}

fn stone(hz: f32, decay: f32) -> Vec<f32> {
    synth::render(2.2, |t| stone_at(t, 0.0, hz, decay) * 0.6)
}

/// A hollow wooden knock at `u` seconds in.
fn wood(u: f32, level: f32) -> f32 {
    if u < 0.0 {
        return 0.0;
    }
    let body = (TAU * 210.0 * u).sin() * (-u * 40.0).exp();
    let knock = (TAU * 520.0 * u).sin() * (-u * 90.0).exp() * 0.6;
    (body + knock) * level
}

/// Sand shifting: soft lowpassed noise with a quick swell.
fn sand(t: f32, len: f32) -> f32 {
    let envelope = (t / 0.015).min(1.0) * (-t / len * 4.0).exp();
    noise_burst(t, 0.0, 10.0, 1.0) * envelope * 0.5
}

/// A quick breath of air, for turning things.
fn swish(secs: f32, level: f32) -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(secs, |t| {
        let n = noise(&mut rng);
        let cutoff = 0.4f32.mul_add(t / secs, 0.08);
        lp.0 = (n - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        lp.1 * (TAU * 0.5 * t / secs).sin() * level * 2.2
    })
}

/// Lifting or setting a thing, by what it is made of. Setting is the
/// heavier of the two; lifting is a short brush of the same material.
fn handle(stuff: Stuff, lifting: bool) -> Vec<f32> {
    let level = if lifting { 0.55 } else { 1.0 };
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = 0.0_f32;
    match stuff {
        Stuff::Wood => synth::render(0.16, |t| wood(t, level * 0.9)),
        Stuff::Cloth => synth::render(0.14, |t| {
            lp = (noise(&mut rng) - lp).mul_add(0.1, lp);
            let thump = (TAU * 90.0 * t).sin() * (-t * 45.0).exp() * 0.6;
            (lp * 2.5).mul_add((-t * 30.0).exp(), thump) * level
        }),
        Stuff::Metal => synth::render(0.35, |t| {
            let base = if lifting { 1320.0 } else { 990.0 };
            let peal = ((TAU * base * 2.76 * t).sin() * (-t * 22.0).exp())
                .mul_add(0.25, (TAU * base * t).sin() * (-t * 14.0).exp() * 0.4);
            let hit = knock(t, 0.0, 400.0, 180.0, 0.5);
            (peal + hit) * level
        }),
        Stuff::Clay => synth::render(0.2, |t| {
            let base = if lifting { 1760.0 } else { 1480.0 };
            let clink = (TAU * base * t).sin() * (-t * 40.0).exp() * 0.5;
            let body = knock(t, 0.0, 300.0, 200.0, 0.5);
            (clink + body) * level
        }),
        Stuff::Bone => synth::render(0.12, |t| {
            let click = (TAU * 900.0 * t).sin() * (-t * 70.0).exp() * 0.6;
            let rattle = (TAU * 1400.0 * (t - 0.018).max(0.0)).sin()
                * if t > 0.018 {
                    (-(t - 0.018) * 90.0).exp()
                } else {
                    0.0
                }
                * 0.4;
            (click + rattle) * level
        }),
        Stuff::Paper => synth::render(0.14, |t| {
            let n = noise(&mut rng);
            lp = (n - lp).mul_add(0.5, lp);
            let crinkle = (n - lp) * (-t * 25.0).exp() * (1.0 + (TAU * 60.0 * t).sin()) * 0.5;
            crinkle * level
        }),
        Stuff::Stone => synth::render(0.2, |t| {
            let thud = knock(t, 0.0, 180.0, 110.0, 0.9);
            thud.mul_add(level, sand(t, 0.1) * 0.4 * level)
        }),
    }
}

/// A brass pan taking a weight: a short ring with a wobble.
fn pan() -> Vec<f32> {
    synth::render(0.5, |t| {
        let wobble = (TAU * 7.0 * t).sin().mul_add(0.004, 1.0);
        let ring = ((TAU * 1250.0 * 2.4 * t).sin() * (-t * 16.0).exp()).mul_add(
            0.2,
            (TAU * 1250.0 * wobble * t).sin() * (-t * 9.0).exp() * 0.35,
        );
        ring + knock(t, 0.0, 300.0, 160.0, 0.45)
    })
}

/// The beam turning on its pin: a short wooden creak.
fn beam() -> Vec<f32> {
    synth::render(0.22, |t| {
        // A train of tiny clicks whose rate sags, which is all a creak is.
        let rate = 70.0_f32.mul_add(-t, 90.0);
        let pulse = (TAU * rate * t).sin().max(0.0).powi(8);
        let body = (TAU * 380.0 * t).sin() * pulse;
        body * (TAU * 0.5 * t / 0.22).sin() * 0.7
    })
}

/// Two palms clasped and a bright little bell: a deal.
fn deal() -> Vec<f32> {
    synth::render(1.0, |t| {
        let clasp = noise_burst(t, 0.0, 0.05, 0.7) + noise_burst(t, 0.09, 0.05, 0.5);
        clasp + chime(t, 0.1, A4 * 2.0, 5.0) * 0.3 + chime(t, 0.18, D5 * 2.0, 4.5) * 0.3
    })
}

/// A trader's face, as a little figure in the motive's note.
fn face(emote: Emote) -> Vec<f32> {
    let tone = |t: f32, start: f32, hz: f32, len: f32| {
        let u = t - start;
        if u < 0.0 || u > len {
            return 0.0;
        }
        let env = (u / 0.01).min(1.0) * ((len - u) / 0.03).min(1.0);
        (TAU * hz * 2.0 * u)
            .sin()
            .mul_add(0.2, (TAU * hz * u).sin())
            * env
            * 0.5
    };
    match emote {
        Emote::Love => synth::render(0.4, |t| tone(t, 0.0, FS4, 0.14) + tone(t, 0.14, A4, 0.24)),
        Emote::Bliss => synth::render(0.4, |t| {
            tone(t, 0.0, D5, 0.09) + tone(t, 0.08, FS4 * 2.0, 0.09) + tone(t, 0.16, A4 * 2.0, 0.2)
        }),
        Emote::Repose => synth::render(0.4, |t| {
            let hz = 20.0_f32.mul_add(-t, A3);
            (TAU * hz * t).sin() * (TAU * 0.5 * t / 0.4).sin() * 0.5
        }),
        Emote::Zeal => synth::render(0.35, |t| {
            let i = (t / 0.045).floor();
            let hz = if i as i32 % 2 == 0 { B4 } else { D5 };
            tone(t, i * 0.045, hz * 1.0f32.max(t.mul_add(1.5, 1.0)), 0.04)
        }),
        Emote::Hate => synth::render(0.3, |t| {
            let saw = ((D3 * t) % 1.0).mul_add(2.0, -1.0);
            let square = if (D3 * 1.06 * t) % 1.0 < 0.5 {
                1.0
            } else {
                -1.0
            };
            (saw + square * 0.5) * (-t * 7.0).exp() * 0.35
        }),
        Emote::Pain => synth::render(0.45, |t| {
            let hz = if t < 0.16 { FS4 } else { E4 };
            let vib = (TAU * 6.0 * t).sin().mul_add(0.01, 1.0);
            (TAU * hz * vib * t).sin() * (TAU * 0.5 * t / 0.45).sin() * 0.5
        }),
        Emote::Anima => synth::render(0.7, |t| {
            let wobble = (TAU * 5.0 * t).sin().mul_add(0.05, 1.0);
            let a = (TAU * D4 * wobble * t).sin();
            let b = (TAU * D4 * 1.41 * wobble * t).sin() * 0.6;
            (a + b) * (TAU * 0.5 * t / 0.7).sin() * 0.4
        }),
    }
}

/// Sitting with a thing: a long slow breath in.
fn ponder() -> Vec<f32> {
    const SECS: f32 = 0.8;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        let cutoff = 0.1f32.mul_add(t / SECS, 0.05);
        lp.0 = (n - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let envelope = (t / SECS).powi(2) * ((SECS - t) / 0.15).min(1.0);
        (lp.1 * envelope).mul_add(4.0, chime(t, SECS - 0.2, D5 * 2.0, 8.0) * 0.08)
    })
}

/// A stone dropped into sand: a thing grew heavier.
fn heavier() -> Vec<f32> {
    synth::render(0.45, |t| {
        knock(t, 0.0, 120.0, 70.0, 0.9) + sand(t, 0.3) * 0.5
    })
}

/// Waking: a heartbeat under a rising, wavering glissando.
fn wake() -> Vec<f32> {
    const SECS: f32 = 1.3;
    let mut phase = 0.0_f32;
    synth::render(SECS, |t| {
        let hz = D4 * (1.0 + t / SECS) * (TAU * 6.0 * t).sin().mul_add(0.03, 1.0);
        phase += TAU * hz / SAMPLE_RATE as f32;
        let gliss = phase.sin() * (t / SECS) * ((SECS - t) / 0.3).min(1.0) * 0.4;
        let beat = knock(t, 0.0, 90.0, 55.0, 0.8) + knock(t, 0.18, 90.0, 55.0, 0.6);
        gliss + beat
    })
}

/// Little feet in the hold.
fn skitter() -> Vec<f32> {
    synth::render(0.3, |t| {
        [0.0, 0.05, 0.09, 0.15, 0.19, 0.24]
            .iter()
            .enumerate()
            .map(|(i, &at)| {
                let u = t - at;
                if u < 0.0 {
                    0.0
                } else {
                    let hz = if i % 2 == 0 { 2200.0 } else { 1900.0 };
                    (TAU * hz * u).sin() * (-u * 120.0).exp() * 0.5
                }
            })
            .sum()
    })
}

/// A swallow of water: two bubbly pitch sweeps.
fn sip() -> Vec<f32> {
    let bubble = |t: f32, start: f32, from: f32| {
        let u = t - start;
        if u < 0.0 {
            return 0.0;
        }
        let hz = from * u.mul_add(8.0, 1.0);
        (TAU * hz * u).sin() * (-u * 30.0).exp()
    };
    synth::render(0.3, |t| {
        f32::midpoint(bubble(t, 0.0, 380.0), bubble(t, 0.11, 460.0) * 0.8)
    })
}

/// A dry rasp.
fn thirst() -> Vec<f32> {
    const SECS: f32 = 0.5;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut hp = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        hp = (n - hp).mul_add(0.3, hp);
        let grain = (TAU * 35.0 * t).sin().max(0.0);
        (n - hp) * grain * (TAU * 0.5 * t / SECS).sin() * 0.6
    })
}

/// Canvas filling: a gust, and the sail cracking taut.
fn sail() -> Vec<f32> {
    const SECS: f32 = 0.9;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.07, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.07, lp.1);
        let gust = lp.1 * 5.0 * (TAU * 0.5 * t / SECS).sin();
        let flap = (TAU * 13.0 * t).sin().max(0.0) * lp.0 * 1.5 * (-t * 4.0).exp();
        let crack = noise_burst(t, 0.28, 0.04, 0.5) + knock(t, 0.28, 160.0, 90.0, 0.4);
        gust + flap + crack
    })
}

/// Mooring: stakes knocked in and a rope creaking tight.
fn moor() -> Vec<f32> {
    synth::render(0.9, |t| {
        let stakes = wood(t, 0.9) + wood(t - 0.2, 0.7);
        let u = t - 0.35;
        let creak = if u > 0.0 {
            let pulse = (TAU * 55.0 * u).sin().max(0.0).powi(6);
            (TAU * 300.0 * u).sin() * pulse * (-u * 5.0).exp() * 0.5
        } else {
            0.0
        };
        stakes + creak
    })
}

/// A whoomph of catching fire, then crackling.
fn burn() -> Vec<f32> {
    const SECS: f32 = 1.6;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0xF1);
    let mut crackle = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        let cutoff = 0.2f32.mul_add((-t * 3.0).exp(), 0.04);
        lp.0 = (n - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let whoomph = lp.1 * 6.0 * (t / 0.08).min(1.0) * (-t * 2.2).exp();
        // Now and then a pop, decaying fast.
        if pops.f32() < 0.0012 * (SECS - t) {
            crackle = pops.f32().mul_add(0.5, 0.5);
        }
        crackle *= 0.992;
        (n * crackle).mul_add(0.5, whoomph)
    })
}

/// Water glugging into jars.
fn pour() -> Vec<f32> {
    const SECS: f32 = 1.0;
    synth::render(SECS, |t| {
        (0..8)
            .map(|i| {
                let start = i as f32 * 0.11;
                let u = t - start;
                if u < 0.0 {
                    return 0.0;
                }
                // Each glug a little higher, as the jar fills.
                let hz = (i as f32).mul_add(35.0, 260.0) * (1.0 + u * 6.0);
                (TAU * hz * u).sin() * (-u * 25.0).exp() * 0.4
            })
            .sum::<f32>()
            * ((SECS - t) / 0.2).min(1.0)
    })
}

/// Distant guns: three low booms rolling into each other.
fn guns() -> Vec<f32> {
    const SECS: f32 = 2.4;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.02, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.02, lp.1);
        [0.0, 0.5, 1.1]
            .iter()
            .map(|&at| {
                let u = t - at;
                if u < 0.0 {
                    0.0
                } else {
                    (lp.1 * 12.0).mul_add((-u * 3.0).exp(), knock(t, at, 70.0, 38.0, 0.8))
                }
            })
            .sum::<f32>()
            * 0.6
    })
}

/// Far-off fireworks: soft pops and a fizz.
fn fireworks() -> Vec<f32> {
    synth::render(2.0, |t| {
        [0.0, 0.35, 0.5, 0.9, 1.25]
            .iter()
            .enumerate()
            .map(|(i, &at)| {
                let pop = knock(t, at, 240.0, 120.0, 0.5);
                let fizz = noise_burst(t, at + 0.05, 0.4, 0.12);
                let sparkle = chime(t, at + 0.06, [D5, B4, A4, FS4, D5][i] * 2.0, 6.0) * 0.08;
                pop + fizz + sparkle
            })
            .sum()
    })
}

/// The dirigible's engines: a beating drone that swells and passes.
fn drone() -> Vec<f32> {
    const SECS: f32 = 3.0;
    synth::render(SECS, |t| {
        let beat = (TAU * 3.0 * t).sin().mul_add(0.3, 0.7);
        let hum = (TAU * 116.5 * t).sin().mul_add(0.5, (TAU * 58.0 * t).sin());
        hum * beat * (TAU * 0.5 * t / SECS).sin() * 0.45
    })
}

/// Wind over the basin, made to loop: noise through a slowly breathing
/// lowpass, with its tail folded onto its head so the seam does not show.
fn wind() -> Vec<f32> {
    const SECS: f32 = 6.0;
    const FOLD: f32 = 1.0;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let total = SECS + FOLD;
    // Rendered raw, without synth::render's end fades: the fold is the fade.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (total * SAMPLE_RATE as f32) as usize;
    let raw: Vec<f32> = (0..count)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            // Whole cycles over SECS, so the breathing loops too.
            let breath = (TAU * t / SECS).sin().mul_add(0.4, 0.6)
                * (TAU * 3.0 * t / SECS).sin().mul_add(0.2, 0.8);
            let cutoff = breath.mul_add(0.03, 0.01);
            lp.0 = (noise(&mut rng) - lp.0).mul_add(cutoff, lp.0);
            lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
            lp.1 * breath * 7.0
        })
        .collect();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (body, fold) = (
        (SECS * SAMPLE_RATE as f32) as usize,
        (FOLD * SAMPLE_RATE as f32) as usize,
    );
    let mut out: Vec<f32> = raw[..body].to_vec();
    for i in 0..fold {
        let w = i as f32 / fold as f32;
        out[i] = raw[i].mul_add(w, raw[body + i] * (1.0 - w));
    }
    out
}

#[cfg(test)]
// Tests turn positive seconds into sample counts.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod tests {
    use super::*;
    use crate::leitmotif::mix::Mixer;

    /// Loudest any pile-up may get.
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
            assert!(loudest > 0.03, "{sfx:?} is nearly silent ({loudest})");
            assert!(loudest < HEADROOM, "{sfx:?} peaks at {loudest}");
            let pinned = samples.iter().filter(|s| s.abs() > 0.999).count();
            assert!(pinned * 200 < samples.len(), "{sfx:?} clips in its buffer");
        }
    }

    #[test]
    fn every_one_shot_starts_and_ends_silently() {
        for sfx in Sfx::all().into_iter().filter(|&s| s != Sfx::Wind) {
            let samples = decoded(sfx);
            let ends = [samples[0], samples[samples.len() - 1]];
            assert!(
                ends.iter().all(|s| s.abs() < 2e-3),
                "{sfx:?} clicks: {ends:?}"
            );
        }
    }

    #[test]
    fn the_wind_loops_without_a_seam() {
        let samples = decoded(Sfx::Wind);
        let (first, last) = (samples[0], samples[samples.len() - 1]);
        assert!(
            (first - last).abs() < 0.02,
            "the loop jumps {first} -> {last}"
        );
    }

    #[test]
    fn handling_sounds_are_quick() {
        for stuff in Stuff::ALL {
            for sfx in [Sfx::Lift(stuff), Sfx::Set(stuff)] {
                let secs = decoded(sfx).len() as f32 / SAMPLE_RATE as f32;
                assert!(secs <= 0.4, "{sfx:?} lasts {secs} s");
            }
        }
    }

    #[test]
    fn a_busy_moment_does_not_clip() {
        // The worst the game does at once: a deal with faces flying, a
        // dawn with everything growing and a thing waking, water, guns
        // far off, over the wind — and a player clicking through all of it.
        let sounds: Vec<(Sfx, Vec<f32>)> =
            Sfx::all().into_iter().map(|s| (s, decoded(s))).collect();
        let play = |mixer: &mut Mixer, at: f32, sfx: Sfx| {
            let (_, samples) = sounds.iter().find(|(s, _)| *s == sfx).unwrap();
            mixer.play(at, samples, sfx.gain());
        };
        let mut mixer = Mixer::new();
        play(&mut mixer, 0.0, Sfx::Wind);
        for step in 0..12 {
            let at = step as f32 * 0.08;
            play(
                &mut mixer,
                at,
                Sfx::Set(Stuff::ALL[step % Stuff::ALL.len()]),
            );
            play(&mut mixer, at + 0.02, Sfx::Lift(Stuff::Metal));
            play(&mut mixer, at + 0.03, Sfx::Tick);
        }
        play(&mut mixer, 0.2, Sfx::Pan);
        play(&mut mixer, 0.22, Sfx::Beam);
        play(&mut mixer, 0.25, Sfx::Face(Emote::Hate));
        play(&mut mixer, 0.3, Sfx::Deal);
        play(&mut mixer, 0.3, Sfx::Dawn);
        play(&mut mixer, 0.3, Sfx::Sip);
        play(&mut mixer, 0.32, Sfx::Heavier);
        play(&mut mixer, 0.34, Sfx::Heavier);
        play(&mut mixer, 0.35, Sfx::Wake);
        play(&mut mixer, 0.4, Sfx::Skitter);
        play(&mut mixer, 0.4, Sfx::Guns);
        play(&mut mixer, 0.5, Sfx::Stone(Motive::Hate));
        play(&mut mixer, 0.5, Sfx::Circuit);
        let loudest = mixer.peak();
        println!("sand nomad pile-up peak: {loudest:.3}");
        assert!(loudest <= HEADROOM, "a busy moment peaks at {loudest}");
    }

    #[test]
    fn generation_is_reproducible() {
        for sfx in [Sfx::Wind, Sfx::Burn, Sfx::Sail, Sfx::Lift(Stuff::Paper)] {
            assert_eq!(render(sfx), render(sfx), "{sfx:?}");
        }
    }

    #[test]
    fn motives_each_have_their_own_note() {
        for (i, a) in Motive::ALL.iter().enumerate() {
            for b in &Motive::ALL[i + 1..] {
                assert!((note(*a) - note(*b)).abs() > 1.0);
            }
        }
    }
}
