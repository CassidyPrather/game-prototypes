//! The main menu: a card per prototype, pick one.
//!
//! Almost no text, like the prototypes themselves: each card is mostly the
//! toy's own emblem, with its name underneath because a shelf of unnamed
//! pictures stops being a menu once there are a few of them. What to press
//! is drawn as key caps.
//!
//! The part that can be wrong — where the pointer is — lives in the
//! library's [`game_prototypes::shell`] and is unit-tested there. This file
//! only reads input and draws.

use game_prototypes::VERSION;
use game_prototypes::shell::{GameId, Menu};
use macroquad::color::Color;
use macroquad::input::{
    KeyCode, MouseButton, is_key_pressed, is_mouse_button_pressed, mouse_position,
};

use crate::games;
use crate::ui::{self, DIM, Frame, HIGHLIGHT, MqVec2, OVERLAY, pulse, with_alpha};

/// Behind the menu.
const BACKDROP: Color = Color::new(0.05, 0.055, 0.08, 1.0);

/// One entry's card, in frame units.
const CARD: MqVec2 = MqVec2::new(360.0, 170.0);

/// Card centres are this far apart.
const CARD_PITCH: f32 = CARD.y + 26.0;

/// Where the stack of cards is centred.
const STACK_Y: f32 = 320.0;

/// Centre of the card for entry `index`, in frame units.
fn card_centre(index: usize) -> MqVec2 {
    // The stack stays centred however many entries there are.
    let offset = (Menu::len() as f32 - 1.0).mul_add(-0.5, index as f32);
    MqVec2::new(ui::FRAME_W * 0.5, offset.mul_add(CARD_PITCH, STACK_Y))
}

/// Whether a frame-unit point is inside the card for `index`.
fn over_card(index: usize, at: MqVec2) -> bool {
    let offset = at - card_centre(index);
    offset.x.abs() < CARD.x * 0.5 && offset.y.abs() < CARD.y * 0.5
}

/// Read the keyboard and mouse for one frame. Returns the prototype to
/// start, if the player asked for one.
///
/// Enter and a click start; Space does not, so that holding it to launch
/// Leitmotif cannot also spend its fermata on the first frame of play.
pub fn update(menu: &mut Menu, frame: &Frame) -> Option<GameId> {
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        menu.move_by(1);
    }
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        menu.move_by(-1);
    }

    let (mouse_x, mouse_y) = mouse_position();
    let at = frame.frame_pos(MqVec2::new(mouse_x, mouse_y));
    let hovered = (0..Menu::len()).find(|&index| over_card(index, at));
    if let Some(index) = hovered {
        menu.point_at(index);
    }

    let clicked = hovered.is_some() && is_mouse_button_pressed(MouseButton::Left);
    (clicked || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter))
        .then(|| menu.selected())
}

pub fn draw(menu: Menu, frame: &Frame, clock: f32) {
    frame.rect(
        frame.at(MqVec2::ZERO),
        MqVec2::new(ui::FRAME_W, ui::FRAME_H),
        BACKDROP,
    );

    collection_mark(frame, frame.at(MqVec2::new(ui::FRAME_W * 0.5, 92.0)), clock);

    for (index, &id) in GameId::ALL.iter().enumerate() {
        card(frame, menu, index, id, clock);
    }

    hints(frame, clock);

    // A launcher is a developer's front door, so it says which build it is.
    // Bottom left, where a long `git describe` has room to be long.
    frame.text_centred(
        VERSION,
        frame.at(MqVec2::new(94.0, ui::FRAME_H - 16.0)),
        13.0,
        with_alpha(DIM, 0.5),
    );
}

/// The collection's own mark: a circle, a square and a triangle, the three
/// shapes every prototype here is built out of. `centre` is in pixels, as
/// every primitive here takes.
fn collection_mark(frame: &Frame, centre: MqVec2, clock: f32) {
    let s = frame.scale();
    let step = 34.0 * s;
    let colour = with_alpha(OVERLAY, 0.5);
    let breath = (clock * 1.5).sin().mul_add(0.04, 1.0);

    frame.ring(centre - MqVec2::new(step, 0.0), 11.0 * breath, 2.0, colour);
    frame.rect_lines_centred(
        centre,
        MqVec2::new(20.0 * breath, 20.0 * breath),
        2.0,
        colour,
    );
    let tip = centre + MqVec2::new(step, -11.0 * breath * s);
    let foot = 11.0 * breath * s;
    frame.line(tip, tip + MqVec2::new(-foot, foot * 2.0), 2.0, colour);
    frame.line(tip, tip + MqVec2::new(foot, foot * 2.0), 2.0, colour);
    frame.line(
        tip + MqVec2::new(-foot, foot * 2.0),
        tip + MqVec2::new(foot, foot * 2.0),
        2.0,
        colour,
    );
}

/// One prototype's card: its emblem, its name, and a border that says
/// whether it is the one under the pointer.
fn card(frame: &Frame, menu: Menu, index: usize, id: GameId, clock: f32) {
    let centre = card_centre(index);
    let chosen = menu.index() == index;
    let glow = if chosen { pulse(clock) } else { 0.0 };

    frame.rect_centred(
        frame.at(centre),
        CARD,
        Color::new(1.0, 1.0, 1.0, glow.mul_add(0.02, 0.03)),
    );
    frame.rect_lines_centred(
        frame.at(centre),
        CARD,
        if chosen { 2.5 } else { 1.5 },
        if chosen {
            with_alpha(HIGHLIGHT, glow.mul_add(0.35, 0.6))
        } else {
            with_alpha(DIM, 0.5)
        },
    );

    games::emblem(
        id,
        frame,
        frame.at(centre - MqVec2::new(0.0, 22.0)),
        CARD.x * 0.62,
        clock,
    );
    frame.text_centred(
        id.name(),
        frame.at(centre + MqVec2::new(0.0, CARD.y.mul_add(0.5, -22.0))),
        24.0,
        if chosen {
            OVERLAY
        } else {
            with_alpha(DIM, 0.8)
        },
    );
}

/// What to press: the arrows only once there is somewhere to move to, and
/// the return key, which is the one thing being asked for.
fn hints(frame: &Frame, clock: f32) {
    let y = ui::FRAME_H - 54.0;
    let glow = with_alpha(HIGHLIGHT, pulse(clock).mul_add(0.4, 0.6));

    if Menu::len() > 1 {
        let dim = with_alpha(DIM, 0.7);
        frame.arrow_cap(
            MqVec2::new(0.0, -1.0),
            frame.at(MqVec2::new(ui::FRAME_W.mul_add(0.5, -118.0), y)),
            24.0,
            dim,
        );
        frame.arrow_cap(
            MqVec2::new(0.0, 1.0),
            frame.at(MqVec2::new(ui::FRAME_W.mul_add(0.5, -88.0), y)),
            24.0,
            dim,
        );
    }
    frame.enter_cap(
        frame.at(MqVec2::new(ui::FRAME_W * 0.5, y)),
        MqVec2::new(96.0, 26.0),
        glow,
    );
}
