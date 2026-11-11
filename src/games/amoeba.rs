//! Amoeba Roguelike Remastered's macroquad frontend: a font atlas, and one
//! command a turn.
//!
//! Everything that decides what happens lives in [`game_prototypes::amoeba::sim`], which
//! resolves the world all the way down to a glyph and two colours per cell
//! before this file sees it. What is left is genuinely only presentation:
//! where the panels go ([`render`]), what goes in them ([`hud`]), which pixels
//! a character is ([`tileset`]), which key or thumb meant which [`Command`]
//! ([`input`]), and what the player has asked for ([`settings`]).
//!
//! The loop ([`Game::update`]) is deliberately plain. The game is turn-based, so there is no
//! timestep and nothing to interpolate: read the world, draw it, hand the sim
//! at most one command, repeat. No clock reaches the sim — the blink counter
//! below is passed in as an argument, keeping [`Sim::view`] the pure function
//! of world and frame that it claims to be.
//!
//! The sim reports what happened twice, and the frontend's two other halves
//! each take one of those reports. [`audio`] turns
//! [`Cue`](game_prototypes::amoeba::sim::Cue)s — a deduplicated set — into noise, and
//! [`anim`] turns [`Effect`](game_prototypes::amoeba::sim::effect::Effect)s — the same news
//! in order and with cells attached — into overlays on the map. Those two and
//! the frame clock are the only places allowed to read a wall clock.
//!
//! ## Holding a command back
//!
//! An animation is the one thing in here that takes time, so it is the one
//! thing that can get between a keypress and a turn. The rule is that it never
//! does: while a turn is showing itself the sim is fed nothing, but any command
//! asked for in the meantime *cuts the animation short* and is kept in
//! `queued` for the very next frame. An idle player watches the whole thing; a
//! player in a hurry loses a frame and no keypress.

pub mod anim;
pub mod audio;
pub mod hud;
pub mod input;
pub mod render;
pub mod settings;
pub mod tileset;

use macroquad::color::Color;
use macroquad::math::{Rect, Vec2};
use macroquad::time::get_time;
use macroquad::window::{screen_height, screen_width};

use game_prototypes::amoeba::sim::grid::Coord;
use game_prototypes::amoeba::sim::{
    Command, Difficulty, InputStyle, Phase, RenderView, Sim, UiMode,
};

use anim::Anim;
use audio::Audio;
use input::{Controls, Input};
use render::{Frontend, Layout};
use settings::Settings;
use tileset::Tileset;

use crate::ui::{Frame, MqVec2, with_alpha};

/// Seconds per animation step, as the original's `ANIMATION_RATE` was.
const ANIMATION_RATE: f64 = 0.25;

/// Everything the old standalone loop kept between frames.
///
/// The one convention it does not share with the other prototypes: it draws
/// across the whole window rather than inside the shell's letterboxed
/// [`Frame`]. The layout was built to fit anything from a portrait phone to a
/// desktop, with an on-screen pad when a thumb turns up, and an 800x600 box
/// would undo all of that. The shell still owns Escape whenever
/// [`Game::wants_escape`](crate::games::Game::wants_escape) says the game has
/// nothing open for it to close.
pub struct Game {
    font: Tileset,
    sim: Sim,
    input: Input,
    audio: Audio,
    settings: Settings,
    anim: Anim,
    /// The one command held back while a turn is showing itself.
    queued: Option<Command>,
}

/// Where everything goes this frame: worked out once for input and again for
/// drawing, since both are pure functions of the window and the world.
struct Arranged {
    view: RenderView,
    layout: Layout,
    camera: Coord,
    controls: Controls,
}

impl Game {
    pub async fn load() -> Self {
        Self {
            font: Tileset::load(),
            sim: Sim::new(input::fresh_seed(), Difficulty::Normal),
            input: Input::new(),
            audio: Audio::load().await,
            settings: Settings::new(),
            anim: Anim::new(),
            queued: None,
        }
    }

    /// Three passes over the same window, because the pad and the panels
    /// each need to know something about the other: which layout is on
    /// screen decides whether there is a pad, the pad decides how much of
    /// the window is left, and what is left decides where the panels go.
    /// The middle step only ever takes room away, and taking room away can
    /// only turn a wide layout narrow — never the other way about — so the
    /// first pass's answer to "is there a pad?" survives the third.
    fn arrange(&self) -> Arranged {
        let (w, h) = (screen_width(), screen_height());
        let view = self.sim.view(anim_frame());
        let probe = Layout::fit(
            Rect::new(0.0, 0.0, w, h),
            view.width,
            view.height,
            self.settings.log,
            false,
        );
        let touch = self.input.wants_controls(probe.mode);
        let area = if touch {
            Controls::free(w, h)
        } else {
            Rect::new(0.0, 0.0, w, h)
        };
        let layout = Layout::fit(area, view.width, view.height, self.settings.log, touch);
        let camera = if layout.scrolls {
            render::camera_origin(
                focus(&view),
                layout.cols,
                layout.rows,
                view.width,
                view.height,
            )
        } else {
            Coord::new(0, 0)
        };
        // The pad steers the amoeba, so it only belongs on screen while there
        // is an amoeba: the title and post-mortem screens have buttons of
        // their own, and the settings panel is a modal with its own way out.
        // The room it claimed stays claimed through all of them, so the map
        // does not jump about underneath a panel that is about to close.
        let controls = Controls::fit(
            w,
            h,
            touch && view.phase == Phase::Playing && !self.settings.is_open(),
        );
        Arranged {
            view,
            layout,
            camera,
            controls,
        }
    }
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        let Arranged {
            view,
            layout,
            camera,
            controls,
        } = self.arrange();
        let frame = self
            .input
            .gather(&view, &layout, camera, &controls, &mut self.settings, dt);

        let command = if self.anim.playing() {
            // Somebody in a hurry is somebody who has stopped watching, so a
            // command arriving now ends the show and waits one frame. A quiet
            // frame leaves whatever was already waiting alone.
            if frame.command.is_some() {
                self.anim.skip();
                self.queued = frame.command;
            }
            None
        } else {
            // The newest ask wins, and the older one is dropped rather than
            // kept: it would arrive after the thing that superseded it.
            let waiting = self.queued.take();
            frame.command.or(waiting)
        };
        self.anim.tick(dt);

        // The sim writes a few lines that name controls; this is the only
        // thing it needs from out here to name them the way the buttons do.
        // After `gather` rather than before, because the very first tap a
        // touchscreen ever gets is the one that picks a difficulty — and that
        // is the command that makes the sim write the help.
        self.sim
            .set_input_style(if self.input.wants_controls(layout.mode) {
                InputStyle::Touch
            } else {
                InputStyle::Keys
            });
        self.sim.advance(command);
        // Both reports live for exactly this one `advance`, so both halves read
        // them here — the cues alongside the mute toggle from the same frame,
        // and the effects for however long they take to draw. A frame that
        // resolved nothing hands over two empty lists and disturbs neither.
        self.audio.update(self.sim.cues(), &frame);
        self.anim.load(self.sim.effects(), &self.settings);
    }

    /// The whole window, not `_frame`: see [`Game`].
    ///
    /// Drawn after the turn `update` resolved, where the standalone loop drew
    /// the world as it stood before it. The difference is one frame, and it is
    /// in the right direction: the animation that turn loaded starts on the
    /// map it belongs to.
    fn draw(&self, _frame: &Frame) {
        let Arranged {
            view,
            layout,
            camera,
            controls,
        } = self.arrange();
        render::draw(
            &self.font,
            &view,
            &layout,
            camera,
            &Frontend {
                controls: &controls,
                page: self.input.page(),
                audio: self.audio.hint(layout.touch),
                sound_on: !self.audio.muted(),
                settings: &self.settings,
                anim: &self.anim,
                pulse: telegraph_phase(),
            },
        );
    }

    /// Escape is the game's own back key — it puts the examine cursor or the
    /// organelle list away and closes the settings panel — so while any of
    /// those is out, a press is the game's. Only with nothing left to close
    /// does it go to the shell and leave for the menu.
    fn wants_escape(&self) -> bool {
        self.settings.is_open()
            || (self.sim.phase() == Phase::Playing && self.sim.mode() != UiMode::Messages)
    }

    /// The map runs up into the top-left corner, and the side panel's
    /// top-right corner is the one its left-aligned text never reaches. In
    /// frame units like any other spot: the shell only asks when the window
    /// is close enough to 4:3 that the frame is the window.
    fn back_spot(&self) -> MqVec2 {
        MqVec2::new(crate::ui::FRAME_W - 30.0, 30.0)
    }
}

/// The menu's picture of the game: a blob of slime cells, breathing, with a
/// nucleus in it, pressed up against a city gate. Drawn in vector squares
/// rather than the font atlas, since the menu draws this without loading
/// anything.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    const SLIME: Color = Color::new(0.427, 0.667, 0.173, 1.0);
    const BRIGHT_SLIME: Color = Color::new(0.518, 0.745, 0.220, 1.0);
    const WALL: Color = Color::new(0.282, 0.302, 0.333, 1.0);
    const NUCLEUS: Color = Color::new(0.816, 0.275, 0.282, 1.0);
    const COLS: i32 = 11;
    const ROWS: i32 = 7;

    let s = frame.scale();
    let cell = width / COLS as f32;
    let at = |col: i32, row: i32| {
        centre
            + MqVec2::new(
                ((COLS - 1) as f32).mul_add(-0.5, col as f32) * cell * s,
                ((ROWS - 1) as f32).mul_add(-0.5, row as f32) * cell * s,
            )
    };
    let size = MqVec2::splat(cell * 0.86);

    // The gate: a column of wall at the right edge, a gap in the middle.
    for row in 0..ROWS {
        if row != ROWS / 2 {
            frame.rect_centred(at(COLS - 1, row), size, WALL);
        }
    }
    // The amoeba: every cell inside a wobbling radius of its middle.
    let middle = Vec2::new(3.5, (ROWS - 1) as f32 * 0.5);
    for row in 0..ROWS {
        for col in 0..COLS - 2 {
            let offset = Vec2::new(col as f32, row as f32) - middle;
            let angle = offset.y.atan2(offset.x);
            let reach = clock.mul_add(1.7, angle * 3.0).sin().mul_add(0.55, 3.0);
            let d = offset.length();
            if d <= reach {
                let edge = (reach - d).min(1.0);
                let colour = if (col + row) % 3 == 0 {
                    BRIGHT_SLIME
                } else {
                    SLIME
                };
                frame.rect_centred(
                    at(col, row),
                    size,
                    with_alpha(colour, edge.mul_add(0.6, 0.4)),
                );
            }
        }
    }
    let beat = (clock * 2.0).sin().mul_add(0.08, 1.0);
    frame.circle(at(3, ROWS / 2), cell * 0.42 * beat, NUCLEUS);
}

/// The blink counter, ticking four times a second of wall time.
///
/// Wall time is fine out here — the sim takes this as an argument and never
/// asks the clock itself, so a given world and a given frame still draw
/// identically anywhere.
fn anim_frame() -> u32 {
    #[allow(clippy::cast_sign_loss)] // `get_time` counts up from process start.
    {
        (get_time() / ANIMATION_RATE) as u32
    }
}

/// Where the telegraph pulse is in its cycle, `0..1`.
///
/// Kept in `f64` until the last moment: this counts up from process start, and
/// taking the fraction first is what stops a long session from turning the
/// pulse into a stutter.
fn telegraph_phase() -> f32 {
    #[allow(clippy::cast_possible_truncation)] // A fraction of one, by construction.
    {
        (get_time() / anim::PULSE_SECS).fract() as f32
    }
}

/// What a scrolling viewport should keep in the middle: the examine cursor
/// while it is out, otherwise the nucleus you are steering.
fn focus(view: &RenderView) -> Coord {
    view.examine.as_ref().map_or_else(
        || {
            view.status.active_nucleus.as_ref().map_or_else(
                || Coord::new(view.width / 2, view.height / 2),
                |(_, at)| *at,
            )
        },
        |examine| examine.pos,
    )
}
