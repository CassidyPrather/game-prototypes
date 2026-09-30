//! The six motives: what the basin believes weight is made of.
//!
//! Three opposing axes around a hexagon, in the order the rune draws them
//! clockwise from the top left: Bliss, Repose, Love, Pain, Zeal, Hate. Each
//! motive sits across the hexagon from its opposite, so Bliss faces Pain,
//! Repose faces Zeal and Love faces Hate.
//!
//! Nobody can prove the motives are real. They popped off anyway, and now
//! every nomad paints them on what they own and every camp keeps waystones
//! that want them.

/// One of the six motives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Motive {
    /// The gold chalice.
    Bliss,
    /// The blue club.
    Repose,
    /// The red heart.
    Love,
    /// The purple spike.
    Pain,
    /// The orange diamond.
    Zeal,
    /// The black spade.
    Hate,
}

impl Motive {
    /// Every motive, clockwise around the rune from the top left.
    pub const ALL: [Self; 6] = [
        Self::Bliss,
        Self::Repose,
        Self::Love,
        Self::Pain,
        Self::Zeal,
        Self::Hate,
    ];

    /// Position around the rune, `0..6`.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Bliss => 0,
            Self::Repose => 1,
            Self::Love => 2,
            Self::Pain => 3,
            Self::Zeal => 4,
            Self::Hate => 5,
        }
    }

    /// The motive across the hexagon. Items bearing opposite motives
    /// quarrel when packed together, and neither grows fonder.
    #[must_use]
    pub const fn opposite(self) -> Self {
        Self::ALL[(self.index() + 3) % 6]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposites_pair_up_across_the_rune() {
        assert_eq!(Motive::Bliss.opposite(), Motive::Pain);
        assert_eq!(Motive::Repose.opposite(), Motive::Zeal);
        assert_eq!(Motive::Love.opposite(), Motive::Hate);
        for motive in Motive::ALL {
            assert_eq!(motive.opposite().opposite(), motive);
            assert_ne!(motive.opposite(), motive);
            assert_eq!(Motive::ALL[motive.index()], motive);
        }
    }
}
