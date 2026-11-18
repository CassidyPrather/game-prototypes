//! Everything Windfall puts on screen.
//!
//! Art direction: a cut-paper diorama of an autumn garden. Every shape is a
//! flat piece of coloured paper with a soft shadow under it, on a cream
//! card; wirenook's creamsicle orange is the hero colour, leaves (never
//! flowers) the motif. The three leaf colours are also three leaf shapes —
//! a maple, a birch and an oak — so no one needs to tell the colours apart.
//!
//! No text. The pieces and the wind are rulers with a leaf on the par, the
//! marks are three paper badges, the gardens are leaf pips along the top.

// Drawing is points, radii and colours, over and over. The float lints ask
// for `mul_add` and `hypot` everywhere; here every value is a pixel drawn
// once and never accumulated, so the formulas stay as written, readable.
#![allow(
    clippy::many_single_char_names,
    clippy::suboptimal_flops,
    clippy::imprecise_flops
)]

use std::f32::consts::{PI, TAU};

use game_prototypes::windfall::garden::{Cell, Dir, Fate, Hue, Leaf, Level, Loss, Marks, Piece};
use game_prototypes::windfall::levels::LEVELS;
use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};

use super::{Game, KickKind, Mode, layout};
use crate::ui::Frame;

const fn hex(rgb: u32) -> Color {
    Color::new(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        1.0,
    )
}

const fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (b.r - a.r).mul_add(t, a.r),
        (b.g - a.g).mul_add(t, a.g),
        (b.b - a.b).mul_add(t, a.b),
        (b.a - a.a).mul_add(t, a.a),
    )
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * 2.0f32.mul_add(-t, 3.0)
}

/// A spring's overshoot, for things popping in: 0 at 0, settling on 1.
fn pop(t: f32) -> f32 {
    if t >= 1.0 {
        return 1.0;
    }
    let t = t.max(0.0);
    1.0 - (1.0 - t).powi(3) * (t * 9.0).cos()
}

// The palette. Paper, and autumn on it.
const SKY_TOP: Color = hex(0xff_f4_e4);
const SKY_LOW: Color = hex(0xfb_dc_bc);
const HILL_FAR: Color = hex(0xd9_c9_9a);
const HILL_NEAR: Color = hex(0xb7_b5_7c);
const GRASS: Color = hex(0x8f_9a_5a);
const GRASS_DARK: Color = hex(0x6e_7a_42);
const CARD: Color = hex(0xff_fa_f0);
const CARD_EDGE: Color = hex(0xec_dc_c4);
const DOT: Color = hex(0xd8_c4_a6);
const INK: Color = hex(0x4a_33_28);
const BARK: Color = hex(0x6b_4a_3a);
const BARK_DARK: Color = hex(0x4e_34_2a);
const CANOPY: [Color; 3] = [hex(0x8a_8f_4e), hex(0xa3_9a_52), hex(0x6f_7a_44)];
const STONE: Color = hex(0xb4_ad_a2);
const STONE_DARK: Color = hex(0x8f_88_7e);
const WICKER: Color = hex(0xc0_8a_52);
const WICKER_DARK: Color = hex(0x95_63_37);
const VANE: Color = hex(0x6f_a8_b8);
const VANE_DARK: Color = hex(0x4c_80_92);
const FLIP: Color = hex(0x9a_7c_c0);
const FLIP_DARK: Color = hex(0x6f_55_96);
const POST: Color = hex(0x8a_6a_50);
const SHADOW: Color = Color::new(0.3, 0.16, 0.06, 0.2);
const LOST: Color = hex(0xd0_4a_3a);
const HIGHLIGHT: Color = hex(0x00f2_8a32);
/// Grey-brown dust, for pieces taken up and leaves that come to grief.
pub const DUST: Color = hex(0xb8_a4_8c);

/// The paper shadow's offset, in frame units.
const LIFT: Vec2 = vec2(2.0, 3.0);

pub const fn leaf_colour(hue: Hue) -> Color {
    match hue {
        // Creamsicle: wirenook's orange.
        Hue::Ember => hex(0xf7_95_4a),
        Hue::Gold => hex(0xe9_c0_3c),
        Hue::Rust => hex(0xb2_45_2e),
    }
}

const fn vein_colour(hue: Hue) -> Color {
    match hue {
        Hue::Ember => hex(0xc8_63_22),
        Hue::Gold => hex(0xb8_8d_1c),
        Hue::Rust => hex(0x7c_2a_1a),
    }
}

/// A frame-unit pen over the frame: positions in frame units in, pixels
/// out, so the drawing code reads in the units the layout is in.
struct Pen<'a> {
    frame: &'a Frame,
}

impl Pen<'_> {
    fn at(&self, p: Vec2) -> Vec2 {
        self.frame.at(p)
    }

    fn circle(&self, p: Vec2, r: f32, c: Color) {
        self.frame.circle(self.at(p), r, c);
    }

    fn rect(&self, p: Vec2, size: Vec2, c: Color) {
        self.frame.rect(self.at(p), size, c);
    }

    fn line(&self, a: Vec2, b: Vec2, w: f32, c: Color) {
        self.frame.line(self.at(a), self.at(b), w, c);
    }

    fn tri(&self, a: Vec2, b: Vec2, c: Vec2, col: Color) {
        Frame::triangle(self.at(a), self.at(b), self.at(c), col);
    }

    /// A filled shape, fanned from `centre`: fine for anything star-shaped
    /// around it, which every leaf is.
    fn fan(&self, centre: Vec2, points: &[Vec2], c: Color) {
        for (i, &a) in points.iter().enumerate() {
            let b = points[(i + 1) % points.len()];
            self.tri(centre, a, b, c);
        }
    }

    /// A rounded rectangle, from its corner.
    fn rounded(&self, p: Vec2, size: Vec2, r: f32, c: Color) {
        let r = r.min(size.x * 0.5).min(size.y * 0.5);
        self.rect(
            p + vec2(r, 0.0),
            vec2(2.0f32.mul_add(-r, size.x), size.y),
            c,
        );
        self.rect(
            p + vec2(0.0, r),
            vec2(size.x, 2.0f32.mul_add(-r, size.y)),
            c,
        );
        for corner in [
            p + Vec2::splat(r),
            p + vec2(size.x - r, r),
            p + vec2(r, size.y - r),
            p + size - Vec2::splat(r),
        ] {
            self.circle(corner, r, c);
        }
    }

    /// A paper cut-out: the shape's shadow, then the shape.
    fn paper(&self, draw: impl Fn(&Self, Vec2, Color), c: Color) {
        draw(self, LIFT, SHADOW);
        draw(self, Vec2::ZERO, c);
    }
}

/// A maple's silhouette, tip up, about two units tall: five pointed lobes
/// and the notch where the stem joins. Star-shaped around the origin.
const MAPLE: [(f32, f32); 22] = [
    (0.0, -1.0),
    (0.16, -0.6),
    (0.36, -0.7),
    (0.3, -0.36),
    (0.86, -0.5),
    (0.66, -0.22),
    (0.8, -0.04),
    (0.48, 0.02),
    (0.62, 0.36),
    (0.22, 0.26),
    (0.06, 0.52),
    (-0.06, 0.52),
    (-0.22, 0.26),
    (-0.62, 0.36),
    (-0.48, 0.02),
    (-0.8, -0.04),
    (-0.66, -0.22),
    (-0.86, -0.5),
    (-0.3, -0.36),
    (-0.36, -0.7),
    (-0.16, -0.6),
    (0.0, -1.0),
];

/// A leaf's outline, `size` from tip to stem, turned by `turn` radians,
/// squashed across by `squash` (a leaf seen edge-on as it tumbles).
fn leaf_outline(hue: Hue, centre: Vec2, size: f32, turn: f32, squash: f32) -> Vec<Vec2> {
    const N: usize = 30;
    let (sin, cos) = turn.sin_cos();
    let place = |local: Vec2| {
        let local = vec2(local.x * squash, local.y) * (size * 0.5);
        centre
            + vec2(
                local.x.mul_add(cos, -local.y * sin),
                local.x.mul_add(sin, local.y * cos),
            )
    };
    if hue == Hue::Ember {
        return MAPLE[..MAPLE.len() - 1]
            .iter()
            .map(|&(x, y)| place(vec2(x, y + 0.2)))
            .collect();
    }
    (0..N)
        .map(|i| {
            let t = i as f32 / N as f32 * TAU;
            // In leaf space: up is the tip.
            let (s, c) = t.sin_cos();
            let r = if hue == Hue::Gold {
                // Birch: a round leaf drawn to a point.
                let point = (c.max(0.0)).powi(8) * 0.35;
                let ellipse = 1.0 / ((s / 0.78).powi(2) + c.powi(2)).sqrt();
                ellipse * 0.72 + point
            } else {
                // Oak: long, with rounded lobes.
                let ellipse = 1.0 / ((s / 0.52).powi(2) + c.powi(2)).sqrt();
                ellipse * 0.16f32.mul_add((9.0 * t).cos().abs(), 0.8)
            };
            place(vec2(t.sin(), -t.cos()) * r)
        })
        .collect()
}

/// One leaf, paper and veins, `size` frame units long.
fn leaf(pen: &Pen, hue: Hue, centre: Vec2, size: f32, turn: f32, squash: f32, fade: f32) {
    let body = leaf_colour(hue);
    let shadow_pts = leaf_outline(hue, centre + LIFT * 0.8, size, turn, squash);
    pen.fan(centre + LIFT * 0.8, &shadow_pts, alpha(SHADOW, fade));
    let pts = leaf_outline(hue, centre, size, turn, squash);
    pen.fan(centre, &pts, alpha(body, fade));
    let (sin, cos) = turn.sin_cos();
    let along = vec2(-sin, cos) * (size * 0.5);
    let vein = alpha(vein_colour(hue), fade);
    let tip = centre - along * if hue == Hue::Ember { 0.55 } else { 0.85 };
    let stem = centre + along * 1.25;
    pen.line(tip, stem, (size * 0.06).max(1.0), vein);
    let across = vec2(cos, sin) * (size * 0.5 * squash);
    let heart = centre + along * 0.25;
    if hue == Hue::Ember {
        for (x, y) in [(0.86, -0.3), (0.62, 0.56)] {
            pen.line(heart, centre + across * x - along * y, 1.0, vein);
            pen.line(heart, centre - across * x - along * y, 1.0, vein);
        }
    } else if size > 18.0 {
        for k in [0.45, -0.05] {
            let from = centre - along * k;
            pen.line(from, from - along * 0.3 + across * 0.45, 1.0, vein);
            pen.line(from, from - along * 0.3 - across * 0.45, 1.0, vein);
        }
    }
}

/// The colour of a piece's pennant, and its darker edge.
const fn piece_colours(piece: Piece) -> (Color, Color) {
    match piece {
        Piece::Vane(_) => (VANE, VANE_DARK),
        Piece::Flip(_) => (FLIP, FLIP_DARK),
    }
}

/// A vane in a cell of side `side`: a post with a paper pennant pointing
/// the way it blows. `swing` turns a flip vane's pennant through its
/// half-turn (0 points `dir`, 1 has come round from the other side);
/// `flutter` shakes it after a gust; `fade` is for ghosts.
#[allow(clippy::too_many_arguments)]
fn vane(
    pen: &Pen,
    piece: Piece,
    dir: Dir,
    centre: Vec2,
    side: f32,
    swing: f32,
    flutter: f32,
    fade: f32,
    fixed: bool,
) {
    let (body, edge) = piece_colours(piece);
    let s = side * 0.42;
    let foot = centre + vec2(0.0, s * 0.95);
    let head = centre - vec2(0.0, s * 0.55);
    // Wind curls ahead of it, brighter in a gust.
    let dx = f32::from(dir.dx());
    let curl = alpha(edge, fade * 0.25f32.mul_add(1.0, flutter * 0.6));
    for (k, y) in [(0.0, -0.15), (0.35, 0.35)] {
        let from = centre + vec2(dx * s * (0.55 + k), s * y);
        pen.line(from, from + vec2(dx * s * 0.45, 0.0), 1.6, curl);
        pen.circle(from + vec2(dx * s * 0.45, -s * 0.06), s * 0.07, curl);
    }
    if fixed {
        // Set in a stone: the garden's own, not the player's.
        pen.paper(
            |p, o, c| {
                p.rounded(
                    foot + o - vec2(s * 0.5, s * 0.22),
                    vec2(s, s * 0.36),
                    s * 0.15,
                    c,
                );
            },
            alpha(STONE, fade),
        );
    }
    pen.paper(
        |p, o, c| p.line(foot + o, head + o, (side * 0.06).max(2.0), c),
        alpha(POST, fade),
    );
    // The pennant: a triangle from the post, its far point the way it
    // blows. Seen side-on while a flip vane swings, it narrows to nothing.
    let facing = if swing > 0.0 {
        (PI * swing).cos() * dx
    } else {
        dx
    };
    let wobble = flutter * (flutter * 40.0).sin() * 0.12;
    let reach = s * 1.15 * facing;
    let top = head;
    let low = head + vec2(0.0, s * 0.62);
    let tip = head + vec2(reach, s * (0.31 + wobble));
    pen.tri(top + LIFT, low + LIFT, tip + LIFT, alpha(SHADOW, fade));
    pen.tri(top, low, tip, alpha(body, fade));
    pen.line(top, tip, 1.2, alpha(edge, fade));
    pen.line(low, tip, 1.2, alpha(edge, fade));
    if matches!(piece, Piece::Flip(_)) {
        // A flip vane's pivot, and a notch on its other side to say it
        // turns.
        pen.circle(head + vec2(0.0, s * 0.31), s * 0.13, alpha(edge, fade));
        pen.circle(
            head + vec2(-dx * s * 0.28, s * 0.31),
            s * 0.06,
            alpha(edge, fade * 0.7),
        );
    }
    pen.circle(head, s * 0.08, alpha(BARK_DARK, fade));
}

/// A basket on the ground, with its leaf token, and how many leaves in it.
fn basket(pen: &Pen, hue: Hue, centre: Vec2, side: f32, full: usize, clock: f32) {
    let w = side * 0.78;
    let h = side * 0.46;
    let rim = centre.y - h * 0.2;
    // The leaves already in it peek over the rim.
    for k in 0..full.min(6) {
        let k = k as f32;
        let x = centre.x + ((k * 2.3).sin() * 0.3) * w;
        leaf(
            pen,
            hue,
            vec2(x, rim - side * 0.04 - (k * 0.7).cos().abs() * side * 0.08),
            side * 0.34,
            (k * 1.9).sin() * 0.9,
            1.0,
            1.0,
        );
    }
    let body = |p: &Pen, o: Vec2, c: Color| {
        let a = vec2(centre.x - w * 0.5, rim) + o;
        let b = vec2(centre.x + w * 0.5, rim) + o;
        let cc = vec2(centre.x + w * 0.36, rim + h) + o;
        let d = vec2(centre.x - w * 0.36, rim + h) + o;
        p.tri(a, b, cc, c);
        p.tri(a, cc, d, c);
    };
    pen.paper(body, WICKER);
    for k in 1..4 {
        let t = k as f32 / 4.0;
        let y = h.mul_add(t, rim);
        let half = (w * 0.5).mul_add(1.0 - t, w * 0.36 * t);
        pen.line(
            vec2(centre.x - half, y),
            vec2(centre.x + half, y),
            1.4,
            WICKER_DARK,
        );
    }
    pen.rounded(
        vec2(centre.x - w * 0.55, rim - side * 0.05),
        vec2(w * 1.1, side * 0.1),
        side * 0.05,
        WICKER_DARK,
    );
    // The token: which leaves belong here.
    let sway = (clock * 1.3 + centre.x * 0.05).sin() * 0.08;
    let tag = vec2(centre.x, rim + h * 0.52);
    pen.circle(tag + LIFT * 0.5, side * 0.15, SHADOW);
    pen.circle(tag, side * 0.15, CARD);
    leaf(pen, hue, tag, side * 0.24, sway, 1.0, 1.0);
}

/// The backdrop: sky, two paper hills, grass.
fn backdrop(pen: &Pen, frame: &Frame, clock: f32) {
    let (lo, hi) = frame.window_bounds();
    frame.gradient(frame.at(lo), hi - lo, [SKY_TOP, SKY_TOP, SKY_LOW, SKY_LOW]);
    for (base, amp, colour, phase) in [(360.0, 26.0, HILL_FAR, 0.0), (420.0, 20.0, HILL_NEAR, 2.0)]
    {
        let steps = 40;
        let width = hi.x - lo.x;
        for i in 0..steps {
            let x0 = (i as f32 / steps as f32).mul_add(width, lo.x);
            let x1 = ((i + 1) as f32 / steps as f32).mul_add(width, lo.x);
            let y = |x: f32| {
                base - amp
                    * ((x * 0.009 + phase).sin() * 0.7 + (x * 0.021 + phase * 1.7).sin() * 0.3)
            };
            let (a, b) = (vec2(x0, y(x0)), vec2(x1, y(x1)));
            pen.tri(a, b, vec2(x1, hi.y), colour);
            pen.tri(a, vec2(x1, hi.y), vec2(x0, hi.y), colour);
        }
    }
    pen.rect(vec2(lo.x, 500.0), vec2(hi.x - lo.x, hi.y - 500.0), GRASS);
    pen.rect(vec2(lo.x, 500.0), vec2(hi.x - lo.x, 3.0), GRASS_DARK);
    // Tufts.
    for k in 0..24 {
        let x = (k as f32).mul_add(37.0, 12.0) % 820.0 - 10.0;
        let y = 512.0 + ((k * 7) % 5) as f32 * 14.0;
        let lean = (clock * 0.9 + k as f32).sin() * 1.5;
        for (dx, h) in [(-3.0, 7.0), (0.0, 10.0), (3.0, 7.0)] {
            pen.line(
                vec2(x + dx, y),
                vec2(x + dx * 1.6 + lean, y - h),
                1.4,
                GRASS_DARK,
            );
        }
    }
}

/// The tree's branch over the garden, and a twig down to each column the
/// tree drops from.
fn branch(pen: &Pen, level: &Level, clock: f32) {
    let side = layout::cell(level);
    let y = layout::spot(level, 0.0, -1.0).y - side * 0.55;
    let left = layout::spot(level, -1.2, -1.0).x;
    let right = layout::spot(level, f32::from(level.width) + 0.2, -1.0).x;
    pen.paper(
        |p, o, c| {
            p.line(
                vec2(-40.0, y - side * 0.5) + o,
                vec2(left, y) + o,
                side * 0.22,
                c,
            );
            p.line(
                vec2(left, y) + o,
                vec2(right, y + side * 0.08) + o,
                side * 0.14,
                c,
            );
        },
        BARK,
    );
    let mut columns: Vec<i8> = level.drops.iter().map(|d| d.x).collect();
    columns.sort_unstable();
    columns.dedup();
    for x in columns {
        let at = layout::spot(level, f32::from(x), -1.0);
        pen.line(vec2(at.x, y), at - vec2(0.0, side * 0.28), 2.0, BARK_DARK);
    }
    // Foliage: paper blobs along the branch, breathing a little.
    let span = right - left;
    // The span is positive, so the count is too.
    #[allow(clippy::cast_sign_loss)]
    let blobs = (span / (side * 0.5)).ceil().max(0.0) as usize + 3;
    for k in 0..blobs {
        let t = k as f32 / (blobs - 1).max(1) as f32;
        let x = (span + side).mul_add(t, left - side * 0.5);
        let bob = (clock * 0.8 + k as f32 * 1.3).sin() * 1.5;
        let r = side * (0.32 + 0.12 * ((k * 5 % 3) as f32 / 2.0));
        let yy = y - side * 0.3 - ((k * 3) % 4) as f32 * side * 0.07 + bob;
        let c = CANOPY[k % CANOPY.len()];
        pen.circle(vec2(x, yy) + LIFT, r, SHADOW);
        pen.circle(vec2(x, yy), r, c);
    }
}

/// Where a leaf is drawn, how it is turned and squashed, and how faded,
/// `alpha` of the way through the current tick.
fn leaf_pose(
    level: &Level,
    n: usize,
    l: &Leaf,
    alpha: f32,
    clock: f32,
    age: f32,
) -> (Vec2, f32, f32, f32) {
    let side = layout::cell(level);
    let seed = n as f32 * 1.7;
    let from = layout::centre(level, l.from);
    let to = layout::centre(level, l.at);
    if l.fate == Fate::Hanging {
        let sway = (clock * 1.6 + seed).sin() * 0.18;
        let at = layout::spot(level, f32::from(l.at.x), -1.0) + vec2(0.0, side * 0.05);
        (at, PI + sway, 1.0, 1.0)
    } else {
        let t = smooth(alpha);
        let mut p = from.lerp(to, t);
        let falling = l.at.x == l.from.x && l.at.y > l.from.y;
        if falling {
            p.x += (t * PI).sin() * side * 0.12 * if n % 2 == 0 { 1.0 } else { -1.0 };
        }
        let tumble = clock.mul_add(2.2, seed);
        let squash = 0.35f32.mul_add(tumble.sin().abs(), 0.65);
        let mut turn = tumble.sin() * 0.7;
        let mut fade = 1.0;
        match l.fate {
            Fate::Lost(Loss::Blown, _) => {
                p.x += f32::from(l.at.x - l.from.x) * side * age * 2.5;
                p.y -= side * age * 1.2;
                fade = (1.0 - age).max(0.0);
            }
            Fate::Lost(Loss::Eddy, _) => {
                turn = clock * 9.0;
                fade = (1.0 - age).max(0.0);
            }
            Fate::Lost(..) => {
                // Lying where it fell, gone dull.
                turn = 1.4 + seed;
                fade = 0.75;
            }
            _ => {}
        }
        (p, turn, squash, fade)
    }
}

/// A small particle of paper.
#[derive(Clone, Copy, Debug)]
pub struct Mote {
    p: Vec2,
    v: Vec2,
    age: f32,
    life: f32,
    colour: Color,
    size: f32,
}

/// A burst of paper bits at `p`, starting `delay` seconds from now.
pub fn burst(motes: &mut Vec<Mote>, p: Vec2, colour: Color, count: usize, delay: f32) {
    for k in 0..count {
        let angle = (k as f32 / count as f32).mul_add(TAU, k as f32 * 0.37) - PI * 0.5;
        let speed = 50.0 + ((k * 37) % 23) as f32 * 3.0;
        motes.push(Mote {
            p,
            v: Vec2::from_angle(angle) * speed - vec2(0.0, 40.0),
            age: -delay,
            life: 0.7,
            colour,
            size: 2.0 + (k % 3) as f32,
        });
    }
}

/// A gentle puff, for a piece taken up.
pub fn puff(motes: &mut Vec<Mote>, p: Vec2, colour: Color, count: usize) {
    for k in 0..count {
        let angle = k as f32 / count as f32 * TAU;
        motes.push(Mote {
            p,
            v: Vec2::from_angle(angle) * 30.0,
            age: 0.0,
            life: 0.45,
            colour,
            size: 3.0,
        });
    }
}

pub fn drift(motes: &mut Vec<Mote>, dt: f32) {
    for m in motes.iter_mut() {
        m.age += dt;
        if m.age > 0.0 {
            m.p += m.v * dt;
            m.v.y += 160.0 * dt;
            m.v *= 1.0 - (dt * 2.0).min(1.0);
        }
    }
    motes.retain(|m| m.age < m.life);
}

/// When the `n`th mark appears, in seconds after the garden settles.
pub fn mark_born(n: u8) -> f32 {
    f32::from(n).mul_add(0.32, 0.55)
}

/// The three marks' pictures: a basket (every leaf home), a vane (few
/// pieces), a curl of wind (little wind).
fn badge(pen: &Pen, n: usize, centre: Vec2, r: f32, lit: bool, fade: f32) {
    let ink = if lit { CARD } else { alpha(INK, 0.5) };
    if lit {
        pen.circle(centre + LIFT, r, alpha(SHADOW, fade));
        pen.circle(centre, r, alpha(HIGHLIGHT, fade));
    } else {
        pen.frame
            .ring(pen.at(centre), r - 1.0, 1.5, alpha(INK, 0.3 * fade));
    }
    let s = r * 0.55;
    match n {
        0 => {
            let a = centre + vec2(-s, -s * 0.3);
            let b = centre + vec2(s, -s * 0.3);
            pen.tri(a, b, centre + vec2(s * 0.7, s * 0.8), alpha(ink, fade));
            pen.tri(
                a,
                centre + vec2(s * 0.7, s * 0.8),
                centre + vec2(-s * 0.7, s * 0.8),
                alpha(ink, fade),
            );
        }
        1 => {
            pen.line(
                centre + vec2(-s * 0.4, s),
                centre + vec2(-s * 0.4, -s),
                2.0,
                alpha(ink, fade),
            );
            pen.tri(
                centre + vec2(-s * 0.4, -s),
                centre + vec2(-s * 0.4, s * 0.1),
                centre + vec2(s, -s * 0.45),
                alpha(ink, fade),
            );
        }
        _ => {
            pen.line(
                centre + vec2(-s, s * 0.1),
                centre + vec2(s * 0.5, s * 0.1),
                2.0,
                alpha(ink, fade),
            );
            pen.frame.arc(
                pen.at(centre + vec2(s * 0.5, -s * 0.25)),
                s * 0.3,
                2.0,
                270.0,
                200.0,
                alpha(ink, fade),
            );
            pen.line(
                centre + vec2(-s * 0.6, s * 0.6),
                centre + vec2(s * 0.2, s * 0.6),
                2.0,
                alpha(ink, fade),
            );
        }
    }
}

/// A ruler of `value` ticks with a leaf on `par`: pieces, or wind.
fn ruler(pen: &Pen, y: f32, value: usize, par: usize, ok: bool, icon: usize, clock: f32) {
    let x0 = layout::RULER_X;
    let span = (value.max(par) + 2).max(8);
    let gap = (300.0 / span as f32).min(14.0);
    // The icon at the left: a vane for pieces, a curl for wind.
    let at = vec2(x0 - 26.0, y);
    match icon {
        0 => vane(
            pen,
            Piece::Vane(Dir::Right),
            Dir::Right,
            at + vec2(-4.0, -2.0),
            34.0,
            0.0,
            0.0,
            1.0,
            false,
        ),
        _ => badge(pen, 2, at, 11.0, false, 1.6),
    }
    pen.line(
        vec2(x0, y + 7.0),
        vec2((span as f32).mul_add(gap, x0), y + 7.0),
        1.5,
        alpha(INK, 0.35),
    );
    for k in 0..=span {
        let x = (k as f32).mul_add(gap, x0);
        let long = k % 5 == 0;
        pen.line(
            vec2(x, y + 7.0),
            vec2(x, y + if long { -3.0 } else { 2.0 }),
            1.2,
            alpha(INK, 0.4),
        );
    }
    if value > 0 {
        let colour = if ok { HIGHLIGHT } else { alpha(INK, 0.55) };
        pen.rounded(
            vec2(x0, y - 1.0),
            vec2(value as f32 * gap, 6.0),
            3.0,
            colour,
        );
    }
    let px = (par as f32).mul_add(gap, x0);
    let sway = (clock * 2.0).sin() * 0.1;
    leaf(
        pen,
        Hue::Ember,
        vec2(px, y - 10.0),
        13.0,
        PI + sway,
        1.0,
        1.0,
    );
}

/// The speaker, with a slash when muted.
fn speaker(pen: &Pen, centre: Vec2, muted: bool) {
    let c = alpha(INK, 0.7);
    pen.rect(centre + vec2(-10.0, -4.0), vec2(6.0, 8.0), c);
    pen.tri(
        centre + vec2(-6.0, -4.0),
        centre + vec2(2.0, -10.0),
        centre + vec2(2.0, 10.0),
        c,
    );
    pen.tri(
        centre + vec2(-6.0, -4.0),
        centre + vec2(2.0, 10.0),
        centre + vec2(-6.0, 4.0),
        c,
    );
    if muted {
        pen.line(centre + vec2(6.0, -6.0), centre + vec2(14.0, 6.0), 2.0, c);
        pen.line(centre + vec2(14.0, -6.0), centre + vec2(6.0, 6.0), 2.0, c);
    } else {
        pen.frame
            .arc(pen.at(centre + vec2(2.0, 0.0)), 6.0, 1.8, -45.0, 90.0, c);
        pen.frame
            .arc(pen.at(centre + vec2(2.0, 0.0)), 11.0, 1.8, -45.0, 90.0, c);
    }
}

/// A paper key cap with a drawing on it.
fn cap(pen: &Pen, centre: Vec2, w: f32, glow: f32, draw: impl Fn(&Pen, Vec2, Color)) {
    let size = vec2(w, layout::CAP);
    let corner = centre - size * 0.5;
    pen.rounded(corner + LIFT, size, 7.0, SHADOW);
    let face = mix(CARD, hex(0xff_d9_b0), glow);
    pen.rounded(corner, size, 7.0, face);
    if glow > 0.0 {
        pen.frame.rect_lines(
            pen.at(corner - Vec2::splat(2.0)),
            size + Vec2::splat(4.0),
            2.0,
            alpha(HIGHLIGHT, glow),
        );
    }
    draw(pen, centre, INK);
}

// One pass down the layers of the diorama, back to front.
#[allow(clippy::too_many_lines)]
pub fn scene(frame: &Frame, game: &Game) {
    let pen = Pen { frame };
    let level = &LEVELS[game.level];
    let side = layout::cell(level);
    let clock = game.clock;
    backdrop(&pen, frame, clock);

    // The bed: a cream card the garden is laid out on.
    let origin = layout::origin(level);
    let bed = vec2(f32::from(level.width), f32::from(level.height)) * side;
    let pad = side * 0.25;
    pen.rounded(
        origin - Vec2::splat(pad) + LIFT * 1.5,
        bed + Vec2::splat(pad * 2.0),
        12.0,
        SHADOW,
    );
    pen.rounded(
        origin - Vec2::splat(pad),
        bed + Vec2::splat(pad * 2.0),
        12.0,
        CARD_EDGE,
    );
    pen.rounded(
        origin - Vec2::splat(pad - 3.0),
        bed + Vec2::splat(pad * 2.0 - 6.0),
        10.0,
        CARD,
    );
    for y in 0..=level.height {
        for x in 0..=level.width {
            pen.circle(origin + vec2(f32::from(x), f32::from(y)) * side, 1.6, DOT);
        }
    }
    for &stone in level.stones {
        let c = layout::centre(level, stone);
        pen.paper(
            |p, o, col| p.circle(c + o + vec2(-side * 0.1, side * 0.05), side * 0.22, col),
            STONE,
        );
        pen.paper(
            |p, o, col| p.circle(c + o + vec2(side * 0.14, side * 0.12), side * 0.15, col),
            STONE_DARK,
        );
    }

    // The hover: a ghost of what a click would put there.
    if game.mode == Mode::Build
        && let Some(at) = game.hover
        && level.open(at)
    {
        let c = layout::centre(level, at);
        pen.rounded(
            c - Vec2::splat(side * 0.46),
            Vec2::splat(side * 0.92),
            8.0,
            alpha(HIGHLIGHT, 0.12),
        );
        let next = Piece::next(game.plan().get(at), level.kit);
        if let Some(piece) = next
            && game.plan().get(at).is_none()
        {
            let dir = match piece {
                Piece::Vane(d) | Piece::Flip(d) => d,
            };
            vane(&pen, piece, dir, c, side, 0.0, 0.0, 0.35, false);
        }
    }

    // Trails: where each leaf will go, while building.
    if game.mode == Mode::Build {
        trails(&pen, game, level, side, clock);
    }

    // The pieces, as the run has them now.
    for y in 0..level.height {
        for x in 0..level.width {
            let at = Cell::new(x, y);
            let Some(piece) = game.run.piece(at) else {
                continue;
            };
            let dir = match piece {
                Piece::Vane(d) | Piece::Flip(d) => d,
            };
            let mut scale = 1.0;
            let mut swing = 0.0;
            let mut flutter = 0.0;
            for kick in game.kicks.iter().filter(|k| k.at == at) {
                match kick.kind {
                    KickKind::Placed => scale = pop(kick.age / 0.35),
                    KickKind::Gust => flutter = (1.0 - kick.age / 0.5).max(0.0),
                    KickKind::Turned => swing = (1.0 - smooth(kick.age / 0.3)).max(0.0),
                }
            }
            let c = layout::centre(level, at);
            vane(
                &pen,
                piece,
                dir,
                c,
                side * scale,
                swing,
                flutter,
                1.0,
                level.is_fixed(at),
            );
        }
    }

    branch(&pen, level, clock);

    // Baskets, with however many leaves have come home to each.
    for b in level.baskets {
        let full = game
            .run
            .leaves()
            .iter()
            .filter(|l| {
                matches!(l.fate, Fate::Home(t) if t < game.run.tick().saturating_sub(1))
                    && l.at.x == b.x
            })
            .count();
        let c = layout::spot(level, f32::from(b.x), f32::from(level.height) + 0.35);
        basket(&pen, b.hue, c, side, full, clock);
    }

    // The leaves.
    let alpha_t = (game.since / game.tick_secs()).clamp(0.0, 1.0);
    for (n, l) in game.run.leaves().iter().enumerate() {
        let just = |t: u16| t + 1 >= game.run.tick();
        let age = match l.fate {
            Fate::Lost(_, t) | Fate::Home(t) if !just(t) => {
                1.0 + f32::from(game.run.tick() - t) * 0.3
            }
            _ => alpha_t,
        };
        if let Fate::Home(t) = l.fate
            && !just(t)
        {
            continue; // In the basket now, drawn with it.
        }
        if l.fate == Fate::Hanging {
            // The queue: the next leaf to fall hangs from the twig; the
            // ones after it wait their turn along the branch, smaller.
            let ahead = game.run.leaves()[..n]
                .iter()
                .filter(|o| o.fate == Fate::Hanging && o.at.x == l.at.x)
                .count();
            if ahead > 0 {
                let twig = layout::spot(level, f32::from(l.at.x), -1.0);
                let sway = (clock * 1.4 + n as f32).sin() * 0.15;
                let p = twig + vec2(ahead as f32 * side * 0.2 + side * 0.06, -side * 0.42);
                leaf(&pen, l.hue, p, side * 0.27, PI + sway, 1.0, 1.0);
                continue;
            }
        }
        let (p, turn, squash, fade) = leaf_pose(level, n, l, alpha_t, clock, age);
        if fade <= 0.0 {
            continue;
        }
        leaf(&pen, l.hue, p, side * 0.42, turn, squash, fade);
        if matches!(l.fate, Fate::Lost(Loss::WrongBasket | Loss::Ground, _)) && age > 0.9 {
            cross(&pen, p + vec2(side * 0.18, -side * 0.18), side * 0.1, LOST);
        }
    }

    for m in &game.motes {
        if m.age > 0.0 {
            let fade = 1.0 - m.age / m.life;
            pen.circle(m.p, m.size * fade.max(0.2), alpha(m.colour, fade));
        }
    }

    hud(&pen, game, level, clock);
}

fn cross(pen: &Pen, at: Vec2, r: f32, c: Color) {
    pen.line(at - Vec2::splat(r), at + Vec2::splat(r), 2.5, c);
    pen.line(at + vec2(-r, r), at + vec2(r, -r), 2.5, c);
}

/// Each leaf's future, as a dotted line in its colour ending in a tick or
/// a cross. Shared stretches are offset a hair per leaf so each stays
/// visible.
fn trails(pen: &Pen, game: &Game, level: &Level, side: f32, clock: f32) {
    let preview = &game.preview;
    // Each stretch of path once, with the colours of every leaf that takes
    // it taking turns along its dots.
    let mut stretches: Vec<(Vec2, Vec2, Vec<Hue>)> = Vec::new();
    for (n, path) in preview.paths.iter().enumerate() {
        let hue = level.drops[n].hue;
        let mut prev = layout::spot(level, f32::from(level.drops[n].x), -1.0);
        for &at in path {
            let p = layout::centre(level, at);
            if let Some(s) = stretches.iter_mut().find(|s| s.0 == prev && s.1 == p) {
                if !s.2.contains(&hue) {
                    s.2.push(hue);
                }
            } else {
                stretches.push((prev, p, vec![hue]));
            }
            prev = p;
        }
    }
    let flow = (clock * 1.2).fract();
    for (a, b, hues) in &stretches {
        let steps = 4;
        for s in 0..steps {
            let t = (s as f32 + flow) / steps as f32;
            let hue = hues[s % hues.len()];
            pen.circle(a.lerp(*b, t), 2.0, alpha(leaf_colour(hue), 0.6));
        }
    }
    for (n, path) in preview.paths.iter().enumerate() {
        if let (Some(&end), Fate::Lost(..)) = (path.last(), preview.fates[n]) {
            cross(pen, layout::centre(level, end), side * 0.09, LOST);
        }
    }
    // In the first gardens, a ring pulses where the first lost leaf's trail
    // begins: the place to try a vane.
    if game.level <= 2
        && game.plan().count() == 0
        && let Some(n) = preview.first_lost
        && let Some(&at) = preview.paths[n].first()
    {
        let c = layout::centre(level, at);
        let pulse = (clock * 3.0).sin().mul_add(0.5, 0.5);
        pen.frame.ring(
            pen.at(c),
            side * (0.3 + 0.06 * pulse),
            2.5,
            alpha(HIGHLIGHT, 0.4 + 0.5 * pulse),
        );
    }
}

// The pips, the rulers, the marks and the caps, in that order.
#[allow(clippy::too_many_lines)]
fn hud(pen: &Pen, game: &Game, level: &Level, clock: f32) {
    let pulse = (clock * 3.0).sin().mul_add(0.5, 0.5);
    // The gardens, as leaf pips: an outline until won, filled once won,
    // with a dot for each further mark.
    for n in 0..LEVELS.len() {
        let at = layout::pip(n, LEVELS.len());
        let best = game.best[n];
        let current = n == game.level;
        let size = if current { 22.0 } else { 16.0 };
        if best.home {
            leaf(pen, Hue::Ember, at, size, PI, 1.0, 1.0);
        } else {
            let pts = leaf_outline(Hue::Ember, at, size, PI, 1.0);
            pen.frame.outline(
                &pts.iter().map(|&p| pen.at(p)).collect::<Vec<_>>(),
                1.2,
                alpha(INK, if current { 0.8 } else { 0.35 }),
            );
        }
        let extra = u8::from(best.few) + u8::from(best.gentle);
        for k in 0..extra {
            pen.circle(
                at + vec2(f32::from(k).mul_add(6.0, -3.0), 15.0),
                2.0,
                HIGHLIGHT,
            );
        }
        if current {
            pen.line(
                at + vec2(-8.0, 20.0),
                at + vec2(8.0, 20.0),
                2.0,
                alpha(INK, 0.6),
            );
        }
    }
    speaker(pen, layout::SPEAKER, game.audio.is_muted());

    // A paper strip the rulers and marks sit on, clear of the grass.
    let strip = vec2(10.0, 538.0);
    let strip_size = vec2(540.0, 54.0);
    pen.rounded(strip + LIFT, strip_size, 10.0, SHADOW);
    pen.rounded(strip, strip_size, 10.0, alpha(CARD, 0.94));

    // The rulers: pieces placed, and wind blown, each with its par.
    let preview = &game.preview;
    let ok = preview.marks.home;
    ruler(
        pen,
        layout::RULER_Y[0],
        game.plan().count(),
        usize::from(level.par_pieces),
        ok && preview.marks.few,
        0,
        clock,
    );
    let wind = if game.mode == Mode::Build {
        preview.gusts
    } else {
        game.run.gusts()
    };
    ruler(
        pen,
        layout::RULER_Y[1],
        usize::from(wind),
        usize::from(level.par_gusts),
        ok && preview.marks.gentle,
        1,
        clock,
    );

    // The marks: this garden's best, lit; what the plan would earn,
    // outlined; and as a garden is won, each rung in turn.
    let best = game.best[game.level];
    let now = Marks::of(&game.run);
    let shown = |n: usize, m: Marks| [m.home, m.few, m.gentle][n];
    for n in 0..3 {
        let at = layout::mark(n);
        let lit = shown(n, best);
        let mut r = 13.0;
        if let Mode::Over { age, .. } = game.mode
            && shown(n, now)
        {
            let born = mark_born(n as u8);
            r *= pop(((age - born) / 0.4).max(0.0)).max(1.0);
            if age < born {
                r = 13.0;
            }
        }
        let fade = if lit || (game.mode == Mode::Build && shown(n, preview.marks)) {
            1.0
        } else {
            0.6
        };
        badge(pen, n, at, r, lit, fade);
        if !lit && game.mode == Mode::Build && shown(n, preview.marks) {
            pen.frame.ring(
                pen.at(at),
                r + 3.0,
                1.5,
                alpha(HIGHLIGHT, 0.5 + 0.4 * pulse),
            );
        }
    }

    // The caps.
    let building = game.mode == Mode::Build;
    let ready = building && preview.marks.home;
    let glow_play = if ready || (!game.played && building) {
        pulse
    } else if matches!(game.mode, Mode::Over { .. }) && game.run.all_home() {
        pulse * 0.6
    } else {
        0.0
    };
    cap(pen, layout::CLEAR, layout::CAP, 0.0, |p, c, ink| {
        let ink = alpha(ink, if building { 0.8 } else { 0.3 });
        cross(p, c, 7.0, ink);
    });
    cap(pen, layout::UNDO, layout::CAP, 0.0, |p, c, ink| {
        let ink = alpha(
            ink,
            if building && !game.undo.is_empty() {
                0.8
            } else {
                0.3
            },
        );
        p.frame
            .arc(p.at(c + vec2(1.0, 2.0)), 7.0, 2.2, 180.0, 220.0, ink);
        p.tri(
            c + vec2(-11.0, 1.0),
            c + vec2(-3.0, 1.0),
            c + vec2(-7.0, -6.0),
            ink,
        );
    });
    cap(
        pen,
        layout::FAST,
        layout::CAP,
        if game.fast { 0.6 } else { 0.0 },
        |p, c, ink| {
            for dx in [-6.0, 3.0] {
                p.tri(
                    c + vec2(dx - 3.0, -7.0),
                    c + vec2(dx - 3.0, 7.0),
                    c + vec2(dx + 6.0, 0.0),
                    ink,
                );
            }
        },
    );
    let over_won = matches!(game.mode, Mode::Over { .. }) && game.run.all_home();
    cap(pen, layout::PLAY, layout::PLAY_W, glow_play, |p, c, ink| {
        if building {
            p.tri(
                c + vec2(-6.0, -9.0),
                c + vec2(-6.0, 9.0),
                c + vec2(9.0, 0.0),
                ink,
            );
        } else if over_won && game.level + 1 < LEVELS.len() {
            // On to the next garden: a leaf with an arrow.
            leaf(p, Hue::Ember, c + vec2(-8.0, 0.0), 16.0, PI * 0.5, 1.0, 1.0);
            p.tri(
                c + vec2(6.0, -7.0),
                c + vec2(6.0, 7.0),
                c + vec2(15.0, 0.0),
                ink,
            );
        } else {
            p.rect(c - Vec2::splat(7.0), Vec2::splat(14.0), ink);
        }
    });
}

/// The menu's card: a branch, leaves letting go one after another, one of
/// them blown by a vane into a basket.
pub fn emblem(frame: &Frame, centre: Vec2, width: f32, clock: f32) {
    let pen = Pen { frame };
    let u = width / 100.0;
    let to = |x: f32, y: f32| centre + vec2(x, y) * u;
    // Cream card and a strip of grass.
    pen.rounded(to(-50.0, -40.0), vec2(100.0, 80.0) * u, 6.0 * u, SKY_TOP);
    pen.rect(to(-50.0, 26.0), vec2(100.0, 14.0) * u, GRASS);
    // The branch and its leaves.
    pen.line(to(-50.0, -30.0), to(30.0, -24.0), 4.0 * u, BARK);
    for (k, x) in [(0, -32.0), (1, -6.0), (2, 18.0)] {
        let c = CANOPY[k];
        pen.circle(to(x, -32.0), 11.0 * u, c);
    }
    // The vane, and the basket it blows into.
    let v = to(-12.0, 4.0);
    vane(
        &pen,
        Piece::Vane(Dir::Right),
        Dir::Right,
        v,
        26.0 * u,
        0.0,
        0.0,
        1.0,
        false,
    );
    basket(&pen, Hue::Ember, to(24.0, 28.0), 34.0 * u, 2, clock);
    // A leaf on its way: falls, is blown right, falls into the basket.
    let t = (clock * 0.45).fract();
    let path = |t: f32| -> Vec2 {
        if t < 0.4 {
            vec2(-12.0, -22.0 + 26.0 * smooth(t / 0.4))
        } else if t < 0.65 {
            vec2(-12.0 + 36.0 * smooth((t - 0.4) / 0.25), 4.0)
        } else {
            vec2(24.0, 4.0 + 18.0 * smooth((t - 0.65) / 0.35))
        }
    };
    let p = path(t);
    let fade = if t > 0.9 { (1.0 - t) * 10.0 } else { 1.0 };
    leaf(
        &pen,
        Hue::Ember,
        to(p.x, p.y),
        14.0 * u,
        (clock * 2.0).sin() * 0.7,
        1.0,
        fade,
    );
    // And one waiting on the branch.
    leaf(
        &pen,
        Hue::Gold,
        to(10.0, -20.0),
        11.0 * u,
        PI + (clock * 1.6).sin() * 0.2,
        1.0,
        1.0,
    );
}
