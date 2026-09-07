//! The score, and the sequencer that walks through it.
//!
//! This is the part of the sim that decides *which notes sound when*. It
//! knows nothing about the arena: the sim turns its [`Event`]s into
//! behaviour, and the frontend turns the very same events into sound. That
//! shared feed is the whole idea of the prototype — if you can hear an
//! instrument, it is acting on the world, and if you cannot, it is not.
//!
//! The song is a loop of [`Section`]s. Each section is one four-bar phrase
//! of a [`Motif`] (a theme, with its own key, tempo and character) played by
//! some subset of the four [`Instrument`]s (the arrangement). On top of the
//! arrangement, the sim may hush any instrument; both decide whether a
//! layer is *active*, and only active layers emit events.
//!
//! Everything here is deterministic: the sequencer advances by sim ticks
//! and holds no clocks of its own.

/// Sixteenth-note grid.
pub const STEPS_PER_BEAT: u32 = 4;

/// Four-four time throughout.
pub const BEATS_PER_BAR: u32 = 4;

/// Steps in one bar.
pub const STEPS_PER_BAR: u32 = STEPS_PER_BEAT * BEATS_PER_BAR;

/// Every section is one phrase this long.
pub const BARS_PER_SECTION: u32 = 4;

/// Steps in one section.
pub const STEPS_PER_SECTION: u32 = STEPS_PER_BAR * BARS_PER_SECTION;

/// General MIDI drum notes, used as the "pitch" of drum events so every
/// note event has the same shape.
pub const KICK: u8 = 36;
/// See [`KICK`].
pub const SNARE: u8 = 38;
/// See [`KICK`].
pub const HAT: u8 = 42;

/// A layer of the arrangement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instrument {
    /// Kick, snare and hat. Moves the stompers.
    Drums,
    /// Raises the walls.
    Bass,
    /// Spawns the sparks.
    Lead,
    /// Draws sparks toward the player.
    Pad,
}

impl Instrument {
    /// Every instrument, in [`Instrument::index`] order.
    pub const ALL: [Self; 4] = [Self::Drums, Self::Bass, Self::Lead, Self::Pad];

    /// Position in [`Instrument::ALL`], for indexing per-layer arrays.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Drums => "drums",
            Self::Bass => "bass",
            Self::Lead => "lead",
            Self::Pad => "pad",
        }
    }
}

/// A theme. Each has its own key, tempo and — over in the sim — its own
/// rules for how the arena behaves while it plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Motif {
    /// C major, easy-going. Stompers wander.
    Wander,
    /// A minor, fast. Stompers hunt the player.
    Pursuit,
    /// F major, slow. Stompers sleep.
    Lullaby,
}

impl Motif {
    /// Display name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wander => "wander",
            Self::Pursuit => "pursuit",
            Self::Lullaby => "lullaby",
        }
    }

    /// Tempo, in beats per minute.
    #[must_use]
    pub const fn bpm(self) -> f32 {
        match self {
            Self::Wander => 110.0,
            Self::Pursuit => 132.0,
            Self::Lullaby => 84.0,
        }
    }

    /// Seconds per beat.
    #[must_use]
    pub const fn beat_secs(self) -> f32 {
        60.0 / self.bpm()
    }

    /// Seconds per sixteenth step.
    #[must_use]
    pub const fn step_secs(self) -> f32 {
        self.beat_secs() / STEPS_PER_BEAT as f32
    }

    /// Seconds per bar.
    #[must_use]
    pub const fn bar_secs(self) -> f32 {
        self.beat_secs() * BEATS_PER_BAR as f32
    }

    /// The chord the motif comes home to: the first bar's pad chord.
    #[must_use]
    pub const fn tonic(self) -> Chord {
        self.score().pad[0]
    }

    /// Lowest and highest lead pitch this motif uses, for mapping pitch
    /// onto space.
    #[must_use]
    pub fn lead_range(self) -> (u8, u8) {
        let mut lo = u8::MAX;
        let mut hi = u8::MIN;
        for &(_, pitch) in self.score().lead {
            lo = lo.min(pitch);
            hi = hi.max(pitch);
        }
        (lo, hi)
    }

    const fn score(self) -> &'static Score {
        match self {
            Self::Wander => &WANDER,
            Self::Pursuit => &PURSUIT,
            Self::Lullaby => &LULLABY,
        }
    }
}

/// Three-note voicing for the pad. Notes are MIDI numbers, lowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub notes: [u8; 3],
}

impl Chord {
    const fn new(notes: [u8; 3]) -> Self {
        Self { notes }
    }

    /// Lowest note, which is also the root in every voicing used here.
    #[must_use]
    pub const fn root(self) -> u8 {
        self.notes[0]
    }
}

/// One motif's notes. Drum patterns are one bar of sixteen characters, `x`
/// for a hit, repeated every bar; bass and lead are `(step, pitch)` pairs
/// over the whole four-bar phrase; the pad is one chord per bar.
struct Score {
    kick: &'static str,
    snare: &'static str,
    hat: &'static str,
    bass: &'static [(u8, u8)],
    lead: &'static [(u8, u8)],
    pad: [Chord; BARS_PER_SECTION as usize],
}

// MIDI note numbers, so the scores below read as music rather than as ints.
const C2: u8 = 36;
const D2: u8 = 38;
const E2: u8 = 40;
const F2: u8 = 41;
const G2: u8 = 43;
const GS2: u8 = 44;
const A2: u8 = 45;
const BB2: u8 = 46;
const E3: u8 = 52;
const F3: u8 = 53;
const G3: u8 = 55;
const GS3: u8 = 56;
const A3: u8 = 57;
const BB3: u8 = 58;
const B3: u8 = 59;
const C4: u8 = 60;
const D4: u8 = 62;
const E4: u8 = 64;
const F4: u8 = 65;
const G4: u8 = 67;
const GS4: u8 = 68;
const A4: u8 = 69;
const BB4: u8 = 70;
const B4: u8 = 71;
const C5: u8 = 72;
const D5: u8 = 74;
const E5: u8 = 76;
const F5: u8 = 77;

/// C major, 110 BPM: C, Am, F, G. A pentatonic tune over a loping beat.
static WANDER: Score = Score {
    kick: "x.....x...x.....",
    snare: "....x.......x...",
    hat: "x.x.x.x.x.x.x.x.",
    bass: &[
        (0, C2),
        (6, C2),
        (8, G2),
        (12, C2),
        (16, A2),
        (22, A2),
        (24, E2),
        (28, A2),
        (32, F2),
        (38, F2),
        (40, C2),
        (44, F2),
        (48, G2),
        (54, G2),
        (56, D2),
        (60, G2),
    ],
    lead: &[
        (0, E4),
        (2, G4),
        (4, A4),
        (8, G4),
        (12, E4),
        (14, D4),
        (16, C4),
        (20, E4),
        (24, A4),
        (26, G4),
        (28, E4),
        (32, F4),
        (36, A4),
        (38, C5),
        (40, A4),
        (44, G4),
        (46, F4),
        (48, D4),
        (52, G4),
        (56, B4),
        (60, D5),
        (62, C5),
    ],
    pad: [
        Chord::new([C4, E4, G4]),
        Chord::new([A3, C4, E4]),
        Chord::new([A3, C4, F4]),
        Chord::new([B3, D4, G4]),
    ],
};

/// A minor, 132 BPM: Am, F, Dm, E. Driving eighths and an arpeggio.
static PURSUIT: Score = Score {
    kick: "x...x..x..x.x...",
    snare: "....x.......x..x",
    hat: "x.x.x.x.x.x.x.x.",
    bass: &[
        (0, A2),
        (2, A2),
        (4, A2),
        (6, A2),
        (8, A2),
        (10, A2),
        (12, G2),
        (14, G2),
        (16, F2),
        (18, F2),
        (20, F2),
        (22, F2),
        (24, F2),
        (26, F2),
        (28, E2),
        (30, E2),
        (32, D2),
        (34, D2),
        (36, D2),
        (38, D2),
        (40, D2),
        (42, D2),
        (44, F2),
        (46, F2),
        (48, E2),
        (50, E2),
        (52, E2),
        (54, E2),
        (56, E2),
        (58, E2),
        (60, GS2),
        (62, GS2),
    ],
    lead: &[
        (0, A4),
        (2, C5),
        (4, E5),
        (6, C5),
        (8, A4),
        (10, C5),
        (12, E5),
        (14, C5),
        (16, A4),
        (18, C5),
        (20, F5),
        (22, C5),
        (24, A4),
        (26, C5),
        (28, F5),
        (30, C5),
        (32, A4),
        (34, D5),
        (36, F5),
        (38, D5),
        (40, A4),
        (42, D5),
        (44, F5),
        (46, D5),
        (48, GS4),
        (50, B4),
        (52, E5),
        (54, B4),
        (56, GS4),
        (58, B4),
        (60, E5),
        (62, B4),
    ],
    pad: [
        Chord::new([A3, C4, E4]),
        Chord::new([A3, C4, F4]),
        Chord::new([A3, D4, F4]),
        Chord::new([GS3, B3, E4]),
    ],
};

/// F major, 84 BPM: F, Bb, F, C. Whole-note bass, brushed hats, a sparse
/// tune. Note that the drums layer is *arranged* here — it just holds no
/// kicks, so the stompers have nothing to step to.
static LULLABY: Score = Score {
    kick: "................",
    snare: "................",
    hat: "x.......x.......",
    bass: &[(0, F2), (16, BB2), (32, F2), (48, C2)],
    lead: &[
        (0, A4),
        (8, C5),
        (16, BB4),
        (24, A4),
        (32, F4),
        (40, G4),
        (48, E4),
        (56, G4),
    ],
    pad: [
        Chord::new([F3, A3, C4]),
        Chord::new([F3, BB3, D4]),
        Chord::new([F3, A3, C4]),
        Chord::new([E3, G3, C4]),
    ],
};

/// One phrase of the song: a motif, and which instruments the arrangement
/// gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section {
    pub motif: Motif,
    /// Indexed by [`Instrument::index`].
    pub arranged: [bool; 4],
}

impl Section {
    const fn new(motif: Motif, arranged: [bool; 4]) -> Self {
        Self { motif, arranged }
    }
}

/// The song form. Loops back to the top when it runs out.
pub const SONG: [Section; 6] = [
    // Intro: bass and tune only, so the arena starts calm.
    Section::new(Motif::Wander, [false, true, true, false]),
    Section::new(Motif::Wander, [true, true, true, true]),
    Section::new(Motif::Pursuit, [true, true, true, true]),
    // The pad drops out: no magnet for the second chase.
    Section::new(Motif::Pursuit, [true, true, true, false]),
    Section::new(Motif::Lullaby, [true, true, true, true]),
    Section::new(Motif::Wander, [true, true, true, true]),
];

/// Something the music did. The sim and the frontend both consume these.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// A note sounded. `step` is the position within the bar, `0..16`.
    Note {
        instrument: Instrument,
        pitch: u8,
        /// `0..=1`, louder on the beat.
        velocity: f32,
        step: u32,
    },
    /// The pad changed chord, at the top of a bar.
    Chord(Chord),
    /// A new section began.
    Section { motif: Motif },
    /// An instrument was hushed or let sound again.
    Layer { instrument: Instrument, on: bool },
}

/// One layer's state, for the HUD.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layer {
    /// The arrangement includes this instrument in the current section.
    pub arranged: bool,
    /// The player is hushing it.
    pub muted: bool,
}

impl Layer {
    /// Whether the layer is sounding, and so acting on the world.
    #[must_use]
    pub const fn active(self) -> bool {
        self.arranged && !self.muted
    }
}

/// Where the sequencer is, for the HUD and the renderer's pulse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Index into [`SONG`].
    pub section: usize,
    pub motif: Motif,
    /// Bar within the section, `0..4`.
    pub bar: u32,
    /// Beat within the bar, `0..4`.
    pub beat: u32,
    /// Step within the bar, `0..16`.
    pub step: u32,
    /// How far through the current step, `0..1`.
    pub step_phase: f32,
    /// Indexed by [`Instrument::index`].
    pub layers: [Layer; 4],
}

impl Position {
    /// How far through the current beat, `0..1`. Handy for pulsing things.
    #[must_use]
    pub fn beat_phase(&self) -> f32 {
        ((self.step % STEPS_PER_BEAT) as f32 + self.step_phase) / STEPS_PER_BEAT as f32
    }
}

/// Walks the song. Advanced by the sim, one tick at a time.
#[derive(Clone, Debug)]
pub struct Sequencer {
    section: usize,
    /// Step within the section, `0..STEPS_PER_SECTION`.
    step: u32,
    /// Seconds into the current step.
    clock: f32,
    started: bool,
    muted: [bool; 4],
    /// Jump to the next section at the next bar.
    skip: bool,
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl Sequencer {
    /// Start at the top of the song. Nothing sounds until the first
    /// [`Sequencer::advance`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            section: 0,
            step: 0,
            clock: 0.0,
            started: false,
            muted: [false; 4],
            skip: false,
        }
    }

    /// Current section.
    #[must_use]
    pub const fn section(&self) -> Section {
        SONG[self.section]
    }

    /// Current motif.
    #[must_use]
    pub const fn motif(&self) -> Motif {
        self.section().motif
    }

    /// Whether an instrument is sounding right now.
    #[must_use]
    pub const fn is_active(&self, instrument: Instrument) -> bool {
        let i = instrument.index();
        self.section().arranged[i] && !self.muted[i]
    }

    /// How many instruments are sounding right now.
    #[must_use]
    pub fn active_count(&self) -> u32 {
        Instrument::ALL
            .iter()
            .filter(|&&instrument| self.is_active(instrument))
            .count() as u32
    }

    /// Hush or unhush an instrument, effective immediately. Notes already
    /// sounding are the frontend's to finish; new ones stop at once, the
    /// way a kill switch works. Appends a [`Event::Layer`] if anything
    /// changed.
    pub fn set_muted(&mut self, instrument: Instrument, muted: bool, events: &mut Vec<Event>) {
        let i = instrument.index();
        if self.muted[i] == muted {
            return;
        }
        self.muted[i] = muted;
        events.push(Event::Layer {
            instrument,
            on: !muted,
        });
    }

    /// Whether the player is hushing an instrument.
    #[must_use]
    pub const fn is_muted(&self, instrument: Instrument) -> bool {
        self.muted[instrument.index()]
    }

    /// Queue a jump to the next section at the next bar.
    pub const fn request_skip(&mut self) {
        self.skip = true;
    }

    /// Snapshot for the HUD.
    #[must_use]
    pub fn position(&self) -> Position {
        let section = self.section();
        let mut layers = [Layer::default(); 4];
        for instrument in Instrument::ALL {
            let i = instrument.index();
            layers[i] = Layer {
                arranged: section.arranged[i],
                muted: self.muted[i],
            };
        }
        Position {
            section: self.section,
            motif: section.motif,
            bar: self.step / STEPS_PER_BAR,
            beat: (self.step % STEPS_PER_BAR) / STEPS_PER_BEAT,
            step: self.step % STEPS_PER_BAR,
            step_phase: (self.clock / section.motif.step_secs()).clamp(0.0, 1.0),
            layers,
        }
    }

    /// Move `dt` seconds along the song, appending whatever sounded.
    pub fn advance(&mut self, dt: f32, events: &mut Vec<Event>) {
        if !self.started {
            self.started = true;
            events.push(Event::Section {
                motif: self.motif(),
            });
            self.emit(events);
        }
        self.clock += dt;
        // The float condition is the fixed-timestep idiom: the loop runs one
        // step per `step_secs` of banked time.
        #[allow(clippy::while_float)]
        while self.clock >= self.motif().step_secs() {
            self.clock -= self.motif().step_secs();
            self.step_forward(events);
            self.emit(events);
        }
    }

    /// Advance one step, handling the bar and section boundaries.
    fn step_forward(&mut self, events: &mut Vec<Event>) {
        self.step += 1;
        if self.step % STEPS_PER_BAR != 0 {
            return;
        }
        if self.skip || self.step >= STEPS_PER_SECTION {
            self.skip = false;
            self.step = 0;
            self.section = (self.section + 1) % SONG.len();
            events.push(Event::Section {
                motif: self.motif(),
            });
        }
    }

    /// Everything that sounds on the current step.
    fn emit(&self, events: &mut Vec<Event>) {
        let score = self.motif().score();
        let step = self.step % STEPS_PER_BAR;
        let velocity = accent(step);

        if self.is_active(Instrument::Drums) {
            for (pattern, pitch, gain) in [
                (score.kick, KICK, 1.0),
                (score.snare, SNARE, 0.9),
                (score.hat, HAT, 0.5),
            ] {
                if pattern.as_bytes()[step as usize] == b'x' {
                    events.push(Event::Note {
                        instrument: Instrument::Drums,
                        pitch,
                        velocity: velocity * gain,
                        step,
                    });
                }
            }
        }
        for (instrument, notes) in [
            (Instrument::Bass, score.bass),
            (Instrument::Lead, score.lead),
        ] {
            if !self.is_active(instrument) {
                continue;
            }
            for &(at, pitch) in notes {
                if u32::from(at) == self.step {
                    events.push(Event::Note {
                        instrument,
                        pitch,
                        velocity,
                        step,
                    });
                }
            }
        }
        if self.is_active(Instrument::Pad) && step == 0 {
            let bar = (self.step / STEPS_PER_BAR) as usize;
            events.push(Event::Chord(score.pad[bar]));
        }
    }
}

/// Louder on the beat, softer off it.
const fn accent(step: u32) -> f32 {
    if step % STEPS_PER_BEAT == 0 {
        1.0
    } else if step % 2 == 0 {
        0.8
    } else {
        0.65
    }
}

/// Every pitch an instrument plays anywhere in the song, sorted and
/// deduplicated. The frontend bakes one buffer per entry.
#[must_use]
pub fn pitches(instrument: Instrument) -> Vec<u8> {
    let mut out: Vec<u8> = match instrument {
        Instrument::Drums => vec![KICK, SNARE, HAT],
        Instrument::Bass => motifs()
            .flat_map(|m| m.score().bass.iter().map(|&(_, p)| p))
            .collect(),
        Instrument::Lead => motifs()
            .flat_map(|m| m.score().lead.iter().map(|&(_, p)| p))
            .collect(),
        Instrument::Pad => chords().iter().flat_map(|c| c.notes).collect(),
    };
    out.sort_unstable();
    out.dedup();
    out
}

/// Every chord the pad plays anywhere in the song, deduplicated.
#[must_use]
pub fn chords() -> Vec<Chord> {
    let mut out: Vec<Chord> = motifs().flat_map(|m| m.score().pad).collect();
    out.sort_unstable_by_key(|c| c.notes);
    out.dedup();
    out
}

fn motifs() -> impl Iterator<Item = Motif> {
    [Motif::Wander, Motif::Pursuit, Motif::Lullaby].into_iter()
}

/// Frequency of a MIDI note, in hertz.
#[must_use]
pub fn hertz(pitch: u8) -> f32 {
    440.0 * ((f32::from(pitch) - 69.0) / 12.0).exp2()
}

#[cfg(test)]
// Tests turn positive second counts into tick counts.
#[allow(clippy::cast_sign_loss)]
mod tests {
    use super::*;

    /// Run the sequencer at 60 Hz for `secs`, collecting everything.
    fn run(seq: &mut Sequencer, secs: f32) -> Vec<Event> {
        let mut events = Vec::new();
        let ticks = (secs * 60.0).round() as u32;
        for _ in 0..ticks {
            seq.advance(1.0 / 60.0, &mut events);
        }
        events
    }

    fn notes_for(events: &[Event], wanted: Instrument) -> Vec<(u8, u32)> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Note {
                    instrument,
                    pitch,
                    step,
                    ..
                } if *instrument == wanted => Some((*pitch, *step)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn scores_are_well_formed() {
        for motif in motifs() {
            let score = motif.score();
            for (name, pattern) in [
                ("kick", score.kick),
                ("snare", score.snare),
                ("hat", score.hat),
            ] {
                assert_eq!(
                    pattern.len(),
                    STEPS_PER_BAR as usize,
                    "{motif:?} {name} pattern is not one bar"
                );
                assert!(
                    pattern.bytes().all(|b| b == b'x' || b == b'.'),
                    "{motif:?} {name} has a stray character"
                );
            }
            for (name, notes) in [("bass", score.bass), ("lead", score.lead)] {
                assert!(!notes.is_empty(), "{motif:?} has no {name}");
                for &(step, _) in notes {
                    assert!(
                        u32::from(step) < STEPS_PER_SECTION,
                        "{motif:?} {name} note at step {step} is past the phrase"
                    );
                }
                let steps: Vec<u8> = notes.iter().map(|&(s, _)| s).collect();
                let mut sorted = steps.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(steps, sorted, "{motif:?} {name} steps are not ascending");
            }
            for chord in score.pad {
                assert!(
                    chord.notes[0] < chord.notes[1] && chord.notes[1] < chord.notes[2],
                    "{motif:?} chord {chord:?} is not voiced lowest-first"
                );
            }
        }
    }

    #[test]
    fn first_advance_announces_the_section_and_the_downbeat() {
        let mut seq = Sequencer::new();
        let mut events = Vec::new();
        seq.advance(0.0, &mut events);
        assert_eq!(
            events.first(),
            Some(&Event::Section {
                motif: SONG[0].motif
            })
        );
        // Section 0 arranges bass and lead, both of which start on step 0.
        assert!(!notes_for(&events, Instrument::Bass).is_empty());
        assert!(!notes_for(&events, Instrument::Lead).is_empty());
        // ...and not drums, which the intro leaves out.
        assert!(notes_for(&events, Instrument::Drums).is_empty());
    }

    #[test]
    fn sections_last_exactly_one_phrase() {
        let mut seq = Sequencer::new();
        let phrase = SONG[0].motif.bar_secs() * BARS_PER_SECTION as f32;
        // Just short of the boundary: still in section 0.
        let events = run(&mut seq, phrase - 0.05);
        let sections = events
            .iter()
            .filter(|e| matches!(e, Event::Section { .. }))
            .count();
        assert_eq!(sections, 1, "left section 0 early");
        assert_eq!(seq.position().section, 0);

        // Over it: section 1.
        let events = run(&mut seq, 0.1);
        assert!(events.contains(&Event::Section {
            motif: SONG[1].motif
        }));
        assert_eq!(seq.position().section, 1);
        assert_eq!(seq.position().bar, 0);
    }

    #[test]
    fn song_loops() {
        let mut seq = Sequencer::new();
        let total: f32 = SONG
            .iter()
            .map(|s| s.motif.bar_secs() * BARS_PER_SECTION as f32)
            .sum();
        run(&mut seq, total + 0.1);
        assert_eq!(seq.position().section, 0, "did not wrap to the top");
    }

    #[test]
    fn hushing_is_immediate_and_announced() {
        let mut seq = Sequencer::new();
        run(&mut seq, 0.1);
        let mut events = Vec::new();
        seq.set_muted(Instrument::Lead, true, &mut events);
        assert_eq!(
            events,
            [Event::Layer {
                instrument: Instrument::Lead,
                on: false
            }]
        );
        assert!(!seq.is_active(Instrument::Lead));
        assert!(seq.position().layers[Instrument::Lead.index()].muted);

        // Nothing from the lead while it is hushed; the bass carries on.
        let events = run(&mut seq, SONG[0].motif.bar_secs());
        assert!(notes_for(&events, Instrument::Lead).is_empty());
        assert!(!notes_for(&events, Instrument::Bass).is_empty());

        // Letting go announces itself once, and repeating it says nothing.
        let mut events = Vec::new();
        seq.set_muted(Instrument::Lead, false, &mut events);
        seq.set_muted(Instrument::Lead, false, &mut events);
        assert_eq!(events.len(), 1);
        assert!(seq.is_active(Instrument::Lead));
    }

    #[test]
    fn hushing_does_not_override_the_arrangement() {
        let mut seq = Sequencer::new();
        run(&mut seq, 0.1);
        // Section 0 has no drums; unhushing cannot conjure them.
        assert!(!seq.is_active(Instrument::Drums));
        seq.set_muted(Instrument::Drums, false, &mut Vec::new());
        let events = run(&mut seq, SONG[0].motif.bar_secs());
        assert!(notes_for(&events, Instrument::Drums).is_empty());
        // And a hush held into the section that has them keeps them quiet.
        seq.set_muted(Instrument::Drums, true, &mut Vec::new());
        let phrase = SONG[0].motif.bar_secs() * BARS_PER_SECTION as f32;
        let events = run(&mut seq, phrase);
        assert_eq!(seq.position().section, 1);
        assert!(notes_for(&events, Instrument::Drums).is_empty());
    }

    #[test]
    fn skip_jumps_at_the_next_bar() {
        let mut seq = Sequencer::new();
        run(&mut seq, 0.1);
        seq.request_skip();
        let events = run(&mut seq, SONG[0].motif.bar_secs());
        assert!(events.contains(&Event::Section {
            motif: SONG[1].motif
        }));
        assert_eq!(seq.position().section, 1);
        assert_eq!(seq.position().step, 0);
    }

    #[test]
    fn pad_plays_one_chord_per_bar() {
        let mut seq = Sequencer::new();
        // Skip the intro, which has no pad.
        let phrase = SONG[0].motif.bar_secs() * BARS_PER_SECTION as f32;
        run(&mut seq, phrase + 0.01);
        assert_eq!(seq.position().section, 1);
        let events = run(&mut seq, SONG[1].motif.bar_secs() * BARS_PER_SECTION as f32);
        let chords = events
            .iter()
            .filter(|e| matches!(e, Event::Chord(_)))
            .count();
        // Four bars: bars 1, 2 and 3 of this section plus bar 0 of the next.
        assert_eq!(chords, BARS_PER_SECTION as usize);
    }

    #[test]
    fn every_pitch_played_is_in_the_bake_list() {
        let mut seq = Sequencer::new();
        let total: f32 = SONG
            .iter()
            .map(|s| s.motif.bar_secs() * BARS_PER_SECTION as f32)
            .sum();
        let events = run(&mut seq, total + 0.1);
        for instrument in Instrument::ALL {
            let baked = pitches(instrument);
            for (pitch, _) in notes_for(&events, instrument) {
                assert!(
                    baked.contains(&pitch),
                    "{instrument:?} plays {pitch}, which is not baked"
                );
            }
        }
        for event in &events {
            if let Event::Chord(chord) = event {
                assert!(chords().contains(chord), "{chord:?} is not baked");
            }
        }
        // And every arranged instrument sounded at least once.
        for instrument in Instrument::ALL {
            assert!(
                !notes_for(&events, instrument).is_empty() || instrument == Instrument::Pad,
                "{instrument:?} never played"
            );
        }
    }

    #[test]
    fn velocity_is_in_range_and_accents_the_beat() {
        let mut seq = Sequencer::new();
        let events = run(&mut seq, 5.0);
        for event in &events {
            if let Event::Note { velocity, step, .. } = event {
                assert!((0.0..=1.0).contains(velocity));
                if *step % STEPS_PER_BEAT != 0 {
                    assert!(*velocity < 1.0, "off-beat note at full velocity");
                }
            }
        }
    }

    #[test]
    fn position_phases_stay_in_range() {
        let mut seq = Sequencer::new();
        for _ in 0..600 {
            seq.advance(0.013, &mut Vec::new());
            let pos = seq.position();
            assert!((0.0..=1.0).contains(&pos.step_phase));
            assert!((0.0..=1.0).contains(&pos.beat_phase()));
            assert!(pos.bar < BARS_PER_SECTION);
            assert!(pos.beat < BEATS_PER_BAR);
            assert!(pos.step < STEPS_PER_BAR);
        }
    }

    #[test]
    fn hertz_matches_concert_pitch() {
        assert!((hertz(69) - 440.0).abs() < 1e-3);
        assert!((hertz(57) - 220.0).abs() < 1e-3);
        assert!((hertz(60) - 261.63).abs() < 0.01);
    }
}
