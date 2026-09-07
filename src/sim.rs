//! The arena: pure, deterministic, and macroquad-free.
//!
//! This is a small survival game whose rules are set by the music. The
//! [`song::Sequencer`] walks the score and hands back [`song::Event`]s; this
//! module turns them into behaviour and, unchanged, into [`Cue`]s for the
//! frontend to play. The two consumers see the same feed, which is what
//! makes the premise honest: what you hear is what is happening.
//!
//! What each part of the music does:
//!
//! - The **motif** (which theme is playing) sets the rules. *Wander*:
//!   stompers drift on the beat and re-aim on every snare. *Pursuit*:
//!   stompers lunge at the player on every kick, faster, and sparks are
//!   worth double. *Lullaby*: stompers sleep and are harmless, the player
//!   slows, sparks are rare but worth triple.
//! - Each **instrument** animates one thing, and only while it is sounding.
//!   Kicks move the stompers. Bass notes raise a wall in the lane of the
//!   note's pitch class. Lead notes spawn sparks, placed by pitch and by
//!   position in the bar, so the melody draws them. The pad pulls nearby
//!   sparks toward the player.
//! - A spark is worth the number of instruments sounding, times the motif's
//!   multiplier: muting layers makes the arena safer and poorer.
//!
//! Time is fixed-step with an accumulator, as in the template: the frontend's
//! only way in is an [`InputFrame`], and its only way out is the getters plus
//! [`Sim::alpha`] for interpolation.

use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};

use crate::song::{self, Chord, Event, Instrument, Motif, Position, Sequencer};

/// Length of one simulation step. Ticks are always exactly this long.
pub const TICK_DT: f32 = 1.0 / 60.0;

/// Logical world width. The renderer scales this onto the window, so the sim
/// never learns what a pixel is.
pub const WORLD_W: f32 = 800.0;

/// Logical world height.
pub const WORLD_H: f32 = 600.0;

/// Longest frame the accumulator will bank, so a backgrounded tab does not
/// come back and try to catch up all at once.
const MAX_FRAME_DT: f32 = 0.25;

/// World-space radius of the player.
pub const PLAYER_RADIUS: f32 = 11.0;

/// World-space radius of a stomper.
pub const STOMPER_RADIUS: f32 = 15.0;

/// World-space radius of a spark.
pub const SPARK_RADIUS: f32 = 6.0;

/// How many stompers share the arena.
pub const STOMPER_COUNT: usize = 5;

/// Hits the player can take.
pub const LIVES: u32 = 3;

/// Walls, one per lane. Bass pitch classes map onto lanes.
pub const WALL_LANES: usize = 4;

/// Half the thickness of a wall.
pub const WALL_HALF_W: f32 = 14.0;

/// Walls span this band of the arena, leaving the top and bottom open so
/// nothing can be boxed in outright.
pub const WALL_TOP: f32 = 110.0;
/// See [`WALL_TOP`].
pub const WALL_BOTTOM: f32 = WORLD_H - WALL_TOP;

/// Sparks spawn inside this margin.
const SPARK_MARGIN_X: f32 = 60.0;
const SPARK_MARGIN_Y: f32 = 80.0;

/// The area sparks spawn across, inside the margins.
const SPARK_SPAN_X: f32 = WORLD_W - 2.0 * SPARK_MARGIN_X;
const SPARK_SPAN_Y: f32 = WORLD_H - 2.0 * SPARK_MARGIN_Y;

/// Sparks live for this many bars.
const SPARK_LIFE_BARS: f32 = 2.0;

/// How close a spark has to be for the pad to pull it in.
pub const MAGNET_RADIUS: f32 = 160.0;

/// How hard the pad pulls.
const MAGNET_ACCEL: f32 = 900.0;

/// Fraction of velocity a spark keeps per tick.
const SPARK_DAMPING: f32 = 0.92;

/// Player top speed, before the motif's scaling.
const PLAYER_SPEED: f32 = 250.0;

/// Fraction of the gap to the target velocity closed per tick.
const PLAYER_ACCEL: f32 = 0.25;

/// Seconds of invulnerability after a hit.
const INVULN_SECS: f32 = 1.5;

/// Knockback speed on a hit.
const KNOCKBACK: f32 = 420.0;

/// Fraction of velocity a stomper keeps per tick.
const STOMPER_DAMPING: f32 = 0.93;

/// Fraction of speed kept when anything bounces off the arena edge.
const RESTITUTION: f32 = 0.5;

/// Beats a wall takes to sink after its last note.
const WALL_SINK_BEATS: f32 = 1.5;

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

    /// Integrate one tick and keep a `radius` circle inside the arena.
    fn step(&mut self, radius: f32) {
        self.prev_pos = self.pos;
        self.pos += self.vel * TICK_DT;
        bounce(&mut self.pos.x, &mut self.vel.x, radius, WORLD_W - radius);
        bounce(&mut self.pos.y, &mut self.vel.y, radius, WORLD_H - radius);
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

/// One lane's wall. Raised by bass notes, sinking between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {
    /// Centre line.
    pub x: f32,
    /// `0..=1`, how far up it is.
    pub solidity: f32,
}

impl Wall {
    /// Whether the wall currently blocks movement.
    #[must_use]
    pub fn is_solid(&self) -> bool {
        self.solidity > WALL_SOLID
    }
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
    /// Out of lives. The music has stopped.
    Over,
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
    /// Edge-triggered mute toggles, indexed by [`Instrument::index`].
    pub toggle_layer: [bool; 4],
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
    /// A mute or unmute took effect.
    Layer { instrument: Instrument, on: bool },
    /// The player collected a spark worth this much.
    Pickup { value: u32 },
    /// A stomper caught the player.
    Hit { fatal: bool },
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
    walls: [Wall; WALL_LANES],
    sparks: Vec<Spark>,
    /// The chord the pad last played, for the renderer's palette.
    chord: Option<Chord>,
    score: u32,
    lives: u32,
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
            .map(|_| spawn_stomper(&mut rng))
            .collect();
        let walls = std::array::from_fn(|lane| Wall {
            x: WORLD_W * (lane as f32 + 1.0) / (WALL_LANES as f32 + 1.0),
            solidity: 0.0,
        });
        Self {
            seed,
            rng,
            sequencer: Sequencer::new(),
            phase: Phase::Title,
            paused: false,
            player: Player {
                body: Body::at(Vec2::new(WORLD_W * 0.5, WORLD_H * 0.5)),
                invuln: 0.0,
            },
            stompers,
            walls,
            sparks: Vec::new(),
            chord: None,
            score: 0,
            lives: LIVES,
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

    /// Points so far.
    #[must_use]
    pub const fn score(&self) -> u32 {
        self.score
    }

    /// Hits left to take.
    #[must_use]
    pub const fn lives(&self) -> u32 {
        self.lives
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

    /// The walls, one per lane.
    #[must_use]
    pub const fn walls(&self) -> &[Wall; WALL_LANES] {
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

    /// What a spark is worth right now: instruments sounding, times the
    /// motif's multiplier. Silence still pays one.
    #[must_use]
    pub fn spark_value(&self) -> u32 {
        self.sequencer.active_count().max(1) * multiplier(self.motif())
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

        for instrument in Instrument::ALL {
            if input.toggle_layer[instrument.index()] {
                self.sequencer.request_toggle(instrument);
            }
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
                    Instrument::Bass => self.walls[lane_of(pitch)].solidity = 1.0,
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

    /// Kicks move the stompers; snares re-aim them; hats are just heard.
    /// Nothing moves during the lullaby, whatever the drums do.
    fn drum_hit(&mut self, pitch: u8) {
        let motif = self.motif();
        if motif == Motif::Lullaby {
            return;
        }
        let player_pos = self.player.body.pos;
        for stomper in &mut self.stompers {
            match (motif, pitch) {
                (Motif::Pursuit, song::KICK) => {
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
                    stomper.heading = random_heading(&mut self.rng);
                    stomper.pulse = 0.6;
                }
                _ => {}
            }
        }
    }

    /// Place a spark by pitch (left to right) and by where in the bar the
    /// note fell (top to bottom), so a bar of melody draws a shape.
    fn spawn_spark(&mut self, pitch: u8, step: u32) {
        let motif = self.motif();
        let (lo, hi) = motif.lead_range();
        let span = f32::from(hi.saturating_sub(lo)).max(1.0);
        let x = (f32::from(pitch.saturating_sub(lo)) / span).mul_add(SPARK_SPAN_X, SPARK_MARGIN_X);
        let y = (step as f32 / song::STEPS_PER_BAR as f32).mul_add(SPARK_SPAN_Y, SPARK_MARGIN_Y);
        let life = motif.bar_secs() * SPARK_LIFE_BARS;
        self.sparks.push(Spark {
            body: Body::at(Vec2::new(x, y)),
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
        body.step(PLAYER_RADIUS);
        for wall in self.walls {
            push_out(wall, body, PLAYER_RADIUS);
        }
        self.player.invuln = (self.player.invuln - TICK_DT).max(0.0);
    }

    fn move_stompers(&mut self) {
        self.separate_stompers();
        for stomper in &mut self.stompers {
            stomper.body.vel *= STOMPER_DAMPING;
            stomper.body.step(STOMPER_RADIUS);
            for wall in self.walls {
                push_out(wall, &mut stomper.body, STOMPER_RADIUS);
            }
            stomper.pulse = (stomper.pulse - TICK_DT / PULSE_FADE).max(0.0);
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
                self.stompers[i].body.vel += shove * -1.0;
                self.stompers[j].body.vel += shove;
            }
        }
    }

    fn sink_walls(&mut self) {
        let rate = 1.0 / (WALL_SINK_BEATS * self.motif().beat_secs());
        for wall in &mut self.walls {
            wall.solidity = rate.mul_add(-TICK_DT, wall.solidity).max(0.0);
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
            spark.body.step(SPARK_RADIUS);
        }
        self.sparks.retain(|spark| spark.life > 0.0);
    }

    fn collect_sparks(&mut self) {
        let value = self.spark_value();
        let player_pos = self.player.body.pos;
        let reach = PLAYER_RADIUS + SPARK_RADIUS;
        let before = self.sparks.len();
        self.sparks
            .retain(|spark| (spark.body.pos - player_pos).length() > reach);
        for _ in self.sparks.len()..before {
            self.score += value;
            self.cues.push(Cue::Pickup { value });
        }
    }

    /// Stompers hurt on contact, except while the lullaby has them asleep
    /// and during the grace period after a hit.
    fn take_hits(&mut self) {
        if self.motif() == Motif::Lullaby || self.player.invuln > 0.0 {
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
        self.lives -= 1;
        let fatal = self.lives == 0;
        if fatal {
            self.phase = Phase::Over;
        }
        self.cues.push(Cue::Hit { fatal });
    }
}

/// Which wall a bass note raises: the twelve pitch classes split across the
/// four lanes, so a bass line walks the walls up and down the arena.
fn lane_of(pitch: u8) -> usize {
    usize::from(pitch % 12) * WALL_LANES / 12
}

/// Score multiplier per motif.
const fn multiplier(motif: Motif) -> u32 {
    match motif {
        Motif::Wander => 1,
        Motif::Pursuit => 2,
        Motif::Lullaby => 3,
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
        Motif::Wander => 260.0,
        Motif::Pursuit => 380.0,
        Motif::Lullaby => 0.0,
    }
}

fn random_heading(rng: &mut fastrand::Rng) -> Vec2 {
    let angle = rng.f32() * std::f32::consts::TAU;
    Vec2::new(angle.cos(), angle.sin())
}

/// Stompers start around the edges, away from the player in the middle.
fn spawn_stomper(rng: &mut fastrand::Rng) -> Stomper {
    let angle = rng.f32() * std::f32::consts::TAU;
    let radius = rng.f32().mul_add(60.0, 200.0);
    let pos = Vec2::new(
        angle.cos().mul_add(radius, WORLD_W * 0.5),
        angle.sin().mul_add(radius, WORLD_H * 0.5),
    );
    Stomper {
        body: Body::at(pos),
        heading: random_heading(rng),
        pulse: 0.0,
    }
}

/// Clamp one axis into `lo..=hi`, reflecting velocity on contact.
fn bounce(pos: &mut f32, vel: &mut f32, lo: f32, hi: f32) {
    if *pos < lo {
        *pos = lo;
    } else if *pos > hi {
        *pos = hi;
    } else {
        return;
    }
    *vel = -*vel * RESTITUTION;
}

/// Keep a circle out of a solid wall by shoving it sideways. Walls are thin
/// and tall, so sideways is always the short way out.
fn push_out(wall: Wall, body: &mut Body, radius: f32) {
    if !wall.is_solid() {
        return;
    }
    let within_band = body.pos.y + radius > WALL_TOP && body.pos.y - radius < WALL_BOTTOM;
    if !within_band {
        return;
    }
    let overlap = WALL_HALF_W + radius - (body.pos.x - wall.x).abs();
    if overlap <= 0.0 {
        return;
    }
    if body.pos.x < wall.x {
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

    fn snapshot(sim: &Sim) -> Vec<Vec2> {
        std::iter::once(sim.player().body.pos)
            .chain(sim.stompers().iter().map(|s| s.body.pos))
            .chain(sim.sparks().iter().map(|s| s.body.pos))
            .collect()
    }

    #[test]
    fn title_screen_is_silent_and_still() {
        let mut sim = Sim::new(1);
        let busy = InputFrame {
            move_dir: Vec2::new(1.0, 0.0),
            ..InputFrame::default()
        };
        let cues = run(&mut sim, 2.0, &busy, |_| true);
        assert_eq!(cues, 0);
        assert_eq!(sim.phase(), Phase::Title);
        assert_eq!(
            sim.player().body.pos,
            Vec2::new(WORLD_W * 0.5, WORLD_H * 0.5)
        );
    }

    #[test]
    fn same_seed_and_inputs_are_bit_identical() {
        let input = InputFrame {
            move_dir: Vec2::new(0.7, -0.3),
            ..InputFrame::default()
        };
        let mut a = playing(0xDEAD_BEEF);
        let mut b = playing(0xDEAD_BEEF);
        for _ in 0..600 {
            a.advance(TICK_DT, &input);
            b.advance(TICK_DT, &input);
        }
        assert_eq!(snapshot(&a), snapshot(&b));
        assert_eq!(a.score(), b.score());
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(snapshot(&Sim::new(1)), snapshot(&Sim::new(2)));
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
        let cues = run(&mut sim, 3.0, &InputFrame::default(), |_| true);
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
    fn lead_notes_spawn_sparks_and_muting_stops_them() {
        let mut sim = playing(3);
        run(&mut sim, 2.0, &InputFrame::default(), |_| true);
        assert!(
            !sim.sparks().is_empty(),
            "two seconds of lead spawned nothing"
        );

        let mut toggle = InputFrame::default();
        toggle.toggle_layer[Instrument::Lead.index()] = true;
        sim.advance(TICK_DT, &toggle);
        // Wait out the bar, then two more bars of lifetime.
        let bar = sim.motif().bar_secs();
        run(
            &mut sim,
            bar * (1.0 + SPARK_LIFE_BARS) + 0.1,
            &InputFrame::default(),
            |_| true,
        );
        assert!(
            sim.sparks().is_empty(),
            "{} sparks survived a muted lead",
            sim.sparks().len()
        );
    }

    #[test]
    fn sparks_land_by_pitch_and_step() {
        let mut sim = playing(3);
        run(&mut sim, 3.0, &InputFrame::default(), |_| true);
        let (lo, hi) = sim.motif().lead_range();
        for spark in sim.sparks() {
            assert!((lo..=hi).contains(&spark.pitch));
            let pos = spark.body.pos;
            assert!(pos.x >= SPARK_MARGIN_X - 1.0 && pos.x <= WORLD_W - SPARK_MARGIN_X + 1.0);
            assert!(pos.y >= SPARK_MARGIN_Y - 1.0 && pos.y <= WORLD_H - SPARK_MARGIN_Y + 1.0);
        }
        // The lowest note sits furthest left, the highest furthest right.
        let leftmost = sim
            .sparks()
            .iter()
            .min_by(|a, b| a.body.pos.x.total_cmp(&b.body.pos.x))
            .unwrap();
        let rightmost = sim
            .sparks()
            .iter()
            .max_by(|a, b| a.body.pos.x.total_cmp(&b.body.pos.x))
            .unwrap();
        assert!(leftmost.pitch <= rightmost.pitch);
    }

    #[test]
    fn bass_notes_raise_walls_which_then_sink() {
        let mut sim = playing(5);
        run(&mut sim, 0.5, &InputFrame::default(), |_| true);
        assert!(
            sim.walls().iter().any(Wall::is_solid),
            "no wall rose to the bass"
        );

        let mut toggle = InputFrame::default();
        toggle.toggle_layer[Instrument::Bass.index()] = true;
        sim.advance(TICK_DT, &toggle);
        let bar = sim.motif().bar_secs();
        run(&mut sim, bar * 2.0, &InputFrame::default(), |_| true);
        assert!(
            sim.walls().iter().all(|w| w.solidity <= 0.0),
            "walls stayed up without bass: {:?}",
            sim.walls()
        );
    }

    #[test]
    fn solid_walls_block_the_player() {
        let mut sim = playing(5);
        // Drive at the first lane's wall from the left, mid-height.
        sim.player.body = Body::at(Vec2::new(sim.walls[0].x - 100.0, WORLD_H * 0.5));
        sim.walls[0].solidity = 1.0;
        let right = InputFrame {
            move_dir: Vec2::new(1.0, 0.0),
            ..InputFrame::default()
        };
        // Bass keeps the wall up regardless; hold solidity to be sure.
        for _ in 0..60 {
            sim.walls[0].solidity = 1.0;
            sim.advance(TICK_DT, &right);
        }
        let wall_x = sim.walls()[0].x;
        assert!(
            sim.player().body.pos.x <= wall_x - WALL_HALF_W - PLAYER_RADIUS + 0.01,
            "player at {} passed a wall at {wall_x}",
            sim.player().body.pos.x
        );
    }

    #[test]
    fn sunk_walls_do_not_block() {
        let mut sim = playing(5);
        sim.player.body = Body::at(Vec2::new(sim.walls[0].x - 60.0, WORLD_H * 0.5));
        let right = InputFrame {
            move_dir: Vec2::new(1.0, 0.0),
            ..InputFrame::default()
        };
        for _ in 0..60 {
            for wall in &mut sim.walls {
                wall.solidity = 0.0;
            }
            sim.advance(TICK_DT, &right);
        }
        assert!(sim.player().body.pos.x > sim.walls()[0].x);
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
        let lullaby = SONG.iter().position(|s| s.motif == Motif::Lullaby).unwrap();
        run_to_section(&mut sim, lullaby);
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
        // And they are harmless: park the player on one.
        sim.player.body = Body::at(sim.stompers()[0].body.pos);
        let hits = run(&mut sim, 0.5, &InputFrame::default(), |c| {
            matches!(c, Cue::Hit { .. })
        });
        assert_eq!(hits, 0);
        assert_eq!(sim.lives(), LIVES);
    }

    #[test]
    fn stompers_chase_during_pursuit() {
        let mut sim = playing(11);
        let pursuit = SONG.iter().position(|s| s.motif == Motif::Pursuit).unwrap();
        run_to_section(&mut sim, pursuit);
        // Park the player in a corner with infinite lives, and watch them come.
        let corner = Vec2::new(60.0, 60.0);
        sim.player.body = Body::at(corner);
        let mean_dist = |sim: &Sim| {
            sim.stompers()
                .iter()
                .map(|s| (s.body.pos - corner).length())
                .sum::<f32>()
                / STOMPER_COUNT as f32
        };
        let before = mean_dist(&sim);
        for _ in 0..(60 * 3) {
            sim.player.body = Body::at(corner);
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
        let pursuit = SONG.iter().position(|s| s.motif == Motif::Pursuit).unwrap();
        run_to_section(&mut sim, pursuit);
        // Hold still in the middle with infinite lives and let the pack
        // arrive, then count how often any two of them are deeply overlapped.
        let centre = Vec2::new(WORLD_W * 0.5, WORLD_H * 0.5);
        let (mut overlapped, mut pairs) = (0_u32, 0_u32);
        for tick in 0..(60 * 8) {
            sim.player.body = Body::at(centre);
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
    fn hits_cost_lives_then_the_game() {
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
        assert_eq!(hits, LIVES as usize);
        assert_eq!(sim.lives(), 0);
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

    #[test]
    fn pickups_score_by_layers_and_motif() {
        let mut sim = playing(3);
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        // Intro: bass and lead sound, so a spark is worth two.
        assert_eq!(sim.spark_value(), 2);
        let spark = sim.sparks()[0];
        sim.player.body = Body::at(spark.body.pos);
        let pickups = run(&mut sim, TICK_DT, &InputFrame::default(), |c| {
            matches!(c, Cue::Pickup { value: 2 })
        });
        assert_eq!(pickups, 1);
        assert_eq!(sim.score(), 2);

        // Full arrangement in pursuit: four layers, doubled.
        let pursuit = SONG.iter().position(|s| s.motif == Motif::Pursuit).unwrap();
        run_to_section(&mut sim, pursuit);
        assert_eq!(sim.spark_value(), 8);
    }

    #[test]
    fn muting_everything_still_pays_one() {
        let mut sim = playing(3);
        let mute_all = InputFrame {
            toggle_layer: [true; 4],
            ..InputFrame::default()
        };
        sim.advance(TICK_DT, &mute_all);
        let bar = sim.motif().bar_secs();
        run(&mut sim, bar, &InputFrame::default(), |_| true);
        assert_eq!(sim.music().layers.iter().filter(|l| l.active()).count(), 0);
        assert_eq!(sim.spark_value(), 1);
    }

    #[test]
    fn pad_pulls_sparks_in() {
        let mut sim = playing(3);
        run_to_section(&mut sim, 1);
        assert!(sim.magnet_on());
        run(&mut sim, 1.0, &InputFrame::default(), |_| true);
        let nearby = sim
            .sparks()
            .iter()
            .position(|s| (s.body.pos - sim.player().body.pos).length() < MAGNET_RADIUS);
        let Some(index) = nearby else {
            // Put one in reach ourselves.
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
            return;
        };
        let before = (sim.sparks()[index].body.pos - sim.player().body.pos).length();
        sim.advance(TICK_DT * 5.0, &InputFrame::default());
        // It either got closer or got eaten.
        let now = sim
            .sparks()
            .get(index)
            .map_or(0.0, |s| (s.body.pos - sim.player().body.pos).length());
        assert!(now < before);
    }

    #[test]
    fn everything_stays_in_bounds() {
        let mut sim = playing(11);
        let input = InputFrame {
            move_dir: Vec2::new(1.0, 1.0),
            ..InputFrame::default()
        };
        for tick in 0..(60 * 40) {
            sim.player.invuln = 10.0;
            // Drive into a different corner every few seconds.
            let dir = match (tick / 240) % 4 {
                0 => Vec2::new(1.0, 1.0),
                1 => Vec2::new(-1.0, 1.0),
                2 => Vec2::new(-1.0, -1.0),
                _ => Vec2::new(1.0, -1.0),
            };
            sim.advance(
                TICK_DT,
                &InputFrame {
                    move_dir: dir,
                    ..input
                },
            );
            for pos in snapshot(&sim) {
                assert!((0.0..=WORLD_W).contains(&pos.x), "x escaped: {}", pos.x);
                assert!((0.0..=WORLD_H).contains(&pos.y), "y escaped: {}", pos.y);
            }
        }
    }

    #[test]
    fn restart_replaces_the_world_and_skips_the_title() {
        let mut sim = playing(5);
        run(&mut sim, 5.0, &InputFrame::default(), |_| true);
        sim.advance(
            TICK_DT,
            &InputFrame {
                restart: Some(6),
                ..InputFrame::default()
            },
        );
        assert_eq!(sim.seed(), 6);
        assert_eq!(sim.phase(), Phase::Playing);
        assert_eq!(sim.score(), 0);
        assert_eq!(sim.lives(), LIVES);
        assert_eq!(sim.cues().first(), Some(&Cue::Restart));
        assert_eq!(sim.music().section, 0);
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
        for _ in 0..((total + 1.0) * 60.0) as u32 {
            sim.player.invuln = 10.0;
            sim.advance(TICK_DT, &InputFrame::default());
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
        sim.advance(
            TICK_DT * 4.0,
            &InputFrame {
                move_dir: Vec2::new(1.0, 0.0),
                ..InputFrame::default()
            },
        );
        let body = sim.player().body;
        assert_eq!(body.interpolated(0.0), body.prev_pos);
        assert_eq!(body.interpolated(1.0), body.pos);
        assert_ne!(body.prev_pos, body.pos);
    }

    #[test]
    fn lanes_cover_every_wall() {
        let lanes: std::collections::BTreeSet<usize> = (0..12).map(lane_of).collect();
        assert_eq!(lanes.len(), WALL_LANES);
        assert!(lanes.iter().all(|&lane| lane < WALL_LANES));
    }
}
