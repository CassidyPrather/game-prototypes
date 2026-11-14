//! The boulder field: the cavern Normal is played in, and the one the original
//! 7DRL generated for every difficulty.
//!
//! An open box with a handful of rectangular blocks of rock dropped into it
//! (`PORT_SPEC.md` §2.2). Up to [`BOULDER_TRIES`] blocks are offered, each 7 to 12
//! cells a side, and a block that would overlap one already placed is thrown
//! away, so a 48 by 48 map ends up with five to eight of them. Floor is
//! everything the blocks did not land on. There are no rooms and no corridors:
//! there is one wide field with rocks standing in it, an open lane round the
//! walls, and always a way round any block.
//!
//! Only the carving is the original's. Everything the original did afterwards
//! — the elbow tunnels that could leave a pocket stranded, the gate search
//! that could spin forever, scattering everything by rejection sampling — is
//! the shared generator's job, so the gates are sunk into the blocks at the end
//! of their own passages like everywhere else.

use std::collections::VecDeque;

use super::Cavern;
use crate::amoeba::sim::Sim;
use crate::amoeba::sim::grid::{Coord, rand_inclusive};

/// Blocks offered. Most of the later ones land on an earlier one and are
/// thrown away.
const BOULDER_TRIES: u32 = 20;
/// Smallest and largest nominal side of a block, inclusive. The original's
/// `Rand.Next(7, 13)`, whose upper bound is exclusive.
const BOULDER_SIDE: (i32, i32) = (7, 12);
/// How far apart two hubs are kept, taxicab.
///
/// The field has no chambers to anchor the humans' beats on, so the hubs are
/// the open places between the blocks instead, and this keeps two of them
/// from naming the same open place.
const HUB_GAP: i32 = 10;
/// Most hubs one field is given.
const HUBS_MAX: usize = 8;
/// Fewest hubs a field needs before it is worth playing.
pub(super) const HUBS_MIN: usize = 3;

/// A block of rock: `x..=x + w` by `y..=y + h`.
///
/// Inclusive at both ends because the original's was. `RogueSharp`'s
/// `Rectangle.Right` is `X + Width` and the fill loop ran to it with `<=`, so a
/// block of nominal size w by h blanked w + 1 by h + 1 cells.
#[derive(Clone, Copy, Debug)]
struct Block {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Block {
    /// Whether two blocks overlap, the way `RogueSharp`'s `Intersects` decided
    /// it: strictly, on the nominal edges. Two blocks that only share an edge
    /// pass, and since each fills one cell past its nominal edge, they fuse
    /// into one bigger block of rock — which is a shape the original made too.
    const fn overlaps(self, other: Self) -> bool {
        other.x < self.x + self.w
            && self.x < other.x + other.w
            && other.y < self.y + self.h
            && self.y < other.y + other.h
    }
}

impl Sim {
    /// Clear the map's interior and drop blocks of rock into it.
    ///
    /// `None` means the field came out with too few open places to anchor the
    /// humans on, which is the retry loop's cue to throw the map away.
    pub(super) fn carve_boulders(&mut self) -> Option<Cavern> {
        let (width, height) = (self.grid.width(), self.grid.height());
        for y in 0..height {
            for x in 0..width {
                self.carve(Coord::new(x, y));
            }
        }
        let mut blocks: Vec<Block> = Vec::new();
        for _ in 0..BOULDER_TRIES {
            let w = rand_inclusive(&mut self.rng, BOULDER_SIDE.0, BOULDER_SIDE.1);
            let h = rand_inclusive(&mut self.rng, BOULDER_SIDE.0, BOULDER_SIDE.1);
            // `Rand.Next(0, width - w - 1)`: exclusive above, which keeps the
            // block's far edge off the border.
            let x = rand_inclusive(&mut self.rng, 0, width - w - 2);
            let y = rand_inclusive(&mut self.rng, 0, height - h - 2);
            let block = Block { x, y, w, h };
            if blocks.iter().all(|placed| !placed.overlaps(block)) {
                blocks.push(block);
            }
        }
        for block in blocks {
            for y in block.y..=block.y + block.h {
                for x in block.x..=block.x + block.w {
                    self.grid.set_props(Coord::new(x, y), false, false, false);
                }
            }
        }
        let hubs = self.open_places();
        if hubs.len() < HUBS_MIN {
            return None;
        }
        // Nothing here is tightened afterwards, so nothing here has to be
        // exempted from the tightening.
        let open_ground = vec![false; self.depth.len()];
        Some(Cavern { hubs, open_ground })
    }

    /// The middles of the field's open places: the cells furthest from any
    /// rock, taken greedily and kept [`HUB_GAP`] apart.
    fn open_places(&mut self) -> Vec<Coord> {
        let cells = self.depth.len();
        let mut clearance = vec![i32::MAX; cells];
        let mut queue = VecDeque::new();
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                let c = Coord::new(x, y);
                if !self.grid.walkable(c)
                    && let Some(i) = self.cell_index(c)
                {
                    clearance[i] = 0;
                    queue.push_back(c);
                }
            }
        }
        while let Some(at) = queue.pop_front() {
            let Some(here) = self.cell_index(at).map(|i| clearance[i]) else {
                continue;
            };
            for next in self.grid.adjacent(at) {
                if let Some(i) = self.cell_index(next)
                    && clearance[i] == i32::MAX
                {
                    clearance[i] = here + 1;
                    queue.push_back(next);
                }
            }
        }
        let mut floor: Vec<(Coord, i32)> = (0..self.grid.height())
            .flat_map(|y| (0..self.grid.width()).map(move |x| Coord::new(x, y)))
            .filter(|c| self.grid.walkable(*c))
            .filter_map(|c| self.cell_index(c).map(|i| (c, clearance[i])))
            .collect();
        // Shuffle before the stable sort so that ties between equally open
        // cells fall to the seed rather than to reading order.
        self.shuffle(&mut floor);
        floor.sort_by_key(|(_, clear)| std::cmp::Reverse(*clear));
        let mut hubs: Vec<Coord> = Vec::new();
        for (c, _) in floor {
            if hubs.len() >= HUBS_MAX {
                break;
            }
            if hubs.iter().all(|hub| hub.taxi(c) >= HUB_GAP) {
                hubs.push(c);
            }
        }
        hubs
    }
}
