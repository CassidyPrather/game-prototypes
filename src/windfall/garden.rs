//! The rules of Windfall: a garden, the leaves its tree lets go, and the
//! vanes that blow them home.
//!
//! A garden is a small grid. Above it a tree holds a few leaves, in the
//! order it will drop them; below it stand baskets, one colour each. A leaf
//! falls one cell a tick. A vane in its cell blows it one cell sideways
//! instead, and a flip vane does the same and then turns to face the other
//! way. A leaf that drops below the last row lands in whatever is under it:
//! home if that is a basket of its colour, lost if not. A leaf blown off
//! either side is lost, and so is one that goes round and round.
//!
//! Every garden can be won with vanes to spare. The game is in winning it
//! with few pieces and little wind, which is what the marks count.
//!
//! Pure and deterministic: [`Run::step`] is one tick, and nothing here
//! reads a clock or rolls a die.

// Coordinates are small `i8`s so a cell is two bytes; every cast to an
// index below follows a check that the cell is inside the garden, so none
// is ever negative.
#![allow(clippy::cast_sign_loss)]

/// A place in the garden. Row 0 is just under the tree; `y == height` is
/// the ground the baskets stand on, and `y == -1` the branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub x: i8,
    pub y: i8,
}

impl Cell {
    #[must_use]
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

/// Which way a vane blows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dir {
    Left,
    Right,
}

impl Dir {
    #[must_use]
    pub const fn dx(self) -> i8 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
        }
    }

    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Something standing in a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    /// Blows every leaf that enters this way.
    Vane(Dir),
    /// Blows a leaf the way it faces, then turns round. The direction is
    /// the one it starts the garden facing.
    Flip(Dir),
}

impl Piece {
    /// What a click on a cell holding this (or nothing) turns it into, in
    /// a garden whose kit is `kit`. Nothing, then each vane, then each flip
    /// vane, then nothing again.
    #[must_use]
    pub fn next(this: Option<Self>, kit: Kit) -> Option<Self> {
        let order = kit.order();
        let at = order.iter().position(|&p| p == this).unwrap_or(0);
        order[(at + 1) % order.len()]
    }

    /// The other way round: a right click, or the piece before.
    #[must_use]
    pub fn previous(this: Option<Self>, kit: Kit) -> Option<Self> {
        let order = kit.order();
        let at = order.iter().position(|&p| p == this).unwrap_or(0);
        order[(at + order.len() - 1) % order.len()]
    }
}

/// Which pieces a garden lets the player place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kit {
    /// Vanes only.
    Vanes,
    /// Vanes and flip vanes.
    Flips,
}

impl Kit {
    const fn order(self) -> &'static [Option<Piece>] {
        match self {
            Self::Vanes => &[
                None,
                Some(Piece::Vane(Dir::Right)),
                Some(Piece::Vane(Dir::Left)),
            ],
            Self::Flips => &[
                None,
                Some(Piece::Vane(Dir::Right)),
                Some(Piece::Vane(Dir::Left)),
                Some(Piece::Flip(Dir::Right)),
                Some(Piece::Flip(Dir::Left)),
            ],
        }
    }

    /// Every piece the kit holds.
    pub fn pieces(self) -> impl Iterator<Item = Piece> {
        self.order().iter().filter_map(|&p| p)
    }
}

/// A leaf's colour. Autumn's three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hue {
    Ember,
    Gold,
    Rust,
}

/// One leaf the tree will drop: when, from over which column, and what
/// colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub tick: u16,
    pub x: i8,
    pub hue: Hue,
}

/// A basket on the ground under column `x`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Basket {
    pub x: i8,
    pub hue: Hue,
}

/// One hand-made garden.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub width: i8,
    pub height: i8,
    /// In the order the tree lets them go; ticks never decrease.
    pub drops: &'static [Drop],
    pub baskets: &'static [Basket],
    /// Pieces the garden came with. The player cannot move them, and they
    /// do not count against the player.
    pub fixed: &'static [(Cell, Piece)],
    /// Cells nothing can be placed in. Leaves pass over them.
    pub stones: &'static [Cell],
    pub kit: Kit,
    /// The fewest pieces that bring every leaf home. Proven by
    /// [`super::solve`], in the levels' tests.
    pub par_pieces: u8,
    /// The least wind (sideways cells, summed over every leaf) any
    /// solution the search found needs.
    pub par_gusts: u16,
}

impl Level {
    #[must_use]
    pub const fn inside(&self, at: Cell) -> bool {
        at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < self.height
    }

    /// Where `at` is in a row-major grid of the garden, if it is in it.
    #[must_use]
    pub fn index(&self, at: Cell) -> Option<usize> {
        self.inside(at)
            .then(|| at.y as usize * self.width as usize + at.x as usize)
    }

    #[must_use]
    pub const fn cells(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[must_use]
    pub fn basket(&self, x: i8) -> Option<Basket> {
        self.baskets.iter().copied().find(|b| b.x == x)
    }

    #[must_use]
    pub fn is_fixed(&self, at: Cell) -> bool {
        self.fixed.iter().any(|&(c, _)| c == at)
    }

    #[must_use]
    pub fn is_stone(&self, at: Cell) -> bool {
        self.stones.contains(&at)
    }

    /// Whether the player may put a piece here.
    #[must_use]
    pub fn open(&self, at: Cell) -> bool {
        self.inside(at) && !self.is_fixed(at) && !self.is_stone(at)
    }

    /// A tick by which any leaf still in the air must be going round in
    /// circles: long enough to cross every cell of the garden twice.
    #[must_use]
    pub const fn patience(&self) -> u16 {
        (self.cells() as u16) * 2 + 4
    }
}

/// What the player has put in the garden: one optional piece per cell,
/// row-major. Fixed pieces are not in it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Plan {
    width: i8,
    cells: Vec<Option<Piece>>,
}

impl Plan {
    #[must_use]
    pub fn empty(level: &Level) -> Self {
        Self {
            width: level.width,
            cells: vec![None; level.cells()],
        }
    }

    #[must_use]
    pub fn get(&self, at: Cell) -> Option<Piece> {
        let i = self.at(at)?;
        self.cells[i]
    }

    /// Put `piece` at `at` (or clear it with `None`). Returns whether
    /// anything changed: a cell outside the garden, or one the player may
    /// not touch, is left alone.
    pub fn set(&mut self, level: &Level, at: Cell, piece: Option<Piece>) -> bool {
        if !level.open(at) {
            return false;
        }
        let Some(i) = self.at(at) else {
            return false;
        };
        let changed = self.cells[i] != piece;
        self.cells[i] = piece;
        changed
    }

    fn at(&self, at: Cell) -> Option<usize> {
        let height = (self.cells.len() / self.width as usize) as i8;
        (at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < height)
            .then(|| at.y as usize * self.width as usize + at.x as usize)
    }

    /// How many pieces the player has placed.
    #[must_use]
    pub fn count(&self) -> usize {
        self.cells.iter().flatten().count()
    }

    /// Every placed piece and where.
    pub fn pieces(&self) -> impl Iterator<Item = (Cell, Piece)> + '_ {
        let width = self.width as usize;
        self.cells.iter().enumerate().filter_map(move |(i, p)| {
            p.map(|p| (Cell::new((i % width) as i8, (i / width) as i8), p))
        })
    }

    pub fn clear(&mut self) {
        self.cells.fill(None);
    }
}

/// Why a leaf did not get home.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loss {
    /// It landed in a basket of another colour.
    WrongBasket,
    /// It landed on the bare ground.
    Ground,
    /// It was blown out of the garden.
    Blown,
    /// It went round and round until the garden gave up on it.
    Eddy,
}

/// Where a leaf is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// Still on the branch.
    Hanging,
    /// Falling, or being blown.
    Aloft,
    /// In a basket of its colour, since this tick.
    Home(u16),
    /// Lost, and how, since this tick.
    Lost(Loss, u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leaf {
    pub hue: Hue,
    pub at: Cell,
    pub fate: Fate,
    /// The tick it last moved, for the frontend's tweening.
    pub from: Cell,
}

impl Leaf {
    #[must_use]
    pub const fn settled(&self) -> bool {
        matches!(self.fate, Fate::Home(_) | Fate::Lost(..))
    }
}

/// What happened during one tick, for sound and motion. A cue says what
/// happened, never what it should look or sound like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// The tree let leaf `leaf` go.
    Dropped { leaf: usize },
    /// Leaf `leaf` was blown sideways out of `from`, by a flip vane if
    /// `flip`.
    Gust { leaf: usize, from: Cell, flip: bool },
    /// The flip vane at `at` turned to face `now`.
    Turned { at: Cell, now: Dir },
    /// Leaf `leaf` reached a basket of its colour.
    Home { leaf: usize },
    /// Leaf `leaf` is lost.
    Lost { leaf: usize, loss: Loss },
    /// Every leaf has settled; `all_home` if none was lost.
    Over { all_home: bool },
}

/// The garden playing out, one tick at a time.
#[derive(Clone, Debug)]
pub struct Run {
    level: Level,
    /// Fixed pieces and the plan together, row-major.
    grid: Vec<Option<Piece>>,
    /// Which way each flip vane faces now (meaningless elsewhere).
    facing: Vec<Dir>,
    leaves: Vec<Leaf>,
    tick: u16,
    gusts: u16,
    pieces: usize,
    cues: Vec<Cue>,
    over: bool,
}

impl Run {
    #[must_use]
    pub fn new(level: &Level, plan: &Plan) -> Self {
        let mut grid = plan.cells.clone();
        for &(at, piece) in level.fixed {
            if let Some(i) = level.index(at) {
                grid[i] = Some(piece);
            }
        }
        let facing = grid
            .iter()
            .map(|p| match p {
                Some(Piece::Flip(dir) | Piece::Vane(dir)) => *dir,
                None => Dir::Right,
            })
            .collect();
        let leaves = level
            .drops
            .iter()
            .map(|d| Leaf {
                hue: d.hue,
                at: Cell::new(d.x, -1),
                from: Cell::new(d.x, -1),
                fate: Fate::Hanging,
            })
            .collect();
        Self {
            level: *level,
            grid,
            facing,
            leaves,
            tick: 0,
            gusts: 0,
            pieces: plan.count(),
            cues: Vec::new(),
            over: level.drops.is_empty(),
        }
    }

    #[must_use]
    pub const fn level(&self) -> &Level {
        &self.level
    }

    #[must_use]
    pub fn leaves(&self) -> &[Leaf] {
        &self.leaves
    }

    /// Ticks played so far.
    #[must_use]
    pub const fn tick(&self) -> u16 {
        self.tick
    }

    /// Sideways cells blown so far, summed over every leaf.
    #[must_use]
    pub const fn gusts(&self) -> u16 {
        self.gusts
    }

    /// Pieces the player placed.
    #[must_use]
    pub const fn pieces(&self) -> usize {
        self.pieces
    }

    #[must_use]
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    #[must_use]
    pub const fn over(&self) -> bool {
        self.over
    }

    #[must_use]
    pub fn all_home(&self) -> bool {
        self.over && self.leaves.iter().all(|l| matches!(l.fate, Fate::Home(_)))
    }

    /// The piece standing at `at` now, facing the way it faces now.
    #[must_use]
    pub fn piece(&self, at: Cell) -> Option<Piece> {
        let i = self.level.index(at)?;
        self.grid[i].map(|p| match p {
            Piece::Vane(dir) => Piece::Vane(dir),
            Piece::Flip(_) => Piece::Flip(self.facing[i]),
        })
    }

    /// Play one tick. Leaves move in the order the tree drops them, so two
    /// leaves through one flip vane in the same tick turn it twice.
    pub fn step(&mut self) {
        self.cues.clear();
        if self.over {
            return;
        }
        let tick = self.tick;
        let patience = self.level.patience();
        for n in 0..self.leaves.len() {
            let leaf = self.leaves[n];
            leaf_from(&mut self.leaves[n]);
            match leaf.fate {
                Fate::Hanging => {
                    if self.level.drops[n].tick <= tick {
                        let to = Cell::new(leaf.at.x, 0);
                        self.leaves[n].at = to;
                        self.leaves[n].fate = Fate::Aloft;
                        self.cues.push(Cue::Dropped { leaf: n });
                    }
                }
                Fate::Aloft => {
                    let drop_tick = self.level.drops[n].tick;
                    if tick.saturating_sub(drop_tick) > patience {
                        self.lose(n, Loss::Eddy);
                        continue;
                    }
                    let i = self.level.index(leaf.at).expect("an aloft leaf is inside");
                    let to = match self.grid[i] {
                        None => Cell::new(leaf.at.x, leaf.at.y + 1),
                        Some(piece) => {
                            let (dir, flip) = match piece {
                                Piece::Vane(dir) => (dir, false),
                                Piece::Flip(_) => (self.facing[i], true),
                            };
                            self.gusts += 1;
                            self.cues.push(Cue::Gust {
                                leaf: n,
                                from: leaf.at,
                                flip,
                            });
                            if flip {
                                self.facing[i] = dir.flipped();
                                self.cues.push(Cue::Turned {
                                    at: leaf.at,
                                    now: self.facing[i],
                                });
                            }
                            Cell::new(leaf.at.x + dir.dx(), leaf.at.y)
                        }
                    };
                    self.leaves[n].at = to;
                    if to.x < 0 || to.x >= self.level.width {
                        self.lose(n, Loss::Blown);
                    } else if to.y >= self.level.height {
                        match self.level.basket(to.x) {
                            Some(b) if b.hue == leaf.hue => {
                                self.leaves[n].fate = Fate::Home(tick);
                                self.cues.push(Cue::Home { leaf: n });
                            }
                            Some(_) => self.lose(n, Loss::WrongBasket),
                            None => self.lose(n, Loss::Ground),
                        }
                    }
                }
                Fate::Home(_) | Fate::Lost(..) => {}
            }
        }
        self.tick += 1;
        if self.leaves.iter().all(Leaf::settled) {
            self.over = true;
            let all_home = self.all_home();
            self.cues.push(Cue::Over { all_home });
        }
    }

    fn lose(&mut self, n: usize, loss: Loss) {
        self.leaves[n].fate = Fate::Lost(loss, self.tick);
        self.cues.push(Cue::Lost { leaf: n, loss });
    }

    /// Play to the end.
    pub fn finish(&mut self) {
        while !self.over {
            self.step();
        }
    }

    /// Every cell a leaf has stood in before the first leaf was lost, or
    /// in the whole run if none was; the cells whose contents decided what
    /// happened. Runs the garden to find out. The search leans on this:
    /// changing any other cell cannot save that first lost leaf.
    #[must_use]
    pub fn deciding_cells(level: &Level, plan: &Plan) -> (Vec<Cell>, Option<usize>) {
        let mut run = Self::new(level, plan);
        let mut seen = vec![false; level.cells()];
        let mut order = Vec::new();
        loop {
            for leaf in &run.leaves {
                if leaf.fate == Fate::Aloft
                    && let Some(i) = level.index(leaf.at)
                    && !seen[i]
                {
                    seen[i] = true;
                    order.push(leaf.at);
                }
            }
            if run.over {
                return (order, None);
            }
            run.step();
            if let Some(&Cue::Lost { leaf, .. }) =
                run.cues.iter().find(|c| matches!(c, Cue::Lost { .. }))
            {
                return (order, Some(leaf));
            }
        }
    }
}

const fn leaf_from(leaf: &mut Leaf) {
    leaf.from = leaf.at;
}

/// What a finished run earned: every leaf home, few pieces, little wind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub home: bool,
    pub few: bool,
    pub gentle: bool,
}

impl Marks {
    #[must_use]
    pub fn of(run: &Run) -> Self {
        let home = run.all_home();
        let level = run.level();
        Self {
            home,
            few: home && run.pieces() <= usize::from(level.par_pieces),
            gentle: home && run.gusts() <= level.par_gusts,
        }
    }

    #[must_use]
    pub fn count(self) -> u8 {
        u8::from(self.home) + u8::from(self.few) + u8::from(self.gentle)
    }

    /// The better of two, mark by mark: a garden's record keeps the best
    /// of each, even from different solutions.
    #[must_use]
    pub const fn best(self, other: Self) -> Self {
        Self {
            home: self.home || other.home,
            few: self.few || other.few,
            gentle: self.gentle || other.gentle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: Level = Level {
        width: 3,
        height: 3,
        drops: &[Drop {
            tick: 0,
            x: 0,
            hue: Hue::Ember,
        }],
        baskets: &[Basket {
            x: 2,
            hue: Hue::Ember,
        }],
        fixed: &[],
        stones: &[],
        kit: Kit::Flips,
        par_pieces: 1,
        par_gusts: 2,
    };

    #[test]
    fn a_leaf_with_nothing_in_its_way_lands_on_the_ground() {
        let mut run = Run::new(&ONE, &Plan::empty(&ONE));
        run.finish();
        assert_eq!(run.leaves()[0].fate, Fate::Lost(Loss::Ground, 3));
        assert!(!run.all_home());
    }

    #[test]
    fn a_run_of_vanes_carries_a_leaf_home() {
        let mut plan = Plan::empty(&ONE);
        plan.set(&ONE, Cell::new(0, 1), Some(Piece::Vane(Dir::Right)));
        plan.set(&ONE, Cell::new(1, 1), Some(Piece::Vane(Dir::Right)));
        let mut run = Run::new(&ONE, &plan);
        run.finish();
        assert!(run.all_home());
        assert_eq!(run.gusts(), 2);
        assert_eq!(Marks::of(&run).count(), 2, "home, gentle, but two pieces");
    }

    #[test]
    fn vanes_facing_each_other_make_an_eddy() {
        let mut plan = Plan::empty(&ONE);
        plan.set(&ONE, Cell::new(0, 0), Some(Piece::Vane(Dir::Right)));
        plan.set(&ONE, Cell::new(1, 0), Some(Piece::Vane(Dir::Left)));
        let mut run = Run::new(&ONE, &plan);
        run.finish();
        assert!(matches!(run.leaves()[0].fate, Fate::Lost(Loss::Eddy, _)));
    }

    #[test]
    fn a_vane_at_the_edge_blows_a_leaf_away() {
        let mut plan = Plan::empty(&ONE);
        plan.set(&ONE, Cell::new(0, 0), Some(Piece::Vane(Dir::Left)));
        let mut run = Run::new(&ONE, &plan);
        run.finish();
        assert!(matches!(run.leaves()[0].fate, Fate::Lost(Loss::Blown, _)));
    }

    #[test]
    fn a_flip_vane_turns_after_each_leaf() {
        const TWO: Level = Level {
            width: 3,
            height: 2,
            drops: &[
                Drop {
                    tick: 0,
                    x: 1,
                    hue: Hue::Ember,
                },
                Drop {
                    tick: 2,
                    x: 1,
                    hue: Hue::Gold,
                },
            ],
            baskets: &[
                Basket {
                    x: 0,
                    hue: Hue::Ember,
                },
                Basket {
                    x: 2,
                    hue: Hue::Gold,
                },
            ],
            fixed: &[],
            stones: &[],
            kit: Kit::Flips,
            par_pieces: 1,
            par_gusts: 2,
        };
        let mut plan = Plan::empty(&TWO);
        plan.set(&TWO, Cell::new(1, 0), Some(Piece::Flip(Dir::Left)));
        let mut run = Run::new(&TWO, &plan);
        run.finish();
        assert!(run.all_home(), "{:?}", run.leaves());
        assert_eq!(Marks::of(&run).count(), 3);
    }

    #[test]
    fn clicking_cycles_through_the_kit_and_back_to_nothing() {
        let mut p = None;
        let mut seen = 0;
        loop {
            p = Piece::next(p, Kit::Flips);
            seen += 1;
            if p.is_none() {
                break;
            }
            assert_eq!(Piece::previous(Piece::next(p, Kit::Flips), Kit::Flips), p);
        }
        assert_eq!(seen, 5);
        assert_eq!(
            Piece::next(Some(Piece::Vane(Dir::Left)), Kit::Vanes),
            None,
            "a vanes-only kit skips the flips"
        );
    }

    #[test]
    fn fixed_pieces_and_stones_cannot_be_built_on() {
        const ROCKY: Level = Level {
            fixed: &[(Cell { x: 1, y: 1 }, Piece::Vane(Dir::Right))],
            stones: &[Cell { x: 2, y: 2 }],
            ..ONE
        };
        let mut plan = Plan::empty(&ROCKY);
        assert!(!plan.set(&ROCKY, Cell::new(1, 1), Some(Piece::Vane(Dir::Left))));
        assert!(!plan.set(&ROCKY, Cell::new(2, 2), Some(Piece::Vane(Dir::Left))));
        assert!(!plan.set(&ROCKY, Cell::new(5, 0), Some(Piece::Vane(Dir::Left))));
        assert!(plan.set(&ROCKY, Cell::new(0, 1), Some(Piece::Vane(Dir::Right))));
        let mut run = Run::new(&ROCKY, &plan);
        run.finish();
        assert!(run.all_home(), "the fixed vane carries it on");
        assert_eq!(run.pieces(), 1, "and does not count");
    }
}
