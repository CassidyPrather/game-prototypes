//! The pixel canvas: a 400x300 render target every piece of Sand Nomad is
//! drawn into at one canvas pixel per art pixel, then stretched into the
//! shell's frame in one go. That keeps every pixel the same size, which
//! is most of what makes pixel art look like pixel art.
//!
//! Everything here takes canvas pixels. The baked sprites live in [`Art`].

// Pixel drawing is x, y, w, h and a colour, over and over.
#![allow(clippy::many_single_char_names)]

use std::collections::HashMap;

use game_prototypes::sand_nomad::art::{self, Sprite};
use game_prototypes::sand_nomad::item::{ANIMA_WEIGHT, FONDNESS_PER_WEIGHT, Item, JAR_SIPS, Kind};
use game_prototypes::sand_nomad::motive::Motive;
use game_prototypes::sand_nomad::world;
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::shapes::{draw_circle, draw_rectangle};
use macroquad::texture::{
    DrawTextureParams, FilterMode, RenderTarget, RenderTargetParams, Texture2D, draw_texture_ex,
    render_target_ex,
};
use macroquad::window::clear_background;

use crate::ui::{FRAME_H, FRAME_W, Frame};

/// Canvas size in pixels.
pub const W: f32 = 400.0;
/// See [`W`].
pub const H: f32 = 300.0;

/// A palette colour.
pub fn ink(ch: char) -> Color {
    let rgb = art::rgb(ch);
    Color::new(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        1.0,
    )
}

/// The same colour at another alpha.
pub const fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// Every sprite, baked into textures once.
pub struct Art {
    textures: HashMap<u64, Texture2D>,
    /// The whole map's ground, baked from the terrain rows.
    pub ground: Texture2D,
    /// The sand underfoot in camp.
    pub camp: Texture2D,
    /// The deck boards along the bottom.
    pub deck: Texture2D,
}

/// A sprite's identity: a hash of its pixels. (Not its address: a `const`
/// may be copied wherever it is used, so the same sprite can live in
/// several places.)
fn key(sprite: Sprite) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for row in sprite.0 {
        for b in row.bytes().chain(std::iter::once(b'|')) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

fn bake(sprite: Sprite) -> Texture2D {
    // Sprites are a few dozen pixels a side.
    #[allow(clippy::cast_possible_truncation)]
    let texture = Texture2D::from_rgba8(
        sprite.width() as u16,
        sprite.height() as u16,
        &sprite.rgba(),
    );
    texture.set_filter(FilterMode::Nearest);
    texture
}

impl Art {
    pub fn bake() -> Self {
        let mut textures = HashMap::new();
        let mut add = |s: Sprite| {
            textures.entry(key(s)).or_insert_with(|| bake(s));
        };
        for kind in Kind::ALL {
            add(art::item(kind));
        }
        for s in art::OTHERS {
            add(s);
        }
        for site in world::Site::ALL {
            add(art::place(site));
        }
        Self {
            textures,
            ground: bake_ground(),
            camp: bake_camp(),
            deck: bake_deck(),
        }
    }

    pub fn tex(&self, sprite: Sprite) -> &Texture2D {
        self.textures
            .get(&key(sprite))
            .expect("every sprite is baked at load")
    }
}

/// A cheap, fixed hash of a pixel, `0..1`: the grain of the sand. The
/// basin is hand-made; this only decides which grains are darker.
pub fn grain(x: i32, y: i32) -> f32 {
    // Bit mixing on purpose.
    #[allow(clippy::cast_sign_loss)]
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0
}

/// Paint the map's ground pixel by pixel from the hand-drawn terrain rows,
/// with the tile edges roughened so the land does not look gridded.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
fn bake_ground() -> Texture2D {
    let (w, h) = (world::MAP_W as usize, world::MAP_H as usize);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let (xi, yi) = (x as i32, y as i32);
            // Look the tile up from a nudged position, so edges wander.
            let jx = (grain(xi / 3, yi / 5) - 0.5) * 9.0;
            let jy = (grain(xi / 5 + 91, yi / 3) - 0.5) * 9.0;
            let tile = world::terrain_at(x as f32 + jx, y as f32 + jy);
            let g = grain(xi, yi);
            let ch = match tile {
                'g' => {
                    if g < 0.08 {
                        'o'
                    } else if g > 0.9 || (x + y * 3) % 23 == 0 {
                        'q'
                    } else {
                        'p'
                    }
                }
                'c' => {
                    // Banded rock, darker lower down each tile.
                    let band = (yi + (grain(xi / 4, 0) * 4.0) as i32) % 16;
                    match band {
                        0..=3 => '9',
                        4..=8 => 'a',
                        9..=12 => '3',
                        _ => '2',
                    }
                }
                'd' => {
                    let ripple = (x as f32 * 0.21)
                        .sin()
                        .mul_add(2.5, (x as f32).mul_add(0.35, y as f32))
                        as i32;
                    match ripple.rem_euclid(7) {
                        0 => '9',
                        1 => '6',
                        _ if g < 0.05 => '9',
                        _ => '8',
                    }
                }
                'w' => {
                    let crack = (x + 2 * y) % 13 == 0 || (3 * x + y) % 17 == 0;
                    if crack && g < 0.7 {
                        'n'
                    } else if g < 0.1 {
                        'k'
                    } else {
                        'S'
                    }
                }
                'k' => {
                    let crack = (x * 3 + y * 5) % 19 == 0 || (x * 7 + y) % 23 == 0;
                    if crack {
                        '3'
                    } else if g < 0.3 {
                        'a'
                    } else {
                        '9'
                    }
                }
                'r' => {
                    if g < 0.15 {
                        'i'
                    } else if g < 0.5 {
                        'a'
                    } else if g > 0.93 {
                        'j'
                    } else {
                        '3'
                    }
                }
                'o' => {
                    let rut = (x + y / 2) % 8;
                    if rut == 1 || rut == 5 { 'a' } else { '9' }
                }
                _ => {
                    if g < 0.06 {
                        '9'
                    } else if g > 0.95 {
                        '6'
                    } else {
                        '8'
                    }
                }
            };
            rgba.extend_from_slice(&art::colour(ch).unwrap_or([255, 0, 255, 255]));
        }
    }
    let texture = Texture2D::from_rgba8(w as u16, h as u16, &rgba);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn texture(w: usize, h: usize, mut pixel: impl FnMut(usize, usize) -> char) -> Texture2D {
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            rgba.extend_from_slice(&art::colour(pixel(x, y)).unwrap_or([0, 0, 0, 0]));
        }
    }
    // Small, fixed sizes.
    #[allow(clippy::cast_possible_truncation)]
    let texture = Texture2D::from_rgba8(w as u16, h as u16, &rgba);
    texture.set_filter(FilterMode::Nearest);
    texture
}

/// Sand underfoot in camp, 400x128, lighter toward the horizon, with
/// ripples and footprints of grain.
#[allow(clippy::cast_possible_wrap)]
fn bake_camp() -> Texture2D {
    texture(400, 128, |x, y| {
        let g = grain(x as i32 + 1000, y as i32);
        let ripple =
            (y as f32).mul_add(0.5, (y as f32).mul_add(0.7, x as f32 * 0.09).sin() * 3.0) as i32;
        if y < 3 || g < 0.04 || (ripple % 9 == 0 && g < 0.5 && y > 20) {
            '9'
        } else if g > 0.97 || (y < 30 && g > 0.8) {
            '6'
        } else {
            '8'
        }
    })
}

/// The deck: boards running across, with seams, nails and grain.
#[allow(clippy::cast_possible_wrap)]
fn bake_deck() -> Texture2D {
    texture(400, 72, |x, y| {
        let board = y / 9;
        let seam = y % 9 == 0;
        let joint = (x + board * 53) % 97 == 0;
        let nail =
            !seam && y % 9 == 4 && ((x + board * 53) % 97 == 3 || (x + board * 53) % 97 == 94);
        let g = grain(x as i32 / 3, y as i32 + 500);
        if seam || joint {
            '1'
        } else if nail {
            'm'
        } else if g < 0.12 {
            '2'
        } else if board % 2 == 0 {
            '3'
        } else if g > 0.9 {
            '4'
        } else {
            '3'
        }
    })
}

/// The render target and the camera that draws into it.
pub struct Canvas {
    target: RenderTarget,
}

impl Canvas {
    pub fn new() -> Self {
        // The canvas is a fixed, small size.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target = render_target_ex(
            W as u32,
            H as u32,
            // Sample count zero: no multisampling, so no resolve pass. The
            // resolve needs WebGL 2's blit, and pixel art wants no blending
            // anyway.
            RenderTargetParams {
                sample_count: 0,
                depth: false,
            },
        );
        target.texture.set_filter(FilterMode::Nearest);
        Self { target }
    }

    /// Start drawing into the canvas, in canvas pixels, y down.
    pub fn begin(&self) {
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, W, H));
        camera.render_target = Some(self.target.clone());
        // A render target's rows run the other way from the screen's, so
        // this camera looks at the canvas upside down, and the canvas is
        // right way up once `end` lays it out.
        camera.zoom.y = -camera.zoom.y;
        set_camera(&camera);
        clear_background(ink('0'));
    }

    /// Stop, and lay the canvas over the whole frame.
    pub fn end(&self, frame: &Frame) {
        set_default_camera();
        frame.image(
            &self.target.texture,
            frame.at(vec2(0.0, 0.0)),
            vec2(FRAME_W, FRAME_H),
            false,
        );
    }
}

/// Canvas pixels per frame unit, for turning the mouse into the canvas.
pub fn to_canvas(frame: &Frame, pixel: Vec2) -> Vec2 {
    frame.frame_pos(pixel) * (W / FRAME_W)
}

// Primitives, all snapped to whole pixels.

pub fn rect(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(x.floor(), y.floor(), w.floor(), h.floor(), c);
}

pub fn outline(x: f32, y: f32, w: f32, h: f32, c: Color) {
    let (x, y, w, h) = (x.floor(), y.floor(), w.floor(), h.floor());
    draw_rectangle(x, y, w, 1.0, c);
    draw_rectangle(x, y + h - 1.0, w, 1.0, c);
    draw_rectangle(x, y, 1.0, h, c);
    draw_rectangle(x + w - 1.0, y, 1.0, h, c);
}

pub fn px(x: f32, y: f32, c: Color) {
    draw_rectangle(x.floor(), y.floor(), 1.0, 1.0, c);
}

/// A one-pixel line, stepped the way a pixel artist would.
// Canvas coordinates are small.
#[allow(clippy::cast_possible_truncation)]
pub fn line(x0: f32, y0: f32, x1: f32, y1: f32, c: Color) {
    let (mut x, mut y) = (x0.round() as i32, y0.round() as i32);
    let (x1, y1) = (x1.round() as i32, y1.round() as i32);
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    for _ in 0..2000 {
        draw_rectangle(x as f32, y as f32, 1.0, 1.0, c);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// A dotted line: `on` pixels drawn, `off` skipped, starting `phase` in.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn dotted(a: Vec2, b: Vec2, on: u32, off: u32, phase: f32, c: Color) {
    let length = a.distance(b);
    let steps = length.round().max(1.0) as u32;
    let period = (on + off).max(1);
    let shift = phase.rem_euclid(period as f32) as u32;
    for i in 0..=steps {
        if (i + period - shift) % period < on {
            let p = a.lerp(b, i as f32 / steps as f32);
            px(p.x.round(), p.y.round(), c);
        }
    }
}

pub fn disc(x: f32, y: f32, r: f32, c: Color) {
    draw_circle(x.floor() + 0.5, y.floor() + 0.5, r, c);
}

/// A sprite with its top left at `(x, y)`.
pub fn sprite(art: &Art, s: Sprite, x: f32, y: f32) {
    sprite_ex(art, s, x, y, Color::new(1.0, 1.0, 1.0, 1.0), false);
}

/// A sprite, tinted, maybe mirrored.
pub fn sprite_ex(art: &Art, s: Sprite, x: f32, y: f32, tint: Color, flip_x: bool) {
    draw_texture_ex(
        art.tex(s),
        x.floor(),
        y.floor(),
        tint,
        DrawTextureParams {
            flip_x,
            ..DrawTextureParams::default()
        },
    );
}

/// A sprite turned a quarter clockwise, its top left at `(x, y)` once
/// turned.
pub fn sprite_turned(art: &Art, s: Sprite, x: f32, y: f32, tint: Color) {
    let (w, h) = (s.width() as f32, s.height() as f32);
    // Rotation spins about the centre of the unturned box; shift so the
    // turned box lands with its corner at (x, y).
    let (cx, cy) = (x.floor() + h / 2.0, y.floor() + w / 2.0);
    draw_texture_ex(
        art.tex(s),
        cx - w / 2.0,
        cy - h / 2.0,
        tint,
        DrawTextureParams {
            rotation: std::f32::consts::FRAC_PI_2,
            ..DrawTextureParams::default()
        },
    );
}

/// Where the six nodes of a rune sit in its 5x5 box, clockwise from the
/// top left in motive order: the motive hexagon, tiny.
pub const RUNE_NODES: [(f32, f32); 6] = [
    (1.0, 0.0),
    (3.0, 0.0),
    (4.0, 2.0),
    (3.0, 4.0),
    (1.0, 4.0),
    (0.0, 2.0),
];

/// The motive a rune node stands for, and whether a thing's weight lights
/// it: weight fills the hexagon from the thing's own motive, clockwise,
/// one motive a point, until at six every feeling is in it and it wakes.
pub fn lit_node(item: Item, node: usize) -> Option<Motive> {
    let motive = item.motive?;
    let from = motive.index();
    let steps = (node + 6 - from) % 6;
    (item.known && steps < usize::from(item.weight)).then_some(Motive::ALL[node])
}

/// A motive's ink, lightened for Hate so it shows on dark wood.
pub fn node_ink(m: Motive) -> Color {
    if m == Motive::Hate {
        ink('n')
    } else {
        ink(art::motive_ink(m))
    }
}

/// A thing's weight as a rune: its six nodes lit from its own motive
/// round, the eye in the middle once it is awake, grey if unknown. Drawn
/// in a 7x7 box with its top left at `(x, y)`. `flash` whitens the newest
/// node.
pub fn rune(x: f32, y: f32, item: Item, flash: f32) {
    let Some(motive) = item.motive else {
        return;
    };
    let (x, y) = (x.floor(), y.floor());
    rect(x + 1.0, y, 5.0, 7.0, fade(ink('0'), 0.85));
    rect(x, y + 1.0, 7.0, 5.0, fade(ink('0'), 0.85));
    let newest = (motive.index() + usize::from(item.weight) + 5) % 6;
    let next = (motive.index() + usize::from(item.weight)) % 6;
    for (i, (nx, ny)) in RUNE_NODES.iter().enumerate() {
        let c = if !item.known {
            ink('m')
        } else if let Some(m) = lit_node(item, i) {
            if i == newest && flash > 0.0 {
                ink('w')
            } else {
                node_ink(m)
            }
        } else if i == next && item.fondness > 0 && !item.is_anima() {
            // The next node, filling.
            fade(
                node_ink(Motive::ALL[i]),
                0.25 + 0.5 * f32::from(item.fondness) / f32::from(FONDNESS_PER_WEIGHT),
            )
        } else {
            ink('2')
        };
        px(x + 1.0 + nx, y + 1.0 + ny, c);
    }
    if !item.known {
        px(x + 3.0, y + 3.0, ink('n'));
    } else if item.weight >= ANIMA_WEIGHT {
        px(x + 3.0, y + 3.0, ink('j'));
    }
}

/// The rune writ large, as the motif hexagon is drawn: six rings joined
/// round the edge and across in two triangles, each ring filled once the
/// thing's weight reaches it. Centred on `c`.
pub fn big_rune(c: Vec2, item: Item, clock: f32) {
    let radius = 11.0;
    let node = |i: usize| {
        let a = (i as f32 / 6.0).mul_add(std::f32::consts::TAU, -std::f32::consts::PI * 2.0 / 3.0);
        c + vec2(a.cos(), a.sin()) * radius
    };
    let edge = ink('l');
    for i in 0..6 {
        let (a, b) = (node(i), node((i + 1) % 6));
        line(a.x, a.y, b.x, b.y, edge);
        let d = node((i + 2) % 6);
        dotted(a, d, 1, 1, 0.0, fade(edge, 0.6));
    }
    for i in 0..6 {
        let p = node(i);
        disc(p.x, p.y, 3.5, ink('0'));
        match lit_node(item, i) {
            Some(m) => disc(p.x, p.y, 2.5, node_ink(m)),
            None => disc(p.x, p.y, 2.5, ink('7')),
        }
    }
    if !item.known {
        // A question, in grey.
        rect(c.x - 2.0, c.y - 4.0, 4.0, 1.0, ink('m'));
        px(c.x + 2.0, c.y - 3.0, ink('m'));
        px(c.x + 1.0, c.y - 2.0, ink('m'));
        px(c.x, c.y - 1.0, ink('m'));
        px(c.x, c.y + 2.0, ink('m'));
    } else if item.is_anima() {
        disc(c.x, c.y, 4.0, ink('E'));
        disc(c.x, c.y, 2.0, ink('j'));
        px(c.x + (clock * 2.0).sin().round(), c.y, ink('0'));
    }
}

/// A jar's water as pips along the bottom of its cell.
pub fn sips(x: f32, y: f32, item: Item) {
    for i in 0..JAR_SIPS {
        let c = if i < item.sips { ink('z') } else { ink('1') };
        px(f32::from(i).mul_add(2.0, x + 2.0), y, c);
    }
}

/// A parchment card.
pub fn card(x: f32, y: f32, w: f32, h: f32) {
    rect(x + 1.0, y + 1.0, w, h, fade(ink('0'), 0.5));
    rect(x, y, w, h, ink('7'));
    outline(x, y, w, h, ink('3'));
}
