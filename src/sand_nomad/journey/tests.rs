//! The journey's rules, and a bot that walks the circuit to prove it can be
//! walked, in reasonable time, better by a good route than a bad one.

use super::*;
use crate::sand_nomad::barter::{self, receive};
use crate::sand_nomad::item::ANIMA_WEIGHT;

/// Sail to `to` in slices of play, as the frontend would.
fn sail(j: &mut Journey, to: Site) {
    j.set_sail(to).expect("can cast off");
    let mut guard = 0;
    while j.phase() == Phase::Sailing {
        j.advance(1.0 / 60.0);
        guard += 1;
        assert!(guard < 60 * 120, "a voyage that never ends");
    }
}

fn find_kind(j: &Journey, kind: Kind) -> ItemId {
    j.hold()
        .placed()
        .iter()
        .find(|p| p.item.kind == kind)
        .map(|p| p.item.id)
        .expect("that kind is in the hold")
}

#[test]
fn a_voyage_takes_the_same_watches_however_it_is_sliced() {
    for slice in [1.0 / 144.0, 1.0 / 60.0, 1.0 / 7.0, 0.5, 30.0] {
        let mut j = Journey::new();
        j.set_sail(Site::SpadeReef).unwrap();
        while j.phase() == Phase::Sailing {
            j.advance(slice);
        }
        assert_eq!(j.watch(), watches_between(Site::Tether, Site::SpadeReef));
        assert_eq!(j.at(), Site::SpadeReef);
    }
}

#[test]
fn every_watch_weighs_what_the_hold_weighs() {
    let mut j = Journey::new();
    let load = j.load();
    assert!(load > 0, "you set out carrying something");
    let flute = find_kind(&j, Kind::Flute);
    j.appraise(flute).unwrap_err();
    // A voyage short enough that nothing grows mid-way.
    sail(&mut j, Site::WellOfTeeth);
    let watches = j.watch();
    assert!(watches >= WATCHES_PER_DAY, "long enough to see a dawn");
    assert!(j.burden() >= load * watches);
}

#[test]
fn the_crew_drinks_and_thirsts() {
    let mut j = Journey::new();
    let sips = j.sips();
    sail(&mut j, Site::SpadeReef);
    assert_eq!(j.sips(), sips - j.day());
    // Pour everything away and see what a dry dawn costs.
    let jars: Vec<ItemId> = j
        .hold()
        .placed()
        .iter()
        .filter(|p| p.item.kind == Kind::Water)
        .map(|p| p.item.id)
        .collect();
    for id in jars {
        j.hold.update(id, |i| i.sips = 0);
    }
    let before = j.burden();
    let day = j.day();
    sail(&mut j, Site::Leviathan);
    let days = j.day() - day;
    assert!(days > 0);
    assert!(j.burden() >= before + days * THIRST_BURDEN);
}

#[test]
fn waystones_want_their_motive_and_a_thing_that_has_been_felt() {
    let mut j = Journey::new();
    sail(&mut j, Site::Lighthouse);
    let comb = find_kind(&j, Kind::Comb);
    assert_eq!(j.give(comb), Err(Refusal::WrongMotive));
    let rug = find_kind(&j, Kind::Rug);
    j.hold.update(rug, |i| i.weight = 0);
    assert_eq!(j.give(rug), Err(Refusal::Weightless));
    let flute = find_kind(&j, Kind::Flute);
    j.hold.update(flute, |i| i.weight = ANIMA_WEIGHT);
    assert_eq!(j.give(flute), Err(Refusal::Awake));
    j.hold.update(flute, |i| i.weight = 2);
    assert_eq!(j.give(flute), Ok(()));
    assert!(j.place(Site::Lighthouse).given);
    assert_eq!(j.give(rug), Err(Refusal::Given));
    assert_eq!(j.circuit().0, 1);
}

#[test]
fn a_heavy_thing_will_not_be_left_behind() {
    let mut j = Journey::new();
    let comb = find_kind(&j, Kind::Comb);
    assert_eq!(j.drop_item(comb), Err(Refusal::Heavy));
    let compass = find_kind(&j, Kind::Compass);
    assert_eq!(j.drop_item(compass), Ok(()));
    assert!(j.hold().get(compass).is_none());
}

#[test]
fn appraising_costs_a_watch_and_reveals_the_weight() {
    let mut j = Journey::new();
    sail(&mut j, Site::Hearthring);
    let locket = j
        .rug()
        .placed()
        .iter()
        .find(|p| p.item.kind == Kind::Locket)
        .unwrap()
        .item;
    assert!(!locket.known);
    let before = j.watch();
    j.appraise(locket.id).unwrap();
    assert_eq!(j.watch(), before + APPRAISE_WATCHES);
    assert!(j.rug().get(locket.id).unwrap().item.known);
    assert_eq!(j.appraise(locket.id), Err(Refusal::Known));
}

#[test]
fn the_scale_decides_and_the_goods_change_hands() {
    let mut j = Journey::new();
    sail(&mut j, Site::Hearthring);
    let doll = j
        .rug()
        .placed()
        .iter()
        .find(|p| p.item.kind == Kind::Doll)
        .unwrap()
        .item
        .id;
    j.ask(doll).unwrap();
    assert_eq!(j.deal(), Err(Refusal::Short));
    let rug = find_kind(&j, Kind::Rug);
    j.offer(rug).unwrap();
    assert!(
        j.cues().contains(&Cue::Face(Emote::Zeal)),
        "drifters want rugs"
    );
    assert!(j.balance().unwrap() >= 0);
    j.deal().unwrap();
    assert!(j.hold().get(doll).is_some());
    assert!(j.hold().get(rug).is_none());
    assert!(j.rug().get(rug).is_some());
    assert!(
        !j.hold().get(doll).unwrap().item.known,
        "bought sight unseen"
    );
}

#[test]
fn nomads_will_not_touch_a_living_thing_but_the_harbor_will() {
    let mut j = Journey::new();
    let comb = find_kind(&j, Kind::Comb);
    j.hold.update(comb, |i| i.weight = ANIMA_WEIGHT);
    sail(&mut j, Site::Hearthring);
    assert_eq!(j.offer(comb), Err(Refusal::Refused));
    assert!(j.cues().contains(&Cue::Face(Emote::Anima)));
    sail(&mut j, Site::Fort);
    assert_eq!(j.offer(comb), Ok(()));
}

#[test]
fn the_reef_changes_hands_after_the_raid() {
    let mut early = Journey::new();
    sail(&mut early, Site::SpadeReef);
    assert_eq!(early.trader(), Some(Culture::Pirate));
    let mut late = Journey::new();
    late.pass_watches(world::RAID_DAY * WATCHES_PER_DAY);
    sail(&mut late, Site::SpadeReef);
    assert_eq!(late.trader(), Some(Culture::Harbor));
    // Every id in the basin is still its own.
    let mut ids: Vec<u16> = late.hold().placed().iter().map(|p| p.item.id.0).collect();
    for place in late.places() {
        ids.extend(place.rug.placed().iter().map(|p| p.item.id.0));
    }
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count);
}

#[test]
fn burning_and_rebuilding_cost_time() {
    let mut j = Journey::new();
    let comb = find_kind(&j, Kind::Comb);
    j.burn(comb).unwrap();
    assert!(j.hold().get(comb).is_none());
    assert_eq!(j.watch(), BURN_WATCHES);
    sail(&mut j, Site::Drydock);
    let beads = find_kind(&j, Kind::Beads);
    let before = j.watch();
    j.renew(beads).unwrap();
    assert_eq!(j.hold().get(beads).unwrap().item.weight, 0);
    assert_eq!(j.watch(), before + RENEW_WATCHES);
}

#[test]
fn home_before_the_circuit_is_only_another_stop() {
    let mut j = Journey::new();
    sail(&mut j, Site::WellOfTeeth);
    sail(&mut j, Site::Tether);
    assert_eq!(j.phase(), Phase::Camp);
}

// The bot.

/// A player who knows the basin: walks a route, gives each waystone the
/// lightest thing that will do, buys what later waystones need where it
/// can, sheds weight where someone will take it, and keeps the jars wet.
struct Bot {
    j: Journey,
    appraisals: u32,
    deals: u32,
}

impl Bot {
    fn new() -> Self {
        Self {
            j: Journey::new(),
            appraisals: 0,
            deals: 0,
        }
    }

    /// Motives still wanted by waystones not yet given, in route order
    /// from here.
    fn needed(&self, route: &[Site]) -> Vec<Motive> {
        route
            .iter()
            .filter(|s| !self.j.place(**s).given)
            .filter_map(|s| s.waystone())
            .collect()
    }

    fn hold_items(&self) -> Vec<Item> {
        self.j.hold().placed().iter().map(|p| p.item).collect()
    }

    /// Things in the hold earmarked for a later waystone: the lightest
    /// usable one per outstanding motive.
    fn earmarked(&self, route: &[Site]) -> Vec<ItemId> {
        let mut out = Vec::new();
        for motive in self.needed(route) {
            let pick = self
                .hold_items()
                .into_iter()
                .filter(|i| i.motive == Some(motive) && !i.is_anima() && !out.contains(&i.id))
                .min_by_key(|i| i.weight);
            if let Some(i) = pick {
                out.push(i.id);
            }
        }
        out
    }

    fn visit(&mut self, route: &[Site], rest: &[Site]) {
        let here = self.j.at();
        if self.j.at().well() {
            let _ = self.j.fill();
        }
        if let Some(culture) = self.j.trader() {
            let mut from_here = vec![here];
            from_here.extend_from_slice(rest);
            self.trade(culture, &from_here);
        }
        // Give the waystone the lightest thing that will do.
        if let Some(wants) = self.j.waystone_here() {
            let pick = self
                .hold_items()
                .into_iter()
                .filter(|i| i.motive == Some(wants) && i.weight > 0 && !i.is_anima())
                .min_by_key(|i| i.weight);
            if let Some(i) = pick {
                self.j.give(i.id).unwrap();
            }
        }
        // Burn anything awake, or nearly, that is not needed.
        if here.pyre() {
            let keep = self.earmarked(rest);
            for i in self.hold_items() {
                if i.weight + 1 >= ANIMA_WEIGHT && !keep.contains(&i.id) {
                    self.j.burn(i.id).unwrap();
                }
            }
        }
        let _ = route;
    }

    /// The best any trader later on the route would give for `item`.
    fn later_price(&self, item: Item, rest: &[Site]) -> i32 {
        rest.iter()
            .filter_map(|s| self.j.place(*s).trader)
            .filter_map(|c| receive(c, &item))
            .max()
            .unwrap_or(0)
    }

    fn trade(&mut self, culture: Culture, rest: &[Site]) {
        // Buy: one thing per motive still needed and not in hand.
        let keep = self.earmarked(rest);
        let have: Vec<Motive> = keep
            .iter()
            .filter_map(|id| self.j.hold().get(*id).and_then(|p| p.item.motive))
            .collect();
        let mut wanted: Vec<Motive> = self.needed(rest);
        wanted.retain(|m| !have.contains(m));
        wanted.dedup();
        for motive in wanted {
            let candidates: Vec<Item> = self
                .j
                .rug()
                .placed()
                .iter()
                .map(|p| p.item)
                .filter(|i| i.motive == Some(motive))
                .collect();
            // Sit with each to learn which is lightest.
            for c in &candidates {
                if !c.known {
                    self.j.appraise(c.id).unwrap();
                    self.appraisals += 1;
                }
            }
            let Some(pick) = candidates
                .iter()
                .filter(|c| c.weight < ANIMA_WEIGHT - 1)
                .min_by_key(|c| c.weight)
            else {
                continue;
            };
            self.buy(pick.id, rest);
        }
        // Water: enough for the next two legs.
        let need = rest
            .windows(2)
            .take(2)
            .map(|w| watches_between(w[0], w[1]) / WATCHES_PER_DAY + 1)
            .sum::<u32>()
            + rest.first().map_or(0, |&s| {
                watches_between(self.j.at(), s) / WATCHES_PER_DAY + 1
            });
        while self.j.sips() < need {
            let jar = self
                .j
                .rug()
                .placed()
                .iter()
                .find(|p| p.item.kind == Kind::Water && p.item.sips > 0)
                .map(|p| p.item.id);
            let Some(jar) = jar else { break };
            if !self.buy(jar, rest) {
                break;
            }
        }
        // Speculate: weightless things someone down the road wants more.
        let bargains: Vec<Item> = self
            .j
            .rug()
            .placed()
            .iter()
            .map(|p| p.item)
            .filter(|i| i.motive.is_none() && i.kind != Kind::Water)
            .filter(|i| self.later_price(*i, rest) - barter::give(culture, i) >= 3)
            .collect();
        for item in bargains {
            let free = self.free_cells();
            if free < 4 {
                break;
            }
            self.buy(item.id, rest);
        }
        // Shed: leave anything heavy and unneeded with them if they will
        // take it at all.
        let keep = self.earmarked(rest);
        let shed: Vec<Item> = self
            .hold_items()
            .into_iter()
            .filter(|i| i.weight > 0 && !keep.contains(&i.id))
            .filter(|i| receive(culture, i).is_some_and(|v| v >= 0))
            .collect();
        for i in &shed {
            let _ = self.j.offer(i.id);
        }
        if self.j.deal_ready().is_ok() {
            self.j.deal().unwrap();
            self.deals += 1;
        }
        self.j.clear_pans();
    }

    fn free_cells(&self) -> usize {
        let hold = self.j.hold();
        let used: usize = hold
            .placed()
            .iter()
            .map(|p| {
                let (w, h) = p.size();
                usize::from(w) * usize::from(h)
            })
            .sum();
        usize::from(hold.width()) * usize::from(hold.height()) - used
    }

    /// Ask for `id` and pay with whatever is least missed: heavy things
    /// first, then whatever this trader prizes over everyone later.
    fn buy(&mut self, id: ItemId, rest: &[Site]) -> bool {
        let culture = self.j.trader().unwrap();
        self.j.clear_pans();
        if self.j.ask(id).is_err() {
            return false;
        }
        let keep = self.earmarked(rest);
        let mut payment: Vec<Item> = self
            .hold_items()
            .into_iter()
            .filter(|i| !keep.contains(&i.id) && i.kind != Kind::Water)
            .filter(|i| receive(culture, i).is_some_and(|v| v > 0))
            .collect();
        payment.sort_by_key(|i| {
            let here = receive(culture, i).unwrap();
            (i.weight == 0, self.later_price(*i, rest) - here, here)
        });
        for i in payment {
            if self.j.balance().is_some_and(|b| b >= 0) {
                break;
            }
            let _ = self.j.offer(i.id);
        }
        // Take the change in water, while it lasts and fits.
        let jars: Vec<Item> = self
            .j
            .rug()
            .present()
            .map(|p| p.item)
            .filter(|i| i.kind == Kind::Water && i.sips > 0)
            .collect();
        for jar in jars {
            let spare = self.j.balance().unwrap_or(0) - barter::give(culture, &jar);
            if spare < 0 || self.free_cells() < 3 {
                break;
            }
            let _ = self.j.ask(jar.id);
            if self.j.deal_ready().is_err() {
                let _ = self.j.withdraw(jar.id);
            }
        }
        if self.j.deal_ready().is_ok() {
            self.j.deal().unwrap();
            self.deals += 1;
            true
        } else {
            self.j.clear_pans();
            false
        }
    }

    fn walk(mut self, route: &[Site]) -> Self {
        for (i, &stop) in route.iter().enumerate() {
            sail(&mut self.j, stop);
            let rest = &route[i + 1..];
            self.visit(route, rest);
            if std::env::var("BOT_TRACE").is_ok() {
                let hold: Vec<String> = self
                    .hold_items()
                    .iter()
                    .map(|i| format!("{:?}/{:?}/{}", i.kind, i.motive, i.weight))
                    .collect();
                println!(
                    "day {:>2} {:?} given {} load {} sips {} | {}",
                    self.j.day(),
                    stop,
                    self.j.place(stop).given,
                    self.j.load(),
                    self.j.sips(),
                    hold.join(" ")
                );
            }
        }
        self
    }
}

/// A route a thoughtful player might take: round the rim, down the east,
/// back through the middle. Hate is only for sale at the reef, so the reef
/// comes before the fort.
const GOOD_ROUTE: [Site; 11] = [
    Site::Lighthouse,
    Site::Hearthring,
    Site::Drydock,
    Site::SaltFlats,
    Site::SpadeReef,
    Site::Leviathan,
    Site::Fort,
    Site::Ashfleet,
    Site::Senate,
    Site::WellOfTeeth,
    Site::Tether,
];

/// The same places in a careless order.
const CARELESS_ROUTE: [Site; 11] = [
    Site::SaltFlats,
    Site::WellOfTeeth,
    Site::Drydock,
    Site::Senate,
    Site::Lighthouse,
    Site::SpadeReef,
    Site::Hearthring,
    Site::Fort,
    Site::Leviathan,
    Site::Ashfleet,
    Site::Tether,
];

fn report(name: &str, bot: &Bot) {
    let (given, total) = bot.j.circuit();
    println!(
        "{name}: {given}/{total} waystones, {} days, burden {}, {} deals, {} appraisals, phase {:?}",
        bot.j.day(),
        bot.j.burden(),
        bot.deals,
        bot.appraisals,
        bot.j.phase(),
    );
}

#[test]
fn a_bot_can_walk_the_circuit() {
    let bot = Bot::new().walk(&GOOD_ROUTE);
    report("good route", &bot);
    let (given, total) = bot.j.circuit();
    let missing: Vec<Site> = bot
        .j
        .places()
        .iter()
        .filter(|p| p.site.waystone().is_some() && !p.given)
        .map(|p| p.site)
        .collect();
    assert_eq!(given, total, "the bot missed {missing:?}");
    assert_eq!(bot.j.phase(), Phase::Home);
}

#[test]
fn a_good_route_weighs_less_than_a_careless_one() {
    let good = Bot::new().walk(&GOOD_ROUTE);
    let careless = Bot::new().walk(&CARELESS_ROUTE);
    report("good", &good);
    report("careless", &careless);
    assert!(good.j.day() < careless.j.day());
    assert!(good.j.burden() < careless.j.burden());
}

#[test]
fn the_circuit_fits_in_under_an_hour() {
    // Sailing time is play time; everything else is roughly a click a
    // few seconds apart. Generous per-action allowances still fit.
    let bot = Bot::new().walk(&GOOD_ROUTE);
    let sailing_secs = GOOD_ROUTE
        .iter()
        .scan(Site::Tether, |at, &to| {
            let (a, b) = (at.pos(), to.pos());
            *at = to;
            Some((a.0 - b.0).hypot(a.1 - b.1) / SAIL_SPEED)
        })
        .sum::<f32>();
    let actions = bot.deals * 6 + bot.appraisals + 11 * 4;
    let minutes = (actions as f32).mul_add(6.0, sailing_secs) / 60.0;
    println!("roughly {minutes:.0} minutes of play, {sailing_secs:.0} s of it sailing");
    assert!(minutes < 45.0);
}

#[test]
fn the_marks_are_earned() {
    // A competent walk of a good route is fair, not light: the best mark
    // needs a better route or cleverer packing than the bot manages.
    let bot = Bot::new().walk(&GOOD_ROUTE);
    assert_eq!(marks(bot.j.burden()), 2, "burden {}", bot.j.burden());
    assert_eq!(marks(0), 3);
    assert_eq!(marks(u32::MAX), 0);
}
