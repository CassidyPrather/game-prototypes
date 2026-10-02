//! Sand Nomad's frontend: the mouse in, pixels and sound out.
//!
//! Everything that decides anything lives in the library's
//! `sand_nomad::journey`. This file turns the pointer into its commands —
//! picking things up, dropping them on pans, waystones, pyres and back into
//! the hold — and turns the journey's cues into sound and motion. Drawing
//! is in [`draw`], onto the low-resolution canvas in [`paint`].
//!
//! No text. What a thing weighs is a rune of six dots; a trader's feelings
//! are the motive emotes; days are tally marks; burden is a jar filling
//! with grains; the circuit is a row of standing stones.

mod audio;
mod draw;
mod paint;

use std::collections::HashMap;

use audio::Audio;
use game_prototypes::sand_nomad::barter::Emote;
use game_prototypes::sand_nomad::item::{Item, ItemId};
use game_prototypes::sand_nomad::journey::{Cue, Journey, Phase, Refusal};
use game_prototypes::sand_nomad::sfx::{Sfx, Stuff};
use game_prototypes::sand_nomad::world::{HOLD_SIZE, Site};
use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position,
};
use macroquad::math::{Rect, Vec2, vec2};
use paint::{Art, Canvas};

use crate::ui::{Frame, MqVec2};

/// Where everything sits on the canvas. Drawing and hit-testing both read
/// these, so what you see is what you can grab.
pub mod layout {
    use macroquad::math::{Rect, Vec2, vec2};

    /// The top of the deck along the bottom, where the hold lives.
    pub const HUD_Y: f32 = 228.0;
    /// One hold cell.
    pub const CELL: f32 = 16.0;
    /// Top left of the hold.
    pub const HOLD: Vec2 = vec2(8.0, 233.0);
    /// Top left of a trader's rug.
    pub const RUG: Vec2 = vec2(290.0, 150.0);
    /// Where the scale's beam turns.
    pub const PIVOT: Vec2 = vec2(178.0, 96.0);
    /// Half the beam.
    pub const BEAM_HALF: f32 = 52.0;
    /// From a beam end down to its pan's plate.
    pub const PAN_DROP: f32 = 56.0;
    /// A pan's plate.
    pub const PAN_W: f32 = 52.0;
    /// The waystone.
    pub const WAYSTONE: Rect = Rect::new(14.0, 116.0, 16.0, 32.0);
    /// The first of a place's well, pyre or bench; a second stands beside.
    pub const SERVICE: Rect = Rect::new(36.0, 116.0, 32.0, 32.0);
    /// How far along the second service stands.
    pub const SERVICE_STEP: f32 = 36.0;
    /// The loupe.
    pub const LOUPE: Rect = Rect::new(144.0, 204.0, 16.0, 16.0);
    /// The handshake.
    pub const HANDS: Rect = Rect::new(196.0, 204.0, 16.0, 16.0);
    /// The trader.
    pub const TRADER: Rect = Rect::new(322.0, 112.0, 32.0, 32.0);
    /// Casting off, back to the map.
    pub const SAIL: Rect = Rect::new(372.0, 268.0, 20.0, 24.0);
    /// The sand pit: leave a weightless thing behind.
    pub const PIT: Rect = Rect::new(344.0, 272.0, 20.0, 20.0);
    /// The burden jar.
    pub const JAR: Rect = Rect::new(144.0, 232.0, 18.0, 64.0);
    /// The mute toggle.
    pub const SPEAKER: Rect = Rect::new(380.0, 231.0, 16.0, 16.0);
    /// First circuit stone.
    pub const STONES: Vec2 = vec2(170.0, 234.0);
    /// Spacing of the circuit stones.
    pub const STONE_STEP: f32 = 17.0;
    /// Top left of the day tally.
    pub const TALLY: Vec2 = vec2(170.0, 262.0);
    /// The key cap to start again, once home.
    pub const AGAIN: Rect = Rect::new(192.0, 102.0, 16.0, 16.0);
    /// How close to a place on the map counts as pointing at it.
    pub const PLACE_REACH: f32 = 15.0;

    /// The hold, whole.
    pub const fn hold_rect(cols: u8, rows: u8) -> Rect {
        Rect::new(HOLD.x, HOLD.y, cols as f32 * CELL, rows as f32 * CELL)
    }

    /// The rug, whole.
    pub const fn rug_rect(cols: u8, rows: u8) -> Rect {
        Rect::new(RUG.x, RUG.y, cols as f32 * CELL, rows as f32 * CELL)
    }
}

/// What the pointer is over.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Spot {
    /// A thing in the hold.
    Held(ItemId),
    /// An empty hold cell.
    Cell(i32, i32),
    /// A thing on the rug.
    Rugged(ItemId),
    /// The rug's empty sand.
    Rug,
    /// A thing on one of the pans; `true` for yours.
    Panned(ItemId, bool),
    /// A pan; `true` for yours.
    Pan(bool),
    Waystone,
    Service(Service),
    Loupe,
    Hands,
    Trader,
    Sail,
    Pit,
    Speaker,
    /// A place on the map.
    Place(Site),
}

/// What a place offers besides trade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    Well,
    Pyre,
    Bench,
}

/// The services at `site`, and where each stands in camp.
pub fn services(site: Site) -> Vec<(Service, Rect)> {
    let mut out = Vec::new();
    for (here, service) in [
        (site.well(), Service::Well),
        (site.pyre(), Service::Pyre),
        (site.bench(), Service::Bench),
    ] {
        if here {
            let mut r = layout::SERVICE;
            r.x = (out.len() as f32).mul_add(layout::SERVICE_STEP, r.x);
            out.push((service, r));
        }
    }
    out
}

/// Where a thing being carried came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum From {
    Hold,
    Rug,
    MyPan,
    TheirPan,
}

/// A thing on the pointer.
#[derive(Clone, Copy, Debug)]
struct Carried {
    id: ItemId,
    from: From,
    /// Pointer offset from the thing's top left.
    grab: Vec2,
    turned: bool,
}

/// The pointer.
#[derive(Default)]
struct Hand {
    pos: Vec2,
    /// Where and on what the button went down.
    press: Option<(Vec2, Option<Spot>)>,
    carried: Option<Carried>,
    /// Clicking things reads them, while the loupe is up.
    loupe: bool,
    hover: Option<Spot>,
    /// Seconds the pointer has rested on the current hover.
    rest: f32,
}

/// A mote of dust or a grain of burden.
#[derive(Clone, Copy, Debug)]
struct Mote {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    life: f32,
    ink: char,
    /// Where a grain of burden is flying: it arcs from where it started to
    /// here instead of falling.
    to: Option<(Vec2, Vec2)>,
    /// Seconds before it sets off.
    delay: f32,
}

/// Motion the cues set going.
#[derive(Default)]
struct Fx {
    /// The beam's angle, radians, positive with your side down, and its
    /// swing.
    beam: f32,
    beam_vel: f32,
    /// The trader's face and how long it has shown.
    face: Option<(Emote, f32)>,
    /// Things hopping, seconds left.
    hops: HashMap<ItemId, f32>,
    /// Runes flashing a new node, seconds left.
    flashes: HashMap<ItemId, f32>,
    /// Things shaking no, seconds left.
    shakes: HashMap<ItemId, f32>,
    /// Things being read, and how far through.
    reading: Option<(ItemId, f32)>,
    motes: Vec<Mote>,
    /// Night falling at dawn, `1` and fading.
    night: f32,
    /// The waystone glowing after an offering.
    stone_glow: f32,
    /// The pyre flaring after a burning.
    flare: f32,
    /// The whole screen flashing no.
    refused: f32,
    /// A waystone hint: things in the hold that would do, lit.
    hint: f32,
    /// What the scale read last, to creak when it changes.
    reading_was: Option<i32>,
    /// The last refusal, and how long its hint has left.
    no: Option<(Refusal, f32)>,
    /// Where each thing is drawn, easing toward where it is, so that
    /// nothing teleports: drops settle, pans fill, deals fly.
    shown: HashMap<ItemId, Vec2>,
}

/// Which view the top of the screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Map,
    Camp,
}

/// A run of Sand Nomad.
pub struct Game {
    journey: Journey,
    art: Art,
    canvas: Canvas,
    audio: Audio,
    screen: Screen,
    hand: Hand,
    fx: Fx,
    clock: f32,
    /// Whether the title is still up, waiting on a first click.
    title: bool,
    /// Seconds since the journey ended.
    ended: f32,
}

impl Game {
    pub async fn load() -> Self {
        Self {
            journey: Journey::new(),
            art: Art::bake(),
            canvas: Canvas::new(),
            audio: Audio::load().await,
            screen: Screen::Camp,
            hand: Hand::default(),
            fx: Fx::default(),
            clock: 0.0,
            title: true,
            ended: 0.0,
        }
    }

    fn restart(&mut self) {
        self.journey = Journey::new();
        self.screen = Screen::Camp;
        self.hand = Hand::default();
        self.fx = Fx::default();
        self.ended = 0.0;
        self.audio.play(Sfx::Moor);
    }
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        self.clock += dt;
        self.audio.update(dt);
        if is_key_pressed(KeyCode::M) {
            self.audio.wake();
            self.audio.toggle_mute();
        }
        self.pointer(dt);
        if self.journey.phase() == Phase::Sailing {
            // Held down, over the map, the wind picks up.
            let over_map = self.hand.pos.y < layout::HUD_Y && self.hand.carried.is_none();
            let hurry = if is_key_down(KeyCode::Space)
                || (over_map && is_mouse_button_down(MouseButton::Left))
            {
                3.0
            } else {
                1.0
            };
            self.journey.advance(dt * hurry);
            self.react();
        }
        if self.journey.phase() == Phase::Home {
            self.ended += dt;
            if is_key_pressed(KeyCode::R) {
                self.restart();
            }
        }
        let wind = match (self.journey.phase(), self.screen) {
            (Phase::Sailing, _) => 1.0,
            (_, Screen::Map) => 0.6,
            _ => 0.35,
        };
        self.audio.wind(wind);
        self.animate(dt);
    }

    fn draw(&self, frame: &Frame) {
        self.canvas.begin();
        draw::scene(self);
        self.canvas.end(frame);
    }

    fn leave(&mut self) {
        self.audio.hush();
    }
}

// The pointer: pressing, carrying, dropping, clicking.
impl Game {
    fn pointer(&mut self, dt: f32) {
        let (mx, my) = mouse_position();
        let frame = Frame::fit(
            macroquad::window::screen_width(),
            macroquad::window::screen_height(),
        );
        let pos = paint::to_canvas(&frame, MqVec2::new(mx, my));
        self.hand.pos = pos;
        let hover = self.spot_at(pos);
        if hover == self.hand.hover {
            self.hand.rest += dt;
        } else {
            self.hand.rest = 0.0;
            if let Some(Spot::Held(_) | Spot::Rugged(_) | Spot::Panned(..) | Spot::Place(_)) = hover
            {
                if self.hand.carried.is_none() {
                    self.audio.play(Sfx::Tick);
                }
            }
        }
        self.hand.hover = hover;

        if self.title {
            if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) {
                self.title = false;
                self.audio.wake();
                self.audio.play(Sfx::Moor);
            }
            return;
        }
        if self.journey.phase() == Phase::Home {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.audio.wake();
                if self.ended > 1.5 && layout::AGAIN.contains(pos) {
                    self.restart();
                }
            }
            return;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            self.audio.wake();
            self.hand.press = Some((pos, hover));
        }
        if let Some(carried) = self.hand.carried.as_mut() {
            if is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::Space) {
                let turnable = self
                    .journey
                    .find(carried.id)
                    .is_some_and(|p| p.item.can_turn());
                if turnable {
                    carried.turned = !carried.turned;
                    carried.grab = vec2(carried.grab.y, carried.grab.x);
                    self.audio.play(Sfx::Turn);
                }
            }
        }
        if is_mouse_button_down(MouseButton::Left) && self.hand.carried.is_none() {
            if let Some((at, Some(spot))) = self.hand.press {
                if at.distance(pos) > 3.0 {
                    self.pick_up(spot, at);
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(carried) = self.hand.carried.take() {
                self.put_down(carried, hover);
            } else if let Some((_, Some(spot))) = self.hand.press {
                if Some(spot) == hover {
                    self.click(spot);
                }
            } else if self.hand.loupe {
                self.hand.loupe = false;
            }
            self.hand.press = None;
        }
    }

    /// Start carrying whatever `spot` is, if it can be carried.
    fn pick_up(&mut self, spot: Spot, at: Vec2) {
        let (id, from) = match spot {
            Spot::Held(id) => (id, From::Hold),
            Spot::Rugged(id) => (id, From::Rug),
            Spot::Panned(id, mine) => (id, if mine { From::MyPan } else { From::TheirPan }),
            _ => return,
        };
        let Some(placed) = self.journey.find(id).copied() else {
            return;
        };
        let top_left = match from {
            From::Hold | From::Rug => draw::cell_origin(from == From::Hold, placed.x, placed.y),
            From::MyPan | From::TheirPan => self
                .pan_layout(from == From::MyPan)
                .into_iter()
                .find(|(i, _)| *i == id)
                .map_or(at, |(_, r)| r.point()),
        };
        let turned = matches!(from, From::Hold | From::Rug) && placed.turned;
        self.hand.carried = Some(Carried {
            id,
            from,
            grab: at - top_left,
            turned,
        });
        self.hand.loupe = false;
        self.audio
            .play(Sfx::Lift(Stuff::of(placed.item.kind.material())));
    }

    /// Let go of a carried thing over `target`.
    fn put_down(&mut self, carried: Carried, target: Option<Spot>) {
        let id = carried.id;
        let result = match (carried.from, target) {
            (From::Hold, Some(Spot::Held(_) | Spot::Cell(..)) | None)
                if layout::hold_rect(HOLD_SIZE.0, HOLD_SIZE.1).contains(self.hand.pos) =>
            {
                let corner =
                    self.hand.pos - carried.grab + vec2(layout::CELL / 2.0, layout::CELL / 2.0);
                let cell = ((corner - layout::HOLD) / layout::CELL).floor();
                // Cells are a handful, so the casts are exact.
                #[allow(clippy::cast_possible_truncation)]
                let r = self
                    .journey
                    .arrange(id, cell.x as i32, cell.y as i32, carried.turned);
                r
            }
            (
                From::Hold,
                Some(
                    Spot::Pan(_)
                    | Spot::Panned(..)
                    | Spot::Trader
                    | Spot::Hands
                    | Spot::Rug
                    | Spot::Rugged(_),
                ),
            ) => self.journey.offer(id),
            (From::Hold, Some(Spot::Waystone)) => self.journey.give(id),
            (From::Hold, Some(Spot::Service(Service::Pyre))) => self.journey.burn(id),
            (From::Hold, Some(Spot::Service(Service::Bench))) => self.journey.renew(id),
            (From::Hold | From::Rug, Some(Spot::Loupe)) => self.journey.appraise(id),
            (From::Hold, Some(Spot::Pit)) => self.journey.drop_item(id),
            (
                From::Rug,
                Some(
                    Spot::Pan(_) | Spot::Panned(..) | Spot::Held(_) | Spot::Cell(..) | Spot::Hands,
                ),
            ) => self.journey.ask(id),
            (From::MyPan, Some(Spot::Pan(true) | Spot::Panned(_, true)))
            | (From::TheirPan, Some(Spot::Pan(false) | Spot::Panned(_, false))) => Ok(()),
            (From::MyPan | From::TheirPan, _) => self.journey.withdraw(id),
            (From::Rug, None | Some(Spot::Rug | Spot::Rugged(_))) | (From::Hold, None) => {
                // Put back where it was, no harm done.
                self.set_sound(id);
                self.hop(id);
                return;
            }
            _ => {
                // Somewhere it cannot go.
                self.shake(id);
                self.audio.play(Sfx::Nope);
                return;
            }
        };
        if result.is_err() {
            self.shake(id);
        }
        self.react();
    }

    fn click(&mut self, spot: Spot) {
        let reading = self.hand.loupe;
        match spot {
            Spot::Held(id) | Spot::Rugged(id) | Spot::Panned(id, _) if reading => {
                let _ = self.journey.appraise(id);
                self.react();
            }
            Spot::Held(id) => {
                if self.screen == Screen::Camp && self.journey.trader().is_some() {
                    let _ = self.journey.offer(id);
                    self.react();
                } else {
                    self.hop(id);
                    if let Some(p) = self.journey.find(id) {
                        self.audio.play(Sfx::Set(Stuff::of(p.item.kind.material())));
                    }
                }
            }
            Spot::Rugged(id) => {
                let _ = self.journey.ask(id);
                self.react();
            }
            Spot::Panned(id, _) => {
                let _ = self.journey.withdraw(id);
                self.react();
            }
            Spot::Loupe => {
                self.hand.loupe = !self.hand.loupe;
                self.audio.play(Sfx::Turn);
            }
            Spot::Hands => {
                let _ = self.journey.deal();
                self.react();
            }
            Spot::Service(Service::Well) => {
                let _ = self.journey.fill();
                self.react();
            }
            Spot::Waystone => {
                self.fx.hint = 1.2;
                self.audio.play(Sfx::Tick);
            }
            Spot::Sail => {
                self.journey.clear_pans();
                self.screen = Screen::Map;
                self.audio.play(Sfx::Turn);
            }
            Spot::Speaker => self.audio.toggle_mute(),
            Spot::Place(site) => {
                if site == self.journey.at() {
                    if self.journey.phase() == Phase::Camp {
                        self.screen = Screen::Camp;
                        self.audio.play(Sfx::Moor);
                    }
                } else if self.journey.set_sail(site).is_ok() {
                    self.react();
                }
            }
            _ => {}
        }
        // The loupe stays up while there are things to read; anything else
        // puts it down.
        let read_a_thing = matches!(spot, Spot::Held(_) | Spot::Rugged(_) | Spot::Panned(..));
        if reading && !read_a_thing && !matches!(spot, Spot::Loupe) {
            self.hand.loupe = false;
        }
    }

    /// What is under canvas position `p`.
    fn spot_at(&self, p: Vec2) -> Option<Spot> {
        let hold = self.journey.hold();
        let hold_rect = layout::hold_rect(hold.width(), hold.height());
        if hold_rect.contains(p) {
            let cell = ((p - layout::HOLD) / layout::CELL).floor();
            // Inside the hold, so small and positive.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (cx, cy) = (cell.x as i32, cell.y as i32);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            return Some(match hold.at(cx as u8, cy as u8) {
                Some(placed) if !placed.on_pan => Spot::Held(placed.item.id),
                _ => Spot::Cell(cx, cy),
            });
        }
        if layout::SPEAKER.contains(p) {
            return Some(Spot::Speaker);
        }
        if layout::PIT.contains(p) {
            return Some(Spot::Pit);
        }
        if self.screen == Screen::Camp && layout::SAIL.contains(p) {
            return Some(Spot::Sail);
        }
        if p.y >= layout::HUD_Y {
            return None;
        }
        match self.screen {
            Screen::Map => Site::ALL
                .iter()
                .map(|&s| (s, vec2(s.pos().0, s.pos().1).distance(p)))
                .filter(|(_, d)| *d < layout::PLACE_REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(s, _)| Spot::Place(s)),
            Screen::Camp => self.camp_spot_at(p),
        }
    }

    fn camp_spot_at(&self, p: Vec2) -> Option<Spot> {
        let here = self.journey.at();
        if here.waystone().is_some() && layout::WAYSTONE.contains(p) {
            return Some(Spot::Waystone);
        }
        for (service, r) in services(here) {
            if r.contains(p) {
                return Some(Spot::Service(service));
            }
        }
        self.journey.trader()?;
        for mine in [true, false] {
            for (id, r) in self.pan_layout(mine) {
                if r.contains(p) {
                    return Some(Spot::Panned(id, mine));
                }
            }
            if self.pan_rect(mine).contains(p) {
                return Some(Spot::Pan(mine));
            }
        }
        if layout::LOUPE.contains(p) {
            return Some(Spot::Loupe);
        }
        if layout::HANDS.contains(p) {
            return Some(Spot::Hands);
        }
        if layout::TRADER.contains(p) {
            return Some(Spot::Trader);
        }
        let rug = self.journey.rug();
        if layout::rug_rect(rug.width(), rug.height()).contains(p) {
            let cell = ((p - layout::RUG) / layout::CELL).floor();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            return Some(match rug.at(cell.x as u8, cell.y as u8) {
                Some(placed) if !placed.on_pan => Spot::Rugged(placed.item.id),
                _ => Spot::Rug,
            });
        }
        None
    }

    /// The scale's beam ends, left then right, as it hangs now.
    fn beam_ends(&self) -> (Vec2, Vec2) {
        let (s, c) = self.fx.beam.sin_cos();
        let along = vec2(c, -s) * layout::BEAM_HALF;
        (layout::PIVOT - along, layout::PIVOT + along)
    }

    /// Where a pan's plate is: yours on the left.
    fn pan_rect(&self, mine: bool) -> Rect {
        let (left, right) = self.beam_ends();
        let end = if mine { left } else { right };
        let plate = end + vec2(0.0, layout::PAN_DROP);
        // The pan reaches up over what is piled on it.
        Rect::new(
            plate.x - layout::PAN_W / 2.0,
            plate.y - 34.0,
            layout::PAN_W,
            38.0,
        )
    }

    /// Where each thing on a pan sits, piled from the plate up.
    fn pan_layout(&self, mine: bool) -> Vec<(ItemId, Rect)> {
        let items: Vec<(Item, bool)> = if mine {
            self.journey
                .hold()
                .out()
                .map(|p| (p.item, p.turned))
                .collect()
        } else {
            self.journey
                .rug()
                .out()
                .map(|p| (p.item, p.turned))
                .collect()
        };
        let plate = self.pan_rect(mine);
        let floor = plate.y + plate.h - 4.0;
        let mut out = Vec::new();
        let (mut x, mut y, mut row_h) = (plate.x + 2.0, floor, 0.0_f32);
        for (item, _) in items {
            let (w, h) = item.footprint(false);
            let (w, h) = (f32::from(w) * layout::CELL, f32::from(h) * layout::CELL);
            if x + w > plate.x + plate.w + 4.0 {
                x = plate.x + 2.0 + 6.0;
                y -= row_h.max(8.0) - 4.0;
                row_h = 0.0;
            }
            out.push((item.id, Rect::new(x, y - h, w, h)));
            x += w - 2.0;
            row_h = row_h.max(h * 0.6);
        }
        out
    }
}

// Cues: turning what happened into sound and motion.
impl Game {
    // One arm per cue: a table, which splitting would only scatter.
    #[allow(clippy::too_many_lines)]
    fn react(&mut self) {
        let cues: Vec<Cue> = self.journey.cues().to_vec();
        let mut grains = 0;
        for cue in cues {
            match cue {
                Cue::Watch => grains += 1,
                Cue::Dawn(_) => {
                    self.fx.night = 1.0;
                    self.audio.play(Sfx::Dawn);
                }
                Cue::Grew(g) => {
                    if g.gained > 0 {
                        self.fx.flashes.insert(g.id, 0.8);
                        self.hop(g.id);
                        self.audio.later(0.15, Sfx::Heavier);
                    }
                    if g.woke {
                        self.fx.shakes.insert(g.id, 1.2);
                        self.audio.later(0.3, Sfx::Wake);
                    }
                }
                Cue::Wandered(id) => {
                    self.hop(id);
                    self.audio.later(0.5, Sfx::Skitter);
                }
                Cue::Sip(_) => self.audio.later(0.1, Sfx::Sip),
                Cue::Thirst => {
                    self.fx.refused = 0.5;
                    self.audio.later(0.1, Sfx::Thirst);
                }
                Cue::SetSail(_) => {
                    self.screen = Screen::Map;
                    self.audio.play(Sfx::Sail);
                    self.dust(vec2(self.journey.ship().0, self.journey.ship().1), 8, '8');
                }
                Cue::Arrived(_) => {
                    if self.journey.phase() == Phase::Camp {
                        self.screen = Screen::Camp;
                    }
                    self.audio.play(Sfx::Moor);
                    let (x, y) = self.journey.ship();
                    self.dust(vec2(x, y), 10, '8');
                    self.fx.face = None;
                }
                Cue::Arranged(id) => {
                    self.hop(id);
                    self.set_sound(id);
                    self.dust_at_item(id);
                }
                Cue::Swapped(a, b) => {
                    self.hop(a);
                    self.hop(b);
                    self.set_sound(a);
                    self.audio.later(0.07, Sfx::Turn);
                    self.dust_at_item(a);
                }
                Cue::Panned(_) => self.audio.play(Sfx::Pan),
                Cue::Unpanned(id) => self.set_sound(id),
                Cue::Face(emote) => {
                    self.fx.face = Some((emote, 0.0));
                    self.audio.later(0.12, Sfx::Face(emote));
                }
                Cue::Deal => {
                    self.audio.play(Sfx::Deal);
                    self.dust(layout::HANDS.center(), 12, '6');
                }
                Cue::Appraised(id) => {
                    self.fx.reading = Some((id, 0.0));
                    self.audio.play(Sfx::Ponder);
                    if let Some(p) = self.journey.find(id) {
                        for i in 0..p.item.weight.min(6) {
                            self.audio
                                .later(f32::from(i).mul_add(0.14, 0.7), Sfx::Dot(i));
                        }
                    }
                }
                Cue::Offered(site) => {
                    self.fx.stone_glow = 2.0;
                    if let Some(m) = site.waystone() {
                        self.audio.play(Sfx::Stone(m));
                    }
                    self.dust(layout::WAYSTONE.center(), 14, 'F');
                }
                Cue::Burned(_) => {
                    self.fx.flare = 1.5;
                    self.audio.play(Sfx::Burn);
                    let at = services(self.journey.at())
                        .into_iter()
                        .find(|(s, _)| *s == Service::Pyre)
                        .map_or(layout::SERVICE.center(), |(_, r)| r.center());
                    self.dust(at, 16, 'R');
                }
                Cue::Renewed(id) => {
                    self.hop(id);
                    self.audio.play(Sfx::Hammer);
                    self.fx.flashes.insert(id, 0.8);
                }
                Cue::Filled => self.audio.play(Sfx::Pour),
                Cue::Dropped(_) => {
                    self.audio.play(Sfx::Drop);
                    self.dust(layout::PIT.center(), 10, '8');
                }
                Cue::No(why) => {
                    self.fx.no = Some((why, 0.9));
                    let sound = match why {
                        Refusal::WrongMotive
                        | Refusal::Weightless
                        | Refusal::Awake
                        | Refusal::Given => Sfx::Clunk,
                        _ => Sfx::Nope,
                    };
                    self.audio.play(sound);
                    if matches!(why, Refusal::Short | Refusal::NoRoom | Refusal::Refused) {
                        self.fx.refused = 0.4;
                    }
                }
                Cue::CircuitWalked => self.audio.later(1.2, Sfx::Circuit),
                Cue::Finished => self.audio.later(0.6, Sfx::Home),
                Cue::Heard(h) => {
                    use game_prototypes::sand_nomad::history::Happening;
                    let sound = match h {
                        Happening::Raid => Sfx::Guns,
                        Happening::Tower => Sfx::Semaphore,
                        Happening::Fireworks => Sfx::Fireworks,
                        Happening::Dirigible => Sfx::Drone,
                    };
                    self.audio.later(0.4, sound);
                }
            }
        }
        if grains > 0 {
            self.pour_grains(grains);
        }
    }

    fn set_sound(&self, id: ItemId) {
        if let Some(p) = self.journey.find(id) {
            self.audio.play(Sfx::Set(Stuff::of(p.item.kind.material())));
        }
    }

    fn hop(&mut self, id: ItemId) {
        self.fx.hops.insert(id, 0.25);
    }

    fn shake(&mut self, id: ItemId) {
        self.fx.shakes.insert(id, 0.35);
    }

    fn dust(&mut self, at: Vec2, count: usize, ink: char) {
        for i in 0..count {
            let a = self.clock.mul_add(7.0, i as f32 * 2.399);
            let speed = ((i % 5) as f32).mul_add(6.0, 12.0);
            self.fx.motes.push(Mote {
                pos: at,
                vel: vec2(a.cos() * speed, a.sin().mul_add(speed, -18.0)),
                age: 0.0,
                life: ((i % 3) as f32).mul_add(0.15, 0.4),
                ink,
                to: None,
                delay: 0.0,
            });
        }
    }

    fn dust_at_item(&mut self, id: ItemId) {
        if let Some(p) = self.journey.hold().get(id) {
            let (w, h) = p.size();
            let o = draw::cell_origin(true, p.x, p.y);
            let at = o + vec2(f32::from(w) * 8.0, f32::from(h) * 16.0);
            self.dust(at, 5, '9');
        }
    }

    /// Grains of burden: every watch, each thing with weight sheds a grain
    /// a point, and they arc from where it sits in the hold into the jar.
    /// You can see what is weighing on you.
    fn pour_grains(&mut self, watches: u32) {
        let mouth = vec2(layout::JAR.x + layout::JAR.w / 2.0, layout::JAR.y + 2.0);
        let mut sources = Vec::new();
        for p in self.journey.hold().placed() {
            let burden = p.item.burden();
            if burden == 0 {
                continue;
            }
            let (w, h) = p.size();
            let o =
                draw::cell_origin(true, p.x, p.y) + vec2(f32::from(w) * 8.0, f32::from(h) * 8.0);
            // A handful of grains, not one per point, or a heavy hold
            // would snow.
            for k in 0..burden.min(6) {
                sources.push((o, k, p.item.is_anima()));
            }
        }
        let spread = watches.clamp(1, 4) as f32 * 0.25;
        let count = sources.len().max(1) as f32;
        for (n, (from, k, alive)) in sources.into_iter().enumerate() {
            self.fx.motes.push(Mote {
                pos: from,
                vel: Vec2::ZERO,
                age: 0.0,
                life: 0.7,
                ink: if alive {
                    'E'
                } else if k % 2 == 0 {
                    '3'
                } else {
                    '9'
                },
                to: Some((from + vec2((k as f32 - 2.0) * 1.5, 0.0), mouth)),
                delay: n as f32 / count * spread,
            });
        }
    }

    /// Where every visible thing belongs right now, top left.
    fn item_targets(&self) -> Vec<(ItemId, Vec2)> {
        let mut out: Vec<(ItemId, Vec2)> = self
            .journey
            .hold()
            .present()
            .map(|p| (p.item.id, draw::cell_origin(true, p.x, p.y)))
            .collect();
        if self.screen == Screen::Camp && self.journey.trader().is_some() {
            out.extend(
                self.journey
                    .rug()
                    .present()
                    .map(|p| (p.item.id, draw::cell_origin(false, p.x, p.y))),
            );
            for mine in [true, false] {
                out.extend(
                    self.pan_layout(mine)
                        .into_iter()
                        .map(|(id, r)| (id, r.point())),
                );
            }
        }
        out
    }

    fn animate(&mut self, dt: f32) {
        let ease = 1.0 - (-24.0 * dt).exp();
        let targets = self.item_targets();
        let mut shown = HashMap::with_capacity(targets.len());
        for (id, target) in targets {
            let at = self.fx.shown.get(&id).map_or(target, |&was| {
                let next = was.lerp(target, ease);
                if next.distance(target) < 0.5 {
                    target
                } else {
                    next
                }
            });
            shown.insert(id, at);
        }
        if let Some(c) = self.hand.carried {
            shown.insert(c.id, self.hand.pos - c.grab);
        }
        self.fx.shown = shown;

        // The beam swings toward what the scale says, and overshoots a bit.
        let target = match (self.screen, self.journey.balance()) {
            (Screen::Camp, Some(b)) => (b as f32).clamp(-8.0, 8.0) * 0.035,
            (Screen::Camp, None) if self.journey.trader().is_some() => {
                (self.clock * 30.0).sin() * 0.05
            }
            _ => 0.0,
        };
        self.fx.beam_vel = ((target - self.fx.beam) * 60.0).mul_add(dt, self.fx.beam_vel);
        self.fx.beam_vel *= (-5.0 * dt).exp();
        self.fx.beam = self.fx.beam_vel.mul_add(dt, self.fx.beam);
        // The beam creaks whenever the reading changes.
        let reading = self.journey.balance().map(|b| b.clamp(-8, 8));
        if self.screen == Screen::Camp && reading != self.fx.reading_was {
            if self.fx.reading_was.is_some() || reading.is_some() {
                self.audio.later(0.08, Sfx::Beam);
            }
            self.fx.reading_was = reading;
        }
        if let Some((_, t)) = self.fx.no.as_mut() {
            *t -= dt;
        }
        if self.fx.no.is_some_and(|(_, t)| t <= 0.0) {
            self.fx.no = None;
        }
        for map in [&mut self.fx.hops, &mut self.fx.flashes, &mut self.fx.shakes] {
            map.retain(|_, t| {
                *t -= dt;
                *t > 0.0
            });
        }
        if let Some((_, age)) = self.fx.face.as_mut() {
            *age += dt;
        }
        if self.fx.face.is_some_and(|(_, age)| age > 2.2) {
            self.fx.face = None;
        }
        if let Some((_, t)) = self.fx.reading.as_mut() {
            *t += dt;
        }
        if self.fx.reading.is_some_and(|(_, t)| t > 1.8) {
            self.fx.reading = None;
        }
        for mote in &mut self.fx.motes {
            mote.age += dt;
            mote.pos += mote.vel * dt;
            mote.vel.y = 60.0f32.mul_add(dt, mote.vel.y);
        }
        self.fx.motes.retain(|m| m.age < m.life);
        self.fx.night = 2.5f32.mul_add(-dt, self.fx.night).max(0.0);
        for t in [
            &mut self.fx.stone_glow,
            &mut self.fx.flare,
            &mut self.fx.refused,
            &mut self.fx.hint,
        ] {
            *t = (*t - dt).max(0.0);
        }
    }
}

/// Draw the prototype's mark for the menu: a sand ship under a motive
/// rune, sailing over dunes.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    draw::emblem(frame, centre, width, clock);
}
