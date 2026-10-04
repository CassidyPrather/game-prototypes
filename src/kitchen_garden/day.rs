//! One day at the kitchen garden, as a deterministic state machine.
//!
//! The frontend drives it with commands — [`Day::move_food`],
//! [`Day::work`], [`Day::start`] — and [`Day::tick`] at
//! [`cooking::TICKS_PER_SEC`]. What happened comes back as [`Cue`]s, which
//! live until the next drain. No clock, no randomness but the seeded one,
//! no macroquad.
//!
//! Everything the cook handles is a [`Food`] in a [`Place`], and moves only
//! through [`Day::move_food`], which checks both ends before touching
//! either: a refused move changes nothing, so food is never lost in a drag.
//! The only ways food leaves the world are a customer eating it, the
//! compost heap, and a recipe consuming it.

use crate::cooking::{self, Food, Process, Station, secs};
use crate::kitchen_garden::prices;

/// Length of the day, in ticks.
pub const DAY: u32 = secs(240);
/// No one new arrives after this, so the last customers can be served.
pub const LAST_ORDERS: u32 = DAY - secs(25);
/// Pantry shelves.
pub const PANTRY: usize = 8;
/// How many of one food a shelf holds.
pub const STACK: u8 = 4;
/// Garden plots.
pub const PLOTS: usize = 6;
/// Places at the hatch.
pub const SEATS: usize = 3;
/// Logs the hearth can have stacked beside it.
pub const LOG_MAX: u8 = 6;
/// How long one log burns, in ticks of a lit hearth.
pub const LOG_BURN: u32 = secs(14);
/// Strokes of the axe per log.
pub const AXE_STROKES: u8 = 4;
/// Turns of the crank per bucket.
pub const CRANK_STROKES: u8 = 3;
/// Pulls per pail of milk.
pub const MILK_STROKES: u8 = 4;
/// Most buckets, pails or eggs waiting at once.
pub const WAITING_MAX: u8 = 2;
/// The cow's udder: most milkings stored.
pub const UDDER_MAX: u8 = 3;
/// Ticks for the cow to make another milking.
pub const UDDER_REFILL: u32 = secs(16);
/// Ticks for the hen to lay.
pub const LAY: u32 = secs(15);
/// Compost the heap holds.
pub const COMPOST_MAX: u8 = 3;
/// What a composted planting adds to its harvest.
pub const COMPOST_BONUS: u8 = 1;
/// Coins to start with.
pub const START_COINS: u32 = 4;
/// Coins for each of the three marks at dusk.
pub const MARKS: [u32; 3] = [30, 55, 80];

/// What grows in the garden, how long it takes, and how many it gives.
#[must_use]
pub const fn crop(food: Food) -> Option<(u32, u8)> {
    match food {
        Food::TeaLeaf => Some((secs(16), 3)),
        Food::Carrot => Some((secs(20), 3)),
        Food::Onion => Some((secs(22), 3)),
        Food::Beans => Some((secs(24), 3)),
        Food::Tomato => Some((secs(26), 3)),
        Food::Wheat | Food::Rice => Some((secs(30), 3)),
        _ => None,
    }
}

/// Everything the garden grows.
pub const CROPS: [Food; 7] = [
    Food::TeaLeaf,
    Food::Carrot,
    Food::Onion,
    Food::Beans,
    Food::Tomato,
    Food::Wheat,
    Food::Rice,
];

/// What the market sells.
pub const MARKET: [Food; 3] = [Food::Salt, Food::Sugar, Food::Spice];

/// The kitchen's stations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StationId {
    /// Chopping board.
    Board,
    /// Millstones, for flour.
    Quern,
    /// Mixing bowl: kneads, mixes, churns.
    Bowl,
    /// Crock on the warm shelf: proves, soaks, steeps.
    Crock,
    /// Pot on the hearth.
    Pot,
    /// Pan on the hearth.
    Pan,
    /// Oven in the hearth.
    Oven,
}

impl StationId {
    pub const ALL: [Self; 7] = [
        Self::Board,
        Self::Quern,
        Self::Bowl,
        Self::Crock,
        Self::Pot,
        Self::Pan,
        Self::Oven,
    ];

    /// What it does.
    #[must_use]
    pub const fn processes(self) -> &'static [Process] {
        match self {
            Self::Board => &[Process::Chop],
            Self::Quern => &[Process::Grind],
            Self::Bowl => &[Process::Knead, Process::Mix, Process::Churn],
            Self::Crock => &[Process::Prove, Process::Soak, Process::Steep],
            Self::Pot => &[Process::Boil],
            Self::Pan => &[Process::Fry],
            Self::Oven => &[Process::Bake],
        }
    }

    /// Whether it needs the hearth lit.
    #[must_use]
    pub const fn heated(self) -> bool {
        matches!(self, Self::Pot | Self::Pan | Self::Oven)
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Somewhere food can be, or come from, or go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// One pantry shelf.
    Shelf(usize),
    /// Whichever pantry shelf will take it. Only ever a destination.
    Pantry,
    Station(StationId),
    /// A garden plot: harvest from it, or plant in it.
    Plot(usize),
    /// The hen's nest.
    Nest,
    /// The milk pail.
    Pail,
    /// The well's bucket.
    Bucket,
    /// One of the market's wares.
    Market(Food),
    /// A customer at the hatch.
    Seat(usize),
    /// The compost heap. Only ever a destination.
    Compost,
}

/// What can be worked by hand, besides the physical stations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chore {
    /// Pull the cow's teats.
    Milk,
    /// Turn the well's crank.
    Crank,
    /// Split a log.
    Axe,
}

/// Why something was not done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The station said no.
    Station(cooking::Refusal),
    /// There is no room there.
    Full,
    /// There is nothing there.
    Nothing,
    /// It does not want that.
    Unwanted,
    /// Not enough coins.
    Poor,
    /// No fire, and no logs to light one.
    NoFuel,
    /// The day is over.
    Dusk,
}

impl From<cooking::Refusal> for Refusal {
    fn from(r: cooking::Refusal) -> Self {
        Self::Station(r)
    }
}

/// A garden plot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plot {
    Empty,
    Growing {
        crop: Food,
        age: u32,
        /// Composted when planted.
        rich: bool,
    },
    Ripe {
        crop: Food,
        left: u8,
    },
}

impl Plot {
    /// How grown, `0..=1`.
    #[must_use]
    pub fn growth(self) -> f32 {
        match self {
            Self::Empty => 0.0,
            Self::Growing { crop: c, age, .. } => {
                crop(c).map_or(0.0, |(t, _)| age as f32 / t as f32)
            }
            Self::Ripe { .. } => 1.0,
        }
    }
}

/// Someone at the hatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Customer {
    /// What they want.
    pub wants: Food,
    /// Ticks they will wait in all.
    pub patience: u32,
    /// Ticks they have waited.
    pub waited: u32,
    /// A number to dress them by.
    pub look: u32,
}

impl Customer {
    /// Patience left, `0..=1`.
    #[must_use]
    pub fn mood(&self) -> f32 {
        1.0 - self.waited as f32 / self.patience.max(1) as f32
    }
}

/// Something that happened, for sound and motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// A station did something.
    Station(StationId, cooking::Event),
    /// Food moved.
    Moved { from: Place, to: Place, food: Food },
    /// Something was refused at this place.
    Refused(Place, Refusal),
    /// A plot came ripe.
    Ripe(usize),
    /// The last of a plot was picked.
    Cleared(usize),
    /// Planted, with compost or without.
    Planted { plot: usize, rich: bool },
    /// The hen laid.
    Laid,
    /// A chore's stroke; `done` when it made something.
    Chore { chore: Chore, done: bool },
    /// A chore could not be done.
    Balked(Chore, Refusal),
    /// The hearth caught.
    Lit,
    /// A fresh log went on.
    Log,
    /// The hearth went out.
    Out,
    /// A customer arrived.
    Arrived(usize),
    /// A customer is running out of patience.
    Impatient(usize),
    /// Served: what it paid, and whether that included the tip.
    Served { seat: usize, coins: u32, tip: bool },
    /// A customer gave up.
    Left(usize),
    /// Coins spent at the market.
    Bought(Food, u32),
    /// The day is over.
    Dusk,
}

/// The whole day.
#[derive(Clone, Debug)]
pub struct Day {
    clock: u32,
    coins: u32,
    earned: u32,
    served: u32,
    pantry: [Option<(Food, u8)>; PANTRY],
    stations: Vec<Station>,
    plots: [Plot; PLOTS],
    seats: [Option<Customer>; SEATS],
    next_arrival: u32,
    eggs: u8,
    lay: u32,
    udder: u8,
    udder_refill: u32,
    pail: u8,
    milk_strokes: u8,
    bucket: u8,
    crank_strokes: u8,
    logs: u8,
    axe_strokes: u8,
    /// Ticks left on the log burning now.
    fire: u32,
    compost: u8,
    rng: fastrand::Rng,
    cues: Vec<Cue>,
}

impl Default for Day {
    fn default() -> Self {
        Self::new(0x00C0_FFEE)
    }
}

impl Day {
    /// A fresh morning. The seed decides who comes, when, and wanting what.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut pantry = [None; PANTRY];
        for (slot, stack) in [
            (Food::Wheat, 1),
            (Food::Rice, 2),
            (Food::Egg, 1),
            (Food::Beans, 1),
            (Food::Tomato, 1),
        ]
        .into_iter()
        .enumerate()
        {
            pantry[slot] = Some(stack);
        }
        Self {
            clock: 0,
            coins: START_COINS,
            earned: 0,
            served: 0,
            pantry,
            stations: StationId::ALL
                .iter()
                .map(|s| Station::new(s.processes()))
                .collect(),
            plots: [
                Plot::Ripe {
                    crop: Food::TeaLeaf,
                    left: 2,
                },
                Plot::Growing {
                    crop: Food::Carrot,
                    age: secs(14),
                    rich: false,
                },
                Plot::Growing {
                    crop: Food::Wheat,
                    age: secs(12),
                    rich: false,
                },
                Plot::Growing {
                    crop: Food::Onion,
                    age: secs(10),
                    rich: false,
                },
                Plot::Empty,
                Plot::Empty,
            ],
            seats: [None; SEATS],
            next_arrival: secs(3),
            eggs: 1,
            lay: 0,
            udder: 2,
            udder_refill: 0,
            pail: 0,
            milk_strokes: 0,
            bucket: 1,
            crank_strokes: 0,
            logs: 3,
            axe_strokes: 0,
            fire: 0,
            compost: 0,
            rng: fastrand::Rng::with_seed(seed),
            cues: Vec::new(),
        }
    }

    // What there is to see.

    /// Ticks since dawn.
    #[must_use]
    pub const fn clock(&self) -> u32 {
        self.clock
    }

    /// How far through the day, `0..=1`.
    #[must_use]
    pub fn daylight(&self) -> f32 {
        (self.clock as f32 / DAY as f32).min(1.0)
    }

    #[must_use]
    pub const fn is_over(&self) -> bool {
        self.clock >= DAY
    }

    #[must_use]
    pub const fn coins(&self) -> u32 {
        self.coins
    }

    /// Coins taken from customers today, tips included.
    #[must_use]
    pub const fn earned(&self) -> u32 {
        self.earned
    }

    /// Dishes served today.
    #[must_use]
    pub const fn served(&self) -> u32 {
        self.served
    }

    /// How many of the three marks the coins in hand reach.
    #[must_use]
    pub fn marks(&self) -> usize {
        MARKS.iter().filter(|&&m| self.coins >= m).count()
    }

    #[must_use]
    pub const fn pantry(&self) -> &[Option<(Food, u8)>; PANTRY] {
        &self.pantry
    }

    #[must_use]
    pub fn station(&self, id: StationId) -> &Station {
        &self.stations[id.index()]
    }

    #[must_use]
    pub const fn plots(&self) -> &[Plot; PLOTS] {
        &self.plots
    }

    #[must_use]
    pub const fn seats(&self) -> &[Option<Customer>; SEATS] {
        &self.seats
    }

    #[must_use]
    pub const fn eggs(&self) -> u8 {
        self.eggs
    }

    /// How close the hen is to laying, `0..=1`.
    #[must_use]
    pub fn lay_progress(&self) -> f32 {
        self.lay as f32 / LAY as f32
    }

    #[must_use]
    pub const fn udder(&self) -> u8 {
        self.udder
    }

    #[must_use]
    pub const fn pail(&self) -> u8 {
        self.pail
    }

    #[must_use]
    pub const fn bucket(&self) -> u8 {
        self.bucket
    }

    #[must_use]
    pub const fn logs(&self) -> u8 {
        self.logs
    }

    /// How much of the log on the fire is left, `0..=1`; 0 when out.
    #[must_use]
    pub fn fire(&self) -> f32 {
        self.fire as f32 / LOG_BURN as f32
    }

    #[must_use]
    pub const fn compost(&self) -> u8 {
        self.compost
    }

    /// Strokes made toward a chore's next result, and how many it takes.
    #[must_use]
    pub const fn chore_progress(&self, chore: Chore) -> (u8, u8) {
        match chore {
            Chore::Milk => (self.milk_strokes, MILK_STROKES),
            Chore::Crank => (self.crank_strokes, CRANK_STROKES),
            Chore::Axe => (self.axe_strokes, AXE_STROKES),
        }
    }

    /// Everything that happened since the last drain.
    pub fn drain(&mut self) -> Vec<Cue> {
        std::mem::take(&mut self.cues)
    }

    // Moving food.

    /// What taking from `from` would give, without taking it.
    pub fn peek(&self, from: Place) -> Result<Food, Refusal> {
        match from {
            Place::Shelf(i) => self
                .pantry
                .get(i)
                .copied()
                .flatten()
                .map(|(f, _)| f)
                .ok_or(Refusal::Nothing),
            Place::Station(id) => self.station(id).peek().ok_or_else(|| {
                if self.station(id).is_idle() {
                    Refusal::Nothing
                } else {
                    Refusal::Station(cooking::Refusal::Busy)
                }
            }),
            Place::Plot(i) => match self.plots.get(i) {
                Some(Plot::Ripe { crop, .. }) => Ok(*crop),
                _ => Err(Refusal::Nothing),
            },
            Place::Nest => (self.eggs > 0).then_some(Food::Egg).ok_or(Refusal::Nothing),
            Place::Pail => (self.pail > 0)
                .then_some(Food::Milk)
                .ok_or(Refusal::Nothing),
            Place::Bucket => (self.bucket > 0)
                .then_some(Food::Water)
                .ok_or(Refusal::Nothing),
            Place::Market(food) => match prices::market_price(food) {
                Some(p) if p <= self.coins => Ok(food),
                Some(_) => Err(Refusal::Poor),
                None => Err(Refusal::Nothing),
            },
            Place::Pantry | Place::Seat(_) | Place::Compost => Err(Refusal::Nothing),
        }
    }

    /// Whether `food` could go to `to`.
    pub fn check(&self, to: Place, food: Food) -> Result<(), Refusal> {
        match to {
            Place::Shelf(i) => match self.pantry.get(i).copied().flatten() {
                None if i < PANTRY => Ok(()),
                Some((f, n)) if f == food && n < STACK => Ok(()),
                _ => Err(Refusal::Full),
            },
            Place::Pantry => self.shelf_for(food).map(|_| ()).ok_or(Refusal::Full),
            Place::Station(id) => Ok(self.station(id).room()?),
            Place::Plot(i) => match self.plots.get(i) {
                Some(Plot::Empty) if crop(food).is_some() => Ok(()),
                Some(Plot::Empty) => Err(Refusal::Unwanted),
                _ => Err(Refusal::Full),
            },
            Place::Seat(i) => match self.seats.get(i).copied().flatten() {
                Some(c) if c.wants == food => Ok(()),
                Some(_) => Err(Refusal::Unwanted),
                None => Err(Refusal::Nothing),
            },
            Place::Compost => Ok(()),
            Place::Nest | Place::Pail | Place::Bucket | Place::Market(_) => Err(Refusal::Unwanted),
        }
    }

    /// Whether moving from `from` to `to` would work, without doing it.
    pub fn can_move(&self, from: Place, to: Place) -> Result<Food, Refusal> {
        if self.is_over() {
            return Err(Refusal::Dusk);
        }
        let food = self.peek(from)?;
        // Shelf to pantry would only ever put it back where it was.
        if from == to || matches!((from, to), (Place::Shelf(_), Place::Pantry)) {
            return Err(Refusal::Unwanted);
        }
        self.check(to, food)?;
        Ok(food)
    }

    /// Move one food from `from` to `to`, or nothing at all.
    pub fn move_food(&mut self, from: Place, to: Place) -> Result<Food, Refusal> {
        let food = match self.can_move(from, to) {
            Ok(food) => food,
            Err(refusal) => {
                self.cues.push(Cue::Refused(to, refusal));
                return Err(refusal);
            }
        };
        self.remove(from);
        self.insert(to, food);
        self.cues.push(Cue::Moved { from, to, food });
        self.collect_station_events();
        Ok(food)
    }

    /// Harvest, gather or buy everything at `from` that fits in the
    /// pantry, one at a time. Returns how many moved.
    pub fn stow_all(&mut self, from: Place) -> usize {
        let mut moved = 0;
        while self.can_move(from, Place::Pantry).is_ok() {
            if self.move_food(from, Place::Pantry).is_err() {
                break;
            }
            moved += 1;
            // One purchase per click: the market never runs dry.
            if matches!(from, Place::Market(_)) {
                break;
            }
        }
        if moved == 0 {
            let refusal = self
                .can_move(from, Place::Pantry)
                .err()
                .unwrap_or(Refusal::Nothing);
            self.cues.push(Cue::Refused(from, refusal));
        }
        moved
    }

    fn shelf_for(&self, food: Food) -> Option<usize> {
        self.pantry
            .iter()
            .position(|s| matches!(s, Some((f, n)) if *f == food && *n < STACK))
            .or_else(|| self.pantry.iter().position(Option::is_none))
    }

    fn remove(&mut self, from: Place) {
        match from {
            Place::Shelf(i) => {
                if let Some((_, n)) = &mut self.pantry[i] {
                    *n -= 1;
                    if *n == 0 {
                        self.pantry[i] = None;
                    }
                }
            }
            Place::Station(id) => {
                let _ = self.stations[id.index()].take();
            }
            Place::Plot(i) => {
                if let Plot::Ripe { left, .. } = &mut self.plots[i] {
                    *left -= 1;
                    if *left == 0 {
                        self.plots[i] = Plot::Empty;
                        self.cues.push(Cue::Cleared(i));
                    }
                }
            }
            Place::Nest => self.eggs -= 1,
            Place::Pail => self.pail -= 1,
            Place::Bucket => self.bucket -= 1,
            Place::Market(food) => {
                let cost = prices::market_price(food).unwrap_or(0);
                self.coins -= cost;
                self.cues.push(Cue::Bought(food, cost));
            }
            Place::Pantry | Place::Seat(_) | Place::Compost => {}
        }
    }

    fn insert(&mut self, to: Place, food: Food) {
        match to {
            Place::Shelf(i) => self.shelve(i, food),
            Place::Pantry => {
                if let Some(i) = self.shelf_for(food) {
                    self.shelve(i, food);
                }
            }
            Place::Station(id) => {
                let _ = self.stations[id.index()].put(food);
            }
            Place::Plot(i) => {
                let rich = self.compost > 0;
                if rich {
                    self.compost -= 1;
                }
                self.plots[i] = Plot::Growing {
                    crop: food,
                    age: 0,
                    rich,
                };
                self.cues.push(Cue::Planted { plot: i, rich });
            }
            Place::Seat(i) => {
                if let Some(c) = self.seats[i].take() {
                    let tip = c.mood() > 0.5;
                    let coins = prices::price(food) + if tip { prices::TIP } else { 0 };
                    self.coins += coins;
                    self.earned += coins;
                    self.served += 1;
                    self.cues.push(Cue::Served {
                        seat: i,
                        coins,
                        tip,
                    });
                }
            }
            Place::Compost => self.compost = (self.compost + 1).min(COMPOST_MAX),
            Place::Nest | Place::Pail | Place::Bucket | Place::Market(_) => {}
        }
    }

    const fn shelve(&mut self, i: usize, food: Food) {
        self.pantry[i] = Some(match self.pantry[i] {
            Some((f, n)) => (f, n + 1),
            None => (food, 1),
        });
    }

    // Working.

    /// One stroke of work on a physical station.
    pub fn work(&mut self, id: StationId) -> Result<(), Refusal> {
        let result = if self.is_over() {
            Err(Refusal::Dusk)
        } else {
            self.stations[id.index()].work().map_err(Refusal::from)
        };
        if let Err(r) = result {
            self.cues.push(Cue::Refused(Place::Station(id), r));
        }
        self.collect_station_events();
        result
    }

    /// Light a clock station: a heated one needs fire or a log.
    pub fn start(&mut self, id: StationId) -> Result<(), Refusal> {
        let result = if self.is_over() {
            Err(Refusal::Dusk)
        } else if id.heated() && self.fire == 0 && self.logs == 0 {
            Err(Refusal::NoFuel)
        } else {
            self.stations[id.index()].start().map_err(Refusal::from)
        };
        if let Err(r) = result {
            self.cues.push(Cue::Refused(Place::Station(id), r));
        }
        self.collect_station_events();
        result
    }

    /// One stroke of a chore.
    pub fn chore(&mut self, chore: Chore) -> Result<(), Refusal> {
        let result = self.try_chore(chore);
        match result {
            Ok(done) => self.cues.push(Cue::Chore { chore, done }),
            Err(r) => self.cues.push(Cue::Balked(chore, r)),
        }
        result.map(|_| ())
    }

    const fn try_chore(&mut self, chore: Chore) -> Result<bool, Refusal> {
        if self.is_over() {
            return Err(Refusal::Dusk);
        }
        let (strokes, goal, full) = match chore {
            Chore::Milk => {
                if self.udder == 0 {
                    return Err(Refusal::Nothing);
                }
                (
                    &mut self.milk_strokes,
                    MILK_STROKES,
                    self.pail >= WAITING_MAX,
                )
            }
            Chore::Crank => (
                &mut self.crank_strokes,
                CRANK_STROKES,
                self.bucket >= WAITING_MAX,
            ),
            Chore::Axe => (&mut self.axe_strokes, AXE_STROKES, self.logs >= LOG_MAX),
        };
        if full {
            return Err(Refusal::Full);
        }
        *strokes += 1;
        if *strokes < goal {
            return Ok(false);
        }
        *strokes = 0;
        match chore {
            Chore::Milk => {
                self.udder -= 1;
                self.pail += 1;
            }
            Chore::Crank => self.bucket += 1,
            Chore::Axe => self.logs += 1,
        }
        Ok(true)
    }

    // Time.

    /// One tick of the day.
    pub fn tick(&mut self) {
        if self.is_over() {
            return;
        }
        self.clock += 1;
        self.grow();
        self.animals();
        self.hearth();
        // Clock stations off the hearth never wait for it.
        for id in StationId::ALL {
            if !id.heated() {
                self.stations[id.index()].tick();
            }
        }
        self.customers();
        self.collect_station_events();
        if self.is_over() {
            for seat in 0..SEATS {
                if self.seats[seat].take().is_some() {
                    self.cues.push(Cue::Left(seat));
                }
            }
            self.cues.push(Cue::Dusk);
        }
    }

    fn grow(&mut self) {
        for (i, plot) in self.plots.iter_mut().enumerate() {
            if let Plot::Growing { crop: c, age, rich } = plot {
                *age += 1;
                let (time, yield_) = crop(*c).unwrap_or((1, 1));
                if *age >= time {
                    let left = yield_ + if *rich { COMPOST_BONUS } else { 0 };
                    *plot = Plot::Ripe { crop: *c, left };
                    self.cues.push(Cue::Ripe(i));
                }
            }
        }
    }

    fn animals(&mut self) {
        if self.eggs < WAITING_MAX {
            self.lay += 1;
            if self.lay >= LAY {
                self.lay = 0;
                self.eggs += 1;
                self.cues.push(Cue::Laid);
            }
        }
        if self.udder < UDDER_MAX {
            self.udder_refill += 1;
            if self.udder_refill >= UDDER_REFILL {
                self.udder_refill = 0;
                self.udder += 1;
            }
        }
    }

    /// The hearth burns while anything on it is cooking, or cooked and not
    /// yet taken off. One fire heats all three at once, so a log goes
    /// further the more is on it.
    fn hearth(&mut self) {
        let wanted = StationId::ALL.iter().any(|&id| {
            id.heated()
                && match self.station(id).state() {
                    cooking::State::Idle => false,
                    cooking::State::Working { .. } => true,
                    cooking::State::Ready { food, .. } => food != Food::Charcoal,
                }
        });
        if !wanted {
            return;
        }
        if self.fire == 0 {
            if self.logs == 0 {
                return;
            }
            self.logs -= 1;
            self.fire = LOG_BURN;
            self.cues.push(Cue::Log);
        }
        self.fire -= 1;
        for id in StationId::ALL {
            if id.heated() {
                self.stations[id.index()].tick();
            }
        }
        if self.fire == 0 && self.logs == 0 {
            self.cues.push(Cue::Out);
        }
    }

    fn customers(&mut self) {
        for seat in 0..SEATS {
            if let Some(c) = &mut self.seats[seat] {
                c.waited += 1;
                if c.waited == c.patience * 3 / 4 {
                    self.cues.push(Cue::Impatient(seat));
                }
                if c.waited >= c.patience {
                    self.seats[seat] = None;
                    self.cues.push(Cue::Left(seat));
                }
            }
        }
        if self.clock >= self.next_arrival && self.clock < LAST_ORDERS {
            let free: Vec<usize> = (0..SEATS).filter(|&s| self.seats[s].is_none()).collect();
            if !free.is_empty() {
                let seat = free[self.rng.usize(..free.len())];
                let wants = self.order();
                self.seats[seat] = Some(Customer {
                    wants,
                    patience: prices::patience(wants),
                    waited: 0,
                    look: self.rng.u32(..),
                });
                self.cues.push(Cue::Arrived(seat));
                self.next_arrival = self.clock + secs(self.rng.u32(9..=16));
            }
        }
    }

    /// What the next customer wants: something simple early on, anything
    /// by the afternoon.
    fn order(&mut self) -> Food {
        // Daylight is `0..=1`, so this is a small positive number.
        #[allow(clippy::cast_sign_loss)]
        let reach = 2 + (self.daylight() * 4.0) as u32;
        let menu: Vec<Food> = Food::ALL
            .into_iter()
            .filter(|&f| f.stage() == cooking::Stage::Dish && prices::steps(f) <= reach)
            .collect();
        menu[self.rng.usize(..menu.len())]
    }

    fn collect_station_events(&mut self) {
        for id in StationId::ALL {
            for event in self.stations[id.index()].drain() {
                // Moves are already cued from the pantry's side.
                if !matches!(event, cooking::Event::Put(_) | cooking::Event::Took(_)) {
                    self.cues.push(Cue::Station(id, event));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
