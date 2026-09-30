//! The grand events of the era, going on without you.
//!
//! You are a bystander. The Crimson Harbor sends a convoy of sand frigates
//! to burn out the pirates' reef, builds a line of semaphore towers across
//! the basin, flies a survey dirigible over the drowned senate, and lights
//! fireworks for its warlord-consul's name-day; a comet crosses the sky.
//! You see as much of it as your route happens to show you, and none of it
//! needs you.
//!
//! Everything here is a pure function of the day, a fraction included, so
//! the map can draw it mid-voyage and a replay shows the same history.

use crate::sand_nomad::world::{MAP_H, RAID_DAY, Site};

/// Where the Harbor's frigates are, if they are out: three hulls in a
/// line, lead first.
#[must_use]
pub fn convoy(day: f32) -> Option<[(f32, f32); 3]> {
    const SET_OUT: f32 = 3.0;
    const HOME_AGAIN: f32 = 26.0;
    let raid = RAID_DAY as f32;
    let fort = Site::Fort.pos();
    let reef = Site::SpadeReef.pos();
    // They swing wide of the leviathan on the way out.
    let bend = (330.0, 214.0);
    let at = |t: f32| -> (f32, f32) {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        (
            (t * t).mul_add(reef.0, (2.0 * u * t).mul_add(bend.0, u * u * fort.0)),
            (t * t).mul_add(reef.1, (2.0 * u * t).mul_add(bend.1, u * u * fort.1)),
        )
    };
    let lead = if (SET_OUT..raid).contains(&day) {
        (day - SET_OUT) / (raid - SET_OUT)
    } else if (raid..raid + 3.0).contains(&day) {
        1.0
    } else if (raid + 3.0..HOME_AGAIN).contains(&day) {
        1.0 - (day - raid - 3.0) / (HOME_AGAIN - raid - 3.0)
    } else {
        return None;
    };
    // The line strings out behind its lead, whichever way it is going.
    let heading_out = day < raid + 3.0;
    let gap = if heading_out { -0.07 } else { 0.07 };
    Some([at(lead), at(lead + gap), at(2.0f32.mul_add(gap, lead))])
}

/// How hard the reef is burning, `0..=1`.
#[must_use]
pub fn reef_smoke(day: f32) -> f32 {
    let since = day - RAID_DAY as f32;
    if since < 0.0 {
        0.0
    } else {
        (1.0 - since / 7.0).max(0.0)
    }
}

/// Whether the reef belongs to the Harbor now.
#[must_use]
pub fn reef_taken(day: f32) -> bool {
    day >= RAID_DAY as f32
}

/// Where the survey dirigible is, if it is over the basin.
#[must_use]
pub fn dirigible(day: f32) -> Option<(f32, f32)> {
    let senate = Site::Senate.pos();
    let over = (senate.0 + 6.0, senate.1 - 34.0);
    let lerp = |a: (f32, f32), b: (f32, f32), t: f32| {
        ((b.0 - a.0).mul_add(t, a.0), (b.1 - a.1).mul_add(t, a.1))
    };
    let from = (-30.0, MAP_H + 20.0);
    let to = (430.0, -30.0);
    match day {
        d if (5.0..11.0).contains(&d) => Some(lerp(from, over, (d - 5.0) / 6.0)),
        d if (11.0..21.0).contains(&d) => {
            // Hanging there, turning a little in the wind.
            let sway = ((d - 11.0) * 2.1).sin() * 3.0;
            Some((over.0 + sway, over.1))
        }
        d if (21.0..33.0).contains(&d) => Some(lerp(over, to, (d - 21.0) / 12.0)),
        _ => None,
    }
}

/// Where the Harbor's semaphore towers stand, north from the fort, and
/// the day each goes up.
pub const TOWERS: [((f32, f32), f32); 4] = [
    ((236.0, 160.0), 4.0),
    ((222.0, 116.0), 9.0),
    ((212.0, 76.0), 16.0),
    ((204.0, 34.0), 23.0),
];

/// How many towers are up.
#[must_use]
pub fn towers_up(day: f32) -> usize {
    TOWERS.iter().filter(|(_, built)| day >= *built).count()
}

/// Whether the fort is letting off fireworks for the warlord-consul's
/// name-day, and how far into the show, `0..1`.
#[must_use]
pub fn fireworks(day: f32) -> Option<f32> {
    const FROM: f32 = 24.5;
    const UNTIL: f32 = 25.5;
    (FROM..UNTIL)
        .contains(&day)
        .then(|| (day - FROM) / (UNTIL - FROM))
}

/// How far across the sky the comet is, `0..1`, while it is visible.
#[must_use]
pub fn comet(day: f32) -> Option<f32> {
    const FROM: f32 = 28.0;
    const UNTIL: f32 = 44.0;
    (FROM..UNTIL)
        .contains(&day)
        .then(|| (day - FROM) / (UNTIL - FROM))
}

/// Something the whole basin hears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Happening {
    /// The convoy's guns at the reef.
    Raid,
    /// A semaphore tower going up.
    Tower,
    /// The name-day fireworks.
    Fireworks,
    /// The dirigible arriving overhead.
    Dirigible,
}

/// What happened on day `day`, the moment it dawned.
#[must_use]
pub fn dawn_of(day: u32) -> Vec<Happening> {
    let mut out = Vec::new();
    if day == RAID_DAY {
        out.push(Happening::Raid);
    }
    // Towers go up on whole days; the fireworks' night starts on day 24.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    if TOWERS.iter().any(|(_, built)| *built as u32 == day) {
        out.push(Happening::Tower);
    }
    if day == 25 {
        out.push(Happening::Fireworks);
    }
    if day == 11 {
        out.push(Happening::Dirigible);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_convoy_sails_out_burns_the_reef_and_comes_home() {
        assert!(convoy(1.0).is_none());
        let out = convoy(8.0).unwrap();
        let fort = Site::Fort.pos();
        let reef = Site::SpadeReef.pos();
        let d = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
        assert!(d(out[0], reef) < d(fort, reef));
        let there = convoy(RAID_DAY as f32 + 1.0).unwrap();
        assert!(d(there[0], reef) < 1.0);
        assert!(convoy(30.0).is_none());
        assert!(reef_smoke(RAID_DAY as f32 + 1.0) > 0.5);
        assert!(!reef_taken(RAID_DAY as f32 - 0.1));
        assert!(reef_taken(RAID_DAY as f32));
    }

    #[test]
    fn history_is_continuous_enough_to_draw() {
        // Nothing jumps more than a few pixels in a tenth of a day.
        let mut last = None;
        for tenth in 0..500 {
            let day = tenth as f32 / 10.0;
            let now = dirigible(day);
            if let (Some(a), Some(b)) = (last, now) {
                let (a, b): ((f32, f32), (f32, f32)) = (a, b);
                assert!(
                    (a.0 - b.0).hypot(a.1 - b.1) < 8.0,
                    "dirigible jumps at {day}"
                );
            }
            last = now;
        }
    }

    #[test]
    fn towers_go_up_in_order() {
        assert_eq!(towers_up(0.0), 0);
        assert_eq!(towers_up(100.0), TOWERS.len());
        for w in TOWERS.windows(2) {
            assert!(w[0].1 < w[1].1);
        }
        assert_eq!(dawn_of(RAID_DAY), vec![Happening::Raid]);
    }
}
