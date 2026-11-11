//! The shell's back button: Escape, for a screen with no keyboard.
//!
//! Drawn over every prototype, in a letterbox bar when the window leaves one
//! wide enough and otherwise on a spot the prototype keeps clear (see
//! [`game_prototypes::shell::back`], which does the placing). It wears the
//! menu's key-cap chip, since it is the menu it leads back to, and lights up
//! under a pointer.

use game_prototypes::shell::back::{self, Placement, Window};
use macroquad::input::{MouseButton, is_mouse_button_pressed, mouse_position, touches};
use macroquad::window::screen_dpi_scale;

use crate::menu::brand;
use crate::ui::{Frame, MqVec2, with_alpha};

/// Seconds the button takes to fade in once a prototype is on screen.
const APPEAR_SECS: f32 = 0.35;

/// How opaque the button sits when nothing is pointing at it: there, but
/// quieter than the game.
const IDLE_ALPHA: f32 = 0.8;

#[derive(Default)]
pub struct BackButton {
    /// Seconds since the prototype came on screen.
    shown: f32,
    /// Where the pointer was when the prototype came on screen. Until it
    /// moves there is no pointer to speak of: a phone reports one parked at
    /// the origin, right on top of the button, and the button should not
    /// light up for a mouse that is not there.
    rest: Option<(f32, f32)>,
    /// Whether a mouse has moved since, and so can hover.
    pointing: bool,
}

impl BackButton {
    /// A prototype has just come on screen.
    pub const fn reset(&mut self) {
        self.shown = 0.0;
        self.rest = None;
        self.pointing = false;
    }

    pub fn tick(&mut self, dt: f32) {
        self.shown = (self.shown + dt).min(APPEAR_SECS);
        let now = mouse_position();
        let rest = *self.rest.get_or_insert(now);
        // A finger drags the emulated mouse about too, and a finger does not
        // hover.
        self.pointing = touches().is_empty() && (self.pointing || now != rest);
    }

    /// Whether the button was pressed this frame. Asked before the prototype
    /// updates, so a press that leaves never reaches the game underneath.
    pub fn pressed(frame: &Frame, spot: MqVec2) -> bool {
        is_mouse_button_pressed(MouseButton::Left) && Self::under_pointer(frame, spot)
    }

    pub fn draw(&self, frame: &Frame, spot: MqVec2) {
        let place = placement(frame, spot);
        let hover = self.pointing && Self::under_pointer(frame, spot);
        let alpha = if hover { 1.0 } else { IDLE_ALPHA } * (self.shown / APPEAR_SECS);
        let centre = MqVec2::from(place.centre);
        // Sizes are in frame units, positions in pixels.
        let side = MqVec2::splat(place.side / frame.scale());
        let px = place.side / 44.0;

        frame.rect_centred(
            centre + MqVec2::new(0.0, 3.0 * px),
            side,
            with_alpha(brand::VIOLET, 0.45 * alpha),
        );
        let (fill, ink) = if hover {
            (brand::MAGENTA, brand::PANEL)
        } else {
            (brand::PANEL, brand::INK)
        };
        frame.rect_centred(centre, side, with_alpha(fill, alpha));
        frame.rect_lines_centred(centre, side, 1.0, with_alpha(brand::LINE, alpha));

        // A left arrow: a stem and a head, drawn rather than typed.
        let ink = with_alpha(ink, alpha);
        let reach = place.side * 0.24;
        let tip = centre - MqVec2::new(reach, 0.0);
        let neck = tip + MqVec2::new(reach * 0.8, 0.0);
        frame.line(
            neck,
            centre + MqVec2::new(reach, 0.0),
            place.side * 0.09 / frame.scale(),
            ink,
        );
        Frame::triangle(
            tip,
            neck - MqVec2::new(0.0, reach * 0.75),
            neck + MqVec2::new(0.0, reach * 0.75),
            ink,
        );
    }

    fn under_pointer(frame: &Frame, spot: MqVec2) -> bool {
        placement(frame, spot).contains(mouse_position().into(), screen_dpi_scale())
    }
}

fn placement(frame: &Frame, spot: MqVec2) -> Placement {
    back::place(
        Window {
            frame_origin: frame.at(MqVec2::ZERO).into(),
            frame_scale: frame.scale(),
            dpi: screen_dpi_scale(),
        },
        spot.into(),
    )
}
