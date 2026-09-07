//! macroquad frontend: input in, shapes and sound out.
//!
//! Everything that decides what happens lives in [`leitmotif::sim`]. This
//! file only translates between the window and the sim's world; the sim's
//! [`Cue`]s become sound over in [`audio`].
//!
//! There is no text. The HUD is built from the same shapes the world is
//! made of — a stomper stands for the drums that move it, a wall for the
//! bass, a strike for the lead, a fermata for the pad that charges it —
//! plus key caps, so it reads without reading. The camera follows the
//! player through an unbounded world; the HUD lives in a fixed letterboxed
//! frame.

mod audio;

use audio::Audio;
use macroquad::color::{Color, hsl_to_rgb};
use macroquad::input::{
    KeyCode, MouseButton, get_last_key_pressed, is_key_down, is_key_pressed,
    is_mouse_button_pressed,
};
use macroquad::shapes::{
    draw_arc, draw_circle, draw_circle_lines, draw_ellipse_lines, draw_line, draw_rectangle,
    draw_rectangle_lines, draw_triangle,
};
use macroquad::text::{draw_text, measure_text};
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, clear_background, next_frame, screen_height, screen_width};

use leitmotif::sim::{
    self, Act, Cue, HOME, HOME_RADIUS, InputFrame, MAX_HEARTS, PLAYER_RADIUS, Phase,
    SPINNER_HALF_LEN, SPINNER_HALF_W, STOMPER_RADIUS, STRIKE_RADIUS, Sim, Vec2, WALL_GAP,
    WALL_HALF_W, WALL_PERIOD_Y, WALL_SPACING,
};
use leitmotif::song::{BEATS_PER_BAR, Instrument, Layer, Motif, SONG, STEPS_PER_BAR};

/// The frame everything is laid out in, in world units. The camera shows
/// exactly this much of the world; the HUD is positioned inside it.
const VIEW_W: f32 = 800.0;
const VIEW_H: f32 = 600.0;

const MARGIN: f32 = 16.0;

/// Width of the strips at the bottom, inside the margins.
const STRIP_W: f32 = VIEW_W - 2.0 * MARGIN;

const OVERLAY: Color = Color::new(0.85, 0.85, 0.90, 0.75);
const DIM: Color = Color::new(0.55, 0.55, 0.62, 0.5);
/// For the one thing on screen that is asking for a press.
const HIGHLIGHT: Color = Color::new(1.0, 0.83, 0.42, 0.95);
const PLAYER: Color = Color::new(0.95, 0.95, 1.0, 1.0);
const HEART: Color = Color::new(0.95, 0.35, 0.45, 1.0);
const SHADE: Color = Color::new(0.0, 0.0, 0.0, 0.6);
const HOME_GLOW: Color = Color::new(0.6, 1.0, 0.8, 1.0);
/// Everything the fermata touches goes this colour while it holds.
const HELD: Color = Color::new(0.75, 0.85, 1.0, 1.0);

/// How fast the camera closes on its target, per second.
const CAMERA_CHASE: f32 = 6.0;

/// The camera looks this far ahead of the player, along its velocity.
const CAMERA_LEAD: f32 = 0.2;

/// Seconds an instrument's HUD icon stays lit after one of its notes.
const FLASH_SECS: f32 = 0.18;

fn window_conf() -> Conf {
    Conf {
        window_title: env!("CARGO_PKG_NAME").to_owned(),
        window_width: VIEW_W as i32,
        window_height: VIEW_H as i32,
        high_dpi: true,
        ..Conf::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sim = Sim::new(fresh_seed());
    let mut audio = Audio::load().await;
    let mut camera = Camera::at(sim.player().body.pos);
    let mut hud = Hud::default();
    let mut clock = 0.0_f32;

    loop {
        let input = gather_input();
        let toggle_mute = is_key_pressed(KeyCode::M);
        let dt = get_frame_time();
        clock += dt;

        sim.advance(dt, &input);
        audio.update(sim.cues(), &input, toggle_mute);
        hud.update(dt, sim.cues());
        camera.follow(&sim, dt);

        let view = View::fit(screen_width(), screen_height(), camera.pos);
        draw(&sim, &view, &audio, &hud, clock);
        next_frame().await;
    }
}

/// Wall-clock seed. Swap in `quad-url` here if you want seeds that survive in
/// a shareable URL fragment (see the docs).
fn fresh_seed() -> u64 {
    macroquad::miniquad::date::now().to_bits()
}

fn gather_input() -> InputFrame {
    let axis = |neg: [KeyCode; 2], pos: [KeyCode; 2]| {
        let held = |keys: [KeyCode; 2]| keys.iter().any(|&k| is_key_down(k));
        f32::from(u8::from(held(pos))) - f32::from(u8::from(held(neg)))
    };
    let move_dir = Vec2::new(
        axis([KeyCode::Left, KeyCode::A], [KeyCode::Right, KeyCode::D]),
        axis([KeyCode::Up, KeyCode::W], [KeyCode::Down, KeyCode::S]),
    );
    // Any press leaves the title screen — except mute, which should not
    // start a game the player just asked to silence.
    let start = get_last_key_pressed().is_some_and(|k| k != KeyCode::M)
        || is_mouse_button_pressed(MouseButton::Left);
    InputFrame {
        move_dir,
        start,
        hold: is_key_down(KeyCode::Space),
        next_section: is_key_pressed(KeyCode::Tab),
        toggle_pause: is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape),
        restart: is_key_pressed(KeyCode::R).then(fresh_seed),
    }
}

/// Follows the player with a little lead and a little lag.
struct Camera {
    pos: Vec2,
}

impl Camera {
    const fn at(pos: Vec2) -> Self {
        Self { pos }
    }

    fn follow(&mut self, sim: &Sim, dt: f32) {
        let player = sim.player().body;
        let target = player.interpolated(sim.alpha()) + player.vel * CAMERA_LEAD;
        let blend = 1.0 - (-CAMERA_CHASE * dt).exp();
        self.pos = self.pos.lerp(target, blend);
    }
}

/// Frontend-only state the HUD animates from cues.
#[derive(Default)]
struct Hud {
    /// Seconds each instrument's icon stays lit, indexed by instrument.
    flash: [f32; 4],
}

impl Hud {
    fn update(&mut self, dt: f32, cues: &[Cue]) {
        for flash in &mut self.flash {
            *flash = (*flash - dt).max(0.0);
        }
        for cue in cues {
            match cue {
                Cue::Note { instrument, .. } => self.flash[instrument.index()] = FLASH_SECS,
                Cue::Chord(_) => self.flash[Instrument::Pad.index()] = FLASH_SECS,
                _ => {}
            }
        }
    }

    fn lit(&self, instrument: Instrument) -> f32 {
        self.flash[instrument.index()] / FLASH_SECS
    }
}

/// Maps the world and the HUD frame onto the window. The frame is
/// letterboxed to keep its aspect; the world scrolls under it.
struct View {
    scale: f32,
    /// Top-left of the letterboxed frame, in pixels.
    origin: Vec2,
    /// World position at the centre of the frame.
    camera: Vec2,
}

impl View {
    fn fit(screen_w: f32, screen_h: f32, camera: Vec2) -> Self {
        // A minimised window or hidden tab reports zero size; the floor keeps
        // the mapping from producing infinities.
        let scale = (screen_w / VIEW_W).min(screen_h / VIEW_H).max(f32::EPSILON);
        Self {
            scale,
            origin: Vec2::new(
                VIEW_W.mul_add(-scale, screen_w) * 0.5,
                VIEW_H.mul_add(-scale, screen_h) * 0.5,
            ),
            camera,
        }
    }

    /// A HUD-frame position, in pixels.
    fn hud(&self, at: Vec2) -> Vec2 {
        at * self.scale + self.origin
    }

    /// A world position, in pixels.
    fn world(&self, at: Vec2) -> Vec2 {
        self.hud(at - self.camera + Vec2::new(VIEW_W * 0.5, VIEW_H * 0.5))
    }

    /// The world rectangle the frame shows: top-left and size.
    fn visible(&self) -> (Vec2, Vec2) {
        (
            self.camera - Vec2::new(VIEW_W * 0.5, VIEW_H * 0.5),
            Vec2::new(VIEW_W, VIEW_H),
        )
    }

    // Primitives take pixel positions and frame-unit sizes.

    fn circle(&self, p: Vec2, radius: f32, color: Color) {
        draw_circle(p.x, p.y, radius * self.scale, color);
    }

    fn ring(&self, p: Vec2, radius: f32, thickness: f32, color: Color) {
        draw_circle_lines(p.x, p.y, radius * self.scale, thickness * self.scale, color);
    }

    fn arc(
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

    fn rect(&self, p: Vec2, size: Vec2, color: Color) {
        draw_rectangle(p.x, p.y, size.x * self.scale, size.y * self.scale, color);
    }

    fn rect_lines(&self, p: Vec2, size: Vec2, thickness: f32, color: Color) {
        draw_rectangle_lines(
            p.x,
            p.y,
            size.x * self.scale,
            size.y * self.scale,
            thickness * self.scale,
            color,
        );
    }

    fn line(&self, a: Vec2, b: Vec2, thickness: f32, color: Color) {
        draw_line(a.x, a.y, b.x, b.y, thickness * self.scale, color);
    }

    fn triangle(a: Vec2, b: Vec2, c: Vec2, color: Color) {
        draw_triangle(to_mq(a), to_mq(b), to_mq(c), color);
    }

    /// One character, centred on `p`. The only text on screen: key caps.
    fn glyph(&self, ch: char, p: Vec2, size: f32, color: Color) {
        // Sizes are positive constants, so the cast cannot lose a sign.
        #[allow(clippy::cast_sign_loss)]
        let px = (size * self.scale).max(1.0) as u16;
        let text = ch.to_string();
        let dims = measure_text(&text, None, px, 1.0);
        draw_text(
            &text,
            dims.width.mul_add(-0.5, p.x),
            dims.offset_y.mul_add(0.5, p.y),
            f32::from(px),
            color,
        );
    }

    /// A keyboard key, `size` wide and `height` tall, centred on `p`, with
    /// a character on it or not.
    fn key_cap(&self, ch: Option<char>, p: Vec2, size: Vec2, color: Color) {
        let half = size * 0.5;
        self.rect_lines(p - half * self.scale, size, 2.0, color);
        if let Some(ch) = ch {
            self.glyph(ch, p, size.y * 0.8, color);
        }
    }

    /// A key cap with a solid triangle on it, pointing along `dir`.
    fn arrow_cap(&self, dir: Vec2, p: Vec2, size: f32, color: Color) {
        self.key_cap(None, p, Vec2::new(size, size), color);
        let reach = size * 0.22 * self.scale;
        let across = Vec2::new(-dir.y, dir.x);
        let tip = p + dir * reach;
        let base = p - dir * (reach * 0.6);
        Self::triangle(tip, base + across * reach, base - across * reach, color);
    }
}

const fn to_mq(v: Vec2) -> macroquad::math::Vec2 {
    macroquad::math::Vec2::new(v.x, v.y)
}

/// Each motif has a look, so the rules change is visible before it is felt.
struct Palette {
    background: Color,
    accent: Color,
    stomper: Color,
}

const fn palette_for(motif: Motif) -> Palette {
    match motif {
        Motif::Wander => Palette {
            background: Color::new(0.06, 0.07, 0.10, 1.0),
            accent: Color::new(1.0, 0.72, 0.36, 1.0),
            stomper: Color::new(0.95, 0.55, 0.25, 1.0),
        },
        Motif::Pursuit => Palette {
            background: Color::new(0.12, 0.03, 0.06, 1.0),
            accent: Color::new(1.0, 0.32, 0.36, 1.0),
            stomper: Color::new(0.95, 0.2, 0.25, 1.0),
        },
        Motif::Lullaby => Palette {
            background: Color::new(0.04, 0.05, 0.12, 1.0),
            accent: Color::new(0.62, 0.62, 1.0, 1.0),
            stomper: Color::new(0.32, 0.34, 0.5, 1.0),
        },
    }
}

const fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

/// A slow pulse in `0..=1` for things that want pressing.
fn pulse(clock: f32) -> f32 {
    (clock * 3.0).sin().mul_add(0.5, 0.5)
}

fn draw(sim: &Sim, view: &View, audio: &Audio, hud: &Hud, clock: f32) {
    let music = sim.music();
    let palette = palette_for(music.motif);
    let alpha = sim.alpha();

    // The world breathes on the beat while something is playing, and goes
    // cool and still while the music is held.
    let beat = if sim.phase() == Phase::Playing && !sim.is_paused() && !sim.holding() {
        (1.0 - music.beat_phase()).powi(3) * 0.05
    } else {
        0.0
    };
    let bg = palette.background;
    clear_background(Color::new(0.0, 0.0, 0.0, 1.0));
    let ground = if sim.holding() {
        Color::new(bg.r * 0.6, bg.g * 0.7, bg.b.mul_add(0.8, 0.06), 1.0)
    } else {
        Color::new(bg.r + beat, bg.g + beat, bg.b + beat, 1.0)
    };
    view.rect(view.hud(Vec2::ZERO), Vec2::new(VIEW_W, VIEW_H), ground);

    draw_ground(view, &palette);
    draw_walls(sim, view, &palette);
    draw_spinners(sim, view, &palette);
    draw_home(view, clock);
    draw_strikes(sim, view);
    draw_stompers(sim, view, alpha, &palette);
    draw_player(sim, view, alpha);
    draw_way_home(sim, view, clock);

    draw_hearts(sim, view);
    draw_layers(sim, view, hud, &palette, clock);
    draw_speaker(view, audio);
    draw_transport(sim, view, &palette);
    draw_journey(sim, view, clock);

    match sim.phase() {
        Phase::Title => draw_title(view, clock),
        Phase::Over => draw_over(view, clock),
        Phase::Won => draw_won(view, clock),
        Phase::Playing if sim.is_paused() => draw_paused(view),
        Phase::Playing => {}
    }
}

/// A faint grid of dots so motion reads even on empty ground.
fn draw_ground(view: &View, palette: &Palette) {
    const SPACING: f32 = 100.0;
    let (top_left, size) = view.visible();
    let colour = with_alpha(palette.accent, 0.07);
    let first_x = (top_left.x / SPACING).floor() as i32;
    let last_x = ((top_left.x + size.x) / SPACING).ceil() as i32;
    let first_y = (top_left.y / SPACING).floor() as i32;
    let last_y = ((top_left.y + size.y) / SPACING).ceil() as i32;
    for gx in first_x..=last_x {
        for gy in first_y..=last_y {
            let at = Vec2::new(gx as f32 * SPACING, gy as f32 * SPACING);
            view.circle(view.world(at), 1.5, colour);
        }
    }
}

/// Wall grid indices that touch the frame: `(first_k, last_k, first_m,
/// last_m)`.
fn visible_grid(view: &View) -> (i32, i32, i32, i32) {
    let (top_left, size) = view.visible();
    (
        ((top_left.x - WALL_SPACING) / WALL_SPACING).floor() as i32,
        ((top_left.x + size.x + WALL_SPACING) / WALL_SPACING).ceil() as i32,
        (top_left.y / WALL_PERIOD_Y).floor() as i32 - 1,
        ((top_left.y + size.y) / WALL_PERIOD_Y).ceil() as i32,
    )
}

fn draw_walls(sim: &Sim, view: &View, palette: &Palette) {
    let (first_k, last_k, first_m, last_m) = visible_grid(view);
    let segment = WALL_PERIOD_Y - WALL_GAP;
    let gates = sim.gates();
    for k in first_k..=last_k {
        let x = sim::wall_x(k);
        let solidity = sim.walls()[sim::lane_of_wall(k)];
        let offset = sim::wall_gap_offset(k);
        for m in first_m..=last_m {
            let top = (m as f32).mul_add(WALL_PERIOD_Y, WALL_GAP * 0.5) + offset;
            // A faint guide so the lanes are legible before any bass plays.
            view.line(
                view.world(Vec2::new(x, top)),
                view.world(Vec2::new(x, top + segment)),
                1.0,
                with_alpha(palette.accent, 0.08),
            );
            if solidity <= 0.0 {
                continue;
            }
            // Rises from the middle outward as the note lands, then fades.
            let half = segment * 0.5 * solidity.sqrt();
            let colour = if sim::wall_is_solid(solidity) {
                with_alpha(palette.accent, solidity.mul_add(0.5, 0.35))
            } else {
                with_alpha(palette.accent, 0.25 * solidity)
            };
            let mid = segment.mul_add(0.5, top);
            view.rect(
                view.world(Vec2::new(x - WALL_HALF_W, mid - half)),
                Vec2::new(WALL_HALF_W * 2.0, half * 2.0),
                colour,
            );
            // The gate: two jaws closing across the gap below this segment
            // on every snare, while the wall stands.
            if !sim::wall_is_solid(solidity) || gates <= 0.0 {
                continue;
            }
            let gap_top = top + segment;
            let jaw = WALL_GAP * 0.5 * gates.sqrt();
            let jaw_colour = with_alpha(HEART, gates.mul_add(0.5, 0.4));
            view.rect(
                view.world(Vec2::new(x - WALL_HALF_W, gap_top)),
                Vec2::new(WALL_HALF_W * 2.0, jaw),
                jaw_colour,
            );
            view.rect(
                view.world(Vec2::new(x - WALL_HALF_W, gap_top + WALL_GAP - jaw)),
                Vec2::new(WALL_HALF_W * 2.0, jaw),
                jaw_colour,
            );
        }
    }
}

/// The bars between the walls, turning on the hats.
fn draw_spinners(sim: &Sim, view: &View, palette: &Palette) {
    let (first_k, last_k, first_m, last_m) = visible_grid(view);
    let along = Vec2::from_angle(sim.spin());
    let colour = if sim.holding() {
        with_alpha(HELD, 0.7)
    } else {
        with_alpha(palette.stomper, 0.85)
    };
    for k in first_k..=last_k {
        for m in first_m..=last_m {
            let Some(centre) = sim::spinner_at(k, m) else {
                continue;
            };
            let a = view.world(centre - along * SPINNER_HALF_LEN);
            let b = view.world(centre + along * SPINNER_HALF_LEN);
            view.line(a, b, SPINNER_HALF_W * 2.0, colour);
            view.circle(view.world(centre), SPINNER_HALF_W * 1.6, colour);
        }
    }
}

/// Concentric rings, breathing. The same glyph stands for home in the HUD.
fn home_glyph(view: &View, p: Vec2, radius: f32, clock: f32, color: Color) {
    let breath = (clock * 2.0).sin().mul_add(0.06, 1.0);
    view.ring(p, radius * breath, radius * 0.08, with_alpha(color, 0.9));
    view.ring(
        p,
        radius * 0.62 * breath,
        radius * 0.08,
        with_alpha(color, 0.7),
    );
    view.circle(p, radius * 0.22, color);
}

fn draw_home(view: &View, clock: f32) {
    let (top_left, size) = view.visible();
    let reach = HOME_RADIUS * 3.0;
    let on_screen = HOME.x > top_left.x - reach
        && HOME.x < top_left.x + size.x + reach
        && HOME.y > top_left.y - reach
        && HOME.y < top_left.y + size.y + reach;
    if !on_screen {
        return;
    }
    let p = view.world(HOME);
    view.circle(p, HOME_RADIUS * 2.0, with_alpha(HOME_GLOW, 0.06));
    home_glyph(view, p, HOME_RADIUS, clock, HOME_GLOW);
}

/// A chevron at the edge of the frame pointing the way home while home is
/// out of sight.
fn draw_way_home(sim: &Sim, view: &View, clock: f32) {
    if sim.phase() != Phase::Playing {
        return;
    }
    let (top_left, size) = view.visible();
    if HOME.x < top_left.x + size.x {
        return;
    }
    let player = sim.player().body.interpolated(sim.alpha());
    let dir = (HOME - player).normalized();
    let y = (player.y - top_left.y).clamp(60.0, VIEW_H - 90.0);
    let p = view.hud(Vec2::new(VIEW_W - MARGIN - 14.0, y));
    let reach = 10.0 * view.scale;
    let across = Vec2::new(-dir.y, dir.x);
    let colour = with_alpha(HOME_GLOW, pulse(clock).mul_add(0.5, 0.3));
    for i in 0..2 {
        let back = p - dir * (reach * 1.2 * i as f32);
        view.line(back - dir * reach + across * reach, back, 3.0, colour);
        view.line(back - dir * reach - across * reach, back, 3.0, colour);
    }
}

fn strike_colour(pitch: u8) -> Color {
    hsl_to_rgb(f32::from(pitch % 12) / 12.0, 0.8, 0.65)
}

/// A strike: a ring that closes on its target over a beat, then a flash.
fn draw_strikes(sim: &Sim, view: &View) {
    for strike in sim.strikes() {
        let p = view.world(strike.pos);
        let colour = if sim.holding() {
            HELD
        } else {
            strike_colour(strike.pitch)
        };
        if strike.bursting() {
            view.circle(p, STRIKE_RADIUS, with_alpha(colour, 0.85));
            view.ring(p, STRIKE_RADIUS * 1.3, 2.0, with_alpha(colour, 0.5));
            continue;
        }
        let closing = strike.closing();
        // The footprint it will burst over, filling as the fuse burns.
        view.circle(
            p,
            STRIKE_RADIUS,
            with_alpha(colour, closing.mul_add(0.25, 0.05)),
        );
        view.ring(p, STRIKE_RADIUS, 1.5, with_alpha(colour, 0.6));
        // And the ring closing in from outside.
        let radius = STRIKE_RADIUS * (1.0 - closing).mul_add(2.0, 1.0);
        view.ring(
            p,
            radius,
            2.0,
            with_alpha(colour, closing.mul_add(0.6, 0.3)),
        );
    }
}

fn draw_stompers(sim: &Sim, view: &View, alpha: f32, palette: &Palette) {
    let asleep = sim.motif() == Motif::Lullaby;
    let body_colour = if sim.holding() { HELD } else { palette.stomper };
    for stomper in sim.stompers() {
        let p = view.world(stomper.body.interpolated(alpha));
        let radius = STOMPER_RADIUS * stomper.pulse.mul_add(0.35, 1.0);
        match stomper.act {
            Act::WindingUp { aim, .. } => {
                // The line it will run down, growing bolder as the beat
                // runs out.
                let far = p + aim * (600.0 * view.scale);
                view.line(
                    p,
                    far,
                    3.0,
                    with_alpha(HEART, stomper.windup.mul_add(0.6, 0.2)),
                );
                view.ring(
                    p,
                    radius * stomper.windup.mul_add(1.2, 1.0),
                    2.0,
                    with_alpha(HEART, 0.8),
                );
            }
            Act::Dashing { aim, .. } => {
                // A streak behind it.
                view.line(
                    p,
                    p - aim * (60.0 * view.scale),
                    radius * 1.2,
                    with_alpha(body_colour, 0.35),
                );
            }
            Act::Swelling { .. } => {
                // Swelling up to the size of the wave it is about to make.
                let reach = stomper.windup * sim::SHOCK_RADIUS;
                view.ring(p, reach.max(radius), 2.0, with_alpha(HEART, 0.5));
                view.circle(p, reach.max(radius), with_alpha(HEART, 0.08));
            }
            Act::Shocking { .. } => {
                let ring = stomper.shock_radius();
                view.ring(p, ring, 14.0, with_alpha(HEART, 0.55));
            }
            Act::Idle => {}
        }
        view.circle(p, radius * 1.5, with_alpha(body_colour, 0.12));
        view.circle(p, radius, body_colour);
        if asleep {
            // Closed eyes: two short lines.
            let eye = Vec2::new(radius * 0.35 * view.scale, -radius * 0.15 * view.scale);
            let w = Vec2::new(radius * 0.18 * view.scale, 0.0);
            view.line(p - eye - w, p - eye + w, 2.0, with_alpha(PLAYER, 0.5));
            let eye = Vec2::new(-eye.x, eye.y);
            view.line(p - eye - w, p - eye + w, 2.0, with_alpha(PLAYER, 0.5));
        } else if sim.motif() == Motif::Wander && stomper.act == Act::Idle {
            // Show where the next kick will send it.
            let tip = p + stomper.heading * ((radius + 10.0) * view.scale);
            view.line(p, tip, 2.0, with_alpha(body_colour, 0.6));
        }
    }
}

fn draw_player(sim: &Sim, view: &View, alpha: f32) {
    let player = sim.player();
    let p = view.world(player.body.interpolated(alpha));
    // Blink through the grace period. The timer never goes negative, so
    // the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    let visible = player.invuln <= 0.0 || (player.invuln * 12.0) as u32 % 2 == 0;
    if visible {
        view.circle(p, PLAYER_RADIUS * 1.8, with_alpha(PLAYER, 0.1));
        view.circle(p, PLAYER_RADIUS, PLAYER);
    }
}

/// Top left: hearts.
fn draw_hearts(sim: &Sim, view: &View) {
    for i in 0..MAX_HEARTS {
        let p = view.hud(Vec2::new(
            (i as f32).mul_add(30.0, MARGIN + 12.0),
            MARGIN + 12.0,
        ));
        if i < sim.hearts() {
            view.circle(p, 9.0, HEART);
        } else {
            view.ring(p, 9.0, 1.5, with_alpha(HEART, 0.35));
        }
    }
}

/// A small speaker and its key, with a slash when muted.
fn draw_speaker(view: &View, audio: &Audio) {
    let p = view.hud(Vec2::new(MARGIN + 12.0, MARGIN + 44.0));
    let colour = if audio.needs_gesture() {
        HIGHLIGHT
    } else {
        DIM
    };
    let s = view.scale;
    view.rect(p - Vec2::new(8.0 * s, 4.0 * s), Vec2::new(5.0, 8.0), colour);
    View::triangle(
        p + Vec2::new(-4.0 * s, -4.0 * s),
        p + Vec2::new(2.0 * s, -9.0 * s),
        p + Vec2::new(2.0 * s, 9.0 * s),
        colour,
    );
    if audio.is_muted() {
        view.line(
            p + Vec2::new(-10.0 * s, 10.0 * s),
            p + Vec2::new(10.0 * s, -10.0 * s),
            2.0,
            HEART,
        );
    } else {
        view.arc(p, 7.0, 1.5, -45.0, 90.0, colour);
        view.arc(p, 11.0, 1.5, -45.0, 90.0, colour);
    }
    view.key_cap(
        Some('M'),
        p + Vec2::new(30.0 * s, 0.0),
        Vec2::new(18.0, 18.0),
        with_alpha(colour, 0.7),
    );
}

/// The motif's glyph: a wave for wander, an eye for pursuit, a crescent
/// for the lullaby.
fn motif_glyph(view: &View, motif: Motif, p: Vec2, size: f32, color: Color) {
    let s = view.scale;
    match motif {
        Motif::Wander => {
            let mut last = None;
            for i in 0..=12 {
                let t = i as f32 / 12.0;
                let at = p + Vec2::new(
                    (t - 0.5) * size * s,
                    (t * std::f32::consts::TAU).sin() * size * 0.25 * s,
                );
                if let Some(prev) = last {
                    view.line(prev, at, 2.5, color);
                }
                last = Some(at);
            }
        }
        Motif::Pursuit => {
            // An eye, open and looking: something is hunting.
            draw_ellipse_lines(
                p.x,
                p.y,
                size * 0.5 * s,
                size * 0.28 * s,
                0.0,
                2.5 * s,
                color,
            );
            view.circle(p, size * 0.12, color);
        }
        Motif::Lullaby => {
            view.arc(p, size * 0.35, size * 0.16, 60.0, 240.0, color);
        }
    }
}

/// The fermata sign: an arc over a dot. The pad's icon, and the action's.
fn fermata_glyph(view: &View, p: Vec2, size: f32, color: Color) {
    view.arc(
        p + Vec2::new(0.0, size * 0.2 * view.scale),
        size * 0.5,
        size * 0.12,
        180.0,
        180.0,
        color,
    );
    view.circle(
        p + Vec2::new(0.0, size * 0.1 * view.scale),
        size * 0.11,
        color,
    );
}

/// The icon for what an instrument drives: the thing itself.
fn instrument_icon(view: &View, instrument: Instrument, p: Vec2, palette: &Palette, color: Color) {
    let s = view.scale;
    match instrument {
        Instrument::Drums => {
            view.circle(p, 8.0, with_alpha(palette.stomper, color.a));
            view.line(p, p + Vec2::new(12.0 * s, 0.0), 2.0, color);
        }
        Instrument::Bass => view.rect(
            p - Vec2::new(4.0 * s, 10.0 * s),
            Vec2::new(8.0, 20.0),
            color,
        ),
        Instrument::Lead => {
            view.ring(p, 9.0, 1.5, with_alpha(strike_colour(64), color.a));
            view.circle(p, 3.0, with_alpha(strike_colour(64), color.a));
        }
        Instrument::Pad => fermata_glyph(view, p, 20.0, color),
    }
}

/// Top right: the motif, then one row per instrument — its icon, lit on
/// each note — then the fermata pool with its key.
fn draw_layers(sim: &Sim, view: &View, hud: &Hud, palette: &Palette, clock: f32) {
    let music = sim.music();
    let right = VIEW_W - MARGIN;
    motif_glyph(
        view,
        music.motif,
        view.hud(Vec2::new(right - 28.0, MARGIN + 14.0)),
        40.0,
        palette.accent,
    );

    for instrument in Instrument::ALL {
        let i = instrument.index();
        let layer: Layer = music.layers[i];
        let y = (i as f32).mul_add(30.0, MARGIN + 60.0);
        let icon = view.hud(Vec2::new(right - 18.0, y));
        let lit = hud.lit(instrument);
        let colour = if !layer.arranged {
            with_alpha(DIM, 0.35)
        } else if sim.holding() {
            with_alpha(HELD, 0.7)
        } else {
            with_alpha(palette.accent, lit.mul_add(0.4, 0.6))
        };
        if layer.arranged && !sim.holding() && lit > 0.0 {
            view.circle(
                icon,
                16.0 * lit.mul_add(0.3, 1.0),
                with_alpha(palette.accent, 0.15 * lit),
            );
        }
        instrument_icon(view, instrument, icon, palette, colour);
    }

    // The pool and its key, under the rows. The bar leads to the pad's
    // icon, which is what fills it.
    let bar_w = 60.0;
    let y = MARGIN + 186.0;
    let top = Vec2::new(right - bar_w, y - 4.0);
    let colour = if sim.pool_dry() {
        with_alpha(HEART, pulse(clock).mul_add(0.4, 0.4))
    } else if sim.holding() {
        HELD
    } else if sim.charging() {
        with_alpha(OVERLAY, pulse(clock).mul_add(0.2, 0.7))
    } else {
        with_alpha(OVERLAY, 0.6)
    };
    view.rect_lines(
        view.hud(top),
        Vec2::new(bar_w, 8.0),
        1.5,
        with_alpha(colour, 0.6),
    );
    view.rect(
        view.hud(top + Vec2::new(1.5, 1.5)),
        Vec2::new((bar_w - 3.0) * sim.pool(), 5.0),
        colour,
    );
    // A wide, empty cap: the space bar.
    let cap_colour = if sim.holding() { HELD } else { colour };
    view.key_cap(
        None,
        view.hud(Vec2::new(right - bar_w - 44.0, y)),
        Vec2::new(64.0, 16.0),
        cap_colour,
    );
}

/// Bottom: the song's sections and the current bar's steps.
fn draw_transport(sim: &Sim, view: &View, palette: &Palette) {
    let music = sim.music();
    let strip_y = VIEW_H - MARGIN - 44.0;
    let accent = if sim.holding() { HELD } else { palette.accent };

    // Sections: a block each in its motif's colour, the live one filling
    // left to right as it plays out.
    let gap = 4.0;
    let block_w = (STRIP_W - gap * (SONG.len() as f32 - 1.0)) / SONG.len() as f32;
    for (i, section) in SONG.iter().enumerate() {
        let x = (i as f32).mul_add(block_w + gap, MARGIN);
        let live = i == music.section;
        let colour = palette_for(section.motif).accent;
        if live {
            let progress = (music.bar as f32
                + (music.step as f32 + music.step_phase) / STEPS_PER_BAR as f32)
                / BEATS_PER_BAR as f32;
            view.rect(
                view.hud(Vec2::new(x, strip_y - 16.0)),
                Vec2::new(block_w * progress, 8.0),
                with_alpha(colour, 0.7),
            );
        }
        view.rect(
            view.hud(Vec2::new(x, strip_y - 16.0)),
            Vec2::new(block_w, 8.0),
            with_alpha(colour, if live { 0.25 } else { 0.12 }),
        );
        motif_glyph(
            view,
            section.motif,
            view.hud(Vec2::new(x + block_w * 0.5, strip_y - 28.0)),
            16.0,
            with_alpha(colour, if live { 0.9 } else { 0.4 }),
        );
    }

    // Steps: sixteen ticks, beats taller, the live one lit.
    let step_w = STRIP_W / STEPS_PER_BAR as f32;
    for step in 0..STEPS_PER_BAR {
        let x = (step as f32).mul_add(step_w, MARGIN);
        let on_beat = step % (STEPS_PER_BAR / BEATS_PER_BAR) == 0;
        let height = if on_beat { 8.0 } else { 5.0 };
        let live = sim.phase() == Phase::Playing && step == music.step;
        let colour = if live {
            accent
        } else {
            with_alpha(accent, if on_beat { 0.35 } else { 0.18 })
        };
        view.rect(
            view.hud(Vec2::new(x + 1.0, strip_y + 8.0 - height)),
            Vec2::new(step_w - 2.0, height),
            colour,
        );
    }
}

/// Bottom-most: the way home as a line, the player a dot along it, home
/// its glyph at the end.
fn draw_journey(sim: &Sim, view: &View, clock: f32) {
    let y = VIEW_H - MARGIN - 8.0;
    let from = Vec2::new(MARGIN + 10.0, y);
    let to = Vec2::new(VIEW_W - MARGIN - 20.0, y);
    view.line(view.hud(from), view.hud(to), 1.5, with_alpha(OVERLAY, 0.3));
    home_glyph(view, view.hud(to), 8.0, clock, HOME_GLOW);
    let at = from.lerp(to, sim.progress());
    view.circle(view.hud(at), 5.0, PLAYER);
}

/// The goal and the controls, as pictures: you, the way, home; the arrows
/// to move, the space bar to hold the music.
fn draw_title(view: &View, clock: f32) {
    view.rect(view.hud(Vec2::ZERO), Vec2::new(VIEW_W, VIEW_H), SHADE);
    let cy = VIEW_H * 0.4;
    let you = view.hud(Vec2::new(VIEW_W * 0.25, cy));
    let home = view.hud(Vec2::new(VIEW_W * 0.75, cy));
    view.circle(you, PLAYER_RADIUS * 1.8, with_alpha(PLAYER, 0.1));
    view.circle(you, PLAYER_RADIUS, PLAYER);
    for i in 1..12 {
        let t = i as f32 / 12.0;
        let p = you.lerp(home, t);
        let glow = (1.0 - ((t * 12.0 - clock * 3.0) % 12.0).abs() / 3.0).max(0.0);
        view.circle(p, 2.5, with_alpha(HOME_GLOW, glow.mul_add(0.6, 0.25)));
    }
    home_glyph(view, home, 34.0, clock, HOME_GLOW);

    let glow = with_alpha(HIGHLIGHT, pulse(clock).mul_add(0.5, 0.5));
    let base = Vec2::new(VIEW_W * 0.38, VIEW_H * 0.72);
    let step = 34.0;
    view.arrow_cap(
        Vec2::new(0.0, -1.0),
        view.hud(base - Vec2::new(0.0, step)),
        28.0,
        glow,
    );
    view.arrow_cap(
        Vec2::new(-1.0, 0.0),
        view.hud(base - Vec2::new(step, 0.0)),
        28.0,
        glow,
    );
    view.arrow_cap(Vec2::new(0.0, 1.0), view.hud(base), 28.0, glow);
    view.arrow_cap(
        Vec2::new(1.0, 0.0),
        view.hud(base + Vec2::new(step, 0.0)),
        28.0,
        glow,
    );

    // The space bar, with the fermata it performs above it.
    let space = view.hud(Vec2::new(VIEW_W * 0.66, VIEW_H * 0.72));
    let soft = with_alpha(HIGHLIGHT, pulse(clock + 1.0).mul_add(0.5, 0.5));
    view.key_cap(None, space, Vec2::new(120.0, 26.0), soft);
    fermata_glyph(view, space - Vec2::new(0.0, 40.0 * view.scale), 32.0, soft);
}

/// A circular arrow: start over.
fn restart_glyph(view: &View, p: Vec2, radius: f32, color: Color) {
    view.arc(p, radius, radius * 0.18, -60.0, 300.0, color);
    let s = view.scale;
    let tip = p + Vec2::new(radius * 0.5 * s, -radius * 0.87 * s);
    View::triangle(
        tip + Vec2::new(radius * 0.45 * s, 0.0),
        tip + Vec2::new(-radius * 0.1 * s, -radius * 0.4 * s),
        tip + Vec2::new(-radius * 0.1 * s, radius * 0.4 * s),
        color,
    );
}

fn draw_over(view: &View, clock: f32) {
    view.rect(view.hud(Vec2::ZERO), Vec2::new(VIEW_W, VIEW_H), SHADE);
    let centre = view.hud(Vec2::new(VIEW_W * 0.5, VIEW_H * 0.42));
    for i in 0..MAX_HEARTS {
        let p = centre + Vec2::new((i as f32 - 1.0) * 40.0 * view.scale, 0.0);
        view.ring(p, 13.0, 2.0, with_alpha(HEART, 0.4));
    }
    draw_restart_prompt(view, clock);
}

fn draw_won(view: &View, clock: f32) {
    view.rect(view.hud(Vec2::ZERO), Vec2::new(VIEW_W, VIEW_H), SHADE);
    let centre = view.hud(Vec2::new(VIEW_W * 0.5, VIEW_H * 0.4));
    view.circle(
        centre,
        110.0,
        with_alpha(HOME_GLOW, pulse(clock).mul_add(0.06, 0.06)),
    );
    home_glyph(view, centre, 60.0, clock, HOME_GLOW);
    view.circle(centre, PLAYER_RADIUS, PLAYER);
    draw_restart_prompt(view, clock);
}

fn draw_restart_prompt(view: &View, clock: f32) {
    let glow = with_alpha(HIGHLIGHT, pulse(clock).mul_add(0.5, 0.5));
    let y = VIEW_H * 0.7;
    restart_glyph(
        view,
        view.hud(Vec2::new(VIEW_W.mul_add(0.5, -30.0), y)),
        16.0,
        glow,
    );
    view.key_cap(
        Some('R'),
        view.hud(Vec2::new(VIEW_W.mul_add(0.5, 30.0), y)),
        Vec2::new(32.0, 32.0),
        glow,
    );
}

fn draw_paused(view: &View) {
    let centre = view.hud(Vec2::new(VIEW_W * 0.5, VIEW_H * 0.5));
    let s = view.scale;
    view.rect(
        centre + Vec2::new(-16.0 * s, -20.0 * s),
        Vec2::new(10.0, 40.0),
        OVERLAY,
    );
    view.rect(
        centre + Vec2::new(6.0 * s, -20.0 * s),
        Vec2::new(10.0, 40.0),
        OVERLAY,
    );
}
