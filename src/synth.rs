//! Procedural instruments, as WAV bytes.
//!
//! Every voice is generated at startup, so the toy ships no audio assets and
//! there is nothing to credit in `CREDITS.md`. macroquad's audio API offers
//! volume and looping and nothing else — no pitch control — so the frontend
//! bakes one buffer per pitch each instrument uses (the song says which),
//! and only gain varies at runtime.
//!
//! On the web these bytes go through the browser's `decodeAudioData`, which
//! never reports failure back in a form macroquad surfaces: a malformed
//! header hangs the loader instead of erroring. The tests at the bottom
//! guard the header for that reason.

use std::f32::consts::TAU;

/// Sample rate of every buffer.
///
/// quad-snd's native mixer runs at 44.1 kHz and nearest-neighbour resamples
/// anything else on load; matching it means that never happens.
pub const SAMPLE_RATE: u32 = 44_100;

/// Fade applied to the head and tail of every buffer, in seconds. A buffer
/// that starts or stops at a nonzero sample clicks.
const FADE: f32 = 0.004;

/// Fixed seed for the noise voices, so a given build always sounds the same.
const NOISE_SEED: u64 = 0x50FA_5EED;

/// Kick: a sine dropping from a knock to a thud, with a noise click on the
/// front so it reads as a hit rather than a note.
#[must_use]
pub fn kick() -> Vec<u8> {
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    let mut phase = 0.0;
    wav(&render(0.28, |t| {
        // Sweeping a sine means integrating frequency; evaluating
        // `sin(TAU * f(t) * t)` with a moving `f` warps the phase and chirps.
        let freq = 130.0_f32.mul_add((-t * 22.0).exp(), 46.0);
        phase += TAU * freq / SAMPLE_RATE as f32;
        let body = phase.sin() * (-t * 9.0).exp();
        let click = rng.f32().mul_add(2.0, -1.0) * (-t * 160.0).exp() * 0.3;
        (body + click) * 0.9
    }))
}

/// Snare: a burst of noise over a short tone, both dying fast.
#[must_use]
pub fn snare() -> Vec<u8> {
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    let mut lp = 0.0;
    wav(&render(0.18, |t| {
        let noise = rng.f32().mul_add(2.0, -1.0);
        lp = (noise - lp).mul_add(0.5, lp);
        let rattle = lp * (-t * 24.0).exp();
        let tone = (TAU * 190.0 * t).sin() * (-t * 40.0).exp() * 0.5;
        (rattle + tone) * 0.8
    }))
}

/// Hat: a tick of bright noise. Highpassed by subtracting a lowpass, which
/// is as much filter design as a hat deserves.
#[must_use]
pub fn hat() -> Vec<u8> {
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    let mut lp = 0.0;
    wav(&render(0.06, |t| {
        let noise = rng.f32().mul_add(2.0, -1.0);
        lp = (noise - lp).mul_add(0.3, lp);
        (noise - lp) * (-t * 70.0).exp() * 0.6
    }))
}

/// Bass: a sine with a couple of harmonics that fade faster than the
/// fundamental, so the note opens bright and settles round.
#[must_use]
pub fn bass(hz: f32) -> Vec<u8> {
    wav(&render(0.4, |t| {
        let fundamental = (TAU * hz * t).sin();
        let second = (TAU * hz * 2.0 * t).sin() * (-t * 12.0).exp() * 0.35;
        let third = (TAU * hz * 3.0 * t).sin() * (-t * 20.0).exp() * 0.15;
        (fundamental + second + third) * (-t * 5.0).exp() * 0.7
    }))
}

/// Lead: Karplus-Strong plucked string. A burst of noise circulates through
/// a delay line one period long, losing a little of its top end each pass,
/// which is all a plucked string physically is.
#[must_use]
pub fn pluck(hz: f32) -> Vec<u8> {
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    // The delay line has to be at least two samples or the averaging below
    // has nothing to average; no pitch in the song comes anywhere near.
    // Frequencies are positive, so the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    let period = ((SAMPLE_RATE as f32 / hz).round() as usize).max(2);
    let mut line: Vec<f32> = (0..period).map(|_| rng.f32().mul_add(2.0, -1.0)).collect();
    let mut head = 0;
    wav(&render(0.6, |t| {
        let next = (head + 1) % period;
        // Two-point average is the lowpass; the gain below it sets the decay.
        let out = (line[head] + line[next]) * 0.5 * 0.996;
        line[head] = out;
        head = next;
        // A gentle overall envelope so the tail does not hang forever.
        out * (-t * 2.5).exp() * 0.5
    }))
}

/// Pad: a soft, slightly detuned chord that swells in and fades out.
/// Sustains a little past the longest bar so chords overlap rather than gap.
#[must_use]
pub fn pad(hz: &[f32]) -> Vec<u8> {
    const SECS: f32 = 3.0;
    const ATTACK: f32 = 0.25;
    const RELEASE: f32 = 0.8;
    let gain = 0.22 / hz.len().max(1) as f32;
    wav(&render(SECS, |t| {
        let attack = (t / ATTACK).min(1.0);
        let release = ((SECS - t) / RELEASE).min(1.0);
        let envelope = attack * release;
        hz.iter()
            .map(|&f| {
                // Two oscillators a few cents apart beat gently against each
                // other; a quiet octave up keeps it from sounding hollow.
                let a = (TAU * f * 0.998 * t).sin();
                let b = (TAU * f * 1.002 * t).sin();
                let high = (TAU * f * 2.0 * t).sin() * 0.15;
                a + b + high
            })
            .sum::<f32>()
            * envelope
            * gain
    }))
}

/// Pickup: a bright bell, an octave and a fifth of partials.
#[must_use]
pub fn chime() -> Vec<u8> {
    wav(&render(0.35, |t| {
        let a = (TAU * 1318.5 * t).sin();
        let b = (TAU * 2637.0 * t).sin() * 0.4;
        let c = (TAU * 1976.0 * t).sin() * 0.25;
        (a + b + c) * (-t * 9.0).exp() * 0.35
    }))
}

/// Hit: a low thud with a rasp of noise on top.
#[must_use]
pub fn thump() -> Vec<u8> {
    let mut rng = fastrand::Rng::with_seed(NOISE_SEED);
    let mut phase = 0.0;
    wav(&render(0.35, |t| {
        let freq = 90.0_f32.mul_add((-t * 14.0).exp(), 38.0);
        phase += TAU * freq / SAMPLE_RATE as f32;
        let body = phase.sin() * (-t * 7.0).exp();
        let rasp = rng.f32().mul_add(2.0, -1.0) * (-t * 30.0).exp() * 0.4;
        (body + rasp) * 0.9
    }))
}

/// Pause and unpause. `rising` picks the direction the pitch slides, so the
/// two states are distinguishable without looking at the screen.
#[must_use]
pub fn blip(rising: bool) -> Vec<u8> {
    const SECS: f32 = 0.08;
    let (from, to): (f32, f32) = if rising {
        (520.0, 880.0)
    } else {
        (880.0, 520.0)
    };
    let mut phase = 0.0;
    wav(&render(SECS, |t| {
        let freq = (to - from).mul_add(t / SECS, from);
        phase += TAU * freq / SAMPLE_RATE as f32;
        phase.sin() * (-t * 16.0).exp() * 0.5
    }))
}

/// Render `secs` of mono audio, then fade both ends so it starts and stops
/// silently.
fn render(secs: f32, mut voice: impl FnMut(f32) -> f32) -> Vec<f32> {
    let mut buffer: Vec<f32> = (0..sample_count(secs))
        .map(|i| voice(i as f32 / SAMPLE_RATE as f32))
        .collect();
    let fade = sample_count(FADE);
    let len = buffer.len();
    for i in 0..fade.min(len / 2) {
        let gain = i as f32 / fade as f32;
        buffer[i] *= gain;
        buffer[len - 1 - i] *= gain;
    }
    buffer
}

/// Seconds to a whole number of samples.
// Every duration fed to this is a positive literal in this file.
#[allow(clippy::cast_sign_loss)]
fn sample_count(secs: f32) -> usize {
    (secs * SAMPLE_RATE as f32) as usize
}

/// Wrap mono `f32` samples as a 16-bit PCM WAV.
///
/// Hand-rolled because the format's uncompressed case is a 44-byte header and
/// pulling a crate in for it would be sillier than writing it out.
fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut out = Vec::with_capacity(44 + data_len);

    // RIFF header: the size counts everything after this field, so the 44-byte
    // header minus the 8 bytes of "RIFF" and the size itself, plus the samples.
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");

    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16_u32.to_le_bytes()); // rest of this chunk
    out.extend_from_slice(&1_u16.to_le_bytes()); // uncompressed PCM
    out.extend_from_slice(&1_u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // bytes per second
    out.extend_from_slice(&2_u16.to_le_bytes()); // bytes per frame
    out.extend_from_slice(&16_u16.to_le_bytes()); // bits per sample

    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in samples {
        // Clamp rather than let the cast wrap: a voice that overshoots should
        // sound squashed, not inverted.
        let scaled = sample.clamp(-1.0, 1.0) * f32::from(i16::MAX);
        out.extend_from_slice(&(scaled as i16).to_le_bytes());
    }

    out
}

#[cfg(test)]
// Tests turn positive frequencies into sample counts.
#[allow(clippy::cast_sign_loss)]
mod tests {
    use super::*;
    use crate::song::{self, Instrument};

    /// Every voice the game bakes, by name.
    fn voices() -> Vec<(String, Vec<u8>)> {
        let mut out = vec![
            ("kick".to_owned(), kick()),
            ("snare".to_owned(), snare()),
            ("hat".to_owned(), hat()),
            ("chime".to_owned(), chime()),
            ("thump".to_owned(), thump()),
            ("blip up".to_owned(), blip(true)),
            ("blip down".to_owned(), blip(false)),
        ];
        for pitch in song::pitches(Instrument::Bass) {
            out.push((format!("bass {pitch}"), bass(song::hertz(pitch))));
        }
        for pitch in song::pitches(Instrument::Lead) {
            out.push((format!("pluck {pitch}"), pluck(song::hertz(pitch))));
        }
        for chord in song::chords() {
            let hz: Vec<f32> = chord.notes.iter().map(|&n| song::hertz(n)).collect();
            out.push((format!("pad {:?}", chord.notes), pad(&hz)));
        }
        out
    }

    /// Pull the samples back out of a WAV the way a decoder would.
    fn decode(bytes: &[u8]) -> Vec<i16> {
        bytes[44..]
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn header_is_a_well_formed_wav() {
        // A bad header does not fail loudly on the web -- decodeAudioData just
        // never calls back, and macroquad waits for it forever. Hence a test.
        for (name, bytes) in voices() {
            assert_eq!(&bytes[0..4], b"RIFF", "{name}");
            assert_eq!(&bytes[8..12], b"WAVE", "{name}");
            assert_eq!(&bytes[12..16], b"fmt ", "{name}");
            assert_eq!(&bytes[36..40], b"data", "{name}");

            let data_len = u32_at(&bytes, 40) as usize;
            assert_eq!(data_len, bytes.len() - 44, "{name} data size");
            assert_eq!(
                u32_at(&bytes, 4) as usize,
                bytes.len() - 8,
                "{name} riff size"
            );
            assert_eq!(data_len % 2, 0, "{name} truncated 16-bit sample");
            assert_eq!(u32_at(&bytes, 24), SAMPLE_RATE, "{name} sample rate");
        }
    }

    #[test]
    fn voices_are_audible_but_not_clipped_flat() {
        for (name, bytes) in voices() {
            let samples = decode(&bytes);
            let peak = samples.iter().map(|s| i32::from(s.abs())).max().unwrap();
            assert!(peak > 3000, "{name} is nearly silent (peak {peak})");

            // Clipping is clamped rather than wrapped, so a voice that runs
            // too hot shows up as a pile of samples pinned at the rail.
            let pinned = samples.iter().filter(|s| s.abs() > 32_700).count();
            assert!(
                pinned * 100 < samples.len(),
                "{name} clips for {pinned} of {} samples",
                samples.len()
            );
        }
    }

    #[test]
    fn voices_start_and_end_silently() {
        // Anything else is an audible click at each end.
        for (name, bytes) in voices() {
            let samples = decode(&bytes);
            assert_eq!(samples.first(), Some(&0), "{name} starts mid-waveform");
            assert!(
                samples.last().unwrap().abs() < 32,
                "{name} ends at {}",
                samples.last().unwrap()
            );
        }
    }

    #[test]
    fn pitched_voices_are_pitched() {
        // A note at f Hz repeats every SAMPLE_RATE / f samples, so its
        // autocorrelation peaks at that lag. Harmonics do not disturb this
        // the way they would a zero-crossing count.
        for (name, bytes, hz) in [
            ("bass", bass(110.0), 110.0_f32),
            ("pluck", pluck(440.0), 440.0),
        ] {
            let samples: Vec<f32> = decode(&bytes).iter().map(|&s| f32::from(s)).collect();
            let start = SAMPLE_RATE as usize / 20;
            let window = &samples[start..start + SAMPLE_RATE as usize / 10];
            let period = (SAMPLE_RATE as f32 / hz).round() as usize;
            let at_period = autocorrelation(window, period);
            let off_period = autocorrelation(window, period * 3 / 5);
            assert!(
                at_period > 0.8,
                "{name} at {hz} Hz does not repeat at its period ({at_period})"
            );
            assert!(
                at_period > off_period,
                "{name} at {hz} Hz correlates better off-period ({off_period} vs {at_period})"
            );
        }
    }

    /// Normalised autocorrelation of `x` at `lag`, in `-1..=1`.
    fn autocorrelation(x: &[f32], lag: usize) -> f32 {
        let n = x.len() - lag;
        let dot: f32 = (0..n).map(|i| x[i] * x[i + lag]).sum();
        let energy: f32 = (0..n).map(|i| x[i] * x[i]).sum();
        dot / energy.max(f32::EPSILON)
    }

    #[test]
    fn generation_is_reproducible() {
        // The noise voices seed a fixed RNG; two builds must agree.
        assert_eq!(kick(), kick());
        assert_eq!(snare(), snare());
        assert_eq!(pluck(330.0), pluck(330.0));
    }
}
