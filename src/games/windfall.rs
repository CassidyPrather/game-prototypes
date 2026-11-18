//! Windfall's macroquad frontend: clicks into pieces, the run onto the
//! screen, cues into sound.
//!
//! The rules are the library's `windfall::garden`; nothing here decides
//! anything. While the player builds, the garden is run to the end after
//! every change and drawn as faint trails, so what a vane will do is never
//! a guess; pressing play then lets the leaves actually fall, one tick at a
//! time, slowly enough to watch. `draw` paints it and `audio` sounds it.

mod audio;
mod draw;

use game_prototypes::windfall::garden::{Cell, Cue, Fate, Loss, Marks, Piece, Plan, Run};
use game_prototypes::windfall::levels::LEVELS;
use game_prototypes::windfall::sound::Sample;
use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position,
};
use macroquad::math::{Vec2, vec2};
use macroquad::window::{screen_height, screen_width};

use crate::ui::{Frame, MqVec2};

use self::audio::Audio;

/// How long one tick takes on screen, at ordinary and fast speed.
const TICK_SECS: f32 = 0.34;
const FAST_SECS: f32 = 0.085;

/// Where things are, in frame units. Drawing and hit-testing both read
/// these, so they cannot disagree.
mod layout {
    use super::{Cell, Vec2, vec2};
    use game_prototypes::windfall::garden::Level;

    /// The middle of the garden bed.
    pub const BED: Vec2 = vec2(400.0, 300.0);
    pub const BED_TOP: f32 = 172.0;
    pub const BED_BOTTOM: f32 = 452.0;
    pub const CAPS_Y: f32 = 564.0;
    pub const CAP: f32 = 40.0;
    pub const CLEAR: Vec2 = vec2(596.0, CAPS_Y);
    pub const UNDO: Vec2 = vec2(642.0, CAPS_Y);
    pub const FAST: Vec2 = vec2(688.0, CAPS_Y);
    pub const PLAY: Vec2 = vec2(748.0, CAPS_Y);
    pub const PLAY_W: f32 = 64.0;
    /// The two rulers, pieces above wind.
    pub const RULER_X: f32 = 52.0;
    pub const RULER_Y: [f32; 2] = [553.0, 578.0];
    /// The three marks, between the rulers and the caps.
    pub const MARKS_X: f32 = 452.0;
    pub const MARK_GAP: f32 = 34.0;
    pub const PIP_Y: f32 = 26.0;
    pub const PIP_GAP: f32 = 30.0;
    pub const SPEAKER: Vec2 = vec2(770.0, 28.0);
    pub const SPEAKER_HIT: f32 = 40.0;

    /// The side of one cell.
    pub fn cell(level: &Level) -> f32 {
        ((BED_BOTTOM - BED_TOP) / f32::from(level.height))
            .min(500.0 / f32::from(level.width))
            .min(76.0)
    }

    /// The top-left corner of the garden.
    pub fn origin(level: &Level) -> Vec2 {
        let side = cell(level);
        vec2(
            f32::from(level.width).mul_add(-side * 0.5, BED.x),
            f32::from(level.height).mul_add(-side, BED_BOTTOM),
        )
    }

    /// The middle of a cell, which need not be a whole one. Row -1 is the
    /// branch, row `height` the baskets.
    pub fn spot(level: &Level, x: f32, y: f32) -> Vec2 {
        let side = cell(level);
        origin(level) + (vec2(x, y) + Vec2::splat(0.5)) * side
    }

    pub fn centre(level: &Level, at: Cell) -> Vec2 {
        spot(level, f32::from(at.x), f32::from(at.y))
    }

    /// The cell under a frame position, if it is in the garden.
    pub fn cell_at(level: &Level, p: Vec2) -> Option<Cell> {
        let local = (p - origin(level)) / cell(level);
        let at = Cell::new(local.x.floor() as i8, local.y.floor() as i8);
        (local.x >= 0.0 && local.y >= 0.0 && level.inside(at)).then_some(at)
    }

    pub fn pip(n: usize, count: usize) -> Vec2 {
        let half = (count as f32 - 1.0) * 0.5;
        vec2((n as f32 - half).mul_add(PIP_GAP, 400.0), PIP_Y)
    }

    pub fn mark(n: usize) -> Vec2 {
        vec2((n as f32).mul_add(MARK_GAP, MARKS_X), CAPS_Y)
    }

    pub fn within(p: Vec2, centre: Vec2, size: Vec2) -> bool {
        (p.x - centre.x).abs() <= size.x * 0.5 && (p.y - centre.y).abs() <= size.y * 0.5
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Cap {
        Clear,
        Undo,
        Fast,
        Play,
    }

    /// The cap under `p`, generously: anything in the bottom strip right
    /// of the marks belongs to the nearest cap, so a thumb need not be
    /// exact.
    pub fn cap_at(p: Vec2) -> Option<Cap> {
        if p.y < CAP.mul_add(-0.75, CAPS_Y) || p.x < CAP.mul_add(-0.75, CLEAR.x) {
            return None;
        }
        [
            (Cap::Clear, CLEAR.x),
            (Cap::Undo, UNDO.x),
            (Cap::Fast, FAST.x),
            (Cap::Play, PLAY.x),
        ]
        .into_iter()
        .min_by(|a, b| (a.1 - p.x).abs().total_cmp(&(b.1 - p.x).abs()))
        .map(|(cap, _)| cap)
    }
}

/// What a plan will do, worked out in advance: each leaf's path and how it
/// ends, and what the run would be worth.
pub struct Preview {
    pub paths: Vec<Vec<Cell>>,
    pub fates: Vec<Fate>,
    pub gusts: u16,
    pub marks: Marks,
    /// The first leaf lost, if any: where the trouble starts.
    pub first_lost: Option<usize>,
}

impl Preview {
    fn of(run: &Run) -> Self {
        let mut run = run.clone();
        let mut paths: Vec<Vec<Cell>> = vec![Vec::new(); run.leaves().len()];
        let mut first_lost = None;
        loop {
            for (path, leaf) in paths.iter_mut().zip(run.leaves()) {
                if leaf.fate != Fate::Hanging && path.last() != Some(&leaf.at) {
                    path.push(leaf.at);
                }
            }
            if run.over() {
                break;
            }
            run.step();
            if first_lost.is_none() {
                first_lost = run.cues().iter().find_map(|c| match c {
                    Cue::Lost { leaf, .. } => Some(*leaf),
                    _ => None,
                });
            }
        }
        for (path, leaf) in paths.iter_mut().zip(run.leaves()) {
            if path.last() != Some(&leaf.at) {
                path.push(leaf.at);
            }
        }
        Self {
            paths,
            fates: run.leaves().iter().map(|l| l.fate).collect(),
            gusts: run.gusts(),
            marks: Marks::of(&run),
            first_lost,
        }
    }
}

/// What the garden is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// The player is placing pieces.
    Build,
    /// The leaves are falling.
    Play,
    /// Every leaf has settled, this long ago. `rung` counts the marks
    /// that have had their moment.
    Over { age: f32, rung: u8 },
}

/// A short-lived flourish: a piece popping in, a vane kicked by a gust, a
/// flip vane swinging round. Keyed by cell, `age` counting up from zero.
#[derive(Clone, Copy, Debug)]
pub struct Kick {
    pub at: Cell,
    pub age: f32,
    pub kind: KickKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KickKind {
    Placed,
    Gust,
    Turned,
}

pub struct Game {
    audio: Audio,
    level: usize,
    /// Every garden's plan, kept while the player looks at others.
    plans: Vec<Plan>,
    /// Earlier plans for this garden, for undo.
    undo: Vec<Plan>,
    /// The best of each mark each garden has earned.
    best: Vec<Marks>,
    run: Run,
    preview: Preview,
    mode: Mode,
    fast: bool,
    clock: f32,
    /// Seconds into the current tick.
    since: f32,
    hover: Option<Cell>,
    pointer: Vec2,
    kicks: Vec<Kick>,
    motes: Vec<draw::Mote>,
    /// Whether the player has ever pressed play; the play cap stops
    /// glowing once they have, unless the garden is ready.
    played: bool,
    /// Which take of a sample plays next, so repeats alternate.
    take: u8,
}

impl Game {
    pub async fn load() -> Self {
        let plans: Vec<Plan> = LEVELS.iter().map(Plan::empty).collect();
        let run = Run::new(&LEVELS[0], &plans[0]);
        let preview = Preview::of(&run);
        Self {
            audio: Audio::load().await,
            level: 0,
            plans,
            undo: Vec::new(),
            best: vec![Marks::default(); LEVELS.len()],
            run,
            preview,
            mode: Mode::Build,
            fast: false,
            clock: 0.0,
            since: 0.0,
            hover: None,
            pointer: Vec2::ZERO,
            kicks: Vec::new(),
            motes: Vec::new(),
            played: false,
            take: 0,
        }
    }

    fn plan(&self) -> &Plan {
        &self.plans[self.level]
    }

    /// Back to building, the run reset to the start of the plan.
    fn rebuild(&mut self) {
        self.run = Run::new(&LEVELS[self.level], &self.plans[self.level]);
        self.preview = Preview::of(&self.run);
        self.mode = Mode::Build;
        self.since = 0.0;
    }

    fn open(&mut self, level: usize) {
        self.level = level.min(LEVELS.len() - 1);
        self.undo.clear();
        self.kicks.clear();
        self.motes.clear();
        self.rebuild();
        self.audio.play(Sample::Release(0));
    }

    fn set(&mut self, at: Cell, piece: Option<Piece>) {
        let level = LEVELS[self.level];
        let before = self.plans[self.level].clone();
        if self.plans[self.level].set(&level, at, piece) {
            self.undo.push(before);
            self.audio.play(if piece.is_some() {
                Sample::Place
            } else {
                Sample::Remove
            });
            self.kicks.push(Kick {
                at,
                age: 0.0,
                kind: KickKind::Placed,
            });
            if piece.is_none() {
                draw::puff(&mut self.motes, layout::centre(&level, at), draw::DUST, 6);
            }
            self.rebuild();
        }
    }

    fn undo(&mut self) {
        if let Some(plan) = self.undo.pop() {
            self.plans[self.level] = plan;
            self.audio.play(Sample::Remove);
            self.rebuild();
        }
    }

    fn clear(&mut self) {
        if self.plan().count() > 0 {
            let level = LEVELS[self.level];
            let spots: Vec<_> = self.plan().pieces().map(|(at, _)| at).collect();
            for at in spots {
                draw::puff(&mut self.motes, layout::centre(&level, at), draw::DUST, 4);
            }
            self.undo.push(self.plan().clone());
            self.plans[self.level].clear();
            self.audio.play(Sample::Remove);
            self.rebuild();
        }
    }

    fn toggle_play(&mut self) {
        match self.mode {
            Mode::Build => {
                self.run = Run::new(&LEVELS[self.level], &self.plans[self.level]);
                self.mode = Mode::Play;
                self.played = true;
                // The first tick starts at once: the press is answered by a
                // leaf letting go, not by a pause.
                self.since = self.tick_secs();
            }
            Mode::Play | Mode::Over { .. } => self.rebuild(),
        }
    }

    const fn tick_secs(&self) -> f32 {
        if self.fast { FAST_SECS } else { TICK_SECS }
    }

    const fn next_take(&mut self) -> u8 {
        self.take = (self.take + 1) % 2;
        self.take
    }

    /// Play one tick and answer its cues.
    fn tick(&mut self) {
        self.run.step();
        let level = LEVELS[self.level];
        let delay = self.tick_secs() * 0.8;
        let (mut dropped, mut gust, mut turned, mut home, mut lost) =
            (false, false, false, false, false);
        for cue in self.run.cues().to_vec() {
            match cue {
                Cue::Dropped { .. } => dropped = true,
                Cue::Gust { from, .. } => {
                    gust = true;
                    self.kicks.push(Kick {
                        at: from,
                        age: 0.0,
                        kind: KickKind::Gust,
                    });
                }
                Cue::Turned { at, .. } => {
                    turned = true;
                    self.kicks.push(Kick {
                        at,
                        age: 0.0,
                        kind: KickKind::Turned,
                    });
                }
                Cue::Home { leaf } => {
                    home = true;
                    let l = self.run.leaves()[leaf];
                    let p = layout::spot(&level, f32::from(l.at.x), f32::from(level.height) - 0.1);
                    draw::burst(&mut self.motes, p, draw::leaf_colour(l.hue), 9, delay);
                }
                Cue::Lost { leaf, loss } => {
                    lost = true;
                    let l = self.run.leaves()[leaf];
                    if loss != Loss::Blown {
                        let p = layout::centre(&level, l.at);
                        draw::burst(&mut self.motes, p, draw::DUST, 7, delay);
                    }
                }
                Cue::Over { all_home } => {
                    self.mode = Mode::Over { age: 0.0, rung: 0 };
                    if all_home {
                        let marks = Marks::of(&self.run);
                        self.best[self.level] = self.best[self.level].best(marks);
                        self.audio.later(delay + 0.25, Sample::Solved);
                    }
                }
            }
        }
        // One of each sound per tick, however many leaves: at speed a
        // flurry of identical samples is only noise.
        if dropped {
            let take = self.next_take();
            self.audio.play(Sample::Release(take));
        }
        if gust {
            let take = self.next_take();
            self.audio.later(0.03, Sample::Gust(take));
        }
        if turned {
            self.audio.later(0.06, Sample::Flip);
        }
        if home {
            self.audio.later(delay, Sample::Catch);
        }
        if lost {
            self.audio.later(delay, Sample::Miss);
        }
    }

    fn read_input(&mut self) {
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
        let key = |codes: &[KeyCode]| codes.iter().any(|&code| is_key_pressed(code));
        let level = LEVELS[self.level];
        let frame = Frame::fit(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        self.pointer = frame.frame_pos(vec2(mx, my));
        self.hover = layout::cell_at(&level, self.pointer);

        let left = is_mouse_button_pressed(MouseButton::Left);
        let right = is_mouse_button_pressed(MouseButton::Right);
        let any_key = macroquad::input::get_last_key_pressed().is_some();
        if left || right || any_key {
            self.audio.wake();
        }

        if key(&[KeyCode::M]) {
            self.audio.toggle_mute();
        }
        if key(&[KeyCode::F]) {
            self.fast = !self.fast;
        }
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if key(&[KeyCode::Space]) {
            self.toggle_play();
        }
        if key(&[KeyCode::Enter, KeyCode::KpEnter]) {
            self.enter();
        }
        if key(&[KeyCode::Z, KeyCode::Backspace, KeyCode::U]) && self.mode == Mode::Build {
            self.undo();
        }
        if key(&[KeyCode::Delete, KeyCode::C]) && self.mode == Mode::Build {
            self.clear();
        }
        if key(&[KeyCode::Right, KeyCode::RightBracket, KeyCode::PageDown]) {
            self.open(self.level + 1);
        } else if key(&[KeyCode::Left, KeyCode::LeftBracket, KeyCode::PageUp]) {
            self.open(self.level.saturating_sub(1));
        } else if let Some(n) = DIGITS
            .iter()
            .position(|&code| is_key_pressed(code))
            .filter(|&n| n < LEVELS.len())
        {
            self.open(n);
        } else if key(&[KeyCode::Key0]) && LEVELS.len() >= 10 {
            self.open(9);
        }

        if !(left || right) {
            return;
        }
        let p = self.pointer;
        if let Some(cap) = layout::cap_at(p) {
            if left {
                match cap {
                    layout::Cap::Play => self.toggle_play(),
                    layout::Cap::Fast => self.fast = !self.fast,
                    layout::Cap::Undo if self.mode == Mode::Build => self.undo(),
                    layout::Cap::Clear if self.mode == Mode::Build => self.clear(),
                    layout::Cap::Undo | layout::Cap::Clear => self.rebuild(),
                }
            }
        } else if layout::within(p, layout::SPEAKER, Vec2::splat(layout::SPEAKER_HIT)) {
            self.audio.toggle_mute();
        } else if let Some(n) = (0..LEVELS.len())
            .find(|&n| layout::within(p, layout::pip(n, LEVELS.len()), vec2(layout::PIP_GAP, 36.0)))
        {
            self.open(n);
        } else if let Some(at) = self.hover {
            if self.mode == Mode::Build {
                let here = self.plan().get(at);
                let piece = if right || shift {
                    Piece::previous(here, level.kit)
                } else {
                    Piece::next(here, level.kit)
                };
                self.set(at, piece);
            } else {
                // Touching the garden mid-run is reaching in to change it.
                self.rebuild();
            }
        } else if matches!(self.mode, Mode::Over { .. }) && self.run.all_home() && left {
            // A click anywhere else, once a garden is won, moves on.
            self.enter();
        }
    }

    /// The return key: play, or once a garden is won, the next one.
    fn enter(&mut self) {
        match self.mode {
            Mode::Over { age, .. } if self.run.all_home() && age > 0.4 => {
                if self.level + 1 < LEVELS.len() {
                    self.open(self.level + 1);
                } else {
                    self.rebuild();
                }
            }
            Mode::Over { .. } => self.rebuild(),
            Mode::Build | Mode::Play => self.toggle_play(),
        }
    }

    fn advance(&mut self, dt: f32) {
        let tick_secs = self.tick_secs();
        match &mut self.mode {
            Mode::Build => {}
            Mode::Play => {
                self.since += dt;
                // At most a few ticks a frame, so a stalled tab does not
                // fire a whole garden's sounds at once.
                let mut budget = 3;
                while self.since >= self.tick_secs() && budget > 0 && self.mode == Mode::Play {
                    self.since -= self.tick_secs();
                    self.tick();
                    budget -= 1;
                }
                self.since = self.since.min(self.tick_secs());
            }
            Mode::Over { age, rung } => {
                *age += dt;
                self.since = (self.since + dt).min(tick_secs);
                if self.run.all_home() && *rung < 3 && *age > draw::mark_born(*rung) {
                    let marks = Marks::of(&self.run);
                    let earned = [marks.home, marks.few, marks.gentle];
                    if earned[usize::from(*rung)] {
                        self.audio.play(Sample::Mark);
                    }
                    *rung += 1;
                }
            }
        }
    }
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        self.clock += dt;
        self.read_input();
        self.advance(dt);
        for kick in &mut self.kicks {
            kick.age += dt;
        }
        self.kicks.retain(|k| k.age < 0.6);
        draw::drift(&mut self.motes, dt);
        self.audio.update(dt);
    }

    fn draw(&self, frame: &Frame) {
        draw::scene(frame, self);
    }

    fn leave(&mut self) {
        self.audio.hush();
        if self.mode != Mode::Build {
            self.rebuild();
        }
    }

    /// The level pips hold the top middle and the speaker the top right,
    /// so the default top-left corner is free.
    fn back_spot(&self) -> MqVec2 {
        MqVec2::from(game_prototypes::shell::back::DEFAULT_SPOT)
    }
}

/// The mark the menu shows for this prototype: a branch letting go of
/// leaves, one blown sideways by a vane into a basket.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    draw::emblem(frame, centre, width, clock);
}

#[cfg(test)]
mod tests {
    use super::layout::{self, Cap, cap_at};
    use game_prototypes::windfall::garden::Cell;
    use game_prototypes::windfall::levels::LEVELS;
    use macroquad::math::vec2;

    #[test]
    fn every_cap_owns_the_strip_around_it() {
        for (cap, at) in [
            (Cap::Clear, layout::CLEAR),
            (Cap::Undo, layout::UNDO),
            (Cap::Fast, layout::FAST),
            (Cap::Play, layout::PLAY),
        ] {
            assert_eq!(cap_at(at), Some(cap), "{cap:?} on itself");
            assert_eq!(cap_at(vec2(at.x, 1200.0)), Some(cap), "{cap:?} low down");
        }
        assert_eq!(cap_at(vec2(900.0, layout::CAPS_Y)), Some(Cap::Play));
        assert_eq!(cap_at(layout::mark(2)), None, "the marks are not caps");
    }

    #[test]
    fn every_garden_fits_between_the_pips_and_the_caps() {
        for level in &LEVELS {
            let top = layout::spot(level, 0.0, -1.0);
            let bottom = layout::spot(level, 0.0, f32::from(level.height));
            let right = layout::spot(level, f32::from(level.width - 1), 0.0);
            assert!(top.y > layout::PIP_Y + 40.0, "branch under the pips");
            assert!(bottom.y < layout::CAPS_Y - 50.0, "baskets above the caps");
            assert!(top.x > 100.0 && right.x < 700.0, "inside the frame");
        }
    }

    #[test]
    fn a_cell_is_found_where_it_is_drawn() {
        for level in &LEVELS {
            for y in 0..level.height {
                for x in 0..level.width {
                    let at = Cell::new(x, y);
                    assert_eq!(layout::cell_at(level, layout::centre(level, at)), Some(at));
                }
            }
            let above = layout::spot(level, 0.0, -1.0);
            assert_eq!(layout::cell_at(level, above), None, "the branch is no cell");
        }
    }
}
