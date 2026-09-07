//! macroquad frontend: input in, shapes and sound out.
//!
//! Everything that decides what happens lives in [`leitmotif::sim`]. This
//! file only translates between the window and the sim's logical world; the
//! sim's [`Cue`](leitmotif::sim::Cue)s become sound over in [`audio`].

mod audio;

use audio::Audio;
use macroquad::color::{Color, hsl_to_rgb};
use macroquad::input::{
    KeyCode, MouseButton, get_last_key_pressed, is_key_down, is_key_pressed,
    is_mouse_button_pressed,
};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle};
use macroquad::text::{draw_text, measure_text};
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, clear_background, next_frame, screen_height, screen_width};

use leitmotif::VERSION;
use leitmotif::sim::{
    InputFrame, MAGNET_RADIUS, PLAYER_RADIUS, Phase, SPARK_RADIUS, STOMPER_RADIUS, Sim, Vec2,
    WALL_BOTTOM, WALL_HALF_W, WALL_TOP, WORLD_H, WORLD_W,
};
use leitmotif::song::{BEATS_PER_BAR, Instrument, Layer, Motif, SONG, STEPS_PER_BAR};

/// HUD text, in world units.
const TEXT_SIZE: f32 = 18.0;
const TITLE_SIZE: f32 = 44.0;
const MARGIN: f32 = 12.0;

/// Width of the HUD strips at the bottom, inside the margins.
const HUD_WIDTH: f32 = WORLD_W - 2.0 * MARGIN;

const OVERLAY: Color = Color::new(0.85, 0.85, 0.90, 0.75);
const DIM: Color = Color::new(0.55, 0.55, 0.62, 0.6);
/// For the one HUD line that is asking for something rather than reporting.
const HIGHLIGHT: Color = Color::new(1.0, 0.83, 0.42, 0.95);
const PLAYER: Color = Color::new(0.95, 0.95, 1.0, 1.0);
const SHADE: Color = Color::new(0.0, 0.0, 0.0, 0.6);

fn window_conf() -> Conf {
    Conf {
        window_title: env!("CARGO_PKG_NAME").to_owned(),
        window_width: WORLD_W as i32,
        window_height: WORLD_H as i32,
        high_dpi: true,
        ..Conf::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sim = Sim::new(fresh_seed());
    let mut audio = Audio::load().await;

    loop {
        let view = View::fit(screen_width(), screen_height());
        let input = gather_input();
        let toggle_mute = is_key_pressed(KeyCode::M);
        let dt = get_frame_time();

        sim.advance(dt, &input);
        audio.update(sim.cues(), &input, toggle_mute);
        draw(&sim, &view, &audio);
        next_frame().await;
    }
}

/// Wall-clock seed. Swap in `quad-url` here if you want seeds that survive in
/// a shareable URL fragment (see the docs).
fn fresh_seed() -> u64 {
    macroquad::miniquad::date::now().to_bits()
}

/// Letterboxed mapping between the sim's fixed logical world and the window.
/// The HUD is laid out in world units too, so it scales with the arena.
struct View {
    scale: f32,
    origin: Vec2,
}

impl View {
    fn fit(screen_w: f32, screen_h: f32) -> Self {
        // A minimised window or hidden tab reports zero size; the floor keeps
        // the mapping from producing infinities.
        let scale = (screen_w / WORLD_W)
            .min(screen_h / WORLD_H)
            .max(f32::EPSILON);
        Self {
            scale,
            origin: Vec2::new(
                WORLD_W.mul_add(-scale, screen_w) * 0.5,
                WORLD_H.mul_add(-scale, screen_h) * 0.5,
            ),
        }
    }

    fn to_screen(&self, world: Vec2) -> Vec2 {
        world * self.scale + self.origin
    }

    fn circle(&self, at: Vec2, radius: f32, color: Color) {
        let p = self.to_screen(at);
        draw_circle(p.x, p.y, radius * self.scale, color);
    }

    fn ring(&self, at: Vec2, radius: f32, thickness: f32, color: Color) {
        let p = self.to_screen(at);
        draw_circle_lines(p.x, p.y, radius * self.scale, thickness * self.scale, color);
    }

    fn rect(&self, at: Vec2, size: Vec2, color: Color) {
        let p = self.to_screen(at);
        draw_rectangle(p.x, p.y, size.x * self.scale, size.y * self.scale, color);
    }

    fn line(&self, from: Vec2, to: Vec2, thickness: f32, color: Color) {
        let a = self.to_screen(from);
        let b = self.to_screen(to);
        draw_line(a.x, a.y, b.x, b.y, thickness * self.scale, color);
    }

    /// Font size in pixels for a world-unit text size.
    // Sizes are positive constants, so the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn px(&self, size: f32) -> u16 {
        (size * self.scale).max(1.0) as u16
    }

    /// World-unit width of `text` at `size`.
    fn text_width(&self, text: &str, size: f32) -> f32 {
        measure_text(text, None, self.px(size), 1.0).width / self.scale
    }

    /// Draw text with its baseline at `at`, returning its world width.
    fn text(&self, text: &str, at: Vec2, size: f32, color: Color) -> f32 {
        let p = self.to_screen(at);
        draw_text(text, p.x, p.y, f32::from(self.px(size)), color);
        self.text_width(text, size)
    }

    /// Draw text with its right edge at `right`.
    fn text_right(&self, text: &str, right: f32, baseline: f32, size: f32, color: Color) {
        let width = self.text_width(text, size);
        self.text(text, Vec2::new(right - width, baseline), size, color);
    }

    /// Draw text centred on `x`.
    fn text_centered(&self, text: &str, x: f32, baseline: f32, size: f32, color: Color) {
        let width = self.text_width(text, size);
        self.text(
            text,
            Vec2::new(width.mul_add(-0.5, x), baseline),
            size,
            color,
        );
    }
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
        toggle_layer: [
            is_key_pressed(KeyCode::Key1),
            is_key_pressed(KeyCode::Key2),
            is_key_pressed(KeyCode::Key3),
            is_key_pressed(KeyCode::Key4),
        ],
        next_section: is_key_pressed(KeyCode::Tab),
        toggle_pause: is_key_pressed(KeyCode::Space),
        restart: is_key_pressed(KeyCode::R).then(fresh_seed),
    }
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

fn draw(sim: &Sim, view: &View, audio: &Audio) {
    let music = sim.music();
    let palette = palette_for(music.motif);
    let alpha = sim.alpha();

    // The arena breathes on the beat while something is playing.
    let pulse = if sim.phase() == Phase::Playing && !sim.is_paused() {
        (1.0 - music.beat_phase()).powi(3) * 0.05
    } else {
        0.0
    };
    let bg = palette.background;
    clear_background(Color::new(0.0, 0.0, 0.0, 1.0));
    view.rect(
        Vec2::ZERO,
        Vec2::new(WORLD_W, WORLD_H),
        Color::new(bg.r + pulse, bg.g + pulse, bg.b + pulse, 1.0),
    );

    draw_walls(sim, view, &palette);
    draw_sparks(sim, view, alpha);
    draw_stompers(sim, view, alpha, &palette);
    draw_player(sim, view, alpha);
    draw_hud(sim, view, audio, &palette);

    match sim.phase() {
        Phase::Title => draw_title(view, &palette),
        Phase::Over => draw_over(sim, view, &palette),
        Phase::Playing => {}
    }
}

fn draw_walls(sim: &Sim, view: &View, palette: &Palette) {
    let band = WALL_BOTTOM - WALL_TOP;
    for wall in sim.walls() {
        // A faint guide so the lanes are legible before any bass plays.
        view.line(
            Vec2::new(wall.x, WALL_TOP),
            Vec2::new(wall.x, WALL_BOTTOM),
            1.0,
            with_alpha(palette.accent, 0.08),
        );
        if wall.solidity <= 0.0 {
            continue;
        }
        // Rises from the middle outward as the note lands, then fades.
        let half = band * 0.5 * wall.solidity.sqrt();
        let colour = if wall.is_solid() {
            with_alpha(palette.accent, wall.solidity.mul_add(0.5, 0.35))
        } else {
            with_alpha(palette.accent, 0.25 * wall.solidity)
        };
        let mid = band.mul_add(0.5, WALL_TOP);
        view.rect(
            Vec2::new(wall.x - WALL_HALF_W, mid - half),
            Vec2::new(WALL_HALF_W * 2.0, half * 2.0),
            colour,
        );
    }
}

fn draw_sparks(sim: &Sim, view: &View, alpha: f32) {
    for spark in sim.sparks() {
        let pos = spark.body.interpolated(alpha);
        let hue = f32::from(spark.pitch % 12) / 12.0;
        let fade = spark.fade();
        let colour = hsl_to_rgb(hue, 0.8, 0.65);
        view.circle(pos, SPARK_RADIUS * 2.2, with_alpha(colour, 0.12 * fade));
        view.circle(
            pos,
            SPARK_RADIUS,
            with_alpha(colour, fade.mul_add(0.65, 0.35)),
        );
    }
}

fn draw_stompers(sim: &Sim, view: &View, alpha: f32, palette: &Palette) {
    let asleep = sim.motif() == Motif::Lullaby;
    for stomper in sim.stompers() {
        let pos = stomper.body.interpolated(alpha);
        let radius = STOMPER_RADIUS * stomper.pulse.mul_add(0.35, 1.0);
        view.circle(pos, radius * 1.5, with_alpha(palette.stomper, 0.12));
        view.circle(pos, radius, palette.stomper);
        if asleep {
            view.text(
                "z",
                Vec2::new(pos.x + radius * 0.6, pos.y - radius * 0.6),
                TEXT_SIZE,
                OVERLAY,
            );
        } else if sim.motif() == Motif::Wander {
            // Show where the next kick will send it.
            let tip = pos + stomper.heading * (radius + 10.0);
            view.line(pos, tip, 2.0, with_alpha(palette.stomper, 0.6));
        }
    }
}

fn draw_player(sim: &Sim, view: &View, alpha: f32) {
    let player = sim.player();
    let pos = player.body.interpolated(alpha);
    if sim.magnet_on() {
        view.ring(pos, MAGNET_RADIUS, 1.0, with_alpha(PLAYER, 0.08));
    }
    // Blink through the grace period. The timer never goes negative, so
    // the cast cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    let visible = player.invuln <= 0.0 || (player.invuln * 12.0) as u32 % 2 == 0;
    if visible {
        view.circle(pos, PLAYER_RADIUS * 1.8, with_alpha(PLAYER, 0.1));
        view.circle(pos, PLAYER_RADIUS, PLAYER);
    }
}

fn draw_hud(sim: &Sim, view: &View, audio: &Audio, palette: &Palette) {
    let music = sim.music();
    let line = |row: f32| TEXT_SIZE.mul_add(row, MARGIN + TEXT_SIZE);

    // Top left: who we are and how we are doing.
    view.text(
        &format!("{} {VERSION}", env!("CARGO_PKG_NAME")),
        Vec2::new(MARGIN, line(0.0)),
        TEXT_SIZE,
        DIM,
    );
    view.text(
        &format!("score {}   x{} per spark", sim.score(), sim.spark_value()),
        Vec2::new(MARGIN, line(1.0)),
        TEXT_SIZE,
        OVERLAY,
    );
    let lives_x = view.text("lives ", Vec2::new(MARGIN, line(2.0)), TEXT_SIZE, OVERLAY);
    for i in 0..sim.lives() {
        let x = (i as f32).mul_add(16.0, MARGIN + lives_x + 8.0);
        view.circle(Vec2::new(x, line(2.0) - 6.0), 5.0, PLAYER);
    }
    view.text(
        &format!("audio: {}", audio.status()),
        Vec2::new(MARGIN, line(3.0)),
        TEXT_SIZE,
        if audio.needs_gesture() {
            HIGHLIGHT
        } else {
            DIM
        },
    );

    // Top right: the music.
    let right = WORLD_W - MARGIN;
    let motif = music.motif.name().to_uppercase();
    view.text_right(&motif, right, line(0.6), TITLE_SIZE * 0.7, palette.accent);
    let detail = format!(
        "{:.0} bpm   section {}/{}   bar {}.{}",
        music.motif.bpm(),
        music.section + 1,
        SONG.len(),
        music.bar + 1,
        music.beat + 1
    );
    view.text_right(&detail, right, line(2.0), TEXT_SIZE, OVERLAY);

    // Layers, one per row, with what each does.
    for instrument in Instrument::ALL {
        let i = instrument.index();
        let layer = music.layers[i];
        let (state, colour) = layer_label(layer, palette);
        let label = format!(
            "[{}] {:<5} {:<6} {}",
            i + 1,
            instrument.name(),
            state,
            layer_role(instrument)
        );
        view.text_right(&label, right, line(3.5 + i as f32), TEXT_SIZE, colour);
    }

    draw_transport(sim, view, palette);

    view.text(
        "arrows/WASD move | 1-4 mute layers | tab next section | space pause | R restart | M mute",
        Vec2::new(MARGIN, WORLD_H - MARGIN - 22.0),
        TEXT_SIZE * 0.8,
        DIM,
    );
}

/// What a layer's row says, and in what colour.
const fn layer_label(layer: Layer, palette: &Palette) -> (&'static str, Color) {
    match (layer.arranged, layer.muted, layer.pending) {
        (_, _, true) => ("...", HIGHLIGHT),
        (false, _, _) => ("tacet", DIM),
        (true, true, _) => ("muted", DIM),
        (true, false, _) => ("on", palette.accent),
    }
}

/// The one-line rule each instrument enforces, so the HUD teaches the game.
const fn layer_role(instrument: Instrument) -> &'static str {
    match instrument {
        Instrument::Drums => "kicks move stompers",
        Instrument::Bass => "notes raise walls",
        Instrument::Lead => "notes drop sparks",
        Instrument::Pad => "pulls sparks in",
    }
}

/// Bottom of the screen: the song's sections and the current bar's steps.
fn draw_transport(sim: &Sim, view: &View, palette: &Palette) {
    let music = sim.music();
    let strip_y = WORLD_H - MARGIN - 40.0;
    let width = HUD_WIDTH;

    // Sections: a block each, motif initial inside, the live one lit.
    let gap = 4.0;
    let block_w = (width - gap * (SONG.len() as f32 - 1.0)) / SONG.len() as f32;
    for (i, section) in SONG.iter().enumerate() {
        let x = (i as f32).mul_add(block_w + gap, MARGIN);
        let live = i == music.section;
        let colour = palette_for(section.motif).accent;
        let fill = if live {
            // Fill the block left to right as the section plays out.
            let progress = (music.bar as f32
                + (music.step as f32 + music.step_phase) / STEPS_PER_BAR as f32)
                / BEATS_PER_BAR as f32;
            view.rect(
                Vec2::new(x, strip_y - 16.0),
                Vec2::new(block_w * progress, 10.0),
                with_alpha(colour, 0.7),
            );
            with_alpha(colour, 0.25)
        } else {
            with_alpha(colour, 0.12)
        };
        view.rect(Vec2::new(x, strip_y - 16.0), Vec2::new(block_w, 10.0), fill);
        let initial = &section.motif.name()[..1].to_uppercase();
        view.text(
            initial,
            Vec2::new(x + 2.0, strip_y - 20.0),
            TEXT_SIZE * 0.7,
            with_alpha(colour, if live { 1.0 } else { 0.5 }),
        );
    }

    // Steps: sixteen ticks, beats taller, the live one lit.
    let step_w = width / STEPS_PER_BAR as f32;
    for step in 0..STEPS_PER_BAR {
        let x = (step as f32).mul_add(step_w, MARGIN);
        let on_beat = step % (STEPS_PER_BAR / BEATS_PER_BAR) == 0;
        let height = if on_beat { 10.0 } else { 6.0 };
        let live = sim.phase() == Phase::Playing && step == music.step;
        let colour = if live {
            palette.accent
        } else {
            with_alpha(palette.accent, if on_beat { 0.35 } else { 0.18 })
        };
        view.rect(
            Vec2::new(x + 1.0, strip_y + 10.0 - height),
            Vec2::new(step_w - 2.0, height),
            colour,
        );
    }
}

fn draw_title(view: &View, palette: &Palette) {
    view.rect(Vec2::ZERO, Vec2::new(WORLD_W, WORLD_H), SHADE);
    let cx = WORLD_W * 0.5;
    view.text_centered("LEITMOTIF", cx, 190.0, TITLE_SIZE, palette.accent);
    let lines = [
        "The music makes the rules.",
        "",
        "Which motif is playing sets how the arena behaves:",
        "WANDER drifts, PURSUIT hunts you, LULLABY sleeps.",
        "Each instrument moves one thing, only while it sounds:",
        "kicks step the stompers, bass raises walls,",
        "the lead drops sparks, the pad draws them in.",
        "",
        "Collect sparks. Each is worth the number of instruments",
        "playing, times the motif. Mute layers with 1-4 to trade",
        "danger for points. Avoid the stompers. Three hits and",
        "the music stops.",
    ];
    for (i, text) in lines.iter().enumerate() {
        view.text_centered(
            text,
            cx,
            (i as f32).mul_add(22.0, 235.0),
            TEXT_SIZE,
            OVERLAY,
        );
    }
    view.text_centered("press any key", cx, 520.0, TEXT_SIZE * 1.2, HIGHLIGHT);
}

fn draw_over(sim: &Sim, view: &View, palette: &Palette) {
    view.rect(Vec2::ZERO, Vec2::new(WORLD_W, WORLD_H), SHADE);
    let cx = WORLD_W * 0.5;
    view.text_centered("the music stopped", cx, 260.0, TITLE_SIZE, palette.accent);
    view.text_centered(
        &format!("score {}", sim.score()),
        cx,
        310.0,
        TEXT_SIZE * 1.4,
        OVERLAY,
    );
    view.text_centered("R to start over", cx, 360.0, TEXT_SIZE * 1.2, HIGHLIGHT);
}
