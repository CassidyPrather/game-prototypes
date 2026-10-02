//! A grid of cells with items packed into it: the hold, or a trader's rug.
//!
//! Items take whole cells, may be turned when they are longer one way than
//! the other, and never overlap. Where they sit matters in the hold: each
//! day an item grows fonder by one, plus one for every neighbour bearing
//! the same motive (things that share a feeling feed it), minus one for
//! every neighbour bearing the opposite (they quarrel, and neither settles
//! in), plus one for every woken neighbour. A woken item also wanders a
//! cell each day if it can, spoiling whatever order you had.
//!
//! An item set out on a pan keeps its cells, so taking it back always fits;
//! the grid only lets go of it when a deal goes through.

use crate::sand_nomad::item::{Item, ItemId};

/// One item and where it sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub item: Item,
    /// Left column.
    pub x: u8,
    /// Top row.
    pub y: u8,
    /// Whether it lies turned a quarter from how it is drawn.
    pub turned: bool,
    /// Whether it is out on a pan, its cells kept for it.
    pub on_pan: bool,
}

impl Placed {
    /// Columns and rows it covers.
    #[must_use]
    pub const fn size(&self) -> (u8, u8) {
        self.item.footprint(self.turned)
    }

    /// Whether it covers cell `(x, y)`.
    #[must_use]
    pub const fn covers(&self, x: u8, y: u8) -> bool {
        let (w, h) = self.size();
        x >= self.x && x < self.x + w && y >= self.y && y < self.y + h
    }

    /// Whether it shares an edge with `other`.
    #[must_use]
    pub const fn touches(&self, other: &Self) -> bool {
        let (aw, ah) = self.size();
        let (bw, bh) = other.size();
        let (ax0, ay0, ax1, ay1) = (self.x, self.y, self.x + aw, self.y + ah);
        let (bx0, by0, bx1, by1) = (other.x, other.y, other.x + bw, other.y + bh);
        let overlap_x = ax0 < bx1 && bx0 < ax1;
        let overlap_y = ay0 < by1 && by0 < ay1;
        (overlap_y && (ax1 == bx0 || bx1 == ax0)) || (overlap_x && (ay1 == by0 || by1 == ay0))
    }
}

/// What a day did to one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub id: ItemId,
    /// Fondness it gathered today.
    pub fondness: u8,
    /// Points of weight that added.
    pub gained: u8,
    /// Whether it woke up today.
    pub woke: bool,
}

/// A woken item moving itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wander {
    pub id: ItemId,
    pub from: (u8, u8),
    pub to: (u8, u8),
}

/// How a thing moved in the hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// Into space that was free.
    Shifted,
    /// Into another thing's place, which took its old one.
    Swapped(ItemId),
}

/// The ways a woken item tries to move, clockwise from up.
const STEPS: [(i8, i8); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// A packed grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    w: u8,
    h: u8,
    placed: Vec<Placed>,
}

impl Grid {
    /// An empty grid of `w` columns and `h` rows.
    #[must_use]
    pub const fn new(w: u8, h: u8) -> Self {
        Self {
            w,
            h,
            placed: Vec::new(),
        }
    }

    /// Columns.
    #[must_use]
    pub const fn width(&self) -> u8 {
        self.w
    }

    /// Rows.
    #[must_use]
    pub const fn height(&self) -> u8 {
        self.h
    }

    /// Everything in it, in the order it arrived.
    #[must_use]
    pub fn placed(&self) -> &[Placed] {
        &self.placed
    }

    /// Everything in it that is not out on a pan.
    pub fn present(&self) -> impl Iterator<Item = &Placed> {
        self.placed.iter().filter(|p| !p.on_pan)
    }

    /// Everything out on a pan.
    pub fn out(&self) -> impl Iterator<Item = &Placed> {
        self.placed.iter().filter(|p| p.on_pan)
    }

    #[must_use]
    pub fn get(&self, id: ItemId) -> Option<&Placed> {
        self.placed.iter().find(|p| p.item.id == id)
    }

    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut Placed> {
        self.placed.iter_mut().find(|p| p.item.id == id)
    }

    /// What sits on cell `(x, y)`, pans included.
    #[must_use]
    pub fn at(&self, x: u8, y: u8) -> Option<&Placed> {
        self.placed.iter().find(|p| p.covers(x, y))
    }

    /// Whether `item` fits with its corner at `(x, y)`, pretending `ignore`
    /// is not there. Positions may be off the grid, which never fits.
    #[must_use]
    pub fn fits(&self, item: &Item, x: i32, y: i32, turned: bool, ignore: Option<ItemId>) -> bool {
        let (w, h) = item.footprint(turned);
        if x < 0
            || y < 0
            || x + i32::from(w) > i32::from(self.w)
            || y + i32::from(h) > i32::from(self.h)
        {
            return false;
        }
        // In range by the check above.
        #[allow(clippy::cast_sign_loss)]
        let ghost = Placed {
            item: *item,
            x: x as u8,
            y: y as u8,
            turned,
            on_pan: false,
        };
        self.placed.iter().all(|p| {
            Some(p.item.id) == ignore || {
                let (pw, ph) = p.size();
                let (gw, gh) = ghost.size();
                ghost.x + gw <= p.x
                    || p.x + pw <= ghost.x
                    || ghost.y + gh <= p.y
                    || p.y + ph <= ghost.y
            }
        })
    }

    /// Put `item` at `(x, y)`, or hand it back if it does not fit.
    pub fn place(&mut self, item: Item, x: u8, y: u8, turned: bool) -> Result<(), Item> {
        if self.fits(&item, i32::from(x), i32::from(y), turned, None) {
            self.placed.push(Placed {
                item,
                x,
                y,
                turned,
                on_pan: false,
            });
            Ok(())
        } else {
            Err(item)
        }
    }

    /// The first spot `item` fits, reading left to right and top to bottom,
    /// either way round, as if the items in `freed` were already gone.
    #[must_use]
    pub fn first_fit(&self, item: &Item, freed: &[ItemId]) -> Option<(u8, u8, bool)> {
        let spare = Self {
            w: self.w,
            h: self.h,
            placed: self
                .placed
                .iter()
                .filter(|p| !freed.contains(&p.item.id))
                .copied()
                .collect(),
        };
        let turns: &[bool] = if item.can_turn() {
            &[false, true]
        } else {
            &[false]
        };
        for y in 0..self.h {
            for x in 0..self.w {
                for &turned in turns {
                    if spare.fits(item, i32::from(x), i32::from(y), turned, None) {
                        return Some((x, y, turned));
                    }
                }
            }
        }
        None
    }

    /// Put `item` wherever it first fits.
    pub fn stow(&mut self, item: Item) -> Result<(), Item> {
        match self.first_fit(&item, &[]) {
            Some((x, y, turned)) => self.place(item, x, y, turned),
            None => Err(item),
        }
    }

    /// Whether every item in `incoming` can be stowed at once, as if the
    /// items in `freed` were gone.
    #[must_use]
    pub fn room_for(&self, incoming: &[Item], freed: &[ItemId]) -> bool {
        let mut trial = Self {
            w: self.w,
            h: self.h,
            placed: self
                .placed
                .iter()
                .filter(|p| !freed.contains(&p.item.id))
                .copied()
                .collect(),
        };
        // Biggest first, the way anyone packs a bag.
        let mut sorted: Vec<Item> = incoming.to_vec();
        sorted.sort_by_key(|i| {
            let (w, h) = i.kind.size();
            std::cmp::Reverse(u16::from(w) * u16::from(h))
        });
        sorted.into_iter().all(|item| trial.stow(item).is_ok())
    }

    /// Move `id` to `(x, y)`, turned or not. Fails without moving it.
    pub fn shift(&mut self, id: ItemId, x: i32, y: i32, turned: bool) -> bool {
        let Some(p) = self.get(id) else {
            return false;
        };
        if p.on_pan || !self.fits(&p.item, x, y, turned, Some(id)) {
            return false;
        }
        if let Some(p) = self.get_mut(id) {
            // In range: `fits` checked it.
            #[allow(clippy::cast_sign_loss)]
            {
                p.x = x as u8;
                p.y = y as u8;
            }
            p.turned = turned;
        }
        true
    }

    /// Move `id` to `(x, y)`, and if exactly one other thing is in the way
    /// and fits where `id` was, swap them. `None` if neither works.
    pub fn shift_or_swap(&mut self, id: ItemId, x: i32, y: i32, turned: bool) -> Option<Moved> {
        if self.shift(id, x, y, turned) {
            return Some(Moved::Shifted);
        }
        let me = self.get(id).copied()?;
        if me.on_pan || x < 0 || y < 0 {
            return None;
        }
        let (w, h) = me.item.footprint(turned);
        // In range: checked non-negative above, and grids are small.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ghost = Placed {
            item: me.item,
            x: x as u8,
            y: y as u8,
            turned,
            on_pan: false,
        };
        let (gw, gh) = (w, h);
        let blockers: Vec<Placed> = self
            .placed
            .iter()
            .filter(|p| p.item.id != id)
            .filter(|p| {
                let (pw, ph) = p.size();
                !(ghost.x + gw <= p.x
                    || p.x + pw <= ghost.x
                    || ghost.y + gh <= p.y
                    || p.y + ph <= ghost.y)
            })
            .copied()
            .collect();
        let [other] = blockers.as_slice() else {
            return None;
        };
        if other.on_pan {
            return None;
        }
        // Try it: take both out, put `id` in its new place, and see whether
        // the other fits where `id` was.
        let saved = self.placed.clone();
        self.placed
            .retain(|p| p.item.id != id && p.item.id != other.item.id);
        let placed_me = self.fits(&me.item, x, y, turned, None);
        if placed_me {
            self.placed.push(ghost);
            if self.fits(
                &other.item,
                i32::from(me.x),
                i32::from(me.y),
                other.turned,
                None,
            ) {
                self.placed.push(Placed {
                    x: me.x,
                    y: me.y,
                    ..*other
                });
                return Some(Moved::Swapped(other.item.id));
            }
        }
        self.placed = saved;
        None
    }

    /// Take `id` out of the grid altogether.
    pub fn take(&mut self, id: ItemId) -> Option<Item> {
        let at = self.placed.iter().position(|p| p.item.id == id)?;
        Some(self.placed.remove(at).item)
    }

    /// Everything sharing an edge with `id`.
    #[must_use]
    pub fn neighbours(&self, id: ItemId) -> Vec<&Placed> {
        let Some(me) = self.get(id) else {
            return Vec::new();
        };
        self.placed
            .iter()
            .filter(|p| p.item.id != id && p.touches(me))
            .collect()
    }

    /// The fondness `id` would gather in a day where it sits now.
    #[must_use]
    pub fn fondness_for(&self, id: ItemId) -> u8 {
        let Some(me) = self.get(id) else {
            return 0;
        };
        let Some(motive) = me.item.motive else {
            return 0;
        };
        let mut total: i32 = 1;
        for n in self.neighbours(id) {
            if let Some(theirs) = n.item.motive {
                if theirs == motive {
                    total += 1;
                } else if theirs == motive.opposite() {
                    total -= 1;
                }
            }
            if n.item.is_anima() {
                total += 1;
            }
        }
        // Clamped to a small non-negative count.
        #[allow(clippy::cast_sign_loss)]
        let fondness = total.clamp(0, 9) as u8;
        fondness
    }

    /// One day in the hold: every item gathers its fondness, all at once,
    /// then anything awake wanders. `day` picks which way each one tries
    /// first, so the same hold on the same day does the same thing.
    pub fn pass_day(&mut self, day: u32) -> (Vec<Growth>, Vec<Wander>) {
        let fondness: Vec<(ItemId, u8)> = self
            .placed
            .iter()
            .map(|p| (p.item.id, self.fondness_for(p.item.id)))
            .collect();
        let mut growth = Vec::new();
        for (id, amount) in fondness {
            if let Some(p) = self.get_mut(id) {
                if !p.item.can_grow() {
                    continue;
                }
                let was_awake = p.item.is_anima();
                let gained = p.item.gather(amount);
                growth.push(Growth {
                    id,
                    fondness: amount,
                    gained,
                    woke: !was_awake && p.item.is_anima(),
                });
            }
        }
        let mut wanders = Vec::new();
        let awake: Vec<ItemId> = self
            .placed
            .iter()
            .filter(|p| p.item.is_anima() && !p.on_pan)
            .map(|p| p.item.id)
            .collect();
        for id in awake {
            let Some(p) = self.get(id).copied() else {
                continue;
            };
            let first = (day as usize + usize::from(id.0)) % STEPS.len();
            for turn in 0..STEPS.len() {
                let (dx, dy) = STEPS[(first + turn) % STEPS.len()];
                let (x, y) = (
                    i32::from(p.x) + i32::from(dx),
                    i32::from(p.y) + i32::from(dy),
                );
                if self.shift(id, x, y, p.turned) {
                    // In range: the shift succeeded.
                    #[allow(clippy::cast_sign_loss)]
                    wanders.push(Wander {
                        id,
                        from: (p.x, p.y),
                        to: (x as u8, y as u8),
                    });
                    break;
                }
            }
        }
        (growth, wanders)
    }

    /// Mark `id` as out on a pan, or back.
    pub fn set_out(&mut self, id: ItemId, out: bool) -> bool {
        self.get_mut(id).is_some_and(|p| {
            let changed = p.on_pan != out;
            p.on_pan = out;
            changed
        })
    }

    /// Bring everything back from the pans.
    pub fn recall(&mut self) {
        for p in &mut self.placed {
            p.on_pan = false;
        }
    }

    /// Change an item in place.
    pub fn update(&mut self, id: ItemId, change: impl FnOnce(&mut Item)) {
        if let Some(p) = self.get_mut(id) {
            change(&mut p.item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sand_nomad::item::Kind;
    use crate::sand_nomad::motive::Motive;

    fn item(id: u16, kind: Kind, motive: Option<Motive>, weight: u8) -> Item {
        Item::new(ItemId(id), kind, motive, weight)
    }

    #[test]
    fn items_do_not_overlap_or_leave_the_grid() {
        let mut grid = Grid::new(4, 2);
        assert!(grid.place(item(1, Kind::Rug, None, 0), 0, 0, false).is_ok());
        assert!(
            grid.place(item(2, Kind::Comb, None, 0), 1, 0, false)
                .is_err()
        );
        assert!(
            grid.place(item(3, Kind::Comb, None, 0), 2, 0, false)
                .is_ok()
        );
        assert!(
            grid.place(item(4, Kind::Rug, None, 0), 3, 0, false)
                .is_err()
        );
        assert!(grid.place(item(5, Kind::Rug, None, 0), 3, 0, true).is_ok());
        assert!(!grid.fits(&item(6, Kind::Comb, None, 0), -1, 0, false, None));
        assert!(!grid.fits(&item(6, Kind::Comb, None, 0), 0, 2, false, None));
    }

    #[test]
    fn shifting_ignores_the_item_itself() {
        let mut grid = Grid::new(4, 1);
        grid.place(item(1, Kind::Rug, None, 0), 0, 0, false)
            .unwrap();
        assert!(
            grid.shift(ItemId(1), 1, 0, false),
            "sliding over its own cells"
        );
        assert_eq!(grid.get(ItemId(1)).unwrap().x, 1);
        assert!(!grid.shift(ItemId(1), 1, 0, true), "too short to stand up");
    }

    #[test]
    fn dropping_on_a_neighbour_swaps_when_both_fit() {
        let mut grid = Grid::new(3, 1);
        grid.place(item(1, Kind::Comb, None, 0), 0, 0, false)
            .unwrap();
        grid.place(item(2, Kind::Shell, None, 0), 2, 0, false)
            .unwrap();
        assert_eq!(
            grid.shift_or_swap(ItemId(1), 2, 0, false),
            Some(Moved::Swapped(ItemId(2)))
        );
        assert_eq!(
            (
                grid.get(ItemId(1)).unwrap().x,
                grid.get(ItemId(2)).unwrap().x
            ),
            (2, 0)
        );
        // A rug landing on two things swaps with neither.
        let mut grid = Grid::new(3, 2);
        grid.place(item(1, Kind::Rug, None, 0), 0, 0, false)
            .unwrap();
        grid.place(item(2, Kind::Comb, None, 0), 2, 0, false)
            .unwrap();
        grid.place(item(3, Kind::Comb, None, 0), 0, 1, false)
            .unwrap();
        grid.place(item(4, Kind::Comb, None, 0), 1, 1, false)
            .unwrap();
        grid.place(item(5, Kind::Comb, None, 0), 2, 1, false)
            .unwrap();
        assert_eq!(
            grid.shift_or_swap(ItemId(3), 2, 0, false),
            Some(Moved::Swapped(ItemId(2)))
        );
        assert_eq!(grid.shift_or_swap(ItemId(1), 1, 1, false), None);
        assert_eq!(grid.placed().len(), 5, "a failed swap loses nothing");
        assert_eq!(grid.get(ItemId(1)).unwrap().x, 0);
    }

    #[test]
    fn neighbours_share_an_edge_not_a_corner() {
        let mut grid = Grid::new(3, 3);
        grid.place(item(1, Kind::Comb, None, 0), 1, 1, false)
            .unwrap();
        grid.place(item(2, Kind::Comb, None, 0), 0, 0, false)
            .unwrap();
        grid.place(item(3, Kind::Comb, None, 0), 1, 0, false)
            .unwrap();
        grid.place(item(4, Kind::Rug, None, 0), 0, 2, false)
            .unwrap();
        let ids: Vec<u16> = grid
            .neighbours(ItemId(1))
            .iter()
            .map(|p| p.item.id.0)
            .collect();
        assert_eq!(ids, vec![3, 4]);
    }

    #[test]
    fn like_feeds_like_and_opposites_quarrel() {
        let mut grid = Grid::new(4, 1);
        grid.place(item(1, Kind::Comb, Some(Motive::Love), 0), 0, 0, false)
            .unwrap();
        grid.place(item(2, Kind::Locket, Some(Motive::Love), 0), 1, 0, false)
            .unwrap();
        grid.place(item(3, Kind::Dagger, Some(Motive::Hate), 0), 2, 0, false)
            .unwrap();
        grid.place(item(4, Kind::Water, None, 0), 3, 0, false)
            .unwrap();
        assert_eq!(grid.fondness_for(ItemId(1)), 2, "one friend");
        assert_eq!(grid.fondness_for(ItemId(2)), 1, "a friend and a quarrel");
        assert_eq!(grid.fondness_for(ItemId(3)), 0, "a quarrel and water");
        assert_eq!(grid.fondness_for(ItemId(4)), 0, "water feels nothing");
    }

    #[test]
    fn a_woken_item_wanders_and_excites_its_neighbours() {
        let mut grid = Grid::new(3, 1);
        grid.place(item(1, Kind::Comb, Some(Motive::Love), 6), 0, 0, false)
            .unwrap();
        grid.place(item(2, Kind::Shell, Some(Motive::Bliss), 0), 1, 0, false)
            .unwrap();
        assert_eq!(grid.fondness_for(ItemId(2)), 2);
        let (_, wanders) = grid.pass_day(0);
        // Boxed in on the left: it cannot move up, right, down or left.
        assert_eq!(wanders, []);
        let mut grid = Grid::new(3, 2);
        grid.place(item(1, Kind::Comb, Some(Motive::Love), 6), 0, 0, false)
            .unwrap();
        let (_, wanders) = grid.pass_day(0);
        assert_eq!(wanders.len(), 1);
        assert_ne!(wanders[0].from, wanders[0].to);
    }

    #[test]
    fn a_day_grows_everything_at_once() {
        let mut grid = Grid::new(2, 1);
        grid.place(item(1, Kind::Comb, Some(Motive::Love), 5), 0, 0, false)
            .unwrap();
        grid.update(ItemId(1), |i| i.fondness = 3);
        grid.place(item(2, Kind::Locket, Some(Motive::Love), 0), 1, 0, false)
            .unwrap();
        let (growth, _) = grid.pass_day(0);
        let comb = growth.iter().find(|g| g.id == ItemId(1)).unwrap();
        assert!(comb.woke);
        // The locket's fondness was worked out before the comb woke.
        let locket = growth.iter().find(|g| g.id == ItemId(2)).unwrap();
        assert_eq!(locket.fondness, 2);
    }

    #[test]
    fn room_for_counts_what_is_leaving() {
        let mut grid = Grid::new(2, 1);
        grid.place(item(1, Kind::Comb, None, 0), 0, 0, false)
            .unwrap();
        grid.place(item(2, Kind::Comb, None, 0), 1, 0, false)
            .unwrap();
        let rug = [item(3, Kind::Rug, None, 0)];
        assert!(!grid.room_for(&rug, &[]));
        assert!(!grid.room_for(&rug, &[ItemId(1)]));
        assert!(grid.room_for(&rug, &[ItemId(1), ItemId(2)]));
    }
}
