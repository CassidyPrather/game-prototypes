//! The main menu's own state: which prototype it is pointing at.
//!
//! Pure and macroquad-free, like every simulation here, so the part of the
//! menu that can be wrong is unit-tested and the part that only draws is
//! not. The binary's `menu` module renders whatever this says.

/// One prototype in the collection.
///
/// Adding a toy is adding a variant, a name, and an arm in the binary's
/// `games::load` and `menu::emblem` — the tests below check the first two
/// stay in step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameId {
    /// A journey home whose rules are set by its music.
    Leitmotif,
}

impl GameId {
    /// Every prototype, in the order the menu lists them.
    pub const ALL: [Self; 1] = [Self::Leitmotif];

    /// Display name, the one piece of text the menu shows.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Leitmotif => "Leitmotif",
        }
    }
}

/// Where the menu is pointing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Menu {
    index: usize,
}

impl Menu {
    /// A menu pointing at the first prototype.
    #[must_use]
    pub const fn new() -> Self {
        Self { index: 0 }
    }

    /// How many prototypes there are to choose between.
    #[must_use]
    pub const fn len() -> usize {
        GameId::ALL.len()
    }

    /// Position of the pointer, for the renderer.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// The prototype under the pointer.
    #[must_use]
    pub const fn selected(self) -> GameId {
        GameId::ALL[self.index]
    }

    /// Move the pointer by `delta` entries, wrapping at both ends. With one
    /// prototype this does nothing, which is why the menu hides its arrows
    /// until there are two.
    // The list is a handful of entries, so nothing is lost turning its
    // length into a step and back.
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub const fn move_by(&mut self, delta: i32) {
        // `rem_euclid` rather than `%` so a negative step wraps to the end
        // of the list instead of landing on a negative index.
        let len = Self::len() as i32;
        self.index = (self.index as i32 + delta).rem_euclid(len) as usize;
    }

    /// Point at `index`, ignoring one that is past the end. The renderer
    /// calls this when the mouse is over an entry.
    pub const fn point_at(&mut self, index: usize) {
        if index < Self::len() {
            self.index = index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_menu_points_at_the_first_prototype() {
        let menu = Menu::new();
        assert_eq!(menu.index(), 0);
        assert_eq!(menu.selected(), GameId::ALL[0]);
        assert_eq!(menu, Menu::default());
    }

    #[test]
    fn moving_wraps_at_both_ends() {
        let mut menu = Menu::new();
        // Walk the whole list twice in each direction and stay in bounds.
        for step in 0..(Menu::len() * 2) {
            assert_eq!(menu.index(), step % Menu::len());
            menu.move_by(1);
        }
        assert_eq!(menu.index(), 0);
        menu.move_by(-1);
        assert_eq!(
            menu.index(),
            Menu::len() - 1,
            "moving up from the top wraps"
        );
        menu.move_by(1);
        assert_eq!(menu.index(), 0, "moving down from the bottom wraps");
    }

    #[test]
    fn a_big_jump_still_lands_on_an_entry() {
        let mut menu = Menu::new();
        for delta in [-97, -1, 0, 1, 3, 1000] {
            menu.move_by(delta);
            assert!(menu.index() < Menu::len(), "landed on {}", menu.index());
        }
    }

    #[test]
    fn pointing_past_the_end_is_ignored() {
        let mut menu = Menu::new();
        menu.point_at(Menu::len() - 1);
        assert_eq!(menu.index(), Menu::len() - 1);
        menu.point_at(Menu::len());
        menu.point_at(usize::MAX);
        assert_eq!(menu.index(), Menu::len() - 1, "an off-list hover moved it");
    }

    #[test]
    fn every_prototype_is_reachable_and_named() {
        let mut menu = Menu::new();
        let mut seen = Vec::new();
        for _ in 0..Menu::len() {
            let id = menu.selected();
            assert!(!id.name().is_empty(), "{id:?} has no name");
            seen.push(id);
            menu.move_by(1);
        }
        assert_eq!(seen, GameId::ALL, "the menu does not visit every entry");
    }
}
