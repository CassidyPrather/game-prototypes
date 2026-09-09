//! The world: pure, deterministic, and macroquad-free.
//!
//! A journey home, ruled by the music. Home lies far to the east. The
//! [`song::Sequencer`] walks the score and hands back [`song::Event`]s; this
//! module turns them into behaviour and, unchanged, into [`Cue`]s for the
//! frontend to play. The two consumers see the same feed, which is what
//! makes the premise honest: what you hear is what is happening.
//!
//! What each part of the music does:
//!
//! - The **motif** (which theme is playing) sets the rules. *Wander*:
//!   stompers drift on the beat, turn on every snare, and on alternate bars
//!   swell for a beat and stomp a shockwave. *Pursuit*: stompers lunge at
//!   the player on every kick, and on alternate bars wind up for a beat and
//!   dash along the line they showed you. *Lullaby*: stompers
//!   sleep where they stand, still solid, walls stay up for most of a bar,
//!   the player slows, and a lost heart comes back.
//! - Each **drum** drives a geometry. Kicks step the stompers. Snares slam
//!   the gates shut across every standing wall's gaps for a beat, and being
//!   in a gap when they slam is a hit. Hats turn the spinners that stand
//!   between the walls, and touching one is a hit.
//! - **Bass** notes raise every wall in the lane of the note's pitch class.
//!   Walls run north-south across the way home, so the bass line decides
//!   when the road is open, and a wall rising into you is a hit.
//! - **Lead** notes drop strikes ahead of the player, placed by pitch and
//!   by position in the bar, so the melody rains down in its own shape. A
//!   strike closes over a beat, then bursts.
//! - The **pad** charges the fermata: hold the key and the music holds,
//!   and with it everything the music drives, for as long as the pool
//!   lasts. The pool refills only while the pad is sounding.
//!
//! The world is unbounded. Walls repeat forever on a grid, stompers respawn
//! around the player as it travels, and the camera is the frontend's
//! problem. Time is fixed-step with an accumulator, as in the template: the
//! frontend's only way in is an [`InputFrame`], and its only way out is the
//! getters plus [`Sim::alpha`] for interpolation.

use std::f32::consts::TAU;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};

use crate::leitmotif::song::{self, Chord, Event, Instrument, Motif, Position, Sequencer};

/// Length of one simulation step. Ticks are always exactly this long.
pub const TICK_DT: f32 = 1.0 / 60.0;

/// Longest frame the accumulator will bank, so a backgrounded tab does not
/// come back and try to catch up all at once.
const MAX_FRAME_DT: f32 = 0.25;

/// How far east home is from the start.
pub const HOME_DISTANCE: f32 = 12000.0;

/// Where home is.
pub const HOME: Vec2 = Vec2::new(HOME_DISTANCE, 0.0);

/// Being this close to home is being home.
pub const HOME_RADIUS: f32 = 70.0;

/// World-space radius of the player.
pub const PLAYER_RADIUS: f32 = 11.0;

/// World-space radius of a stomper.
pub const STOMPER_RADIUS: f32 = 15.0;

/// How many stompers travel with the player.
pub const STOMPER_COUNT: usize = 7;

/// Hearts the player starts with, and the most it can hold.
pub const MAX_HEARTS: u32 = 3;

/// Walls stand every this far along the way home.
pub const WALL_SPACING: f32 = 200.0;

/// Half the thickness of a wall.
pub const WALL_HALF_W: f32 = 14.0;

/// Walls repeat north-south with this period, broken by a gap each time, so
/// there is always a way through if you look for it. Every other wall's
/// gaps sit half a period off, so the way through zigzags.
pub const WALL_PERIOD_Y: f32 = 480.0;

/// Length of the gap in each wall period.
pub const WALL_GAP: f32 = 64.0;

/// How many lanes walls cycle through. Bass pitch classes map onto lanes.
pub const WALL_LANES: usize = 4;

/// Radius a strike bursts over.
pub const STRIKE_RADIUS: f32 = 44.0;

/// Half the length of a spinner's bar.
pub const SPINNER_HALF_LEN: f32 = 60.0;

/// Half the thickness of a spinner's bar.
pub const SPINNER_HALF_W: f32 = 7.0;

/// Reach of a stomper's shockwave.
pub const SHOCK_RADIUS: f32 = 110.0;

/// Seconds a shockwave takes to reach its full radius.
pub const SHOCK_SECS: f32 = 0.35;

/// How much of the fermata pool one second of holding spends.
pub const FERMATA_DRAIN: f32 = 0.5;

/// How much of the pool refills per second while the pad sounds.
pub const FERMATA_RECHARGE: f32 = 0.2;

/// Once the pool runs dry it must refill to this before it can be spent
/// again, so a held key pulses rather than flickers.
pub const FERMATA_RELOCK: f32 = 0.5;

/// Strikes land this far ahead of the player, spread by their step.
const STRIKE_AHEAD_MIN: f32 = 160.0;
const STRIKE_AHEAD_SPAN: f32 = 420.0;

/// Strikes spread this far sideways, by their pitch.
const STRIKE_LATERAL: f32 = 440.0;

/// Beats between a strike landing and bursting.
const STRIKE_FUSE_BEATS: f32 = 1.0;

/// Seconds a burst stays dangerous.
const STRIKE_BURST_SECS: f32 = 0.18;

/// Beats a gate stays shut after a snare.
const GATE_SHUT_BEATS: f32 = 1.0;

/// How far each hat turns the spinners.
const SPIN_STEP: f32 = TAU / 16.0;

/// How fast a spinner eases toward its next notch, per second.
const SPIN_EASE: f32 = 14.0;

/// Width of the band a shockwave hurts in, either side of its edge.
const SHOCK_BAND: f32 = 22.0;

/// Stompers this close wind up to charge on a pursuit bar.
const CHARGE_RANGE: f32 = 520.0;

/// Stompers further than this have not noticed the player: no lunging.
const SIGHT: f32 = 450.0;

/// Only this many stompers hunt at once — the nearest ones. The rest hang
/// back and drift, so a crowd is a threat to be read, not a wall of teeth.
const HUNTERS: usize = 3;

/// Beats a charge is telegraphed for.
const CHARGE_WINDUP_BEATS: f32 = 1.0;

/// Beats a dash lasts, and how fast it goes.
const DASH_BEATS: f32 = 0.75;
const DASH_SPEED: f32 = 850.0;

/// Stompers this close wind up to stomp on a wander bar.
const STOMP_RANGE: f32 = 300.0;

/// Beats a stomp is telegraphed for.
const STOMP_WINDUP_BEATS: f32 = 1.0;

/// Stompers and strikes further than this from the player are recycled.
const FAR: f32 = 1100.0;

/// Stompers respawn this far from the player.
const RESPAWN_MIN: f32 = 480.0;
const RESPAWN_SPAN: f32 = 220.0;

/// Stompers respawn ahead of the player, on the road home, this often.
const AMBUSH_CHANCE: f32 = 0.7;

/// How far off due east an ambush may spawn, in radians.
const AMBUSH_SPREAD: f32 = 1.0;

/// Chance a wandering stomper's snare turn faces the player.
const WANDER_NOTICE: f32 = 0.3;

/// Player top speed, before the motif's scaling.
const PLAYER_SPEED: f32 = 250.0;

/// Fraction of the gap to the target velocity closed per tick.
const PLAYER_ACCEL: f32 = 0.25;

/// Seconds of invulnerability after a hit.
const INVULN_SECS: f32 = 1.0;

/// Knockback speed on a hit.
const KNOCKBACK: f32 = 420.0;

/// Fraction of velocity a stomper keeps per tick.
const STOMPER_DAMPING: f32 = 0.93;

/// A wall blocks once its solidity is over this.
const WALL_SOLID: f32 = 0.35;

/// Seconds a stomper's pulse takes to fade.
const PULSE_FADE: f32 = 0.3;

/// How hard overlapping stompers shove each other apart, so a pack chasing
/// one target spreads into a front instead of stacking into one blob.
const SEPARATION: f32 = 1800.0;

/// A 2D vector, kept deliberately tiny: the sim needs a handful of
/// operations, and pulling in a math crate for that would be silly.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Construct a vector.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean length.
    #[must_use]
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Unit vector in the same direction, or zero for zero.
    #[must_use]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > f32::EPSILON {
            self * len.recip()
        } else {
            Self::ZERO
        }
    }

    /// Dot product.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }

    /// Linear interpolation toward `other`, `t` clamped to `0..=1`.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::new(
            (other.x - self.x).mul_add(t, self.x),
            (other.y - self.y).mul_add(t, self.y),
        )
    }

    /// Unit vector at `angle` radians.
    #[must_use]
    pub fn from_angle(angle: f32) -> Self {
        Self::new(angle.cos(), angle.sin())
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

/// Something with a position worth interpolating. `prev_pos` is last tick's
/// position, so the renderer can blend rather than stutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub vel: Vec2,
}

impl Body {
    const fn at(pos: Vec2) -> Self {
        Self {
            pos,
            prev_pos: pos,
            vel: Vec2::ZERO,
        }
    }

    /// Position to draw at, blending `prev_pos` and `pos` by [`Sim::alpha`].
    #[must_use]
    pub fn interpolated(&self, alpha: f32) -> Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }

    /// Integrate one tick.
    fn step(&mut self) {
        self.prev_pos = self.pos;
        self.pos += self.vel * TICK_DT;
    }
}

/// The player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub body: Body,
    /// Seconds of invulnerability left after a hit.
    pub invuln: f32,
}

/// What a stomper is in the middle of.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Idle,
    /// Telegraphing a charge: it will dash along `aim` when `left` runs out.
    WindingUp {
        left: f32,
        aim: Vec2,
    },
    /// Dashing along `aim`.
    Dashing {
        left: f32,
        aim: Vec2,
    },
    /// Telegraphing a stomp: a shockwave when `left` runs out.
    Swelling {
        left: f32,
    },
    /// The shockwave, `age` seconds in.
    Shocking {
        age: f32,
    },
}

/// An enemy that only moves when the drums tell it to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stomper {
    pub body: Body,
    /// Direction of the next lunge while wandering.
    pub heading: Vec2,
    /// `0..=1`, set on each step and fading, for the renderer.
    pub pulse: f32,
    pub act: Act,
    /// How far the telegraph has come, `0..=1`, for the renderer.
    pub windup: f32,
}

impl Stomper {
    /// Radius of the shockwave right now, or zero.
    #[must_use]
    pub fn shock_radius(&self) -> f32 {
        match self.act {
            Act::Shocking { age } => SHOCK_RADIUS * (age / SHOCK_SECS).min(1.0),
            _ => 0.0,
        }
    }
}

/// A strike from the melody: lands, closes over a beat, bursts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub pos: Vec2,
    /// Seconds until it bursts.
    pub fuse: f32,
    /// Seconds it had to begin with.
    pub fuse_max: f32,
    /// Seconds of burst left, once the fuse has run out.
    pub burst: f32,
    /// The note that made it, for colouring.
    pub pitch: u8,
}

impl Strike {
    /// How far the fuse has burned, `0..=1`.
    #[must_use]
    pub fn closing(&self) -> f32 {
        1.0 - (self.fuse / self.fuse_max).clamp(0.0, 1.0)
    }

    /// Whether it is bursting right now.
    #[must_use]
    pub fn bursting(&self) -> bool {
        self.fuse <= 0.0 && self.burst > 0.0
    }
}

/// Where the game is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for a first press. The music has not started.
    Title,
    Playing,
    /// Out of hearts. The music has stopped.
    Over,
    /// Home. The music has resolved.
    Won,
}

/// Everything the sim is allowed to know about the outside world for one
/// frame. The sim never reads global input state, which is what makes replay
/// and testing possible.
// Four flags for four keys; a bitset would only make the call sites worse.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InputFrame {
    /// Movement, each axis in `-1..=1`.
    pub move_dir: Vec2,
    /// Any press: leaves the title screen.
    pub start: bool,
    /// Held to hold the music: the fermata.
    pub hold: bool,
    /// Edge-triggered: cue the next section.
    pub next_section: bool,
    /// Edge-triggered: flips the pause flag.
    pub toggle_pause: bool,
    /// Start over with a new seed.
    pub restart: Option<u64>,
}

/// Something that happened and is worth hearing. Music events pass through
/// verbatim; the rest are the game's own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// A note sounded. Pitch is a MIDI number; drums use [`song::KICK`],
    /// [`song::SNARE`] and [`song::HAT`].
    Note {
        instrument: Instrument,
        pitch: u8,
        velocity: f32,
    },
    /// The pad changed chord.
    Chord(Chord),
    /// A new section of the song began.
    Section(Motif),
    /// The fermata began or ended.
    Fermata { held: bool },
    /// Stompers began telegraphing a move.
    Windup,
    /// A stomper's shockwave went off.
    Shock,
    /// A stomper caught the player, or something moved into them.
    Hit { fatal: bool },
    /// A bell in the current key: a heart back, reaching home, starting
    /// over.
    Bell { pitch: u8 },
    /// Pause was toggled. `paused` is the state just entered.
    Pause { paused: bool },
    /// The game started over.
    Restart,
}

/// The simulation. Same seed plus same [`InputFrame`] sequence gives
/// bit-identical states on every run.
#[derive(Clone, Debug)]
pub struct Sim {
    seed: u64,
    rng: fastrand::Rng,
    sequencer: Sequencer,
    phase: Phase,
    paused: bool,
    player: Player,
    stompers: Vec<Stomper>,
    /// How far up each lane's walls are, `0..=1`.
    walls: [f32; WALL_LANES],
    strikes: Vec<Strike>,
    /// Seconds the gates stay shut.
    gate_shut: f32,
    /// Spinner angle, and the notch it is easing toward.
    spin: f32,
    spin_target: f32,
    /// The chord the pad last played, for the renderer's palette.
    chord: Option<Chord>,
    hearts: u32,
    /// The fermata pool, `0..=1`.
    pool: f32,
    /// The pool ran dry and has not yet refilled to [`FERMATA_RELOCK`].
    pool_dry: bool,
    /// Whether the music is being held right now.
    holding: bool,
    accumulator: f32,
    cues: Vec<Cue>,
    /// Scratch space for the sequencer, kept to avoid reallocating per tick.
    events: Vec<Event>,
}

impl Sim {
    /// Build a sim from a seed, on the title screen.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);
        let stompers = (0..STOMPER_COUNT)
            .map(|_| spawn_stomper(&mut rng, Vec2::ZERO))
            .collect();
        Self {
            seed,
            rng,
            sequencer: Sequencer::new(),
            phase: Phase::Title,
            paused: false,
            player: Player {
                body: Body::at(Vec2::ZERO),
                invuln: 0.0,
            },
            stompers,
            walls: [0.0; WALL_LANES],
            strikes: Vec::new(),
            gate_shut: 0.0,
            spin: 0.0,
            spin_target: 0.0,
            chord: None,
            hearts: MAX_HEARTS,
            pool: 1.0,
            pool_dry: false,
            holding: false,
            accumulator: 0.0,
            cues: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Seed this sim was built from.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Where the game is.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// Whether ticking is currently suspended by the player.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Hearts left.
    #[must_use]
    pub const fn hearts(&self) -> u32 {
        self.hearts
    }

    /// How much fermata is left to spend, `0..=1`.
    #[must_use]
    pub const fn pool(&self) -> f32 {
        self.pool
    }

    /// Whether the pool is refilling from empty and cannot be spent.
    #[must_use]
    pub const fn pool_dry(&self) -> bool {
        self.pool_dry
    }

    /// Whether the music is being held right now.
    #[must_use]
    pub const fn holding(&self) -> bool {
        self.holding
    }

    /// Whether the pad is sounding, and so refilling the pool.
    #[must_use]
    pub const fn charging(&self) -> bool {
        self.sequencer.is_active(Instrument::Pad)
    }

    /// How far along the way home the player is, `0..=1`. Home counts as
    /// all the way, however you arrived.
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.phase == Phase::Won {
            return 1.0;
        }
        (self.player.body.pos.x / HOME_DISTANCE).clamp(0.0, 1.0)
    }

    /// The player.
    #[must_use]
    pub const fn player(&self) -> &Player {
        &self.player
    }

    /// The stompers.
    #[must_use]
    pub fn stompers(&self) -> &[Stomper] {
        &self.stompers
    }

    /// How far up each lane's walls are, `0..=1`, indexed by lane.
    #[must_use]
    pub const fn walls(&self) -> &[f32; WALL_LANES] {
        &self.walls
    }

    /// Strikes in flight or bursting.
    #[must_use]
    pub fn strikes(&self) -> &[Strike] {
        &self.strikes
    }

    /// How shut the gates are, `0..=1`.
    #[must_use]
    pub fn gates(&self) -> f32 {
        (self.gate_shut / (GATE_SHUT_BEATS * self.motif().beat_secs())).clamp(0.0, 1.0)
    }

    /// Whether the gates are shut right now.
    #[must_use]
    pub fn gates_shut(&self) -> bool {
        self.gate_shut > 0.0
    }

    /// Angle of every spinner, in radians.
    #[must_use]
    pub const fn spin(&self) -> f32 {
        self.spin
    }

    /// The chord the pad last played, if it has.
    #[must_use]
    pub const fn chord(&self) -> Option<Chord> {
        self.chord
    }

    /// Where the music is.
    #[must_use]
    pub fn music(&self) -> Position {
        self.sequencer.position()
    }

    /// Current motif.
    #[must_use]
    pub const fn motif(&self) -> Motif {
        self.sequencer.motif()
    }

    /// How far the leftover accumulator has carried us into the next tick, in
    /// `0..1`. Feed this to [`Body::interpolated`].
    #[must_use]
    pub const fn alpha(&self) -> f32 {
        self.accumulator / TICK_DT
    }

    /// What the most recent [`Sim::advance`] produced worth hearing. Valid
    /// until the next call, which clears it.
    #[must_use]
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    /// Consume one frame's worth of real time, returning how many fixed ticks
    /// ran. `frame_dt` is clamped to [`MAX_FRAME_DT`].
    pub fn advance(&mut self, frame_dt: f32, input: &InputFrame) -> u32 {
        // Cues describe this frame only; last frame's have been consumed.
        self.cues.clear();

        if let Some(seed) = input.restart {
            *self = Self::new(seed);
            self.phase = Phase::Playing;
            self.cues.push(Cue::Restart);
            self.cues.push(Cue::Bell {
                pitch: self.motif().tonic().root(),
            });
        }
        if self.phase == Phase::Title && input.start {
            self.phase = Phase::Playing;
        }
        if self.phase != Phase::Playing {
            return 0;
        }
        if input.toggle_pause {
            self.paused = !self.paused;
            self.cues.push(Cue::Pause {
                paused: self.paused,
            });
        }
        if self.paused {
            return 0;
        }
        if input.next_section {
            self.sequencer.request_skip();
        }

        self.accumulator += frame_dt.clamp(0.0, MAX_FRAME_DT);
        let mut ticks = 0;
        // The float condition is the fixed-timestep idiom; the clamp above
        // bounds the loop at MAX_FRAME_DT / TICK_DT iterations.
        #[allow(clippy::while_float)]
        while self.accumulator >= TICK_DT && self.phase == Phase::Playing {
            self.accumulator -= TICK_DT;
            self.tick(input);
            ticks += 1;
        }
        ticks
    }

    /// One fixed step.
    fn tick(&mut self, input: &InputFrame) {
        self.apply_fermata(input.hold);
        if !self.holding {
            // The music first, so this tick's notes act on this tick.
            let mut events = std::mem::take(&mut self.events);
            events.clear();
            self.sequencer.advance(TICK_DT, &mut events);
            for event in &events {
                self.apply(*event);
            }
            self.events = events;

            self.move_stompers();
            self.act_stompers();
            self.sink_walls();
            self.burn_strikes();
            self.gate_shut = (self.gate_shut - TICK_DT).max(0.0);
            self.spin += (self.spin_target - self.spin) * (1.0 - (-SPIN_EASE * TICK_DT).exp());
        }
        self.move_player(input.move_dir);
        self.take_hits();
        self.arrive();
    }

    /// Spend the pool while the key is held, or let it refill under the
    /// pad. Announces the fermata starting and ending.
    fn apply_fermata(&mut self, hold: bool) {
        let usable = !self.pool_dry && self.pool > 0.0;
        let holding = hold && usable;
        if holding != self.holding {
            self.holding = holding;
            self.cues.push(Cue::Fermata { held: holding });
        }
        if holding {
            self.pool -= FERMATA_DRAIN * TICK_DT;
            if self.pool <= 0.0 {
                self.pool = 0.0;
                self.pool_dry = true;
            }
        } else if self.charging() {
            self.pool = FERMATA_RECHARGE.mul_add(TICK_DT, self.pool).min(1.0);
            if self.pool_dry && self.pool >= FERMATA_RELOCK {
                self.pool_dry = false;
            }
        }
    }

    /// Turn one music event into behaviour, and forward it as a cue.
    fn apply(&mut self, event: Event) {
        match event {
            Event::Note {
                instrument,
                pitch,
                velocity,
                step,
            } => {
                match instrument {
                    Instrument::Drums => self.drum_hit(pitch),
                    Instrument::Bass => self.raise_walls(pitch),
                    Instrument::Lead => self.drop_strike(pitch, step),
                    Instrument::Pad => {}
                }
                self.cues.push(Cue::Note {
                    instrument,
                    pitch,
                    velocity,
                });
            }
            Event::Chord(chord) => {
                self.chord = Some(chord);
                self.cues.push(Cue::Chord(chord));
            }
            Event::Bar { bar } => self.bar_begins(bar),
            Event::Section { motif } => {
                if motif == Motif::Lullaby && self.hearts < MAX_HEARTS {
                    self.hearts += 1;
                    self.cues.push(Cue::Bell {
                        pitch: motif.tonic().root() + 12,
                    });
                }
                self.cues.push(Cue::Section(motif));
            }
            Event::Layer { .. } => {}
        }
    }

    /// Indices of the stompers allowed to hunt right now: the nearest
    /// [`HUNTERS`] idle ones within `range`.
    fn hunters(&self, range: f32) -> Vec<usize> {
        let player_pos = self.player.body.pos;
        let mut near: Vec<(f32, usize)> = self
            .stompers
            .iter()
            .enumerate()
            .filter(|(_, s)| s.act == Act::Idle)
            .map(|(i, s)| ((s.body.pos - player_pos).length(), i))
            .filter(|(dist, _)| *dist < range)
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        near.into_iter().take(HUNTERS).map(|(_, i)| i).collect()
    }

    /// Kicks step the stompers, snares turn them and slam the gates, hats
    /// turn the spinners. Nothing but the geometry moves during the
    /// lullaby, whatever the drums do.
    fn drum_hit(&mut self, pitch: u8) {
        match pitch {
            song::SNARE => self.slam_gates(),
            song::HAT => self.spin_target += SPIN_STEP,
            _ => {}
        }
        let motif = self.motif();
        if motif == Motif::Lullaby {
            return;
        }
        let player_pos = self.player.body.pos;
        let hunters = self.hunters(SIGHT);
        for (i, stomper) in self.stompers.iter_mut().enumerate() {
            if stomper.act != Act::Idle {
                continue;
            }
            let hunting = hunters.contains(&i);
            match (motif, pitch) {
                (Motif::Pursuit, song::KICK) if hunting => {
                    let toward = (player_pos - stomper.body.pos).normalized();
                    stomper.heading = toward;
                    stomper.body.vel += toward * lunge(motif);
                    stomper.pulse = 1.0;
                }
                (Motif::Wander, song::KICK) => {
                    stomper.body.vel += stomper.heading * lunge(motif);
                    stomper.pulse = 1.0;
                }
                (Motif::Wander, song::SNARE) => {
                    // Half the time it looks your way.
                    stomper.heading = if self.rng.f32() < WANDER_NOTICE {
                        (player_pos - stomper.body.pos).normalized()
                    } else {
                        Vec2::from_angle(self.rng.f32() * TAU)
                    };
                    stomper.pulse = 0.6;
                }
                _ => {}
            }
        }
    }

    /// Every bar, stompers near enough telegraph their motif's move.
    fn bar_begins(&mut self, bar: u32) {
        let motif = self.motif();
        let beat = motif.beat_secs();
        let player_pos = self.player.body.pos;
        let hunters = match motif {
            Motif::Pursuit if bar % 2 == 0 => self.hunters(CHARGE_RANGE),
            Motif::Wander if bar % 2 == 1 => self.hunters(STOMP_RANGE),
            _ => return,
        };
        let mut any = false;
        for i in hunters {
            let stomper = &mut self.stompers[i];
            let offset = player_pos - stomper.body.pos;
            stomper.act = match motif {
                Motif::Pursuit => Act::WindingUp {
                    left: CHARGE_WINDUP_BEATS * beat,
                    aim: offset.normalized(),
                },
                _ => Act::Swelling {
                    left: STOMP_WINDUP_BEATS * beat,
                },
            };
            stomper.windup = 0.0;
            any = true;
        }
        if any {
            self.cues.push(Cue::Windup);
        }
    }

    /// Shut every standing wall's gaps for a beat. Anyone in one is hit and
    /// shoved out.
    fn slam_gates(&mut self) {
        let was_shut = self.gate_shut > 0.0;
        self.gate_shut = GATE_SHUT_BEATS * self.motif().beat_secs();
        if was_shut {
            return;
        }
        let pos = self.player.body.pos;
        let k = (pos.x / WALL_SPACING).round() as i32;
        let in_gap =
            !wall_stands_at(k, pos.y) && (pos.x - wall_x(k)).abs() < WALL_HALF_W + PLAYER_RADIUS;
        if in_gap && wall_is_solid(self.walls[lane_of_wall(k)]) {
            let away = Vec2::new(if pos.x < wall_x(k) { -1.0 } else { 1.0 }, 0.0);
            self.hurt(away);
        }
    }

    /// Raise a lane's walls. One rising into the player is a hit.
    fn raise_walls(&mut self, pitch: u8) {
        let lane = lane_of_pitch(pitch);
        let was_solid = wall_is_solid(self.walls[lane]);
        self.walls[lane] = 1.0;
        let pos = self.player.body.pos;
        let k = (pos.x / WALL_SPACING).round() as i32;
        if !was_solid && lane_of_wall(k) == lane && inside_wall(pos, PLAYER_RADIUS) {
            let away = Vec2::new(if pos.x < wall_x(k) { -1.0 } else { 1.0 }, 0.0);
            self.hurt(away);
        }
    }

    /// Drop a strike ahead of the player: further ahead the later in the
    /// bar the note fell, further to the side the further its pitch is from
    /// the middle of the tune. A bar of melody rains in its own shape.
    fn drop_strike(&mut self, pitch: u8, step: u32) {
        let motif = self.motif();
        let (lo, hi) = motif.lead_range();
        let span = f32::from(hi.saturating_sub(lo)).max(1.0);
        let ahead =
            (step as f32 / song::STEPS_PER_BAR as f32).mul_add(STRIKE_AHEAD_SPAN, STRIKE_AHEAD_MIN);
        // Higher notes land further north, which is up on screen.
        let lateral = (0.5 - f32::from(pitch.saturating_sub(lo)) / span) * STRIKE_LATERAL;
        let fuse = motif.beat_secs() * STRIKE_FUSE_BEATS;
        self.strikes.push(Strike {
            pos: self.player.body.pos + Vec2::new(ahead, lateral),
            fuse,
            fuse_max: fuse,
            burst: STRIKE_BURST_SECS,
            pitch,
        });
    }

    fn move_player(&mut self, move_dir: Vec2) {
        let dir = if move_dir.length() > 1.0 {
            move_dir.normalized()
        } else {
            move_dir
        };
        let target = dir * (PLAYER_SPEED * speed_scale(self.motif()));
        let body = &mut self.player.body;
        body.vel += (target - body.vel) * PLAYER_ACCEL;
        body.step();
        push_out(&self.walls, self.gate_shut > 0.0, body, PLAYER_RADIUS);
        self.player.invuln = (self.player.invuln - TICK_DT).max(0.0);
    }

    fn move_stompers(&mut self) {
        self.separate_stompers();
        let player_pos = self.player.body.pos;
        for i in 0..self.stompers.len() {
            let stomper = &mut self.stompers[i];
            if let Act::Dashing { aim, .. } = stomper.act {
                stomper.body.vel = aim * DASH_SPEED;
            } else {
                stomper.body.vel *= STOMPER_DAMPING;
            }
            stomper.body.step();
            push_out(
                &self.walls,
                self.gate_shut > 0.0,
                &mut stomper.body,
                STOMPER_RADIUS,
            );
            stomper.pulse = (stomper.pulse - TICK_DT / PULSE_FADE).max(0.0);
            // Left behind: come back somewhere new near the player.
            if (stomper.body.pos - player_pos).length() > FAR {
                self.stompers[i] = spawn_stomper(&mut self.rng, player_pos);
            }
        }
    }

    /// Advance every stomper's telegraphed move.
    fn act_stompers(&mut self) {
        let beat = self.motif().beat_secs();
        let mut shocked = false;
        for stomper in &mut self.stompers {
            stomper.act = match stomper.act {
                Act::Idle => Act::Idle,
                Act::WindingUp { left, aim } => {
                    stomper.windup = 1.0 - left / (CHARGE_WINDUP_BEATS * beat);
                    if left > TICK_DT {
                        Act::WindingUp {
                            left: left - TICK_DT,
                            aim,
                        }
                    } else {
                        stomper.pulse = 1.0;
                        Act::Dashing {
                            left: DASH_BEATS * beat,
                            aim,
                        }
                    }
                }
                Act::Dashing { left, aim } => {
                    if left > TICK_DT {
                        Act::Dashing {
                            left: left - TICK_DT,
                            aim,
                        }
                    } else {
                        stomper.windup = 0.0;
                        Act::Idle
                    }
                }
                Act::Swelling { left } => {
                    stomper.windup = 1.0 - left / (STOMP_WINDUP_BEATS * beat);
                    if left > TICK_DT {
                        Act::Swelling {
                            left: left - TICK_DT,
                        }
                    } else {
                        shocked = true;
                        stomper.pulse = 1.0;
                        Act::Shocking { age: 0.0 }
                    }
                }
                Act::Shocking { age } => {
                    if age < SHOCK_SECS {
                        Act::Shocking { age: age + TICK_DT }
                    } else {
                        stomper.windup = 0.0;
                        Act::Idle
                    }
                }
            };
        }
        if shocked {
            self.cues.push(Cue::Shock);
        }
    }

    /// Push overlapping stompers apart. Only pairs that actually overlap
    /// feel it, so it never changes where a lone stomper goes.
    fn separate_stompers(&mut self) {
        let reach = STOMPER_RADIUS * 2.0;
        for i in 0..self.stompers.len() {
            for j in (i + 1)..self.stompers.len() {
                let offset = self.stompers[j].body.pos - self.stompers[i].body.pos;
                let dist = offset.length();
                if dist >= reach {
                    continue;
                }
                // Coincident stompers have no direction to part along; give
                // them one so they do not stay fused.
                let away = if dist > f32::EPSILON {
                    offset * dist.recip()
                } else {
                    Vec2::new(1.0, 0.0)
                };
                let shove = away * (SEPARATION * (1.0 - dist / reach) * TICK_DT);
                self.stompers[i].body.vel += -shove;
                self.stompers[j].body.vel += shove;
            }
        }
    }

    fn sink_walls(&mut self) {
        let motif = self.motif();
        let rate = 1.0 / (sink_beats(motif) * motif.beat_secs());
        for wall in &mut self.walls {
            *wall = rate.mul_add(-TICK_DT, *wall).max(0.0);
        }
    }

    fn burn_strikes(&mut self) {
        let player_pos = self.player.body.pos;
        for strike in &mut self.strikes {
            if strike.fuse > 0.0 {
                strike.fuse -= TICK_DT;
            } else {
                strike.burst -= TICK_DT;
            }
        }
        self.strikes
            .retain(|s| s.burst > 0.0 && (s.pos - player_pos).length() < FAR);
    }

    /// Everything that hurts on contact: stompers, bursting strikes,
    /// shockwaves, spinners. The grace period after a hit covers them all.
    fn take_hits(&mut self) {
        if self.player.invuln > 0.0 {
            return;
        }
        let pos = self.player.body.pos;
        let reach = PLAYER_RADIUS + STOMPER_RADIUS;
        let mut away = None;
        for stomper in &self.stompers {
            let offset = pos - stomper.body.pos;
            let dist = offset.length();
            if dist < reach {
                away = Some(offset.normalized());
                break;
            }
            let ring = stomper.shock_radius();
            if ring > 0.0 && (dist - ring).abs() < SHOCK_BAND + PLAYER_RADIUS {
                away = Some(offset.normalized());
                break;
            }
        }
        if away.is_none() {
            away = self
                .strikes
                .iter()
                .find(|s| s.bursting() && (pos - s.pos).length() < STRIKE_RADIUS + PLAYER_RADIUS)
                .map(|s| (pos - s.pos).normalized());
        }
        if away.is_none() {
            away = spinner_normal(self.spin, pos, PLAYER_RADIUS);
        }
        if let Some(away) = away {
            self.hurt(away);
        }
    }

    /// Lose a heart, get shoved along `away`, and go briefly untouchable.
    fn hurt(&mut self, away: Vec2) {
        if self.player.invuln > 0.0 {
            return;
        }
        let away = if away.length() > f32::EPSILON {
            away.normalized()
        } else {
            Vec2::new(-1.0, 0.0)
        };
        self.player.body.vel = away * KNOCKBACK;
        self.player.invuln = INVULN_SECS;
        self.hearts -= 1;
        let fatal = self.hearts == 0;
        if fatal {
            self.phase = Phase::Over;
        }
        self.cues.push(Cue::Hit { fatal });
    }

    /// Reaching home ends the journey on the motif's home chord.
    fn arrive(&mut self) {
        if (self.player.body.pos - HOME).length() > HOME_RADIUS {
            return;
        }
        self.phase = Phase::Won;
        let tonic = self.motif().tonic();
        self.chord = Some(tonic);
        self.cues.push(Cue::Chord(tonic));
        self.cues.push(Cue::Bell {
            pitch: tonic.root() + 24,
        });
    }
}

/// Hooks for other modules' tests to stage a scene.
#[cfg(test)]
impl Sim {
    pub(crate) fn place_player(&mut self, pos: Vec2) {
        self.player.body = Body::at(pos);
    }

    pub(crate) fn shield_player(&mut self) {
        self.player.invuln = 10.0;
    }
}

/// Which lane a bass note raises: the twelve pitch classes split across the
/// lanes, so a bass line walks the walls up and down the road.
fn lane_of_pitch(pitch: u8) -> usize {
    usize::from(pitch % 12) * WALL_LANES / 12
}

/// Which lane the wall with grid index `k` belongs to. Walls cycle through
/// the lanes along the way home.
#[must_use]
pub const fn lane_of_wall(k: i32) -> usize {
    // Four lanes fit in an i32 on any target.
    #[allow(clippy::cast_possible_wrap)]
    let lanes = WALL_LANES as i32;
    k.rem_euclid(lanes) as usize
}

/// Centre line of the wall with grid index `k`.
#[must_use]
pub fn wall_x(k: i32) -> f32 {
    k as f32 * WALL_SPACING
}

/// Whether a wall at solidity `solidity` blocks.
#[must_use]
pub fn wall_is_solid(solidity: f32) -> bool {
    solidity > WALL_SOLID
}

/// How far the gaps in wall `k` sit from those of an even wall: every
/// other wall's gaps are half a period off, so the road zigzags.
#[must_use]
pub fn wall_gap_offset(k: i32) -> f32 {
    if k.rem_euclid(2) == 0 {
        0.0
    } else {
        WALL_PERIOD_Y * 0.5
    }
}

/// Whether wall `k` stands at `y`, or `y` falls in one of its gaps.
#[must_use]
pub fn wall_stands_at(k: i32, y: f32) -> bool {
    let yy = (y - wall_gap_offset(k)).rem_euclid(WALL_PERIOD_Y);
    yy > WALL_GAP * 0.5 && yy < WALL_GAP.mul_add(-0.5, WALL_PERIOD_Y)
}

/// How many wall columns apart the spinners stand.
pub const SPINNER_EVERY: i32 = 4;

/// Where the spinner between walls `k` and `k + 1` stands, if there is one.
///
/// Every fourth column has them, one per wall period, a quarter period
/// down from the even walls' gaps so they sit beside the way through
/// rather than on it.
#[must_use]
pub fn spinner_at(k: i32, m: i32) -> Option<Vec2> {
    if k.rem_euclid(SPINNER_EVERY) != 0 {
        return None;
    }
    Some(Vec2::new(
        (k as f32 + 0.5) * WALL_SPACING,
        (m as f32 + 0.25) * WALL_PERIOD_Y,
    ))
}

/// If a circle at `pos` touches the nearest spinner's bar, the direction
/// out of it.
fn spinner_normal(spin: f32, pos: Vec2, radius: f32) -> Option<Vec2> {
    let k = (pos.x / WALL_SPACING - 0.5).round() as i32;
    let m = (pos.y / WALL_PERIOD_Y - 0.25).round() as i32;
    let centre = spinner_at(k, m)?;
    let along = Vec2::from_angle(spin);
    let rel = pos - centre;
    let t = rel.dot(along).clamp(-SPINNER_HALF_LEN, SPINNER_HALF_LEN);
    let nearest = centre + along * t;
    let off = pos - nearest;
    if off.length() < SPINNER_HALF_W + radius {
        let normal = if off.length() > f32::EPSILON {
            off.normalized()
        } else {
            Vec2::new(-along.y, along.x)
        };
        Some(normal)
    } else {
        None
    }
}

/// Beats a wall takes to sink after its last note. The chase and the
/// lullaby both leave them standing longer: one to trap, one to linger.
const fn sink_beats(motif: Motif) -> f32 {
    match motif {
        Motif::Wander => 1.5,
        Motif::Pursuit => 2.5,
        Motif::Lullaby => 4.0,
    }
}

/// Player speed per motif, as a fraction of [`PLAYER_SPEED`].
const fn speed_scale(motif: Motif) -> f32 {
    match motif {
        Motif::Wander => 1.0,
        Motif::Pursuit => 1.15,
        Motif::Lullaby => 0.7,
    }
}

/// Speed a kick adds to a stomper.
const fn lunge(motif: Motif) -> f32 {
    match motif {
        Motif::Wander => 240.0,
        Motif::Pursuit => 360.0,
        Motif::Lullaby => 0.0,
    }
}

/// A stomper somewhere around `near`, far enough away to be fair, and not
/// inside a wall, which would shove it the moment the bass played. More
/// often than not it waits on the road ahead.
fn spawn_stomper(rng: &mut fastrand::Rng, near: Vec2) -> Stomper {
    let mut pos = near;
    for _ in 0..16 {
        let angle = if rng.f32() < AMBUSH_CHANCE {
            rng.f32().mul_add(2.0, -1.0) * AMBUSH_SPREAD
        } else {
            rng.f32() * TAU
        };
        let radius = rng.f32().mul_add(RESPAWN_SPAN, RESPAWN_MIN);
        pos = near + Vec2::from_angle(angle) * radius;
        if !inside_wall(pos, STOMPER_RADIUS) {
            break;
        }
    }
    Stomper {
        body: Body::at(pos),
        heading: Vec2::from_angle(rng.f32() * TAU),
        pulse: 0.0,
        act: Act::Idle,
        windup: 0.0,
    }
}

/// Whether a circle overlaps where a wall stands, up or not.
fn inside_wall(pos: Vec2, radius: f32) -> bool {
    let k = (pos.x / WALL_SPACING).round() as i32;
    let in_band = wall_stands_at(k, pos.y - radius) || wall_stands_at(k, pos.y + radius);
    in_band && (pos.x - wall_x(k)).abs() < WALL_HALF_W + radius
}

/// Keep a circle out of the nearest solid wall, gap included while the
/// gates are shut, by shoving it sideways. Walls are thin and tall, so
/// sideways is always the short way out.
fn push_out(walls: &[f32; WALL_LANES], gates_shut: bool, body: &mut Body, radius: f32) {
    // Walls are far enough apart that only the nearest can touch a body.
    let k = (body.pos.x / WALL_SPACING).round() as i32;
    if !wall_is_solid(walls[lane_of_wall(k)]) {
        return;
    }
    // Rounded to the body's reach, so it cannot clip a wall's end.
    let in_wall = gates_shut
        || wall_stands_at(k, body.pos.y - radius)
        || wall_stands_at(k, body.pos.y + radius);
    if !in_wall {
        return;
    }
    let x = wall_x(k);
    let overlap = WALL_HALF_W + radius - (body.pos.x - x).abs();
    if overlap <= 0.0 {
        return;
    }
    if body.pos.x < x {
        body.pos.x -= overlap;
        body.vel.x = body.vel.x.min(0.0);
    } else {
        body.pos.x += overlap;
        body.vel.x = body.vel.x.max(0.0);
    }
}

#[cfg(test)]
// Tests turn positive second counts into tick counts.
#[allow(clippy::cast_sign_loss)]
mod tests {
    use super::*;
    use crate::leitmotif::song::{BARS_PER_SECTION, SONG};

    /// A sim that has left the title screen.
    fn playing(seed: u64) -> Sim {
        let mut sim = Sim::new(seed);
        sim.advance(
            0.0,
            &InputFrame {
                start: true,
                ..InputFrame::default()
            },
        );
        assert_eq!(sim.phase(), Phase::Playing);
        sim
    }

    fn moving(x: f32, y: f32) -> InputFrame {
        InputFrame {
            move_dir: Vec2::new(x, y),
            ..InputFrame::default()
        }
    }

    const HOLDING: InputFrame = InputFrame {
        move_dir: Vec2::ZERO,
        start: false,
        hold: true,
        next_section: false,
        toggle_pause: false,
        restart: None,
    };

    /// Run `secs` of sim time at 60 Hz, counting cues that match.
    fn run(sim: &mut Sim, secs: f32, input: &InputFrame, want: impl Fn(&Cue) -> bool) -> usize {
        let ticks = (secs * 60.0).round() as u32;
        let mut count = 0;
        for _ in 0..ticks {
            sim.advance(TICK_DT, input);
            count += sim.cues().iter().filter(|c| want(c)).count();
        }
        count
    }

    /// Run until the section index is `section`, or panic. The player is
    /// kept invulnerable on the way, since a stomper ending the game early
    /// would stop the music and strand the test.
    fn run_to_section(sim: &mut Sim, section: usize) {
        for _ in 0..(60 * 120) {
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
            if sim.music().section == section {
                return;
            }
        }
        panic!("never reached section {section}");
    }

    fn section_of(motif: Motif) -> usize {
        SONG.iter().position(|s| s.motif == motif).unwrap()
    }

    fn snapshot(sim: &Sim) -> Vec<Vec2> {
        std::iter::once(sim.player().body.pos)
            .chain(sim.stompers().iter().map(|s| s.body.pos))
            .chain(sim.strikes().iter().map(|s| s.pos))
            .collect()
    }

    // Used as an `Fn(&Cue)` filter, so it takes a reference on purpose.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_hit(cue: &Cue) -> bool {
        matches!(cue, Cue::Hit { .. })
    }

    /// Where wall 1 stands, and where one of its gaps is. Wall 1 is odd,
    /// so its gaps sit half a period off the even walls'.
    const WALLED_Y: f32 = 0.0;
    const GAP_Y: f32 = WALL_PERIOD_Y * 0.5;

    /// A spot nowhere near a spinner, with no wall to speak of.
    fn clear_spot() -> Vec2 {
        Vec2::new(WALL_SPACING * 1.5, WALL_PERIOD_Y * 0.25)
    }

    #[test]
    fn title_screen_is_silent_and_still() {
        let mut sim = Sim::new(1);
        let cues = run(&mut sim, 2.0, &moving(1.0, 0.0), |_| true);
        assert_eq!(cues, 0);
        assert_eq!(sim.phase(), Phase::Title);
        assert_eq!(sim.player().body.pos, Vec2::ZERO);
    }

    #[test]
    fn same_seed_and_inputs_are_bit_identical() {
        let input = moving(0.7, -0.3);
        let mut a = playing(0xDEAD_BEEF);
        let mut b = playing(0xDEAD_BEEF);
        for _ in 0..600 {
            a.advance(TICK_DT, &input);
            b.advance(TICK_DT, &input);
        }
        assert_eq!(snapshot(&a), snapshot(&b));
        assert_eq!(a.hearts(), b.hearts());
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(snapshot(&Sim::new(1)), snapshot(&Sim::new(2)));
    }

    #[test]
    fn stompers_start_out_of_reach_and_out_of_walls() {
        for seed in 0..50 {
            let sim = Sim::new(seed);
            for stomper in sim.stompers() {
                let dist = stomper.body.pos.length();
                assert!(
                    dist >= RESPAWN_MIN,
                    "seed {seed}: stomper spawned {dist} away"
                );
                assert!(
                    !inside_wall(stomper.body.pos, STOMPER_RADIUS),
                    "seed {seed}: stomper spawned in a wall at {:?}",
                    stomper.body.pos
                );
            }
        }
    }

    #[test]
    fn accumulator_runs_whole_ticks_and_carries_the_remainder() {
        let mut sim = playing(1);
        assert_eq!(sim.advance(TICK_DT * 2.5, &InputFrame::default()), 2);
        assert!(
            (sim.alpha() - 0.5).abs() < 1e-3,
            "alpha was {}",
            sim.alpha()
        );
    }

    #[test]
    fn long_frame_dt_is_clamped() {
        let mut sim = playing(1);
        let ticks = sim.advance(10.0, &InputFrame::default());
        assert!(
            (14..=15).contains(&ticks),
            "clamped frame ran {ticks} ticks"
        );
    }

    #[test]
    fn pause_freezes_state_and_music() {
        let mut sim = playing(7);
        let toggle = InputFrame {
            toggle_pause: true,
            ..InputFrame::default()
        };
        sim.advance(TICK_DT, &toggle);
        assert!(sim.is_paused());
        assert_eq!(sim.cues(), [Cue::Pause { paused: true }]);

        let before = snapshot(&sim);
        let music = sim.music();
        let cues = run(&mut sim, 3.0, &moving(1.0, 0.0), |_| true);
        assert_eq!(cues, 0);
        assert_eq!(snapshot(&sim), before);
        assert_eq!(sim.music(), music);
    }

    #[test]
    fn music_starts_with_the_game_and_reaches_the_frontend() {
        let mut sim = playing(3);
        let notes = run(&mut sim, 1.0, &InputFrame::default(), |c| {
            matches!(c, Cue::Note { .. })
        });
        assert!(notes > 0, "a second of play made no notes");
        assert!(sim.cues().len() <= 16, "cues piling up: {:?}", sim.cues());
    }

    #[test]
    fn lead_notes_rain_strikes_ahead_that_burst_on_the_beat() {
        let mut sim = playing(3);
        sim.place_player(clear_spot());
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        assert!(
            !sim.strikes().is_empty(),
            "a second of lead dropped nothing"
        );
        let (lo, hi) = sim.motif().lead_range();
        for strike in sim.strikes() {
            assert!((lo..=hi).contains(&strike.pitch));
            let offset = strike.pos - sim.player().body.pos;
            assert!(
                offset.x >= STRIKE_AHEAD_MIN - 1.0,
                "strike behind: {offset:?}"
            );
            assert!(offset.x <= STRIKE_AHEAD_MIN + STRIKE_AHEAD_SPAN + 1.0);
            assert!(offset.y.abs() <= STRIKE_LATERAL.mul_add(0.5, 1.0));
        }
        // Standing still, none of them ever reach the player.
        let hits = run(&mut sim, 4.0, &InputFrame::default(), is_hit);
        assert_eq!(hits, 0);
        // But standing on one when it bursts is a hit.
        let strike = sim
            .strikes()
            .iter()
            .max_by(|a, b| a.fuse.total_cmp(&b.fuse))
            .copied()
            .expect("a strike in flight");
        sim.place_player(strike.pos);
        sim.player.invuln = 0.0;
        let hits = run(&mut sim, strike.fuse + 0.1, &InputFrame::default(), is_hit);
        assert_eq!(hits, 1, "standing on a burst was not a hit");
    }

    #[test]
    fn strikes_expire_after_bursting() {
        let mut sim = playing(3);
        sim.place_player(clear_spot());
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        let bar = sim.motif().bar_secs();
        // Stop the lead, then let everything it laid down go off.
        sim.sequencer
            .set_muted(Instrument::Lead, true, &mut Vec::new());
        run(&mut sim, bar, &InputFrame::default(), |_| true);
        assert!(
            sim.strikes().is_empty(),
            "{} strikes lingered",
            sim.strikes().len()
        );
    }

    #[test]
    fn bass_notes_raise_walls_which_then_sink() {
        let mut sim = playing(5);
        sim.place_player(clear_spot());
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        assert!(
            sim.walls().iter().any(|&w| wall_is_solid(w)),
            "no wall rose to the bass"
        );
        sim.sequencer
            .set_muted(Instrument::Bass, true, &mut Vec::new());
        let secs = sim.motif().beat_secs() * (sink_beats(sim.motif()) + 0.5);
        run(&mut sim, secs, &InputFrame::default(), |_| true);
        assert!(
            sim.walls().iter().all(|&w| w <= 0.0),
            "walls stayed up without bass: {:?}",
            sim.walls()
        );
    }

    #[test]
    fn a_wall_rising_into_you_hurts() {
        let mut sim = playing(5);
        // Stand inside wall 1 while it is down, then raise its lane.
        sim.place_player(Vec2::new(wall_x(1), WALLED_Y));
        sim.walls = [0.0; WALL_LANES];
        let lane = lane_of_wall(1);
        let pitch = (0..12_u8).find(|&p| lane_of_pitch(p) == lane).unwrap() + 36;
        sim.raise_walls(pitch);
        assert_eq!(sim.cues(), [Cue::Hit { fatal: false }]);
        assert_eq!(sim.hearts(), MAX_HEARTS - 1);
    }

    /// Drive east from just west of wall 1 for a second, holding every
    /// lane at `solidity`, and return where the player ended up.
    fn drive_at_wall(y: f32, solidity: f32) -> f32 {
        let mut sim = playing(5);
        sim.place_player(Vec2::new(wall_x(1) - 100.0, y));
        for _ in 0..60 {
            sim.walls = [solidity; WALL_LANES];
            sim.gate_shut = 0.0;
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &moving(1.0, 0.0));
        }
        sim.player().body.pos.x
    }

    #[test]
    fn solid_walls_block_the_player() {
        let x = drive_at_wall(WALLED_Y, 1.0);
        assert!(
            x <= wall_x(1) - WALL_HALF_W - PLAYER_RADIUS + 0.01,
            "player at {x} passed a wall at {}",
            wall_x(1)
        );
    }

    #[test]
    fn sunk_walls_do_not_block() {
        assert!(drive_at_wall(WALLED_Y, 0.0) > wall_x(1));
    }

    #[test]
    fn wall_gaps_let_the_player_through() {
        assert!(!wall_stands_at(1, GAP_Y));
        assert!(wall_stands_at(1, WALLED_Y));
        assert!(drive_at_wall(GAP_Y, 1.0) > wall_x(1));
        assert!(drive_at_wall(WALL_PERIOD_Y.mul_add(3.0, GAP_Y), 1.0) > wall_x(1));
    }

    #[test]
    fn neighbouring_walls_stagger_their_gaps() {
        // A gap in one wall faces the middle of a segment in the next, so
        // no straight line east threads every wall.
        for k in -4..4 {
            assert_ne!(wall_stands_at(k, GAP_Y), wall_stands_at(k + 1, GAP_Y));
            assert_ne!(wall_stands_at(k, WALLED_Y), wall_stands_at(k + 1, WALLED_Y));
        }
    }

    #[test]
    fn snares_slam_the_gates_and_catch_whoever_is_in_one() {
        let mut sim = playing(5);
        sim.walls = [1.0; WALL_LANES];
        sim.place_player(Vec2::new(wall_x(1), GAP_Y));
        sim.player.invuln = 0.0;
        assert!(!sim.gates_shut());
        sim.drum_hit(song::SNARE);
        assert!(sim.gates_shut());
        assert!(sim.gates() > 0.99);
        assert_eq!(sim.cues(), [Cue::Hit { fatal: false }]);
        // Shut gates block the gap like the wall around it.
        sim.place_player(Vec2::new(wall_x(1) - 60.0, GAP_Y));
        sim.player.invuln = 10.0;
        for _ in 0..30 {
            sim.walls = [1.0; WALL_LANES];
            sim.gate_shut = 1.0;
            sim.advance(TICK_DT, &moving(1.0, 0.0));
        }
        assert!(sim.player().body.pos.x < wall_x(1) - WALL_HALF_W);
        // And a snare on a sunk wall's gap is harmless.
        let mut sim = playing(5);
        sim.walls = [0.0; WALL_LANES];
        sim.place_player(Vec2::new(wall_x(1), GAP_Y));
        sim.player.invuln = 0.0;
        sim.drum_hit(song::SNARE);
        assert!(sim.cues().is_empty());
    }

    #[test]
    fn hats_turn_the_spinners_and_spinners_hurt() {
        let mut sim = playing(5);
        assert!(sim.spin().abs() < 1e-6);
        for _ in 0..4 {
            sim.drum_hit(song::HAT);
        }
        sim.place_player(clear_spot());
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        assert!(
            4.0_f32.mul_add(-SPIN_STEP, sim.spin()).abs() < 0.05,
            "spin {}",
            sim.spin()
        );

        // A spinner stands between walls 0 and 1; its bar lies along the
        // current spin. Stand on the bar's end.
        let centre = spinner_at(0, 0).unwrap();
        let along = Vec2::from_angle(sim.spin());
        sim.place_player(centre + along * (SPINNER_HALF_LEN * 0.5));
        sim.player.invuln = 0.0;
        sim.advance(TICK_DT, &InputFrame::default());
        assert!(sim.cues().contains(&Cue::Hit { fatal: false }));
        // Off the bar is fine.
        sim.place_player(centre + Vec2::new(-along.y, along.x) * 60.0);
        sim.player.invuln = 0.0;
        sim.advance(TICK_DT, &InputFrame::default());
        assert!(!sim.cues().iter().any(is_hit));
        assert!(spinner_at(1, 0).is_none(), "only every fourth column spins");
        assert!(spinner_at(2, 0).is_none(), "only every fourth column spins");
        assert!(spinner_at(4, 0).is_some());
    }

    #[test]
    fn stompers_only_move_to_drums() {
        // The intro has no drums arranged, so nothing should move.
        let mut sim = playing(11);
        sim.place_player(clear_spot());
        let before: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        let after: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        assert_eq!(before, after, "stompers moved with no drums");

        // Section 1 brings the drums in.
        run_to_section(&mut sim, 1);
        run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        let moved: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        assert_ne!(after, moved, "stompers ignored the drums");
    }

    #[test]
    fn stompers_sleep_through_the_lullaby_but_still_hurt() {
        let mut sim = playing(11);
        run_to_section(&mut sim, section_of(Motif::Lullaby));
        // Let any lingering momentum from the chase die down.
        run(&mut sim, 1.5, &InputFrame::default(), |_| true);
        let before: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        run(&mut sim, 3.0, &InputFrame::default(), |_| true);
        let after: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        for (a, b) in before.iter().zip(&after) {
            assert!(
                (*a - *b).length() < 1.0,
                "a stomper moved during the lullaby"
            );
        }
        assert!(sim.stompers().iter().all(|s| s.act == Act::Idle));
        sim.place_player(sim.stompers()[0].body.pos);
        sim.player.invuln = 0.0;
        let hearts = sim.hearts();
        let hits = run(&mut sim, 0.5, &InputFrame::default(), is_hit);
        assert_eq!(hits, 1);
        assert_eq!(sim.hearts(), hearts - 1);
    }

    #[test]
    fn the_lullaby_gives_a_heart_back() {
        let mut sim = playing(11);
        sim.hearts = 1;
        run_to_section(&mut sim, section_of(Motif::Lullaby));
        assert_eq!(sim.hearts(), 2);
        assert!(sim.cues().iter().any(|c| matches!(c, Cue::Bell { .. })));
    }

    #[test]
    fn pursuit_stompers_wind_up_then_dash_where_they_aimed() {
        let mut sim = playing(11);
        run_to_section(&mut sim, section_of(Motif::Pursuit));
        // Stage it in the gap row of wall 1, with open ground between: the
        // stomper at the east end of the column, the player at the west.
        let here = Vec2::new(wall_x(1) + 15.0, WALL_PERIOD_Y * 0.5);
        let start = Vec2::new(wall_x(2) - 30.0, WALL_PERIOD_Y * 0.5);
        // Only a wind-up that began from `start` counts: one left over
        // from an earlier bar would be aimed at where the player used to be.
        let mut placed = false;
        let mut wound = false;
        for _ in 0..(60 * 8) {
            sim.place_player(here);
            sim.player.invuln = 10.0;
            if sim.stompers[0].act == Act::Idle {
                sim.stompers[0].body = Body::at(start);
                placed = true;
            }
            sim.advance(TICK_DT, &InputFrame::default());
            if placed && matches!(sim.stompers()[0].act, Act::WindingUp { .. }) {
                wound = true;
                break;
            }
        }
        assert!(wound, "the nearest stomper never wound up in a pursuit bar");
        let Act::WindingUp { aim, .. } = sim.stompers()[0].act else {
            unreachable!()
        };
        // Aimed at the player, and going nowhere yet.
        let at = sim.stompers()[0].body.pos;
        assert!(
            aim.dot((here - at).normalized()) > 0.99,
            "aim {aim:?} from {at:?} toward {here:?}"
        );
        let beat = sim.motif().beat_secs();
        for _ in 0..((beat * 0.9) / TICK_DT) as u32 {
            sim.place_player(here);
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
        }
        assert!(matches!(sim.stompers()[0].act, Act::WindingUp { .. }));
        assert!((sim.stompers()[0].body.pos - start).length() < 40.0);
        // Then it dashes along the aim, fast.
        for _ in 0..((beat * 0.5) / TICK_DT) as u32 {
            sim.place_player(here);
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
        }
        assert!(matches!(sim.stompers()[0].act, Act::Dashing { .. }));
        let travelled = sim.stompers()[0].body.pos - start;
        assert!(
            travelled.length() > 100.0,
            "dashed only {}",
            travelled.length()
        );
        assert!(travelled.normalized().dot(aim) > 0.9);
    }

    #[test]
    fn wander_stompers_swell_then_shock() {
        let mut sim = playing(11);
        run_to_section(&mut sim, 1);
        let here = clear_spot();
        // Park a stomper close and wait for an odd bar.
        let mut shocked = false;
        for _ in 0..(60 * 12) {
            sim.place_player(here);
            sim.player.invuln = 10.0;
            if sim.stompers[0].act == Act::Idle {
                sim.stompers[0].body = Body::at(here + Vec2::new(60.0, 0.0));
            }
            sim.advance(TICK_DT, &InputFrame::default());
            if sim.cues().contains(&Cue::Shock) {
                shocked = true;
                break;
            }
        }
        assert!(shocked, "no stomp in twelve seconds of wander");
        assert!(matches!(sim.stompers()[0].act, Act::Shocking { .. }));
        // The ring reaches the player, and hurts as it passes.
        sim.player.invuln = 0.0;
        let hits = run(&mut sim, SHOCK_SECS + 0.05, &InputFrame::default(), is_hit);
        assert_eq!(hits, 1);
    }

    #[test]
    fn stompers_do_not_stack_while_chasing() {
        let mut sim = playing(11);
        run_to_section(&mut sim, section_of(Motif::Pursuit));
        // Hold still with infinite hearts and let the pack arrive, then
        // count how often any two of them are deeply overlapped.
        let here = sim.player().body.pos;
        let (mut overlapped, mut pairs) = (0_u32, 0_u32);
        for tick in 0..(60 * 8) {
            sim.place_player(here);
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
            if tick < 60 * 3 {
                continue;
            }
            for (i, a) in sim.stompers().iter().enumerate() {
                for b in &sim.stompers()[i + 1..] {
                    pairs += 1;
                    if (a.body.pos - b.body.pos).length() < STOMPER_RADIUS {
                        overlapped += 1;
                    }
                }
            }
        }
        // Crossing paths on a lunge is fine; fusing into one blob is not.
        let share = overlapped as f32 / pairs as f32;
        assert!(
            share < 0.1,
            "stompers spent {:.0}% of pair-ticks fused together",
            share * 100.0
        );
    }

    #[test]
    fn stompers_follow_the_player_across_the_world() {
        let mut sim = playing(11);
        for _ in 0..(60 * 20) {
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &moving(1.0, 0.0));
        }
        let here = sim.player().body.pos;
        assert!(here.x > 3000.0, "player only got to {here:?}");
        for stomper in sim.stompers() {
            let dist = (stomper.body.pos - here).length();
            assert!(dist <= FAR, "a stomper was left {dist} behind");
        }
    }

    #[test]
    fn hits_cost_hearts_then_the_game() {
        let mut sim = playing(11);
        run_to_section(&mut sim, 1);
        sim.player.invuln = 0.0;
        let mut hits = 0;
        for _ in 0..(60 * 30) {
            // Keep teleporting onto a stomper until it is over.
            sim.place_player(sim.stompers()[0].body.pos);
            sim.advance(TICK_DT, &InputFrame::default());
            hits += sim.cues().iter().filter(|c| is_hit(c)).count();
            if sim.phase() == Phase::Over {
                break;
            }
        }
        assert_eq!(hits, MAX_HEARTS as usize);
        assert_eq!(sim.hearts(), 0);
        assert_eq!(sim.phase(), Phase::Over);
        assert_eq!(sim.cues().last(), Some(&Cue::Hit { fatal: true }));

        // Over means over: no more ticks, no more music.
        let cues = run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        assert_eq!(cues, 0);
    }

    #[test]
    fn hits_grant_a_grace_period() {
        let mut sim = playing(11);
        run_to_section(&mut sim, 1);
        sim.player.invuln = 0.0;
        let mut first = None;
        for tick in 0..(60 * 5) {
            sim.place_player(sim.stompers()[0].body.pos);
            sim.advance(TICK_DT, &InputFrame::default());
            if sim.cues().iter().any(is_hit) {
                match first {
                    None => first = Some(tick),
                    Some(at) => {
                        let gap = (tick - at) as f32 * TICK_DT;
                        assert!(gap >= INVULN_SECS - TICK_DT, "second hit after {gap}s");
                        return;
                    }
                }
            }
        }
        panic!("expected two hits in five seconds");
    }

    #[test]
    fn the_fermata_holds_the_music_and_everything_it_drives() {
        let mut sim = playing(3);
        run_to_section(&mut sim, 1);
        sim.place_player(clear_spot());
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        let music = sim.music();
        let stompers: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        let strikes: Vec<f32> = sim.strikes().iter().map(|s| s.fuse).collect();
        let walls = *sim.walls();

        // The first held tick says so; nothing sounds; nothing moves.
        sim.advance(TICK_DT, &HOLDING);
        assert!(sim.cues().contains(&Cue::Fermata { held: true }));
        let notes = run(&mut sim, 1.0, &HOLDING, |c| {
            matches!(c, Cue::Note { .. } | Cue::Chord(_))
        });
        assert_eq!(notes, 0);
        assert_eq!(sim.music(), music);
        let held: Vec<Vec2> = sim.stompers().iter().map(|s| s.body.pos).collect();
        assert_eq!(held, stompers);
        let fuses: Vec<f32> = sim.strikes().iter().map(|s| s.fuse).collect();
        assert_eq!(fuses, strikes);
        assert!(
            sim.walls()
                .iter()
                .zip(&walls)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
        let spent = 1.0 - sim.pool();
        assert!(
            (spent - FERMATA_DRAIN).abs() < 0.02,
            "a second spent {spent}"
        );

        // But the player still moves.
        let before = sim.player().body.pos;
        run(
            &mut sim,
            0.2,
            &InputFrame {
                hold: true,
                ..moving(1.0, 0.0)
            },
            |_| true,
        );
        assert!(sim.player().body.pos.x > before.x + 20.0);

        // Letting go announces it and the music picks up where it was.
        sim.advance(TICK_DT, &InputFrame::default());
        assert!(sim.cues().contains(&Cue::Fermata { held: false }));
        assert_eq!(sim.music().section, music.section);
        let notes = run(&mut sim, 1.0, &InputFrame::default(), |c| {
            matches!(c, Cue::Note { .. })
        });
        assert!(notes > 0);
    }

    #[test]
    fn the_pool_refills_only_under_the_pad() {
        // The intro has no pad: no refilling.
        let mut sim = playing(3);
        sim.place_player(clear_spot());
        run(&mut sim, 1.0, &HOLDING, |_| true);
        let after_hold = sim.pool();
        run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        assert!(
            (sim.pool() - after_hold).abs() < 1e-5,
            "refilled without a pad"
        );
        assert!(!sim.charging());

        // Section 1 has one.
        run_to_section(&mut sim, 1);
        assert!(sim.charging());
        let before = sim.pool();
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        assert!((sim.pool() - before - FERMATA_RECHARGE).abs() < 0.02);
    }

    #[test]
    fn a_dry_pool_lets_go_and_relocks_until_refilled() {
        let mut sim = playing(3);
        run_to_section(&mut sim, 1);
        sim.place_player(clear_spot());
        // Hold well past the pool's worth: the music comes back on its own.
        run(&mut sim, 1.0 / FERMATA_DRAIN + 0.2, &HOLDING, |_| true);
        assert!(sim.pool_dry());
        assert!(!sim.holding());
        let notes = run(&mut sim, 1.0, &HOLDING, |c| matches!(c, Cue::Note { .. }));
        assert!(notes > 0, "music stayed held on an empty pool");
        // Keep holding: it plays until the pool refills to the relock mark,
        // then holds again. (The second above already refilled some.)
        let to_relock = FERMATA_RELOCK / FERMATA_RECHARGE - 1.0 + 0.2;
        run(&mut sim, to_relock, &HOLDING, |_| true);
        assert!(!sim.pool_dry());
        assert!(sim.holding());
    }

    #[test]
    fn reaching_home_wins_and_resolves() {
        let mut sim = playing(5);
        sim.place_player(HOME - Vec2::new(HOME_RADIUS + 150.0, 0.0));
        let mut cues = Vec::new();
        for _ in 0..(60 * 5) {
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &moving(1.0, 0.0));
            cues.extend_from_slice(sim.cues());
            if sim.phase() == Phase::Won {
                break;
            }
        }
        assert_eq!(sim.phase(), Phase::Won);
        assert!((sim.progress() - 1.0).abs() < 1e-6);
        let tonic = sim.motif().tonic();
        let at = cues
            .iter()
            .position(|c| {
                *c == Cue::Bell {
                    pitch: tonic.root() + 24,
                }
            })
            .expect("home bell");
        // Resolves on the motif's home chord, cued just before.
        assert_eq!(cues[at - 1], Cue::Chord(tonic));
        assert_eq!(sim.chord(), Some(tonic));

        // Won means done: no more ticks, no more music.
        let more = run(&mut sim, 2.0, &moving(1.0, 0.0), |_| true);
        assert_eq!(more, 0);
    }

    #[test]
    fn progress_tracks_the_way_home() {
        let mut sim = playing(5);
        assert!(sim.progress().abs() < 1e-6);
        sim.place_player(Vec2::new(HOME_DISTANCE * 0.5, 300.0));
        sim.advance(0.0, &InputFrame::default());
        assert!((sim.progress() - 0.5).abs() < 1e-3);
        sim.place_player(Vec2::new(-500.0, 0.0));
        assert!(
            sim.progress().abs() < 1e-6,
            "walking backwards is not negative progress"
        );
    }

    #[test]
    fn restart_replaces_the_world_and_skips_the_title() {
        let mut sim = playing(5);
        run(&mut sim, 5.0, &moving(1.0, 0.0), |_| true);
        sim.advance(
            TICK_DT,
            &InputFrame {
                restart: Some(6),
                ..InputFrame::default()
            },
        );
        assert_eq!(sim.seed(), 6);
        assert_eq!(sim.phase(), Phase::Playing);
        assert_eq!(sim.hearts(), MAX_HEARTS);
        assert_eq!(sim.cues().first(), Some(&Cue::Restart));
        assert_eq!(sim.music().section, 0);
        assert!(sim.progress() < 0.01);
    }

    #[test]
    fn next_section_skips_at_the_bar() {
        let mut sim = playing(5);
        run(&mut sim, 0.1, &InputFrame::default(), |_| true);
        sim.advance(
            TICK_DT,
            &InputFrame {
                next_section: true,
                ..InputFrame::default()
            },
        );
        assert_eq!(sim.music().section, 0);
        let bar = sim.motif().bar_secs();
        let sections = run(&mut sim, bar, &InputFrame::default(), |c| {
            matches!(c, Cue::Section(_))
        });
        assert_eq!(sections, 1);
        assert_eq!(sim.music().section, 1);
    }

    #[test]
    fn cues_do_not_outlive_the_frame_that_made_them() {
        let mut sim = playing(7);
        sim.advance(
            TICK_DT,
            &InputFrame {
                toggle_pause: true,
                ..InputFrame::default()
            },
        );
        assert!(!sim.cues().is_empty());
        sim.advance(TICK_DT, &InputFrame::default());
        assert!(sim.cues().is_empty(), "stale cues: {:?}", sim.cues());
    }

    #[test]
    fn a_full_song_plays_through_without_incident() {
        let mut sim = playing(0xA5);
        let total: f32 = SONG
            .iter()
            .map(|s| s.motif.bar_secs() * BARS_PER_SECTION as f32)
            .sum();
        let mut sections = 0;
        for tick in 0..((total + 1.0) * 60.0) as u32 {
            sim.player.invuln = 10.0;
            // Wander about, so strikes and walls get exercised.
            let dir = Vec2::from_angle(tick as f32 * 0.01);
            sim.advance(TICK_DT, &moving(dir.x, dir.y));
            sections += sim
                .cues()
                .iter()
                .filter(|c| matches!(c, Cue::Section(_)))
                .count();
        }
        // Every section announced once, plus the wrap back to the top.
        assert_eq!(sections, SONG.len() + 1);
        assert_eq!(sim.phase(), Phase::Playing);
    }

    #[test]
    fn interpolation_walks_from_prev_to_current() {
        let mut sim = playing(9);
        sim.advance(TICK_DT * 4.0, &moving(1.0, 0.0));
        let body = sim.player().body;
        assert_eq!(body.interpolated(0.0), body.prev_pos);
        assert_eq!(body.interpolated(1.0), body.pos);
        assert_ne!(body.prev_pos, body.pos);
    }

    #[test]
    fn lanes_cover_every_wall() {
        let lanes: std::collections::BTreeSet<usize> = (0..12).map(lane_of_pitch).collect();
        assert_eq!(lanes.len(), WALL_LANES);
        assert!(lanes.iter().all(|&lane| lane < WALL_LANES));
        for k in -8..8 {
            assert!(lane_of_wall(k) < WALL_LANES);
        }
        assert_eq!(lane_of_wall(-1), WALL_LANES - 1);
    }

    /// Mostly east, weaving a little, the way a person plays when they
    /// have decided the game is simple.
    fn weaving(tick: u32) -> InputFrame {
        moving(1.0, (tick as f32 * 0.05).sin() * 0.8)
    }

    #[test]
    fn holding_right_and_weaving_does_not_get_you_home() {
        for seed in [1_u64, 2, 3, 5, 8, 13] {
            let mut sim = playing(seed);
            for tick in 0..(60 * 180) {
                sim.advance(TICK_DT, &weaving(tick));
                if sim.phase() != Phase::Playing {
                    break;
                }
            }
            assert_eq!(
                sim.phase(),
                Phase::Over,
                "seed {seed}: weaving east got {:.0}% of the way",
                sim.progress() * 100.0
            );
        }
    }

    /// A bot that reads the road: heads for the next wall's nearest gap,
    /// crosses right after the gates open (holding the music while it
    /// does), steps off strikes and aim lines, gets clear of shockwaves,
    /// slides around spinners, and holds the music when it gets crowded.
    fn careful(sim: &Sim) -> InputFrame {
        let me = sim.player().body.pos;
        let k = (me.x / WALL_SPACING).floor() as i32 + 1;
        let offset = wall_gap_offset(k);
        let m = ((me.y - offset) / WALL_PERIOD_Y).round();
        let mut gap_y = m.mul_add(WALL_PERIOD_Y, offset);
        // A stomper parked in the doorway: use the next door along.
        let doorway = |y: f32| {
            sim.stompers()
                .iter()
                .any(|s| (s.body.pos - Vec2::new(wall_x(k), y)).length() < 70.0)
        };
        if doorway(gap_y) {
            let other = if me.y >= gap_y {
                gap_y + WALL_PERIOD_Y
            } else {
                gap_y - WALL_PERIOD_Y
            };
            if !doorway(other) {
                gap_y = other;
            }
        }
        let to_wall = wall_x(k) - me.x;
        // The gap is 64 tall and the player 22 wide: 20 either way fits.
        let aligned = (gap_y - me.y).abs() <= 20.0;
        // Snares fall on steps 4 and 12 (and 15 in pursuit); the gates stay
        // shut a beat after each, so cross right after they open.
        let step = sim.music().step;
        let safe_to_cross = !sim.gates_shut()
            && (step % 8 <= 2 || !sim.pool_dry() || sim.motif() == Motif::Lullaby);
        let crossing = to_wall.abs() < 50.0;
        // Near the door, always steer for the gap's centre; only the push
        // east waits on being lined up and on the gates.
        let centring = (gap_y - me.y).clamp(-40.0, 40.0) / 40.0;
        let mut dir = if me.x > HOME.x - 700.0 {
            (HOME - me).normalized()
        } else if to_wall > 140.0 {
            // Far from the door: make for it diagonally.
            (Vec2::new(wall_x(k) - 120.0, gap_y) - me).normalized()
        } else if aligned && safe_to_cross {
            Vec2::new(1.0, centring)
        } else if to_wall < 40.0 && !aligned {
            Vec2::new(-0.4, centring)
        } else {
            Vec2::new(0.0, centring)
        };
        let mut crowded = 0;
        for stomper in sim.stompers() {
            let off = stomper.body.pos - me;
            let dist = off.length();
            if dist < 110.0 {
                dir += -off.normalized() * 2.5;
            }
            if dist < 220.0 {
                crowded += 1;
            }
            if matches!(stomper.act, Act::Swelling { .. } | Act::Shocking { .. })
                && dist < SHOCK_RADIUS + 90.0
            {
                // Get clear of the wave before it comes.
                dir += -off.normalized() * 3.0;
            }
            if let Act::WindingUp { aim, .. } | Act::Dashing { aim, .. } = stomper.act {
                // Step off the line it is about to run down.
                let along = off.dot(aim);
                if along < 0.0 && along > -600.0 {
                    let side = Vec2::new(-aim.y, aim.x);
                    let sign = if side.dot(-off) >= 0.0 { 1.0 } else { -1.0 };
                    dir += side * (2.0 * sign);
                }
            }
        }
        for strike in sim.strikes() {
            let off = me - strike.pos;
            if off.length() < STRIKE_RADIUS + 30.0 {
                dir += off.normalized() * 1.2;
            }
        }
        // Slide around spinners rather than into them.
        let sk = (me.x / WALL_SPACING - 0.5).round() as i32;
        let sm = (me.y / WALL_PERIOD_Y - 0.25).round() as i32;
        if let Some(centre) = spinner_at(sk, sm) {
            let off = me - centre;
            if off.length() < SPINNER_HALF_LEN + 70.0 {
                let around = Vec2::new(-off.y, off.x).normalized();
                let sign = if around.dot(dir) >= 0.0 { 1.0 } else { -1.0 };
                dir += around * (2.0 * sign) + off.normalized();
            }
        }
        // Do not loiter inside a wall's footprint: it may rise. Get out the
        // side already closer, unless this is the gap being threaded.
        let kk = (me.x / WALL_SPACING).round() as i32;
        let gap_here = ((me.y - wall_gap_offset(kk)) / WALL_PERIOD_Y)
            .round()
            .mul_add(WALL_PERIOD_Y, wall_gap_offset(kk));
        if (me.x - wall_x(kk)).abs() < WALL_HALF_W + PLAYER_RADIUS + 10.0
            && (gap_here - me.y).abs() > 26.0
        {
            dir = Vec2::new(if me.x >= wall_x(kk) { 1.0 } else { -1.0 }, 0.0);
        }
        InputFrame {
            move_dir: dir.normalized(),
            hold: !sim.pool_dry()
                && ((crowded >= 2 && sim.pool() > 0.3)
                    || (crossing && aligned && sim.pool() > 0.6)),
            ..InputFrame::default()
        }
    }

    /// A tuning aid: where the careful bot is every few seconds on one seed.
    #[test]
    #[ignore = "diagnostic; prints a trace"]
    fn trace_the_careful_bot() {
        let mut sim = playing(1);
        for tick in 0..(60 * 60) {
            let input = careful(&sim);
            sim.advance(TICK_DT, &input);
            let hit = sim.cues().iter().any(is_hit);
            if tick % 60 == 0 || hit {
                let me = sim.player().body.pos;
                let cause = if !hit {
                    "-"
                } else if sim.stompers().iter().any(|s| {
                    s.shock_radius() > 0.0
                        && ((s.body.pos - me).length() - s.shock_radius()).abs() < 60.0
                }) {
                    "shock"
                } else if sim
                    .stompers()
                    .iter()
                    .any(|s| (s.body.pos - me).length() < PLAYER_RADIUS + STOMPER_RADIUS + 30.0)
                {
                    "stomper"
                } else if sim
                    .strikes()
                    .iter()
                    .any(|s| (s.pos - me).length() < STRIKE_RADIUS + PLAYER_RADIUS + 12.0)
                {
                    "strike"
                } else if spinner_normal(sim.spin(), me, PLAYER_RADIUS + 12.0).is_some() {
                    "spinner"
                } else {
                    "gate/wall"
                };
                println!(
                    "t={:>3} pos=({:>6.0},{:>6.0}) vel=({:+.0},{:+.0}) dir=({:+.1},{:+.1}) hold={} pool={:.2} charging={} section={} gates={} motif={:?} hearts={} {cause}",
                    tick / 60,
                    me.x,
                    me.y,
                    sim.player().body.vel.x,
                    sim.player().body.vel.y,
                    input.move_dir.x,
                    input.move_dir.y,
                    input.hold,
                    sim.pool(),
                    sim.charging(),
                    sim.music().section,
                    sim.gates_shut(),
                    sim.motif(),
                    sim.hearts(),
                );
            }
            if sim.phase() != Phase::Playing {
                break;
            }
        }
    }

    #[test]
    fn reading_the_road_gets_you_further_than_weaving() {
        // The careful bot is no champion, so this does not ask it to get
        // home. It asks that care pays: over the same seeds it gets well
        // past where weaving east ever does, and never dies at the door.
        let seeds = [1_u64, 2, 3, 5, 8, 13, 21, 34];
        let mut careful_total = 0.0;
        let mut weaving_total = 0.0;
        let mut best = 0.0_f32;
        for seed in seeds {
            let mut sim = playing(seed);
            for _ in 0..(60 * 300) {
                let input = careful(&sim);
                sim.advance(TICK_DT, &input);
                if sim.phase() != Phase::Playing {
                    break;
                }
            }
            println!(
                "seed {seed}: careful {:?} at {:.0}% with {} hearts",
                sim.phase(),
                sim.progress() * 100.0,
                sim.hearts()
            );
            careful_total += sim.progress();
            best = best.max(sim.progress());
            assert!(
                sim.progress() > 0.03,
                "seed {seed}: careful died at the door"
            );

            let mut sim = playing(seed);
            for tick in 0..(60 * 300) {
                sim.advance(TICK_DT, &weaving(tick));
                if sim.phase() != Phase::Playing {
                    break;
                }
            }
            weaving_total += sim.progress();
        }
        let careful_mean = careful_total / seeds.len() as f32;
        let weaving_mean = weaving_total / seeds.len() as f32;
        println!(
            "mean progress: careful {careful_mean:.2}, weaving {weaving_mean:.2}, best careful {best:.2}"
        );
        assert!(
            careful_mean > weaving_mean * 1.3,
            "care did not pay: careful {careful_mean:.2} vs weaving {weaving_mean:.2}"
        );
        assert!(
            best >= 0.2,
            "no careful run got a fifth of the way ({best:.2})"
        );
    }
}
