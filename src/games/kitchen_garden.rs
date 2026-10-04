//! Kitchen Garden's frontend: the mouse in, soft shapes and sound out.
//!
//! Everything that decides anything lives in the library's
//! `kitchen_garden::day`, which in turn runs the reusable `cooking` module.
//! This file turns the pointer into the day's commands — dragging food
//! between places, holding the button down on something that wants work,
//! clicking to light, harvest, gather or buy — and turns its cues into
//! sound and motion. Drawing is in [`draw`]; the food is drawn by [`icons`]
//! with the shapes in [`pen`].
//!
//! No text. A dish's price is a row of coins, a customer's patience is a
//! ring, the time of day is the sun, the score is a jar of coins with three
//! notches, and how a food is made is a card of pictures: what goes in,
//! the mark of the process, and what comes out.

mod audio;
mod draw;
mod fx;
mod icons;
mod pen;

use std::collections::HashMap;

use game_prototypes::cooking::{self, Drive, Food, Process, TICKS_PER_SEC};
use game_prototypes::kitchen_garden::day::{
    Chore, Cue, Customer, Day, PANTRY, PLOTS, Place, Refusal, SEATS, StationId,
};
use game_prototypes::kitchen_garden::sfx::Sfx;
use macroquad::input::{
    KeyCode, MouseButton, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    is_mouse_button_released, mouse_position,
};
use macroquad::math::{Rect, Vec2, vec2};

use crate::ui::{Frame, MqVec2};
use audio::Audio;
use fx::Fx;

/// Seconds per tick of the day.
const TICK: f32 = 1.0 / TICKS_PER_SEC as f32;
/// Strokes a second while the button is held on something that wants work.
const STROKE_RATE: f32 = 6.5;
/// How long a press on a loaded board waits before it becomes work rather
/// than a grab, in seconds.
const HOLD_TO_WORK: f32 = 0.16;
/// How far the pointer moves before a press becomes a drag, frame units.
const DRAG_SLOP: f32 = 7.0;

/// Where everything sits, in frame units. Drawing and hit-testing both read
/// these, so what you see is what you can grab.
pub mod layout {
    use game_prototypes::kitchen_garden::day::{MARKET, Place, StationId};
    use macroquad::math::{Rect, Vec2, vec2};

    /// The sky along the top.
    pub const SKY_H: f32 = 62.0;
    /// The garden is everything left of this.
    pub const GARDEN_W: f32 = 300.0;
    /// The hatch's column is everything right of this.
    pub const HATCH_X: f32 = 690.0;

    /// A garden plot.
    pub fn plot(i: usize) -> Rect {
        let (col, row) = ((i % 2) as f32, (i / 2) as f32);
        Rect::new(
            col.mul_add(138.0, 14.0),
            row.mul_add(74.0, 76.0),
            130.0,
            64.0,
        )
    }
    pub const HEN: Rect = Rect::new(14.0, 306.0, 76.0, 62.0);
    pub const NEST: Rect = Rect::new(14.0, 368.0, 78.0, 36.0);
    pub const COW: Rect = Rect::new(104.0, 300.0, 136.0, 104.0);
    pub const PAIL: Rect = Rect::new(240.0, 356.0, 52.0, 48.0);
    pub const WELL: Rect = Rect::new(12.0, 420.0, 100.0, 132.0);
    pub const BUCKET: Rect = Rect::new(112.0, 506.0, 50.0, 46.0);
    pub const STUMP: Rect = Rect::new(168.0, 438.0, 72.0, 84.0);
    pub const COMPOST: Rect = Rect::new(236.0, 500.0, 60.0, 72.0);

    /// The hearth's stonework, under the pot and pan.
    pub const HEARTH: Rect = Rect::new(306.0, 104.0, 216.0, 200.0);
    /// The firebox in the hearth.
    pub const FIREBOX: Rect = Rect::new(330.0, 222.0, 168.0, 70.0);
    /// Where the split logs are stacked, under the hatch's ledge.
    pub const LOGS: Rect = Rect::new(306.0, 300.0, 216.0, 14.0);

    /// A station.
    pub const fn station(id: StationId) -> Rect {
        match id {
            StationId::Pot => Rect::new(316.0, 112.0, 96.0, 102.0),
            StationId::Pan => Rect::new(414.0, 140.0, 104.0, 74.0),
            StationId::Oven => Rect::new(530.0, 92.0, 154.0, 212.0),
            StationId::Board => Rect::new(306.0, 326.0, 92.0, 104.0),
            StationId::Quern => Rect::new(402.0, 326.0, 92.0, 104.0),
            StationId::Bowl => Rect::new(498.0, 326.0, 92.0, 104.0),
            StationId::Crock => Rect::new(594.0, 326.0, 92.0, 104.0),
        }
    }

    /// The counter the four worked stations stand on.
    pub const COUNTER: Rect = Rect::new(300.0, 404.0, 390.0, 34.0);

    /// A pantry shelf slot.
    pub fn shelf(i: usize) -> Rect {
        let (col, row) = ((i % 4) as f32, (i / 4) as f32);
        Rect::new(
            col.mul_add(63.0, 312.0),
            row.mul_add(72.0, 452.0),
            58.0,
            64.0,
        )
    }

    /// One of the market's wares.
    pub fn ware(i: usize) -> Rect {
        Rect::new(570.0, (i as f32).mul_add(48.0, 452.0), 114.0, 44.0)
    }

    /// A place at the hatch.
    pub fn seat(i: usize) -> Rect {
        Rect::new(HATCH_X + 4.0, (i as f32).mul_add(122.0, 72.0), 104.0, 118.0)
    }

    /// The coin jar.
    pub const JAR: Rect = Rect::new(712.0, 446.0, 70.0, 140.0);
    /// The mute toggle, in the sky.
    pub const SPEAKER: Rect = Rect::new(8.0, 8.0, 26.0, 26.0);

    /// The rectangle a place is drawn in.
    pub fn place(place: Place) -> Rect {
        match place {
            Place::Shelf(i) => shelf(i),
            Place::Pantry => shelf(0),
            Place::Station(id) => station(id),
            Place::Plot(i) => plot(i),
            Place::Nest => NEST,
            Place::Pail => PAIL,
            Place::Bucket => BUCKET,
            Place::Market(food) => {
                let i = MARKET.iter().position(|&f| f == food).unwrap_or(0);
                ware(i)
            }
            Place::Seat(i) => seat(i),
            Place::Compost => COMPOST,
        }
    }

    /// The middle of a rect.
    pub fn mid(r: Rect) -> Vec2 {
        vec2(r.w.mul_add(0.5, r.x), r.h.mul_add(0.5, r.y))
    }
}

/// What the pointer is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spot {
    Place(Place),
    Chore(Chore),
    Hen,
    Speaker,
}

/// A press in progress.
#[derive(Clone, Copy, Debug)]
struct Press {
    spot: Spot,
    at: Vec2,
    held: f32,
    /// Whether this press has done work; a working press never drags.
    worked: bool,
    /// Seconds to the next stroke while held.
    next_stroke: f32,
}

/// Food on its way somewhere under the pointer.
#[derive(Clone, Copy, Debug)]
pub struct Carry {
    pub from: Place,
    pub food: Food,
}

/// Someone leaving the hatch, for the frontend to see off.
#[derive(Clone, Copy, Debug)]
pub struct Leaving {
    pub customer: Customer,
    pub seat: usize,
    pub happy: bool,
    pub age: f32,
}

/// A run of Kitchen Garden.
pub struct Game {
    day: Day,
    audio: Audio,
    fx: Fx,
    /// Where the pointer is, frame units.
    pointer: Vec2,
    press: Option<Press>,
    carry: Option<Carry>,
    hover: Option<Spot>,
    /// Seconds the pointer has rested on `hover`.
    hover_for: f32,
    /// Seconds of day not yet ticked.
    lag: f32,
    clock: f32,
    /// Whether the title is still up, waiting on a first click.
    title: bool,
    /// Seconds since dusk.
    dusk_for: f32,
    /// Seconds since each seat's customer arrived.
    arrived: [f32; SEATS],
    leaving: Vec<Leaving>,
    /// Every dish served today, in order.
    served: Vec<Food>,
    /// Seconds since something was last done to a place, for its bounce.
    bumps: HashMap<Spot, f32>,
    /// How many runs have started, so each day gets its own customers.
    runs: u64,
    /// Seconds to the next puff of steam, smoke or embers.
    puff: f32,
    /// Each seat's customer as last seen, so one who has just gone can
    /// still be drawn going.
    seen: [Option<Customer>; SEATS],
}

impl Game {
    pub async fn load() -> Self {
        Self {
            day: Day::new(0),
            audio: Audio::load().await,
            fx: Fx::default(),
            pointer: Vec2::ZERO,
            press: None,
            carry: None,
            hover: None,
            hover_for: 0.0,
            lag: 0.0,
            clock: 0.0,
            title: true,
            dusk_for: 0.0,
            arrived: [0.0; SEATS],
            leaving: Vec::new(),
            served: Vec::new(),
            bumps: HashMap::new(),
            runs: 0,
            seen: [None; SEATS],
            puff: 0.0,
        }
    }

    fn restart(&mut self) {
        self.runs += 1;
        self.day = Day::new(self.runs.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        self.fx = Fx::default();
        self.press = None;
        self.carry = None;
        self.lag = 0.0;
        self.dusk_for = 0.0;
        self.leaving.clear();
        self.served.clear();
        self.audio.play(Sfx::Dawn);
    }
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        self.clock += dt;
        if is_key_pressed(KeyCode::M) {
            self.audio.wake();
            self.audio.toggle_mute();
        }
        self.pointer_input(dt);
        if !self.title && !self.day.is_over() {
            self.lag = (self.lag + dt).min(0.25);
            // A fixed step: whole ticks out of the frame time.
            #[allow(clippy::while_float)]
            while self.lag >= TICK {
                self.lag -= TICK;
                self.day.tick();
                self.react();
            }
        }
        if self.day.is_over() {
            self.dusk_for += dt;
            if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
                self.restart();
            }
        }
        for t in self.bumps.values_mut() {
            *t += dt;
        }
        for (seat, t) in self.arrived.iter_mut().enumerate() {
            if self.day.seats()[seat].is_some() {
                *t += dt;
            }
        }
        for l in &mut self.leaving {
            l.age += dt;
        }
        self.leaving.retain(|l| l.age < 1.6);
        self.ambient(dt);
        self.fx.update(dt);
        self.audio.update(dt, self.day.fire(), self.title);
    }

    fn draw(&self, frame: &Frame) {
        draw::scene(self, frame);
    }

    fn leave(&mut self) {
        self.audio.hush();
        self.press = None;
        self.carry = None;
    }
}

// The pointer.
impl Game {
    fn pointer_input(&mut self, dt: f32) {
        let frame = Frame::fit(
            macroquad::window::screen_width(),
            macroquad::window::screen_height(),
        );
        let (mx, my) = mouse_position();
        self.pointer = frame.frame_pos(MqVec2::new(mx, my));
        let spot = spot_at(self.pointer);
        if spot == self.hover {
            self.hover_for += dt;
        } else {
            if spot.is_some() && self.carry.is_none() && self.press.is_none() {
                self.audio.play(Sfx::Tick);
            }
            self.hover = spot;
            self.hover_for = 0.0;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            self.audio.wake();
            if self.title {
                self.title = false;
                self.audio.play(Sfx::Dawn);
                return;
            }
            if self.day.is_over() {
                if self.dusk_for > 1.5 {
                    self.restart();
                }
                return;
            }
            if let Some(spot) = spot {
                self.press_on(spot);
            }
        }

        if let Some(mut press) = self.press {
            if is_mouse_button_down(MouseButton::Left) {
                press.held += dt;
                let moved = self.pointer.distance(press.at) > DRAG_SLOP;
                if self.carry.is_none() && !press.worked && moved {
                    self.grab(press.spot);
                } else if self.carry.is_none() && self.wants_work(press.spot, press.held) {
                    press.next_stroke -= dt;
                    if !press.worked || press.next_stroke <= 0.0 {
                        press.worked = true;
                        press.next_stroke += 1.0 / STROKE_RATE;
                        press.next_stroke = press.next_stroke.max(0.0);
                        self.stroke(press.spot);
                    }
                }
                self.press = Some(press);
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            if let Some(carry) = self.carry.take() {
                self.drop_on(carry, spot);
            } else if let Some(press) = self.press {
                if !press.worked && spot == Some(press.spot) {
                    self.click(press.spot);
                }
            }
            self.press = None;
        }
    }

    const fn press_on(&mut self, spot: Spot) {
        self.press = Some(Press {
            spot,
            at: self.pointer,
            held: 0.0,
            worked: false,
            next_stroke: 0.0,
        });
    }

    /// Whether holding the button on `spot` for `held` seconds is work.
    fn wants_work(&self, spot: Spot, held: f32) -> bool {
        match spot {
            Spot::Chore(_) => true,
            Spot::Place(Place::Station(id)) => {
                let station = self.day.station(id);
                if station.drive() != Drive::Effort {
                    return false;
                }
                match station.state() {
                    cooking::State::Working { .. } => true,
                    // A loaded board could be grabbed back instead, so it
                    // waits a moment to be sure.
                    cooking::State::Idle => !station.contents().is_empty() && held >= HOLD_TO_WORK,
                    cooking::State::Ready { .. } => false,
                }
            }
            _ => false,
        }
    }

    fn stroke(&mut self, spot: Spot) {
        match spot {
            Spot::Chore(chore) => {
                let _ = self.day.chore(chore);
            }
            Spot::Place(Place::Station(id)) => {
                let _ = self.day.work(id);
            }
            _ => {}
        }
        self.bump(spot);
        self.react();
    }

    fn grab(&mut self, spot: Spot) {
        let Spot::Place(from) = spot else {
            return;
        };
        if let Ok(food) = self.day.can_move(from, Place::Compost) {
            self.carry = Some(Carry { from, food });
            self.audio.play(Sfx::Lift);
        } else if let Err(refusal) = self.day.peek(from) {
            // A market ware you cannot afford still says so.
            if refusal == Refusal::Poor {
                self.audio.play(Sfx::Nope);
                self.bump(spot);
            }
        }
    }

    fn drop_on(&mut self, carry: Carry, spot: Option<Spot>) {
        let to = match spot {
            Some(Spot::Place(to)) if to != carry.from => to,
            // Back where it came from: nothing to do.
            Some(Spot::Place(_)) => {
                self.audio.play(Sfx::Set);
                return;
            }
            // Dropped on nothing in particular: put it away, unless it
            // came from the pantry, where it simply goes back.
            _ if matches!(carry.from, Place::Shelf(_)) => {
                self.audio.play(Sfx::Set);
                return;
            }
            _ => Place::Pantry,
        };
        match self.day.move_food(carry.from, to) {
            Ok(_) => {
                if to == Place::Pantry {
                    self.fly(carry.food, self.pointer, self.shelf_rect_for(carry.food));
                }
            }
            Err(_) => {
                self.fx.fly(
                    carry.food,
                    self.pointer,
                    layout::mid(layout::place(carry.from)),
                );
            }
        }
        self.react();
    }

    fn click(&mut self, spot: Spot) {
        match spot {
            Spot::Speaker => self.audio.toggle_mute(),
            Spot::Hen => {
                self.audio.play(Sfx::Cluck);
                self.bump(spot);
            }
            Spot::Chore(_) => {}
            Spot::Place(place) => match place {
                Place::Station(id) => {
                    let station = self.day.station(id);
                    if station.ready().is_some() {
                        self.stow(place);
                    } else if station.is_idle() && !station.contents().is_empty() {
                        match station.drive() {
                            Drive::Clock => {
                                let _ = self.day.start(id);
                            }
                            Drive::Effort => {
                                let _ = self.day.work(id);
                            }
                        }
                        self.bump(spot);
                        self.react();
                    } else {
                        self.audio.play(Sfx::Nope);
                    }
                }
                Place::Plot(_) | Place::Nest | Place::Pail | Place::Bucket | Place::Market(_) => {
                    self.stow(place);
                }
                Place::Shelf(_) | Place::Pantry | Place::Seat(_) | Place::Compost => {
                    self.bump(spot);
                }
            },
        }
    }

    /// Put everything at `from` away, flying each into the pantry.
    fn stow(&mut self, from: Place) {
        let food = self.day.peek(from).ok();
        let moved = self.day.stow_all(from);
        if let Some(food) = food {
            let start = layout::mid(layout::place(from));
            let end = self.shelf_rect_for(food);
            for i in 0..moved {
                self.fx.fly_after(food, start, end, i as f32 * 0.07);
            }
        }
        self.bump(Spot::Place(from));
        self.react();
    }

    fn fly(&mut self, food: Food, from: Vec2, to: Vec2) {
        self.fx.fly(food, from, to);
    }

    /// The middle of the shelf `food` would be stacked on.
    fn shelf_rect_for(&self, food: Food) -> Vec2 {
        let i = self
            .day
            .pantry()
            .iter()
            .position(|s| matches!(s, Some((f, _)) if *f == food))
            .unwrap_or(0);
        layout::mid(layout::shelf(i))
    }

    fn bump(&mut self, spot: Spot) {
        self.bumps.insert(spot, 0.0);
    }

    /// Seconds since `spot` was last bumped.
    #[must_use]
    pub fn bumped(&self, spot: Spot) -> f32 {
        self.bumps.get(&spot).copied().unwrap_or(f32::MAX)
    }
}

// Things that go on while nothing happens.
impl Game {
    /// Steam off the pot, smoke off anything burning, embers off the fire.
    fn ambient(&mut self, dt: f32) {
        self.puff -= dt;
        if self.puff > 0.0 || self.title {
            return;
        }
        self.puff = 0.14;
        for id in [StationId::Pot, StationId::Pan, StationId::Oven] {
            let st = self.day.station(id);
            let r = layout::station(id);
            let top = vec2(r.w.mul_add(0.5, r.x), r.y + 22.0);
            match st.state() {
                cooking::State::Idle => {}
                cooking::State::Working { .. } => match id {
                    StationId::Pot => self.fx.steam(top, 1),
                    StationId::Oven => self
                        .fx
                        .steam(vec2(r.w.mul_add(0.5, r.x) + 40.0, r.y - 12.0), 1),
                    _ => self.fx.embers(top + vec2(-10.0, 30.0), 1),
                },
                cooking::State::Ready { food, .. } => {
                    if food == Food::Charcoal || st.scorch() > 0.5 {
                        self.fx.smoke(top, 1);
                    } else if id == StationId::Pot {
                        self.fx.steam(top, 1);
                    }
                }
            }
        }
        if self.day.fire() > 0.0 {
            self.fx
                .embers(layout::mid(layout::FIREBOX) - vec2(0.0, 10.0), 1);
        }
    }
}

/// What is under `p`.
fn spot_at(p: Vec2) -> Option<Spot> {
    let inside = |r: Rect| r.contains(p);
    if inside(layout::SPEAKER) {
        return Some(Spot::Speaker);
    }
    for seat in 0..SEATS {
        if inside(layout::seat(seat)) {
            return Some(Spot::Place(Place::Seat(seat)));
        }
    }
    for id in StationId::ALL {
        if inside(layout::station(id)) {
            return Some(Spot::Place(Place::Station(id)));
        }
    }
    for i in 0..PANTRY {
        if inside(layout::shelf(i)) {
            return Some(Spot::Place(Place::Shelf(i)));
        }
    }
    for (i, &food) in game_prototypes::kitchen_garden::day::MARKET
        .iter()
        .enumerate()
    {
        if inside(layout::ware(i)) {
            return Some(Spot::Place(Place::Market(food)));
        }
    }
    for i in 0..PLOTS {
        if inside(layout::plot(i)) {
            return Some(Spot::Place(Place::Plot(i)));
        }
    }
    [
        (layout::NEST, Spot::Place(Place::Nest)),
        (layout::PAIL, Spot::Place(Place::Pail)),
        (layout::BUCKET, Spot::Place(Place::Bucket)),
        (layout::COMPOST, Spot::Place(Place::Compost)),
        (layout::HEN, Spot::Hen),
        (layout::COW, Spot::Chore(Chore::Milk)),
        (layout::WELL, Spot::Chore(Chore::Crank)),
        (layout::STUMP, Spot::Chore(Chore::Axe)),
    ]
    .into_iter()
    .find(|(r, _)| inside(*r))
    .map(|(_, s)| s)
}

// Cues: what the day says happened, as sound and motion.
impl Game {
    fn react(&mut self) {
        for cue in self.day.drain() {
            match cue {
                Cue::Station(id, event) => self.station_event(id, event),
                Cue::Moved { from, to, food } => {
                    self.bump(Spot::Place(to));
                    match (from, to) {
                        (Place::Plot(_), _) => self.audio.play(Sfx::Pluck),
                        (_, Place::Compost) => {
                            self.audio.play(Sfx::Compost);
                            self.fx
                                .dust(layout::mid(layout::COMPOST), 8, pen::ink::SOIL);
                        }
                        (_, Place::Seat(_) | Place::Plot(_)) | (Place::Market(_), _) => {}
                        _ => self.audio.play(Sfx::Set),
                    }
                    if let Place::Plot(i) = from {
                        self.fx
                            .dust(layout::mid(layout::plot(i)), 6, pen::ink::SOIL);
                    }
                    let _ = food;
                }
                Cue::Refused(place, _) => {
                    self.audio.play(Sfx::Nope);
                    self.bump(Spot::Place(place));
                }
                Cue::Balked(chore, refusal) => {
                    if chore == Chore::Milk && refusal == Refusal::Nothing {
                        self.audio.play(Sfx::Moo);
                    } else {
                        self.audio.play(Sfx::Nope);
                    }
                    self.bump(Spot::Chore(chore));
                }
                Cue::Ripe(i) => {
                    self.audio.play(Sfx::Ripe);
                    self.fx.sparkles(layout::mid(layout::plot(i)), 6);
                    self.bump(Spot::Place(Place::Plot(i)));
                }
                Cue::Planted { plot, rich } => {
                    self.audio.play(Sfx::Plant);
                    let at = layout::mid(layout::plot(plot));
                    self.fx.dust(at, 10, pen::ink::SOIL);
                    if rich {
                        self.fx.sparkles(at, 5);
                    }
                }
                Cue::Laid => {
                    self.audio.play(Sfx::Cluck);
                    self.bump(Spot::Hen);
                    self.bump(Spot::Place(Place::Nest));
                }
                Cue::Chore { chore, done } => self.chore_cue(chore, done),
                Cue::Lit | Cue::Log => {
                    self.audio.play(Sfx::Ignite);
                    self.fx.embers(layout::mid(layout::FIREBOX), 14);
                }
                Cue::Cleared(_) | Cue::Out => {}
                Cue::Arrived(seat) => {
                    self.arrived[seat] = 0.0;
                    self.audio.later(0.25, Sfx::Bell);
                }
                Cue::Impatient(seat) => {
                    self.audio.play(Sfx::Tap);
                    self.bump(Spot::Place(Place::Seat(seat)));
                }
                Cue::Served { seat, coins, tip } => self.served(seat, coins, tip),
                Cue::Left(seat) => {
                    if let Some(c) = self.last_seen(seat) {
                        self.leaving.push(Leaving {
                            customer: c,
                            seat,
                            happy: false,
                            age: 0.0,
                        });
                    }
                    if !self.day.is_over() {
                        self.audio.play(Sfx::Huff);
                    }
                }
                Cue::Bought(food, _) => {
                    self.audio.play(Sfx::Buy);
                    let i = game_prototypes::kitchen_garden::day::MARKET
                        .iter()
                        .position(|&f| f == food)
                        .unwrap_or(0);
                    self.fx.coin_burst(layout::mid(layout::ware(i)), 3);
                }
                Cue::Dusk => {
                    self.audio.play(Sfx::Dusk);
                    for mark in 0..self.day.marks() {
                        self.audio
                            .later((mark as f32).mul_add(0.45, 1.4), Sfx::Mark(mark as u8));
                    }
                }
            }
        }
        self.remember_seats();
    }

    fn station_event(&mut self, id: StationId, event: cooking::Event) {
        let rect = layout::station(id);
        let at = layout::mid(rect);
        match event {
            cooking::Event::Began(process, _) => {
                self.bump(Spot::Place(Place::Station(id)));
                let sfx = match process {
                    Process::Boil => Some(Sfx::Bubble),
                    Process::Fry => Some(Sfx::Sizzle),
                    Process::Bake => Some(Sfx::Oven),
                    Process::Prove | Process::Soak | Process::Steep => Some(Sfx::Lid),
                    Process::Chop
                    | Process::Grind
                    | Process::Churn
                    | Process::Knead
                    | Process::Mix => None,
                };
                if let Some(sfx) = sfx {
                    self.audio.play(sfx);
                }
            }
            cooking::Event::Stroke { .. } => {
                let process = match self.day.station(id).state() {
                    cooking::State::Working { process, .. }
                    | cooking::State::Ready { process, .. } => Some(process),
                    cooking::State::Idle => None,
                };
                let (sfx, ink) = match process {
                    Some(Process::Grind) => (Sfx::Grind, pen::ink::FLOUR),
                    Some(Process::Knead) => (Sfx::Knead, pen::ink::DOUGH),
                    Some(Process::Mix) => (Sfx::Mix, pen::ink::CREAM),
                    Some(Process::Churn) => (Sfx::Churn, pen::ink::MILK),
                    _ => (Sfx::Chop, pen::ink::WOOD_LIGHT),
                };
                self.audio.play(sfx);
                self.fx.dust(at + vec2(0.0, 6.0), 3, ink);
            }
            cooking::Event::Done(food) => {
                if food == Food::Mush {
                    self.audio.play(Sfx::Mush);
                } else {
                    self.audio.play(Sfx::Done);
                    self.fx.sparkles(at, 8);
                }
                self.bump(Spot::Place(Place::Station(id)));
            }
            cooking::Event::Smoking => {
                self.audio.play(Sfx::Smoking);
                self.bump(Spot::Place(Place::Station(id)));
            }
            cooking::Event::Burnt => {
                self.audio.play(Sfx::Burnt);
                self.fx.smoke(at, 12);
                self.fx.shake(0.35);
            }
            cooking::Event::Put(_) | cooking::Event::Took(_) | cooking::Event::Aborted => {}
        }
    }

    fn chore_cue(&mut self, chore: Chore, done: bool) {
        let (rect, sfx, finish, ink) = match chore {
            Chore::Milk => (layout::PAIL, Sfx::Squirt, None, pen::ink::MILK),
            Chore::Crank => (layout::WELL, Sfx::Crank, Some(Sfx::Splash), pen::ink::WATER),
            Chore::Axe => (
                layout::STUMP,
                Sfx::Axe,
                Some(Sfx::Split),
                pen::ink::WOOD_LIGHT,
            ),
        };
        self.audio.play(sfx);
        self.fx.dust(layout::mid(rect), 3, ink);
        if done {
            if let Some(finish) = finish {
                self.audio.later(0.06, finish);
            }
            self.fx.sparkles(layout::mid(rect), 4);
            match chore {
                Chore::Milk => self.bump(Spot::Place(Place::Pail)),
                Chore::Crank => self.bump(Spot::Place(Place::Bucket)),
                Chore::Axe => self.fx.chips(layout::mid(layout::LOGS), 6),
            }
        }
    }

    fn served(&mut self, seat: usize, coins: u32, tip: bool) {
        self.audio.play(Sfx::Coins);
        if tip {
            self.audio.later(0.18, Sfx::Tip);
        }
        let at = layout::mid(layout::seat(seat));
        if let Some(c) = self.last_seen(seat) {
            self.served.push(c.wants);
            self.leaving.push(Leaving {
                customer: c,
                seat,
                happy: true,
                age: 0.0,
            });
        }
        self.fx.hearts(at, if tip { 5 } else { 3 });
        self.fx
            .coins_to(at, layout::mid(layout::JAR), coins as usize);
    }

    /// Customers as they were before this round of cues, so one who has
    /// just left can still be drawn leaving.
    const fn last_seen(&self, seat: usize) -> Option<Customer> {
        self.seen[seat]
    }

    fn remember_seats(&mut self) {
        for (seat, c) in self.day.seats().iter().enumerate() {
            if c.is_some() {
                self.seen[seat] = *c;
            }
        }
    }
}

/// The menu's picture of Kitchen Garden.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    draw::emblem(frame, centre, width, clock);
}
