//! The day's rules, and a bot that plays one through.
//!
//! The bot is the balance check: a careful, unhurried cook who works one
//! order at a time, keeps the fire fed and the garden planted. It must
//! reach the first mark and not the last — the last is for players who
//! juggle. A cook who does nothing must earn nothing. Run with
//! `--nocapture` to see how its day went.

// Tests count small things.
#![allow(clippy::cast_possible_truncation)]

use super::*;
use crate::cooking::{Drive, recipe};

fn shelf_of(day: &Day, food: Food) -> Option<usize> {
    day.pantry()
        .iter()
        .position(|s| matches!(s, Some((f, _)) if *f == food))
}

fn held(day: &Day, food: Food) -> u32 {
    day.pantry()
        .iter()
        .flatten()
        .filter(|(f, _)| *f == food)
        .map(|(_, n)| u32::from(*n))
        .sum()
}

fn run(day: &mut Day, ticks: u32) {
    for _ in 0..ticks {
        day.tick();
    }
}

#[test]
fn a_refused_move_changes_nothing() {
    let mut day = Day::default();
    let before = (*day.pantry(), day.coins(), *day.plots());
    // Nobody is at the hatch, the market is not a destination, an empty
    // shelf has nothing to give.
    assert!(day.move_food(Place::Shelf(0), Place::Seat(0)).is_err());
    assert!(
        day.move_food(Place::Shelf(0), Place::Market(Food::Salt))
            .is_err()
    );
    assert!(day.move_food(Place::Shelf(7), Place::Compost).is_err());
    // A plot only takes something that grows.
    assert_eq!(
        day.move_food(Place::Shelf(2), Place::Plot(4)),
        Err(Refusal::Unwanted)
    );
    assert_eq!((*day.pantry(), day.coins(), *day.plots()), before);
    assert!(
        day.drain().iter().all(|c| matches!(c, Cue::Refused(..))),
        "a refusal cued something else"
    );
}

#[test]
fn planting_costs_a_seed_and_compost_makes_it_richer() {
    let mut day = Day::default();
    let rice = shelf_of(&day, Food::Rice).unwrap();
    day.move_food(Place::Shelf(rice), Place::Plot(4)).unwrap();
    assert_eq!(held(&day, Food::Rice), 1);
    // Compost first, then plant the last of the rice.
    let egg = shelf_of(&day, Food::Egg).unwrap();
    day.move_food(Place::Shelf(egg), Place::Compost).unwrap();
    assert_eq!(day.compost(), 1);
    let rice = shelf_of(&day, Food::Rice).unwrap();
    day.move_food(Place::Shelf(rice), Place::Plot(5)).unwrap();
    assert_eq!(day.compost(), 0);
    let (grow, yield_) = crop(Food::Rice).unwrap();
    run(&mut day, grow);
    assert_eq!(
        day.plots()[4],
        Plot::Ripe {
            crop: Food::Rice,
            left: yield_
        }
    );
    assert_eq!(
        day.plots()[5],
        Plot::Ripe {
            crop: Food::Rice,
            left: yield_ + COMPOST_BONUS
        }
    );
    assert_eq!(day.stow_all(Place::Plot(4)), usize::from(yield_));
    assert_eq!(day.plots()[4], Plot::Empty);
    assert_eq!(held(&day, Food::Rice), u32::from(yield_));
}

#[test]
fn the_hearth_needs_wood_and_one_fire_heats_everything() {
    let mut day = Day::default();
    // Burn the logs away on an empty-handed boil, a log at a time.
    day.move_food(Place::Bucket, Place::Station(StationId::Pot))
        .unwrap();
    day.start(StationId::Pot).unwrap();
    let logs = day.logs();
    run(&mut day, 1);
    assert_eq!(day.logs(), logs - 1, "lighting took no log");
    assert!(day.drain().contains(&Cue::Log));
    // Two things on the same fire burn no faster than one.
    let egg = shelf_of(&day, Food::Egg).unwrap();
    day.move_food(Place::Shelf(egg), Place::Station(StationId::Pan))
        .unwrap();
    day.start(StationId::Pan).unwrap();
    run(&mut day, LOG_BURN - 1);
    assert_eq!(day.logs(), logs - 1, "a second pan burned a second log");
    // With no wood at all, nothing can be lit.
    let mut cold = Day {
        logs: 0,
        ..Day::default()
    };
    cold.move_food(Place::Bucket, Place::Station(StationId::Pot))
        .unwrap();
    assert_eq!(cold.start(StationId::Pot), Err(Refusal::NoFuel));
}

#[test]
fn a_dish_left_on_a_dead_fire_does_not_burn() {
    let mut day = Day::default();
    let egg = shelf_of(&day, Food::Egg).unwrap();
    day.move_food(Place::Shelf(egg), Place::Station(StationId::Pan))
        .unwrap();
    day.start(StationId::Pan).unwrap();
    run(&mut day, secs(5));
    assert_eq!(day.station(StationId::Pan).ready(), Some(Food::FriedEgg));
    // The last of the last log.
    day.fire = 1;
    day.logs = 0;
    run(&mut day, secs(60));
    assert_eq!(day.station(StationId::Pan).ready(), Some(Food::FriedEgg));
    assert!(day.fire() <= 0.0, "the fire is still lit");
}

#[test]
fn the_market_takes_coins_and_turns_the_poor_away() {
    let mut day = Day {
        coins: 3,
        ..Day::default()
    };
    day.stow_all(Place::Market(Food::Sugar));
    assert_eq!(day.coins(), 1);
    assert_eq!(held(&day, Food::Sugar), 1);
    assert_eq!(
        day.move_food(Place::Market(Food::Spice), Place::Pantry),
        Err(Refusal::Poor)
    );
    assert_eq!(day.coins(), 1);
}

#[test]
fn chores_make_things_and_stop_when_full() {
    let mut day = Day::default();
    for _ in 0..CRANK_STROKES {
        day.chore(Chore::Crank).unwrap();
    }
    assert_eq!(day.bucket(), 2);
    assert_eq!(day.chore(Chore::Crank), Err(Refusal::Full));
    for _ in 0..MILK_STROKES * 2 {
        day.chore(Chore::Milk).unwrap();
    }
    assert_eq!(day.pail(), 2);
    assert_eq!(day.udder(), 0);
    day.stow_all(Place::Pail);
    assert_eq!(day.chore(Chore::Milk), Err(Refusal::Nothing), "milked dry");
    run(&mut day, UDDER_REFILL);
    assert!(day.chore(Chore::Milk).is_ok(), "the udder never refilled");
}

#[test]
fn serving_pays_the_bill_price_and_a_tip_when_quick() {
    let mut day = Day::default();
    run(&mut day, secs(3));
    let seat = (0..SEATS).find(|&s| day.seats()[s].is_some()).unwrap();
    let wants = day.seats()[seat].unwrap().wants;
    // Conjure the dish; the test is of the hatch, not the kitchen.
    day.pantry[7] = Some((wants, 1));
    let coins = day.coins();
    day.move_food(Place::Shelf(7), Place::Seat(seat)).unwrap();
    assert_eq!(
        day.coins(),
        coins + prices::price(wants) + prices::TIP,
        "{wants:?}"
    );
    assert!(day.seats()[seat].is_none());
    assert_eq!(day.served(), 1);
}

#[test]
fn customers_give_up_and_dusk_sends_everyone_home() {
    let mut day = Day::default();
    run(&mut day, secs(3));
    let seat = (0..SEATS).find(|&s| day.seats()[s].is_some()).unwrap();
    let patience = day.seats()[seat].unwrap().patience;
    run(&mut day, patience);
    let cues = day.drain();
    assert!(cues.contains(&Cue::Impatient(seat)));
    assert!(cues.contains(&Cue::Left(seat)));
    run(&mut day, DAY);
    assert!(day.is_over());
    assert!(day.seats().iter().all(Option::is_none));
    assert_eq!(day.chore(Chore::Axe), Err(Refusal::Dusk));
    assert!(day.move_food(Place::Bucket, Place::Pantry).is_err());
}

#[test]
fn early_customers_want_simple_things() {
    for seed in 0..20 {
        let mut day = Day::new(seed);
        run(&mut day, secs(4));
        for c in day.seats().iter().flatten() {
            assert!(prices::steps(c.wants) <= 2, "{:?} at dawn", c.wants);
        }
    }
}

#[test]
fn a_monkey_cannot_break_it() {
    // Arbitrary commands at arbitrary places: nothing may panic, and
    // nothing may go missing from the counters.
    let places = |rng: &mut fastrand::Rng| match rng.u8(0..9) {
        0 => Place::Shelf(rng.usize(0..PANTRY + 2)),
        1 => Place::Pantry,
        2 => Place::Station(StationId::ALL[rng.usize(0..7)]),
        3 => Place::Plot(rng.usize(0..=PLOTS)),
        4 => Place::Nest,
        5 => [Place::Pail, Place::Bucket][rng.usize(0..2)],
        6 => Place::Market(Food::ALL[rng.usize(0..Food::COUNT)]),
        7 => Place::Seat(rng.usize(0..=SEATS)),
        _ => Place::Compost,
    };
    for seed in 0..6 {
        let mut rng = fastrand::Rng::with_seed(seed);
        let mut day = Day::new(seed);
        for _ in 0..DAY + secs(5) {
            match rng.u8(0..12) {
                0..=3 => {
                    let (from, to) = (places(&mut rng), places(&mut rng));
                    let _ = day.move_food(from, to);
                }
                4 => {
                    let _ = day.stow_all(places(&mut rng));
                }
                5 | 6 => {
                    let _ = day.work(StationId::ALL[rng.usize(0..7)]);
                }
                7 => {
                    let _ = day.start(StationId::ALL[rng.usize(0..7)]);
                }
                8 => {
                    let _ = day.chore([Chore::Milk, Chore::Crank, Chore::Axe][rng.usize(0..3)]);
                }
                _ => {}
            }
            day.tick();
            assert!(
                day.pantry()
                    .iter()
                    .flatten()
                    .all(|&(_, n)| n > 0 && n <= STACK)
            );
            assert!(day.logs() <= LOG_MAX && day.compost() <= COMPOST_MAX);
            assert!(day.eggs() <= WAITING_MAX && day.bucket() <= WAITING_MAX);
            let _ = day.drain();
        }
        assert!(day.is_over());
    }
}

// The bot.

/// Ticks between the bot's actions: five a second, a busy human's pace.
const ACT_EVERY: u32 = 4;

/// What happened when the bot tried to make something.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// It did something toward it.
    Acted,
    /// It is on its way; nothing to do for it right now.
    Waiting,
    /// It cannot be made from here.
    Stuck,
}

fn station_for(process: cooking::Process) -> StationId {
    StationId::ALL
        .into_iter()
        .find(|s| s.processes().contains(&process))
        .expect("every process has a station")
}

struct Bot {
    day: Day,
}

impl Bot {
    /// One action, or none.
    fn act(&mut self) {
        let _ = self.serve()
            || self.clear_waste()
            || self.rescue()
            || self.fuel(2)
            || self.cook()
            || self.harvest()
            || self.plant()
            || self.potter();
    }

    fn serve(&mut self) -> bool {
        for seat in 0..SEATS {
            let Some(c) = self.day.seats()[seat] else {
                continue;
            };
            for id in StationId::ALL {
                if self.day.station(id).ready() == Some(c.wants) {
                    return self
                        .day
                        .move_food(Place::Station(id), Place::Seat(seat))
                        .is_ok();
                }
            }
            if let Some(shelf) = shelf_of(&self.day, c.wants) {
                return self
                    .day
                    .move_food(Place::Shelf(shelf), Place::Seat(seat))
                    .is_ok();
            }
        }
        false
    }

    fn clear_waste(&mut self) -> bool {
        for id in StationId::ALL {
            if matches!(
                self.day.station(id).ready(),
                Some(Food::Mush | Food::Charcoal)
            ) {
                return self
                    .day
                    .move_food(Place::Station(id), Place::Compost)
                    .is_ok();
            }
        }
        false
    }

    /// Take anything finished off the hearth before it burns.
    fn rescue(&mut self) -> bool {
        for id in StationId::ALL {
            if id.heated() && self.day.station(id).scorch() > 0.3 {
                return self.stow(Place::Station(id));
            }
        }
        false
    }

    fn fuel(&mut self, want: u8) -> bool {
        self.day.logs() < want && self.day.chore(Chore::Axe).is_ok()
    }

    /// Stow, composting the most plentiful shelf to make room if needed.
    fn stow(&mut self, from: Place) -> bool {
        if self.day.move_food(from, Place::Pantry).is_ok() {
            return true;
        }
        let fullest = (0..PANTRY).max_by_key(|&i| self.day.pantry()[i].map_or(0, |(_, n)| n));
        fullest.is_some_and(|i| self.day.move_food(Place::Shelf(i), Place::Compost).is_ok())
    }

    /// Work toward the most hurried order that can be made.
    fn cook(&mut self) -> bool {
        let mut orders: Vec<Customer> = self.day.seats().iter().flatten().copied().collect();
        orders.sort_by(|a, b| a.mood().total_cmp(&b.mood()));
        for order in orders {
            match self.toward(order.wants, 0) {
                Step::Acted => return true,
                Step::Waiting => return false,
                Step::Stuck => {}
            }
        }
        false
    }

    fn toward(&mut self, food: Food, depth: u32) -> Step {
        if depth > 6 {
            return Step::Stuck;
        }
        if shelf_of(&self.day, food).is_some() {
            return Step::Waiting;
        }
        if food.stage() == cooking::Stage::Raw {
            return self.gather(food);
        }
        let Some(recipe) = recipe::making(food).next() else {
            return Step::Stuck;
        };
        let id = station_for(recipe.process);
        let station = self.day.station(id);
        match station.state() {
            cooking::State::Ready { food: done, .. } => {
                if self.stow(Place::Station(id)) {
                    Step::Acted
                } else if done == food {
                    Step::Waiting
                } else {
                    Step::Stuck
                }
            }
            cooking::State::Working { recipe: r, .. } => {
                let ours = r.is_some_and(|r| r.output == food);
                if ours && station.drive() == Drive::Effort {
                    let _ = self.day.work(id);
                    Step::Acted
                } else {
                    Step::Waiting
                }
            }
            cooking::State::Idle => {
                let contents = station.contents().to_vec();
                if !recipe::within(&contents, recipe.inputs) {
                    return if self.stow(Place::Station(id)) {
                        Step::Acted
                    } else {
                        Step::Stuck
                    };
                }
                let missing = recipe::missing(recipe, &contents);
                let Some(&next) = missing.first() else {
                    let started = match station.drive() {
                        Drive::Effort => self.day.work(id),
                        Drive::Clock => self.day.start(id),
                    };
                    return match started {
                        Ok(()) => Step::Acted,
                        Err(Refusal::NoFuel) if self.fuel(LOG_MAX) => Step::Acted,
                        Err(_) => Step::Stuck,
                    };
                };
                for &input in &missing {
                    if let Some(shelf) = shelf_of(&self.day, input) {
                        let _ = self.day.move_food(Place::Shelf(shelf), Place::Station(id));
                        return Step::Acted;
                    }
                }
                self.toward(next, depth + 1)
            }
        }
    }

    fn gather(&mut self, food: Food) -> Step {
        let acted = |ok: bool| if ok { Step::Acted } else { Step::Stuck };
        let day = &self.day;
        match food {
            Food::Water if day.bucket() > 0 => acted(self.stow(Place::Bucket)),
            Food::Water => acted(self.day.chore(Chore::Crank).is_ok()),
            Food::Milk if day.pail() > 0 => acted(self.stow(Place::Pail)),
            Food::Milk if day.udder() > 0 => acted(self.day.chore(Chore::Milk).is_ok()),
            Food::Egg if day.eggs() > 0 => acted(self.stow(Place::Nest)),
            Food::Milk | Food::Egg => Step::Waiting,
            _ if MARKET.contains(&food) => acted(self.day.stow_all(Place::Market(food)) > 0),
            _ => {
                let plots = *day.plots();
                let ripe = plots
                    .iter()
                    .position(|p| matches!(p, Plot::Ripe { crop, .. } if *crop == food));
                let growing = plots
                    .iter()
                    .any(|p| matches!(p, Plot::Growing { crop, .. } if *crop == food));
                match (ripe, growing) {
                    (Some(i), _) => acted(self.stow(Place::Plot(i))),
                    (None, true) => Step::Waiting,
                    (None, false) => Step::Stuck,
                }
            }
        }
    }

    fn harvest(&mut self) -> bool {
        let ripe = self
            .day
            .plots()
            .iter()
            .position(|p| matches!(p, Plot::Ripe { .. }));
        ripe.is_some_and(|i| self.day.stow_all(Place::Plot(i)) > 0)
    }

    /// Plant whatever crop there is most of, keeping one back to cook.
    fn plant(&mut self) -> bool {
        let Some(plot) = self.day.plots().iter().position(|p| *p == Plot::Empty) else {
            return false;
        };
        let seed = CROPS
            .into_iter()
            .filter(|&c| held(&self.day, c) >= 2)
            .max_by_key(|&c| held(&self.day, c))
            .or_else(|| {
                // A crop with nothing growing and only one left is planted
                // rather than lost to the garden for good.
                CROPS.into_iter().find(|&c| {
                    held(&self.day, c) == 1
                        && !self.day.plots().iter().any(
                            |p| matches!(p, Plot::Growing { crop, .. } | Plot::Ripe { crop, .. } if *crop == c),
                        )
                })
            });
        seed.and_then(|c| shelf_of(&self.day, c))
            .is_some_and(|shelf| {
                self.day
                    .move_food(Place::Shelf(shelf), Place::Plot(plot))
                    .is_ok()
            })
    }

    /// With nothing better to do, fill the bucket, the pail and the woodpile.
    fn potter(&mut self) -> bool {
        self.day.chore(Chore::Crank).is_ok()
            || self.fuel(4)
            || (self.day.pail() < WAITING_MAX && self.day.chore(Chore::Milk).is_ok())
    }
}

fn bot_day(seed: u64) -> Day {
    let mut bot = Bot {
        day: Day::new(seed),
    };
    while !bot.day.is_over() {
        if bot.day.clock() % ACT_EVERY == 0 {
            bot.act();
        }
        bot.day.tick();
        let _ = bot.day.drain();
    }
    bot.day
}

#[test]
fn a_careful_cook_makes_the_first_mark_but_not_the_last() {
    let mut total = 0;
    for seed in 0..8 {
        let day = bot_day(seed);
        println!(
            "seed {seed}: {} coins, {} served, {} marks",
            day.coins(),
            day.served(),
            day.marks()
        );
        assert!(
            day.marks() >= 1,
            "seed {seed}: the bot missed the first mark"
        );
        assert!(day.marks() < 3, "seed {seed}: the bot took every mark");
        total += day.coins();
    }
    println!("mean {} coins", total / 8);
}

#[test]
fn a_cook_who_does_nothing_earns_nothing() {
    let mut day = Day::default();
    run(&mut day, DAY);
    assert_eq!(day.coins(), START_COINS);
    assert_eq!(day.marks(), 0);
}
