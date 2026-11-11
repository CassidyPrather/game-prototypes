# Starting a new game

This page replaces Cassidy's old `game-template` repository. There is no
template anymore. A new game either moves in here as one more prototype, or
starts as its own repository from a copy of this one, which it then guts.
Either way there is one copy of the shared code to keep up to date, and it is
this one.

## Small or big?

**Small** games live here. Most ideas start small, and it's the default:

- it fits the shared shape: macroquad, the menu, the wasm bundle, and a
  sim that is a pure library;
- it doesn't need a URL, release cadence or collaborators of its own;
- its dependencies don't make the shared bundle much heavier (CI's
  `MAX_WASM_BYTES` will tell you).

**Big** games get their own repository when any of that stops being true.
For example, Extraction is a Bevy + OpenXR VR game, and nothing about it fits
a carousel in a browser tab. A small game can grow into a big one, and it
doesn't have to leave when it does. Space Trucking has both: a prototype
here, and its own repository where the bigger game carries on. The two copies
don't track each other.

Either way, give the game a **reflection**: a blog post on wirenook.net about
what it was trying to do and what came of it. See
[the README](../README.md#reflections) for where those live.

## A small game: add a prototype

The tests check that the menu can reach every entry, and the compiler catches
any match arm you miss.

1. **A `GameId` variant**, its name and its slug in `src/shell.rs`,
   appended to `GameId::ALL` in the order the menu should show it. The slug
   is the game's name in URLs (`?game=<slug>`, and its reflection's address
   on wirenook.net), so it never changes once published.
2. **The library half**: `src/<game>.rs` and `src/<game>/`, declared in
   `src/lib.rs`. Pure and deterministic, no macroquad. The shape every
   prototype here keeps: a seed and an input struct go in, the sim advances
   in fixed ticks (or one command per advance, if the game is turn-based),
   and drawable state plus `Cue`s come out. A `Cue` says what happened and
   how hard, never what it should sound like. Sounds go in a `synth.rs` or
   `sfx.rs` next to the sim, with the WAV header tests copied from one of
   the others.
3. **The frontend half**: `src/games/<game>.rs` (and `src/games/<game>/`),
   declared in `src/games.rs`. It needs a `Game` with `load`, `update` and
   `draw`, plus `leave` if anything loops, `wants_escape` if the game
   uses Escape itself, and `back_spot` if its HUD holds the top-left corner
   (the shell's back button sits there when the window has no letterbox bar
   to put it in). It also needs a free function `emblem` that draws the
   game's card in the menu without loading anything.
4. **Two match arms** in `src/games.rs`: one in `load`, one in `emblem`.
5. **Docs**: `docs/<GAME>.md` (what it is, its rules and knobs, its
   commands), an entry in the README's list, a section in `SKILL.md`, and a
   `CREDITS.md` row for every asset at intake.
6. **Checks**, the same as CI runs:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo clippy --target wasm32-unknown-unknown -- -D warnings
   cargo test
   ./scripts/build-web.sh
   ```

If a game needs to break one of the house rules in `SKILL.md`, it can; Space
Trucking and Amoeba both do. Write down which rule and why in its `SKILL.md`
section.

## A big game: copy this repo, then gut it

GitHub won't let you fork your own repository into the same account, and the
new game doesn't want this one's history anyway. Start from a plain copy:

```bash
git clone --depth 1 https://github.com/CassidyPrather/game-prototypes my-game
cd my-game
rm -rf .git && git init -b main
```

Then create the empty repository on GitHub and push to it. After that, treat
everything you copied as reference. The goal is to delete it quickly, not to
maintain it.

### Rename

1. `Cargo.toml`: `name` (kebab-case), the `[lib]` name (snake_case), the
   `[[bin]]` name (kebab-case), `description`, `repository`, `keywords`,
   and whatever dependencies the new game doesn't need.
2. Everywhere the binary name is hard-coded:
   - `web/index.html`: the `.wasm` filename passed to `load()`
   - `scripts/build-web.sh`: `BIN=`
   - `.github/workflows/ci-cd.yml`: the wasm filename in the size-budget
     step, the web zip's name, and `BIN=` in the package step
   - `.vscode/launch.json`: the `--bin`/`--package` arguments and names
   - `scripts/cloud-setup.sh` and `docs/CLOUD_SETUP.md`: the raw URL
3. `MAX_WASM_BYTES` in the CI workflow. It's sized for this whole
   collection, so set it for the new game.

### Gut

Delete everything that belongs to the other games:

- every prototype's library and frontend: `src/leitmotif*`,
  `src/sand_nomad*`, `src/space_trucking*`, `src/cooking*` (copy the cooking
  module out first if you want it; see [COOKING.md](COOKING.md)),
  `src/kitchen_garden*`, `src/amoeba*`, and their counterparts under
  `src/games/`;
- `benches/`, `tests/` and `examples/` (keep one bench, rewritten against
  your own sim's hot path);
- `docs/`, apart from `GETTING_STARTED.md`, `DEPLOYING.md`,
  `CLOUD_SETUP.md` and whatever this page is still useful for;
- every `CREDITS.md` row for an asset that is gone;
- Space Trucking's duties in `web/index.html` (the reduced-motion and
  deep-night mirrors, the `#pretty-please` ceremony, the telemetry consent
  card) and its plugins `web/quad-storage.js` and `web/sapp_jsutils.js`.

The menu goes too, unless the new game wants a menu: `src/shell.rs`,
`src/shell/`, `src/menu.rs`, `src/sounds.rs` and `src/games.rs`. Then
`src/main.rs` becomes the game's own loop. Amoeba's `src/games/amoeba.rs`
is a good model, since it was a standalone loop until recently.

Keep these. They're not tied to any game:

- `src/ui.rs`'s letterboxed `Frame`, if a fixed design resolution suits the
  game;
- `build.rs` (the `git describe` version string);
- `web/gl.js` and `web/audio.js`, and the order rules `web/index.html`
  documents for them;
- `scripts/build-web.sh`, `scripts/cloud-setup.sh`, the CI workflow, the
  lint table in `Cargo.toml`, `.githooks/`, `LICENSE`.

If the game isn't macroquad at all (Extraction is Bevy), keep even less: the
CI shape, the lints, the house rules that still apply, and the doc
conventions. Read the prototypes here as reference rather than copying them.

### Rewrite the docs

Replace the README and `SKILL.md` with the new game's own. Copy the README's
**Taste** section across as it is: agents working on the new game need
wirenook.net just as much as agents here do.

## Machine setup

`scripts/cloud-setup.sh` sets up a fresh machine for any of this: Rust with
the wasm target, ALSA headers, `wasm-opt`, Blender and GIMP. It's written to
be a Claude Code cloud environment's setup script. See
[CLOUD_SETUP.md](CLOUD_SETUP.md).
