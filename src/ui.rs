//! The drawing surface the menu and every prototype share.
//!
//! One convention throughout: a [`Frame`] is a fixed 800x600 box letterboxed
//! onto whatever window the player has, and every primitive takes a *pixel*
//! position (from [`Frame::at`], or from a prototype's own camera mapping)
//! with sizes in *frame units*, so a shape keeps its proportions at any
//! window size. Nothing here knows what a prototype is; it only knows how to
//! put a circle on the screen.

use macroquad::color::Color;
use macroquad::math::Vec2;

/// macroquad's vector, under the name the prototypes use for it when they
/// have a vector type of their own.
pub use macroquad::math::Vec2 as MqVec2;
use macroquad::shapes::{
    draw_arc, draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines,
    draw_triangle,
};
use macroquad::text::{draw_text, measure_text};

/// The box everything is laid out in, in frame units.
pub const FRAME_W: f32 = 800.0;
/// See [`FRAME_W`].
pub const FRAME_H: f32 = 600.0;

/// Ordinary foreground.
pub const OVERLAY: Color = Color::new(0.85, 0.85, 0.90, 0.75);
/// Something present but not being offered.
pub const DIM: Color = Color::new(0.55, 0.55, 0.62, 0.5);
/// The one thing on screen asking for a press.
pub const HIGHLIGHT: Color = Color::new(1.0, 0.83, 0.42, 0.95);
/// Laid over the world to put a screen in front of it.
pub const SHADE: Color = Color::new(0.0, 0.0, 0.0, 0.6);

/// The same colour at a different alpha.
#[must_use]
pub const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

/// A slow pulse in `0..=1`, for things that want pressing.
#[must_use]
pub fn pulse(clock: f32) -> f32 {
    (clock * 3.0).sin().mul_add(0.5, 0.5)
}

/// The letterboxed mapping from frame units onto the window.
///
/// Small and `Copy`, so a prototype can keep one beside its camera rather
/// than borrowing the shell's.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    scale: f32,
    /// Top-left of the letterboxed box, in pixels.
    origin: Vec2,
}

impl Frame {
    /// Fit the frame into a window of this size.
    #[must_use]
    pub fn fit(screen_w: f32, screen_h: f32) -> Self {
        // A minimised window or hidden tab reports zero size; the floor keeps
        // the mapping from producing infinities.
        let scale = (screen_w / FRAME_W)
            .min(screen_h / FRAME_H)
            .max(f32::EPSILON);
        Self {
            scale,
            origin: Vec2::new(
                FRAME_W.mul_add(-scale, screen_w) * 0.5,
                FRAME_H.mul_add(-scale, screen_h) * 0.5,
            ),
        }
    }

    /// Pixels per frame unit.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// A frame position, in pixels.
    #[must_use]
    pub fn at(&self, at: Vec2) -> Vec2 {
        at * self.scale + self.origin
    }

    /// A pixel position, in frame units. The inverse of [`Frame::at`], for
    /// asking where the mouse is.
    #[must_use]
    pub fn frame_pos(&self, pixel: Vec2) -> Vec2 {
        (pixel - self.origin) / self.scale
    }

    // Primitives take pixel positions and frame-unit sizes.

    pub fn circle(&self, p: Vec2, radius: f32, color: Color) {
        draw_circle(p.x, p.y, radius * self.scale, color);
    }

    pub fn ring(&self, p: Vec2, radius: f32, thickness: f32, color: Color) {
        draw_circle_lines(p.x, p.y, radius * self.scale, thickness * self.scale, color);
    }

    pub fn arc(
        &self,
        p: Vec2,
        radius: f32,
        thickness: f32,
        from_deg: f32,
        span_deg: f32,
        color: Color,
    ) {
        draw_arc(
            p.x,
            p.y,
            48,
            radius * self.scale,
            from_deg,
            thickness * self.scale,
            span_deg,
            color,
        );
    }

    pub fn rect(&self, p: Vec2, size: Vec2, color: Color) {
        draw_rectangle(p.x, p.y, size.x * self.scale, size.y * self.scale, color);
    }

    /// A rectangle from its centre rather than its corner.
    pub fn rect_centred(&self, centre: Vec2, size: Vec2, color: Color) {
        self.rect(centre - size * (0.5 * self.scale), size, color);
    }

    pub fn rect_lines(&self, p: Vec2, size: Vec2, thickness: f32, color: Color) {
        draw_rectangle_lines(
            p.x,
            p.y,
            size.x * self.scale,
            size.y * self.scale,
            thickness * self.scale,
            color,
        );
    }

    /// An outlined rectangle from its centre rather than its corner.
    pub fn rect_lines_centred(&self, centre: Vec2, size: Vec2, thickness: f32, color: Color) {
        self.rect_lines(centre - size * (0.5 * self.scale), size, thickness, color);
    }

    pub fn line(&self, a: Vec2, b: Vec2, thickness: f32, color: Color) {
        draw_line(a.x, a.y, b.x, b.y, thickness * self.scale, color);
    }

    pub fn triangle(a: Vec2, b: Vec2, c: Vec2, color: Color) {
        draw_triangle(a, b, c, color);
    }

    /// Font size in pixels for a frame-unit text size.
    // Sizes are positive constants, so the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn px(&self, size: f32) -> u16 {
        (size * self.scale).max(1.0) as u16
    }

    /// Text centred on `p`.
    pub fn text_centred(&self, text: &str, p: Vec2, size: f32, color: Color) {
        let px = self.px(size);
        let dims = measure_text(text, None, px, 1.0);
        draw_text(
            text,
            dims.width.mul_add(-0.5, p.x),
            dims.offset_y.mul_add(0.5, p.y),
            f32::from(px),
            color,
        );
    }

    /// One character, centred on `p`.
    pub fn glyph(&self, ch: char, p: Vec2, size: f32, color: Color) {
        self.text_centred(&ch.to_string(), p, size, color);
    }

    /// A keyboard key of this size, centred on `p`, with a character on it
    /// or not.
    pub fn key_cap(&self, ch: Option<char>, p: Vec2, size: Vec2, color: Color) {
        self.rect_lines_centred(p, size, 2.0, color);
        if let Some(ch) = ch {
            self.glyph(ch, p, size.y * 0.8, color);
        }
    }

    /// A key cap with a solid triangle on it, pointing along `dir`.
    pub fn arrow_cap(&self, dir: Vec2, p: Vec2, size: f32, color: Color) {
        self.key_cap(None, p, Vec2::new(size, size), color);
        let reach = size * 0.22 * self.scale;
        let across = Vec2::new(-dir.y, dir.x);
        let tip = p + dir * reach;
        let base = p - dir * (reach * 0.6);
        Self::triangle(tip, base + across * reach, base - across * reach, color);
    }

    /// A wide key cap with a return arrow on it: a hooked line pointing
    /// left, drawn rather than typed so no font has to have the character.
    pub fn enter_cap(&self, p: Vec2, size: Vec2, color: Color) {
        self.key_cap(None, p, size, color);
        let s = self.scale;
        let reach = size.y * 0.26 * s;
        let tip = p + Vec2::new(-reach, reach * 0.4);
        let elbow = Vec2::new(p.x + reach, tip.y);
        self.line(tip, elbow, 2.0, color);
        self.line(elbow, elbow - Vec2::new(0.0, reach), 2.0, color);
        Self::triangle(
            tip,
            tip + Vec2::new(reach * 0.6, -reach * 0.45),
            tip + Vec2::new(reach * 0.6, reach * 0.45),
            color,
        );
    }
}
