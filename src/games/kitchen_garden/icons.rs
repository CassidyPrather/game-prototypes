//! A picture for every food and process, drawn with the [`Pen`].
//!
//! Each icon is laid out in a 32x32 box of local units centred on the
//! origin (y down), then scaled to whatever size the caller asks for. Every
//! icon is drawn twice: once as a silhouette grown by the outline width in
//! [`ink::LINE`], then in colour on top. Outlines come for free that way,
//! stay one width however a picture is built, and never cut through the
//! inside of a shape.

// Step counts are small positive numbers made from spans.
#![allow(clippy::cast_sign_loss)]

use std::f32::consts::PI;

use game_prototypes::cooking::{Food, Kind, Process};
use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};

use super::pen::{Pen, alpha, dark, hex, ink, light, mix};

/// Outline width, in local units.
const OUTLINE: f32 = 1.05;

// Colours the palette doesn't have.
const SKIN: Color = hex(0xf2_c29b);
const SLEEVE: Color = hex(0x6f_9fc4);
const BURLAP: Color = hex(0xd6_b47c);
const GLASS: Color = hex(0xdd_ecf2);
const BATTER: Color = hex(0xf8_e29a);
const BOWL_BLUE: Color = hex(0x7d_a8c8);
const BOWL_PINK: Color = hex(0xe3_9aa6);
const BOWL_TEAL: Color = hex(0x6c_b0a4);
const PLATE: Color = hex(0xe2_ecf2);
const MILK_TEA: Color = hex(0xd3_a271);
const OMELETTE: Color = hex(0xf9_d04e);
const PANCAKE: Color = hex(0xe0_9a40);
const PANCAKE_SIDE: Color = hex(0xf4_d595);
const SYRUP: Color = hex(0xb2_5a1a);
const SPONGE: Color = hex(0xf1_c47e);
const FLATBREAD: Color = hex(0xea_c38a);
const CINNAMON: Color = hex(0x9c_5a30);
const FRIED_RICE: Color = hex(0xf0_c560);
const PEA: Color = hex(0x6c_b84a);
const CARAMEL: Color = hex(0xc4_7428);
const PUDDING: Color = hex(0xf8_ecd4);
const FLESH: Color = hex(0xf2_6a55);
const GEL: Color = hex(0xf8_9a7e);
const SEED: Color = hex(0xfa_e0a0);
const ONION_WHITE: Color = hex(0xf7_e9f2);

/// Draw `food` centred on `c`, fitting a `size` x `size` box (frame units).
pub fn food(pen: &Pen, food: Food, c: Vec2, size: f32) {
    let s = Sketch::new(pen, c, size);
    draw_food(&s.traced(), food);
    draw_food(&s, food);
}

/// Draw a small mark for a process (fits `size` box).
pub fn process(pen: &Pen, process: Process, c: Vec2, size: f32) {
    let s = Sketch::new(pen, c, size);
    draw_process(&s.traced(), process);
    draw_process(&s, process);
}

/// Draw a mark for a kind of process: Physical = a hand/fist, Heat = a
/// flame, Time = an hourglass.
pub fn kind(pen: &Pen, kind: Kind, c: Vec2, size: f32) {
    let s = Sketch::new(pen, c, size);
    draw_kind(&s.traced(), kind);
    draw_kind(&s, kind);
}

fn draw_food(s: &Sketch, food: Food) {
    match food {
        Food::Water => water(s),
        Food::Milk => milk(s),
        Food::Egg => egg(s),
        Food::Carrot => carrot(s),
        Food::Onion => onion(s),
        Food::Tomato => tomato(s),
        Food::Wheat => wheat(s),
        Food::Rice => rice(s),
        Food::Beans => beans(s),
        Food::Sugar => sugar(s),
        Food::TeaLeaf => tea_leaf(s),
        Food::Spice => spice(s),
        Food::Salt => salt(s),
        Food::Flour => flour(s),
        Food::Dough => dough(s),
        Food::RisenDough => risen_dough(s),
        Food::Batter => batter(s),
        Food::Butter => butter(s),
        Food::HotWater => hot_water(s),
        Food::ChoppedCarrot => chopped_carrot(s),
        Food::ChoppedOnion => chopped_onion(s),
        Food::ChoppedTomato => chopped_tomato(s),
        Food::SoakedBeans => soaked_beans(s),
        Food::Tea => tea(s, false),
        Food::MilkTea => tea(s, true),
        Food::Salad => salad(s),
        Food::BoiledEgg => boiled_egg(s),
        Food::SteamedRice => steamed_rice(s),
        Food::Soup => soup(s),
        Food::BeanStew => bean_stew(s),
        Food::Custard => custard(s),
        Food::RicePudding => rice_pudding(s),
        Food::FriedEgg => fried_egg(s),
        Food::Omelette => omelette(s),
        Food::Pancake => pancake(s),
        Food::FriedRice => fried_rice(s),
        Food::Flatbread => flatbread(s),
        Food::Bread => bread(s),
        Food::Cake => cake(s),
        Food::Mush => mush(s),
        Food::Charcoal => charcoal(s),
    }
}

fn draw_process(s: &Sketch, process: Process) {
    match process {
        Process::Chop => knife(s),
        Process::Grind => millstones(s),
        Process::Churn => churn(s),
        Process::Knead => knead(s),
        Process::Mix => whisk(s),
        Process::Boil => pot(s),
        Process::Fry => pan(s),
        Process::Bake => oven(s),
        Process::Prove => prove(s),
        Process::Soak => soak(s),
        Process::Steep => tea_bag(s),
    }
}

fn draw_kind(s: &Sketch, kind: Kind) {
    match kind {
        Kind::Physical => fist(s),
        Kind::Heat => flame(s),
        Kind::Time => hourglass(s),
    }
}

// ---------------------------------------------------------------------------
// The sketch: local units, rotation, and the silhouette pass.

/// A [`Pen`] in an icon's local units: a 32x32 box centred on the origin,
/// optionally turned, and optionally tracing silhouettes for the outline.
#[derive(Clone, Copy)]
struct Sketch<'a> {
    pen: Pen<'a>,
    /// Where local (0, 0) is, in frame units.
    origin: Vec2,
    /// Frame units per local unit.
    unit: f32,
    /// The turn, as a unit vector.
    turn: Vec2,
    /// Outline width, in frame units.
    line: f32,
    /// On the outline pass, how far to grow every shape, in frame units.
    trace: Option<f32>,
}

impl<'a> Sketch<'a> {
    fn new(pen: &Pen<'a>, c: Vec2, size: f32) -> Self {
        Self {
            pen: *pen,
            origin: c,
            unit: size / 32.0,
            turn: Vec2::X,
            line: size / 32.0 * OUTLINE,
            trace: None,
        }
    }

    /// The same sketch on its silhouette pass.
    const fn traced(self) -> Self {
        Self {
            trace: Some(self.line),
            ..self
        }
    }

    /// Draw `f` with an outline of its own, over whatever is already drawn,
    /// so overlapping pieces stay separate.
    fn solo(&self, f: impl Fn(&Self)) {
        if !self.tracing() {
            f(&self.traced());
        }
        f(self);
    }

    const fn tracing(&self) -> bool {
        self.trace.is_some()
    }

    /// A sketch at local `at`, scaled by `scale` and turned by `deg`.
    fn sub(&self, at: Vec2, scale: f32, deg: f32) -> Self {
        Self {
            origin: self.at(at),
            unit: self.unit * scale,
            turn: self.turn.rotate(Vec2::from_angle(deg.to_radians())),
            ..*self
        }
    }

    fn at(&self, p: Vec2) -> Vec2 {
        self.origin + self.turn.rotate(p) * self.unit
    }

    fn angle(&self) -> f32 {
        self.turn.y.atan2(self.turn.x)
    }

    /// Growth on the outline pass, in frame units.
    fn grow(&self) -> f32 {
        self.trace.unwrap_or(0.0)
    }

    /// Growth on the outline pass, in local units.
    fn local_grow(&self) -> f32 {
        self.grow() / self.unit
    }

    const fn ink(&self, c: Color) -> Color {
        if self.tracing() { ink::LINE } else { c }
    }

    fn circle(&self, p: Vec2, r: f32, c: Color) {
        self.pen
            .circle(self.at(p), r.mul_add(self.unit, self.grow()), self.ink(c));
    }

    fn ellipse(&self, p: Vec2, r: Vec2, deg: f32, c: Color) {
        self.pen.ellipse(
            self.at(p),
            r * self.unit + Vec2::splat(self.grow()),
            self.angle() + deg.to_radians(),
            self.ink(c),
        );
    }

    fn capsule(&self, a: Vec2, b: Vec2, r: f32, c: Color) {
        self.pen.capsule(
            self.at(a),
            self.at(b),
            r.mul_add(self.unit, self.grow()),
            self.ink(c),
        );
    }

    /// A chain of capsules through `points`.
    fn chain(&self, points: &[Vec2], r: f32, c: Color) {
        for w in points.windows(2) {
            self.capsule(w[0], w[1], r, c);
        }
    }

    /// A convex polygon. Traced, it grows by rolling a disc round its edge.
    fn poly(&self, points: &[Vec2], c: Color) {
        let pts: Vec<Vec2> = points.iter().map(|&p| self.at(p)).collect();
        let col = self.ink(c);
        if let Some(g) = self.trace {
            for (i, &p) in pts.iter().enumerate() {
                self.pen.capsule(p, pts[(i + 1) % pts.len()], g, col);
            }
        }
        self.pen.poly(&pts, col);
    }

    /// A convex polygon through literal local points.
    fn shape(&self, points: &[(f32, f32)], col: Color) {
        let pts: Vec<Vec2> = points.iter().map(|&(x, y)| vec2(x, y)).collect();
        self.poly(&pts, col);
    }

    /// A box centred on `pos` with corners of radius `round`.
    fn rrect(&self, pos: Vec2, size: Vec2, round: f32, col: Color) {
        let grow = self.local_grow();
        let half = size * 0.5 + Vec2::splat(grow);
        let round = (round + grow).min(half.x).min(half.y).max(0.0);
        let mut pts = Vec::with_capacity(24);
        for (corner, from) in [
            (vec2(half.x - round, half.y - round), 0.0_f32),
            (vec2(round - half.x, half.y - round), 90.0),
            (vec2(round - half.x, round - half.y), 180.0),
            (vec2(half.x - round, round - half.y), 270.0),
        ] {
            for step in 0..=5 {
                let deg = 18.0f32.mul_add(step as f32, from);
                pts.push(self.at(pos + corner + Vec2::from_angle(deg.to_radians()) * round));
            }
        }
        self.pen.poly(&pts, self.ink(col));
    }

    /// The part of an ellipse from `from` for `span` degrees, closed by its
    /// chord: a half-ellipse, a dome, a bowl.
    fn seg(&self, pos: Vec2, r: Vec2, from: f32, span: f32, col: Color) {
        let n = 20;
        let pts: Vec<Vec2> = (0..=n)
            .map(|i| {
                let a = (span * i as f32 / n as f32 + from).to_radians();
                pos + vec2(a.cos() * r.x, a.sin() * r.y)
            })
            .collect();
        self.poly(&pts, col);
    }

    /// A pie slice.
    fn pie(&self, pos: Vec2, r: f32, from: f32, span: f32, col: Color) {
        let n = 12;
        let mut pts = vec![pos];
        pts.extend((0..=n).map(|i| {
            let a = (span * i as f32 / n as f32 + from).to_radians();
            pos + Vec2::from_angle(a) * r
        }));
        self.poly(&pts, col);
    }

    /// A band `thick` wide along an ellipse, round the `arc` given as
    /// (from, span) in degrees.
    fn band(&self, pos: Vec2, radii: Vec2, thick: f32, arc: (f32, f32), col: Color) {
        let grow = self.local_grow();
        let (thick, ext) = if self.tracing() {
            (
                2.0f32.mul_add(grow, thick),
                (grow / radii.x.min(radii.y)).to_degrees(),
            )
        } else {
            (thick, 0.0)
        };
        let (from, span) = (arc.0 - ext, 2.0f32.mul_add(ext, arc.1).min(360.0));
        let steps = ((span / 10.0).ceil() as usize).max(2);
        let point = |step: usize, off: f32| {
            let rad = (span * step as f32 / steps as f32 + from).to_radians();
            self.at(pos + vec2(rad.cos() * (radii.x + off), rad.sin() * (radii.y + off)))
        };
        let half = thick * 0.5;
        let col = self.ink(col);
        for step in 0..steps {
            self.pen.poly(
                &[
                    point(step, -half),
                    point(step, half),
                    point(step + 1, half),
                    point(step + 1, -half),
                ],
                col,
            );
        }
    }

    fn ring(&self, p: Vec2, r: f32, thick: f32, c: Color) {
        self.band(p, Vec2::splat(r), thick, (0.0, 360.0), c);
    }

    /// A pointed leaf from `base`, `len` long, pointing `deg`.
    fn leaf(&self, base: Vec2, len: f32, width: f32, deg: f32, c: Color) {
        let s = self.sub(base, 1.0, deg);
        let n = 10;
        let edge = |side: f32| {
            (0..=n).map(move |i| {
                let t = i as f32 / n as f32;
                vec2(t * len, side * width * 0.5 * (PI * t).sin().powf(0.7))
            })
        };
        let top: Vec<Vec2> = edge(-1.0).collect();
        let mut whole = top.clone();
        whole.extend(edge(1.0).rev().skip(1).take(n - 1));
        s.poly(&whole, dark(c, 0.2));
        if self.tracing() {
            return;
        }
        s.poly(&top, c);
        s.capsule(
            vec2(len * 0.15, 0.0),
            vec2(len * 0.75, 0.0),
            0.3,
            light(c, 0.2),
        );
    }

    /// An ellipse with a shaded lower-right and a highlight upper-left.
    fn ball(&self, p: Vec2, r: Vec2, deg: f32, c: Color) {
        self.ellipse(p, r, deg, dark(c, 0.2));
        if self.tracing() {
            return;
        }
        let s = self.sub(p, 1.0, deg);
        s.ellipse(vec2(-r.x * 0.08, -r.y * 0.1), r * 0.86, 0.0, c);
        s.ellipse(
            vec2(-r.x * 0.42, -r.y * 0.42),
            vec2(r.x * 0.2, r.y * 0.13),
            -40.0,
            light(c, 0.55),
        );
    }

    /// A wisp of steam rising `h` from `p`. Not outlined.
    fn steam(&self, p: Vec2, h: f32, phase: f32) {
        self.wisp(p, h, phase, alpha(ink::WHITE, 0.85));
    }

    /// A tapering wavy line rising `height` from `pos`. Not outlined.
    fn wisp(&self, pos: Vec2, height: f32, phase: f32, col: Color) {
        if self.tracing() {
            return;
        }
        let steps = 12;
        let pts: Vec<Vec2> = (0..=steps)
            .map(|i| {
                let t = i as f32 / steps as f32;
                pos + vec2((t * PI).mul_add(2.0, phase).sin() * 1.5, -t * height)
            })
            .collect();
        let width = |i: usize| 1.3 * (1.0 - i as f32 / steps as f32).sqrt().max(0.25);
        let normal = |i: usize| {
            let along = pts[(i + 1).min(steps)] - pts[i.saturating_sub(1)];
            vec2(-along.y, along.x).normalize_or_zero() * width(i)
        };
        for i in 0..steps {
            let (from, to) = (pts[i], pts[i + 1]);
            let (n0, n1) = (normal(i), normal(i + 1));
            self.pen.poly(
                &[
                    self.at(from + n0),
                    self.at(to + n1),
                    self.at(to - n1),
                    self.at(from - n0),
                ],
                col,
            );
        }
    }

    /// A band across a straight-sided body, shaded on its right.
    fn rect_band(&self, pos: Vec2, size: Vec2, col: Color) {
        self.rrect(pos, size, 0.0, dark(col, 0.25));
        self.rrect(
            pos - vec2(size.x * 0.1, 0.0),
            vec2(size.x * 0.8, size.y),
            0.0,
            col,
        );
    }
}

// ---------------------------------------------------------------------------
// Shared furniture: bowls, plates, cups.

/// Where a bowl's rim sits, and its radii.
const RIM_Y: f32 = -1.0;
const RIM: Vec2 = vec2(14.5, 4.6);
/// The opening inside the rim, where food goes.
const INSIDE: Vec2 = vec2(12.9, 3.4);

/// A bowl with `inside` drawn into it, between the back and front of the
/// rim.
fn bowl(s: &Sketch, body: Color, inside: impl Fn(&Sketch)) {
    s.rrect(vec2(0.0, 12.6), vec2(11.0, 3.2), 1.4, dark(body, 0.3));
    s.seg(
        vec2(0.0, RIM_Y),
        vec2(RIM.x, 13.0),
        0.0,
        180.0,
        dark(body, 0.2),
    );
    if !s.tracing() {
        s.seg(vec2(-1.2, RIM_Y), vec2(RIM.x - 2.2, 11.6), 0.0, 180.0, body);
        s.ellipse(vec2(-8.5, 4.5), vec2(1.3, 2.8), -35.0, light(body, 0.45));
    }
    s.ellipse(vec2(0.0, RIM_Y), RIM, 0.0, light(body, 0.3));
    s.ellipse(vec2(0.0, RIM_Y + 0.3), INSIDE, 0.0, dark(body, 0.45));
    inside(s);
    s.band(
        vec2(0.0, RIM_Y),
        RIM - Vec2::splat(0.75),
        1.5,
        (0.0, 180.0),
        light(body, 0.3),
    );
}

/// A flat liquid filling a bowl, with a sheen.
fn surface(s: &Sketch, c: Color) {
    s.ellipse(vec2(0.0, RIM_Y + 0.3), INSIDE, 0.0, c);
    if !s.tracing() {
        s.ellipse(vec2(-4.5, RIM_Y - 0.4), vec2(3.5, 0.8), 0.0, light(c, 0.3));
    }
}

/// A heap rising `h` above a bowl's rim.
fn mound(s: &Sketch, c: Color, h: f32) {
    let base = vec2(0.0, RIM_Y + 0.3);
    s.ellipse(base, INSIDE, 0.0, dark(c, 0.12));
    s.seg(base, vec2(INSIDE.x, h), 180.0, 180.0, dark(c, 0.12));
    if !s.tracing() {
        s.seg(
            base + vec2(-1.0, 0.0),
            vec2(INSIDE.x - 1.6, h - 1.0),
            180.0,
            180.0,
            c,
        );
        s.ellipse(
            vec2(-4.5, h.mul_add(-0.55, RIM_Y)),
            vec2(2.4, 1.3),
            -20.0,
            light(c, 0.45),
        );
    }
}

/// A plate seen from a little above, centred at height `y`.
fn plate(s: &Sketch, y: f32, rx: f32) {
    let ry = rx * 0.32;
    s.ellipse(vec2(0.0, y + 1.5), vec2(rx, ry), 0.0, dark(PLATE, 0.35));
    s.ellipse(vec2(0.0, y), vec2(rx, ry), 0.0, PLATE);
    if !s.tracing() {
        s.band(
            vec2(0.0, y),
            vec2(rx - 1.1, ry - 0.7),
            0.7,
            (0.0, 360.0),
            light(ink::GLAZE, 0.2),
        );
        s.ellipse(
            vec2(0.0, y + 0.3),
            vec2(rx * 0.66, ry * 0.6),
            0.0,
            dark(PLATE, 0.08),
        );
    }
}

/// A teacup on a saucer, holding `liquid`.
fn cup(s: &Sketch, liquid: Color) {
    let china = ink::CERAMIC;
    s.ellipse(vec2(0.0, 12.8), vec2(14.5, 3.4), 0.0, dark(china, 0.3));
    s.ellipse(vec2(0.0, 11.9), vec2(14.5, 3.2), 0.0, china);
    if !s.tracing() {
        s.ellipse(vec2(0.0, 12.0), vec2(7.5, 1.6), 0.0, dark(china, 0.1));
    }
    s.band(
        vec2(10.3, 3.2),
        vec2(3.4, 3.8),
        2.2,
        (-100.0, 200.0),
        dark(china, 0.12),
    );
    s.seg(
        vec2(0.0, -1.0),
        vec2(10.5, 12.5),
        0.0,
        180.0,
        dark(china, 0.2),
    );
    if !s.tracing() {
        s.seg(vec2(-1.0, -1.0), vec2(9.2, 11.4), 0.0, 180.0, china);
        s.band(
            vec2(-0.4, -1.0),
            vec2(9.8, 6.0),
            1.4,
            (15.0, 150.0),
            ink::GLAZE,
        );
        s.ellipse(vec2(-6.5, 3.5), vec2(1.1, 2.4), -30.0, ink::WHITE);
    }
    s.ellipse(vec2(0.0, -1.0), vec2(10.5, 3.1), 0.0, light(china, 0.4));
    s.ellipse(vec2(0.0, -0.7), vec2(9.0, 2.2), 0.0, liquid);
    if !s.tracing() {
        s.ellipse(vec2(-3.5, -1.2), vec2(2.8, 0.6), 0.0, light(liquid, 0.3));
    }
    s.steam(vec2(-3.0, -4.5), 10.0, 0.0);
    s.steam(vec2(3.0, -5.0), 9.0, 2.2);
}

/// An egg shape, `k` times the size of a whole raw egg.
fn egg_shape(s: &Sketch, p: Vec2, k: f32, c: Color) {
    s.ellipse(p + vec2(0.0, 2.5) * k, vec2(10.0, 10.0) * k, 0.0, c);
    s.ellipse(p + vec2(0.0, -1.0) * k, vec2(8.4, 12.0) * k, 0.0, c);
}

/// A droplet: a circle of radius `r` at `p` with a point above it.
fn drop_shape(s: &Sketch, p: Vec2, r: f32, c: Color) {
    let (sx, cy) = (0.821 * r, 0.571 * r);
    s.poly(
        &[
            p + vec2(0.0, -1.75 * r),
            p + vec2(sx, -cy),
            p + vec2(-sx, -cy),
        ],
        c,
    );
    s.circle(p, r, c);
}

// ---------------------------------------------------------------------------
// Raw.

fn water(s: &Sketch) {
    drop_shape(s, vec2(0.0, 4.0), 10.0, ink::WATER_DARK);
    if s.tracing() {
        return;
    }
    drop_shape(s, vec2(-0.9, 3.2), 8.5, ink::WATER);
    s.ellipse(
        vec2(-4.3, 4.0),
        vec2(1.5, 3.2),
        15.0,
        light(ink::WATER, 0.6),
    );
    s.circle(vec2(-2.8, -3.0), 1.0, light(ink::WATER, 0.6));
}

fn milk(s: &Sketch) {
    let side = hex(0xcf_dbe4);
    let glass = ink::MILK;
    s.rrect(vec2(0.0, 5.0), vec2(17.0, 19.0), 5.0, side);
    s.shape(
        &[(-8.5, -2.0), (8.5, -2.0), (4.2, -8.5), (-4.2, -8.5)],
        side,
    );
    s.rrect(vec2(0.0, -9.5), vec2(8.4, 4.0), 1.0, side);
    if !s.tracing() {
        s.rrect(vec2(-1.2, 4.8), vec2(14.0, 18.0), 4.0, glass);
        s.shape(
            &[(-8.0, -2.0), (5.6, -2.0), (2.6, -8.5), (-4.0, -8.5)],
            glass,
        );
        s.rrect(vec2(-0.8, -9.5), vec2(6.6, 4.0), 1.0, glass);
        s.rect_band(vec2(0.0, 6.0), vec2(17.0, 5.0), ink::GLAZE);
        s.capsule(vec2(-5.0, -0.5), vec2(-5.0, 1.5), 0.9, ink::WHITE);
        s.capsule(vec2(-5.0, 10.0), vec2(-5.0, 11.5), 0.9, ink::WHITE);
    }
    s.rrect(vec2(0.0, -12.5), vec2(10.0, 4.0), 1.6, ink::GLAZE);
    if !s.tracing() {
        s.rrect(
            vec2(-0.8, -13.2),
            vec2(7.0, 1.4),
            0.7,
            light(ink::GLAZE, 0.4),
        );
    }
}

fn egg(s: &Sketch) {
    egg_shape(s, vec2(0.0, 1.0), 1.0, dark(ink::EGG, 0.2));
    if s.tracing() {
        return;
    }
    egg_shape(s, vec2(-0.9, 0.2), 0.88, ink::EGG);
    s.ellipse(vec2(-4.2, -4.5), vec2(1.8, 3.4), 20.0, light(ink::EGG, 0.6));
}

fn carrot(s: &Sketch) {
    let s = s.sub(vec2(1.0, 1.0), 1.0, 32.0);
    for (deg, len, c) in [
        (-122.0, 9.5, ink::LEAF),
        (-58.0, 9.5, ink::LEAF),
        (-90.0, 11.0, ink::LEAF_LIGHT),
    ] {
        s.leaf(vec2(0.0, -6.0), len, 4.6, deg, c);
    }
    let body = [
        vec2(-6.2, -6.5),
        vec2(6.2, -6.5),
        vec2(1.3, 13.5),
        vec2(-1.3, 13.5),
    ];
    s.poly(&body, dark(ink::CARROT, 0.2));
    s.circle(vec2(0.0, 13.5), 1.3, dark(ink::CARROT, 0.2));
    s.ellipse(vec2(0.0, -6.5), vec2(6.2, 2.2), 0.0, dark(ink::CARROT, 0.2));
    if s.tracing() {
        return;
    }
    s.shape(
        &[(-6.2, -6.5), (2.0, -6.5), (0.0, 13.5), (-1.3, 13.5)],
        ink::CARROT,
    );
    s.ellipse(
        vec2(0.0, -6.5),
        vec2(6.2, 2.2),
        0.0,
        light(ink::CARROT, 0.25),
    );
    s.ellipse(vec2(0.0, -6.6), vec2(2.4, 0.9), 0.0, ink::LEAF_DARK);
    let ridge = dark(ink::CARROT, 0.4);
    s.capsule(vec2(-4.8, -1.5), vec2(-2.0, -1.1), 0.45, ridge);
    s.capsule(vec2(1.2, 3.0), vec2(3.6, 2.7), 0.45, ridge);
    s.capsule(vec2(-2.3, 8.0), vec2(-0.6, 8.2), 0.45, ridge);
    s.capsule(
        vec2(-3.4, -3.8),
        vec2(-2.4, 2.5),
        0.8,
        light(ink::CARROT, 0.45),
    );
}

fn onion(s: &Sketch) {
    let skin = ink::ONION;
    s.shape(
        &[(-6.5, -3.0), (6.5, -3.0), (1.2, -12.0), (-1.2, -12.0)],
        dark(skin, 0.2),
    );
    s.capsule(vec2(0.0, -12.0), vec2(1.0, -15.0), 1.0, ink::ONION_SKIN);
    for (a, b) in [(-2.0, -3.5), (0.0, 0.0), (2.0, 3.5)] {
        s.capsule(vec2(a, 13.0), vec2(b, 15.3), 0.55, ink::ONION_SKIN);
    }
    s.ball(vec2(0.0, 3.0), vec2(11.0, 10.5), 0.0, skin);
    if s.tracing() {
        return;
    }
    s.shape(
        &[(-6.0, -3.0), (4.0, -3.0), (0.3, -11.6), (-1.0, -11.6)],
        skin,
    );
    let stripe = light(skin, 0.3);
    s.band(vec2(0.0, 3.0), vec2(5.5, 10.0), 0.8, (110.0, 140.0), stripe);
    s.band(
        vec2(0.0, 3.0),
        vec2(5.5, 10.0),
        0.8,
        (-70.0, 140.0),
        dark(skin, 0.3),
    );
    s.ellipse(vec2(-6.5, 0.0), vec2(1.4, 3.0), 20.0, light(skin, 0.55));
}

fn tomato(s: &Sketch) {
    s.ball(vec2(0.0, 2.5), vec2(12.5, 10.5), 0.0, ink::TOMATO);
    s.capsule(vec2(0.0, -7.5), vec2(1.5, -12.5), 1.2, ink::LEAF_DARK);
    for deg in [160.0, -155.0, 20.0, -25.0, 95.0] {
        s.leaf(vec2(0.0, -7.5), 6.5, 3.2, deg, ink::LEAF);
    }
}

fn wheat_ear(s: &Sketch, c: Color) {
    let (shade, grain) = (dark(c, 0.25), c);
    s.capsule(vec2(0.0, 15.0), vec2(0.0, -2.0), 0.9, shade);
    s.leaf(vec2(0.0, 9.0), 7.0, 2.8, -45.0, shade);
    s.ellipse(vec2(0.0, -12.5), vec2(2.0, 3.6), 0.0, shade);
    for y in [-8.5, -4.5, -0.5] {
        for side in [-1.0, 1.0] {
            s.ellipse(vec2(side * 2.4, y), vec2(2.2, 3.6), side * 25.0, shade);
        }
    }
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-0.4, -12.9), vec2(1.3, 2.8), 0.0, grain);
    for y in [-8.5, -4.5, -0.5] {
        for side in [-1.0_f32, 1.0] {
            s.ellipse(
                vec2(side.mul_add(2.4, -0.4), y - 0.4),
                vec2(1.4, 2.8),
                side * 25.0,
                grain,
            );
        }
    }
    for dx in [-1.5, 0.0, 1.5] {
        s.capsule(vec2(dx * 0.3, -15.0), vec2(dx, -17.5), 0.25, shade);
    }
}

fn wheat(s: &Sketch) {
    wheat_ear(&s.sub(vec2(-4.5, 1.5), 0.85, -20.0), dark(ink::WHEAT, 0.08));
    s.sub(vec2(2.5, 0.5), 1.0, 14.0)
        .solo(|s| wheat_ear(s, ink::WHEAT));
    s.rrect(vec2(-0.5, 10.5), vec2(6.0, 2.4), 1.0, ink::WOOD_LIGHT);
}

fn grain(s: &Sketch, p: Vec2, deg: f32, c: Color) {
    s.ellipse(p, vec2(1.3, 0.75), deg, c);
}

fn rice(s: &Sketch) {
    let sack = BURLAP;
    s.ellipse(vec2(0.0, 7.5), vec2(12.0, 7.5), 0.0, dark(sack, 0.2));
    s.shape(
        &[(-9.0, -3.0), (9.0, -3.0), (12.0, 7.0), (-12.0, 7.0)],
        dark(sack, 0.2),
    );
    if !s.tracing() {
        s.ellipse(vec2(-1.2, 7.0), vec2(10.4, 6.8), 0.0, sack);
        s.shape(&[(-8.8, -3.0), (6.6, -3.0), (9.4, 7.0), (-11.6, 7.0)], sack);
        s.rrect(vec2(4.5, 6.5), vec2(5.0, 4.4), 0.8, dark(sack, 0.12));
        for x in [2.6, 6.4] {
            s.capsule(vec2(x, 4.8), vec2(x, 8.2), 0.25, dark(sack, 0.35));
        }
        s.ellipse(vec2(-7.0, 4.0), vec2(1.2, 2.6), 10.0, light(sack, 0.35));
    }
    s.ellipse(vec2(0.0, -3.5), vec2(9.6, 3.3), 0.0, light(sack, 0.25));
    s.seg(vec2(0.0, -3.5), vec2(7.8, 6.0), 180.0, 180.0, ink::RICE);
    s.ellipse(vec2(0.0, -3.5), vec2(7.8, 2.0), 0.0, ink::RICE);
    if s.tracing() {
        return;
    }
    let speck = dark(ink::RICE, 0.12);
    for (x, y, d) in [
        (-3.0, -6.0, 20.0),
        (1.0, -8.0, -30.0),
        (3.5, -5.0, 60.0),
        (-0.5, -4.0, -10.0),
        (-5.0, -3.5, 40.0),
    ] {
        grain(s, vec2(x, y), d, speck);
    }
    s.band(
        vec2(0.0, -3.5),
        vec2(9.0, 2.8),
        1.5,
        (0.0, 180.0),
        light(sack, 0.25),
    );
}

fn bean(s: &Sketch, p: Vec2, deg: f32, k: f32, c: Color) {
    s.sub(p, k, deg).solo(|s| bean_shape(s, c));
}

fn bean_shape(s: &Sketch, c: Color) {
    let shade = dark(c, 0.3);
    s.ellipse(vec2(-2.2, 0.3), vec2(3.6, 3.4), 0.0, shade);
    s.ellipse(vec2(2.2, 0.3), vec2(3.6, 3.4), 0.0, shade);
    s.ellipse(vec2(0.0, 1.0), vec2(4.0, 2.8), 0.0, shade);
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-2.2, -0.1), vec2(3.0, 2.8), 0.0, c);
    s.ellipse(vec2(2.0, -0.1), vec2(3.0, 2.8), 0.0, c);
    s.ellipse(vec2(0.0, 0.5), vec2(3.4, 2.2), 0.0, c);
    s.capsule(vec2(-3.6, -1.0), vec2(-1.0, -1.7), 0.6, light(c, 0.4));
    s.ellipse(vec2(0.3, -2.2), vec2(1.1, 0.45), 0.0, light(c, 0.75));
}

fn beans(s: &Sketch) {
    bean(s, vec2(0.5, -5.0), 5.0, 1.25, ink::BEAN);
    bean(s, vec2(-5.5, 4.5), 25.0, 1.25, ink::BEAN);
    bean(s, vec2(6.0, 6.0), -18.0, 1.25, ink::BEAN);
}

/// An isometric cube of side `k` whose nearest top corner is `f`.
fn cube(s: &Sketch, f: Vec2, k: f32) {
    let (dx, dy) = (k * 0.87, k * 0.5);
    let top = [f, f + vec2(dx, -dy), f + vec2(0.0, -k), f + vec2(-dx, -dy)];
    let left = [
        f + vec2(-dx, -dy),
        f,
        f + vec2(0.0, k),
        f + vec2(-dx, k - dy),
    ];
    let right = [f, f + vec2(dx, -dy), f + vec2(dx, k - dy), f + vec2(0.0, k)];
    s.poly(&left, hex(0xe8_ebf1));
    s.poly(&right, hex(0xc4_ccd8));
    s.poly(&top, ink::SUGAR);
    if s.tracing() {
        return;
    }
    for (dx, dy) in [(-0.45, 0.3), (-0.6, 0.75), (0.5, 0.55)] {
        s.circle(f + vec2(dx, dy) * vec2(0.87, 1.0) * k, 0.45, hex(0xa9_b4c4));
    }
}

fn sugar(s: &Sketch) {
    let k = 7.4;
    for front in [vec2(5.0, 1.0), vec2(-4.0, 4.5), vec2(1.0, -5.0)] {
        s.solo(|s| cube(s, front, k));
    }
}

fn tea_leaf(s: &Sketch) {
    s.chain(
        &[vec2(1.5, 14.5), vec2(0.6, 6.0), vec2(0.0, -3.0)],
        0.85,
        ink::LEAF_DARK,
    );
    s.leaf(vec2(0.3, 2.0), 11.0, 6.8, -148.0, ink::TEA_LEAF);
    s.leaf(vec2(0.8, 7.0), 10.5, 6.4, -28.0, ink::TEA_LEAF);
    s.leaf(
        vec2(0.0, -3.0),
        11.5,
        7.0,
        -92.0,
        light(ink::TEA_LEAF, 0.15),
    );
}

fn spice(s: &Sketch) {
    let (p0, p1, p2) = (vec2(-5.0, -6.5), vec2(10.0, -5.0), vec2(5.0, 13.5));
    let n = 16;
    let point = |t: f32| {
        let m = 1.0 - t;
        p0 * (m * m) + p1 * (2.0 * m * t) + p2 * (t * t)
    };
    let radius = |t: f32| 4.6f32.mul_add((1.0 - t).powf(0.7), 0.7);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        s.circle(point(t), radius(t), dark(ink::CHILI, 0.25));
    }
    s.ellipse(vec2(-5.5, -7.0), vec2(3.0, 4.4), -40.0, ink::LEAF_DARK);
    s.chain(
        &[vec2(-6.5, -8.5), vec2(-9.0, -11.5), vec2(-8.0, -14.5)],
        1.0,
        ink::LEAF_DARK,
    );
    if s.tracing() {
        return;
    }
    for i in 0..=n {
        let t = i as f32 / n as f32;
        s.circle(point(t) + vec2(-0.5, -0.5), radius(t) * 0.78, ink::CHILI);
    }
    let shine: Vec<Vec2> = (2..=7)
        .map(|i| {
            let t = i as f32 / n as f32;
            point(t) + vec2(-1.2, -2.2) * (1.0 - t)
        })
        .collect();
    s.chain(&shine, 0.7, light(ink::CHILI, 0.45));
    s.ellipse(vec2(-5.8, -7.4), vec2(2.0, 3.4), -40.0, ink::LEAF);
    s.ellipse(vec2(-6.6, -8.0), vec2(0.7, 1.4), -40.0, ink::LEAF_LIGHT);
}

fn salt(s: &Sketch) {
    let glass = GLASS;
    s.rrect(vec2(0.0, 4.5), vec2(15.0, 18.0), 4.5, dark(glass, 0.2));
    if !s.tracing() {
        s.rrect(vec2(-0.8, 4.5), vec2(12.6, 17.0), 4.0, glass);
        s.rrect(vec2(-0.3, 8.0), vec2(12.0, 9.0), 3.5, ink::SALT);
        s.rrect(vec2(-1.0, 7.5), vec2(9.0, 7.0), 3.0, ink::WHITE);
        s.capsule(vec2(-4.5, -1.0), vec2(-4.5, 1.5), 0.8, ink::WHITE);
    }
    s.seg(
        vec2(0.0, -4.5),
        vec2(7.6, 6.0),
        180.0,
        180.0,
        ink::STONE_LIGHT,
    );
    s.rrect(vec2(0.0, -4.5), vec2(16.0, 3.2), 1.2, ink::STONE);
    if s.tracing() {
        return;
    }
    s.ellipse(
        vec2(-3.0, -7.8),
        vec2(1.4, 1.8),
        -30.0,
        light(ink::STONE_LIGHT, 0.5),
    );
    for (x, y) in [(-2.4, -7.5), (0.6, -9.0), (2.8, -7.0)] {
        s.circle(vec2(x, y), 0.75, ink::STONE_DARK);
    }
    s.rrect(vec2(-0.8, -5.0), vec2(13.0, 1.0), 0.5, ink::STONE_LIGHT);
    for (x, y) in [(6.5, -13.0), (9.0, -11.0), (10.5, -14.5)] {
        s.rrect(vec2(x, y), vec2(1.3, 1.3), 0.2, ink::WHITE);
    }
}

// ---------------------------------------------------------------------------
// Prepared.

fn sack(s: &Sketch, c: Color) {
    let body = |s: &Sketch, dx: f32, w: f32, c: Color| {
        s.ellipse(vec2(dx, 7.0), vec2(w, 7.6), 0.0, c);
        s.shape(
            &[
                (-8.6, -3.0),
                (8.6 - 12.0 + w, -3.0),
                (dx + w, 7.0),
                (dx - w, 7.0),
            ],
            c,
        );
    };
    let shade = dark(c, 0.15);
    body(s, 0.0, 12.0, shade);
    s.shape(
        &[(-8.6, -3.0), (8.6, -3.0), (2.8, -8.5), (-2.8, -8.5)],
        shade,
    );
    s.shape(
        &[(-2.8, -8.5), (2.8, -8.5), (6.0, -13.5), (-6.0, -13.5)],
        shade,
    );
    for x in [-4.0, 0.0, 4.0] {
        s.ellipse(vec2(x, -13.2), vec2(2.6, 1.8), 0.0, shade);
    }
    if !s.tracing() {
        body(s, -1.2, 10.6, c);
        s.shape(&[(-8.4, -3.0), (6.0, -3.0), (1.5, -8.5), (-2.8, -8.5)], c);
        s.shape(&[(-2.6, -8.6), (1.2, -8.6), (3.2, -13.0), (-5.4, -13.0)], c);
        s.ellipse(vec2(-4.2, -13.4), vec2(1.8, 1.2), 0.0, c);
        s.ellipse(vec2(-0.4, -13.6), vec2(1.8, 1.2), 0.0, c);
    }
    s.rrect(vec2(0.0, -8.5), vec2(8.0, 2.6), 1.2, ink::WOOD_LIGHT);
}

fn flour(s: &Sketch) {
    sack(s, ink::FLOUR);
    if !s.tracing() {
        wheat_ear(&s.sub(vec2(-0.6, 4.5), 0.36, 0.0), ink::WHEAT);
        for (x, y) in [(10.5, 13.0), (13.0, 11.0), (-11.5, 13.5)] {
            s.circle(vec2(x, y), 0.9, ink::WHITE);
        }
    }
}

fn dough(s: &Sketch) {
    let c = ink::DOUGH;
    s.ellipse(vec2(0.0, 7.0), vec2(13.5, 6.5), 0.0, dark(c, 0.18));
    s.ellipse(vec2(-1.0, 3.5), vec2(10.0, 7.0), 0.0, dark(c, 0.18));
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-0.8, 6.0), vec2(12.2, 5.6), 0.0, c);
    s.ellipse(vec2(-1.8, 2.8), vec2(8.6, 5.8), 0.0, c);
    s.ellipse(vec2(-5.0, -0.5), vec2(3.0, 1.3), -15.0, light(c, 0.55));
    s.ellipse(vec2(3.0, 4.0), vec2(2.6, 1.4), 0.0, dark(c, 0.28));
    s.ellipse(vec2(3.0, 4.6), vec2(2.0, 0.8), 0.0, dark(c, 0.12));
    s.band(
        vec2(-2.0, 7.0),
        vec2(6.0, 3.0),
        0.6,
        (20.0, 120.0),
        dark(c, 0.15),
    );
    for (x, y) in [(-8.0, 4.0), (6.0, 0.0), (-3.0, 9.0)] {
        s.circle(vec2(x, y), 0.6, ink::WHITE);
    }
}

fn risen_dough(s: &Sketch) {
    let c = ink::DOUGH;
    s.seg(
        vec2(0.0, 9.0),
        vec2(14.5, 20.0),
        180.0,
        180.0,
        dark(c, 0.18),
    );
    s.ellipse(vec2(0.0, 9.5), vec2(14.5, 3.6), 0.0, dark(c, 0.18));
    if s.tracing() {
        return;
    }
    s.seg(vec2(-1.2, 8.5), vec2(12.6, 18.6), 180.0, 180.0, c);
    s.ellipse(vec2(-1.2, 8.8), vec2(12.6, 2.8), 0.0, c);
    s.ellipse(vec2(-6.0, -2.5), vec2(2.4, 5.0), 25.0, light(c, 0.55));
    for (x, y, r) in [
        (3.5, -4.0, 0.9),
        (6.0, 2.0, 0.7),
        (-2.0, 3.0, 0.6),
        (1.0, -8.5, 0.6),
    ] {
        s.circle(vec2(x, y), r, dark(c, 0.2));
    }
    for (x, y) in [(-3.0, -7.0), (2.0, -1.0), (-6.5, 4.5)] {
        s.circle(vec2(x, y), 0.6, ink::WHITE);
    }
}

fn batter(s: &Sketch) {
    let jug = ink::GLAZE;
    s.band(
        vec2(9.5, 0.0),
        vec2(4.6, 6.0),
        2.4,
        (-90.0, 180.0),
        dark(jug, 0.25),
    );
    s.shape(&[(-6.0, -10.0), (-12.5, -11.0), (-6.5, -4.0)], jug);
    s.shape(
        &[(-7.2, -8.0), (7.2, -8.0), (10.5, 1.0), (-10.5, 1.0)],
        dark(jug, 0.2),
    );
    s.seg(vec2(0.0, 1.0), vec2(10.5, 12.0), 0.0, 180.0, dark(jug, 0.2));
    if !s.tracing() {
        s.shape(&[(-7.0, -8.0), (5.0, -8.0), (7.8, 1.0), (-10.2, 1.0)], jug);
        s.seg(vec2(-1.2, 1.0), vec2(9.0, 10.8), 0.0, 180.0, jug);
        s.capsule(vec2(-7.0, -2.0), vec2(-7.4, 4.0), 1.0, light(jug, 0.45));
        for (x, y) in [(-2.0, 0.0), (3.0, 5.0), (-3.5, 7.5), (1.5, -4.0)] {
            s.circle(vec2(x, y), 1.1, light(jug, 0.7));
        }
    }
    s.ellipse(vec2(0.0, -8.0), vec2(7.6, 2.6), 0.0, light(jug, 0.35));
    s.ellipse(vec2(0.0, -7.9), vec2(6.2, 1.8), 0.0, BATTER);
    s.capsule(vec2(-10.8, -10.3), vec2(-10.4, -5.0), 1.1, BATTER);
    if !s.tracing() {
        s.ellipse(vec2(-9.0, -10.4), vec2(3.0, 1.0), -10.0, BATTER);
    }
}

fn butter(s: &Sketch) {
    let block = |s: &Sketch, x0: f32, x1: f32| {
        let d = vec2(4.5, -4.0);
        let (fy0, fy1) = (-1.0, 8.5);
        let front = [vec2(x0, fy0), vec2(x1, fy0), vec2(x1, fy1), vec2(x0, fy1)];
        let top = [
            vec2(x0, fy0),
            vec2(x1, fy0),
            vec2(x1, fy0) + d,
            vec2(x0, fy0) + d,
        ];
        let side = [
            vec2(x1, fy0),
            vec2(x1, fy0) + d,
            vec2(x1, fy1) + d,
            vec2(x1, fy1),
        ];
        s.poly(&side, dark(ink::BUTTER, 0.2));
        s.poly(&front, ink::BUTTER);
        s.poly(&top, light(ink::BUTTER, 0.45));
    };
    let s = s.sub(vec2(1.0, 0.5), 1.0, 0.0);
    s.shape(
        &[(-15.0, 11.5), (9.0, 11.5), (14.5, 5.0), (-9.0, 5.0)],
        ink::WHITE,
    );
    block(&s, -14.0, -10.0);
    block(&s, -8.0, 6.5);
    if !s.tracing() {
        s.capsule(
            vec2(-5.0, 1.0),
            vec2(-5.0, 6.0),
            0.7,
            light(ink::BUTTER, 0.5),
        );
    }
}

fn hot_water(s: &Sketch) {
    let c = ink::COPPER;
    let s = s.sub(vec2(-1.5, 0.5), 1.0, 0.0);
    s.band(
        vec2(0.0, -2.5),
        vec2(9.0, 8.0),
        2.2,
        (195.0, 150.0),
        ink::IRON,
    );
    s.shape(
        &[(8.0, 0.0), (9.5, 7.0), (16.0, -6.0), (14.0, -7.0)],
        dark(c, 0.15),
    );
    s.ellipse(vec2(0.0, 4.0), vec2(12.0, 9.0), 0.0, dark(c, 0.25));
    s.rrect(vec2(0.0, 9.5), vec2(22.0, 7.0), 3.0, dark(c, 0.25));
    if !s.tracing() {
        s.ellipse(vec2(-1.2, 3.4), vec2(10.4, 8.0), 0.0, c);
        s.rrect(vec2(-1.2, 9.0), vec2(19.6, 6.0), 3.0, c);
        s.ellipse(vec2(-5.5, 1.0), vec2(1.8, 3.5), 25.0, light(c, 0.5));
        s.circle(vec2(-7.4, 5.5), 0.8, light(c, 0.5));
    }
    s.ellipse(vec2(0.0, -4.2), vec2(6.0, 2.0), 0.0, light(c, 0.2));
    s.circle(vec2(0.0, -6.5), 1.8, ink::WOOD_DARK);
    s.steam(vec2(15.0, -8.5), 8.0, 0.5);
    s.steam(vec2(-4.0, -7.0), 8.0, 2.5);
}

fn coin(s: &Sketch, p: Vec2, r: f32) {
    s.solo(|s| coin_shape(s, p, r));
}

fn coin_shape(s: &Sketch, p: Vec2, r: f32) {
    let ry = r * 0.6;
    let side = dark(ink::CARROT, 0.25);
    s.ellipse(p + vec2(0.0, 2.0), vec2(r, ry), 0.0, side);
    s.rrect(p + vec2(0.0, 1.0), vec2(2.0 * r, 2.0), 0.0, side);
    s.ellipse(p, vec2(r, ry), 0.0, ink::CARROT);
    if s.tracing() {
        return;
    }
    s.ellipse(p, vec2(r * 0.55, ry * 0.55), 0.0, light(ink::CARROT, 0.35));
    s.ellipse(p, vec2(r * 0.2, ry * 0.2), 0.0, light(ink::CARROT, 0.6));
}

fn chopped_carrot(s: &Sketch) {
    coin(s, vec2(1.0, -4.0), 7.5);
    coin(s, vec2(-6.0, 5.0), 7.0);
    coin(s, vec2(6.5, 6.5), 6.5);
}

fn onion_slice(s: &Sketch, p: Vec2, r: f32, deg: f32) {
    s.sub(p, 1.0, deg).solo(|s| half_moon(s, r));
}

fn half_moon(s: &Sketch, r: f32) {
    s.pie(Vec2::ZERO, r, 180.0, 180.0, ink::ONION);
    if s.tracing() {
        return;
    }
    let rings = (0..8)
        .map(|i| (i, (i as f32).mul_add(-1.4, r - 1.3)))
        .take_while(|&(_, k)| k > 0.8);
    for (i, k) in rings {
        let c = if i % 2 == 0 {
            ONION_WHITE
        } else {
            light(ink::ONION, 0.35)
        };
        s.pie(Vec2::ZERO, k, 180.0, 180.0, c);
    }
}

fn chopped_onion(s: &Sketch) {
    onion_slice(s, vec2(-3.0, 0.5), 8.5, -20.0);
    onion_slice(s, vec2(4.5, 11.0), 9.0, 0.0);
    s.solo(|s| {
        s.ring(vec2(5.5, -5.0), 6.0, 2.4, ink::ONION);
        if !s.tracing() {
            s.ring(vec2(5.5, -5.4), 5.8, 0.8, ONION_WHITE);
            s.ring(vec2(5.5, -5.0), 5.0, 0.6, dark(ink::ONION, 0.15));
        }
    });
}

fn chopped_tomato(s: &Sketch) {
    let p = vec2(-2.5, -1.5);
    s.circle(p, 11.5, ink::TOMATO);
    if !s.tracing() {
        tomato_section(s, p);
    }
    s.sub(vec2(9.0, 10.0), 1.0, 0.0).solo(|w| {
        w.pie(vec2(0.0, -4.0), 8.0, 50.0, 80.0, dark(ink::TOMATO, 0.15));
        if !w.tracing() {
            w.pie(vec2(0.0, -4.0), 6.6, 52.0, 76.0, FLESH);
            w.ellipse(vec2(0.0, 1.0), vec2(1.6, 0.9), 0.0, GEL);
            w.ellipse(vec2(0.0, 1.0), vec2(0.5, 0.4), 0.0, SEED);
        }
    });
}

/// The inside of a tomato cut across, centred on `p`: three seed pockets.
fn tomato_section(s: &Sketch, p: Vec2) {
    s.circle(p, 10.0, FLESH);
    for i in 0..3 {
        let deg = (i as f32).mul_add(120.0, -90.0);
        let dir = Vec2::from_angle(deg.to_radians());
        let pocket = p + dir * 5.0;
        s.ellipse(pocket, vec2(2.6, 3.6), deg - 90.0, GEL);
        for side in [-1.1, 1.1] {
            s.ellipse(pocket + dir.perp() * side, vec2(0.5, 0.9), deg, SEED);
        }
    }
    s.circle(p, 2.0, light(FLESH, 0.4));
    s.ellipse(
        p + vec2(-6.0, -6.0),
        vec2(1.6, 1.0),
        -45.0,
        light(ink::TOMATO, 0.5),
    );
}

fn soaked_beans(s: &Sketch) {
    // A glass bowl of water, beans sunk in it.
    let water = alpha(ink::WATER, 0.3);
    s.seg(vec2(0.0, RIM_Y), vec2(RIM.x, 13.5), 0.0, 180.0, GLASS);
    s.ellipse(vec2(0.0, RIM_Y), RIM, 0.0, light(GLASS, 0.3));
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(0.0, RIM_Y + 0.3), INSIDE, 0.0, dark(GLASS, 0.1));
    let c = mix(ink::BEAN, ink::CLAY, 0.3);
    bean(s, vec2(-5.0, 6.5), 15.0, 1.2, c);
    bean(s, vec2(5.0, 7.5), -20.0, 1.2, c);
    s.seg(
        vec2(0.0, RIM_Y + 0.6),
        vec2(RIM.x - 0.6, 12.9),
        0.0,
        180.0,
        water,
    );
    s.ellipse(vec2(0.0, RIM_Y + 0.6), INSIDE - vec2(0.3, 0.3), 0.0, water);
    s.band(
        vec2(-0.5, 0.6),
        vec2(7.5, 2.0),
        0.5,
        (0.0, 360.0),
        light(ink::WATER, 0.5),
    );
    bean(s, vec2(-0.5, -0.5), 5.0, 1.2, c);
    s.capsule(
        vec2(-10.0, 3.0),
        vec2(-7.5, 8.5),
        0.9,
        alpha(ink::WHITE, 0.8),
    );
    for (x, y, r) in [(8.0, 1.5, 0.9), (9.5, -2.5, 0.6), (-3.0, 10.0, 0.6)] {
        s.ring(vec2(x, y), r, 0.4, ink::WHITE);
    }
    s.band(
        vec2(0.0, RIM_Y),
        RIM - Vec2::splat(0.75),
        1.5,
        (0.0, 180.0),
        light(GLASS, 0.3),
    );
}

// ---------------------------------------------------------------------------
// Dishes.

fn tea(s: &Sketch, milky: bool) {
    let liquid = if milky { MILK_TEA } else { ink::TEA };
    cup(s, liquid);
    if milky && !s.tracing() {
        let f = light(MILK_TEA, 0.6);
        s.ellipse(vec2(-1.0, -1.0), vec2(1.3, 0.8), 0.0, f);
        s.ellipse(vec2(1.0, -1.0), vec2(1.3, 0.8), 0.0, f);
        s.shape(&[(-2.2, -0.8), (2.2, -0.8), (0.0, 0.9)], f);
    }
}

fn salad(s: &Sketch) {
    bowl(s, ink::WOOD_LIGHT, |s| {
        for (x, deg, c) in [
            (-8.0, -140.0, ink::LEAF),
            (-3.0, -110.0, ink::LEAF_LIGHT),
            (3.0, -75.0, ink::LEAF),
            (8.5, -40.0, ink::LEAF_LIGHT),
            (0.0, -92.0, ink::GRASS),
        ] {
            s.leaf(vec2(x * 0.6, RIM_Y + 1.0), 9.0, 9.0, deg, c);
        }
        s.circle(vec2(-4.5, -4.0), 2.2, ink::TOMATO);
        s.circle(vec2(5.0, -5.5), 2.0, ink::TOMATO);
        coin(s, vec2(1.0, -2.0), 2.4);
        if !s.tracing() {
            s.circle(vec2(-5.0, -4.6), 0.6, light(ink::TOMATO, 0.5));
            s.circle(vec2(4.5, -6.0), 0.6, light(ink::TOMATO, 0.5));
        }
    });
}

fn boiled_egg(s: &Sketch) {
    let cup = ink::GLAZE;
    s.ellipse(vec2(0.0, 13.6), vec2(7.5, 2.3), 0.0, dark(cup, 0.25));
    s.rrect(vec2(0.0, 11.0), vec2(4.6, 5.0), 1.5, dark(cup, 0.15));
    s.ellipse(vec2(0.0, 3.0), vec2(9.0, 2.2), 0.0, dark(cup, 0.4));
    let egg = vec2(0.0, -1.0);
    // The egg below the cut, about 30 degrees above its middle.
    s.seg(egg, vec2(7.6, 10.5), -31.6, 243.2, ink::EGG);
    if !s.tracing() {
        s.ellipse(vec2(-4.0, -3.0), vec2(1.2, 2.2), 15.0, light(ink::EGG, 0.6));
    }
    s.ellipse(vec2(0.0, -6.5), vec2(6.4, 2.2), 0.0, ink::WHITE);
    if !s.tracing() {
        s.ellipse(vec2(0.0, -6.5), vec2(3.6, 1.3), 0.0, ink::YOLK);
        s.ellipse(vec2(-0.8, -6.9), vec2(1.2, 0.4), 0.0, light(ink::YOLK, 0.5));
    }
    s.seg(vec2(0.0, 3.0), vec2(9.0, 7.5), 0.0, 180.0, dark(cup, 0.2));
    if !s.tracing() {
        s.seg(vec2(-0.8, 3.0), vec2(7.8, 6.6), 0.0, 180.0, cup);
        s.ellipse(vec2(-5.0, 5.5), vec2(1.0, 1.8), -40.0, light(cup, 0.5));
    }
}

fn steamed_rice(s: &Sketch) {
    bowl(s, BOWL_BLUE, |s| {
        mound(s, ink::RICE, 9.0);
        if !s.tracing() {
            for (x, y, d) in [
                (-6.0, -3.0, 20.0),
                (-2.0, -6.5, -30.0),
                (3.0, -7.0, 50.0),
                (6.5, -3.5, -20.0),
                (0.5, -3.0, 70.0),
                (-4.0, -6.0, 0.0),
                (2.5, -9.0, -60.0),
            ] {
                grain(s, vec2(x, y), d, dark(ink::RICE, 0.12));
            }
        }
    });
    s.steam(vec2(-3.0, -11.0), 6.0, 0.0);
    s.steam(vec2(3.0, -12.0), 5.0, 2.0);
}

fn soup(s: &Sketch) {
    bowl(s, ink::CERAMIC, |s| {
        surface(s, ink::BROTH);
        if !s.tracing() {
            for (x, y) in [(-5.5, -0.5), (4.0, 0.3), (0.0, -2.2)] {
                s.ellipse(vec2(x, y), vec2(1.7, 1.0), 0.0, ink::CARROT);
                s.ellipse(vec2(x, y), vec2(0.7, 0.4), 0.0, light(ink::CARROT, 0.5));
            }
            s.band(
                vec2(7.5, -1.0),
                vec2(1.5, 1.0),
                0.6,
                (180.0, 180.0),
                ONION_WHITE,
            );
            s.band(
                vec2(-1.5, 0.8),
                vec2(1.5, 1.0),
                0.6,
                (180.0, 180.0),
                ONION_WHITE,
            );
            for (x, y) in [(-2.5, -1.5), (2.0, -0.8), (8.0, 0.5), (-8.0, 0.2)] {
                s.circle(vec2(x, y), 0.45, ink::LEAF);
            }
        }
    });
    s.steam(vec2(-3.0, -4.0), 10.0, 0.0);
    s.steam(vec2(3.0, -4.5), 9.0, 2.0);
}

fn bean_stew(s: &Sketch) {
    bowl(s, ink::CLAY, |s| {
        surface(s, ink::STEW);
        if !s.tracing() {
            for (x, y, d) in [
                (-6.0, -0.5, 20.0),
                (-1.5, -2.0, -10.0),
                (3.5, -0.5, 30.0),
                (7.5, -1.5, -20.0),
                (0.5, 0.8, 0.0),
            ] {
                s.ellipse(vec2(x, y), vec2(1.6, 0.9), d, dark(ink::BEAN, 0.25));
                s.ellipse(
                    vec2(x - 0.3, y - 0.3),
                    vec2(0.6, 0.3),
                    d,
                    light(ink::BEAN, 0.35),
                );
            }
        }
        s.leaf(vec2(-3.5, -1.0), 4.5, 2.4, -30.0, ink::LEAF);
    });
    s.steam(vec2(-3.0, -4.0), 10.0, 0.5);
    s.steam(vec2(3.0, -4.5), 9.0, 2.5);
}

fn custard(s: &Sketch) {
    let dish = ink::CERAMIC;
    s.ellipse(vec2(0.0, 11.0), vec2(11.5, 3.6), 0.0, dark(dish, 0.25));
    s.rrect(vec2(0.0, 5.0), vec2(23.0, 12.0), 0.0, dark(dish, 0.25));
    if !s.tracing() {
        s.ellipse(vec2(-1.0, 10.5), vec2(10.0, 3.2), 0.0, dish);
        s.rrect(vec2(-1.0, 5.0), vec2(20.0, 11.0), 0.0, dish);
        for x in [-8.0, -4.5, -1.0, 2.5, 6.0] {
            s.capsule(vec2(x, 2.5), vec2(x, 11.5), 0.55, dark(dish, 0.12));
        }
    }
    s.ellipse(vec2(0.0, -1.0), vec2(11.5, 4.3), 0.0, light(dish, 0.4));
    s.ellipse(vec2(0.0, -0.8), vec2(9.8, 3.4), 0.0, ink::CUSTARD);
    if !s.tracing() {
        s.ellipse(vec2(1.5, -1.2), vec2(5.0, 1.9), -5.0, CARAMEL);
        s.ellipse(vec2(0.8, -1.6), vec2(2.6, 0.8), -5.0, light(CARAMEL, 0.35));
        s.ellipse(
            vec2(-6.0, -0.4),
            vec2(1.6, 0.6),
            0.0,
            light(ink::CUSTARD, 0.5),
        );
    }
}

fn rice_pudding(s: &Sketch) {
    bowl(s, BOWL_PINK, |s| {
        surface(s, PUDDING);
        s.capsule(vec2(1.5, -3.0), vec2(12.0, -9.0), 1.4, CINNAMON);
        if !s.tracing() {
            for (x, y, d) in [
                (-6.0, 0.3, 20.0),
                (-3.0, -1.8, -30.0),
                (4.5, 0.5, 60.0),
                (-0.5, 0.8, 0.0),
            ] {
                grain(s, vec2(x, y), d, dark(PUDDING, 0.1));
            }
            for (x, y) in [
                (-4.0, 0.0),
                (0.0, -1.2),
                (3.0, 1.0),
                (-7.5, -1.0),
                (6.0, -1.0),
            ] {
                s.circle(vec2(x, y), 0.45, CINNAMON);
            }
            s.capsule(
                vec2(2.5, -3.9),
                vec2(11.5, -9.0),
                0.4,
                light(CINNAMON, 0.35),
            );
            s.ellipse(vec2(12.0, -9.0), vec2(1.4, 1.4), 0.0, light(CINNAMON, 0.2));
        }
    });
}

fn fried_egg(s: &Sketch) {
    plate(s, 7.0, 15.0);
    let s = s.sub(vec2(0.0, 1.0), 1.0, 0.0);
    let white = ink::WHITE;
    for (p, r) in [
        (vec2(-3.0, 4.0), vec2(8.0, 5.2)),
        (vec2(4.0, 5.0), vec2(7.0, 4.6)),
        (vec2(0.0, 1.5), vec2(6.5, 4.4)),
        (vec2(-7.0, 6.0), vec2(4.5, 3.0)),
    ] {
        s.ellipse(p, r, 0.0, white);
    }
    s.circle(vec2(0.5, 1.5), 4.0, dark(ink::YOLK, 0.15));
    if s.tracing() {
        return;
    }
    s.circle(vec2(0.0, 1.0), 3.3, ink::YOLK);
    s.ellipse(
        vec2(-1.2, -0.2),
        vec2(1.2, 0.8),
        -30.0,
        light(ink::YOLK, 0.6),
    );
}

fn omelette(s: &Sketch) {
    plate(s, 7.5, 15.0);
    let o = s.sub(vec2(0.0, 3.0), 1.0, -8.0);
    o.seg(
        vec2(0.0, 4.0),
        vec2(12.0, 11.0),
        180.0,
        180.0,
        dark(OMELETTE, 0.2),
    );
    if s.tracing() {
        return;
    }
    o.seg(vec2(-0.6, 3.0), vec2(11.0, 10.0), 180.0, 180.0, OMELETTE);
    o.band(
        vec2(0.0, 4.0),
        vec2(9.5, 8.5),
        1.0,
        (205.0, 120.0),
        light(OMELETTE, 0.45),
    );
    for (x, y) in [
        (-5.0, -2.0),
        (1.0, -5.0),
        (4.0, 0.0),
        (-1.5, 1.0),
        (6.5, -3.5),
    ] {
        o.ellipse(vec2(x, y), vec2(0.9, 0.5), 30.0, ink::LEAF);
    }
    o.chain(
        &[
            vec2(-6.0, -3.5),
            vec2(-3.0, -6.0),
            vec2(0.0, -3.0),
            vec2(3.0, -6.0),
            vec2(6.0, -3.0),
        ],
        0.6,
        ink::TOMATO,
    );
}

fn pancake(s: &Sketch) {
    plate(s, 9.5, 15.0);
    let (rx, ry) = (11.5, 3.6);
    for i in 0..3 {
        let y = (i as f32).mul_add(-4.0, 6.0);
        s.ellipse(vec2(0.0, y + 2.0), vec2(rx, ry), 0.0, dark(PANCAKE, 0.25));
        s.rrect(vec2(0.0, y + 1.0), vec2(rx * 2.0, 2.0), 0.0, PANCAKE_SIDE);
        if !s.tracing() {
            s.band(
                vec2(0.0, y + 2.0),
                vec2(rx - 0.3, ry - 0.3),
                0.8,
                (0.0, 180.0),
                PANCAKE,
            );
        }
    }
    s.ellipse(vec2(0.0, -2.0), vec2(rx, ry), 0.0, PANCAKE);
    if !s.tracing() {
        s.ellipse(vec2(1.5, -1.6), vec2(7.5, 2.2), 0.0, SYRUP);
        s.capsule(vec2(7.5, -0.6), vec2(7.7, 4.5), 0.9, SYRUP);
        s.capsule(vec2(-4.0, -0.2), vec2(-4.0, 1.8), 0.8, SYRUP);
    }
    s.shape(
        &[
            (-2.5, -4.5),
            (2.0, -4.5),
            (4.0, -2.5),
            (3.5, -1.0),
            (-1.0, -0.5),
            (-3.0, -2.5),
        ],
        ink::BUTTER,
    );
    if s.tracing() {
        return;
    }
    s.shape(
        &[(-2.5, -4.5), (2.0, -4.5), (0.5, -3.2), (-2.0, -3.2)],
        light(ink::BUTTER, 0.5),
    );
    s.ellipse(vec2(-6.0, -2.6), vec2(2.2, 0.6), 0.0, light(PANCAKE, 0.4));
}

fn fried_rice(s: &Sketch) {
    bowl(s, BOWL_TEAL, |s| {
        mound(s, FRIED_RICE, 8.5);
        if s.tracing() {
            return;
        }
        for (x, y) in [(-6.0, -3.0), (2.5, -6.0), (6.0, -2.5), (-1.5, -2.0)] {
            s.circle(vec2(x, y), 1.0, PEA);
        }
        for (x, y) in [(-3.0, -6.0), (4.5, -4.0), (0.0, -8.5)] {
            s.rrect(vec2(x, y), vec2(1.7, 1.7), 0.3, ink::CARROT);
        }
        for (x, y) in [(-8.0, -1.5), (1.0, -4.0), (8.5, -1.0)] {
            s.ellipse(vec2(x, y), vec2(1.4, 0.9), 0.0, ink::WHITE);
            s.ellipse(vec2(x + 0.4, y), vec2(0.8, 0.6), 0.0, ink::YOLK);
        }
    });
}

fn flatbread(s: &Sketch) {
    let back = s.sub(vec2(3.5, -3.5), 0.8, 10.0);
    back.ellipse(Vec2::ZERO, vec2(13.0, 8.0), 0.0, dark(FLATBREAD, 0.2));
    if !s.tracing() {
        back.ellipse(
            vec2(-0.5, -0.5),
            vec2(11.8, 7.0),
            0.0,
            light(FLATBREAD, 0.05),
        );
    }
    let f = s.sub(vec2(-1.0, 3.5), 1.0, -8.0);
    f.ellipse(Vec2::ZERO, vec2(14.0, 9.0), 0.0, dark(FLATBREAD, 0.25));
    if s.tracing() {
        return;
    }
    f.ellipse(vec2(-0.6, -0.6), vec2(12.8, 8.0), 0.0, FLATBREAD);
    for (x, y, r) in [
        (-7.0, -1.0, 2.0),
        (-1.5, -4.0, 1.7),
        (4.0, -1.0, 2.2),
        (8.0, 3.0, 1.5),
        (-3.0, 4.0, 1.8),
        (2.0, 5.0, 1.2),
    ] {
        f.ellipse(vec2(x, y), vec2(r, r * 0.7), 0.0, ink::CRUST);
        f.ellipse(
            vec2(x + 0.3, y + 0.2),
            vec2(r * 0.45, r * 0.3),
            0.0,
            ink::CRUST_DARK,
        );
    }
    for (x, y) in [(-4.5, 1.0), (1.0, -1.5), (6.0, -4.0)] {
        f.circle(vec2(x, y), 0.5, ink::LEAF);
    }
}

fn bread(s: &Sketch) {
    let c = ink::CRUST;
    s.ellipse(vec2(0.0, 1.5), vec2(14.5, 9.5), 0.0, ink::CRUST_DARK);
    s.rrect(vec2(0.0, 7.5), vec2(28.0, 7.0), 3.5, ink::CRUST_DARK);
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-0.8, 0.6), vec2(13.2, 8.6), 0.0, c);
    s.rrect(vec2(-0.8, 6.2), vec2(26.0, 6.0), 3.0, c);
    s.ellipse(vec2(-6.0, -3.5), vec2(3.5, 1.4), -20.0, light(c, 0.4));
    for x in [-6.5, -0.5, 5.5] {
        s.capsule(
            vec2(x - 2.5, 3.0),
            vec2(x + 2.5, -4.5),
            1.4,
            ink::CRUST_DARK,
        );
        s.capsule(vec2(x - 2.7, 2.4), vec2(x + 2.1, -4.8), 1.0, ink::DOUGH);
    }
}

fn cake(s: &Sketch) {
    plate(s, 12.0, 15.0);
    let sponge = SPONGE;
    s.ellipse(vec2(0.0, 10.0), vec2(12.0, 3.4), 0.0, dark(sponge, 0.2));
    s.rrect(vec2(0.0, 4.5), vec2(24.0, 11.0), 0.0, dark(sponge, 0.2));
    if !s.tracing() {
        s.ellipse(vec2(-1.0, 9.6), vec2(11.0, 3.0), 0.0, sponge);
        s.rrect(vec2(-1.0, 4.5), vec2(22.0, 11.0), 0.0, sponge);
        s.rrect(vec2(0.0, 5.5), vec2(24.0, 2.2), 0.0, ink::ICING);
        s.rrect(vec2(0.0, 6.9), vec2(24.0, 0.8), 0.0, ink::BERRY);
    }
    s.ellipse(vec2(0.0, -1.0), vec2(12.0, 3.6), 0.0, ink::ICING);
    for (x, y) in [
        (-9.5, 3.5),
        (-5.0, 2.0),
        (1.0, 4.0),
        (6.5, 2.5),
        (10.5, 1.5),
    ] {
        s.capsule(vec2(x, 0.0), vec2(x, y), 1.5, ink::ICING);
    }
    s.circle(vec2(0.5, -4.5), 3.0, ink::BERRY);
    s.leaf(vec2(1.0, -7.0), 3.5, 2.0, -50.0, ink::LEAF);
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-3.5, -1.6), vec2(4.0, 1.0), 0.0, ink::WHITE);
    s.circle(vec2(-0.6, -5.4), 0.8, light(ink::BERRY, 0.6));
}

// ---------------------------------------------------------------------------
// Waste.

fn mush(s: &Sketch) {
    let c = ink::MUSH;
    let blob = |s: &Sketch, c: Color, k: f32, d: Vec2| {
        s.ellipse(vec2(0.0, 8.0) + d, vec2(14.0, 6.0) * k, 0.0, c);
        s.ellipse(vec2(-1.0, 2.0) + d, vec2(9.5, 8.0) * k, 0.0, c);
        s.ellipse(vec2(5.0, 4.0) + d, vec2(6.0, 6.0) * k, 0.0, c);
    };
    blob(s, dark(c, 0.25), 1.0, Vec2::ZERO);
    s.capsule(vec2(-7.0, 12.0), vec2(-7.0, 15.0), 1.3, dark(c, 0.25));
    s.capsule(vec2(6.0, 12.5), vec2(6.0, 14.0), 1.0, dark(c, 0.25));
    if s.tracing() {
        return;
    }
    blob(s, c, 0.88, vec2(-0.8, -0.8));
    s.ring(vec2(-4.5, -3.0), 1.8, 0.6, light(c, 0.45));
    s.ring(vec2(6.5, 0.5), 1.2, 0.5, light(c, 0.45));
    s.circle(vec2(9.0, 6.0), 0.8, light(c, 0.35));
    let eye = ink::LINE;
    s.ellipse(vec2(-3.5, 4.0), vec2(0.9, 1.3), 0.0, eye);
    s.ellipse(vec2(2.5, 4.0), vec2(0.9, 1.3), 0.0, eye);
    s.capsule(vec2(-5.0, 2.4), vec2(-2.6, 1.4), 0.35, eye);
    s.capsule(vec2(1.6, 1.4), vec2(4.0, 2.4), 0.35, eye);
    s.band(vec2(-0.5, 10.0), vec2(2.4, 1.8), 0.6, (200.0, 140.0), eye);
    s.wisp(vec2(-8.0, -4.0), 8.0, 1.0, alpha(ink::GRASS_DARK, 0.6));
    s.wisp(vec2(9.0, -2.0), 7.0, 3.0, alpha(ink::GRASS_DARK, 0.6));
}

fn charcoal(s: &Sketch) {
    let lump = [
        vec2(-12.0, 7.0),
        vec2(-9.5, -2.0),
        vec2(-3.0, -7.0),
        vec2(6.0, -6.5),
        vec2(12.0, -0.5),
        vec2(12.5, 7.5),
        vec2(5.0, 12.0),
        vec2(-6.0, 11.5),
    ];
    s.poly(&lump, ink::CHAR);
    if s.tracing() {
        return;
    }
    let facet = hex(0x4a_4240);
    s.shape(
        &[(-9.5, -2.0), (-3.0, -7.0), (6.0, -6.5), (1.0, -1.5)],
        facet,
    );
    s.shape(
        &[(1.0, -1.5), (6.0, -6.5), (12.0, -0.5), (7.0, 2.0)],
        hex(0x38_302e),
    );
    s.shape(&[(-12.0, 7.0), (-9.5, -2.0), (-6.0, 3.0)], hex(0x3e_3634));
    let crack = [
        vec2(-8.0, 6.0),
        vec2(-4.0, 3.0),
        vec2(-1.0, 7.0),
        vec2(3.0, 3.5),
        vec2(8.0, 6.5),
    ];
    s.chain(&crack, 1.0, ink::EMBER);
    s.chain(&crack, 0.4, ink::FLAME_HOT);
    s.circle(vec2(-7.0, -1.0), 0.6, ink::FLAME);
    s.wisp(vec2(2.0, -8.0), 8.0, 0.0, alpha(ink::SMOKE, 0.75));
}

// ---------------------------------------------------------------------------
// Processes.

fn knife(s: &Sketch) {
    let s = s.sub(vec2(0.0, 0.5), 1.0, -35.0);
    s.shape(
        &[
            (-3.0, -4.0),
            (10.0, -4.0),
            (16.5, -2.6),
            (12.0, 1.0),
            (7.0, 3.0),
            (-3.0, 3.0),
        ],
        ink::STONE_LIGHT,
    );
    s.rrect(vec2(-10.0, -0.5), vec2(12.0, 5.0), 2.3, ink::WOOD);
    s.rrect(vec2(-3.5, -0.5), vec2(2.4, 7.6), 0.8, ink::IRON_LIGHT);
    if s.tracing() {
        return;
    }
    s.shape(
        &[
            (-2.0, 1.2),
            (8.0, 1.2),
            (14.5, -1.8),
            (12.0, 1.0),
            (7.0, 3.0),
            (-2.0, 3.0),
        ],
        light(ink::STONE_LIGHT, 0.6),
    );
    s.rrect(vec2(3.5, -3.4), vec2(13.0, 1.0), 0.4, ink::STONE);
    s.rrect(vec2(-10.3, -1.5), vec2(10.0, 1.2), 0.6, ink::WOOD_LIGHT);
    for x in [-13.0, -8.0] {
        s.circle(vec2(x, 0.0), 0.8, ink::BRASS);
    }
}

fn stone(s: &Sketch, y: f32, rx: f32, h: f32) {
    let ry = rx * 0.3;
    s.ellipse(vec2(0.0, y + h), vec2(rx, ry), 0.0, ink::STONE_DARK);
    let mid = h.mul_add(0.5, y);
    s.rrect(vec2(0.0, mid), vec2(rx * 2.0, h), 0.0, ink::STONE_DARK);
    if !s.tracing() {
        s.ellipse(
            vec2(-1.0, y + h - 0.3),
            vec2(rx - 1.5, ry - 0.4),
            0.0,
            ink::STONE,
        );
        s.rrect(
            vec2(-1.0, mid),
            vec2(rx.mul_add(2.0, -3.0), h),
            0.0,
            ink::STONE,
        );
    }
    s.ellipse(vec2(0.0, y), vec2(rx, ry), 0.0, ink::STONE_LIGHT);
}

fn millstones(s: &Sketch) {
    s.ellipse(vec2(10.0, 13.0), vec2(4.5, 2.0), 0.0, ink::FLOUR);
    stone(s, 4.0, 14.0, 5.0);
    stone(s, -4.0, 11.0, 4.5);
    s.capsule(vec2(7.0, -4.5), vec2(8.0, -12.0), 1.4, ink::WOOD);
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(0.0, -4.0), vec2(2.4, 0.9), 0.0, ink::STONE_DARK);
    s.band(
        vec2(0.0, -4.0),
        vec2(6.0, 1.8),
        0.5,
        (200.0, 140.0),
        ink::STONE,
    );
    s.capsule(vec2(7.2, -6.0), vec2(7.6, -11.0), 0.5, ink::WOOD_LIGHT);
    for (x, y) in [(13.5, 11.0), (6.5, 12.5)] {
        s.circle(vec2(x, y), 0.7, ink::WHITE);
    }
}

fn churn(s: &Sketch) {
    s.capsule(vec2(0.0, -8.0), vec2(0.0, -14.0), 1.2, ink::WOOD_LIGHT);
    s.circle(vec2(0.0, -14.5), 2.0, ink::WOOD_DARK);
    let body = [
        vec2(-7.0, -7.0),
        vec2(7.0, -7.0),
        vec2(9.5, 11.0),
        vec2(-9.5, 11.0),
    ];
    s.poly(&body, ink::WOOD_DARK);
    s.ellipse(vec2(0.0, 11.0), vec2(9.5, 2.8), 0.0, ink::WOOD_DARK);
    if !s.tracing() {
        s.shape(
            &[(-7.0, -7.0), (4.5, -7.0), (6.5, 11.0), (-9.5, 11.0)],
            ink::WOOD,
        );
        for x in [-4.0, 0.5] {
            s.capsule(vec2(x, -5.0), vec2(x * 1.25, 11.0), 0.35, ink::WOOD_DARK);
        }
        for (y, w) in [(-3.0, 7.6), (7.5, 9.0)] {
            s.rrect(vec2(0.0, y), vec2(w * 2.0, 1.8), 0.0, ink::IRON_LIGHT);
        }
        s.capsule(vec2(-6.0, -3.0), vec2(-7.5, 8.0), 0.8, ink::WOOD_LIGHT);
    }
    s.ellipse(vec2(0.0, -7.0), vec2(7.6, 2.4), 0.0, ink::WOOD_LIGHT);
    s.ellipse(vec2(4.0, -7.5), vec2(2.4, 1.2), 0.0, ink::BUTTER);
    if !s.tracing() {
        s.circle(vec2(0.0, -7.0), 1.4, ink::WOOD_DARK);
    }
}

fn fist(s: &Sketch) {
    let skin = SKIN;
    s.rrect(vec2(0.0, 12.0), vec2(13.0, 6.0), 1.5, SLEEVE);
    s.rrect(vec2(0.0, 1.5), vec2(19.0, 16.0), 5.0, dark(skin, 0.18));
    for i in 0..4 {
        let x = (i as f32).mul_add(4.5, -6.75);
        s.rrect(vec2(x, -4.5), vec2(4.6, 8.0), 2.3, dark(skin, 0.18));
    }
    if s.tracing() {
        return;
    }
    s.rrect(vec2(-0.6, 2.0), vec2(17.0, 14.0), 4.5, skin);
    for i in 0..4 {
        let x = (i as f32).mul_add(4.5, -6.75);
        s.rrect(vec2(x - 0.3, -4.8), vec2(3.6, 7.0), 1.8, skin);
        s.circle(vec2(x - 0.6, -6.6), 0.7, light(skin, 0.5));
    }
    s.capsule(vec2(-8.0, 3.5), vec2(1.5, 4.5), 2.6, dark(skin, 0.2));
    s.capsule(vec2(-8.0, 3.0), vec2(1.2, 4.0), 2.1, skin);
    s.capsule(vec2(-5.5, 2.4), vec2(0.0, 3.0), 0.6, light(skin, 0.45));
    s.rrect(vec2(-0.8, 11.5), vec2(11.0, 5.0), 1.2, light(SLEEVE, 0.2));
}

fn knead(s: &Sketch) {
    dough(&s.sub(vec2(0.0, 5.0), 0.78, 0.0));
    fist(&s.sub(vec2(0.0, -4.0), 0.66, 180.0));
    if !s.tracing() {
        for side in [-1.0_f32, 1.0] {
            let at = |x: f32, y: f32| vec2(side * x, y);
            s.capsule(at(9.0, -2.0), at(11.5, -3.5), 0.5, ink::LINE);
            s.capsule(at(9.5, 1.5), at(12.5, 1.0), 0.5, ink::LINE);
        }
    }
}

fn whisk(s: &Sketch) {
    let b = s.sub(vec2(0.0, 4.0), 0.8, 0.0);
    bowl(&b, BOWL_BLUE, |b| {
        surface(b, BATTER);
        let w = b.sub(vec2(2.0, -8.0), 1.0, 25.0);
        w.capsule(vec2(0.0, -14.0), vec2(0.0, -4.0), 1.8, ink::WOOD);
        w.rrect(vec2(0.0, -4.0), vec2(4.0, 2.4), 0.8, ink::IRON_LIGHT);
        for rx in [1.5, 3.5, 5.5] {
            w.band(
                vec2(0.0, 2.6),
                vec2(rx, 6.4),
                0.9,
                (0.0, 360.0),
                ink::STONE_LIGHT,
            );
        }
    });
}

fn pot(s: &Sketch) {
    let body = ink::IRON_LIGHT;
    s.rrect(vec2(-13.0, 1.0), vec2(4.0, 2.6), 1.2, ink::IRON);
    s.rrect(vec2(13.0, 1.0), vec2(4.0, 2.6), 1.2, ink::IRON);
    s.rrect(vec2(0.0, 5.5), vec2(24.0, 15.0), 3.0, dark(body, 0.3));
    if !s.tracing() {
        s.rrect(vec2(-1.2, 5.2), vec2(21.0, 14.0), 2.6, body);
        s.capsule(vec2(-8.0, 3.0), vec2(-8.0, 9.5), 0.9, light(body, 0.4));
    }
    s.ellipse(vec2(0.0, -2.0), vec2(12.5, 3.5), 0.0, light(body, 0.25));
    s.ellipse(vec2(0.0, -1.7), vec2(10.8, 2.5), 0.0, ink::WATER);
    for (x, y, r) in [
        (-4.0, -6.0, 1.8),
        (2.0, -9.5, 2.2),
        (5.5, -5.0, 1.3),
        (-1.5, -13.5, 1.4),
    ] {
        s.circle(vec2(x, y), r, light(ink::WATER, 0.5));
        if !s.tracing() {
            s.circle(vec2(x, y) - Vec2::splat(r * 0.35), r * 0.3, ink::WHITE);
        }
    }
}

fn pan(s: &Sketch) {
    s.capsule(vec2(6.0, 1.0), vec2(14.5, -7.5), 1.9, ink::WOOD_DARK);
    s.ellipse(vec2(-3.0, 4.0), vec2(12.0, 9.0), 0.0, ink::IRON);
    if s.tracing() {
        return;
    }
    s.ellipse(vec2(-3.0, 4.6), vec2(10.0, 7.0), 0.0, ink::IRON_LIGHT);
    s.ellipse(vec2(-3.0, 5.0), vec2(8.6, 5.8), 0.0, hex(0x55_5052));
    s.ellipse(
        vec2(-6.0, 3.0),
        vec2(3.0, 1.6),
        -20.0,
        alpha(ink::GLOW, 0.6),
    );
    s.capsule(vec2(12.0, -5.0), vec2(13.5, -6.5), 0.6, ink::WOOD_LIGHT);
    for (x, y) in [(-8.0, -7.5), (-2.0, -9.5), (3.5, -7.5)] {
        s.circle(vec2(x, y), 1.0, ink::FLAME_HOT);
    }
}

fn oven(s: &Sketch) {
    s.rrect(vec2(0.0, 10.0), vec2(29.0, 6.0), 1.5, ink::BRICK_DARK);
    s.pie(vec2(0.0, 7.0), 14.0, 180.0, 180.0, ink::BRICK);
    if s.tracing() {
        return;
    }
    s.band(
        vec2(0.0, 7.0),
        vec2(9.6, 9.6),
        0.5,
        (180.0, 180.0),
        ink::BRICK_DARK,
    );
    for deg in [200.0, 235.0, 270.0, 305.0, 340.0] {
        let d = Vec2::from_angle(f32::to_radians(deg));
        s.capsule(
            vec2(0.0, 7.0) + d * 9.8,
            vec2(0.0, 7.0) + d * 13.6,
            0.3,
            ink::BRICK_DARK,
        );
    }
    s.ellipse(
        vec2(-5.0, -2.0),
        vec2(2.0, 1.0),
        -40.0,
        light(ink::BRICK, 0.3),
    );
    s.pie(vec2(0.0, 8.0), 7.5, 180.0, 180.0, ink::SOIL_RICH);
    s.rrect(vec2(0.0, 9.5), vec2(15.0, 3.0), 0.0, ink::SOIL_RICH);
    s.ellipse(vec2(0.0, 10.0), vec2(6.0, 1.5), 0.0, ink::FLAME);
    s.ellipse(vec2(0.0, 6.5), vec2(4.6, 2.8), 0.0, ink::CRUST);
    s.ellipse(vec2(-1.0, 5.6), vec2(2.4, 1.0), 0.0, light(ink::CRUST, 0.4));
    s.rrect(vec2(0.0, 11.0), vec2(16.0, 1.4), 0.0, ink::STONE_LIGHT);
}

fn prove(s: &Sketch) {
    let b = s.sub(vec2(0.0, 5.0), 0.75, 0.0);
    bowl(&b, ink::WOOD_LIGHT, |b| mound(b, ink::DOUGH, 10.0));
    s.chain(
        &[vec2(-5.0, -9.0), vec2(0.0, -14.0), vec2(5.0, -9.0)],
        1.5,
        ink::GOOD,
    );
}

fn soak(s: &Sketch) {
    let b = s.sub(vec2(0.0, 5.0), 0.8, 0.0);
    bowl(&b, BOWL_BLUE, |b| {
        surface(b, ink::WATER);
        if !b.tracing() {
            b.band(
                vec2(0.0, -0.6),
                vec2(5.0, 1.4),
                0.6,
                (0.0, 360.0),
                light(ink::WATER, 0.5),
            );
        }
    });
    drop_shape(s, vec2(0.0, -8.0), 3.6, ink::WATER);
    if !s.tracing() {
        s.circle(vec2(-1.2, -8.5), 0.8, light(ink::WATER, 0.6));
    }
}

fn tea_bag(s: &Sketch) {
    let paper = hex(0xf3_e6c8);
    s.ellipse(vec2(0.0, 12.0), vec2(13.0, 3.0), 0.0, ink::TEA);
    let t = s.sub(vec2(-2.0, 0.0), 1.0, 8.0);
    t.capsule(vec2(0.0, -6.0), vec2(6.0, -11.0), 0.3, ink::STONE_DARK);
    t.rrect(vec2(7.0, -12.0), vec2(6.0, 5.0), 1.0, ink::GOOD);
    t.rrect(vec2(0.0, 3.5), vec2(12.0, 15.0), 2.0, paper);
    t.shape(
        &[(-6.0, -4.0), (6.0, -4.0), (2.5, -7.0), (-2.5, -7.0)],
        paper,
    );
    if s.tracing() {
        return;
    }
    t.rrect(vec2(0.0, 5.5), vec2(9.0, 10.0), 1.5, light(ink::TEA, 0.2));
    t.leaf(vec2(-2.5, 7.0), 6.0, 3.2, -45.0, ink::TEA_LEAF);
    t.rrect(vec2(0.0, -4.0), vec2(12.0, 1.0), 0.0, dark(paper, 0.12));
    t.leaf(vec2(5.5, -11.0), 3.0, 1.8, -20.0, light(ink::GOOD, 0.5));
    s.band(
        vec2(0.0, 12.0),
        vec2(9.0, 1.8),
        0.5,
        (0.0, 360.0),
        light(ink::TEA, 0.3),
    );
}

// ---------------------------------------------------------------------------
// Kinds.

fn flame(s: &Sketch) {
    s.shape(&[(-10.0, 5.0), (-9.0, -7.0), (-4.0, 0.0)], ink::EMBER);
    s.shape(&[(10.0, 5.0), (8.0, -5.0), (4.0, 0.0)], ink::EMBER);
    drop_shape(s, vec2(0.0, 5.0), 9.5, ink::EMBER);
    if s.tracing() {
        return;
    }
    drop_shape(s, vec2(0.0, 6.5), 7.0, ink::FLAME);
    drop_shape(s, vec2(0.0, 8.5), 4.2, ink::FLAME_HOT);
}

fn hourglass(s: &Sketch) {
    for x in [-8.5, 8.5] {
        s.capsule(vec2(x, -11.0), vec2(x, 11.0), 1.0, ink::WOOD_DARK);
    }
    let top = [
        vec2(-7.0, -10.5),
        vec2(7.0, -10.5),
        vec2(1.0, 0.0),
        vec2(-1.0, 0.0),
    ];
    let bottom = [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(7.0, 10.5),
        vec2(-7.0, 10.5),
    ];
    s.poly(&top, GLASS);
    s.poly(&bottom, GLASS);
    if !s.tracing() {
        s.shape(
            &[(-4.0, -5.0), (4.0, -5.0), (0.8, -0.6), (-0.8, -0.6)],
            ink::WHEAT,
        );
        s.capsule(vec2(0.0, -0.5), vec2(0.0, 8.0), 0.4, ink::WHEAT);
        s.shape(&[(-6.6, 10.4), (6.6, 10.4), (0.0, 5.0)], ink::WHEAT);
        s.capsule(vec2(-5.0, -8.5), vec2(-2.0, -3.5), 0.5, ink::WHITE);
    }
    s.rrect(vec2(0.0, -12.0), vec2(21.0, 3.4), 1.4, ink::WOOD);
    s.rrect(vec2(0.0, 12.0), vec2(21.0, 3.4), 1.4, ink::WOOD);
}
