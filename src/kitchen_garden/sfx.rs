//! What the kitchen sounds like. No music: wood, clay, iron, water, fire,
//! coins, and a garden through the open door.
//!
//! Pure and macroquad-free like the other synths here, so the tests below
//! check every sound is a WAV the browser will decode and that the worst
//! pile-up a player can cause stays under full scale. The frontend bakes
//! one buffer per [`Sfx`] and plays it at [`Sfx::gain`].
//!
//! Work is done by holding the mouse down, and every stroke of the knife,
//! the millstones or the whisk replays the same baked buffer up to seven
//! times a second, so strokes are short, soft and rounded: a lowpass on
//! every noise, nothing above a gentle click. Everything pitched belongs
//! to one G major pentatonic, so a finished pot, a customer's bell, the
//! coins on the counter and the birds outside all agree.

use std::f32::consts::{PI, TAU};

use crate::leitmotif::synth::{self, SAMPLE_RATE};

/// Loudest a single sound plays; the per-sound gains are fractions of it.
pub const MASTER: f32 = 0.5;

/// Fixed noise seed, so a build always sounds the same.
const SEED: u64 = 0x6A2D_E4C0;

/// The kitchen's scale, G major pentatonic, low to high.
const G2: f32 = 98.0;
const A2: f32 = 110.0;
const B2: f32 = 123.47;
const G3: f32 = 196.0;
const A3: f32 = 220.0;
const B3: f32 = 246.94;
const D4: f32 = 293.66;
const E4: f32 = 329.63;
const G4: f32 = 392.0;
const B4: f32 = 493.88;
const D5: f32 = 587.33;
const E5: f32 = 659.26;
const G5: f32 = 783.99;
const B5: f32 = 987.77;
const D6: f32 = 1174.66;
const E6: f32 = 1318.51;
const G6: f32 = 1567.98;
const B6: f32 = 1975.53;
const D7: f32 = 2349.32;
const E7: f32 = 2637.02;
const G7: f32 = 3135.96;

/// One sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    /// Picking food up.
    Lift,
    /// Setting food down: on a shelf, into a station.
    Set,
    /// It will not go there / cannot be done.
    Nope,
    /// The pointer settling on something new.
    Tick,
    /// One stroke of the knife on the board.
    Chop,
    /// One turn of the millstones.
    Grind,
    /// One push of dough in the bowl.
    Knead,
    /// One turn of the whisk in the bowl.
    Mix,
    /// One beat of the churn.
    Churn,
    /// One turn of the well's crank (a wooden creak).
    Crank,
    /// The bucket comes up full.
    Splash,
    /// One pull of milk into the pail.
    Squirt,
    /// One stroke of the axe.
    Axe,
    /// A log splits off the block.
    Split,
    /// A fresh log catches on the hearth (whoomph).
    Ignite,
    /// The pot comes to the boil.
    Bubble,
    /// Food hits the hot pan.
    Sizzle,
    /// The oven door shuts.
    Oven,
    /// The crock's lid goes on (clay clink).
    Lid,
    /// A station finished: a small bright bell.
    Done,
    /// Something on the hearth is about to burn: a warning hiss.
    Smoking,
    /// It burned: a dull crackling thump.
    Burnt,
    /// It came out as mush: a squelch.
    Mush,
    /// A seed into the soil.
    Plant,
    /// Pulling a crop.
    Pluck,
    /// A plot came ripe: a soft sparkle.
    Ripe,
    /// The hen laid: a little cluck.
    Cluck,
    /// The cow, gently.
    Moo,
    /// A customer at the hatch: a shop bell.
    Bell,
    /// Served and paid: coins.
    Coins,
    /// ...and a tip on top: a sparkle.
    Tip,
    /// A customer gave up and left: a sigh/low falling tone.
    Huff,
    /// A customer growing impatient: a couple of taps.
    Tap,
    /// Coins spent at the market.
    Buy,
    /// Something onto the compost heap.
    Compost,
    /// Morning.
    Dawn,
    /// Evening: the day is done.
    Dusk,
    /// One of the three marks reached at dusk, `0..3`, rising.
    Mark(u8),
    /// The hearth crackling, looped.
    Fire,
    /// The garden outside: a breeze and the odd bird, looped.
    Garden,
}

impl Sfx {
    /// Every sound, for baking.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all = vec![
            Self::Lift,
            Self::Set,
            Self::Nope,
            Self::Tick,
            Self::Chop,
            Self::Grind,
            Self::Knead,
            Self::Mix,
            Self::Churn,
            Self::Crank,
            Self::Splash,
            Self::Squirt,
            Self::Axe,
            Self::Split,
            Self::Ignite,
            Self::Bubble,
            Self::Sizzle,
            Self::Oven,
            Self::Lid,
            Self::Done,
            Self::Smoking,
            Self::Burnt,
            Self::Mush,
            Self::Plant,
            Self::Pluck,
            Self::Ripe,
            Self::Cluck,
            Self::Moo,
            Self::Bell,
            Self::Coins,
            Self::Tip,
            Self::Huff,
            Self::Tap,
            Self::Buy,
            Self::Compost,
            Self::Dawn,
            Self::Dusk,
        ];
        for mark in 0..3 {
            all.push(Self::Mark(mark));
        }
        all.extend([Self::Fire, Self::Garden]);
        all
    }

    /// Playback volume, `0..=1`. Lives here so the tests measure what the
    /// speakers get.
    #[must_use]
    pub const fn gain(self) -> f32 {
        let share = match self {
            Self::Tick | Self::Garden => 0.25,
            Self::Fire => 0.4,
            Self::Grind | Self::Mix | Self::Crank | Self::Squirt => 0.45,
            Self::Lift
            | Self::Nope
            | Self::Chop
            | Self::Knead
            | Self::Churn
            | Self::Bubble
            | Self::Sizzle
            | Self::Plant
            | Self::Pluck
            | Self::Ripe
            | Self::Cluck
            | Self::Tap => 0.5,
            Self::Set
            | Self::Axe
            | Self::Splash
            | Self::Lid
            | Self::Smoking
            | Self::Mush
            | Self::Moo
            | Self::Huff
            | Self::Compost => 0.55,
            Self::Ignite | Self::Oven | Self::Burnt | Self::Tip | Self::Buy | Self::Dawn => 0.6,
            Self::Split => 0.65,
            Self::Done | Self::Bell | Self::Coins => 0.75,
            Self::Dusk | Self::Mark(_) => 0.8,
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
        Sfx::Lift => synth::render(0.12, |t| {
            knock(t, 0.0, 200.0, E4, 0.65) + noise_burst(t, 0.0, 0.02, 0.1)
        }),
        Sfx::Set => synth::render(0.16, |t| wood(t, G3, 0.8) + noise_burst(t, 0.0, 0.012, 0.1)),
        Sfx::Nope => synth::render(0.22, |t| {
            knock(t, 0.0, 170.0, B2, 0.8) + knock(t, 0.08, 150.0, G2, 0.65)
        }),
        Sfx::Tick => synth::render(0.03, |t| (TAU * D7 * t).sin() * (-t * 180.0).exp() * 1.2),
        Sfx::Chop => chop(),
        Sfx::Grind => grind(),
        Sfx::Knead => knead(),
        Sfx::Mix => mix(),
        Sfx::Churn => churn(),
        Sfx::Crank => crank(),
        Sfx::Splash => splash(),
        Sfx::Squirt => squirt(),
        Sfx::Axe => synth::render(0.2, |t| {
            knock(t, 0.0, 320.0, A2, 0.8) + wood(t, D4, 0.35) + noise_burst(t, 0.0, 0.01, 0.2)
        }),
        Sfx::Split => synth::render(0.6, |t| {
            let crack = knock(t, 0.0, 420.0, B2, 0.6) + noise_burst(t, 0.0, 0.03, 0.35);
            // The two halves tumbling off the block.
            let halves =
                wood(t - 0.2, G3, 0.6) + wood(t - 0.29, A3, 0.45) + wood(t - 0.37, G3, 0.2);
            crack + halves
        }),
        Sfx::Ignite => ignite(),
        Sfx::Bubble => bubble(),
        Sfx::Sizzle => sizzle(),
        Sfx::Oven => oven(),
        Sfx::Lid => synth::render(0.35, |t| {
            let rock = clink(t, 0.07, E6, 45.0, 0.15) + clink(t, 0.11, E6, 45.0, 0.07);
            clink(t, 0.0, E6, 40.0, 0.3) + knock(t, 0.0, 320.0, D4, 0.3) + rock
        }),
        Sfx::Done => synth::render(1.1, |t| {
            chime(t, 0.0, D6, 5.0).mul_add(0.4, chime(t, 0.1, G6, 4.0) * 0.4)
        }),
        Sfx::Smoking => smoking(),
        Sfx::Burnt => burnt(),
        Sfx::Mush => mush(),
        Sfx::Plant => plant(),
        Sfx::Pluck => pluck(),
        Sfx::Ripe => synth::render(0.9, |t| {
            [G6, B6, D7]
                .iter()
                .enumerate()
                .map(|(i, &hz)| chime(t, i as f32 * 0.07, hz, 6.0) * 0.3)
                .sum()
        }),
        Sfx::Cluck => cluck(),
        Sfx::Moo => moo(),
        Sfx::Bell => synth::render(1.4, |t| {
            shop_bell(t, 0.0, B5, 0.55)
                + shop_bell(t, 0.12, B5, 0.35)
                + shop_bell(t, 0.22, B5, 0.15)
        }),
        Sfx::Coins => synth::render(0.7, |t| {
            let counter = knock(t, 0.0, 300.0, D4, 0.25);
            [
                (0.0, E6, 0.45),
                (0.05, G6, 0.38),
                (0.11, D6, 0.33),
                (0.16, B6, 0.27),
                (0.24, E6, 0.2),
                (0.31, G6, 0.12),
            ]
            .iter()
            .map(|&(at, hz, level)| clink(t, at, hz, 28.0, level))
            .sum::<f32>()
                + counter
        }),
        Sfx::Tip => synth::render(0.8, |t| {
            [G6, B6, D7, G7]
                .iter()
                .enumerate()
                .map(|(i, &hz)| chime(t, i as f32 * 0.05, hz, 7.0) * 0.28)
                .sum()
        }),
        Sfx::Huff => huff(),
        Sfx::Tap => synth::render(0.32, |t| wood(t, A3, 0.6) + wood(t - 0.14, A3, 0.5)),
        Sfx::Buy => buy(),
        Sfx::Compost => compost(),
        Sfx::Dawn => dawn(),
        Sfx::Dusk => dusk(),
        Sfx::Mark(i) => {
            let hz = mark_note(i);
            synth::render(1.4, |t| {
                chime(t, 0.0, hz, 3.0).mul_add(0.4, chime(t, 0.0, hz * 0.5, 2.5) * 0.25)
            })
        }
        Sfx::Fire => fire(),
        Sfx::Garden => garden(),
    }
}

/// The note of the `i`th mark reached at dusk: G, B, then D, climbing.
fn mark_note(i: u8) -> f32 {
    [G5, B5, D6][usize::from(i).min(2)]
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

/// Phase, in radians, `u` seconds into a pitch gliding exponentially from
/// `from` to `to` Hz at `rate` per second.
fn glide(u: f32, from: f32, to: f32, rate: f32) -> f32 {
    TAU * to.mul_add(u, (from - to) / rate * (1.0 - (-u * rate).exp()))
}

/// A pitch sliding quickly from `from` to `to` Hz, struck at `start`.
fn knock(t: f32, start: f32, from: f32, to: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    glide(u, from, to, 30.0).sin() * (-u * 28.0).exp() * level
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

/// A small hard clink, a coin or a glazed lid: a high note with one
/// clangorous partial over it, dying at `decay` per second.
fn clink(t: f32, start: f32, hz: f32, decay: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let ring = (TAU * hz * 2.76 * u).sin() * (-u * decay * 1.8).exp();
    ring.mul_add(0.3, (TAU * hz * u).sin() * (-u * decay).exp()) * level
}

/// A hollow wooden knock at `u` seconds in: a body at `hz` and a tock a
/// twelfth above it, which keeps it in the scale.
fn wood(u: f32, hz: f32, level: f32) -> f32 {
    if u < 0.0 {
        return 0.0;
    }
    let body = (TAU * hz * u).sin() * (-u * 40.0).exp();
    let tock = (TAU * hz * 3.0 * u).sin() * (-u * 90.0).exp();
    tock.mul_add(0.5, body) * level
}

/// A bubble rising to the surface: a short upward sweep from `hz`.
fn blip(t: f32, start: f32, hz: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    // The pitch is `hz * (1 + 8u)`; this is its integral.
    let phase = TAU * hz * (4.0 * u).mul_add(u, u);
    phase.sin() * (-u * 30.0).exp() * level
}

/// A small brass shop bell: a bright fundamental, its octave, and the
/// inharmonic shimmer that makes it brass.
fn shop_bell(t: f32, start: f32, hz: f32, level: f32) -> f32 {
    let u = t - start;
    if u < 0.0 {
        return 0.0;
    }
    let partial = |ratio: f32, decay: f32, level: f32| {
        (TAU * hz * ratio * u).sin() * (-u * decay).exp() * level
    };
    let ring = partial(1.0, 3.0, 1.0) + partial(2.0, 5.0, 0.4);
    let shimmer = partial(2.76, 14.0, 0.15) + partial(5.4, 25.0, 0.05);
    (ring + shimmer) * level
}

/// Render `secs` of a looping voice: `fold` seconds more are rendered and
/// laid back over the head, so the end runs straight into the start. Any
/// periodic modulation in `voice` must make whole cycles over `secs`.
fn looped(secs: f32, fold: f32, mut voice: impl FnMut(f32) -> f32) -> Vec<f32> {
    // Rendered raw, without synth::render's end fades: the fold is the fade.
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

/// The knife: a soft edge of noise into the board, and the board's knock.
fn chop() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = 0.0_f32;
    synth::render(0.12, |t| {
        lp = (noise(&mut rng) - lp).mul_add(0.4, lp);
        let edge = lp * (-t * 150.0).exp() * 0.6;
        let board = (TAU * B5 * t).sin() * (-t * 140.0).exp();
        let body = (TAU * D4 * t).sin() * (-t * 50.0).exp();
        board.mul_add(0.15, body.mul_add(0.5, edge))
    })
}

/// The millstones: a low gritty rumble that catches and lets go.
fn grind() -> Vec<f32> {
    const SECS: f32 = 0.22;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.12, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.12, lp.1);
        // Stone dragging over stone catches and slips a few dozen times a second.
        let catch = (TAU * 28.0 * t).sin().abs().mul_add(0.7, 0.3);
        let rumble = (TAU * G2 * t).sin() * 0.25;
        lp.1.mul_add(3.0, rumble) * catch * (PI * t / SECS).sin()
    })
}

/// Dough pushed into the bowl: a soft squash with a low thud under it.
fn knead() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(0.2, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.06, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.06, lp.1);
        let press = (t / 0.03).min(1.0) * (-t * 14.0).exp();
        lp.1.mul_add(6.0 * press, knock(t, 0.0, 130.0, G2, 0.45))
    })
}

/// The whisk going round: a swirl of air and a wire touching the bowl.
fn mix() -> Vec<f32> {
    const SECS: f32 = 0.2;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.35, lp.0);
        lp.1 = (n - lp.1).mul_add(0.06, lp.1);
        let swirl = (PI * t / SECS).sin().powi(2);
        (lp.0 - lp.1).mul_add(swirl * 1.2, chime(t, 0.13, G6, 40.0) * 0.06)
    })
}

/// The churn's dasher: a wooden thump and cream sloshing round it.
fn churn() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(0.22, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.08, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.08, lp.1);
        let slosh = lp.1 * (t / 0.02).min(1.0) * (-t * 18.0).exp() * 4.0;
        knock(t, 0.0, 160.0, A2, 0.6) + slosh + blip(t, 0.05, B3, 0.25)
    })
}

/// The well's crank turning: a short wooden creak.
fn crank() -> Vec<f32> {
    const SECS: f32 = 0.22;
    synth::render(SECS, |t| {
        // A train of tiny clicks whose rate sags, which is all a creak is.
        let rate = 60.0_f32.mul_add(-t, 80.0);
        let pulse = (TAU * rate * t).sin().max(0.0).powi(8);
        let body = (TAU * G4 * t).sin() * pulse;
        body * (PI * t / SECS).sin() * 0.7
    })
}

/// The bucket breaking the surface: a slap, a wash of water and a few
/// drops falling back.
fn splash() -> Vec<f32> {
    const SECS: f32 = 0.7;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        let cutoff = 0.25f32.mul_add((-t * 6.0).exp(), 0.05);
        lp.0 = (n - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let wash = lp.1 * 2.5 * (t / 0.01).min(1.0) * (-t * 6.0).exp();
        let drops = blip(t, 0.18, D4, 0.2)
            + blip(t, 0.27, G4, 0.15)
            + blip(t, 0.36, E4, 0.12)
            + blip(t, 0.47, B4, 0.08);
        wash + drops + knock(t, 0.0, 180.0, G2, 0.4)
    })
}

/// Milk into the pail: a hiss of liquid and the tin ringing faintly.
fn squirt() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(0.18, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.3, lp.0);
        lp.1 = (n - lp.1).mul_add(0.04, lp.1);
        let env = (t / 0.01).min(1.0) * (-t * 14.0).exp();
        (lp.0 - lp.1).mul_add(env * 1.5, chime(t, 0.01, B5, 18.0) * 0.08)
    })
}

/// A log catching: a whoomph of air, then a little crackling.
fn ignite() -> Vec<f32> {
    const SECS: f32 = 1.0;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0xF1);
    let mut crackle = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        let cutoff = 0.15f32.mul_add((-t * 3.0).exp(), 0.03);
        lp.0 = (n - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let whoomph = lp.1 * 6.0 * (t / 0.1).min(1.0) * (-t * 3.0).exp();
        // Now and then a pop, decaying fast.
        if pops.f32() < 0.0008 * (SECS - t) {
            crackle = pops.f32().mul_add(0.4, 0.3);
        }
        crackle *= 0.99;
        (lp.0 * crackle).mul_add(1.5, whoomph)
    })
}

/// The pot coming to the boil: bubbles over a low simmer.
fn bubble() -> Vec<f32> {
    const SECS: f32 = 1.0;
    const BUBBLES: [(f32, f32); 9] = [
        (0.04, G3),
        (0.15, B3),
        (0.24, D4),
        (0.36, A3),
        (0.43, E4),
        (0.54, G3),
        (0.61, D4),
        (0.72, B3),
        (0.8, G3),
    ];
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.03, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.03, lp.1);
        let simmer = lp.1 * 6.0 * (PI * t / SECS).sin();
        BUBBLES
            .iter()
            .map(|&(at, hz)| blip(t, at, hz, 0.35))
            .sum::<f32>()
            + simmer
    })
}

/// Food hitting the hot pan: a soft thump and a hiss that settles.
fn sizzle() -> Vec<f32> {
    const SECS: f32 = 1.1;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0x55);
    let mut spit = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        // The top end rounded off, the bottom taken away: a soft hiss.
        lp.0 = (n - lp.0).mul_add(0.5, lp.0);
        lp.1 = (n - lp.1).mul_add(0.08, lp.1);
        if pops.f32() < 0.0006 {
            spit = pops.f32().mul_add(0.5, 0.3);
        }
        spit *= 0.995;
        let env = (t / 0.01).min(1.0) * (-t * 2.2).exp() * ((SECS - t) / 0.2).min(1.0);
        (lp.0 - lp.1).mul_add(env * (0.7 + spit), knock(t, 0.0, 180.0, G2, 0.45))
    })
}

/// The oven door: an iron thump, the door ringing, the latch dropping.
fn oven() -> Vec<f32> {
    synth::render(0.7, |t| {
        let partial = |ratio: f32, decay: f32, level: f32| {
            (TAU * G3 * ratio * t).sin() * (-t * decay).exp() * level
        };
        let ring =
            (partial(1.0, 7.0, 1.0) + partial(2.76, 12.0, 0.4) + partial(5.4, 20.0, 0.2)) * 0.25;
        let latch = noise_burst(t, 0.14, 0.01, 0.2) + chime(t, 0.14, D6, 40.0) * 0.12;
        knock(t, 0.0, 150.0, G2, 0.9) + ring + latch
    })
}

/// A warning hiss: smoke puffing off something left too long.
fn smoking() -> Vec<f32> {
    const SECS: f32 = 0.8;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.25, lp.0);
        lp.1 = (n - lp.1).mul_add(0.03, lp.1);
        // Four whole puffs over the sound.
        let puffs = (TAU * 5.0 * t).sin().mul_add(0.35, 0.65);
        (lp.0 - lp.1) * puffs * (PI * t / SECS).sin() * 1.5
    })
}

/// Burnt: a dull thump, a puff of char and a dying crackle.
fn burnt() -> Vec<f32> {
    const SECS: f32 = 0.7;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0xB0);
    let mut crackle = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.04, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.04, lp.1);
        let puff = lp.1 * 5.0 * (t / 0.02).min(1.0) * (-t * 5.0).exp();
        if pops.f32() < 0.0015 * (SECS - t) {
            crackle = pops.f32().mul_add(0.4, 0.3);
        }
        crackle *= 0.99;
        let crackling = (n * crackle).mul_add(0.4, puff);
        knock(t, 0.0, 120.0, G2, 0.8) + crackling
    })
}

/// Mush: a wet, wobbling squelch that sags.
fn mush() -> Vec<f32> {
    const SECS: f32 = 0.4;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut phase = 0.0_f32;
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.1, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.1, lp.1);
        let wobble = (TAU * 22.0 * t).sin().mul_add(0.08, 1.0);
        let hz = G3 * 0.6f32.mul_add((-t * 12.0).exp(), 1.0) * wobble;
        phase += TAU * hz / SAMPLE_RATE as f32;
        let env = (t / 0.02).min(1.0) * (-t * 9.0).exp();
        let wet = lp.1.mul_add(3.0, 0.6);
        phase.sin() * wet * env * 0.9
    })
}

/// A seed into the soil: two soft pats of earth.
fn plant() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = 0.0_f32;
    synth::render(0.25, |t| {
        lp = (noise(&mut rng) - lp).mul_add(0.12, lp);
        [(0.0, 1.0), (0.09, 0.7)]
            .iter()
            .map(|&(at, level)| {
                let u = t - at;
                if u < 0.0 {
                    return 0.0;
                }
                lp.mul_add(1.5 * (-u * 35.0).exp(), knock(t, at, 150.0, G2, 0.3)) * level
            })
            .sum()
    })
}

/// Pulling a crop: a tug through the soil, a pop, and earth falling off.
fn pluck() -> Vec<f32> {
    const PULL: f32 = 0.12;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = 0.0_f32;
    synth::render(0.3, |t| {
        let cutoff = 0.15f32.mul_add((t / PULL).min(1.0), 0.03);
        lp = (noise(&mut rng) - lp).mul_add(cutoff, lp);
        let tug = if t < PULL { (t / PULL).powi(2) } else { 0.0 };
        let pop = knock(t, PULL, 250.0, D5, 0.5);
        lp.mul_add(tug, pop) + noise_burst(t, PULL + 0.01, 0.06, 0.1)
    })
}

/// The hen: two little clucks and a pleased one.
fn cluck() -> Vec<f32> {
    let buk = |t: f32, start: f32, from: f32, to: f32, len: f32| {
        let u = t - start;
        if u < 0.0 || u > len {
            return 0.0;
        }
        let phase = glide(u, from, to, 25.0);
        let env = (u / 0.006).min(1.0) * ((len - u) / 0.02).min(1.0) * (-u * 8.0).exp();
        // A buzzy little voice: the first four harmonics, falling away.
        let voice = [1.0, 0.5, 0.3, 0.15]
            .iter()
            .enumerate()
            .map(|(k, &level)| ((k + 1) as f32 * phase).sin() * level)
            .sum::<f32>();
        voice * env * 0.5
    };
    synth::render(0.5, |t| {
        buk(t, 0.0, D5, B4, 0.06) + buk(t, 0.1, D5, B4, 0.06) + buk(t, 0.22, E5, G4, 0.16)
    })
}

/// The cow, gently: a low moo that opens up and closes again, from G2 to
/// D3 and back.
fn moo() -> Vec<f32> {
    const SECS: f32 = 1.3;
    let mut phase = 0.0_f32;
    synth::render(SECS, |t| {
        let arc = (PI * t / SECS).sin();
        let vibrato = (TAU * 5.0 * t).sin().mul_add(0.01, 1.0);
        let hz = G2 * arc.mul_add(0.5, 1.0) * vibrato;
        phase += TAU * hz / SAMPLE_RATE as f32;
        // The mouth opening: higher harmonics come up as it does.
        let open = arc.mul_add(0.5, 0.3);
        let voice: f32 = (1..=6)
            .map(|k| {
                let k = k as f32;
                (k * phase).sin() * open.powf(k - 1.0) / k
            })
            .sum();
        let env = (t / 0.15).min(1.0) * ((SECS - t) / 0.3).min(1.0);
        voice * env * 0.55
    })
}

/// A sigh, and a low tone falling from D4 to G3.
fn huff() -> Vec<f32> {
    const SECS: f32 = 0.9;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(SECS, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.1, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.1, lp.1);
        let env = (PI * t / SECS).sin();
        let tone = glide(t, D4, G3, 4.0).sin() * 0.3;
        lp.1.mul_add(3.0, tone) * env
    })
}

/// Paying at the market: coins leaving, and the purse closing.
fn buy() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = 0.0_f32;
    synth::render(0.55, |t| {
        lp = (noise(&mut rng) - lp).mul_add(0.1, lp);
        let coins = clink(t, 0.0, G6, 28.0, 0.25)
            + clink(t, 0.07, D6, 28.0, 0.2)
            + clink(t, 0.14, B5, 28.0, 0.15);
        let u = (t - 0.28).max(0.0);
        let purse = if t < 0.28 {
            0.0
        } else {
            let thud = (TAU * G2 * u).sin().mul_add(0.6, lp * 2.0);
            thud * (-u * 35.0).exp()
        };
        coins + purse * 0.6
    })
}

/// Onto the heap: a wet thud and a settling rustle.
fn compost() -> Vec<f32> {
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    synth::render(0.45, |t| {
        let n = noise(&mut rng);
        lp.0 = (n - lp.0).mul_add(0.06, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.06, lp.1);
        let rustle = lp.1 * 5.0 * (t / 0.01).min(1.0) * (-t * 8.0).exp();
        knock(t, 0.0, 140.0, G2, 0.6) + rustle + blip(t, 0.03, G3, 0.12)
    })
}

/// Morning: a warm swell under three chimes climbing G-B-D.
fn dawn() -> Vec<f32> {
    const SECS: f32 = 1.8;
    synth::render(SECS, |t| {
        let swell = (PI * t / SECS).sin().powi(2);
        let pad = (TAU * D5 * t).sin().mul_add(0.6, (TAU * G4 * t).sin()) * swell * 0.15;
        let chimes: f32 = [G5, B5, D6]
            .iter()
            .enumerate()
            .map(|(i, &hz)| chime(t, (i as f32).mul_add(0.2, 0.2), hz, 3.0) * 0.22)
            .sum();
        pad + chimes
    })
}

/// Evening: chimes stepping down D-B-G over a low hum settling home.
fn dusk() -> Vec<f32> {
    const SECS: f32 = 2.4;
    synth::render(SECS, |t| {
        let hum_env = (t / 0.3).min(1.0) * (-t * 1.2).exp();
        let hum = (TAU * D4 * t).sin().mul_add(0.5, (TAU * G3 * t).sin()) * hum_env * 0.25;
        let chimes: f32 = [D6, B5, G5]
            .iter()
            .enumerate()
            .map(|(i, &hz)| chime(t, i as f32 * 0.25, hz, 2.5) * 0.28)
            .sum();
        hum + chimes
    })
}

/// The hearth, made to loop: a low breathing rumble with the odd log
/// popping over it.
fn fire() -> Vec<f32> {
    const SECS: f32 = 6.0;
    const FOLD: f32 = 1.0;
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut pops = fastrand::Rng::with_seed(SEED ^ 0xF1);
    let mut lp = (0.0_f32, 0.0_f32);
    let mut snap = 0.0_f32;
    let mut crackle = 0.0_f32;
    looped(SECS, FOLD, |t| {
        let n = noise(&mut rng);
        // Whole cycles over SECS, so the flames' breathing loops too.
        let breath = (TAU * 2.0 * t / SECS).sin().mul_add(0.2, 0.8)
            * (TAU * 5.0 * t / SECS).sin().mul_add(0.15, 0.85);
        lp.0 = (n - lp.0).mul_add(0.02, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(0.02, lp.1);
        let rumble = lp.1 * 9.0 * breath;
        // Pops, about six a second, none right at the seam.
        let at_seam = (t - SECS).abs() < 0.02;
        if pops.f32() < 0.000_14 && !at_seam {
            snap = pops.f32().powi(2).mul_add(0.7, 0.15);
        }
        snap *= 0.985;
        crackle = n.mul_add(snap, -crackle).mul_add(0.5, crackle);
        crackle.mul_add(1.2, rumble)
    })
}

/// The garden through the door, made to loop: a quiet breeze, and now and
/// then a bird, never across the seam.
fn garden() -> Vec<f32> {
    const SECS: f32 = 8.0;
    const FOLD: f32 = 1.5;
    /// When each chirp starts, its pitch sweep and length.
    const CHIRPS: [(f32, f32, f32, f32); 7] = [
        (2.0, D7, G6, 0.07),
        (2.12, D7, G6, 0.07),
        (4.3, B6, D7, 0.05),
        (4.38, B6, D7, 0.05),
        (4.46, B6, E7, 0.08),
        (6.2, E7, B6, 0.12),
        (6.45, G6, D7, 0.06),
    ];
    let mut rng = fastrand::Rng::with_seed(SEED);
    let mut lp = (0.0_f32, 0.0_f32);
    looped(SECS, FOLD, |t| {
        // Whole cycles over SECS, so the gusts loop too.
        let gust = (TAU * t / SECS).sin().mul_add(0.35, 0.65)
            * (TAU * 3.0 * t / SECS).sin().mul_add(0.15, 0.85);
        let cutoff = gust.mul_add(0.025, 0.008);
        lp.0 = (noise(&mut rng) - lp.0).mul_add(cutoff, lp.0);
        lp.1 = (lp.0 - lp.1).mul_add(cutoff, lp.1);
        let breeze = lp.1 * gust * 3.5;
        let birds: f32 = CHIRPS
            .iter()
            .map(|&(at, from, to, len)| {
                let u = t - at;
                if u < 0.0 || u > len {
                    return 0.0;
                }
                // A straight sweep from `from` to `to`; this is its phase.
                let phase = TAU * ((to - from) / (2.0 * len)).mul_add(u * u, from * u);
                phase.sin() * (PI * u / len).sin().powi(2) * 0.45
            })
            .sum();
        breeze + birds
    })
}

#[cfg(test)]
// Tests turn positive seconds into sample counts.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
mod tests {
    use super::*;
    use crate::leitmotif::mix::Mixer;

    /// Loudest any pile-up may get.
    const HEADROOM: f32 = 0.9;

    /// The sounds that loop rather than play once.
    const LOOPS: [Sfx; 2] = [Sfx::Fire, Sfx::Garden];

    /// The sounds a held mouse button repeats, several times a second.
    const STROKES: [Sfx; 8] = [
        Sfx::Chop,
        Sfx::Grind,
        Sfx::Knead,
        Sfx::Mix,
        Sfx::Churn,
        Sfx::Crank,
        Sfx::Squirt,
        Sfx::Axe,
    ];

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
        for mark in 0..3 {
            assert!(all.contains(&Sfx::Mark(mark)), "Mark({mark}) is missing");
        }
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
    fn strokes_are_quick() {
        for sfx in STROKES {
            let secs = decoded(sfx).len() as f32 / SAMPLE_RATE as f32;
            assert!(secs <= 0.25, "{sfx:?} lasts {secs} s");
        }
    }

    #[test]
    fn marks_rise() {
        // Each mark's note is higher than the last; a falling run would
        // sound like losing.
        let marks: Vec<f32> = (0..3).map(mark_note).collect();
        assert!(marks.windows(2).all(|w| w[0] < w[1]));
        assert_ne!(render(Sfx::Mark(0)), render(Sfx::Mark(2)));
    }

    #[test]
    fn a_busy_moment_does_not_clip() {
        // The worst the game does at once: the hearth and the garden going,
        // the knife at full tilt while the pan sizzles, the pot boils and a
        // log catches, a station finishing as a customer rings and pays
        // with a tip — and the player shuffling food around through all of
        // it.
        let sounds: Vec<(Sfx, Vec<f32>)> =
            Sfx::all().into_iter().map(|s| (s, decoded(s))).collect();
        let play = |mixer: &mut Mixer, at: f32, sfx: Sfx| {
            let (_, samples) = sounds.iter().find(|(s, _)| *s == sfx).unwrap();
            mixer.play(at, samples, sfx.gain());
        };
        let mut mixer = Mixer::new();
        play(&mut mixer, 0.0, Sfx::Fire);
        play(&mut mixer, 0.0, Sfx::Garden);
        for step in 0..12 {
            let at = step as f32 * 0.14;
            play(&mut mixer, at, Sfx::Chop);
            play(&mut mixer, at + 0.03, Sfx::Tick);
            if step % 3 == 0 {
                play(&mut mixer, at + 0.05, Sfx::Lift);
                play(&mut mixer, at + 0.1, Sfx::Set);
            }
        }
        play(&mut mixer, 0.2, Sfx::Sizzle);
        play(&mut mixer, 0.22, Sfx::Bubble);
        play(&mut mixer, 0.25, Sfx::Ignite);
        play(&mut mixer, 0.3, Sfx::Smoking);
        play(&mut mixer, 0.4, Sfx::Done);
        play(&mut mixer, 0.42, Sfx::Bell);
        play(&mut mixer, 0.45, Sfx::Coins);
        play(&mut mixer, 0.5, Sfx::Tip);
        play(&mut mixer, 0.55, Sfx::Cluck);
        play(&mut mixer, 0.6, Sfx::Oven);
        let loudest = mixer.peak();
        println!("kitchen garden pile-up peak: {loudest:.3}");
        assert!(loudest <= HEADROOM, "a busy moment peaks at {loudest}");
    }

    #[test]
    fn generation_is_reproducible() {
        for sfx in [Sfx::Fire, Sfx::Garden, Sfx::Sizzle, Sfx::Chop, Sfx::Ignite] {
            assert_eq!(render(sfx), render(sfx), "{sfx:?}");
        }
    }
}
