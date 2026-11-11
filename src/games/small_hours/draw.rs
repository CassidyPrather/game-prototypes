//! Everything Small Hours puts on screen.
//!
//! The one idea the picture has to carry is the rule itself: a shade goes
//! for the nearest light. So the ground is painted by whose light reaches
//! it first — the hearth's orange, or a keeper's own colour — and a shade's
//! eyes take the colour of the ground it stands on. Nobody has to count
//! squares; the board is already the answer.
//!
//! No text. The hours are a ruler with a moon on it, the keepers are
//! lanterns in a row, the keys are caps with a drawing over each.

// Drawing is points, radii and colours, over and over.
#![allow(clippy::many_single_char_names)]

use std::f32::consts::{PI, TAU};

use game_prototypes::small_hours::night::{Cell, Ground, Step, Target, Watch, World};
use game_prototypes::small_hours::nights::NIGHTS;
use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};

use super::{Game, Mode, STEP_SECS, layout};
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
    Color::new(c.r, c.g, c.b, a)
}

/// `t` of the way from `a` to `b`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
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

// The night is wirenook's ink and violet; dawn is its sky.
const NIGHT_TOP: Color = hex(0x08_08_1c);
const NIGHT_LOW: Color = hex(0x1c_0e_52);
const SMALL_LOW: Color = hex(0x47_1f_78);
const DAWN_TOP: Color = hex(0x5a_a8_dc);
const DAWN_LOW: Color = hex(0xff_d6_f5);
const COLD_TOP: Color = hex(0x3c_44_5c);
const COLD_LOW: Color = hex(0x8c_8f_a6);
const TILE: Color = hex(0x13_10_38);
const ASH: Color = hex(0x5c_58_78);
const ICE: Color = hex(0xd7_fb_ff);
const BANANA: Color = hex(0xff_f9_b1);
const INK: Color = hex(0x06_04_14);
const SHADE_RIM: Color = hex(0x4a_22_a8);
const PINE: Color = hex(0x0d_24_3a);
const PINE_LIT: Color = hex(0x1b_42_56);
const BARK: Color = hex(0x2a_1c_30);
const STONE: Color = hex(0x3a_36_58);
/// The hearth's light.
pub const EMBER: Color = hex(0xff_9a_3c);
const KEEPERS: [Color; 4] = [
    hex(0x73_f4_ff),
    hex(0xff_8f_d8),
    hex(0xa6_ff_b4),
    hex(0xb9_a8_ff),
];

/// The colour keeper `i` wears and lights the ground with.
pub const fn keeper_colour(i: usize) -> Color {
    KEEPERS[i % KEEPERS.len()]
}

const fn light_colour(world: &World, target: Target) -> Color {
    match target {
        Target::Hearth => {
            if world.lit() {
                EMBER
            } else {
                ASH
            }
        }
        Target::Keeper(i) => keeper_colour(i),
    }
}

/// A frame-unit drawing surface with its own origin and size, so the same
/// keeper can be drawn on the board and on the menu's card.
struct Pen<'a> {
    frame: &'a Frame,
    origin: Vec2,
    k: f32,
}

impl Pen<'_> {
    fn px(&self, p: Vec2) -> Vec2 {
        self.origin + p * (self.k * self.frame.scale())
    }

    fn circle(&self, p: Vec2, r: f32, c: Color) {
        if r * self.k * self.frame.scale() > 14.0 {
            self.frame.poly(self.px(p), 48, r * self.k, 0.0, c);
        } else {
            self.frame.circle(self.px(p), r * self.k, c);
        }
    }

    fn ring(&self, p: Vec2, r: f32, thickness: f32, c: Color) {
        self.frame
            .poly_lines(self.px(p), 32, r * self.k, 0.0, thickness * self.k, c);
    }

    fn line(&self, a: Vec2, b: Vec2, thickness: f32, c: Color) {
        self.frame
            .line(self.px(a), self.px(b), thickness * self.k, c);
    }

    fn tri(&self, a: Vec2, b: Vec2, c: Vec2, colour: Color) {
        Frame::triangle(self.px(a), self.px(b), self.px(c), colour);
    }

    fn rect(&self, centre: Vec2, size: Vec2, c: Color) {
        self.frame.rect_centred(self.px(centre), size * self.k, c);
    }

    /// A rectangle with rounded corners.
    fn round(&self, centre: Vec2, size: Vec2, radius: f32, c: Color) {
        let r = radius.min(size.x * 0.5).min(size.y * 0.5);
        self.rect(centre, vec2(r.mul_add(-2.0, size.x), size.y), c);
        self.rect(centre, vec2(size.x, r.mul_add(-2.0, size.y)), c);
        let reach = size * 0.5 - Vec2::splat(r);
        for corner in [
            vec2(1.0, 1.0),
            vec2(-1.0, 1.0),
            vec2(1.0, -1.0),
            vec2(-1.0, -1.0),
        ] {
            self.frame
                .circle(self.px(centre + reach * corner), r * self.k, c);
        }
    }

    fn sparkle(&self, p: Vec2, r: f32, c: Color) {
        self.frame.sparkle(self.px(p), r * self.k, c);
    }

    /// Soft light: a stack of faint discs, brightest in the middle.
    fn glow(&self, p: Vec2, r: f32, c: Color, strength: f32) {
        const LAYERS: u8 = 18;
        for i in 0..LAYERS {
            let t = f32::from(LAYERS - i) / f32::from(LAYERS);
            self.circle(p, r * t, alpha(c, strength / f32::from(LAYERS)));
        }
    }

    /// A small arrowhead at `p` pointing along `dir`.
    fn chevron(&self, p: Vec2, dir: Vec2, size: f32, c: Color) {
        let across = vec2(-dir.y, dir.x);
        self.tri(
            p + dir * size,
            p - dir * (size * 0.5) + across * (size * 0.8),
            p - dir * (size * 0.5) - across * (size * 0.8),
            c,
        );
    }
}

/// A speck of something: a spark off the fire, a puff off a lantern.
pub struct Mote {
    p: Vec2,
    v: Vec2,
    /// Negative while it waits to appear.
    age: f32,
    life: f32,
    colour: Color,
    size: f32,
    smoke: bool,
}

/// Sparks flying out from `at`, after `delay` seconds.
pub fn burst(motes: &mut Vec<Mote>, at: Vec2, colour: Color, count: usize, delay: f32) {
    for i in 0..count {
        let angle = (i as f32 / count as f32).mul_add(TAU, fastrand::f32() * 0.6);
        motes.push(Mote {
            p: at,
            v: Vec2::from_angle(angle) * fastrand::f32().mul_add(50.0, 40.0),
            age: -delay,
            life: fastrand::f32().mul_add(0.3, 0.45),
            colour,
            size: fastrand::f32().mul_add(2.5, 2.5),
            smoke: false,
        });
    }
}

/// Smoke rising from `at`, after `delay` seconds.
pub fn smoke(motes: &mut Vec<Mote>, at: Vec2, count: usize, delay: f32) {
    for _ in 0..count {
        motes.push(Mote {
            p: at + vec2(fastrand::f32().mul_add(16.0, -8.0), 0.0),
            v: vec2(
                fastrand::f32().mul_add(24.0, -12.0),
                fastrand::f32().mul_add(-30.0, -18.0),
            ),
            age: fastrand::f32().mul_add(-0.15, -delay),
            life: fastrand::f32().mul_add(0.6, 0.7),
            colour: ASH,
            size: fastrand::f32().mul_add(5.0, 4.0),
            smoke: true,
        });
    }
}

/// Move the motes on and drop the spent ones.
pub fn drift(motes: &mut Vec<Mote>, dt: f32) {
    for mote in motes.iter_mut() {
        mote.age += dt;
        if mote.age > 0.0 {
            mote.p += mote.v * dt;
            mote.v *= 1.0 - (dt * 2.5).min(1.0);
        }
    }
    motes.retain(|mote| mote.age < mote.life);
}

/// The two moments the screen is between, and how far along it is.
pub struct Shown<'a> {
    from: &'a World,
    to: &'a World,
    /// How far the keepers are from `from` to `to`, `0..=1`.
    keepers: f32,
    /// How far the shades are. They start after the keepers do.
    shades: f32,
    /// The moment the ground is lit for.
    pub tint: &'a World,
    /// Whether the night is running backward.
    rewinding: bool,
}

pub fn shown(game: &Game) -> Shown<'_> {
    if let Mode::Rewind { film, age, secs } = &game.mode {
        let last = film.len() - 1;
        let pos = (1.0 - smooth(age / secs)) * last as f32;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let i = (pos.floor() as usize).min(last);
        let j = (i + 1).min(last);
        let t = pos - i as f32;
        return Shown {
            from: &film[i],
            to: &film[j],
            keepers: t,
            shades: t,
            tint: &film[i],
            rewinding: true,
        };
    }
    let history = game.watch.history();
    let to = &history[history.len() - 1];
    if game.moved && history.len() >= 2 && game.since < STEP_SECS {
        let p = game.since / STEP_SECS;
        Shown {
            from: &history[history.len() - 2],
            to,
            keepers: smooth(p / 0.55),
            shades: smooth((p - 0.4) / 0.6),
            tint: to,
            rewinding: false,
        }
    } else {
        Shown {
            from: to,
            to,
            keepers: 1.0,
            shades: 1.0,
            tint: to,
            rewinding: false,
        }
    }
}

/// What colour every cell wants to be: the colour of the light that
/// reaches it first, fainter the further that light is.
pub fn tile_targets(watch: &Watch, world: &World) -> Vec<Color> {
    let ground = watch.ground();
    let mut tiles = Vec::new();
    for y in 0..ground.height {
        for x in 0..ground.width {
            let at = Cell::new(x, y);
            let (target, toward) = world.nearest_light(ground, at);
            let reach = f32::from(at.distance(toward));
            let strength = reach.mul_add(-0.065, 0.56).max(0.13);
            tiles.push(mix(TILE, light_colour(world, target), strength));
        }
    }
    tiles
}

fn lerp_cell(ground: &Ground, a: Cell, b: Cell, t: f32) -> Vec2 {
    layout::centre(ground, a).lerp(layout::centre(ground, b), t)
}

fn step_dir(step: Step) -> Vec2 {
    let (dx, dy) = step.delta();
    vec2(f32::from(dx), f32::from(dy))
}

/// When mark `i` of three appears, in seconds after a kept dawn.
pub fn mark_born(i: u8) -> f32 {
    f32::from(i).mul_add(0.3, 0.9)
}

/// How far dawn has come, `0..=1`, and whether it found the hearth lit.
fn dawn(game: &Game) -> (f32, bool) {
    match game.mode {
        Mode::Kept { age, .. } => (smooth(age / 1.4), true),
        Mode::Cold { age } => (smooth(age / 0.9), false),
        _ => (0.0, true),
    }
}

pub fn scene(frame: &Frame, game: &Game) {
    let pen = Pen {
        frame,
        origin: frame.at(Vec2::ZERO),
        k: 1.0,
    };
    let shown = shown(game);
    let ground = game.watch.ground();
    let side = layout::cell(ground);
    let (dawn, kept) = dawn(game);
    let hours = f32::from(game.watch.level().hours);
    let hour = (f32::from(shown.to.hour) - f32::from(shown.from.hour))
        .mul_add(shown.keepers, f32::from(shown.from.hour));

    sky(&pen, game.clock, hour / hours, dawn, kept);

    // The ground, by whose light reaches it.
    for y in 0..ground.height {
        for x in 0..ground.width {
            #[allow(clippy::cast_sign_loss)]
            let index = y as usize * ground.width as usize + x as usize;
            let mut colour = game.tiles.get(index).copied().unwrap_or(TILE);
            if dawn > 0.0 {
                let day = if kept { ICE } else { COLD_LOW };
                colour = mix(colour, day, dawn * 0.45);
            }
            let centre = layout::centre(ground, Cell::new(x, y));
            pen.round(centre, Vec2::splat(side - 4.0), side * 0.16, colour);
        }
    }
    if let Some(at) = game.hover
        && matches!(game.mode, Mode::Night)
        && ground.open(at)
        && shown.to.keepers[game.me()].at.distance(at) == 1
    {
        let centre = layout::centre(ground, at);
        pen.frame.rect_lines_centred(
            pen.px(centre),
            Vec2::splat(side - 14.0),
            2.0,
            alpha(ICE, 0.35),
        );
    }

    beds(&pen, game, ground, side);

    let settled = shown.shades >= 1.0 && !shown.rewinding && matches!(game.mode, Mode::Night);
    if settled {
        threads(&pen, game, shown.to, ground, side);
    }

    hearth(&pen, game, &shown, ground, side, dawn);
    for y in 0..ground.height {
        for x in 0..ground.width {
            let at = Cell::new(x, y);
            if ground.tree(at) {
                let lit = at.distance(ground.hearth) <= 2 && shown.to.lit();
                tree(&pen, layout::centre(ground, at), side, lit);
            }
        }
    }

    keepers(&pen, game, &shown, ground, side, dawn > 0.0 && kept);
    shades(&pen, game, &shown, ground, side, dawn);

    for mote in &game.motes {
        if mote.age < 0.0 {
            continue;
        }
        let t = mote.age / mote.life;
        if mote.smoke {
            pen.circle(
                mote.p,
                mote.size * t.mul_add(1.2, 0.6),
                alpha(mote.colour, (1.0 - t) * 0.35),
            );
        } else {
            pen.sparkle(mote.p, mote.size * (1.0 - t), alpha(mote.colour, 1.0 - t));
        }
    }

    if shown.rewinding {
        rewind_wash(&pen, game.clock);
    }
    if !game.stepped && matches!(game.mode, Mode::Night) {
        arrow_caps(&pen, game, ground, side);
    }
    hud(&pen, game, &shown, hour, dawn, kept);
}

fn sky(pen: &Pen, clock: f32, through: f32, dawn: f32, kept: bool) {
    let frame = pen.frame;
    let (top_left, bottom_right) = frame.window_bounds();
    let low = mix(NIGHT_LOW, SMALL_LOW, through * through);
    let (day_top, day_low) = if kept {
        (DAWN_TOP, DAWN_LOW)
    } else {
        (COLD_TOP, COLD_LOW)
    };
    let top = mix(NIGHT_TOP, day_top, dawn);
    let low = mix(low, day_low, dawn);
    frame.gradient(
        frame.at(top_left),
        bottom_right - top_left,
        [top, top, low, low],
    );
    // Stars, wirenook's four-pointed kind, going out as the sky lightens.
    for i in 0..46_u16 {
        let seed = f32::from(i);
        let x = (seed * 127.1).sin().mul_add(43_758.547, 0.0).fract().abs();
        let y = (seed * 311.7).sin().mul_add(12_543.123, 0.0).fract().abs();
        let p = vec2(
            x.mul_add(bottom_right.x - top_left.x, top_left.x),
            y.mul_add(bottom_right.y - top_left.y, top_left.y),
        );
        let twinkle = seed.mul_add(1.7, clock * (0.6 + x)).sin().mul_add(0.5, 0.5);
        let level = twinkle.mul_add(0.5, 0.2) * (1.0 - dawn);
        let colour = if i % 5 == 0 { BANANA } else { ICE };
        pen.sparkle(p, x.mul_add(3.0, 2.0), alpha(colour, level * 0.6));
    }
}

fn beds(pen: &Pen, game: &Game, ground: &Ground, side: f32) {
    for (i, &bed) in ground.beds.iter().enumerate() {
        let centre = layout::centre(ground, bed) + vec2(0.0, side * 0.24);
        let colour = keeper_colour(i);
        // Opaque on purpose: a rounded box is overlapping pieces, and a
        // see-through one shows its seams.
        let awake = i < game.watch.keepers();
        let depth = if awake { 0.62 } else { 0.74 };
        pen.round(
            centre,
            vec2(side * 0.6, side * 0.2),
            side * 0.08,
            mix(colour, TILE, depth),
        );
        pen.round(
            centre - vec2(side * 0.2, 0.0),
            vec2(side * 0.14, side * 0.14),
            side * 0.05,
            mix(colour, TILE, depth - 0.22),
        );
    }
}

/// A line of drifting dots from every shade to the light it is after, and
/// an arrowhead on the step it means to take.
fn threads(pen: &Pen, game: &Game, world: &World, ground: &Ground, side: f32) {
    let fade = smooth((game.since - STEP_SECS) / 0.2);
    for (shade, intent) in world.shades.iter().zip(world.intents(ground)) {
        let (Some(from), Some(intent)) = (shade, intent) else {
            continue;
        };
        let colour = light_colour(world, intent.target);
        let a = layout::centre(ground, *from);
        let b = layout::centre(ground, intent.toward);
        let length = a.distance(b);
        let dir = (b - a).normalize_or_zero();
        let gap = 13.0;
        let phase = (game.clock * 22.0).rem_euclid(gap);
        let along = side.mul_add(0.34, phase);
        let dots = ((side.mul_add(-0.3, length) - along) / gap).ceil().max(0.0);
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        for dot in 0..dots as u16 {
            let at = f32::from(dot).mul_add(gap, along);
            pen.circle(a + dir * at, 1.8, alpha(colour, 0.5 * fade));
        }
        if let Some(next) = intent.next {
            let to = layout::centre(ground, next);
            let step = (to - a).normalize_or_zero();
            pen.chevron(
                a.lerp(to, 0.5),
                step,
                side * 0.11,
                alpha(colour, 0.9 * fade),
            );
        }
    }
}

fn hearth(pen: &Pen, game: &Game, shown: &Shown, ground: &Ground, side: f32, dawn: f32) {
    let centre = layout::centre(ground, ground.hearth);
    let world = if shown.shades > 0.85 {
        shown.to
    } else {
        shown.from
    };
    let full = f32::from(game.watch.level().embers.max(1));
    let embers = f32::from(world.embers);
    hearth_at(pen, centre, side, embers, full, game.clock, dawn);
}

fn hearth_at(pen: &Pen, centre: Vec2, side: f32, embers: f32, full: f32, clock: f32, dawn: f32) {
    let flicker = (clock * 9.0)
        .sin()
        .mul_add(0.04, (clock * 5.3).sin() * 0.03);
    if embers > 0.0 {
        let reach = (embers / full).mul_add(0.9, 1.5) * side * (1.0 + flicker);
        pen.glow(centre, reach, EMBER, 0.34 * dawn.mul_add(-0.8, 1.0));
    }
    // A ring of stones.
    for i in 0..8_u8 {
        let angle = f32::from(i) * TAU / 8.0 + 0.3;
        let p = centre
            + Vec2::from_angle(angle) * vec2(side * 0.36, side * 0.3)
            + vec2(0.0, side * 0.1);
        pen.circle(p, side * 0.09, STONE);
    }
    // Two logs.
    let base = centre + vec2(0.0, side * 0.12);
    pen.line(
        base + vec2(-side * 0.2, side * 0.06),
        base + vec2(side * 0.2, -side * 0.04),
        side * 0.1,
        BARK,
    );
    pen.line(
        base + vec2(-side * 0.2, -side * 0.04),
        base + vec2(side * 0.2, side * 0.06),
        side * 0.1,
        BARK,
    );
    if embers <= 0.0 {
        // Cold: a thread of smoke.
        for i in 0..4_u8 {
            let t = f32::from(i).mul_add(0.25, clock * 0.5).fract();
            let p = base + vec2(t.mul_add(9.0, clock).sin() * side * 0.06, -t * side * 0.6);
            pen.circle(p, side * t.mul_add(0.08, 0.04), alpha(ASH, (1.0 - t) * 0.4));
        }
        return;
    }
    // One tongue of flame per ember, the middle one tallest.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let tongues = embers.round().clamp(1.0, 3.0) as u8;
    for i in 0..tongues {
        let offset = f32::from(tongues - 1).mul_add(-0.5, f32::from(i));
        let wave = f32::from(i).mul_add(2.1, clock * 7.0).sin();
        let height = side * wave.mul_add(0.05, offset.abs().mul_add(-0.1, 0.42));
        let foot = base + vec2(offset * side * 0.2, 0.0);
        let tip = foot + vec2(wave * side * 0.04, -height);
        let half = side * 0.13;
        pen.tri(tip, foot + vec2(half, 0.0), foot - vec2(half, 0.0), EMBER);
        pen.circle(foot, half, EMBER);
        let core = foot.lerp(tip, 0.55);
        pen.tri(
            core,
            foot + vec2(half * 0.5, 0.0),
            foot - vec2(half * 0.5, 0.0),
            BANANA,
        );
        pen.circle(foot, half * 0.5, BANANA);
    }
}

fn tree(pen: &Pen, centre: Vec2, side: f32, lit: bool) {
    let colour = if lit { PINE_LIT } else { PINE };
    pen.rect(
        centre + vec2(0.0, side * 0.3),
        vec2(side * 0.1, side * 0.2),
        BARK,
    );
    for (i, (y, half)) in [(0.22, 0.34), (0.02, 0.27), (-0.16, 0.19)]
        .into_iter()
        .enumerate()
    {
        let base = centre + vec2(0.0, side * y);
        let tint = mix(colour, ICE, i as f32 * 0.035);
        pen.tri(
            base + vec2(0.0, -side * 0.3),
            base + vec2(side * half, 0.0),
            base - vec2(side * half, 0.0),
            tint,
        );
    }
}

fn keepers(pen: &Pen, game: &Game, shown: &Shown, ground: &Ground, side: f32, cheering: bool) {
    let me = game.me();
    for (i, to) in shown.to.keepers.iter().enumerate() {
        let from = shown.from.keepers.get(i).copied().unwrap_or(*to);
        let lit = if shown.shades > 0.85 {
            to.lit
        } else {
            from.lit
        };
        let moving = from.at != to.at;
        let hop = if moving {
            (shown.keepers * PI).sin() * side * 0.12
        } else {
            0.0
        };
        let cheer = if cheering && lit {
            game.clock.mul_add(9.0, i as f32).sin().abs() * side * 0.12
        } else {
            0.0
        };
        let p = lerp_cell(ground, from.at, to.at, shown.keepers) - vec2(0.0, hop + cheer);
        // Keepers sharing a cell stand a little apart.
        let crowd = shown.to.keepers[..i]
            .iter()
            .filter(|k| k.at == to.at)
            .count();
        let p = p + vec2(crowd as f32 * side * 0.14, crowd as f32 * side * -0.06);
        let mine = i == me && matches!(game.mode, Mode::Night);
        if mine {
            let pulse = (game.clock * 3.0).sin().mul_add(0.5, 0.5);
            pen.ring(
                p + vec2(0.0, side * 0.3),
                side * pulse.mul_add(0.03, 0.3),
                2.0,
                alpha(keeper_colour(i), pulse.mul_add(0.3, 0.35)),
            );
        }
        keeper(pen, p, side, keeper_colour(i), lit, game.clock + i as f32);
        // What a remembered keeper is about to do.
        let settled = shown.shades >= 1.0 && !shown.rewinding && matches!(game.mode, Mode::Night);
        if settled
            && lit
            && let Some(step) = game.watch.remembered(i)
        {
            let colour = alpha(keeper_colour(i), 0.85);
            if step == Step::Wait {
                pen.circle(p - vec2(0.0, side * 0.46), side * 0.045, colour);
            } else {
                let dir = step_dir(step);
                pen.chevron(p + dir * side * 0.44, dir, side * 0.09, colour);
            }
        }
    }
}

fn keeper(pen: &Pen, p: Vec2, side: f32, colour: Color, lit: bool, clock: f32) {
    let body = p + vec2(-side * 0.04, side * 0.1);
    let head = p + vec2(-side * 0.04, -side * 0.14);
    let lantern = p + vec2(side * 0.24, side * 0.08);
    if !lit {
        // A ghost: the outline of where it would be, lantern dark.
        let faint = alpha(colour, 0.38);
        pen.ring(body, side * 0.19, 1.6, faint);
        pen.ring(head, side * 0.14, 1.6, faint);
        pen.ring(lantern, side * 0.06, 1.4, alpha(ASH, 0.6));
        return;
    }
    let breathe = (clock * 2.4).sin() * side * 0.008;
    pen.glow(lantern, side * 0.62, colour, 0.3);
    let robe = mix(colour, INK, 0.62);
    pen.circle(body, side.mul_add(0.2, breathe), robe);
    pen.rect(
        body + vec2(0.0, side * 0.1),
        vec2(side * 0.4, side * 0.2),
        robe,
    );
    pen.circle(body + vec2(-side * 0.1, side * 0.2), side * 0.1, robe);
    pen.circle(body + vec2(side * 0.1, side * 0.2), side * 0.1, robe);
    // Hood, face, eyes.
    pen.circle(head, side * 0.16, colour);
    pen.circle(
        head + vec2(side * 0.02, side * 0.02),
        side * 0.11,
        hex(0x2a_1a_3c),
    );
    for eye in [-1.0_f32, 1.0] {
        pen.circle(
            head + vec2(eye.mul_add(side * 0.045, side * 0.03), side * 0.02),
            side * 0.024,
            BANANA,
        );
    }
    // The lantern on its handle.
    pen.line(
        body + vec2(side * 0.12, 0.0),
        lantern - vec2(0.0, side * 0.1),
        1.6,
        robe,
    );
    pen.round(
        lantern,
        vec2(side * 0.13, side * 0.17),
        side * 0.04,
        mix(colour, BANANA, 0.5),
    );
    pen.circle(lantern, side * 0.04, hex(0xff_ff_ff));
}

fn shades(pen: &Pen, game: &Game, shown: &Shown, ground: &Ground, side: f32, dawn: f32) {
    let hearth = layout::centre(ground, ground.hearth);
    for (i, from) in shown.from.shades.iter().enumerate() {
        let Some(from) = from else { continue };
        let to = shown.to.shades.get(i).copied().flatten();
        let (p, fade) = to.map_or_else(
            // It walked into the fire: go there and go out.
            || {
                let p = layout::centre(ground, *from).lerp(hearth, shown.shades);
                (p, 1.0 - shown.shades)
            },
            |to| (lerp_cell(ground, *from, to, shown.shades), 1.0),
        );
        // What it is looking at, as of the moment being settled into.
        let world = if shown.shades > 0.5 {
            shown.to
        } else {
            shown.from
        };
        let at = to.unwrap_or(*from);
        let (target, toward) = world.nearest_light(ground, at);
        let look = (layout::centre(ground, toward) - p).normalize_or_zero();
        let eyes = light_colour(world, target);
        let rise = vec2(0.0, -dawn * side * 0.5);
        shade(
            pen,
            p + rise,
            side,
            look,
            eyes,
            fade * (1.0 - dawn),
            (i as f32).mul_add(1.3, game.clock),
        );
    }
}

fn shade(pen: &Pen, p: Vec2, side: f32, look: Vec2, eyes: Color, level: f32, clock: f32) {
    if level <= 0.01 {
        return;
    }
    let sway = (clock * 2.2).sin();
    let p = p + vec2(0.0, (clock * 1.7).sin() * side * 0.02);
    // A ragged hem under a round head.
    for (i, x) in [-0.16_f32, 0.0, 0.16].into_iter().enumerate() {
        let wag = (i as f32).mul_add(1.9, clock * 3.0).sin() * side * 0.03;
        let foot = p + vec2(x.mul_add(side, wag), side * 0.24);
        pen.circle(foot, side * 0.1, alpha(INK, level));
    }
    pen.circle(p, side * 0.3, alpha(SHADE_RIM, 0.55 * level));
    pen.circle(p, side * 0.27, alpha(INK, level));
    pen.rect(
        p + vec2(0.0, side * 0.12),
        vec2(side * 0.52, side * 0.24),
        alpha(INK, level),
    );
    let gaze = p + look * side * 0.07 + vec2(sway * side * 0.01, -side * 0.03);
    for eye in [-1.0_f32, 1.0] {
        let at = gaze + vec2(eye * side * 0.1, 0.0);
        pen.circle(at, side * 0.085, alpha(eyes, 0.25 * level));
        pen.circle(at, side * 0.05, alpha(eyes, level));
    }
}

/// Faint streaks sliding the wrong way while the night runs backward.
fn rewind_wash(pen: &Pen, clock: f32) {
    for i in 0..9_u8 {
        let seed = f32::from(i);
        let y = (seed * 71.3).sin().mul_add(0.5, 0.5) * 600.0;
        let x = 800.0 - seed.mul_add(233.0, clock * 900.0).rem_euclid(1000.0);
        pen.rect(vec2(x, y), vec2(120.0, 1.5), alpha(ICE, 0.16));
    }
    pen.rect(vec2(400.0, 300.0), vec2(800.0, 600.0), alpha(ICE, 0.04));
}

/// Four arrow caps around the keeper, until the first step is taken.
fn arrow_caps(pen: &Pen, game: &Game, ground: &Ground, side: f32) {
    let me = game.watch.world().keepers[game.me()].at;
    let pulse = (game.clock * 3.0).sin().mul_add(0.5, 0.5);
    for step in [Step::North, Step::East, Step::South, Step::West] {
        let to = me.stepped(step);
        if !ground.open(to) {
            continue;
        }
        let p = layout::centre(ground, to);
        pen.frame.arrow_cap(
            step_dir(step),
            pen.px(p),
            side * 0.46,
            alpha(BANANA, pulse.mul_add(0.5, 0.4)),
        );
    }
}

fn hud(pen: &Pen, game: &Game, shown: &Shown, hour: f32, dawn: f32, kept: bool) {
    let level = game.watch.level();
    let hours = f32::from(level.hours);

    // The hours: a ruler, a moon crossing it, and the sun at the far end.
    let gap = (340.0 / hours).min(46.0);
    let left = hours.mul_add(-gap * 0.5, 400.0);
    let ruler = if kept { 1.0 - dawn } else { 1.0 };
    for i in 0..=level.hours {
        let x = f32::from(i).mul_add(gap, left);
        let past = f32::from(i) < hour - 0.01;
        let colour = alpha(ICE, if past { 0.22 } else { 0.7 } * ruler);
        pen.rect(vec2(x, layout::RULER_Y + 12.0), vec2(2.0, 9.0), colour);
    }
    let sun = vec2(hours.mul_add(gap, left), layout::RULER_Y - 6.0);
    let sun_colour = if kept { BANANA } else { mix(BANANA, ASH, dawn) };
    let near = smooth((hour - (hours - 2.0)) / 2.0);
    let sun_size = near.mul_add(2.0, 6.0) * ruler;
    pen.glow(sun, sun_size * 2.6, sun_colour, near * 0.5 * ruler);
    pen.sparkle(sun, sun_size * 1.7, alpha(sun_colour, 0.55 * ruler));
    pen.circle(sun, sun_size, alpha(sun_colour, ruler));
    let moon = vec2(hour.mul_add(gap, left), layout::RULER_Y - 6.0);
    let moon_level = ruler * smooth((hour - (hours - 1.0)) / 1.0).mul_add(-0.8, 1.0);
    pen.circle(moon, 8.0, alpha(ICE, moon_level));
    pen.circle(
        moon + vec2(4.0, -2.5),
        7.0,
        alpha(mix(NIGHT_TOP, DAWN_TOP, dawn), moon_level),
    );

    marks(pen, game, dawn);
    pips(pen, game, dawn, kept);
    lanterns(pen, game, shown.to);
    caps(pen, game, dawn, kept);
}

/// Dawn on a lit hearth: three marks where the ruler was, each over a
/// small drawing of what it is for.
fn marks(pen: &Pen, game: &Game, dawn: f32) {
    if let Mode::Kept { age, earned, .. } = game.mode {
        let ink = alpha(hex(0x27_14_52), 0.75 * dawn);
        for i in 0..3_u8 {
            let pop = smooth((age - mark_born(i)) / 0.25);
            let p = vec2(
                (f32::from(i) - 1.0).mul_add(64.0, 400.0),
                layout::RULER_Y - 6.0,
            );
            pen.sparkle(p, 16.0, alpha(hex(0x27_14_52), 0.3 * dawn));
            if earned[usize::from(i)] && pop > 0.0 {
                let size = (1.0 - pop).mul_add(10.0, 14.0) * pop.sqrt();
                pen.sparkle(p, size, BANANA);
                pen.sparkle(p, size * 0.5, hex(0xff_ff_ff));
            }
            let below = p + vec2(0.0, 27.0);
            match i {
                // The hearth, lit.
                0 => {
                    pen.tri(
                        below - vec2(0.0, 8.0),
                        below + vec2(5.0, 4.0),
                        below + vec2(-5.0, 4.0),
                        ink,
                    );
                    pen.circle(below + vec2(0.0, 3.0), 5.0, ink);
                }
                // As few lanterns as the night can be kept with.
                1 => {
                    let par = game.watch.level().par;
                    for k in 0..par {
                        let x = f32::from(par - 1).mul_add(-0.5, f32::from(k)) * 11.0;
                        pen.round(below + vec2(x, 0.0), vec2(7.0, 10.0), 2.0, ink);
                    }
                }
                // Every lantern still lit.
                _ => {
                    pen.round(below, vec2(7.0, 10.0), 2.0, ink);
                    pen.ring(below, 9.5, 1.5, ink);
                }
            }
        }
    }
}

/// The nights, as a row of pips with their marks under them.
fn pips(pen: &Pen, game: &Game, dawn: f32, kept: bool) {
    for night in 0..NIGHTS.len() {
        let p = layout::pip(night);
        let best = game.best[night];
        let here = night == game.night;
        let ink = if kept {
            mix(ICE, hex(0x27_14_52), dawn)
        } else {
            ICE
        };
        if here {
            pen.ring(p, 8.5, 2.0, alpha(ink, 0.9));
        }
        if best > 0 {
            pen.circle(p, 5.0, alpha(mix(BANANA, ink, 0.2), 0.95));
        } else {
            pen.ring(p, 4.0, 1.5, alpha(ink, 0.6));
        }
        for mark in 0..best {
            let x = f32::from(best - 1)
                .mul_add(-0.5, f32::from(mark))
                .mul_add(6.0, p.x);
            pen.sparkle(vec2(x, p.y + 15.0), 3.2, alpha(mix(BANANA, ink, 0.3), 0.95));
        }
    }
}

/// The keepers, as lanterns: lit, out, or not yet woken.
fn lanterns(pen: &Pen, game: &Game, world: &World) {
    for i in 0..game.watch.ground().beds.len() {
        let p = vec2(
            (i as f32).mul_add(layout::LANTERN_GAP, layout::LANTERN_X),
            layout::CAPS_Y,
        );
        let colour = keeper_colour(i);
        match world.keepers.get(i) {
            Some(keeper) if keeper.lit => {
                pen.glow(p, 16.0, colour, 0.35);
                pen.round(p, vec2(12.0, 16.0), 3.0, mix(colour, BANANA, 0.4));
                pen.circle(p, 3.0, hex(0xff_ff_ff));
            }
            Some(_) => {
                pen.round(
                    p,
                    vec2(12.0, 16.0),
                    3.0,
                    mix(mix(colour, ASH, 0.7), TILE, 0.3),
                );
            }
            None => {
                pen.frame
                    .rect_lines_centred(pen.px(p), vec2(12.0, 16.0), 1.5, alpha(ICE, 0.3));
            }
        }
        pen.line(
            p - vec2(3.0, 11.0),
            p + vec2(3.0, -11.0),
            1.5,
            alpha(ICE, 0.4),
        );
    }
}

/// The keys along the bottom, each under a drawing of what it does.
fn caps(pen: &Pen, game: &Game, dawn: f32, kept: bool) {
    let ink = if kept {
        mix(ICE, hex(0x27_14_52), dawn)
    } else {
        ICE
    };
    let quiet = alpha(ink, 0.6);
    let frame = pen.frame;
    let cap = Vec2::splat(layout::CAP);
    let icon = |p: Vec2| p - vec2(0.0, 28.0);

    // Z: one hour back. A hooked arrow.
    frame.key_cap(Some('Z'), pen.px(layout::UNDO), cap, quiet);
    let p = icon(layout::UNDO);
    frame.arc(pen.px(p), 6.0, 2.0, -90.0, 250.0, quiet);
    pen.chevron(p + vec2(-2.0, -6.5), vec2(-1.0, 0.0), 4.0, quiet);

    // R: the whole night over. Back to the bar.
    frame.key_cap(Some('R'), pen.px(layout::RESTART), cap, quiet);
    let p = icon(layout::RESTART);
    pen.rect(p - vec2(9.0, 0.0), vec2(2.0, 11.0), quiet);
    pen.chevron(p - vec2(2.0, 0.0), vec2(-1.0, 0.0), 4.5, quiet);
    pen.chevron(p + vec2(6.0, 0.0), vec2(-1.0, 0.0), 4.5, quiet);

    // Space: stay where you are. The dot a waiting keeper shows.
    frame.key_cap(
        None,
        pen.px(layout::WAIT),
        vec2(layout::WAIT_W, layout::CAP),
        quiet,
    );
    pen.circle(icon(layout::WAIT), 3.0, quiet);

    // Return: on, to dawn, or to the next night.
    let wanted = !matches!(game.mode, Mode::Night | Mode::Rewind { .. });
    let pulse = (game.clock * 3.0).sin().mul_add(0.5, 0.5);
    let colour = if wanted {
        alpha(
            if kept { hex(0x27_14_52) } else { BANANA },
            pulse.mul_add(0.4, 0.6),
        )
    } else if game.sleeping {
        alpha(BANANA, 0.9)
    } else {
        quiet
    };
    frame.enter_cap(
        pen.px(layout::ENTER),
        vec2(layout::ENTER_W, layout::CAP),
        colour,
    );
    let p = icon(layout::ENTER);
    pen.chevron(p - vec2(5.0, 0.0), vec2(1.0, 0.0), 4.5, colour);
    pen.chevron(p + vec2(4.0, 0.0), vec2(1.0, 0.0), 4.5, colour);
    speaker(pen, game.audio.is_muted(), quiet);
}

/// The mute toggle: a speaker, with its sound or struck through.
fn speaker(pen: &Pen, muted: bool, colour: Color) {
    let p = layout::SPEAKER;
    pen.rect(p - vec2(6.0, 0.0), vec2(5.0, 7.0), colour);
    pen.tri(
        p - vec2(6.0, 0.0),
        p + vec2(1.0, -8.0),
        p + vec2(1.0, 8.0),
        colour,
    );
    if muted {
        pen.line(p + vec2(4.0, -4.0), p + vec2(11.0, 4.0), 2.0, colour);
        pen.line(p + vec2(4.0, 4.0), p + vec2(11.0, -4.0), 2.0, colour);
    } else {
        pen.line(p + vec2(5.0, -3.0), p + vec2(5.0, 3.0), 2.0, colour);
        pen.line(p + vec2(9.0, -6.0), p + vec2(9.0, 6.0), 2.0, colour);
    }
}

pub fn emblem(frame: &Frame, centre: Vec2, width: f32, clock: f32) {
    let pen = Pen {
        frame,
        origin: centre,
        k: width / 300.0,
    };
    let side = 78.0;
    let fire = vec2(-10.0, 18.0);
    // Lit ground: the hearth's, the keeper's, and the dark between.
    for (x, colour, level) in [
        (-2.0, keeper_colour(0), 0.34),
        (-1.0, keeper_colour(0), 0.42),
        (0.0, EMBER, 0.5),
        (1.0, EMBER, 0.4),
        (2.0, EMBER, 0.28),
    ] {
        let p = fire + vec2(x * side, side * 0.08);
        pen.round(
            p,
            vec2(side - 5.0, side - 5.0),
            side * 0.16,
            alpha(mix(TILE, colour, level), 0.9),
        );
    }
    hearth_at(&pen, fire, side, 2.0, 2.0, clock, 0.0);
    let hop = (clock * 2.0).sin().max(0.0) * 3.0;
    keeper(
        &pen,
        fire + vec2(-side * 1.15, -hop),
        side,
        keeper_colour(0),
        true,
        clock,
    );
    let creep = (clock * 0.7).sin() * 6.0;
    shade(
        &pen,
        fire + vec2(side.mul_add(1.75, creep), 0.0),
        side,
        vec2(-1.0, 0.0),
        EMBER,
        1.0,
        clock,
    );
    for (x, y, size) in [
        (-120.0, -62.0, 7.0),
        (40.0, -74.0, 5.0),
        (118.0, -52.0, 6.0),
    ] {
        let twinkle = clock.mul_add(1.5, x).sin().mul_add(0.3, 0.6);
        pen.sparkle(vec2(x, y), size, alpha(ICE, twinkle));
    }
}
