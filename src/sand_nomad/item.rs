//! Things a nomad can carry, and the weight they gather.
//!
//! An [`Item`] is a [`Kind`] (what it is: its shape in the hold, its
//! material, what it is worth to anyone who does not care about weight),
//! an optional [`Motive`] painted on it, and a weight, `0..=6`. Weight is
//! the emotional attachment the basin believes gathers in things. At six
//! the thing wakes up: it is *anima*, alive, and a nuisance.
//!
//! Weight is invisible. The motive is painted on where anyone can see it,
//! but how heavy a thing is can only be learned by sitting with it, which
//! takes time — see `journey`'s appraisal. An item remembers whether its
//! weight is known.

use crate::sand_nomad::motive::Motive;

/// Weight at which an item wakes up.
pub const ANIMA_WEIGHT: u8 = 6;

/// Fondness an item gathers per point of weight. A lone item gains one a
/// day, so a lone item grows a point of weight every this many days.
pub const FONDNESS_PER_WEIGHT: u8 = 4;

/// Sips in a full water jar. The crew drinks one a day.
pub const JAR_SIPS: u8 = 6;

/// What a woken item costs each watch, in place of its weight: a living
/// thing in the hold is a good deal more bother than a heavy one.
pub const ANIMA_BURDEN: u32 = 12;

/// Identifies one item for the whole journey, wherever it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(pub u16);

/// What something is made of. Cultures price materials differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Material {
    Wood,
    Cloth,
    Metal,
    Brass,
    Ceramic,
    Bone,
    Paper,
    Glass,
    Salt,
    /// Anything the old republic left behind.
    Relic,
}

impl Material {
    /// Every material, in [`Material::index`] order.
    pub const ALL: [Self; 10] = [
        Self::Wood,
        Self::Cloth,
        Self::Metal,
        Self::Brass,
        Self::Ceramic,
        Self::Bone,
        Self::Paper,
        Self::Glass,
        Self::Salt,
        Self::Relic,
    ];

    /// Position in [`Material::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Wood => 0,
            Self::Cloth => 1,
            Self::Metal => 2,
            Self::Brass => 3,
            Self::Ceramic => 4,
            Self::Bone => 5,
            Self::Paper => 6,
            Self::Glass => 7,
            Self::Salt => 8,
            Self::Relic => 9,
        }
    }
}

/// What an item is. Each has its own sprite in the frontend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A water jar. The one thing everyone needs; it never gathers weight.
    Water,
    Flute,
    Comb,
    Beads,
    Rug,
    Compass,
    Plank,
    Letters,
    Spyglass,
    Shell,
    MusicBox,
    Doll,
    Locket,
    Kettle,
    Rope,
    /// A little model of a sand ship.
    Figurine,
    Oar,
    Salt,
    /// Lightning glass from the flats.
    GlassFlower,
    Dagger,
    Flag,
    Pistol,
    /// A purse of the Harbor's coin.
    Coins,
    /// A Harbor ration tin.
    Tin,
    /// A rib from the leviathan.
    Rib,
    Tooth,
    /// A clockwork songbird.
    Clockbird,
    Mask,
    Idol,
    /// A bust of some republic senator.
    Bust,
    /// A republic medallion.
    Medal,
    Map,
    Lamp,
    Charm,
}

impl Kind {
    /// Every kind, for the frontend to bake a sprite for each.
    pub const ALL: [Self; 34] = [
        Self::Water,
        Self::Flute,
        Self::Comb,
        Self::Beads,
        Self::Rug,
        Self::Compass,
        Self::Plank,
        Self::Letters,
        Self::Spyglass,
        Self::Shell,
        Self::MusicBox,
        Self::Doll,
        Self::Locket,
        Self::Kettle,
        Self::Rope,
        Self::Figurine,
        Self::Oar,
        Self::Salt,
        Self::GlassFlower,
        Self::Dagger,
        Self::Flag,
        Self::Pistol,
        Self::Coins,
        Self::Tin,
        Self::Rib,
        Self::Tooth,
        Self::Clockbird,
        Self::Mask,
        Self::Idol,
        Self::Bust,
        Self::Medal,
        Self::Map,
        Self::Lamp,
        Self::Charm,
    ];

    /// Footprint in hold cells, unrotated: columns, rows.
    #[must_use]
    pub const fn size(self) -> (u8, u8) {
        match self {
            Self::Rug | Self::Plank | Self::Spyglass | Self::Oar => (2, 1),
            Self::Rib => (1, 2),
            Self::Bust => (2, 2),
            _ => (1, 1),
        }
    }

    /// What it is made of.
    #[must_use]
    pub const fn material(self) -> Material {
        match self {
            Self::Beads | Self::Plank | Self::Figurine | Self::Oar | Self::Mask | Self::Idol => {
                Material::Wood
            }
            Self::Rug | Self::Doll | Self::Rope | Self::Flag => Material::Cloth,
            Self::Locket | Self::Dagger | Self::Pistol | Self::Coins | Self::Tin => {
                Material::Metal
            }
            Self::Compass | Self::Spyglass | Self::MusicBox | Self::Clockbird => Material::Brass,
            Self::Water | Self::Kettle | Self::Lamp => Material::Ceramic,
            Self::Flute | Self::Comb | Self::Shell | Self::Rib | Self::Tooth | Self::Charm => {
                Material::Bone
            }
            Self::Letters | Self::Map => Material::Paper,
            Self::GlassFlower => Material::Glass,
            Self::Salt => Material::Salt,
            Self::Bust | Self::Medal => Material::Relic,
        }
    }

    /// What it is worth to someone who only sees the thing itself.
    #[must_use]
    pub const fn worth(self) -> i32 {
        match self {
            Self::Comb
            | Self::Beads
            | Self::Plank
            | Self::Letters
            | Self::Shell
            | Self::Doll
            | Self::Rope
            | Self::Salt
            | Self::Tin
            | Self::Idol
            | Self::Charm
            | Self::Water => 1,
            Self::Flute
            | Self::Kettle
            | Self::Figurine
            | Self::Oar
            | Self::GlassFlower
            | Self::Dagger
            | Self::Flag
            | Self::Rib
            | Self::Tooth
            | Self::Mask
            | Self::Medal
            | Self::Map
            | Self::Lamp => 2,
            Self::Rug
            | Self::Compass
            | Self::Spyglass
            | Self::MusicBox
            | Self::Locket
            | Self::Pistol
            | Self::Coins => 3,
            Self::Clockbird => 4,
            Self::Bust => 6,
        }
    }
}

/// One thing, wherever it is: in the hold, on a pan, in a trader's stock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: ItemId,
    pub kind: Kind,
    /// The motive painted on it, if any. Things without one — water, the
    /// Harbor's orderly wares — never gather weight.
    pub motive: Option<Motive>,
    /// Attachment, `0..=ANIMA_WEIGHT`.
    pub weight: u8,
    /// Progress toward the next point of weight, `0..FONDNESS_PER_WEIGHT`.
    pub fondness: u8,
    /// Whether the player has learned its weight.
    pub known: bool,
    /// Water left, for a jar; zero for anything else.
    pub sips: u8,
}

impl Item {
    /// A new item of `kind` with `motive` and `weight`, its weight unknown.
    #[must_use]
    pub const fn new(id: ItemId, kind: Kind, motive: Option<Motive>, weight: u8) -> Self {
        let sips = if matches!(kind, Kind::Water) {
            JAR_SIPS
        } else {
            0
        };
        Self {
            id,
            kind,
            motive,
            weight,
            fondness: 0,
            known: motive.is_none(),
            sips,
        }
    }

    /// Whether it has woken up.
    #[must_use]
    pub const fn is_anima(&self) -> bool {
        self.weight >= ANIMA_WEIGHT
    }

    /// Whether it can gather weight at all.
    #[must_use]
    pub const fn can_grow(&self) -> bool {
        self.motive.is_some() && !self.is_anima()
    }

    /// What carrying it costs each watch.
    #[must_use]
    pub const fn burden(&self) -> u32 {
        if self.is_anima() {
            ANIMA_BURDEN
        } else {
            self.weight as u32
        }
    }

    /// Footprint in cells, columns then rows, turned or not.
    #[must_use]
    pub const fn footprint(&self, turned: bool) -> (u8, u8) {
        let (w, h) = self.kind.size();
        if turned { (h, w) } else { (w, h) }
    }

    /// Whether its footprint changes when turned.
    #[must_use]
    pub const fn can_turn(&self) -> bool {
        let (w, h) = self.kind.size();
        w != h
    }

    /// Add a day's fondness, returning whether that added a point of
    /// weight. Items that cannot grow ignore it.
    pub const fn gather(&mut self, fondness: u8) -> u8 {
        if !self.can_grow() {
            return 0;
        }
        let mut gained = 0;
        let mut total = self.fondness + fondness;
        while total >= FONDNESS_PER_WEIGHT && self.weight < ANIMA_WEIGHT {
            total -= FONDNESS_PER_WEIGHT;
            self.weight += 1;
            gained += 1;
        }
        self.fondness = if self.weight >= ANIMA_WEIGHT { 0 } else { total };
        gained
    }

    /// Forget every attachment: the Theseans' rebuild.
    pub const fn renew(&mut self) {
        self.weight = 0;
        self.fondness = 0;
        self.known = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: Kind, motive: Option<Motive>, weight: u8) -> Item {
        Item::new(ItemId(0), kind, motive, weight)
    }

    #[test]
    fn fondness_turns_into_weight_and_then_life() {
        let mut comb = item(Kind::Comb, Some(Motive::Love), 4);
        assert_eq!(comb.gather(3), 0);
        assert_eq!(comb.gather(1), 1);
        assert_eq!(comb.weight, 5);
        assert!(!comb.is_anima());
        // A big helping of fondness stops at waking.
        assert_eq!(comb.gather(40), 1);
        assert!(comb.is_anima());
        assert_eq!(comb.burden(), ANIMA_BURDEN);
        assert_eq!(comb.gather(40), 0, "a woken thing grows no heavier");
    }

    #[test]
    fn things_without_a_motive_never_gather_weight() {
        let mut jar = item(Kind::Water, None, 0);
        assert_eq!(jar.gather(100), 0);
        assert_eq!(jar.weight, 0);
        assert!(jar.known, "orderly things hide nothing");
        assert_eq!(jar.sips, JAR_SIPS);
    }

    #[test]
    fn turning_swaps_a_long_footprint() {
        let oar = item(Kind::Oar, None, 0);
        assert_eq!(oar.footprint(false), (2, 1));
        assert_eq!(oar.footprint(true), (1, 2));
        assert!(oar.can_turn());
        assert!(!item(Kind::Bust, None, 0).can_turn());
    }

    #[test]
    fn every_kind_is_listed_once() {
        for (i, a) in Kind::ALL.iter().enumerate() {
            for b in &Kind::ALL[i + 1..] {
                assert_ne!(a, b);
            }
            assert!(a.worth() > 0);
        }
        for (i, m) in Material::ALL.iter().enumerate() {
            assert_eq!(m.index(), i);
        }
    }
}
