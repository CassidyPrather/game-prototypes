//! The rules of one night, as a deterministic, turn-based state machine.
//!
//! Three kinds of thing stand on a small grid: the hearth, the keepers (all
//! of them you), and the shades. One rule drives everything: **a shade walks
//! toward the nearest light and puts out what it touches.** A keeper is a
//! light; so is the hearth. Nobody can hurt a shade, and only dawn gets rid
//! of one. All a keeper can do is walk, and so be nearer than the fire, or
//! not.
//!
//! [`World`] is one moment of the night and [`World::advance`] moves it on
//! by one hour, given a [`Step`] for every keeper. [`Watch`] wraps that in
//! the loop the game is about: the night is lived once per keeper, each
//! earlier keeper replaying the steps it was given, until a night ends with
//! the hearth still lit.
//!
//! Nothing is rolled. A level and a list of steps always give the same
//! night, which is what lets [`super::solve`] say how few keepers one needs.

/// One square of the grid. `x` grows east, `y` grows south.
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

    /// Steps between two cells when nothing can move diagonally.
    #[must_use]
    pub const fn distance(self, other: Self) -> u8 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    /// The cell one step that way.
    #[must_use]
    pub const fn stepped(self, step: Step) -> Self {
        let (dx, dy) = step.delta();
        Self::new(self.x + dx, self.y + dy)
    }
}

/// What a keeper does with one hour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    Wait,
    North,
    East,
    South,
    West,
}

impl Step {
    pub const ALL: [Self; 5] = [Self::Wait, Self::North, Self::East, Self::South, Self::West];

    #[must_use]
    pub const fn delta(self) -> (i8, i8) {
        match self {
            Self::Wait => (0, 0),
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    /// One letter, for writing a night's steps down in a test.
    #[must_use]
    pub const fn letter(self) -> char {
        match self {
            Self::Wait => '.',
            Self::North => 'N',
            Self::East => 'E',
            Self::South => 'S',
            Self::West => 'W',
        }
    }

    #[must_use]
    pub const fn from_letter(letter: char) -> Option<Self> {
        Some(match letter {
            '.' => Self::Wait,
            'N' => Self::North,
            'E' => Self::East,
            'S' => Self::South,
            'W' => Self::West,
            _ => return None,
        })
    }
}

/// The most keepers one night can hold. Small on purpose: the fourth is
/// already an admission.
pub const MAX_KEEPERS: usize = 4;

/// One night as it is written down in [`super::nights`].
///
/// `rows` is the map: `.` open ground, `#` a tree, `F` the hearth, `s` a
/// shade, and `1` to `4` the bedrolls the keepers wake on, in order.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub rows: &'static [&'static str],
    /// Hours until dawn. Every keeper gets this many steps.
    pub hours: u8,
    /// How many shades the hearth can take and still be lit at dawn, plus
    /// one.
    pub embers: u8,
    /// The fewest keepers that can see the hearth through the night.
    pub par: u8,
}

/// A level's map, parsed: the part of a night that never changes.
#[derive(Clone, Debug)]
pub struct Ground {
    pub width: i8,
    pub height: i8,
    trees: Vec<bool>,
    pub hearth: Cell,
    /// Where each keeper wakes, in order.
    pub beds: Vec<Cell>,
    /// Where each shade starts.
    pub shades: Vec<Cell>,
}

impl Ground {
    /// Read a map. Panics on a malformed one; the maps are constants and
    /// `nights`' tests parse every one.
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    pub fn parse(rows: &[&str]) -> Self {
        let height = rows.len() as i8;
        let width = rows[0].chars().count() as i8;
        let mut trees = Vec::new();
        let mut hearth = None;
        let mut beds = [None; MAX_KEEPERS];
        let mut shades = Vec::new();
        for (y, row) in rows.iter().enumerate() {
            assert_eq!(row.chars().count(), width as usize, "ragged map");
            for (x, ch) in row.chars().enumerate() {
                let at = Cell::new(x as i8, y as i8);
                trees.push(ch == '#');
                match ch {
                    '.' | '#' => {}
                    'F' => {
                        assert!(hearth.replace(at).is_none(), "two hearths");
                    }
                    's' => shades.push(at),
                    '1'..='4' => {
                        let slot = ch as usize - '1' as usize;
                        assert!(beds[slot].replace(at).is_none(), "two beds {ch}");
                    }
                    _ => panic!("unknown map character {ch:?}"),
                }
            }
        }
        let beds: Vec<Cell> = beds.iter().map(|b| b.expect("a missing bed")).collect();
        Self {
            width,
            height,
            trees,
            hearth: hearth.expect("no hearth"),
            beds,
            shades,
        }
    }

    #[must_use]
    pub const fn inside(&self, at: Cell) -> bool {
        at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < self.height
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn tree(&self, at: Cell) -> bool {
        self.inside(at) && self.trees[at.y as usize * self.width as usize + at.x as usize]
    }

    /// Whether a keeper can stand here.
    #[must_use]
    pub fn open(&self, at: Cell) -> bool {
        self.inside(at) && !self.tree(at) && at != self.hearth
    }
}

/// A keeper at one moment: where, and whether its lantern is lit.
///
/// A snuffed keeper keeps its place and keeps taking steps. Nothing sees it
/// and it touches nothing, but the steps it is given still count: if a later
/// night saves it, it will walk exactly where its ghost walked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Keeper {
    pub at: Cell,
    pub lit: bool,
}

/// What a shade is walking toward.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Hearth,
    Keeper(usize),
}

/// What a shade means to do with the next hour, if nobody moves first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    pub target: Target,
    /// The light it is after.
    pub toward: Cell,
    /// The cell it will step into, or `None` if it is stuck.
    pub next: Option<Cell>,
}

/// Something that happened in one hour, in the order it happened. Says
/// what, never what it should look or sound like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// A keeper walked. `lit` is false for a ghost.
    Walked {
        keeper: usize,
        from: Cell,
        to: Cell,
        lit: bool,
    },
    /// A keeper tried to walk into a tree, the hearth or the edge.
    Balked { keeper: usize, at: Cell, step: Step },
    /// A shade moved.
    Crept { shade: usize, from: Cell, to: Cell },
    /// A shade and a keeper met `at`, and the keeper's lantern is out. `from`
    /// is where the shade came from (`at`, if the keeper walked into it).
    Snuffed {
        keeper: usize,
        shade: usize,
        from: Cell,
        at: Cell,
    },
    /// A shade reached the hearth from `from`; it and an ember are gone.
    Fed {
        shade: usize,
        from: Cell,
        embers: u8,
    },
    /// A shade turned to a different light than it was after an hour ago.
    Turned { shade: usize, target: Target },
    /// The night is over. Whatever shades are left burn off.
    Dawn { lit: bool },
}

/// One moment of one night.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct World {
    /// Hours gone.
    pub hour: u8,
    /// What the hearth has left. At zero it is out, though the shades still
    /// come to it.
    pub embers: u8,
    pub keepers: Vec<Keeper>,
    /// `None` once a shade is gone.
    pub shades: Vec<Option<Cell>>,
}

impl World {
    /// Dusk, with this many keepers awake.
    #[must_use]
    pub fn dusk(ground: &Ground, level: &Level, keepers: usize) -> Self {
        Self {
            hour: 0,
            embers: level.embers,
            keepers: ground.beds[..keepers]
                .iter()
                .map(|&at| Keeper { at, lit: true })
                .collect(),
            shades: ground.shades.iter().copied().map(Some).collect(),
        }
    }

    /// The nearest light to `from`. A keeper wins a tie with the hearth, and
    /// a later keeper wins a tie with an earlier one: the shades prefer the
    /// living.
    #[must_use]
    pub fn nearest_light(&self, ground: &Ground, from: Cell) -> (Target, Cell) {
        let mut best = (Target::Hearth, ground.hearth);
        let mut best_distance = from.distance(ground.hearth);
        for (i, keeper) in self.keepers.iter().enumerate() {
            if keeper.lit && from.distance(keeper.at) <= best_distance {
                best_distance = from.distance(keeper.at);
                best = (Target::Keeper(i), keeper.at);
            }
        }
        best
    }

    fn shade_at(&self, at: Cell) -> Option<usize> {
        self.shades.iter().position(|&s| s == Some(at))
    }

    /// Where a shade at `from` steps to get nearer `toward`: along the
    /// longer side first (east-west on a tie), along the other if a tree or
    /// another shade is in the way, and nowhere if that is blocked too.
    fn creep(&self, ground: &Ground, from: Cell, toward: Cell) -> Option<Cell> {
        let dx = toward.x - from.x;
        let dy = toward.y - from.y;
        let along_x = Cell::new(from.x + dx.signum(), from.y);
        let along_y = Cell::new(from.x, from.y + dy.signum());
        let order = if dx.abs() >= dy.abs() {
            [(dx, along_x), (dy, along_y)]
        } else {
            [(dy, along_y), (dx, along_x)]
        };
        order
            .into_iter()
            .filter(|&(delta, _)| delta != 0)
            .map(|(_, to)| to)
            .find(|&to| !ground.tree(to) && self.shade_at(to).is_none())
    }

    /// What every shade means to do next, if no keeper moves first.
    #[must_use]
    pub fn intents(&self, ground: &Ground) -> Vec<Option<Intent>> {
        self.shades
            .iter()
            .map(|shade| {
                shade.map(|from| {
                    let (target, toward) = self.nearest_light(ground, from);
                    Intent {
                        target,
                        toward,
                        next: self.creep(ground, from, toward),
                    }
                })
            })
            .collect()
    }

    /// One hour: every keeper takes its step, then every shade takes one.
    ///
    /// `steps` has one entry per keeper. The shades all choose their light
    /// at the same moment, after the keepers have moved, and then step in
    /// order of who is nearest theirs, so a line of them moves as a line.
    pub fn advance(&mut self, ground: &Ground, steps: &[Step], cues: &mut Vec<Cue>) {
        for (i, &step) in steps.iter().enumerate().take(self.keepers.len()) {
            let keeper = self.keepers[i];
            let to = keeper.at.stepped(step);
            if step == Step::Wait {
                continue;
            }
            if !ground.open(to) {
                if keeper.lit {
                    cues.push(Cue::Balked {
                        keeper: i,
                        at: keeper.at,
                        step,
                    });
                }
                continue;
            }
            self.keepers[i].at = to;
            cues.push(Cue::Walked {
                keeper: i,
                from: keeper.at,
                to,
                lit: keeper.lit,
            });
            if keeper.lit
                && let Some(shade) = self.shade_at(to)
            {
                self.keepers[i].lit = false;
                cues.push(Cue::Snuffed {
                    keeper: i,
                    shade,
                    from: to,
                    at: to,
                });
            }
        }

        let mut order: Vec<(u8, usize, Cell, Cell)> = self
            .shades
            .iter()
            .enumerate()
            .filter_map(|(i, shade)| {
                shade.map(|from| {
                    let (_, toward) = self.nearest_light(ground, from);
                    (from.distance(toward), i, from, toward)
                })
            })
            .collect();
        order.sort_unstable_by_key(|&(distance, i, ..)| (distance, i));
        for (_, shade, from, toward) in order {
            let Some(to) = self.creep(ground, from, toward) else {
                continue;
            };
            if to == ground.hearth {
                self.shades[shade] = None;
                self.embers = self.embers.saturating_sub(1);
                cues.push(Cue::Fed {
                    shade,
                    from,
                    embers: self.embers,
                });
            } else {
                self.shades[shade] = Some(to);
                cues.push(Cue::Crept { shade, from, to });
                for (keeper, k) in self.keepers.iter_mut().enumerate() {
                    if k.lit && k.at == to {
                        k.lit = false;
                        cues.push(Cue::Snuffed {
                            keeper,
                            shade,
                            from,
                            at: to,
                        });
                    }
                }
            }
        }
        self.hour += 1;
    }

    /// Whether the hearth is still burning.
    #[must_use]
    pub const fn lit(&self) -> bool {
        self.embers > 0
    }
}

/// Where the game stands between presses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The night is going; the newest keeper is waiting for a step.
    Night,
    /// Dawn came and the hearth was out. [`Watch::again`] lives it again.
    Cold,
    /// Dawn came on a lit hearth, with this many marks out of three.
    /// See [`Watch::marks`].
    Kept { marks: u8 },
}

/// A night being lived over and over: the level, every keeper's steps so
/// far, and the moment the newest keeper has reached.
#[derive(Clone, Debug)]
pub struct Watch {
    level: Level,
    ground: Ground,
    /// One list of steps per keeper; the last is the one being written.
    tapes: Vec<Vec<Step>>,
    /// The world at every hour of the current telling, dusk first.
    history: Vec<World>,
    cues: Vec<Cue>,
}

impl Watch {
    #[must_use]
    pub fn new(level: Level) -> Self {
        let ground = Ground::parse(level.rows);
        let dusk = World::dusk(&ground, &level, 1);
        Self {
            level,
            ground,
            tapes: vec![Vec::new()],
            history: vec![dusk],
            cues: Vec::new(),
        }
    }

    #[must_use]
    pub const fn level(&self) -> &Level {
        &self.level
    }

    #[must_use]
    pub const fn ground(&self) -> &Ground {
        &self.ground
    }

    /// The night as it stands.
    #[must_use]
    pub fn world(&self) -> &World {
        self.history.last().expect("history starts with dusk")
    }

    /// Every hour of this telling so far, dusk first.
    #[must_use]
    pub fn history(&self) -> &[World] {
        &self.history
    }

    /// How many keepers are awake, counting the one being played.
    #[must_use]
    pub fn keepers(&self) -> usize {
        self.tapes.len()
    }

    /// What happened in the last [`Watch::step`], in order. Cleared by the
    /// next call that changes anything.
    #[must_use]
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    /// The step keeper `i` is about to take, if it is one that already
    /// lived this night.
    #[must_use]
    pub fn remembered(&self, keeper: usize) -> Option<Step> {
        if keeper + 1 >= self.tapes.len() {
            return None;
        }
        Some(self.taped(keeper, usize::from(self.world().hour)))
    }

    fn taped(&self, keeper: usize, hour: usize) -> Step {
        self.tapes[keeper].get(hour).copied().unwrap_or(Step::Wait)
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        let world = self.world();
        if world.hour < self.level.hours {
            Phase::Night
        } else if world.lit() {
            Phase::Kept {
                marks: self.marks(),
            }
        } else {
            Phase::Cold
        }
    }

    /// One mark for a hearth still lit at dawn, one more for doing it with
    /// as few keepers as the night can be kept with, and one more if no
    /// lantern went out.
    #[must_use]
    pub fn marks(&self) -> u8 {
        let few = self.tapes.len() <= usize::from(self.level.par);
        let whole = self.world().keepers.iter().all(|keeper| keeper.lit);
        1 + u8::from(few) + u8::from(whole)
    }

    /// Whether this telling has as few keepers as the night needs.
    #[must_use]
    pub fn few(&self) -> bool {
        self.tapes.len() <= usize::from(self.level.par)
    }

    /// Whether every lantern is still lit.
    #[must_use]
    pub fn whole(&self) -> bool {
        self.world().keepers.iter().all(|keeper| keeper.lit)
    }

    /// Spend an hour: the newest keeper takes `step`, the others the steps
    /// they remember. Does nothing once dawn has come.
    pub fn step(&mut self, step: Step) -> bool {
        if self.phase() != Phase::Night {
            return false;
        }
        self.cues.clear();
        let before = self.world().intents(&self.ground);
        let hour = usize::from(self.world().hour);
        self.tapes.last_mut().expect("one tape").push(step);
        let steps: Vec<Step> = (0..self.tapes.len())
            .map(|keeper| self.taped(keeper, hour))
            .collect();
        let mut world = self.world().clone();
        world.advance(&self.ground, &steps, &mut self.cues);
        for (shade, (was, now)) in before.iter().zip(world.intents(&self.ground)).enumerate() {
            if let (Some(was), Some(now)) = (was, now)
                && was.target != now.target
            {
                self.cues.push(Cue::Turned {
                    shade,
                    target: now.target,
                });
            }
        }
        if world.hour == self.level.hours {
            self.cues.push(Cue::Dawn { lit: world.lit() });
        }
        self.history.push(world);
        true
    }

    /// Take the last hour back. At dusk, that is the last hour of the
    /// keeper before, who goes back to being the one played.
    pub fn undo(&mut self) -> bool {
        self.cues.clear();
        if self.history.len() > 1 {
            self.history.pop();
            self.tapes.last_mut().expect("one tape").pop();
            return true;
        }
        if self.tapes.len() == 1 {
            return false;
        }
        self.tapes.pop();
        self.tapes.last_mut().expect("one tape").pop();
        self.retell();
        true
    }

    /// After a cold dawn: live the night again with one more keeper. With
    /// every bedroll already taken, the last keeper tries again instead.
    pub fn again(&mut self) -> bool {
        if self.phase() != Phase::Cold {
            return false;
        }
        if self.tapes.len() < MAX_KEEPERS {
            self.tapes.push(Vec::new());
        } else {
            self.tapes.last_mut().expect("one tape").clear();
        }
        self.retell();
        true
    }

    /// Forget everything and start the night over with one keeper.
    pub fn restart(&mut self) {
        self.tapes = vec![Vec::new()];
        self.retell();
    }

    /// Rebuild the history from the tapes, up to the end of the last one.
    fn retell(&mut self) {
        self.cues.clear();
        let keepers = self.tapes.len();
        let mut world = World::dusk(&self.ground, &self.level, keepers);
        self.history = vec![world.clone()];
        let mut scratch = Vec::new();
        for hour in 0..self.tapes[keepers - 1].len() {
            let steps: Vec<Step> = (0..keepers).map(|k| self.taped(k, hour)).collect();
            world.advance(&self.ground, &steps, &mut scratch);
            self.history.push(world.clone());
        }
    }

    /// Every keeper's steps, for tests and for writing a night down.
    #[must_use]
    pub fn tapes(&self) -> &[Vec<Step>] {
        &self.tapes
    }
}

/// Play whole tapes through a level and say whether the hearth saw dawn.
/// Tapes shorter than the night wait out the rest of it.
#[must_use]
pub fn kept(level: &Level, tapes: &[Vec<Step>]) -> bool {
    let ground = Ground::parse(level.rows);
    let mut world = World::dusk(&ground, level, tapes.len());
    let mut cues = Vec::new();
    for hour in 0..usize::from(level.hours) {
        let steps: Vec<Step> = tapes
            .iter()
            .map(|tape| tape.get(hour).copied().unwrap_or(Step::Wait))
            .collect();
        world.advance(&ground, &steps, &mut cues);
    }
    world.lit()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: Level = Level {
        rows: &[".......", ".......", "..1234.", ".F....s", "......."],
        hours: 6,
        embers: 1,
        par: 1,
    };

    const HEDGE: Level = Level {
        rows: &["1234...", ".F.#..s", "......."],
        hours: 5,
        embers: 2,
        par: 1,
    };

    fn play(level: Level, steps: &str) -> Watch {
        let mut watch = Watch::new(level);
        for letter in steps.chars() {
            watch.step(Step::from_letter(letter).expect("a step letter"));
        }
        watch
    }

    #[test]
    fn a_shade_goes_for_the_nearest_light_and_a_keeper_wins_a_tie() {
        let watch = Watch::new(OPEN);
        let world = watch.world();
        let ground = watch.ground();
        // The shade is five from the hearth and five from the keeper.
        let shade = ground.shades[0];
        assert_eq!(shade.distance(ground.hearth), 5);
        assert_eq!(shade.distance(world.keepers[0].at), 5);
        assert_eq!(world.nearest_light(ground, shade).0, Target::Keeper(0));
        // From the far side of the hearth, the hearth is nearer.
        assert_eq!(
            world.nearest_light(ground, Cell::new(0, 4)).0,
            Target::Hearth
        );
    }

    #[test]
    fn a_later_keeper_wins_a_tie_with_an_earlier_one() {
        let ground = Ground::parse(OPEN.rows);
        let world = World::dusk(&ground, &OPEN, 3);
        // (3,0) is two from keeper 1 at (3,2) and three from the others...
        assert_eq!(
            world.nearest_light(&ground, Cell::new(3, 0)).0,
            Target::Keeper(1)
        );
        // ...and (3,1) between keepers 0 and 2 goes to keeper 1 outright,
        // but the cell above the gap between 1 and 2 ties, and 2 has it.
        let between = Cell::new(3, 2).stepped(Step::North);
        assert_eq!(between.distance(Cell::new(4, 2)), 2);
        let tie = Cell::new(4, 1);
        assert_eq!(tie.distance(Cell::new(3, 2)), 2);
        assert_eq!(tie.distance(Cell::new(5, 2)), 2);
        assert_eq!(tie.distance(Cell::new(4, 2)), 1);
        assert_eq!(world.nearest_light(&ground, tie).0, Target::Keeper(2));
    }

    #[test]
    fn left_alone_a_shade_reaches_the_hearth_and_dawn_is_cold() {
        // The keeper walks off west and north, out of the way.
        let watch = play(OPEN, "WWN...");
        assert_eq!(watch.phase(), Phase::Cold);
        assert!(!watch.world().lit());
        assert!(watch.world().shades[0].is_none(), "the hearth took it");
    }

    #[test]
    fn a_shade_led_away_does_not_come_back_in_time() {
        let watch = play(OPEN, "EENNEE");
        assert_eq!(watch.phase(), Phase::Kept { marks: 3 });
        assert!(watch.world().keepers[0].lit);
    }

    #[test]
    fn a_keeper_caught_is_snuffed_and_the_shade_goes_on() {
        const LONG: Level = Level { hours: 9, ..OPEN };
        let mut watch = Watch::new(LONG);
        for _ in 0..5 {
            watch.step(Step::Wait);
        }
        assert!(
            !watch.world().keepers[0].lit,
            "it should have met the shade"
        );
        assert!(watch.world().shades[0].is_some(), "a shade is never spent");
        assert!(
            watch
                .cues()
                .iter()
                .any(|cue| matches!(cue, Cue::Snuffed { keeper: 0, .. }))
        );
        while watch.phase() == Phase::Night {
            watch.step(Step::Wait);
        }
        assert_eq!(watch.phase(), Phase::Cold);
    }

    #[test]
    fn walking_into_a_shade_is_as_good_as_being_caught() {
        const CLOSE: Level = Level {
            rows: &["1s.234.", ".....F."],
            hours: 3,
            embers: 1,
            par: 1,
        };
        let mut watch = Watch::new(CLOSE);
        watch.step(Step::East);
        assert!(!watch.world().keepers[0].lit);
        assert_eq!(watch.world().shades[0], Some(Cell::new(2, 0)));
    }

    #[test]
    fn a_snuffed_keeper_still_walks_as_a_ghost() {
        let mut watch = Watch::new(OPEN);
        for _ in 0..5 {
            watch.step(Step::Wait);
        }
        let fell = watch.world().keepers[0].at;
        assert!(watch.step(Step::North));
        let ghost = watch.world().keepers[0];
        assert!(!ghost.lit);
        assert_eq!(ghost.at, fell.stepped(Step::North));
        // Nothing is after it any more.
        let intents = watch.world().intents(watch.ground());
        assert_eq!(intents[0].expect("the shade").target, Target::Hearth);
    }

    #[test]
    fn a_tree_dead_ahead_stops_a_shade_for_good() {
        // Nobody near: it wants the hearth, the tree is in the way, and
        // it has no reason to step aside.
        let watch = play(HEDGE, ".....");
        assert_eq!(watch.world().shades[0], Some(Cell::new(4, 1)));
        assert_eq!(watch.phase(), Phase::Kept { marks: 3 });
        let intent = watch.world().intents(watch.ground())[0].expect("the shade");
        assert_eq!(intent.next, None);
    }

    #[test]
    fn waking_a_stuck_shade_brings_it_round_the_tree() {
        // Walk east along the top until it prefers the keeper, which is
        // off its line, so it can step.
        let watch = play(HEDGE, "EEEE.");
        assert_ne!(watch.world().shades[0], Some(Cell::new(4, 1)));
    }

    #[test]
    fn embers_let_the_hearth_take_a_shade() {
        const TWO: Level = Level {
            rows: &["1234...", ".F...s.", "....s.."],
            hours: 4,
            embers: 2,
            par: 1,
        };
        const ONE: Level = Level {
            rows: &["1234...", ".F...s.", "......."],
            ..TWO
        };
        let mut watch = Watch::new(TWO);
        watch.step(Step::West);
        for _ in 0..3 {
            watch.step(Step::Wait);
        }
        // Both arrive; the second finds one ember and takes it.
        assert_eq!(watch.world().embers, 0);
        assert_eq!(watch.phase(), Phase::Cold);
        // With only one of them, the hearth has an ember to spare.
        let watch = play(ONE, "W...");
        assert_eq!(watch.world().embers, 1);
        assert_eq!(watch.phase(), Phase::Kept { marks: 3 });
    }

    #[test]
    fn a_cold_dawn_is_lived_again_with_one_more() {
        let mut watch = play(OPEN, "WWN...");
        assert!(watch.again());
        assert_eq!(watch.keepers(), 2);
        assert_eq!(watch.world().hour, 0);
        // The first keeper walks where it walked, without being told.
        assert_eq!(watch.remembered(0), Some(Step::West));
        assert_eq!(watch.remembered(1), None);
        watch.step(Step::Wait);
        assert_eq!(
            watch.world().keepers[0].at,
            watch.ground().beds[0].stepped(Step::West)
        );
    }

    #[test]
    fn marks_are_the_hearth_then_few_keepers_then_whole_lanterns() {
        const DETOUR: Level = Level {
            rows: &["....s..", ".......", ".12....", ".3F....", "..4...."],
            hours: 6,
            embers: 1,
            par: 1,
        };
        // Led away by the one keeper the night needs: all three.
        let watch = play(OPEN, "EENNEE");
        assert!(watch.few() && watch.whole());
        assert_eq!(watch.phase(), Phase::Kept { marks: 3 });
        // The same night, if it could be kept with none: not few.
        let watch = play(Level { par: 0, ..OPEN }, "EENNEE");
        assert!(!watch.few() && watch.whole());
        assert_eq!(watch.phase(), Phase::Kept { marks: 2 });
        // A keeper who stands still, is caught, and has still cost the
        // shade enough of the night: few, but not whole.
        let watch = play(DETOUR, "......");
        assert!(watch.few() && !watch.whole());
        assert_eq!(watch.phase(), Phase::Kept { marks: 2 });
    }

    #[test]
    fn the_last_bedroll_is_tried_again_rather_than_a_fifth() {
        // A night nobody can keep: the shade is at the hearth already.
        const DOOMED: Level = Level {
            rows: &["1234...", ".....Fs"],
            hours: 2,
            embers: 1,
            par: 1,
        };
        let mut watch = Watch::new(DOOMED);
        for _ in 0..MAX_KEEPERS + 2 {
            for _ in 0..DOOMED.hours {
                watch.step(Step::Wait);
            }
            assert_eq!(watch.phase(), Phase::Cold);
            assert!(watch.again());
            assert!(watch.keepers() <= MAX_KEEPERS);
        }
        assert_eq!(watch.keepers(), MAX_KEEPERS);
        assert_eq!(watch.world().hour, 0);
    }

    #[test]
    fn undo_takes_back_one_hour_and_then_one_keeper() {
        let mut watch = play(OPEN, "EE");
        let before = watch.world().clone();
        watch.step(Step::North);
        assert!(watch.undo());
        assert_eq!(*watch.world(), before);

        // From dusk of the second telling, undo is the last hour of the
        // first.
        let mut watch = play(OPEN, "WWN...");
        let dawn = watch.world().clone();
        watch.again();
        assert!(watch.undo());
        assert_eq!(watch.keepers(), 1);
        assert_eq!(watch.world().hour, OPEN.hours - 1);
        watch.step(Step::Wait);
        assert_eq!(*watch.world(), dawn);

        // And at the very start there is nothing to take back.
        let mut watch = Watch::new(OPEN);
        assert!(!watch.undo());
    }

    #[test]
    fn the_same_steps_always_give_the_same_night() {
        let a = play(OPEN, "ENWS.E");
        let b = play(OPEN, "ENWS.E");
        assert_eq!(a.history(), b.history());
        assert_eq!(a.cues(), b.cues());
    }

    #[test]
    fn cues_say_who_turned_and_when_dawn_came() {
        let mut watch = Watch::new(OPEN);
        // West twice: the hearth becomes the nearer light.
        watch.step(Step::West);
        assert!(watch.cues().iter().any(|cue| matches!(
            cue,
            Cue::Turned {
                target: Target::Hearth,
                ..
            }
        )));
        while watch.phase() == Phase::Night {
            watch.step(Step::Wait);
        }
        assert_eq!(watch.cues().last(), Some(&Cue::Dawn { lit: false }));
    }
}
