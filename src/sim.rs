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
//!   stompers drift on the beat and turn on every snare, half the time
//!   toward the player. *Pursuit*: stompers lunge at the player on every
//!   kick and snare, harder, walls linger, and the player is a little
//!   quicker. *Lullaby*: stompers sleep where they stand — still solid,
//!   still sore to touch — walls stay up for most of a bar, and the player
//!   slows.
//! - Each **instrument** animates one thing, and only while it is sounding.
//!   Kicks move the stompers. Bass notes raise every wall in the lane of
//!   the note's pitch class — walls run north-south across the way home, so
//!   the bass line decides when the road is open. Lead notes drop sparks
//!   ahead of the player, placed by pitch and by position in the bar, so the
//!   melody lays a trail toward home. The pad pulls nearby sparks in.
//! - Sparks heal: a few of them restore a lost heart. Stompers hurt. Lose
//!   every heart and the music stops; reach home and it resolves.
//! - The player can **hush** an instrument by holding its key, which
//!   silences it — and so stops whatever it drives — for as long as a
//!   slowly refilling pool lasts. Hush the drums and the stompers freeze;
//!   hush the bass and the road opens; but nothing new is happening while
//!   you do, and the pool runs dry.
//!
//! The world is unbounded. Walls repeat forever on a grid, stompers respawn
//! around the player as it travels, and the camera is the frontend's
//! problem. Time is fixed-step with an accumulator, as in the template: the
//! frontend's only way in is an [`InputFrame`], and its only way out is the
//! getters plus [`Sim::alpha`] for interpolation.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};

use crate::song::{self, Chord, Event, Instrument, Motif, Position, Sequencer};

/// Length of one simulation step. Ticks are always exactly this long.
pub const TICK_DT: f32 = 1.0 / 60.0;

/// Longest frame the accumulator will bank, so a backgrounded tab does not
/// come back and try to catch up all at once.
const MAX_FRAME_DT: f32 = 0.25;

/// How far east home is from the start.
pub const HOME_DISTANCE: f32 = 14000.0;

/// Where home is.
pub const HOME: Vec2 = Vec2::new(HOME_DISTANCE, 0.0);

/// Being this close to home is being home.
pub const HOME_RADIUS: f32 = 70.0;

/// World-space radius of the player.
pub const PLAYER_RADIUS: f32 = 11.0;

/// World-space radius of a stomper.
pub const STOMPER_RADIUS: f32 = 15.0;

/// World-space radius of a spark.
pub const SPARK_RADIUS: f32 = 6.0;

/// How many stompers travel with the player.
pub const STOMPER_COUNT: usize = 7;

/// Hearts the player starts with, and the most it can hold.
pub const MAX_HEARTS: u32 = 3;

/// Sparks it takes to restore a heart.
pub const SPARKS_PER_HEART: u32 = 8;

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

/// How close a spark has to be for the pad to pull it in.
pub const MAGNET_RADIUS: f32 = 160.0;

/// How much of the hush pool one hushed instrument spends per second.
pub const HUSH_DRAIN: f32 = 0.45;

/// How much of the hush pool refills per second while nothing is hushed.
pub const HUSH_RECHARGE: f32 = 0.15;

/// Once the pool runs dry it must refill to this before it can be spent
/// again, so a held key pulses rather than flickers.
pub const HUSH_RELOCK: f32 = 0.5;

/// How hard the pad pulls.
const MAGNET_ACCEL: f32 = 900.0;

/// Sparks land this far ahead of the player, spread by their step.
const SPARK_AHEAD_MIN: f32 = 180.0;
const SPARK_AHEAD_SPAN: f32 = 420.0;

/// Sparks spread this far sideways, by their pitch.
const SPARK_LATERAL: f32 = 420.0;

/// Sparks live for this many bars.
const SPARK_LIFE_BARS: f32 = 2.0;

/// Sparks and stompers further than this from the player are recycled.
const FAR: f32 = 1100.0;

/// Stompers respawn this far from the player.
const RESPAWN_MIN: f32 = 520.0;
const RESPAWN_SPAN: f32 = 200.0;

/// Fraction of velocity a spark keeps per tick.
const SPARK_DAMPING: f32 = 0.92;

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

/// Stompers respawn ahead of the player, on the road home, this often.
const AMBUSH_CHANCE: f32 = 0.7;

/// How far off due east an ambush may spawn, in radians.
const AMBUSH_SPREAD: f32 = 1.0;

/// Chance a wandering stomper's snare turn faces the player.
const WANDER_NOTICE: f32 = 0.5;

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

/// An enemy that only moves when the drums tell it to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stomper {
    pub body: Body,
    /// Direction of the next lunge while wandering.
    pub heading: Vec2,
    /// `0..=1`, set on each step and fading, for the renderer.
    pub pulse: f32,
}

/// A pickup, spawned by a lead note.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spark {
    pub body: Body,
    /// The note that spawned it, for colouring.
    pub pitch: u8,
    /// Seconds left.
    pub life: f32,
    /// Seconds it started with.
    pub max_life: f32,
}

impl Spark {
    /// Remaining life as a fraction, for fading.
    #[must_use]
    pub fn fade(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
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
#[derive(Clone, Copy, Debug, Default)]
pub struct InputFrame {
    /// Movement, each axis in `-1..=1`.
    pub move_dir: Vec2,
    /// Any press: leaves the title screen.
    pub start: bool,
    /// Held to hush an instrument, indexed by [`Instrument::index`].
    pub hold_layer: [bool; 4],
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
    /// An instrument was hushed or let sound again.
    Layer { instrument: Instrument, on: bool },
    /// The player collected a spark. `healed` if it was the one that
    /// restored a heart.
    Pickup { healed: bool },
    /// A stomper caught the player.
    Hit { fatal: bool },
    /// The player reached home. Comes with a [`Cue::Chord`] to resolve on.
    Home,
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
    sparks: Vec<Spark>,
    /// The chord the pad last played, for the renderer's palette.
    chord: Option<Chord>,
    hearts: u32,
    /// Sparks collected toward the next heart.
    heal: u32,
    /// The hush pool, `0..=1`.
    hush: f32,
    /// The pool ran dry and has not yet refilled to [`HUSH_RELOCK`].
    hush_dry: bool,
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
            sparks: Vec::new(),
            chord: None,
            hearts: MAX_HEARTS,
            heal: 0,
            hush: 1.0,
            hush_dry: false,
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

    /// How far toward the next heart the collected sparks have got, `0..1`.
    #[must_use]
    pub fn heal_progress(&self) -> f32 {
        self.heal as f32 / SPARKS_PER_HEART as f32
    }

    /// How much hush is left to spend, `0..=1`.
    #[must_use]
    pub const fn hush(&self) -> f32 {
        self.hush
    }

    /// Whether the hush pool is refilling from empty and cannot be spent.
    #[must_use]
    pub const fn hush_dry(&self) -> bool {
        self.hush_dry
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

    /// Live sparks.
    #[must_use]
    pub fn sparks(&self) -> &[Spark] {
        &self.sparks
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

    /// Whether the pad is pulling sparks in right now.
    #[must_use]
    pub const fn magnet_on(&self) -> bool {
        self.sequencer.is_active(Instrument::Pad)
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
        // The music first, so this tick's notes act on this tick.
        let mut events = std::mem::take(&mut self.events);
        events.clear();
        self.apply_hush(input.hold_layer, &mut events);
        self.sequencer.advance(TICK_DT, &mut events);
        for event in &events {
            self.apply(*event);
        }
        self.events = events;

        self.move_player(input.move_dir);
        self.move_stompers();
        self.sink_walls();
        self.move_sparks();
        self.collect_sparks();
        self.take_hits();
        self.arrive();
    }

    /// Spend the hush pool on whatever the player is holding down, or let
    /// it refill. Only instruments the arrangement is actually playing cost
    /// anything; there is nothing to hush in a silent layer.
    fn apply_hush(&mut self, hold: [bool; 4], events: &mut Vec<Event>) {
        let usable = !self.hush_dry && self.hush > 0.0;
        let arranged = self.sequencer.section().arranged;
        let mut holding = 0_u32;
        for instrument in Instrument::ALL {
            let i = instrument.index();
            let want = hold[i] && usable && arranged[i];
            self.sequencer.set_muted(instrument, want, events);
            holding += u32::from(want);
        }
        if holding > 0 {
            self.hush -= HUSH_DRAIN * holding as f32 * TICK_DT;
            if self.hush <= 0.0 {
                self.hush = 0.0;
                self.hush_dry = true;
            }
        } else {
            self.hush = HUSH_RECHARGE.mul_add(TICK_DT, self.hush).min(1.0);
            if self.hush_dry && self.hush >= HUSH_RELOCK {
                self.hush_dry = false;
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
                    Instrument::Bass => self.walls[lane_of_pitch(pitch)] = 1.0,
                    Instrument::Lead => self.spawn_spark(pitch, step),
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
            Event::Section { motif } => self.cues.push(Cue::Section(motif)),
            Event::Layer { instrument, on } => self.cues.push(Cue::Layer { instrument, on }),
        }
    }

    /// Kicks move the stompers; snares re-aim them (and, in pursuit, drive
    /// them on); hats are just heard. Nothing moves during the lullaby,
    /// whatever the drums do.
    fn drum_hit(&mut self, pitch: u8) {
        let motif = self.motif();
        if motif == Motif::Lullaby {
            return;
        }
        let player_pos = self.player.body.pos;
        for stomper in &mut self.stompers {
            match (motif, pitch) {
                (Motif::Pursuit, song::KICK | song::SNARE) => {
                    let toward = (player_pos - stomper.body.pos).normalized();
                    stomper.heading = toward;
                    let strength = if pitch == song::KICK { 1.0 } else { 0.5 };
                    stomper.body.vel += toward * (lunge(motif) * strength);
                    stomper.pulse = strength;
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
                        Vec2::from_angle(self.rng.f32() * std::f32::consts::TAU)
                    };
                    stomper.pulse = 0.6;
                }
                _ => {}
            }
        }
    }

    /// Drop a spark ahead of the player: further ahead the later in the bar
    /// the note fell, further to the side the further its pitch is from
    /// the middle of the tune. A bar of melody lays a trail.
    fn spawn_spark(&mut self, pitch: u8, step: u32) {
        let motif = self.motif();
        let (lo, hi) = motif.lead_range();
        let span = f32::from(hi.saturating_sub(lo)).max(1.0);
        let ahead =
            (step as f32 / song::STEPS_PER_BAR as f32).mul_add(SPARK_AHEAD_SPAN, SPARK_AHEAD_MIN);
        // Higher notes land further north, which is up on screen.
        let lateral = (0.5 - f32::from(pitch.saturating_sub(lo)) / span) * SPARK_LATERAL;
        let life = motif.bar_secs() * SPARK_LIFE_BARS;
        self.sparks.push(Spark {
            body: Body::at(self.player.body.pos + Vec2::new(ahead, lateral)),
            pitch,
            life,
            max_life: life,
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
        push_out(&self.walls, body, PLAYER_RADIUS);
        self.player.invuln = (self.player.invuln - TICK_DT).max(0.0);
    }

    fn move_stompers(&mut self) {
        self.separate_stompers();
        let player_pos = self.player.body.pos;
        for i in 0..self.stompers.len() {
            let stomper = &mut self.stompers[i];
            stomper.body.vel *= STOMPER_DAMPING;
            stomper.body.step();
            push_out(&self.walls, &mut stomper.body, STOMPER_RADIUS);
            stomper.pulse = (stomper.pulse - TICK_DT / PULSE_FADE).max(0.0);
            // Left behind: come back somewhere new near the player.
            if (stomper.body.pos - player_pos).length() > FAR {
                self.stompers[i] = spawn_stomper(&mut self.rng, player_pos);
            }
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

    fn move_sparks(&mut self) {
        let magnet = self.magnet_on();
        let player_pos = self.player.body.pos;
        for spark in &mut self.sparks {
            spark.life -= TICK_DT;
            if magnet {
                let offset = player_pos - spark.body.pos;
                let dist = offset.length();
                if dist < MAGNET_RADIUS && dist > f32::EPSILON {
                    // Stronger the closer it gets, so the last stretch snaps.
                    let pull = MAGNET_ACCEL * (1.0 - dist / MAGNET_RADIUS).mul_add(0.5, 0.5);
                    spark.body.vel += offset * (pull * TICK_DT / dist);
                }
            }
            spark.body.vel *= SPARK_DAMPING;
            spark.body.step();
        }
        self.sparks
            .retain(|spark| spark.life > 0.0 && (spark.body.pos - player_pos).length() < FAR);
    }

    fn collect_sparks(&mut self) {
        let player_pos = self.player.body.pos;
        let reach = PLAYER_RADIUS + SPARK_RADIUS;
        let before = self.sparks.len();
        self.sparks
            .retain(|spark| (spark.body.pos - player_pos).length() > reach);
        for _ in self.sparks.len()..before {
            let mut healed = false;
            if self.hearts < MAX_HEARTS {
                self.heal += 1;
                if self.heal >= SPARKS_PER_HEART {
                    self.heal = 0;
                    self.hearts += 1;
                    healed = true;
                }
            }
            self.cues.push(Cue::Pickup { healed });
        }
    }

    /// Stompers hurt on contact, asleep or not, except during the grace
    /// period after a hit.
    fn take_hits(&mut self) {
        if self.player.invuln > 0.0 {
            return;
        }
        let reach = PLAYER_RADIUS + STOMPER_RADIUS;
        let player_pos = self.player.body.pos;
        let Some(stomper) = self
            .stompers
            .iter()
            .find(|s| (s.body.pos - player_pos).length() < reach)
        else {
            return;
        };
        let away = (player_pos - stomper.body.pos).normalized();
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
        self.cues.push(Cue::Home);
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
        Motif::Wander => 300.0,
        Motif::Pursuit => 460.0,
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
            rng.f32() * std::f32::consts::TAU
        };
        let radius = rng.f32().mul_add(RESPAWN_SPAN, RESPAWN_MIN);
        pos = near + Vec2::from_angle(angle) * radius;
        if !inside_wall(pos, STOMPER_RADIUS) {
            break;
        }
    }
    Stomper {
        body: Body::at(pos),
        heading: Vec2::from_angle(rng.f32() * std::f32::consts::TAU),
        pulse: 0.0,
    }
}

/// Whether a circle overlaps where a wall stands, up or not.
fn inside_wall(pos: Vec2, radius: f32) -> bool {
    let k = (pos.x / WALL_SPACING).round() as i32;
    let in_band = wall_stands_at(k, pos.y - radius) || wall_stands_at(k, pos.y + radius);
    in_band && (pos.x - wall_x(k)).abs() < WALL_HALF_W + radius
}

/// Keep a circle out of the nearest solid wall by shoving it sideways.
/// Walls are thin and tall, so sideways is always the short way out.
fn push_out(walls: &[f32; WALL_LANES], body: &mut Body, radius: f32) {
    // Walls are far enough apart that only the nearest can touch a body.
    let k = (body.pos.x / WALL_SPACING).round() as i32;
    if !wall_is_solid(walls[lane_of_wall(k)]) {
        return;
    }
    // Rounded to the body's reach, so it cannot clip a wall's end.
    let in_wall = wall_stands_at(k, body.pos.y - radius) || wall_stands_at(k, body.pos.y + radius);
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
    use crate::song::{BARS_PER_SECTION, SONG};

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

    fn holding(instrument: Instrument) -> InputFrame {
        let mut input = InputFrame::default();
        input.hold_layer[instrument.index()] = true;
        input
    }

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
            .chain(sim.sparks().iter().map(|s| s.body.pos))
            .collect()
    }

    /// Where wall 1 stands, and where one of its gaps is. Wall 1 is odd,
    /// so its gaps sit half a period off the even walls'.
    const WALLED_Y: f32 = 0.0;
    const GAP_Y: f32 = WALL_PERIOD_Y * 0.5;

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
    fn lead_notes_lay_a_trail_ahead_and_muting_stops_them() {
        let mut sim = playing(3);
        run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        assert!(
            !sim.sparks().is_empty(),
            "two seconds of lead spawned nothing"
        );
        let (lo, hi) = sim.motif().lead_range();
        for spark in sim.sparks() {
            assert!((lo..=hi).contains(&spark.pitch));
            let offset = spark.body.pos - sim.player().body.pos;
            assert!(
                offset.x >= SPARK_AHEAD_MIN - 1.0,
                "spark behind: {offset:?}"
            );
            assert!(offset.x <= SPARK_AHEAD_MIN + SPARK_AHEAD_SPAN + 1.0);
            assert!(offset.y.abs() <= SPARK_LATERAL.mul_add(0.5, 1.0));
        }
        // Higher notes land further north.
        let top = sim
            .sparks()
            .iter()
            .min_by(|a, b| a.body.pos.y.total_cmp(&b.body.pos.y))
            .unwrap();
        let bottom = sim
            .sparks()
            .iter()
            .max_by(|a, b| a.body.pos.y.total_cmp(&b.body.pos.y))
            .unwrap();
        assert!(top.pitch >= bottom.pitch);

        // Hush the lead for most of the pool, and no new sparks appear:
        // every survivor is older than the hush.
        let secs = 0.8 / HUSH_DRAIN;
        run(&mut sim, secs, &holding(Instrument::Lead), |_| true);
        for spark in sim.sparks() {
            let age = spark.max_life - spark.life;
            assert!(
                age >= secs - 0.05,
                "a spark spawned {age}s ago, during the hush"
            );
        }
    }

    #[test]
    fn bass_notes_raise_walls_which_then_sink() {
        let mut sim = playing(5);
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        assert!(
            sim.walls().iter().any(|&w| wall_is_solid(w)),
            "no wall rose to the bass"
        );

        // Hush the bass for a couple of beats and every wall sinks.
        let secs = sim.motif().beat_secs() * (sink_beats(sim.motif()) + 0.5);
        assert!(secs < 1.0 / HUSH_DRAIN, "pool would run dry mid-test");
        run(&mut sim, secs, &holding(Instrument::Bass), |_| true);
        assert!(
            sim.walls().iter().all(|&w| w <= 0.0),
            "walls stayed up without bass: {:?}",
            sim.walls()
        );
    }

    /// Drive east from just west of wall 1 for a second, holding every
    /// lane at `solidity`, and return where the player ended up.
    fn drive_at_wall(y: f32, solidity: f32) -> f32 {
        let mut sim = playing(5);
        sim.player.body = Body::at(Vec2::new(wall_x(1) - 100.0, y));
        for _ in 0..60 {
            sim.walls = [solidity; WALL_LANES];
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

    /// A bot that reads the road: heads for the next wall's nearest gap,
    /// sidesteps close stompers, and hushes the drums when they are near.
    fn careful(sim: &Sim) -> InputFrame {
        let me = sim.player().body.pos;
        let k = (me.x / WALL_SPACING).floor() as i32 + 1;
        // Gap centres of the next wall sit at offset + m * period.
        let offset = wall_gap_offset(k);
        let m = ((me.y - offset) / WALL_PERIOD_Y).round();
        let gap_y = m.mul_add(WALL_PERIOD_Y, offset);
        let mut dir = if me.x > HOME.x - 700.0 {
            // Past the last wall that matters: go straight for the door.
            (HOME - me).normalized()
        } else {
            Vec2::new(1.0, (gap_y - me.y).clamp(-60.0, 60.0) / 60.0)
        };
        let nearest = sim
            .stompers()
            .iter()
            .map(|s| s.body.pos - me)
            .min_by(|a, b| a.length().total_cmp(&b.length()))
            .unwrap();
        let threat = nearest.length();
        if threat < 140.0 {
            let away = -nearest.normalized();
            dir += away * 1.5;
        }
        let mut input = InputFrame {
            move_dir: dir.normalized(),
            ..InputFrame::default()
        };
        input.hold_layer[Instrument::Drums.index()] = threat < 260.0 && !sim.hush_dry();
        input
    }

    #[test]
    fn a_careful_player_can_still_get_home() {
        let mut wins = 0;
        for seed in [1_u64, 2, 3, 5, 8, 13, 21, 34] {
            let mut sim = playing(seed);
            for _ in 0..(60 * 240) {
                let input = careful(&sim);
                sim.advance(TICK_DT, &input);
                if sim.phase() != Phase::Playing {
                    break;
                }
            }
            println!(
                "seed {seed}: {:?} at {:.0}% with {} hearts",
                sim.phase(),
                sim.progress() * 100.0,
                sim.hearts()
            );
            wins += u32::from(sim.phase() == Phase::Won);
        }
        assert!(wins >= 3, "only {wins} of 8 careful runs got home");
    }

    #[test]
    fn holding_right_alone_does_not_get_you_home() {
        // The road has to be read, not run. A player who only ever holds
        // east loses their hearts before home on every seed tried.
        for seed in [1_u64, 2, 3, 5, 8] {
            let mut sim = playing(seed);
            for _ in 0..(60 * 180) {
                sim.advance(TICK_DT, &moving(1.0, 0.0));
                if sim.phase() != Phase::Playing {
                    break;
                }
            }
            assert_eq!(
                sim.phase(),
                Phase::Over,
                "seed {seed}: holding right got {:.0}% of the way",
                sim.progress() * 100.0
            );
        }
    }

    #[test]
    fn stompers_only_move_to_drums() {
        // The intro has no drums arranged, so nothing should move.
        let mut sim = playing(11);
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
    fn stompers_sleep_through_the_lullaby() {
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
        // Asleep is not gone: walking into one still hurts.
        sim.player.body = Body::at(sim.stompers()[0].body.pos);
        sim.player.invuln = 0.0;
        let hits = run(&mut sim, 0.5, &InputFrame::default(), |c| {
            matches!(c, Cue::Hit { .. })
        });
        assert_eq!(hits, 1);
        assert_eq!(sim.hearts(), MAX_HEARTS - 1);
    }

    #[test]
    fn stompers_chase_during_pursuit() {
        let mut sim = playing(11);
        run_to_section(&mut sim, section_of(Motif::Pursuit));
        let here = sim.player().body.pos;
        let mean_dist = |sim: &Sim| {
            sim.stompers()
                .iter()
                .map(|s| (s.body.pos - here).length())
                .sum::<f32>()
                / STOMPER_COUNT as f32
        };
        let before = mean_dist(&sim);
        for _ in 0..(60 * 3) {
            sim.player.body = Body::at(here);
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
        }
        assert!(
            mean_dist(&sim) < before,
            "stompers did not close in: {before} -> {}",
            mean_dist(&sim)
        );
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
            sim.player.body = Body::at(here);
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
            sim.player.body = Body::at(sim.stompers()[0].body.pos);
            sim.advance(TICK_DT, &InputFrame::default());
            hits += sim
                .cues()
                .iter()
                .filter(|c| matches!(c, Cue::Hit { .. }))
                .count();
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
            sim.player.body = Body::at(sim.stompers()[0].body.pos);
            sim.advance(TICK_DT, &InputFrame::default());
            if sim.cues().iter().any(|c| matches!(c, Cue::Hit { .. })) {
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

    /// Put a spark on the player and tick once, returning the pickup cue.
    fn eat_a_spark(sim: &mut Sim) -> Cue {
        sim.sparks.push(Spark {
            body: Body::at(sim.player().body.pos),
            pitch: 60,
            life: 10.0,
            max_life: 10.0,
        });
        sim.advance(TICK_DT, &InputFrame::default());
        *sim.cues()
            .iter()
            .find(|c| matches!(c, Cue::Pickup { .. }))
            .expect("a spark on the player is collected")
    }

    #[test]
    fn sparks_heal_a_heart_at_a_time() {
        let mut sim = playing(3);
        sim.hearts = 1;
        for n in 1..SPARKS_PER_HEART {
            assert_eq!(eat_a_spark(&mut sim), Cue::Pickup { healed: false });
            assert!((sim.heal_progress() - n as f32 / SPARKS_PER_HEART as f32).abs() < 1e-5);
            assert_eq!(sim.hearts(), 1);
        }
        assert_eq!(eat_a_spark(&mut sim), Cue::Pickup { healed: true });
        assert_eq!(sim.hearts(), 2);
        assert!(sim.heal_progress().abs() < 1e-5);
    }

    #[test]
    fn full_hearts_do_not_bank_sparks() {
        let mut sim = playing(3);
        for _ in 0..(SPARKS_PER_HEART * 2) {
            assert_eq!(eat_a_spark(&mut sim), Cue::Pickup { healed: false });
        }
        assert_eq!(sim.hearts(), MAX_HEARTS);
        assert!(sim.heal_progress().abs() < 1e-5);
    }

    #[test]
    fn holding_hushes_at_once_and_spends_the_pool() {
        let mut sim = playing(3);
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        assert!((sim.hush() - 1.0).abs() < 1e-6);
        let lead_notes = |c: &Cue| {
            matches!(
                c,
                Cue::Note {
                    instrument: Instrument::Lead,
                    ..
                }
            )
        };
        // The very first hushed tick announces it, and no lead note follows.
        sim.advance(TICK_DT, &holding(Instrument::Lead));
        assert!(sim.cues().contains(&Cue::Layer {
            instrument: Instrument::Lead,
            on: false
        }));
        let notes = run(&mut sim, 1.0, &holding(Instrument::Lead), lead_notes);
        assert_eq!(notes, 0);
        let spent = 1.0 - sim.hush();
        assert!(
            (spent - HUSH_DRAIN).abs() < 0.02,
            "a second of hushing spent {spent}"
        );
        // Letting go announces that too, and the pool refills.
        sim.advance(TICK_DT, &InputFrame::default());
        assert!(sim.cues().contains(&Cue::Layer {
            instrument: Instrument::Lead,
            on: true
        }));
        let before = sim.hush();
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        assert!((sim.hush() - before - HUSH_RECHARGE).abs() < 0.02);
    }

    #[test]
    fn a_dry_pool_lets_go_and_relocks_until_refilled() {
        let mut sim = playing(3);
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        let lead_notes = |c: &Cue| {
            matches!(
                c,
                Cue::Note {
                    instrument: Instrument::Lead,
                    ..
                }
            )
        };
        // Hold well past the pool's worth: the lead comes back on its own.
        run(
            &mut sim,
            1.0 / HUSH_DRAIN + 0.2,
            &holding(Instrument::Lead),
            |_| true,
        );
        assert!(sim.hush_dry());
        let notes = run(&mut sim, 1.0, &holding(Instrument::Lead), lead_notes);
        assert!(notes > 0, "lead stayed hushed on an empty pool");
        // Keep holding: it stays audible until the pool refills to the
        // relock mark, then hushes again.
        // (The second of listening above already refilled some of it.)
        let to_relock = HUSH_RELOCK / HUSH_RECHARGE - 1.0 + 0.1;
        run(&mut sim, to_relock, &holding(Instrument::Lead), |_| true);
        assert!(!sim.hush_dry());
        let notes = run(&mut sim, 0.5, &holding(Instrument::Lead), lead_notes);
        assert_eq!(notes, 0, "lead did not hush again once the pool refilled");
    }

    #[test]
    fn hushing_a_silent_layer_costs_nothing() {
        // The intro arranges no drums, so there is nothing to spend on.
        let mut sim = playing(3);
        run(&mut sim, 1.0, &holding(Instrument::Drums), |_| true);
        assert!((sim.hush() - 1.0).abs() < 1e-6);
        assert!(!sim.music().layers[Instrument::Drums.index()].muted);
    }

    #[test]
    fn holding_two_spends_twice_as_fast() {
        let mut sim = playing(3);
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        let mut both = InputFrame::default();
        both.hold_layer[Instrument::Bass.index()] = true;
        both.hold_layer[Instrument::Lead.index()] = true;
        run(&mut sim, 1.0, &both, |_| true);
        let spent = 1.0 - sim.hush();
        assert!(
            2.0_f32.mul_add(-HUSH_DRAIN, spent).abs() < 0.03,
            "spent {spent}"
        );
    }

    #[test]
    fn pad_pulls_sparks_in() {
        let mut sim = playing(3);
        run_to_section(&mut sim, 1);
        assert!(sim.magnet_on());
        let pos = sim.player().body.pos + Vec2::new(MAGNET_RADIUS * 0.6, 0.0);
        sim.sparks.push(Spark {
            body: Body::at(pos),
            pitch: 60,
            life: 10.0,
            max_life: 10.0,
        });
        let before = (pos - sim.player().body.pos).length();
        sim.advance(TICK_DT * 10.0, &InputFrame::default());
        let spark = sim.sparks().last().unwrap();
        assert!((spark.body.pos - sim.player().body.pos).length() < before);
    }

    #[test]
    fn reaching_home_wins_and_resolves() {
        let mut sim = playing(5);
        sim.player.body = Body::at(HOME - Vec2::new(HOME_RADIUS + 150.0, 0.0));
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
        let at = cues.iter().position(|c| *c == Cue::Home).expect("home cue");
        // Resolves on the motif's home chord, cued just before.
        assert_eq!(cues[at - 1], Cue::Chord(sim.motif().tonic()));
        assert_eq!(sim.chord(), Some(sim.motif().tonic()));

        // Won means done: no more ticks, no more music.
        let more = run(&mut sim, 2.0, &moving(1.0, 0.0), |_| true);
        assert_eq!(more, 0);
    }

    #[test]
    fn progress_tracks_the_way_home() {
        let mut sim = playing(5);
        assert!(sim.progress().abs() < 1e-6);
        sim.player.body = Body::at(Vec2::new(HOME_DISTANCE * 0.5, 300.0));
        sim.advance(0.0, &InputFrame::default());
        assert!((sim.progress() - 0.5).abs() < 1e-3);
        sim.player.body = Body::at(Vec2::new(-500.0, 0.0));
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
            // Wander about, so sparks and walls get exercised.
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
}
