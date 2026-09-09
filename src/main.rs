//! The shell: a menu of prototypes, and whichever one is running.
//!
//! The loop is the only place that decides *what* is on screen. Everything
//! it can put there — the menu, each prototype — paints inside one
//! letterboxed [`ui::Frame`], so a window of any size shows the same layout.
//!
//! Escape leaves a prototype for the menu without unloading it, so coming
//! back is a pause rather than another wait on a sound bank. Starting a
//! fresh run is the prototype's own business; Leitmotif does it with `R`.

// macroquad drives the whole binary on the one thread that owns the GL and
// audio contexts, and a prototype's sound handles are not `Send` for that
// reason. Nothing here ever crosses a thread, and the lint fires inside the
// `macroquad::main` expansion where a narrower allow cannot reach.
#![allow(clippy::future_not_send)]

mod games;
mod menu;
mod ui;

use macroquad::color::Color;
use macroquad::input::{KeyCode, is_key_pressed};
use macroquad::time::get_frame_time;
use macroquad::window::{Conf, clear_background, next_frame, screen_height, screen_width};

use game_prototypes::shell::{GameId, Menu};

use crate::games::Game;
use crate::ui::{FRAME_H, FRAME_W, Frame};

/// Behind the letterbox bars, outside the frame.
const LETTERBOX: Color = Color::new(0.0, 0.0, 0.0, 1.0);

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
    let mut menu = Menu::new();
    // The prototype in memory, if any, and which one it is. Loading is slow
    // enough to be worth doing once.
    let mut running: Option<(GameId, Box<dyn Game>)> = None;
    let mut playing = false;
    let mut clock = 0.0_f32;

    loop {
        let dt = get_frame_time();
        clock += dt;
        let frame = Frame::fit(screen_width(), screen_height());
        clear_background(LETTERBOX);

        // Leaving is checked before ticking, so the frame a player presses
        // Escape on is the last one the prototype sees.
        if playing && is_key_pressed(KeyCode::Escape) {
            playing = false;
        }

        let mut chosen = None;
        if playing {
            if let Some((_, game)) = running.as_mut() {
                game.update(dt);
                game.draw(&frame);
            }
        } else {
            chosen = menu::update(&mut menu, &frame);
            menu::draw(menu, &frame, clock);
        }

        // Loading holds the loop for a while, so it happens after this
        // frame is drawn and the menu is what stays on screen meanwhile.
        if let Some(id) = chosen {
            if running.as_ref().is_none_or(|(loaded, _)| *loaded != id) {
                running = Some((id, games::load(id).await));
            }
            playing = true;
        }

        next_frame().await;
    }
}
