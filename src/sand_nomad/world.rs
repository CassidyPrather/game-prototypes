//! The basin, by hand: where everything is and what everyone has.
//!
//! Nothing here is rolled. The Grand Basin is the dry bed of what was the
//! ocean of the Azure Steppe, and it is the same basin every time: eleven
//! places, their waystones, their people and the things those people are
//! willing to part with. A story needs a fixed stage.
//!
//! Map positions are in the frontend's map pixels, `400 x 228`, the
//! ancient shoreline along the top and the Harbor's road coming in from
//! the south.

use crate::sand_nomad::barter::Culture;
use crate::sand_nomad::grid::Grid;
use crate::sand_nomad::item::{Item, ItemId, Kind};
use crate::sand_nomad::motive::Motive;

/// Width of the map, in map pixels.
pub const MAP_W: f32 = 400.0;
/// Height of the map, in map pixels.
pub const MAP_H: f32 = 228.0;

/// Columns and rows of the hold.
pub const HOLD_SIZE: (u8, u8) = (8, 4);
/// Columns and rows of a trader's rug.
pub const STOCK_SIZE: (u8, u8) = (6, 4);

/// The day the Harbor's convoy reaches the pirates' reef, and the reef
/// changes hands.
pub const RAID_DAY: u32 = 14;

/// One of the eleven places.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Site {
    /// Home: your kin's mooring stakes, a well and a pyre. The circuit
    /// starts and ends here.
    Tether,
    /// A lighthouse on the ancient shore, still kept, facing no sea.
    Lighthouse,
    /// A ring of drifter tents.
    Hearthring,
    /// Thesean shipwrights working inside the wreck of a seagoing ship.
    Drydock,
    /// The white salt flats.
    SaltFlats,
    /// Coral pillars and the pirates who live in them.
    SpadeReef,
    /// The skeleton of something vast from when this was sea.
    Leviathan,
    /// The Crimson Harbor's fort at the basin's southern edge.
    Fort,
    /// The Pyre-Kin's fleet, moored around its great pyre.
    Ashfleet,
    /// The drowned senate of the old republic, half out of the sand.
    Senate,
    /// A well under an arch of jawbone.
    WellOfTeeth,
}

impl Site {
    /// Every place, home first.
    pub const ALL: [Self; 11] = [
        Self::Tether,
        Self::Lighthouse,
        Self::Hearthring,
        Self::Drydock,
        Self::SaltFlats,
        Self::SpadeReef,
        Self::Leviathan,
        Self::Fort,
        Self::Ashfleet,
        Self::Senate,
        Self::WellOfTeeth,
    ];

    /// Position in [`Site::ALL`].
    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&s| s == self)
            .expect("every site is listed")
    }

    /// Where it is on the map.
    #[must_use]
    pub const fn pos(self) -> (f32, f32) {
        match self {
            Self::Tether => (56.0, 112.0),
            Self::Lighthouse => (40.0, 34.0),
            Self::Hearthring => (132.0, 62.0),
            Self::Drydock => (258.0, 42.0),
            Self::SaltFlats => (352.0, 70.0),
            Self::SpadeReef => (360.0, 174.0),
            Self::Leviathan => (286.0, 126.0),
            Self::Fort => (250.0, 200.0),
            Self::Ashfleet => (200.0, 124.0),
            Self::Senate => (150.0, 196.0),
            Self::WellOfTeeth => (98.0, 158.0),
        }
    }

    /// The motive its waystone asks for. Home has none: coming back is the
    /// offering.
    #[must_use]
    pub const fn waystone(self) -> Option<Motive> {
        match self {
            Self::Tether => None,
            Self::Lighthouse | Self::SaltFlats => Some(Motive::Bliss),
            Self::Hearthring | Self::WellOfTeeth => Some(Motive::Love),
            Self::Drydock | Self::Leviathan => Some(Motive::Repose),
            Self::SpadeReef | Self::Ashfleet => Some(Motive::Zeal),
            Self::Fort => Some(Motive::Hate),
            Self::Senate => Some(Motive::Pain),
        }
    }

    /// Who trades here before the raid.
    #[must_use]
    pub const fn trader(self) -> Option<Culture> {
        match self {
            Self::Tether | Self::WellOfTeeth => None,
            Self::Lighthouse => Some(Culture::Keeper),
            Self::Hearthring => Some(Culture::Drifter),
            Self::Drydock => Some(Culture::Thesean),
            Self::SaltFlats => Some(Culture::Scraper),
            Self::SpadeReef => Some(Culture::Pirate),
            Self::Leviathan => Some(Culture::Bones),
            Self::Fort => Some(Culture::Harbor),
            Self::Ashfleet => Some(Culture::PyreKin),
            Self::Senate => Some(Culture::Hermit),
        }
    }

    /// Whether there is a well to fill jars at, free.
    #[must_use]
    pub const fn well(self) -> bool {
        matches!(self, Self::Tether | Self::WellOfTeeth)
    }

    /// Whether there is a pyre to burn things on.
    #[must_use]
    pub const fn pyre(self) -> bool {
        matches!(self, Self::Tether | Self::Ashfleet)
    }

    /// Whether the Theseans' workbench is here.
    #[must_use]
    pub const fn bench(self) -> bool {
        matches!(self, Self::Drydock)
    }

    /// What the trader here has on their rug at the start.
    #[must_use]
    pub const fn stock(self) -> &'static [Stock] {
        use Kind as K;
        use Motive as M;
        match self {
            Self::Tether | Self::WellOfTeeth => &[],
            Self::Lighthouse => &[
                Stock(K::Spyglass, None, 0),
                Stock(K::Letters, Some(M::Love), 3),
                Stock(K::MusicBox, Some(M::Repose), 2),
                Stock(K::Map, Some(M::Repose), 2),
                Stock(K::Kettle, Some(M::Repose), 4),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
            ],
            Self::Hearthring => &[
                Stock(K::Doll, Some(M::Bliss), 2),
                Stock(K::Locket, Some(M::Love), 5),
                Stock(K::Comb, Some(M::Love), 1),
                Stock(K::Lamp, Some(M::Love), 1),
                Stock(K::Flute, Some(M::Bliss), 3),
                Stock(K::Kettle, Some(M::Bliss), 0),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
            ],
            Self::Drydock => &[
                Stock(K::Oar, Some(M::Zeal), 1),
                Stock(K::Plank, Some(M::Repose), 0),
                Stock(K::Rope, Some(M::Repose), 1),
                Stock(K::Figurine, Some(M::Repose), 3),
                Stock(K::Beads, Some(M::Repose), 2),
                Stock(K::Flag, Some(M::Zeal), 2),
                Stock(K::Water, None, 0),
            ],
            Self::SaltFlats => &[
                Stock(K::Salt, None, 0),
                Stock(K::Salt, None, 0),
                Stock(K::Salt, None, 0),
                Stock(K::GlassFlower, Some(M::Bliss), 1),
                Stock(K::GlassFlower, Some(M::Bliss), 3),
                Stock(K::Shell, Some(M::Bliss), 2),
                Stock(K::Charm, Some(M::Pain), 3),
            ],
            Self::SpadeReef => &[
                Stock(K::Dagger, Some(M::Hate), 2),
                Stock(K::Dagger, Some(M::Hate), 4),
                Stock(K::Flag, Some(M::Hate), 3),
                Stock(K::Pistol, Some(M::Zeal), 2),
                Stock(K::Mask, Some(M::Zeal), 4),
                Stock(K::Coins, None, 0),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
            ],
            Self::Leviathan => &[
                Stock(K::Rib, Some(M::Repose), 2),
                Stock(K::Tooth, Some(M::Pain), 3),
                Stock(K::Shell, Some(M::Bliss), 1),
                Stock(K::Charm, Some(M::Pain), 1),
                Stock(K::Tooth, Some(M::Pain), 5),
            ],
            Self::Fort => &[
                Stock(K::Clockbird, None, 0),
                Stock(K::Compass, None, 0),
                Stock(K::Locket, Some(M::Love), 5),
                Stock(K::Letters, Some(M::Love), 2),
                Stock(K::Coins, None, 0),
                Stock(K::Coins, None, 0),
                Stock(K::Tin, None, 0),
                Stock(K::Tin, None, 0),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
            ],
            Self::Ashfleet => &[
                Stock(K::Mask, Some(M::Zeal), 2),
                Stock(K::Idol, Some(M::Zeal), 1),
                Stock(K::Lamp, Some(M::Zeal), 1),
                Stock(K::Charm, Some(M::Pain), 2),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
                Stock(K::Water, None, 0),
            ],
            Self::Senate => &[
                Stock(K::Bust, Some(M::Pain), 3),
                Stock(K::Medal, Some(M::Pain), 1),
                Stock(K::Medal, Some(M::Repose), 2),
                Stock(K::Beads, Some(M::Pain), 2),
                Stock(K::Water, None, 0),
            ],
        }
    }

    /// What the Harbor's sergeant sells from the reef after the raid: the
    /// pirates' things, heavier now.
    #[must_use]
    pub const fn stock_after_raid() -> &'static [Stock] {
        use Kind as K;
        use Motive as M;
        &[
            Stock(K::Dagger, Some(M::Hate), 3),
            Stock(K::Flag, Some(M::Hate), 4),
            Stock(K::Pistol, Some(M::Zeal), 4),
            Stock(K::Coins, None, 0),
            Stock(K::Tin, None, 0),
            Stock(K::Water, None, 0),
            Stock(K::Water, None, 0),
            Stock(K::Water, None, 0),
        ]
    }
}

/// The lie of the land, one character per 16-pixel tile of the map.
///
/// The steppe's grass along the top (`g`) above the cliffs of the old shore
/// (`c`), open sand (`s`), dunes (`d`), the white salt flats (`w`), cracked
/// clay (`k`), the reef's red rock (`r`) and the Harbor's road (`o`).
pub const TERRAIN: [&str; 15] = [
    "ggggggggggggggggggggggggg",
    "ggggggggcccggggggggcccggg",
    "gggccccsssscccccssssssccg",
    "gccsssssssssssccssssswwcg",
    "gcsssddssssssssssssswwwws",
    "gcsssdddssssddsssssswwwss",
    "gcssssssssssdddssssssssss",
    "gcsssssssssssddsssskkssss",
    "gcssssssddssssssssskkksss",
    "gccsssssdddsssssssssssrrs",
    "ggcsssssssssssssdddssrrrs",
    "gccsskkkssssssssddsssrrrs",
    "gcsskkkkkssssssoosssssrss",
    "gcsssskkssssssoossssssssd",
    "ggcsssssssssssoosssssssdd",
];

/// The terrain under a map position.
#[must_use]
pub fn terrain_at(x: f32, y: f32) -> char {
    // Clamped into the map, so the cast is in range.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (col, row) = (
        (x / 16.0).clamp(0.0, 24.0) as usize,
        (y / 16.0).clamp(0.0, 14.0) as usize,
    );
    TERRAIN[row].as_bytes()[col] as char
}

/// One entry on a trader's rug: kind, motive, weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock(pub Kind, pub Option<Motive>, pub u8);

/// What you set out with, and where it sits in the hold: kind, motive,
/// weight, column, row, turned.
pub const START_HOLD: [(Kind, Option<Motive>, u8, u8, u8, bool); 12] = [
    (Kind::Water, None, 0, 0, 0, false),
    (Kind::Water, None, 0, 0, 1, false),
    (Kind::Water, None, 0, 0, 2, false),
    (Kind::Water, None, 0, 0, 3, false),
    (Kind::Coins, None, 0, 6, 0, false),
    (Kind::Salt, None, 0, 7, 0, false),
    (Kind::Flute, Some(Motive::Bliss), 1, 1, 0, false),
    (Kind::Rug, Some(Motive::Bliss), 0, 2, 0, false),
    (Kind::Comb, Some(Motive::Love), 2, 2, 2, false),
    (Kind::Beads, Some(Motive::Repose), 1, 4, 2, false),
    (Kind::Compass, None, 0, 5, 0, false),
    (Kind::Plank, None, 0, 5, 3, false),
];

/// Hands out item ids, so every thing in the basin has its own.
#[derive(Clone, Debug, Default)]
pub struct Ids(u16);

impl Ids {
    /// The next id.
    pub const fn next(&mut self) -> ItemId {
        self.0 += 1;
        ItemId(self.0)
    }
}

/// Lay `stock` out on a fresh rug, biggest things first.
#[must_use]
pub fn rug(stock: &[Stock], ids: &mut Ids) -> Grid {
    let (w, h) = STOCK_SIZE;
    let mut grid = Grid::new(w, h);
    let mut sorted: Vec<Stock> = stock.to_vec();
    sorted.sort_by_key(|s| {
        let (w, h) = s.0.size();
        std::cmp::Reverse(u16::from(w) * u16::from(h))
    });
    for Stock(kind, motive, weight) in sorted {
        let item = Item::new(ids.next(), kind, motive, weight);
        // Every rug is authored to fit; the test below checks.
        let _ = grid.stow(item);
    }
    grid
}

/// The hold you start with, every item's weight known.
#[must_use]
pub fn start_hold(ids: &mut Ids) -> Grid {
    let (w, h) = HOLD_SIZE;
    let mut grid = Grid::new(w, h);
    for (kind, motive, weight, x, y, turned) in START_HOLD {
        let mut item = Item::new(ids.next(), kind, motive, weight);
        item.known = true;
        let _ = grid.place(item, x, y, turned);
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rug_fits_everything_on_it() {
        let mut ids = Ids::default();
        for site in Site::ALL {
            let grid = rug(site.stock(), &mut ids);
            assert_eq!(grid.placed().len(), site.stock().len(), "{site:?}");
        }
        let grid = rug(Site::stock_after_raid(), &mut ids);
        assert_eq!(grid.placed().len(), Site::stock_after_raid().len());
    }

    #[test]
    fn the_start_hold_is_packed_as_written() {
        let hold = start_hold(&mut Ids::default());
        assert_eq!(hold.placed().len(), START_HOLD.len());
    }

    #[test]
    fn places_are_apart_and_on_the_map() {
        for (i, a) in Site::ALL.iter().enumerate() {
            let (ax, ay) = a.pos();
            assert!((16.0..MAP_W - 16.0).contains(&ax), "{a:?}");
            assert!((16.0..MAP_H - 16.0).contains(&ay), "{a:?}");
            assert_eq!(a.index(), i);
            for b in &Site::ALL[i + 1..] {
                let (bx, by) = b.pos();
                assert!((ax - bx).hypot(ay - by) > 40.0, "{a:?} crowds {b:?}");
            }
        }
    }

    #[test]
    fn the_terrain_covers_the_map() {
        for row in TERRAIN {
            assert_eq!(row.len(), 25);
            assert!(row.chars().all(|c| "gcsdwkro".contains(c)), "{row}");
        }
        assert!(TERRAIN.len() as f32 * 16.0 >= MAP_H);
        // Nobody lives on a cliff face, except the keeper.
        for site in Site::ALL {
            let (x, y) = site.pos();
            let ground = terrain_at(x, y);
            assert!(
                site == Site::Lighthouse || ground != 'c',
                "{site:?} is on a cliff"
            );
        }
    }

    #[test]
    fn every_motive_is_asked_for_somewhere_and_sold_somewhere() {
        for motive in Motive::ALL {
            assert!(Site::ALL.iter().any(|s| s.waystone() == Some(motive)));
            assert!(
                Site::ALL
                    .iter()
                    .any(|s| s.stock().iter().any(|st| st.1 == Some(motive))),
                "nobody sells {motive:?}"
            );
        }
    }
}
