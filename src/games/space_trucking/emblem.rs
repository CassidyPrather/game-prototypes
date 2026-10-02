//! Space Trucking's mark on the menu: the radar the ship's console is built
//! around, with the worlds turning on their orbits and a freighter nosing
//! across them.

use std::f32::consts::TAU;

use super::palette::{
    AMBER, GLINT, PHOSPHOR, PHOSPHOR_DIM, POI_EARTH, POI_GUILD_EDGE, POI_MARS, POI_NEPTUNE, fade,
};
use crate::ui::{Frame, MqVec2};

/// How flat the orbits are drawn, as if the system were seen at a tilt.
const SQUASH: f32 = 0.4;

/// Draw the mark, `width` frame units across and centred on `centre`.
///
/// Everything stays within half the width across and a fifth of it up and
/// down, like the other emblems: the orbits are squashed to fit.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    let unit = width / 100.0;
    let px = frame.scale() * unit;
    let at = |x: f32, y: f32| centre + MqVec2::new(x, y) * px;
    let sun = at(-8.0, 0.0);
    // A point on an orbit of this radius, in emblem units, at this angle.
    let on_orbit = |radius: f32, angle: f32| {
        sun + MqVec2::new(angle.cos() * radius, angle.sin() * radius * SQUASH) * px
    };

    // The orbits, as faint range rings, and a world riding each. Inner
    // worlds are quicker; the outermost goes the wrong way round, like the
    // Guild's station.
    let worlds = [
        (14.0, 0.9, 0.0, POI_EARTH, 2.4),
        (27.0, 0.45, 2.1, POI_MARS, 2.8),
        (40.0, -0.28, 4.4, POI_GUILD_EDGE, 3.2),
    ];
    for (radius, rate, phase, colour, size) in worlds {
        let ring: Vec<MqVec2> = (0..40_u8)
            .map(|step| on_orbit(radius, f32::from(step) / 40.0 * TAU))
            .collect();
        frame.outline(&ring, 0.5 * unit, fade(PHOSPHOR_DIM, 0.8));
        frame.circle(
            on_orbit(radius, clock.mul_add(rate, phase)),
            size * unit,
            colour,
        );
    }
    // Far out, a blue giant on its slow loop.
    frame.circle(
        on_orbit(47.0, clock.mul_add(0.12, 1.0)),
        2.0 * unit,
        POI_NEPTUNE,
    );

    // The sun, flickering a little.
    let flicker = (clock * 3.0).sin().mul_add(0.06, 0.9);
    frame.circle(sun, 6.5 * unit, fade(AMBER, 0.85 * flicker));
    frame.circle(sun, 3.2 * unit, fade(GLINT, 0.9));

    // The charted course, in dashes, and the freighter riding it. `along`
    // runs 0 to 1 from the top left of the system to the bottom right.
    let course = |along: f32| {
        at(
            44.0_f32.mul_add(along, -6.0),
            24.0_f32.mul_add(along, -13.0),
        )
    };
    for dash in 0..5_u8 {
        let start = f32::from(dash) / 4.5;
        frame.line(
            course(start),
            course(start + 0.08),
            0.7 * unit,
            fade(PHOSPHOR, 0.55),
        );
    }
    // Crawling, not racing: one trip takes the better part of a minute.
    let ship = course((clock * 0.045).fract());
    let rel = |x: f32, y: f32| ship + MqVec2::new(x, y) * px;
    frame.rect_centred(ship, MqVec2::new(7.0, 3.4) * unit, AMBER);
    Frame::triangle(rel(3.4, -1.7), rel(6.4, 0.0), rel(3.4, 1.7), GLINT);
    // The drive, pulsing.
    frame.circle(
        rel(-4.6, 0.0),
        (clock * 9.0).sin().abs().mul_add(0.5, 1.1) * unit,
        GLINT,
    );
}
