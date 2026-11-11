//! Where the shell's on-screen back button goes.
//!
//! Escape leaves a prototype for the menu, but a phone has no Escape, so the
//! shell also draws a button that does the same thing. This is the
//! arithmetic of where it sits, kept here so it can be tested; the binary's
//! `back` module draws it and listens for the press.
//!
//! The button would rather not cover any of the game. A phone is rarely
//! 4:3, so the letterbox around the shell's 800x600 frame usually leaves a
//! bar wide enough to hold it, and then it goes in the window's top-left
//! corner, over nothing. When the window is too close to 4:3 for that (a
//! desktop window, an iPad on its side) it has to sit on the frame, and each
//! prototype names a spot its own HUD leaves free.
//!
//! Positions and sizes here are in window pixels unless they say otherwise.
//! Spots are in frame units, like everything a prototype draws.

/// The smallest the button gets as a touch target, in CSS pixels: the
/// usual platform guidance for anything a thumb has to hit.
pub const TOUCH_TARGET: f32 = 44.0;

/// Gap between the button and the window's edge, in CSS pixels.
pub const MARGIN: f32 = 8.0;

/// The button's side when it sits on the frame, in frame units, as
/// `(min, max)`. Big enough to hit on a small window, small enough not to
/// cover HUD on a big one.
pub const ON_FRAME_SIDE: (f32, f32) = (28.0, 44.0);

/// Where the button sits on the frame unless a prototype says otherwise: the
/// top-left corner, a margin in. Frame units, the centre of the button.
pub const DEFAULT_SPOT: [f32; 2] = [
    MARGIN + ON_FRAME_SIDE.1 * 0.5,
    MARGIN + ON_FRAME_SIDE.1 * 0.5,
];

/// The window as the button sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    /// Top-left of the letterboxed frame, in pixels. Its two coordinates
    /// are the widths of the left and top bars.
    pub frame_origin: [f32; 2],
    /// Pixels per frame unit.
    pub frame_scale: f32,
    /// Pixels per CSS pixel (or per logical pixel, natively).
    pub dpi: f32,
}

/// Where the button is this frame: a square, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub centre: [f32; 2],
    pub side: f32,
    /// Whether it sits in a letterbox bar rather than on the frame.
    pub in_bar: bool,
}

impl Placement {
    /// Whether a press at `point` hits the button. The target reaches a
    /// margin past what is drawn, since a thumb is not a cursor.
    #[must_use]
    pub fn contains(&self, point: [f32; 2], dpi: f32) -> bool {
        let reach = self.side.mul_add(0.5, MARGIN * dpi);
        (point[0] - self.centre[0]).abs() <= reach && (point[1] - self.centre[1]).abs() <= reach
    }
}

/// Where the button goes in `window`, given the prototype's `spot` for it
/// on the frame.
#[must_use]
pub fn place(window: Window, spot: [f32; 2]) -> Placement {
    let side = TOUCH_TARGET * window.dpi;
    let margin = MARGIN * window.dpi;
    let room = margin.mul_add(2.0, side);
    let [left, top] = window.frame_origin;
    if left >= room || top >= room {
        let at = side.mul_add(0.5, margin);
        return Placement {
            centre: [at, at],
            side,
            in_bar: true,
        };
    }
    // A minimised window reports a scale near zero; the clamp keeps the
    // side finite.
    let units = (side / window.frame_scale).clamp(ON_FRAME_SIDE.0, ON_FRAME_SIDE.1);
    Placement {
        centre: [
            spot[0].mul_add(window.frame_scale, left),
            spot[1].mul_add(window.frame_scale, top),
        ],
        side: units * window.frame_scale,
        in_bar: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shell's frame letterboxed into a window of `w` by `h` pixels.
    fn window(w: f32, h: f32, dpi: f32) -> Window {
        let scale = (w / 800.0).min(h / 600.0);
        Window {
            frame_origin: [
                800.0f32.mul_add(-scale, w) * 0.5,
                600.0f32.mul_add(-scale, h) * 0.5,
            ],
            frame_scale: scale,
            dpi,
        }
    }

    #[test]
    fn a_portrait_phone_puts_it_in_the_top_bar() {
        // An iPhone 13: 390x844 CSS pixels at 3x.
        let win = window(1170.0, 2532.0, 3.0);
        let place = place(win, DEFAULT_SPOT);
        assert!(place.in_bar);
        assert!(
            place.side.mul_add(0.5, place.centre[1]) < win.frame_origin[1],
            "the button reaches into the frame"
        );
        assert!((place.side - 132.0).abs() < 1e-3, "not a full touch target");
    }

    #[test]
    fn a_landscape_phone_puts_it_in_the_left_bar() {
        let win = window(2532.0, 1170.0, 3.0);
        let place = place(win, DEFAULT_SPOT);
        assert!(place.in_bar);
        assert!(place.side.mul_add(0.5, place.centre[0]) < win.frame_origin[0]);
    }

    #[test]
    fn a_four_by_three_window_puts_it_on_the_prototypes_spot() {
        let place = place(window(800.0, 600.0, 1.0), [770.0, 30.0]);
        assert!(!place.in_bar);
        assert!((place.centre[0] - 770.0).abs() < 1e-3 && (place.centre[1] - 30.0).abs() < 1e-3);
        assert!((place.side - 44.0).abs() < 1e-3);
    }

    #[test]
    fn on_the_frame_the_side_stays_within_its_bounds() {
        // A huge low-dpi window wants a small button in frame units...
        let big = place(window(3200.0, 2400.0, 1.0), DEFAULT_SPOT);
        assert!((big.side / 4.0 - ON_FRAME_SIDE.0).abs() < 1e-3);
        // ...and a tiny high-dpi one a huge one.
        let small = place(window(400.0, 300.0, 3.0), DEFAULT_SPOT);
        assert!((small.side / 0.5 - ON_FRAME_SIDE.1).abs() < 1e-3);
        // The default spot keeps the largest button a margin off the edges.
        let corner = ON_FRAME_SIDE.1.mul_add(-0.5, DEFAULT_SPOT[0]);
        assert!((corner - MARGIN).abs() < 1e-3);
    }

    #[test]
    fn a_minimised_window_is_still_finite() {
        let place = place(
            Window {
                frame_origin: [0.0, 0.0],
                frame_scale: f32::EPSILON,
                dpi: 1.0,
            },
            DEFAULT_SPOT,
        );
        assert!(place.side.is_finite() && place.centre.iter().all(|c| c.is_finite()));
    }

    #[test]
    fn a_press_just_outside_the_drawing_still_counts() {
        let place = place(window(800.0, 600.0, 1.0), DEFAULT_SPOT);
        let edge = place.side.mul_add(0.5, place.centre[0]);
        assert!(place.contains([edge + MARGIN - 1.0, place.centre[1]], 1.0));
        assert!(!place.contains([edge + MARGIN + 1.0, place.centre[1]], 1.0));
        assert!(!place.contains([400.0, 300.0], 1.0));
    }
}
