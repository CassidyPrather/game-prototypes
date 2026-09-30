//! Who wants what: the basin's cultures and how each prices a thing.
//!
//! There is no money in the basin, only the scale. A trader weighs what you
//! set on your pan against what you asked for on theirs, each by their own
//! [`Taste`]: materials they prize, motives they revere or shun, a few
//! things they want outright, and — the part that makes the basin the basin
//! — how they feel about *weight*.
//!
//! Most nomads dread it and will pay less, or nothing, for a heavy thing.
//! The Pyre-Kin feed it to their ships and pay extra. The Crimson Harbor
//! does not believe in it at all, which makes its people the easiest place
//! in the basin to leave a heavy thing, and the pirates have stopped
//! caring.
//!
//! There is a manners rule too: nobody speaks of the weight of what *they*
//! are handing over. A trader prices their own goods as if they weighed
//! nothing, so a heavy thing comes to you looking like a bargain. Only
//! sitting with it (appraisal, which costs time) tells you what it is.

use crate::sand_nomad::item::{Item, JAR_SIPS, Kind};
use crate::sand_nomad::motive::Motive;

/// Extra a trader will give for a thing they want outright.
pub const WANT_BONUS: i32 = 3;

/// What a trader knocks off a thing they cannot stand.
pub const SPURN_PENALTY: i32 = 2;

/// A people of the basin, and so a way of trading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Culture {
    /// The ordinary tent-dwelling nomads: sentimental, so weight-shy.
    Drifter,
    /// The family that stayed at the old lighthouse on the ancient shore.
    Keeper,
    /// Shipwrights who replace every plank as it gathers weight, until
    /// nothing of the old ship is left and it is still the same ship.
    Thesean,
    /// Salt-scrapers on the white flats.
    Scraper,
    /// Reef pirates. Hardened past caring about weight.
    Pirate,
    /// The Crimson Harbor, which believes in order and not in weight.
    Harbor,
    /// A hermit sifting the drowned republic's senate.
    Hermit,
    /// Nomads whose ships are alive and their friends, and who burn weight
    /// to feed them.
    PyreKin,
    /// Nobody: bones in the sand. Take what you like, leave what you like.
    Bones,
}

/// How a culture prices things.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Taste {
    /// Added per material, in
    /// [`Material::index`](crate::sand_nomad::item::Material::index) order.
    pub material: [i32; 10],
    /// Added per motive, in [`Motive::index`] order.
    pub motive: [i32; 6],
    /// Added per point of weight, when they are the ones receiving it.
    pub per_weight: i32,
    /// What a woken thing is worth to them on top of its price, or `None`
    /// if they will not touch one.
    pub anima: Option<i32>,
    /// Kinds they want outright.
    pub wants: &'static [Kind],
    /// Kinds they cannot stand.
    pub spurns: &'static [Kind],
}

impl Culture {
    /// Every culture.
    pub const ALL: [Self; 9] = [
        Self::Drifter,
        Self::Keeper,
        Self::Thesean,
        Self::Scraper,
        Self::Pirate,
        Self::Harbor,
        Self::Hermit,
        Self::PyreKin,
        Self::Bones,
    ];

    /// How this culture prices things.
    #[must_use]
    pub const fn taste(self) -> Taste {
        //                  Wd Cl Mt Br Ce Bn Pa Gl Sa Re
        //                  Bl Rp Lv Pn Zl Ht
        match self {
            Self::Drifter => Taste {
                material: [0, 2, 0, -1, 1, 0, 0, 1, 1, -1],
                motive: [2, 0, 2, -1, 0, -2],
                per_weight: -2,
                anima: None,
                wants: &[Kind::Rug, Kind::Salt],
                spurns: &[Kind::Coins, Kind::Tin],
            },
            Self::Keeper => Taste {
                material: [0, 0, 0, 1, 2, 0, 1, 2, 0, 0],
                motive: [0, 2, 1, 0, -2, 0],
                per_weight: -1,
                anima: None,
                wants: &[Kind::Lamp, Kind::GlassFlower],
                spurns: &[Kind::Pistol],
            },
            Self::Thesean => Taste {
                material: [2, 1, 1, 0, 0, 0, 0, 0, 0, 0],
                motive: [0, 2, 0, 0, -2, 0],
                per_weight: -1,
                anima: None,
                wants: &[Kind::Plank, Kind::Rope],
                spurns: &[],
            },
            Self::Scraper => Taste {
                material: [0, 1, 0, 0, 1, 0, 0, 0, -1, 0],
                motive: [2, 0, 0, -2, 0, 0],
                per_weight: -2,
                anima: None,
                wants: &[Kind::Water],
                spurns: &[Kind::Salt],
            },
            Self::Pirate => Taste {
                material: [0, 0, 2, 1, 0, 0, 0, 0, 0, 0],
                motive: [0, -1, -2, 0, 2, 2],
                per_weight: 0,
                anima: Some(3),
                wants: &[Kind::Pistol, Kind::Coins],
                spurns: &[],
            },
            Self::Harbor => Taste {
                material: [-1, 0, 1, 2, 1, -1, 2, 0, 0, 3],
                motive: [0; 6],
                per_weight: 0,
                anima: Some(0),
                wants: &[Kind::Bust, Kind::Map],
                spurns: &[],
            },
            Self::Hermit => Taste {
                material: [0, 0, 0, 0, 0, 0, 2, 0, 1, 2],
                motive: [0, 1, 0, 2, -2, 0],
                per_weight: -1,
                anima: None,
                wants: &[Kind::Map, Kind::Letters],
                spurns: &[Kind::Coins],
            },
            Self::PyreKin => Taste {
                material: [2, 1, 0, -1, 0, 1, 0, 0, 0, 0],
                motive: [0, -2, 0, 2, 2, 0],
                per_weight: 2,
                anima: Some(10),
                wants: &[Kind::Plank, Kind::Oar],
                spurns: &[Kind::Tin],
            },
            Self::Bones => Taste {
                material: [0; 10],
                motive: [0; 6],
                per_weight: 0,
                anima: Some(0),
                wants: &[],
                spurns: &[],
            },
        }
    }

    /// Whether this culture believes in weight. The Harbor does not, and
    /// bones have no opinions.
    #[must_use]
    pub const fn practises(self) -> bool {
        !matches!(self, Self::Harbor | Self::Bones)
    }
}

/// A face a trader makes, drawn with the motive emotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Emote {
    /// They love it.
    Love,
    /// Pleased.
    Bliss,
    /// Indifferent.
    Repose,
    /// They want exactly that.
    Zeal,
    /// They dislike it.
    Hate,
    /// It is too heavy for them.
    Pain,
    /// It is alive, and they want nothing to do with it.
    Anima,
}

/// A thing's price before anyone thinks about weight.
fn base_price(culture: Culture, item: &Item) -> i32 {
    if culture == Culture::Bones {
        return 0;
    }
    let taste = culture.taste();
    let kind = item.kind;
    let mut price = kind.worth() + taste.material[kind.material().index()];
    if let Some(motive) = item.motive {
        price += taste.motive[motive.index()];
    }
    if taste.wants.contains(&kind) {
        price += WANT_BONUS;
    }
    if taste.spurns.contains(&kind) {
        price -= SPURN_PENALTY;
    }
    if kind == Kind::Water {
        // An empty jar is only a jar.
        price = price * i32::from(item.sips) / i32::from(JAR_SIPS);
    }
    price
}

/// What `item` is worth to `culture` if you hand it over, or `None` if they
/// will not take it.
#[must_use]
pub fn receive(culture: Culture, item: &Item) -> Option<i32> {
    let taste = culture.taste();
    let base = base_price(culture, item);
    if item.is_anima() {
        return taste.anima.map(|bonus| base + bonus);
    }
    Some(base + taste.per_weight * i32::from(item.weight))
}

/// What `culture` asks for `item` of theirs. Weight goes unmentioned. A
/// thing they cannot stand may cost less than nothing: they will pay to be
/// rid of it.
#[must_use]
pub fn give(culture: Culture, item: &Item) -> i32 {
    base_price(culture, item).max(-2)
}

/// The face `culture` makes at being handed `item`.
#[must_use]
pub fn reaction(culture: Culture, item: &Item) -> Emote {
    let Some(price) = receive(culture, item) else {
        return Emote::Anima;
    };
    let taste = culture.taste();
    if taste.wants.contains(&item.kind) && price > 0 {
        return Emote::Zeal;
    }
    if price < 0 {
        let weightless = base_price(culture, item);
        return if weightless >= 0 && item.weight > 0 {
            Emote::Pain
        } else {
            Emote::Hate
        };
    }
    match price {
        5.. => Emote::Love,
        2..=4 => Emote::Bliss,
        _ => Emote::Repose,
    }
}

/// The face `culture` makes at being asked for `item` of theirs: glad to
/// see the back of it, or not bothered.
#[must_use]
pub fn parting(culture: Culture, item: &Item) -> Emote {
    if give(culture, item) <= 0 && culture != Culture::Bones {
        Emote::Bliss
    } else {
        Emote::Repose
    }
}

/// The scale's reading: what you offer minus what you ask, both by the
/// trader's reckoning, or `None` if they refuse something you offered.
#[must_use]
pub fn balance<'a>(
    culture: Culture,
    offered: impl IntoIterator<Item = &'a Item>,
    asked: impl IntoIterator<Item = &'a Item>,
) -> Option<i32> {
    let mut total = 0;
    for item in offered {
        total += receive(culture, item)?;
    }
    for item in asked {
        total -= give(culture, item);
    }
    Some(total)
}

/// Which way the motive cuts for a culture: revered, shunned, or neither.
#[must_use]
pub const fn regard(culture: Culture, motive: Motive) -> i32 {
    culture.taste().motive[motive.index()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sand_nomad::item::ItemId;

    fn item(kind: Kind, motive: Option<Motive>, weight: u8) -> Item {
        Item::new(ItemId(0), kind, motive, weight)
    }

    #[test]
    fn nomads_dread_weight_and_the_harbor_does_not_notice() {
        let light = item(Kind::Locket, Some(Motive::Love), 0);
        let heavy = item(Kind::Locket, Some(Motive::Love), 4);
        let drifter_light = receive(Culture::Drifter, &light).unwrap();
        let drifter_heavy = receive(Culture::Drifter, &heavy).unwrap();
        assert!(drifter_heavy < drifter_light);
        assert!(drifter_heavy < 0, "a heavy locket is a burden to a drifter");
        assert_eq!(receive(Culture::Harbor, &light), receive(Culture::Harbor, &heavy));
        assert!(
            receive(Culture::PyreKin, &heavy) > receive(Culture::PyreKin, &light),
            "the pyre-kin feed weight to their ships"
        );
    }

    #[test]
    fn nobody_mentions_the_weight_of_what_they_hand_over() {
        let light = item(Kind::Locket, Some(Motive::Love), 0);
        let heavy = item(Kind::Locket, Some(Motive::Love), 5);
        for culture in Culture::ALL {
            assert_eq!(give(culture, &light), give(culture, &heavy), "{culture:?}");
        }
    }

    #[test]
    fn only_some_will_touch_a_living_thing() {
        let awake = item(Kind::Doll, Some(Motive::Bliss), 6);
        for culture in Culture::ALL {
            let taken = receive(culture, &awake).is_some();
            let expected = matches!(
                culture,
                Culture::Harbor | Culture::PyreKin | Culture::Pirate | Culture::Bones
            );
            assert_eq!(taken, expected, "{culture:?}");
        }
        assert_eq!(reaction(Culture::Drifter, &awake), Emote::Anima);
    }

    #[test]
    fn faces_say_why() {
        let rug = item(Kind::Rug, Some(Motive::Bliss), 0);
        assert_eq!(reaction(Culture::Drifter, &rug), Emote::Zeal);
        let heavy = item(Kind::Comb, Some(Motive::Love), 4);
        assert_eq!(reaction(Culture::Drifter, &heavy), Emote::Pain);
        let dagger = item(Kind::Dagger, Some(Motive::Hate), 0);
        assert_eq!(reaction(Culture::Drifter, &dagger), Emote::Repose);
        let flag = item(Kind::Flag, Some(Motive::Zeal), 0);
        assert_eq!(reaction(Culture::Thesean, &flag), Emote::Repose);
        let pistol = item(Kind::Pistol, Some(Motive::Zeal), 0);
        assert_eq!(parting(Culture::Keeper, &pistol), Emote::Bliss, "glad to be rid");
        assert!(give(Culture::Keeper, &pistol) < 0);
    }

    #[test]
    fn the_bones_ask_nothing_and_want_nothing() {
        let bust = item(Kind::Bust, Some(Motive::Pain), 3);
        assert_eq!(give(Culture::Bones, &bust), 0);
        assert_eq!(receive(Culture::Bones, &bust), Some(0));
        assert_eq!(balance(Culture::Bones, [&bust], [&bust]), Some(0));
    }

    #[test]
    fn an_empty_jar_is_only_a_jar() {
        let mut jar = item(Kind::Water, None, 0);
        let full = receive(Culture::Scraper, &jar).unwrap();
        jar.sips = 0;
        assert!(full > 3, "water is what the flats want");
        assert_eq!(receive(Culture::Scraper, &jar), Some(0));
    }
}
