//! The shell: a menu of prototypes, and whichever one is running.
//!
//! The loop is the only place that decides *what* is on screen. Everything
//! it can put there — the menu, each prototype — paints inside one
//! letterboxed [`ui::Frame`], so a window of any size shows the same layout.
//!
//! Escape leaves a prototype for the menu without unloading it, so coming
//! back is a pause rather than another wait on a sound bank. Starting a
//! fresh run is the prototype's own business; Leitmotif does it with `R`.
//!
//! One command-line mode skips the menu: `--replay <file>` plays a Space
//! Trucking flight-recorder tape back (see `docs/SPACE_TRUCKING.md`). The
//! web has no arguments, so it never triggers there.
//!
//! Going in and out is one continuous motion: the menu closes an iris of
//! night on the chosen window, the prototype loads behind it, and the iris
//! opens again on the prototype; leaving plays it in reverse.

// macroquad drives the whole binary on the one thread that owns the GL and
// audio contexts, and a prototype's sound handles are not `Send` for that
// reason. Nothing here ever crosses a thread, and the lint fires inside the
// `macroquad::main` expansion where a narrower allow cannot reach.
#![allow(clippy::future_not_send)]

mod games;
mod menu;
mod sounds;
mod ui;

use macroquad::color::Color;
use macroquad::input::{KeyCode, is_key_pressed};
use macroquad::text::load_ttf_font_from_bytes;
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, clear_background, next_frame, screen_height, screen_width};

use game_prototypes::shell::sfx::Sfx;
use game_prototypes::shell::{GameId, Menu};

use crate::games::Game;
use crate::sounds::Sounds;
use crate::ui::{FRAME_H, FRAME_W, Frame};

/// Behind the letterbox bars, outside the frame.
const LETTERBOX: Color = Color::new(0.0, 0.0, 0.0, 1.0);

/// The menu's typeface, wirenook's body face. See `CREDITS.md`.
const FONT: &[u8] = include_bytes!("../assets/fonts/AtkinsonHyperlegible-Bold.ttf");

fn window_conf() -> Conf {
    Conf {
        window_title: "game prototypes".to_owned(),
        window_width: FRAME_W as i32,
        window_height: FRAME_H as i32,
        high_dpi: true,
        ..Conf::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Scoped so the non-Send `Args` iterator never lives across an await.
    let (replay_mode, replay_path) = {
        let mut args = std::env::args();
        (args.nth(1).as_deref() == Some("--replay"), args.next())
    };
    if replay_mode {
        games::space_trucking::replay_session(replay_path).await;
        return;
    }

    let font = load_ttf_font_from_bytes(FONT).expect("the vendored font parses");
    let mut screen = menu::Screen::new(font);
    let mut sounds = Sounds::load().await;
    let mut menu = Menu::new();
    // The prototype in memory, if any, and which one it is. Loading is slow
    // enough to be worth doing once.
    let mut running: Option<(GameId, Box<dyn Game>)> = None;
    let mut playing = false;
    // Seconds since a prototype came on screen, while its iris is opening.
    let mut revealing: Option<f32> = None;

    loop {
        let dt = get_frame_time();
        let frame = Frame::fit(screen_width(), screen_height());
        clear_background(LETTERBOX);

        // Leaving is checked before ticking, so the frame a player presses
        // Escape on is the last one the prototype sees.
        if playing && is_key_pressed(KeyCode::Escape) {
            playing = false;
            revealing = None;
            if let Some((_, game)) = running.as_mut() {
                game.leave();
            }
            screen.show_after_game();
            sounds.play(Sfx::Back);
        }

        let mut chosen = None;
        if playing {
            if let Some((_, game)) = running.as_mut() {
                game.update(dt);
                game.draw(&frame);
            }
            if let Some(age) = revealing.as_mut() {
                menu::reveal(&frame, *age);
                *age += dt;
                revealing = revealing.filter(|&age| age < menu::REVEAL_SECS);
            }
        } else {
            chosen = screen.update(&mut menu, &frame, dt, &mut sounds);
            screen.draw(menu, &frame, running.as_ref().map(|(id, _)| *id));
        }

        // Loading holds the loop for a while, so it happens after this
        // frame is drawn, when the menu's iris has covered the screen.
        if let Some(id) = chosen {
            if running.as_ref().is_none_or(|(loaded, _)| *loaded != id) {
                running = Some((id, games::load(id).await));
            }
            playing = true;
            revealing = Some(0.0);
        }

        next_frame().await;
    }
}
