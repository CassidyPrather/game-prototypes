//! Small Hours' macroquad frontend: keys and pointer into steps, and the
//! night onto the screen.
//!
//! The rules are the library's `small_hours::night`; nothing here decides
//! anything. This file turns presses into [`Step`]s, plays the cues that
//! come back, and keeps the few timers the drawing needs: how far through
//! the last step's motion we are, how long dawn has been up, and the film a
//! cold dawn runs backward. `draw` paints it, `audio` sounds it.
//!
//! An animation never stands between a press and an hour: a step that
//! arrives while the last one is still moving simply starts from where the
//! rules already are.

mod audio;
mod draw;

use game_prototypes::small_hours::night::{Cell, Cue, Step, Target, Watch, World};
use game_prototypes::small_hours::nights::NIGHTS;
use game_prototypes::small_hours::sfx::Sfx;
use macroquad::color::Color;
use macroquad::input::{
    KeyCode, MouseButton, is_key_pressed, is_mouse_button_pressed, mouse_position,
};
use macroquad::math::{Vec2, vec2};
use macroquad::window::{screen_height, screen_width};

use crate::ui::{Frame, MqVec2};

use self::audio::Audio;

/// How long one hour takes to play out on screen: the keepers go first,
/// then the shades.
const STEP_SECS: f32 = 0.26;
/// The gap between hours while sleeping through to dawn.
const SLEEP_SECS: f32 = 0.2;
/// How long a cold dawn sits before the night runs backward by itself.
const COLD_SECS: f32 = 1.5;

/// Where things are, in frame units. Drawing and hit-testing both read
/// these, so they cannot disagree.
mod layout {
    use super::{Cell, Vec2, vec2};
    use game_prototypes::small_hours::night::Ground;

    pub const BOARD: Vec2 = vec2(400.0, 298.0);
    pub const RULER_Y: f32 = 36.0;
    pub const CAPS_Y: f32 = 566.0;
    pub const CAP: f32 = 30.0;
    pub const UNDO: Vec2 = vec2(566.0, CAPS_Y);
    pub const RESTART: Vec2 = vec2(608.0, CAPS_Y);
    pub const WAIT: Vec2 = vec2(672.0, CAPS_Y);
    pub const WAIT_W: f32 = 74.0;
    pub const ENTER: Vec2 = vec2(750.0, CAPS_Y);
    pub const ENTER_W: f32 = 46.0;
    pub const PIP_X: f32 = 30.0;
    pub const PIP_Y: f32 = 30.0;
    pub const PIP_GAP: f32 = 21.0;
    pub const LANTERN_X: f32 = 36.0;
    pub const LANTERN_GAP: f32 = 28.0;
    /// The mute toggle, top right, beside where the shell's back button
    /// goes (the night pips hold the top-left corner).
    pub const SPEAKER: Vec2 = vec2(722.0, 30.0);
    /// The square around [`SPEAKER`] that takes a press.
    pub const SPEAKER_HIT: f32 = 36.0;
    /// The shell's back button, when it has to sit on the frame.
    pub const BACK: Vec2 = vec2(770.0, 30.0);
    /// The caps take a press anywhere below this and right of
    /// [`CAPS_LEFT`], each owning the strip nearest it, so a thumb on a
    /// phone, where a cap is a few millimetres across, need not land on
    /// one. The strip runs on into the letterbox below the frame.
    pub const CAPS_TOP: f32 = 524.0;
    pub const CAPS_LEFT: f32 = 540.0;

    /// The side of one cell.
    pub fn cell(ground: &Ground) -> f32 {
        (434.0 / f32::from(ground.height))
            .min(640.0 / f32::from(ground.width))
            .min(62.0)
    }

    /// The middle of a cell, which need not be a whole one.
    pub fn spot(ground: &Ground, x: f32, y: f32) -> Vec2 {
        let side = cell(ground);
        let half = vec2(f32::from(ground.width), f32::from(ground.height)) * 0.5;
        BOARD + (vec2(x, y) + Vec2::splat(0.5) - half) * side
    }

    pub fn centre(ground: &Ground, at: Cell) -> Vec2 {
        spot(ground, f32::from(at.x), f32::from(at.y))
    }

    /// The cell under a frame position, if it is on the board.
    #[allow(clippy::cast_possible_truncation)]
    pub fn cell_at(ground: &Ground, p: Vec2) -> Option<Cell> {
        let side = cell(ground);
        let half = vec2(f32::from(ground.width), f32::from(ground.height)) * 0.5;
        let local = (p - BOARD) / side + half;
        let at = Cell::new(local.x.floor() as i8, local.y.floor() as i8);
        ground.inside(at).then_some(at)
    }

    pub fn pip(night: usize) -> Vec2 {
        vec2((night as f32).mul_add(PIP_GAP, PIP_X), PIP_Y)
    }

    pub fn within(p: Vec2, centre: Vec2, size: Vec2) -> bool {
        (p.x - centre.x).abs() <= size.x * 0.5 && (p.y - centre.y).abs() <= size.y * 0.5
    }

    /// One of the caps along the bottom.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Cap {
        Undo,
        Restart,
        Wait,
        Enter,
    }

    /// The cap whose strip `p` is in, if any: see [`CAPS_TOP`].
    pub fn cap_at(p: Vec2) -> Option<Cap> {
        if p.y < CAPS_TOP || p.x < CAPS_LEFT {
            return None;
        }
        [
            (Cap::Undo, UNDO.x),
            (Cap::Restart, RESTART.x),
            (Cap::Wait, WAIT.x),
            (Cap::Enter, ENTER.x),
        ]
        .into_iter()
        .min_by(|a, b| (a.1 - p.x).abs().total_cmp(&(b.1 - p.x).abs()))
        .map(|(cap, _)| cap)
    }
}

/// What is on screen besides the night itself.
enum Mode {
    Night,
    /// Dawn on a dead hearth, for this long. Negative while the last hour
    /// is still playing out.
    Cold {
        age: f32,
    },
    /// Dawn on a lit one. `earned` is the three marks in order — the
    /// hearth, few keepers, whole lanterns — and `rung` counts how many of
    /// the three have had their moment.
    Kept {
        age: f32,
        earned: [bool; 3],
        rung: u8,
    },
    /// The night running backward through `film`, before living it again.
    Rewind {
        film: Vec<World>,
        age: f32,
        secs: f32,
    },
}

/// One press, whatever it came from.
#[derive(Default)]
struct Press {
    step: Option<Step>,
    undo: bool,
    restart: bool,
    enter: bool,
    night: Option<usize>,
}

impl Press {
    const fn any(&self) -> bool {
        self.step.is_some() || self.undo || self.restart || self.enter || self.night.is_some()
    }
}

pub struct Game {
    audio: Audio,
    night: usize,
    watch: Watch,
    /// The most marks each night has been kept with; zero is not yet.
    best: [u8; NIGHTS.len()],
    clock: f32,
    /// Seconds since the last hour was spent.
    since: f32,
    /// Whether the last change was an hour spent (so it animates) rather
    /// than a jump.
    moved: bool,
    mode: Mode,
    /// Each cell's colour, eased toward whose light reaches it.
    tiles: Vec<Color>,
    motes: Vec<draw::Mote>,
    /// Whether the player has taken a step yet; the arrow caps go once
    /// they have.
    stepped: bool,
    /// Waiting out the rest of the night, an hour at a time.
    sleeping: bool,
    hover: Option<Cell>,
}

impl Game {
    pub async fn load() -> Self {
        let watch = Watch::new(NIGHTS[0]);
        let mut game = Self {
            audio: Audio::load().await,
            night: 0,
            watch,
            best: [0; NIGHTS.len()],
            clock: 0.0,
            since: 9.0,
            moved: false,
            mode: Mode::Night,
            tiles: Vec::new(),
            motes: Vec::new(),
            stepped: false,
            sleeping: false,
            hover: None,
        };
        game.settle_tiles();
        game
    }

    fn open_night(&mut self, night: usize) {
        self.night = night % NIGHTS.len();
        self.watch = Watch::new(NIGHTS[self.night]);
        self.mode = Mode::Night;
        self.moved = false;
        self.sleeping = false;
        self.motes.clear();
        self.settle_tiles();
        self.wake_mote(0);
        self.audio.play(Sfx::Wake(0));
    }

    /// Put every tile straight at its colour, for a change of night.
    fn settle_tiles(&mut self) {
        self.tiles = draw::tile_targets(&self.watch, self.watch.world());
    }

    /// The keeper being played.
    fn me(&self) -> usize {
        self.watch.keepers() - 1
    }

    fn wake_mote(&mut self, keeper: usize) {
        let ground = self.watch.ground();
        let at = layout::centre(ground, ground.beds[keeper]);
        draw::burst(&mut self.motes, at, draw::keeper_colour(keeper), 10, 0.0);
    }

    /// Spend an hour and play what came of it.
    fn spend(&mut self, step: Step) {
        if !self.watch.step(step) {
            return;
        }
        self.stepped = true;
        self.since = 0.0;
        self.moved = true;
        let ground = self.watch.ground().clone();
        let me = self.me();
        let (mut crept, mut noticed, mut drifted) = (false, false, false);
        for cue in self.watch.cues().to_vec() {
            match cue {
                Cue::Walked { keeper, lit, .. } => {
                    if lit || keeper == me {
                        self.audio.play(Sfx::Step(sound_index(keeper)));
                    }
                }
                Cue::Balked { keeper, .. } => {
                    if keeper == me {
                        self.audio.play(Sfx::Balk);
                    }
                }
                Cue::Crept { .. } => crept = true,
                Cue::Snuffed { from, at, .. } => {
                    let delay = if from == at { 0.4 } else { 0.85 } * STEP_SECS;
                    self.audio.later(delay, Sfx::Snuff);
                    let p = layout::centre(&ground, at);
                    draw::smoke(&mut self.motes, p, 9, delay);
                }
                Cue::Fed { embers, .. } => {
                    let delay = 0.85 * STEP_SECS;
                    let out = embers == 0;
                    self.audio
                        .later(delay, if out { Sfx::Out } else { Sfx::Fed });
                    let p = layout::centre(&ground, ground.hearth);
                    draw::smoke(&mut self.motes, p, if out { 16 } else { 7 }, delay);
                    draw::burst(&mut self.motes, p, draw::EMBER, 8, delay);
                }
                Cue::Turned { target, .. } => match target {
                    Target::Keeper(_) => noticed = true,
                    Target::Hearth => drifted = true,
                },
                Cue::Dawn { lit } => {
                    self.sleeping = false;
                    self.mode = if lit {
                        let marks = self.watch.marks();
                        self.best[self.night] = self.best[self.night].max(marks);
                        self.audio.later(STEP_SECS, Sfx::Dawn);
                        Mode::Kept {
                            age: -STEP_SECS,
                            earned: [true, self.watch.few(), self.watch.whole()],
                            rung: 0,
                        }
                    } else {
                        self.audio.later(STEP_SECS, Sfx::Cold);
                        Mode::Cold { age: -STEP_SECS }
                    };
                }
            }
        }
        if crept {
            self.audio.later(STEP_SECS * 0.4, Sfx::Creep);
        }
        if noticed {
            self.audio.later(STEP_SECS * 0.9, Sfx::Notice);
        } else if drifted {
            self.audio.later(STEP_SECS * 0.9, Sfx::Drift);
        }
    }

    fn undo(&mut self) {
        if self.watch.undo() {
            self.audio.play(Sfx::Undo);
            self.mode = Mode::Night;
            self.moved = false;
            self.sleeping = false;
        } else {
            self.audio.play(Sfx::Balk);
        }
    }

    /// Start running the night backward; it is lived again when the film
    /// reaches dusk.
    fn rewind(&mut self) {
        let film = self.watch.history().to_vec();
        let secs = (film.len() as f32 * 0.09).max(0.7);
        self.audio.play(Sfx::Rewind);
        self.mode = Mode::Rewind {
            film,
            age: 0.0,
            secs,
        };
    }

    fn read_press(&mut self) -> Press {
        const DIGITS: [KeyCode; 9] = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        let mut press = Press::default();
        let key = |codes: &[KeyCode]| codes.iter().any(|&code| is_key_pressed(code));
        if key(&[KeyCode::Up, KeyCode::W]) {
            press.step = Some(Step::North);
        } else if key(&[KeyCode::Right, KeyCode::D]) {
            press.step = Some(Step::East);
        } else if key(&[KeyCode::Down, KeyCode::S]) {
            press.step = Some(Step::South);
        } else if key(&[KeyCode::Left, KeyCode::A]) {
            press.step = Some(Step::West);
        } else if key(&[KeyCode::Space, KeyCode::Period]) {
            press.step = Some(Step::Wait);
        }
        press.undo = key(&[KeyCode::Z, KeyCode::Backspace, KeyCode::U]);
        press.restart = key(&[KeyCode::R]);
        let mut mute = key(&[KeyCode::M]);
        press.enter = key(&[KeyCode::Enter, KeyCode::KpEnter]);
        press.night = DIGITS
            .iter()
            .position(|&code| is_key_pressed(code))
            .filter(|&night| night < NIGHTS.len());

        let frame = Frame::fit(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        let pointer = frame.frame_pos(vec2(mx, my));
        let ground = self.watch.ground();
        self.hover = layout::cell_at(ground, pointer);
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(cap) = layout::cap_at(pointer) {
                match cap {
                    layout::Cap::Undo => press.undo = true,
                    layout::Cap::Restart => press.restart = true,
                    layout::Cap::Wait => press.step = Some(Step::Wait),
                    layout::Cap::Enter => press.enter = true,
                }
            } else if layout::within(pointer, layout::SPEAKER, Vec2::splat(layout::SPEAKER_HIT)) {
                mute = true;
            } else if let Some(night) = (0..NIGHTS.len())
                .find(|&night| layout::within(pointer, layout::pip(night), Vec2::splat(20.0)))
            {
                press.night = Some(night);
            } else if let Some(at) = self.hover {
                // A click on the board is one step toward it, along the
                // longer side; a click on yourself is a wait.
                let me = self.watch.world().keepers[self.me()].at;
                let (dx, dy) = (at.x - me.x, at.y - me.y);
                press.step = Some(if dx == 0 && dy == 0 {
                    Step::Wait
                } else if dx.abs() >= dy.abs() {
                    if dx > 0 { Step::East } else { Step::West }
                } else if dy > 0 {
                    Step::South
                } else {
                    Step::North
                });
                if !matches!(self.mode, Mode::Night) {
                    // Off the night, a click anywhere on the board is the
                    // same as the return key.
                    press.step = None;
                    press.enter = true;
                }
            }
        }
        if mute {
            // Mute is a press like any other, so it is also the gesture
            // that lets the browser play sound at all.
            self.audio.wake();
            self.audio.toggle_mute();
        }
        press
    }

    fn act(&mut self, press: &Press) {
        if let Some(night) = press.night {
            self.open_night(night);
            return;
        }
        if press.restart {
            self.watch.restart();
            self.mode = Mode::Night;
            self.moved = false;
            self.sleeping = false;
            self.audio.play(Sfx::Wake(0));
            self.wake_mote(0);
            return;
        }
        match self.mode {
            Mode::Night => {
                if press.undo {
                    self.undo();
                } else if let Some(step) = press.step {
                    self.sleeping = false;
                    self.spend(step);
                } else if press.enter {
                    self.sleeping = !self.sleeping;
                }
            }
            Mode::Cold { .. } => {
                if press.undo {
                    self.undo();
                } else if press.enter || press.step.is_some() {
                    self.rewind();
                }
            }
            Mode::Kept { age, .. } => {
                if press.undo {
                    self.undo();
                } else if press.enter && age > 0.3 {
                    self.open_night(self.night + 1);
                }
            }
            Mode::Rewind { .. } => {}
        }
    }

    fn tick_mode(&mut self, dt: f32) {
        match &mut self.mode {
            Mode::Night => {
                if self.sleeping && self.since >= SLEEP_SECS {
                    self.spend(Step::Wait);
                }
            }
            Mode::Cold { age } => {
                *age += dt;
                if *age > COLD_SECS {
                    self.rewind();
                }
            }
            Mode::Kept { age, earned, rung } => {
                *age += dt;
                if *rung < 3 && *age > draw::mark_born(*rung) {
                    if earned[usize::from(*rung)] {
                        self.audio.play(Sfx::Mark(*rung));
                    }
                    *rung += 1;
                }
            }
            Mode::Rewind { age, secs, .. } => {
                *age += dt;
                if *age >= *secs {
                    self.watch.again();
                    self.mode = Mode::Night;
                    self.moved = false;
                    let me = self.me();
                    self.audio.play(Sfx::Wake(sound_index(me)));
                    self.wake_mote(me);
                }
            }
        }
    }

    /// How much the hearth and the dark should be heard, each `0..=1`.
    fn loop_levels(&self) -> (f32, f32) {
        let world = self.watch.world();
        let fire = f32::from(world.embers) / f32::from(self.watch.level().embers.max(1));
        let fire = if world.lit() {
            fire.mul_add(0.6, 0.4)
        } else {
            0.0
        };
        let dark = match self.mode {
            Mode::Kept { age, .. } => (1.0 - age).clamp(0.15, 1.0),
            _ => 1.0,
        };
        (fire, dark)
    }
}

/// A keeper's index as the sounds count them.
#[allow(clippy::cast_possible_truncation)]
const fn sound_index(keeper: usize) -> u8 {
    keeper as u8
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        self.clock += dt;
        self.since += dt;
        let press = self.read_press();
        if press.any() {
            self.audio.wake();
            self.act(&press);
        }
        self.tick_mode(dt);

        let shown = draw::shown(self);
        let targets = draw::tile_targets(&self.watch, shown.tint);
        let ease = (dt * 9.0).min(1.0);
        if self.tiles.len() == targets.len() {
            for (tile, target) in self.tiles.iter_mut().zip(targets) {
                *tile = draw::mix(*tile, target, ease);
            }
        } else {
            self.tiles = targets;
        }
        draw::drift(&mut self.motes, dt);

        let (fire, dark) = self.loop_levels();
        self.audio.update(dt, fire, dark);
    }

    fn draw(&self, frame: &Frame) {
        draw::scene(frame, self);
    }

    fn leave(&mut self) {
        self.audio.hush();
        self.sleeping = false;
    }

    /// The night pips hold the top-left corner.
    fn back_spot(&self) -> MqVec2 {
        layout::BACK
    }
}

/// The mark the menu shows for this prototype: the hearth, one keeper
/// beside it, and something with eyes at the edge of the light.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    draw::emblem(frame, centre, width, clock);
}

#[cfg(test)]
mod tests {
    use super::layout::{self, Cap, cap_at};
    use macroquad::math::vec2;

    #[test]
    fn every_cap_owns_the_strip_around_it() {
        for (cap, at) in [
            (Cap::Undo, layout::UNDO),
            (Cap::Restart, layout::RESTART),
            (Cap::Wait, layout::WAIT),
            (Cap::Enter, layout::ENTER),
        ] {
            assert_eq!(cap_at(at), Some(cap), "{cap:?} on itself");
            // Above the icon, and far down in a phone's letterbox.
            assert_eq!(cap_at(vec2(at.x, layout::CAPS_TOP)), Some(cap));
            assert_eq!(cap_at(vec2(at.x, 1200.0)), Some(cap));
        }
        // Past the right edge of the frame is still the last cap.
        assert_eq!(cap_at(vec2(900.0, layout::CAPS_Y)), Some(Cap::Enter));
    }

    #[test]
    fn the_strip_leaves_the_board_and_the_lanterns_alone() {
        assert_eq!(cap_at(vec2(400.0, layout::BOARD.y)), None);
        assert_eq!(cap_at(vec2(layout::LANTERN_X, layout::CAPS_Y)), None);
        assert_eq!(cap_at(vec2(layout::UNDO.x, layout::CAPS_TOP - 1.0)), None);
    }

    #[test]
    fn the_back_button_and_the_speaker_keep_apart() {
        let back = game_prototypes::shell::back::ON_FRAME_SIDE.1 * 0.5;
        let speaker = layout::SPEAKER_HIT * 0.5;
        let gap = (layout::BACK.x - back) - (layout::SPEAKER.x + speaker);
        assert!(gap >= 0.0, "they overlap by {}", -gap);
    }
}
