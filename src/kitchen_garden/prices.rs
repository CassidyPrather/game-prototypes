//! What things are worth, derived from the cooking module's bills.
//!
//! This is the demo of the tuning hook: no dish has a hand-set price or
//! patience. Each is computed from [`cooking::bill`], so a recipe made
//! longer or richer reprices itself, and a customer waits in proportion to
//! how long their order takes. Change the weights here and every dish
//! follows.

use crate::cooking::{Food, Stage, TICKS_PER_SEC, bill};

/// What a raw ingredient is worth, in coins. Water is free: the well only
/// asks for work.
#[must_use]
pub const fn raw_value(food: Food) -> u32 {
    match food {
        Food::Water => 0,
        Food::Sugar | Food::Spice => 2,
        _ => 1,
    }
}

/// What the market charges for what it sells, and `None` for what it does
/// not. Only things the garden cannot grow are for sale.
#[must_use]
pub const fn market_price(food: Food) -> Option<u32> {
    match food {
        Food::Salt => Some(1),
        Food::Sugar | Food::Spice => Some(2),
        _ => None,
    }
}

/// What a customer pays for a dish: its raw ingredients back, and a coin
/// for every step it took. Zero for anything that is not a dish.
#[must_use]
pub fn price(food: Food) -> u32 {
    if food.stage() != Stage::Dish {
        return 0;
    }
    bill(food).map_or(0, |b| b.raw_value(raw_value) + b.steps)
}

/// Extra for serving while the customer is still more than half patient.
pub const TIP: u32 = 1;

/// How long a customer will wait for a dish, in ticks: a base, plus the
/// dish's own clock time, plus a second for every two strokes of work.
#[must_use]
pub fn patience(food: Food) -> u32 {
    const BASE_SECS: u32 = 40;
    bill(food).map_or(0, |b| {
        BASE_SECS * TICKS_PER_SEC + b.clock() + b.effort * TICKS_PER_SEC / 2
    })
}

/// How involved a dish is, for deciding when in the day it is ordered:
/// the number of steps from the ground.
#[must_use]
pub fn steps(food: Food) -> u32 {
    bill(food).map_or(0, |b| b.steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dish_is_worth_more_than_what_went_into_it() {
        for food in Food::ALL {
            if food.stage() == Stage::Dish {
                let b = bill(food).unwrap();
                assert!(price(food) > b.raw_value(raw_value), "{food:?}");
            }
        }
    }

    #[test]
    fn the_market_sells_at_value() {
        for food in Food::ALL {
            if let Some(p) = market_price(food) {
                assert_eq!(p, raw_value(food), "{food:?}");
            }
        }
    }

    #[test]
    fn bigger_dishes_pay_more_and_are_waited_for_longer() {
        assert!(price(Food::Cake) > price(Food::Tea));
        assert!(patience(Food::Bread) > patience(Food::FriedEgg));
        assert_eq!(price(Food::Flour), 0, "an ingredient has a price");
    }
}
