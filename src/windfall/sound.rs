//! What Windfall sounds like: which recorded sample answers which cue, and
//! how loud.
//!
//! The samples themselves are CC0 recordings in `assets/windfall/` (see
//! `CREDITS.md`); the frontend embeds and plays them. The gains live here,
//! in the library, so they are reviewed in one place.

/// One sample in the bank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sample {
    /// A leaf let go of the branch, in two takes so a flurry does not
    /// sound like a machine.
    Release(u8),
    /// A vane blowing a leaf along, in two takes.
    Gust(u8),
    /// A flip vane turning round.
    Flip,
    /// A leaf landing in its own basket.
    Catch,
    /// A leaf lost: wrong basket, bare ground, blown away, or eddying.
    Miss,
    /// A piece set down in the garden.
    Place,
    /// A piece taken up.
    Remove,
    /// Every leaf home.
    Solved,
    /// One mark, rung after the garden is solved.
    Mark,
    /// The garden's air, looped under everything.
    Ambience,
}

impl Sample {
    /// Every sample, for baking the bank.
    pub const ALL: [Self; 12] = [
        Self::Release(0),
        Self::Release(1),
        Self::Gust(0),
        Self::Gust(1),
        Self::Flip,
        Self::Catch,
        Self::Miss,
        Self::Place,
        Self::Remove,
        Self::Solved,
        Self::Mark,
        Self::Ambience,
    ];

    /// Playback volume, `0..=1`. The files are normalised; these set the
    /// balance between them.
    #[must_use]
    pub const fn gain(self) -> f32 {
        match self {
            Self::Gust(_) => 0.3,
            Self::Release(_) => 0.35,
            Self::Remove => 0.4,
            Self::Flip | Self::Ambience => 0.45,
            Self::Place | Self::Mark => 0.5,
            Self::Miss | Self::Solved => 0.55,
            Self::Catch => 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sample;

    #[test]
    fn every_sample_is_in_the_bank_once_and_none_clips() {
        for (i, a) in Sample::ALL.iter().enumerate() {
            assert!((0.0..=1.0).contains(&a.gain()), "{a:?}");
            assert!(!Sample::ALL[i + 1..].contains(a), "{a:?} twice");
        }
    }
}
