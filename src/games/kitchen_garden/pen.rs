//! Soft shapes in frame units, and the palette they are painted in.
//!
//! Kitchen Garden is drawn rather than pixelled: rounded boxes, ellipses
//! and capsules over the shell's [`Frame`], in a warm palette. Everything
//! here takes *frame* positions and sizes (the 800x600 box) and converts
//! once, so the scene reads like a layout.

// Segment counts are small positive numbers made from sizes.
#![allow(clippy::cast_sign_loss)]
// Drawing is points, radii and colours, over and over.
#![allow(clippy::many_single_char_names)]

use std::f32::consts::TAU;

use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};

use crate::ui::Frame;

/// A colour from `0xRRGGBB`.
#[must_use]
pub const fn hex(rgb: u32) -> Color {
    Color::new(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        1.0,
    )
}

/// The same colour at another alpha.
#[must_use]
pub const fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// Mix two colours, `t` of the way from `a` to `b`.
#[must_use]
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (b.r - a.r).mul_add(t, a.r),
        (b.g - a.g).mul_add(t, a.g),
        (b.b - a.b).mul_add(t, a.b),
        (b.a - a.a).mul_add(t, a.a),
    )
}

/// Lighter by `t` (toward white).
#[must_use]
pub fn light(c: Color, t: f32) -> Color {
    mix(c, Color::new(1.0, 1.0, 1.0, c.a), t)
}

/// Darker by `t` (toward a warm near-black, so shade never goes grey).
#[must_use]
pub fn dark(c: Color, t: f32) -> Color {
    mix(c, Color::new(0.16, 0.09, 0.07, c.a), t)
}

/// The palette, by role. Warm wood and clay indoors, greens and earth
/// outside, and a few clear food colours that read against both.
pub mod ink {
    use super::hex;
    use macroquad::color::Color;

    // Outline and shadow.
    pub const LINE: Color = hex(0x3b_2a22);
    pub const SHADOW: Color = Color::new(0.16, 0.09, 0.07, 0.28);
    pub const WHITE: Color = hex(0xff_f8ec);
    pub const CREAM: Color = hex(0xf6_e7c8);

    // Kitchen.
    pub const WALL: Color = hex(0xe9_c99a);
    pub const WALL_DARK: Color = hex(0xd4_ad7c);
    pub const WOOD: Color = hex(0xa8_6a3f);
    pub const WOOD_LIGHT: Color = hex(0xd0_9460);
    pub const WOOD_DARK: Color = hex(0x6e_4026);
    pub const BRICK: Color = hex(0xb5_5a3c);
    pub const BRICK_DARK: Color = hex(0x7e_3a2a);
    pub const STONE: Color = hex(0x9c_948a);
    pub const STONE_LIGHT: Color = hex(0xc7_bfb2);
    pub const STONE_DARK: Color = hex(0x5f_5852);
    pub const IRON: Color = hex(0x3d_3a3c);
    pub const IRON_LIGHT: Color = hex(0x6c_6669);
    pub const CLAY: Color = hex(0xc9_7b4f);
    pub const CERAMIC: Color = hex(0xf2_efe6);
    pub const GLAZE: Color = hex(0x5f_8fb0);
    pub const BRASS: Color = hex(0xd9_a441);
    pub const COPPER: Color = hex(0xc8_6b3a);

    // Fire.
    pub const FLAME: Color = hex(0xff_9b30);
    pub const FLAME_HOT: Color = hex(0xff_e27a);
    pub const EMBER: Color = hex(0xd8_3c1e);
    pub const SMOKE: Color = hex(0x8a_8580);

    // Garden.
    pub const GRASS: Color = hex(0x8f_bf5a);
    pub const GRASS_DARK: Color = hex(0x6a_9a43);
    pub const GRASS_LIGHT: Color = hex(0xb5_d97a);
    pub const SOIL: Color = hex(0x7a_4e30);
    pub const SOIL_DARK: Color = hex(0x58_3520);
    pub const SOIL_RICH: Color = hex(0x4a_2c1c);
    pub const LEAF: Color = hex(0x4f_9a3c);
    pub const LEAF_DARK: Color = hex(0x2f_6b2a);
    pub const LEAF_LIGHT: Color = hex(0x8b_cf5c);
    pub const WATER: Color = hex(0x5d_b2e0);
    pub const WATER_DARK: Color = hex(0x2f_7fb5);
    pub const SKY: Color = hex(0x9f_d8f2);
    pub const SKY_HIGH: Color = hex(0x6c_b8e8);

    // Food.
    pub const MILK: Color = hex(0xfb_fbf5);
    pub const EGG: Color = hex(0xf3_e2c4);
    pub const YOLK: Color = hex(0xf6_b632);
    pub const CARROT: Color = hex(0xf0_8a24);
    pub const ONION: Color = hex(0xb6_6aa8);
    pub const ONION_SKIN: Color = hex(0xd8_a86a);
    pub const TOMATO: Color = hex(0xe2_3d32);
    pub const WHEAT: Color = hex(0xe8_be5a);
    pub const RICE: Color = hex(0xfa_f6ea);
    pub const BEAN: Color = hex(0x8b_3f2a);
    pub const SUGAR: Color = hex(0xff_ffff);
    pub const TEA: Color = hex(0x9a_5a2a);
    pub const TEA_LEAF: Color = hex(0x5a_a04a);
    pub const CHILI: Color = hex(0xd1_2a1f);
    pub const SALT: Color = hex(0xe8_eef2);
    pub const FLOUR: Color = hex(0xfa_f3e3);
    pub const DOUGH: Color = hex(0xf0_d7a8);
    pub const CRUST: Color = hex(0xc4_7a34);
    pub const CRUST_DARK: Color = hex(0x8e_4f1f);
    pub const BUTTER: Color = hex(0xf9_dc5c);
    pub const BROTH: Color = hex(0xe8_a64a);
    pub const STEW: Color = hex(0xa3_3a22);
    pub const CUSTARD: Color = hex(0xf6_d36a);
    pub const ICING: Color = hex(0xff_f0f4);
    pub const BERRY: Color = hex(0xd8_2b52);
    pub const MUSH: Color = hex(0x8f_9a5a);
    pub const CHAR: Color = hex(0x2a_2422);

    // Coins and marks.
    pub const GOLD: Color = hex(0xf5_c542);
    pub const GOLD_DARK: Color = hex(0xb8_8418);
    pub const HEART: Color = hex(0xec_5a72);
    pub const GOOD: Color = hex(0x6c_c26a);
    pub const BAD: Color = hex(0xd9_4a3a);
    pub const GLOW: Color = hex(0xff_e9a8);
}

/// Draws in frame units onto a [`Frame`].
#[derive(Clone, Copy)]
pub struct Pen<'a> {
    pub frame: &'a Frame,
    /// Added to every position, for shaking the whole scene.
    shift: Vec2,
}

impl<'a> Pen<'a> {
    #[must_use]
    pub const fn new(frame: &'a Frame) -> Self {
        Self {
            frame,
            shift: Vec2::ZERO,
        }
    }

    /// The same pen, drawing everything `by` frame units over.
    #[must_use]
    pub const fn shifted(self, by: Vec2) -> Self {
        Self { shift: by, ..self }
    }

    /// A frame position in pixels.
    #[must_use]
    pub fn px(&self, at: Vec2) -> Vec2 {
        self.frame.at(at + self.shift)
    }

    pub fn circle(&self, c: Vec2, r: f32, color: Color) {
        // Polygons rather than macroquad's circle so big ones stay round.
        let sides = (r * self.frame.scale()).clamp(12.0, 64.0) as u8;
        self.frame.poly(self.px(c), sides, r, 0.0, color);
    }

    pub fn ring(&self, c: Vec2, r: f32, thick: f32, color: Color) {
        self.arc(c, thick.mul_add(-0.5, r), thick, 0.0, 360.0, color);
    }

    /// A thick arc, `from_deg` clockwise for `span_deg`, centred on radius `r`.
    pub fn arc(&self, c: Vec2, r: f32, thick: f32, from_deg: f32, span_deg: f32, color: Color) {
        if span_deg <= 0.0 {
            return;
        }
        let steps = (span_deg / 8.0).ceil().max(2.0) as usize;
        let (inner, outer) = (thick.mul_add(-0.5, r), thick.mul_add(0.5, r));
        for i in 0..steps {
            let a0 = (from_deg + span_deg * i as f32 / steps as f32).to_radians();
            let a1 = (from_deg + span_deg * (i + 1) as f32 / steps as f32).to_radians();
            let (d0, d1) = (Vec2::from_angle(a0), Vec2::from_angle(a1));
            let (p0, p1) = (c + d0 * inner, c + d1 * inner);
            let (q0, q1) = (c + d0 * outer, c + d1 * outer);
            self.tri(p0, q0, q1, color);
            self.tri(p0, q1, p1, color);
        }
    }

    /// A filled pie slice.
    pub fn pie(&self, c: Vec2, r: f32, from_deg: f32, span_deg: f32, color: Color) {
        let steps = (span_deg / 8.0).ceil().max(2.0) as usize;
        for i in 0..steps {
            let a0 = (from_deg + span_deg * i as f32 / steps as f32).to_radians();
            let a1 = (from_deg + span_deg * (i + 1) as f32 / steps as f32).to_radians();
            self.tri(
                c,
                c + Vec2::from_angle(a0) * r,
                c + Vec2::from_angle(a1) * r,
                color,
            );
        }
    }

    /// An ellipse with radii `r`, turned by `turn` radians.
    pub fn ellipse(&self, c: Vec2, r: Vec2, turn: f32, color: Color) {
        let n = ((r.x.max(r.y)) * self.frame.scale() * 0.8).clamp(14.0, 56.0) as usize;
        let (s, k) = turn.sin_cos();
        let point = |i: usize| {
            let a = TAU * i as f32 / n as f32;
            let local = vec2(a.cos() * r.x, a.sin() * r.y);
            c + vec2(
                local.y.mul_add(-s, local.x * k),
                local.y.mul_add(k, local.x * s),
            )
        };
        for i in 0..n {
            self.tri(c, point(i), point(i + 1), color);
        }
    }

    /// A convex polygon through frame positions.
    pub fn poly(&self, points: &[Vec2], color: Color) {
        for i in 1..points.len().saturating_sub(1) {
            self.tri(points[0], points[i], points[i + 1], color);
        }
    }

    pub fn tri(&self, a: Vec2, b: Vec2, c: Vec2, color: Color) {
        Frame::triangle(self.px(a), self.px(b), self.px(c), color);
    }

    pub fn rect(&self, min: Vec2, size: Vec2, color: Color) {
        self.frame.rect(self.px(min), size, color);
    }

    pub fn rect_centred(&self, c: Vec2, size: Vec2, color: Color) {
        self.rect(c - size * 0.5, size, color);
    }

    /// A box with rounded corners of radius `r`.
    pub fn rrect(&self, min: Vec2, size: Vec2, r: f32, color: Color) {
        let r = r.min(size.x * 0.5).min(size.y * 0.5).max(0.0);
        if r <= 0.01 {
            self.rect(min, size, color);
            return;
        }
        self.rect(
            min + vec2(r, 0.0),
            vec2(2.0f32.mul_add(-r, size.x), size.y),
            color,
        );
        self.rect(
            min + vec2(0.0, r),
            vec2(r, 2.0f32.mul_add(-r, size.y)),
            color,
        );
        self.rect(
            min + vec2(size.x - r, r),
            vec2(r, 2.0f32.mul_add(-r, size.y)),
            color,
        );
        for (corner, from) in [
            (min + vec2(r, r), 180.0),
            (min + vec2(size.x - r, r), 270.0),
            (min + vec2(size.x - r, size.y - r), 0.0),
            (min + vec2(r, size.y - r), 90.0),
        ] {
            self.pie(corner, r, from, 90.0, color);
        }
    }

    pub fn rrect_centred(&self, c: Vec2, size: Vec2, r: f32, color: Color) {
        self.rrect(c - size * 0.5, size, r, color);
    }

    /// A rounded box with a darker rim `rim` wide around it: the house
    /// style for anything solid.
    pub fn panel(&self, min: Vec2, size: Vec2, r: f32, rim: f32, fill: Color, edge: Color) {
        self.rrect(min, size, r, edge);
        self.rrect(
            min + Vec2::splat(rim),
            size - Vec2::splat(rim * 2.0),
            (r - rim).max(0.0),
            fill,
        );
    }

    /// A line with round ends.
    pub fn capsule(&self, a: Vec2, b: Vec2, r: f32, color: Color) {
        self.line(a, b, r * 2.0, color);
        self.circle(a, r, color);
        self.circle(b, r, color);
    }

    pub fn line(&self, a: Vec2, b: Vec2, thick: f32, color: Color) {
        let d = b - a;
        if d.length_squared() < 1e-6 {
            return;
        }
        let n = vec2(-d.y, d.x).normalize() * (thick * 0.5);
        self.tri(a + n, b + n, b - n, color);
        self.tri(a + n, b - n, a - n, color);
    }

    /// A soft shadow on the ground under something.
    pub fn shadow(&self, c: Vec2, w: f32) {
        self.ellipse(c, vec2(w * 0.5, w * 0.16), 0.0, ink::SHADOW);
    }

    /// A vertical gradient over a box, `top` to `bottom`.
    pub fn gradient(&self, min: Vec2, size: Vec2, top: Color, bottom: Color) {
        self.frame
            .gradient(self.px(min), size, [top, top, bottom, bottom]);
    }

    /// A four-pointed sparkle.
    pub fn sparkle(&self, c: Vec2, r: f32, color: Color) {
        self.frame.sparkle(self.px(c), r, color);
    }

    /// A heart, `r` across the lobes.
    pub fn heart(&self, c: Vec2, r: f32, color: Color) {
        let lobe = r * 0.52;
        self.circle(c + vec2(-r * 0.48, -r * 0.2), lobe, color);
        self.circle(c + vec2(r * 0.48, -r * 0.2), lobe, color);
        self.tri(
            c + vec2(-r * 0.98, -r * 0.05),
            c + vec2(r * 0.98, -r * 0.05),
            c + vec2(0.0, r * 0.95),
            color,
        );
    }

    /// A coin, `r` in radius.
    pub fn coin(&self, c: Vec2, r: f32) {
        self.circle(c + vec2(0.0, r * 0.18), r, ink::GOLD_DARK);
        self.circle(c, r, ink::GOLD);
        self.ring(c, r * 0.68, r * 0.14, ink::GOLD_DARK);
        self.circle(
            c + vec2(-r * 0.32, -r * 0.34),
            r * 0.2,
            light(ink::GOLD, 0.6),
        );
    }
}
