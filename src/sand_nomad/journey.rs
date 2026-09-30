//! The whole game, as a pure, deterministic state machine.
//!
//! You set out from the Tether, your kin's mooring, to walk the circuit:
//! every waystone in the basin wants an offering bearing its motive, and
//! when all ten are given you sail home. What you are scored on is
//! *burden*: every watch, everything you carry weighs on you by its
//! weight, so the shorter the voyage and the lighter the hold, the better.
//! It is a travelling salesman's problem with a hold full of feelings.
//!
//! Time is the one currency the game spends on your behalf. Sailing takes
//! it (a day per [`DAY_PX`] of distance), and so does sitting with a thing
//! to learn its weight, burning a thing, or waiting while the Theseans
//! rebuild one. A day is four watches. Every watch adds the hold's weight
//! to your burden; every dawn the crew drinks a sip of water and the hold
//! grows fonder of what is in it (see [`Grid::pass_day`]).
//!
//! The frontend drives it with commands (arrange, offer, deal, set sail…)
//! and [`Journey::advance`] while sailing, and reads it back through the
//! getters and the [`Cue`]s each call leaves behind. Nothing here reads a
//! clock or a random number.

use crate::sand_nomad::barter::{self, Culture, Emote};
use crate::sand_nomad::grid::{Grid, Growth, Placed};
use crate::sand_nomad::history::{self, Happening};
use crate::sand_nomad::item::{Item, ItemId, JAR_SIPS, Kind};
use crate::sand_nomad::motive::Motive;
use crate::sand_nomad::world::{self, Ids, Site};

/// Watches in a day.
pub const WATCHES_PER_DAY: u32 = 4;

/// Map pixels sailed in a day.
pub const DAY_PX: f32 = 32.0;

/// Map pixels sailed each second of play. A day goes by in under a
/// second; the frontend may hurry it.
pub const SAIL_SPEED: f32 = 44.0;

/// Watches it takes to sit with a thing and learn its weight.
pub const APPRAISE_WATCHES: u32 = 1;
/// Watches a burning takes.
pub const BURN_WATCHES: u32 = 1;
/// Watches the Theseans take to rebuild a thing.
pub const RENEW_WATCHES: u32 = 2;

/// Burden each dawn the crew has nothing to drink.
pub const THIRST_BURDEN: u32 = 8;

/// Most things a pan holds.
pub const PAN_LIMIT: usize = 6;

/// Burden marks on the jar, best first: a light circuit, a fair one, and
/// one that got home. Set against the bot in the tests, which walks a good
/// route competently and lands between the first two.
pub const PAR: [u32; 3] = [900, 1500, 2400];

/// How many of the [`PAR`] marks a finished burden beats: 3 is the best.
#[must_use]
pub fn marks(burden: u32) -> usize {
    PAR.iter().filter(|&&par| burden <= par).count()
}

/// Where the journey stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Moored at a place.
    Camp,
    /// Under sail.
    Sailing,
    /// Home with the circuit walked. The end.
    Home,
}

/// Why a command did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No such item where it would have to be.
    Missing,
    /// That needs you moored.
    AtSea,
    /// Nobody trades here.
    NoTrader,
    /// The trader will not touch it.
    Refused,
    /// The pan is full.
    PanFull,
    /// Nothing is on the scale.
    Empty,
    /// The scale is against you.
    Short,
    /// The hold cannot take what you would get.
    NoRoom,
    /// There is no waystone here.
    NoWaystone,
    /// This waystone has had its offering.
    Given,
    /// The waystone wants another motive.
    WrongMotive,
    /// The waystone wants a thing that has been felt; this weighs nothing.
    Weightless,
    /// A living thing cannot be left at a waystone.
    Awake,
    /// No pyre here.
    NoPyre,
    /// No workbench here.
    NoBench,
    /// You already know its weight.
    Known,
    /// It has weight, and a thing with weight follows you if you drop it.
    Heavy,
    /// You are already here.
    Here,
    /// Not before the circuit is walked.
    Unfinished,
    /// The spot is taken.
    Blocked,
}

/// Something that happened, for the frontend to show or sound. Cleared at
/// the start of every command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// A watch went by.
    Watch,
    /// A new day.
    Dawn(u32),
    /// An item gathered fondness at dawn, and maybe weight.
    Grew(Growth),
    /// A woken item moved itself.
    Wandered(ItemId),
    /// The crew drank from this jar.
    Sip(ItemId),
    /// There was nothing to drink.
    Thirst,
    /// Set out for a place.
    SetSail(Site),
    /// Moored at a place.
    Arrived(Site),
    /// Moved an item within the hold.
    Arranged(ItemId),
    /// Put an item on a pan.
    Panned(ItemId),
    /// Took an item back off a pan.
    Unpanned(ItemId),
    /// The trader's face at what was just put on the scale.
    Face(Emote),
    /// A deal was struck.
    Deal,
    /// Learned an item's weight.
    Appraised(ItemId),
    /// A waystone took its offering.
    Offered(Site),
    /// Burned an item.
    Burned(ItemId),
    /// The Theseans rebuilt an item.
    Renewed(ItemId),
    /// Filled jars at a well.
    Filled,
    /// Left an item in the sand.
    Dropped(ItemId),
    /// A command came to nothing.
    No(Refusal),
    /// The last waystone is given; home is waiting.
    CircuitWalked,
    /// Home, and done.
    Finished,
    /// Something the whole basin heard.
    Heard(Happening),
}

/// A place and how it stands now.
#[derive(Clone, Debug)]
pub struct Place {
    pub site: Site,
    /// Who trades here now.
    pub trader: Option<Culture>,
    /// What they have out.
    pub rug: Grid,
    /// Whether its waystone has had its offering.
    pub given: bool,
    /// Whether you have been here.
    pub seen: bool,
}

/// A voyage under way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voyage {
    pub from: Site,
    pub to: Site,
    /// Map pixels covered.
    pub covered: f32,
    /// Map pixels in all.
    pub length: f32,
    /// Watches the whole voyage takes.
    pub watches: u32,
    /// Watches gone by so far.
    pub passed: u32,
}

impl Voyage {
    /// Where the ship is on the map.
    #[must_use]
    pub fn position(&self) -> (f32, f32) {
        let (a, b) = (self.from.pos(), self.to.pos());
        let t = (self.covered / self.length).clamp(0.0, 1.0);
        ((b.0 - a.0).mul_add(t, a.0), (b.1 - a.1).mul_add(t, a.1))
    }
}

/// Watches a voyage between two places takes.
#[must_use]
pub fn watches_between(a: Site, b: Site) -> u32 {
    let (ax, ay) = a.pos();
    let (bx, by) = b.pos();
    let watch_px = DAY_PX / WATCHES_PER_DAY as f32;
    // Distances are short and positive.
    #[allow(clippy::cast_sign_loss)]
    let watches = ((ax - bx).hypot(ay - by) / watch_px).ceil() as u32;
    watches.max(1)
}

/// The circuit.
#[derive(Clone, Debug)]
pub struct Journey {
    hold: Grid,
    places: Vec<Place>,
    at: Site,
    phase: Phase,
    voyage: Option<Voyage>,
    watch: u32,
    burden: u32,
    /// Every place moored at, in order, home first.
    wake: Vec<Site>,
    cues: Vec<Cue>,
    ids: Ids,
}

impl Default for Journey {
    fn default() -> Self {
        Self::new()
    }
}

impl Journey {
    /// Moored at home at first light, with what you set out with.
    #[must_use]
    pub fn new() -> Self {
        let mut ids = Ids::default();
        let hold = world::start_hold(&mut ids);
        let places = Site::ALL
            .iter()
            .map(|&site| Place {
                site,
                trader: site.trader(),
                rug: world::rug(site.stock(), &mut ids),
                given: false,
                seen: site == Site::Tether,
            })
            .collect();
        Self {
            hold,
            places,
            at: Site::Tether,
            phase: Phase::Camp,
            voyage: None,
            watch: 0,
            burden: 0,
            wake: vec![Site::Tether],
            cues: Vec::new(),
            ids,
        }
    }

    // What the frontend reads.

    #[must_use]
    pub const fn hold(&self) -> &Grid {
        &self.hold
    }

    #[must_use]
    pub fn places(&self) -> &[Place] {
        &self.places
    }

    #[must_use]
    pub fn place(&self, site: Site) -> &Place {
        &self.places[site.index()]
    }

    fn place_mut(&mut self, site: Site) -> &mut Place {
        &mut self.places[site.index()]
    }

    /// Where you are moored, or last moored.
    #[must_use]
    pub const fn at(&self) -> Site {
        self.at
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub const fn voyage(&self) -> Option<&Voyage> {
        self.voyage.as_ref()
    }

    /// Watches since setting out.
    #[must_use]
    pub const fn watch(&self) -> u32 {
        self.watch
    }

    /// Whole days since setting out.
    #[must_use]
    pub const fn day(&self) -> u32 {
        self.watch / WATCHES_PER_DAY
    }

    /// Days since setting out, with the fraction a voyage is part way
    /// through a watch, for drawing history smoothly.
    #[must_use]
    pub fn clock(&self) -> f32 {
        let mut watches = self.watch as f32;
        if let Some(v) = &self.voyage {
            let watch_px = v.length / v.watches as f32;
            let into = v.covered / watch_px - v.passed as f32;
            watches += into.clamp(0.0, 1.0);
        }
        watches / WATCHES_PER_DAY as f32
    }

    /// Everything carried has weighed on you this much, all told.
    #[must_use]
    pub const fn burden(&self) -> u32 {
        self.burden
    }

    /// The burden the hold adds each watch as it stands.
    #[must_use]
    pub fn load(&self) -> u32 {
        self.hold.placed().iter().map(|p| p.item.burden()).sum()
    }

    /// Where the ship is on the map.
    #[must_use]
    pub fn ship(&self) -> (f32, f32) {
        self.voyage.map_or_else(|| self.at.pos(), |v| v.position())
    }

    /// Every place moored at, in order.
    #[must_use]
    pub fn wake(&self) -> &[Site] {
        &self.wake
    }

    /// What the last command did.
    #[must_use]
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    /// Waystones given so far, and how many there are.
    #[must_use]
    pub fn circuit(&self) -> (usize, usize) {
        let stones = self.places.iter().filter(|p| p.site.waystone().is_some());
        let total = stones.clone().count();
        (stones.filter(|p| p.given).count(), total)
    }

    /// Whether every waystone has had its offering.
    #[must_use]
    pub fn walked(&self) -> bool {
        let (given, total) = self.circuit();
        given == total
    }

    /// Sips of water in the hold.
    #[must_use]
    pub fn sips(&self) -> u32 {
        self.hold
            .placed()
            .iter()
            .filter(|p| p.item.kind == Kind::Water)
            .map(|p| u32::from(p.item.sips))
            .sum()
    }

    /// Who trades where you are moored, if anyone.
    #[must_use]
    pub fn trader(&self) -> Option<Culture> {
        if self.phase == Phase::Camp {
            self.place(self.at).trader
        } else {
            None
        }
    }

    /// The rug where you are moored.
    #[must_use]
    pub fn rug(&self) -> &Grid {
        &self.place(self.at).rug
    }

    /// Your side of the scale.
    pub fn offered(&self) -> impl Iterator<Item = &Item> {
        self.hold.out().map(|p| &p.item)
    }

    /// Their side of the scale.
    pub fn asked(&self) -> impl Iterator<Item = &Item> {
        self.rug().out().map(|p| &p.item)
    }

    /// What the scale reads: positive when your side outweighs theirs by
    /// their reckoning. `None` with no trader or a refused item.
    #[must_use]
    pub fn balance(&self) -> Option<i32> {
        let culture = self.trader()?;
        barter::balance(culture, self.offered(), self.asked())
    }

    /// Whether a deal would go through now, and why not.
    pub fn deal_ready(&self) -> Result<(), Refusal> {
        if self.trader().is_none() {
            return Err(Refusal::NoTrader);
        }
        if self.offered().count() + self.asked().count() == 0 {
            return Err(Refusal::Empty);
        }
        match self.balance() {
            None => return Err(Refusal::Refused),
            Some(b) if b < 0 => return Err(Refusal::Short),
            Some(_) => {}
        }
        let incoming: Vec<Item> = self.asked().copied().collect();
        let freed: Vec<ItemId> = self.offered().map(|i| i.id).collect();
        if !self.hold.room_for(&incoming, &freed) {
            return Err(Refusal::NoRoom);
        }
        Ok(())
    }

    /// Where an item is: the hold, or the rug here.
    #[must_use]
    pub fn find(&self, id: ItemId) -> Option<&Placed> {
        self.hold.get(id).or_else(|| {
            if self.phase == Phase::Camp {
                self.rug().get(id)
            } else {
                None
            }
        })
    }

    /// How many watches a voyage to `to` would take.
    #[must_use]
    pub fn watches_to(&self, to: Site) -> u32 {
        watches_between(self.at, to)
    }

    // Commands.

    fn begin(&mut self) {
        self.cues.clear();
    }

    fn no(&mut self, why: Refusal) -> Result<(), Refusal> {
        self.cues.push(Cue::No(why));
        Err(why)
    }

    fn moored(&self) -> Result<(), Refusal> {
        if self.phase == Phase::Camp {
            Ok(())
        } else {
            Err(Refusal::AtSea)
        }
    }

    /// Move an item within the hold.
    pub fn arrange(&mut self, id: ItemId, x: i32, y: i32, turned: bool) -> Result<(), Refusal> {
        self.begin();
        if self.hold.get(id).is_none() {
            return self.no(Refusal::Missing);
        }
        if self.hold.shift(id, x, y, turned) {
            self.cues.push(Cue::Arranged(id));
            Ok(())
        } else {
            self.no(Refusal::Blocked)
        }
    }

    /// Leave something in the sand. Only a thing with no weight lets you go.
    pub fn drop_item(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        let Some(p) = self.hold.get(id) else {
            return self.no(Refusal::Missing);
        };
        if p.on_pan {
            return self.no(Refusal::Missing);
        }
        if p.item.weight > 0 {
            return self.no(Refusal::Heavy);
        }
        self.hold.take(id);
        self.cues.push(Cue::Dropped(id));
        Ok(())
    }

    /// Set a thing from the hold on your pan.
    pub fn offer(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        let Some(culture) = self.trader() else {
            return self.no(Refusal::NoTrader);
        };
        let Some(p) = self.hold.get(id).copied() else {
            return self.no(Refusal::Missing);
        };
        if p.on_pan {
            return self.no(Refusal::Missing);
        }
        if self.offered().count() >= PAN_LIMIT {
            return self.no(Refusal::PanFull);
        }
        let face = barter::reaction(culture, &p.item);
        if barter::receive(culture, &p.item).is_none() {
            self.cues.push(Cue::Face(face));
            return self.no(Refusal::Refused);
        }
        self.hold.set_out(id, true);
        self.cues.push(Cue::Panned(id));
        self.cues.push(Cue::Face(face));
        Ok(())
    }

    /// Ask for a thing from the trader's rug.
    pub fn ask(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        let Some(culture) = self.trader() else {
            return self.no(Refusal::NoTrader);
        };
        let Some(p) = self.rug().get(id).copied() else {
            return self.no(Refusal::Missing);
        };
        if p.on_pan {
            return self.no(Refusal::Missing);
        }
        if self.asked().count() >= PAN_LIMIT {
            return self.no(Refusal::PanFull);
        }
        let at = self.at;
        self.place_mut(at).rug.set_out(id, true);
        self.cues.push(Cue::Panned(id));
        self.cues.push(Cue::Face(barter::parting(culture, &p.item)));
        Ok(())
    }

    /// Take a thing back off either pan.
    pub fn withdraw(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        let at = self.at;
        if self.hold.set_out(id, false) || self.place_mut(at).rug.set_out(id, false) {
            self.cues.push(Cue::Unpanned(id));
            Ok(())
        } else {
            self.no(Refusal::Missing)
        }
    }

    /// Clear both pans.
    pub fn clear_pans(&mut self) {
        self.hold.recall();
        let at = self.at;
        self.place_mut(at).rug.recall();
    }

    /// Strike the deal on the scale: your side goes onto their rug, theirs
    /// into your hold.
    pub fn deal(&mut self) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.deal_ready() {
            return self.no(why);
        }
        let at = self.at;
        let outgoing: Vec<ItemId> = self.offered().map(|i| i.id).collect();
        let incoming: Vec<ItemId> = self.asked().map(|i| i.id).collect();
        let mut going = Vec::new();
        for id in outgoing {
            if let Some(item) = self.hold.take(id) {
                going.push(item);
            }
        }
        let mut coming = Vec::new();
        for id in incoming {
            if let Some(item) = self.place_mut(at).rug.take(id) {
                coming.push(item);
            }
        }
        // Biggest first, as `room_for` checked.
        coming.sort_by_key(|i| {
            let (w, h) = i.kind.size();
            std::cmp::Reverse(u16::from(w) * u16::from(h))
        });
        for item in coming {
            let stowed = self.hold.stow(item);
            debug_assert!(stowed.is_ok(), "room_for promised room");
        }
        for item in going {
            // A full rug means they tuck it away out of sight.
            let _ = self.place_mut(at).rug.stow(item);
        }
        self.cues.push(Cue::Deal);
        self.cues.push(Cue::Face(Emote::Bliss));
        Ok(())
    }

    /// Sit with a thing until you know its weight.
    pub fn appraise(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        let Some(p) = self.find(id) else {
            return self.no(Refusal::Missing);
        };
        if p.item.known {
            return self.no(Refusal::Known);
        }
        let at = self.at;
        if self.hold.get(id).is_some() {
            self.hold.update(id, |i| i.known = true);
        } else {
            self.place_mut(at).rug.update(id, |i| i.known = true);
        }
        self.cues.push(Cue::Appraised(id));
        self.pass_watches(APPRAISE_WATCHES);
        Ok(())
    }

    /// Leave a thing at the waystone here.
    pub fn give(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        let Some(wants) = self.at.waystone() else {
            return self.no(Refusal::NoWaystone);
        };
        if self.place(self.at).given {
            return self.no(Refusal::Given);
        }
        let Some(p) = self.hold.get(id).copied() else {
            return self.no(Refusal::Missing);
        };
        if p.on_pan {
            return self.no(Refusal::Missing);
        }
        if p.item.motive != Some(wants) {
            return self.no(Refusal::WrongMotive);
        }
        if p.item.is_anima() {
            return self.no(Refusal::Awake);
        }
        if p.item.weight == 0 {
            return self.no(Refusal::Weightless);
        }
        self.hold.take(id);
        let at = self.at;
        self.place_mut(at).given = true;
        self.cues.push(Cue::Offered(at));
        if self.walked() {
            self.cues.push(Cue::CircuitWalked);
        }
        Ok(())
    }

    /// Burn a thing on the pyre here.
    pub fn burn(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        if !self.at.pyre() {
            return self.no(Refusal::NoPyre);
        }
        match self.hold.get(id) {
            Some(p) if !p.on_pan => {}
            _ => return self.no(Refusal::Missing),
        }
        self.hold.take(id);
        self.cues.push(Cue::Burned(id));
        self.pass_watches(BURN_WATCHES);
        Ok(())
    }

    /// Have the Theseans rebuild a thing, plank by plank, until nothing of
    /// the old one and none of its weight is left.
    pub fn renew(&mut self, id: ItemId) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        if !self.at.bench() {
            return self.no(Refusal::NoBench);
        }
        match self.hold.get(id) {
            Some(p) if !p.on_pan => {}
            _ => return self.no(Refusal::Missing),
        }
        self.hold.update(id, Item::renew);
        self.cues.push(Cue::Renewed(id));
        self.pass_watches(RENEW_WATCHES);
        Ok(())
    }

    /// Fill every jar at the well here.
    pub fn fill(&mut self) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        if !self.at.well() {
            return self.no(Refusal::Missing);
        }
        let jars: Vec<ItemId> = self
            .hold
            .placed()
            .iter()
            .filter(|p| p.item.kind == Kind::Water && p.item.sips < JAR_SIPS)
            .map(|p| p.item.id)
            .collect();
        if jars.is_empty() {
            return self.no(Refusal::Missing);
        }
        for id in jars {
            self.hold.update(id, |i| i.sips = JAR_SIPS);
        }
        self.cues.push(Cue::Filled);
        Ok(())
    }

    /// Cast off for `to`.
    pub fn set_sail(&mut self, to: Site) -> Result<(), Refusal> {
        self.begin();
        if let Err(why) = self.moored() {
            return self.no(why);
        }
        if to == self.at {
            return self.no(Refusal::Here);
        }
        self.clear_pans();
        let (ax, ay) = self.at.pos();
        let (bx, by) = to.pos();
        self.voyage = Some(Voyage {
            from: self.at,
            to,
            covered: 0.0,
            length: (ax - bx).hypot(ay - by),
            watches: watches_between(self.at, to),
            passed: 0,
        });
        self.phase = Phase::Sailing;
        self.cues.push(Cue::SetSail(to));
        Ok(())
    }

    /// Sail on for `secs` seconds of play. Watches pass as the ship crosses
    /// them, and exactly [`watches_between`] of them pass by the time it
    /// moors, however the seconds were sliced.
    pub fn advance(&mut self, secs: f32) {
        self.begin();
        let Some(mut v) = self.voyage else {
            return;
        };
        v.covered = secs.max(0.0).mul_add(SAIL_SPEED, v.covered).min(v.length);
        let watch_px = v.length / v.watches as f32;
        // Positive and bounded by the voyage.
        #[allow(clippy::cast_sign_loss)]
        let due = if v.covered >= v.length {
            v.watches
        } else {
            ((v.covered / watch_px) as u32).min(v.watches)
        };
        let to_pass = due - v.passed;
        v.passed = due;
        self.voyage = Some(v);
        self.pass_watches(to_pass);
        if v.covered >= v.length {
            self.moor(v.to);
        }
    }

    fn moor(&mut self, site: Site) {
        self.voyage = None;
        self.at = site;
        self.wake.push(site);
        self.place_mut(site).seen = true;
        if site == Site::SpadeReef {
            self.settle_reef();
        }
        if site == Site::Tether && self.walked() {
            self.phase = Phase::Home;
            self.cues.push(Cue::Arrived(site));
            self.cues.push(Cue::Finished);
        } else {
            self.phase = Phase::Camp;
            self.cues.push(Cue::Arrived(site));
        }
    }

    /// Once the raid has happened, the pirates are gone from the reef and
    /// a Harbor sergeant sells off what they left.
    fn settle_reef(&mut self) {
        let raided = history::reef_taken(self.day() as f32);
        let reef = self.place(Site::SpadeReef);
        if raided && reef.trader == Some(Culture::Pirate) {
            let rug = world::rug(Site::stock_after_raid(), &mut self.ids);
            let reef = self.place_mut(Site::SpadeReef);
            reef.trader = Some(Culture::Harbor);
            reef.rug = rug;
        }
    }

    /// Let `count` watches go by.
    fn pass_watches(&mut self, count: u32) {
        for _ in 0..count {
            self.watch += 1;
            self.burden += self.load();
            self.cues.push(Cue::Watch);
            if self.watch % WATCHES_PER_DAY == 0 {
                self.dawn();
            }
        }
    }

    fn dawn(&mut self) {
        let day = self.day();
        self.cues.push(Cue::Dawn(day));
        // The crew drinks first, from the emptiest jar that has any.
        let jar = self
            .hold
            .placed()
            .iter()
            .filter(|p| p.item.kind == Kind::Water && p.item.sips > 0)
            .min_by_key(|p| (p.item.sips, p.item.id))
            .map(|p| p.item.id);
        if let Some(id) = jar {
            self.hold.update(id, |i| i.sips -= 1);
            self.cues.push(Cue::Sip(id));
        } else {
            self.burden += THIRST_BURDEN;
            self.cues.push(Cue::Thirst);
        }
        let (growth, wanders) = self.hold.pass_day(day);
        for g in growth {
            self.cues.push(Cue::Grew(g));
        }
        for w in wanders {
            self.cues.push(Cue::Wandered(w.id));
        }
        for happening in history::dawn_of(day) {
            self.cues.push(Cue::Heard(happening));
        }
    }

    /// The motive the waystone here wants, if it still wants one.
    #[must_use]
    pub fn waystone_here(&self) -> Option<Motive> {
        if self.phase != Phase::Camp || self.place(self.at).given {
            return None;
        }
        self.at.waystone()
    }
}

#[cfg(test)]
mod tests;
