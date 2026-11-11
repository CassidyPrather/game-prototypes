//! The shell: a menu of prototypes, and whichever one is running.
//!
//! The loop is the only place that decides *what* is on screen. Everything
//! it can put there — the menu, each prototype — paints inside one
//! letterboxed [`ui::Frame`], so a window of any size shows the same layout.
//!
//! Escape, or the back button the shell draws over every prototype for
//! screens without one, leaves a prototype for the menu without unloading
//! it, so coming back is a pause rather than another wait on a sound bank.
//! Starting a fresh run is the prototype's own business; Leitmotif does it
//! with `R`.
//!
//! `--game <slug>` (natively) or `?game=<slug>` (on the web) opens straight
//! into one prototype, with the menu a back press away; wirenook.net embeds
//! the games that way. One more command-line mode skips the menu
//! altogether: `--replay <file>` plays a Space Trucking flight-recorder tape
//! back (see `docs/SPACE_TRUCKING.md`). The web has no arguments, so it
//! never triggers there.
//!
//! Going in and out is one continuous motion: the menu closes an iris of
//! night on the chosen window, the prototype loads behind it, and the iris
//! opens again on the prototype; leaving plays it in reverse.

// macroquad drives the whole binary on the one thread that owns the GL and
// audio contexts, and a prototype's sound handles are not `Send` for that
// reason. Nothing here ever crosses a thread, and the lint fires inside the
// `macroquad::main` expansion where a narrower allow cannot reach.
#![allow(clippy::future_not_send)]

mod back;
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

use crate::back::BackButton;
use crate::games::Game;
use crate::sounds::Sounds;
use crate::ui::{FRAME_H, FRAME_W, Frame};

/// Behind the letterbox bars, outside the frame.
const LETTERBOX: Color = Color::new(0.0, 0.0, 0.0, 1.0);

/// The menu's typeface, wirenook's body face. See `CREDITS.md`.
const FONT: &[u8] = include_bytes!("../assets/fonts/AtkinsonHyperlegible-Bold.ttf");

/// The localStorage key `web/index.html` copies a `?game=` query into, for
/// the wasm to read through quad-storage, which is the one road from the
/// page into the game that needs no `unsafe`.
#[cfg(target_arch = "wasm32")]
const OPEN_KEY: &str = "game-prototypes/open";

/// The prototype the player asked to open straight into, if any.
fn requested_game() -> Option<GameId> {
    #[cfg(target_arch = "wasm32")]
    let slug = quad_storage::STORAGE.lock().ok()?.get(OPEN_KEY);
    #[cfg(not(target_arch = "wasm32"))]
    let slug = std::env::args().skip_while(|arg| arg != "--game").nth(1);
    GameId::from_slug(&slug?)
}

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
    let mut back = BackButton::default();

    if let Some(id) = requested_game() {
        menu.point_to(id);
        running = Some((id, games::load(id).await));
        playing = true;
        revealing = Some(0.0);
    }

    loop {
        let dt = get_frame_time();
        let frame = Frame::fit(screen_width(), screen_height());
        clear_background(LETTERBOX);

        // Leaving is checked before ticking, so the frame a player presses
        // Escape or the back button on is the last one the prototype sees,
        // and the press that left never lands on the game underneath.
        let escape_is_ours = running
            .as_ref()
            .is_none_or(|(_, game)| !game.wants_escape());
        let back_spot = running.as_ref().map(|(_, game)| game.back_spot());
        let back_pressed = back_spot.is_some_and(|spot| BackButton::pressed(&frame, spot));
        if playing && (back_pressed || escape_is_ours && is_key_pressed(KeyCode::Escape)) {
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
                back.tick(dt);
                back.draw(&frame, game.back_spot());
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
            back.reset();
        }

        next_frame().await;
    }
}
