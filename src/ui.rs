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
use macroquad::models::{Mesh, Vertex, draw_mesh};

/// macroquad's vector, under the name the prototypes use for it when they
/// have a vector type of their own.
pub use macroquad::math::Vec2 as MqVec2;
use macroquad::shapes::{
    draw_arc, draw_circle, draw_circle_lines, draw_line, draw_poly, draw_rectangle,
    draw_rectangle_lines, draw_triangle,
};
use macroquad::text::{Font, TextParams, draw_text_ex, measure_text};
use macroquad::texture::{DrawTextureParams, Texture2D, draw_texture_ex};

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
    /// The whole window, in pixels.
    window: Vec2,
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
            window: Vec2::new(screen_w, screen_h),
        }
    }

    /// The whole window, in frame units: the frame plus whatever letterbox
    /// bars surround it. Backdrops paint across this so they fill the window;
    /// anything that has to be readable stays inside `0..FRAME_W` by
    /// `0..FRAME_H`. Returns the top-left and bottom-right corners.
    #[must_use]
    pub fn window_bounds(&self) -> (Vec2, Vec2) {
        (self.frame_pos(Vec2::ZERO), self.frame_pos(self.window))
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

    // The segment count is clamped positive before the cast.
    #[allow(clippy::cast_sign_loss)]
    pub fn arc(
        &self,
        p: Vec2,
        radius: f32,
        thickness: f32,
        from_deg: f32,
        span_deg: f32,
        color: Color,
    ) {
        // Enough segments that a big arc's edge stays round; never fewer
        // than a small one has always had.
        let sides = ((radius + thickness) * self.scale * 0.4).clamp(48.0, 240.0) as u8;
        draw_arc(
            p.x,
            p.y,
            sides,
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

    /// A regular polygon of `sides` around `p`, turned by `rotation_deg`.
    /// Many sides make a smoother circle than [`Frame::circle`] does, which
    /// matters once one fills the screen.
    pub fn poly(&self, p: Vec2, sides: u8, radius: f32, rotation_deg: f32, color: Color) {
        draw_poly(p.x, p.y, sides, radius * self.scale, rotation_deg, color);
    }

    /// The outline of a regular polygon, drawn outward from `radius`.
    pub fn poly_lines(
        &self,
        p: Vec2,
        sides: u8,
        radius: f32,
        rotation_deg: f32,
        thickness: f32,
        color: Color,
    ) {
        let rot = rotation_deg.to_radians();
        let corners: Vec<Vec2> = (0..sides)
            .map(|i| {
                let angle = f32::from(i).mul_add(std::f32::consts::TAU / f32::from(sides), rot);
                p + Vec2::from_angle(angle) * (radius * self.scale)
            })
            .collect();
        self.outline(&corners, thickness, color);
    }

    /// A closed outline through pixel positions.
    pub fn outline(&self, corners: &[Vec2], thickness: f32, color: Color) {
        for (i, &a) in corners.iter().enumerate() {
            let b = corners[(i + 1) % corners.len()];
            self.line(a, b, thickness, color);
            // A dot on every corner, so thick outlines meet without a notch.
            self.circle(a, thickness * 0.5, color);
        }
    }

    /// A rectangle shaded between four corner colours, clockwise from the
    /// top left. Stands in for a CSS gradient.
    pub fn gradient(&self, p: Vec2, size: Vec2, corners: [Color; 4]) {
        let size = size * self.scale;
        let points = [
            p,
            p + Vec2::new(size.x, 0.0),
            p + size,
            p + Vec2::new(0.0, size.y),
        ];
        draw_mesh(&Mesh {
            vertices: points
                .iter()
                .zip(corners)
                .map(|(at, color)| Vertex::new(at.x, at.y, 0.0, 0.0, 0.0, color))
                .collect(),
            indices: vec![0, 1, 2, 0, 2, 3],
            texture: None,
        });
    }

    /// A four-pointed sparkle, the star the wirenook wave field is made of.
    pub fn sparkle(&self, p: Vec2, radius: f32, color: Color) {
        let reach = radius * self.scale;
        let waist = reach * 0.39;
        for (along, across) in [(Vec2::Y, Vec2::X), (Vec2::X, Vec2::Y)] {
            for tip in [p + along * reach, p - along * reach] {
                Self::triangle(tip, p + across * waist, p - across * waist, color);
            }
        }
    }

    /// A texture stretched over a box of the frame, `size` frame units from
    /// `p` (pixels). A prototype that paints at low resolution — pixel art
    /// into a render target — hands its canvas over through this, so it
    /// letterboxes like everything else.
    pub fn image(&self, texture: &Texture2D, p: Vec2, size: Vec2, flip_y: bool) {
        draw_texture_ex(
            texture,
            p.x,
            p.y,
            Color::new(1.0, 1.0, 1.0, 1.0),
            DrawTextureParams {
                dest_size: Some(size * self.scale),
                flip_y,
                ..DrawTextureParams::default()
            },
        );
    }

    /// Font size in pixels for a frame-unit text size.
    // Sizes are positive constants, so the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn px(&self, size: f32) -> u16 {
        (size * self.scale).max(1.0) as u16
    }

    /// Text centred on `p`.
    pub fn text_centred(&self, text: &str, p: Vec2, size: f32, color: Color) {
        self.label(None, text, p, size, color);
    }

    /// Text centred on `p` in a font of the caller's, or the built-in one.
    pub fn label(&self, font: Option<&Font>, text: &str, p: Vec2, size: f32, color: Color) {
        let px = self.px(size);
        let dims = measure_text(text, font, px, 1.0);
        draw_text_ex(
            text,
            dims.width.mul_add(-0.5, p.x),
            dims.offset_y.mul_add(0.5, p.y),
            TextParams {
                font,
                font_size: px,
                color,
                ..TextParams::default()
            },
        );
    }

    /// How wide [`Frame::label`] would draw `text`, in frame units.
    #[must_use]
    pub fn label_width(&self, font: Option<&Font>, text: &str, size: f32) -> f32 {
        measure_text(text, font, self.px(size), 1.0).width / self.scale
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
