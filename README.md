# Game Prototypes

Cassidy's small game prototypes, behind one menu. Rust +
[macroquad](https://macroquad.rs/), compiled to wasm and deployed as a
static page, playable at <https://wirenook.net/game-prototypes/>. Native
desktop builds work too.

This is also where Cassidy's new games start: a small one moves in here as
another prototype, and a big one starts as its own repository from a copy of
this one. [docs/NEW_GAME.md](docs/NEW_GAME.md) covers both.

Every prototype here keeps the same split: the simulation is a pure,
deterministic library that never mentions macroquad, so the interesting
half runs headless in `cargo test`, and the frontend is a thin shell that
turns input into a frame struct and state into shapes and sound.

## The menu

The binary opens on a menu of prototypes, dressed in
[wirenook's style](https://wirenook.net/style/): a sky gradient, a wave field
of sparkles, tick rulers that follow the pointer, and one striped window per
prototype in a carousel. Each window is mostly the toy's own emblem, drawn
by the toy on a night-sky screen, because the prototypes are meant to read
without reading; the name is there so a shelf of them stays usable.

| Key | Action |
| --- | --- |
| Arrows, WASD, mouse wheel | Move along the carousel |
| Enter, or click the centre window | Start that prototype |
| Click a window peeking in at the side | Bring it to the centre |
| Escape | Leave a prototype for the menu (Amoeba first spends it closing its own panels) |
| The back arrow in a corner, tapped or clicked | Leave a prototype for the menu, on a screen with no Escape key |

Moving, hovering, hitting the end of the shelf, starting and leaving each
have a short synthesised sound and a bit of motion; starting closes an iris
on the chosen window and opens it again on the prototype.

The back arrow is drawn over every prototype, for phones. It sits in the
letterbox bar beside the frame when the window leaves room for one, which a
phone nearly always does, and otherwise on a corner the prototype keeps
clear.

Open the page with `?game=<slug>` (the slugs are in the table under
[Reflections](#reflections)), or run the binary with `--game <slug>`, and it
starts straight into that prototype, with the menu a back press away.
wirenook.net embeds each game in its reflection this way.

Leaving a prototype does not unload it, so coming back is a pause rather
than another wait on its sound bank. Starting a fresh run is the
prototype's own business; Leitmotif does it with `R`.

## The prototypes

- **[Leitmotif](docs/LEITMOTIF.md)** — a journey home whose rules are set by
  its music. Which motif is playing decides how the world behaves, and every
  voice in the band drives something that can hurt you.
- **[Sand Nomad](docs/SAND_NOMAD.md)** — a trading circuit of a dry ocean,
  where everything you grow attached to gathers weight and weighs on you.
  Visit every waystone and come home carrying as little as you can; barter
  on a scale with traders who never mention how heavy their goods are.
- **[Space Trucking](docs/SPACE_TRUCKING.md)** — an ambient game of hauling
  cargo across the solar system, meant to be played in the background:
  launch, go do something else, come back to barter cargo for cargo. The
  ship keeps flying while the tab is closed.
- **[Kitchen Garden](docs/KITCHEN_GARDEN.md)** — one day of cooking from a
  garden for whoever comes to the hatch: grow, gather, chop, boil, bake and
  serve, with never quite enough time, wood, coins or hands. It is the demo
  of **[the cooking module](docs/COOKING.md)**, a std-only, copy-pasteable
  set of foods, processes, recipes and a cooking state machine, meant to be
  lifted into other games.
- **[Amoeba Roguelike Remastered](docs/AMOEBA.md)** — play as a giant,
  constantly evolving amoeba: engulf the humans defending their cities, grow
  organelles from their remains, and break the gates to escape. A Rust
  remaster of a 2021 7DRL entry, and the one prototype that is all text.
- **[Small Hours](docs/SMALL_HOURS.md)** — keep one fire lit through one
  night. Shades walk to the nearest light and you cannot hurt them, only be
  nearer than the fire. When that is not enough, the night runs backward
  and you live it again beside the keeper you just were. Turn-based, eight
  short nights.

## Taste

Cassidy writes their opinions down at **[wirenook.net](https://wirenook.net/)**,
and that is the reference for what these games should be like. It covers
hundreds of reviews of games, board games, books and visual novels, essays
on design, and a style guide. When a question comes down to taste (what
would be fun, what would feel cheap, what it should look like), read there
before guessing, and say which post you leaned on.

- [wirenook.net/llms.txt](https://wirenook.net/llms.txt): the map, written
  for agents.
- [Reviews](https://wirenook.net/blog/tags/review/), mirrored from Steam,
  Goodreads and VNDB, plus board games.
- [The blog](https://wirenook.net/blog/), and every post's full text as JSON
  at [/blog/index.json](https://wirenook.net/blog/index.json). For game
  design, start with "Agency vs. Replayability" and "Rapid prototyping is
  cool".
- [The style guide](https://wirenook.net/style/), which the menu here is
  dressed in.
- The site's source,
  [CassidyPrather/cassidyprather.github.io](https://github.com/CassidyPrather/cassidyprather.github.io):
  every post is Markdown under `content/blog/`.

## Reflections

Every game here gets a **reflection**: a post on wirenook.net about what it
was trying to do and what came of it. The website is the source of truth.
This repository only links to reflections and never keeps copies. A game's
reflection lives at `https://wirenook.net/blog/reflections/<slug>/` and is
tagged so it also appears at
[/blog/tags/reflection/](https://wirenook.net/blog/tags/reflection/).

| Game | Slug | Reflection |
| --- | --- | --- |
| Leitmotif | `leitmotif` | [Leitmotif](https://wirenook.net/blog/reflections/leitmotif/) |
| Sand Nomad | `sand-nomad` | [Sand Nomad](https://wirenook.net/blog/reflections/sand-nomad/) |
| Space Trucking | `space-trucking` | [Space Trucking](https://wirenook.net/blog/reflections/space-trucking/) |
| Kitchen Garden | `kitchen-garden` | [Kitchen Garden](https://wirenook.net/blog/reflections/kitchen-garden/) |
| Amoeba Roguelike Remastered | `amoeba` | [AmoebaRL](https://wirenook.net/blog/reflections/amoeba/) |
| Small Hours | `small-hours` | not yet written |

The slug is also the game's `GameId::slug` in `src/shell.rs`, which is what
`?game=` reads. When a new one is published, link it here and at the top of
the game's doc.

## How it is built

- `src/lib.rs` — the library: everything pure and testable.
  - `src/shell.rs` — the menu's own state, which prototype it points at.
    - `src/shell/sfx.rs` — the menu's sounds, synthesised and tested.
  - `src/leitmotif/` — one prototype's simulation, score, synthesis and mix.
  - `src/sand_nomad/` — another's rules, world, pixel art and sound.
  - `src/space_trucking/` — another's sim, lockstep netcode, flight
    recorder, opt-in telemetry and synthesis.
  - `src/cooking/` — the reusable cooking module: foods, processes,
    recipes, stations, bills. Std only; copy it out whole.
  - `src/kitchen_garden/` — the cooking demo's day, prices and sounds.
  - `src/amoeba/` — Amoeba's whole game, turn-based, resolved down to a
    glyph per cell, and its synthesis.
  - `src/small_hours/` — Small Hours' rules, its eight nights, the
    exhaustive solver that proves each night's par, and its sounds.
- `src/main.rs` — the shell's loop: the menu, or whichever prototype is
  running.
  - `src/ui.rs` — the letterboxed 800x600 frame and the drawing primitives
    the menu and every prototype share.
  - `src/menu.rs` — the menu screen: input, motion and drawing.
  - `src/sounds.rs` — bakes the menu's sounds and plays them.
  - `src/games.rs` — the `Game` trait, and the two matches that wire a
    prototype in: how to load it, and how to draw its emblem.
  - `src/games/leitmotif.rs` — that prototype's macroquad frontend.
  - `src/games/sand_nomad.rs` — and Sand Nomad's, which paints pixel art
    into a low-resolution canvas and lays it over the frame.
  - `src/games/space_trucking.rs` — and Space Trucking's, which also keeps
    the save slot and the flight recorder.
  - `src/games/kitchen_garden.rs` — and Kitchen Garden's, drawn in soft
    vector shapes rather than pixels.
  - `src/games/amoeba.rs` — and Amoeba's, which draws CP437 glyphs across
    the whole window rather than the shared frame, and brings an on-screen
    pad for phones.
  - `src/games/small_hours.rs` — and Small Hours', which paints the ground
    by whose light is nearest.

Adding a prototype is a `GameId` variant, a module under each half, and an
arm in `games::load` and `games::emblem`. The library tests check the menu
can reach every entry. The whole checklist, and how to start a big game
somewhere else instead, is in [docs/NEW_GAME.md](docs/NEW_GAME.md).

## Development

Requires [Rust](https://rustup.rs/). Native builds on Linux also need ALSA's
development files (`libasound2-dev` on Debian/Ubuntu, `alsa-lib-devel` on
Fedora) — without them the link step fails with `unable to find library
-lasound`. The wasm build needs none of this; the browser handles audio.

Build: `cargo build`

Run (native): `cargo run`

Lint: `cargo clippy --all-targets --all-features -- -D warnings`

Lint (wasm): `cargo clippy --target wasm32-unknown-unknown -- -D warnings`

Format: `cargo fmt`

Test: `cargo test`

Audio headroom report: `cargo test --lib mix:: -- --nocapture` (and
`sfx::` for the menu's, Sand Nomad's and Kitchen Garden's sounds)

Space Trucking's native-only command-line modes: `cargo run -- --dev`
unlocks its fast-forward, and `cargo run -- --replay <file>` plays a
flight-recorder tape back (see [its doc](docs/SPACE_TRUCKING.md)).

Web build: `./scripts/build-web.sh` (needs
`rustup target add wasm32-unknown-unknown`; uses `wasm-opt` from
[binaryen](https://github.com/WebAssembly/binaryen) if installed)

Serve the result: `python3 -m http.server --directory dist/web 8080`

### Advanced

Benchmark: `cargo bench --bench sim_bench -- --quick` (and
`--bench space_trucking_bench`, `--bench amoeba_bench`)

Amoeba's whole-playthrough bots: `cargo test --release --lib amoeba -- --ignored`

Small Hours' workbench, each night's par and a way to keep it:
`cargo run --release --example small_hours_lab`

Space Trucking's performance budgets:
`cargo test --release --test space_trucking_perf -- --ignored`

Security audit: `cargo audit` (requires `cargo install cargo-audit`)

Pre-commit hook: `git config core.hooksPath .githooks` (runs `cargo fmt`)

Machine setup: `./scripts/cloud-setup.sh` installs all of the above, plus
Blender and GIMP for making assets. It's written to be a Claude Code cloud
environment's setup script; see [docs/CLOUD_SETUP.md](docs/CLOUD_SETUP.md).

## License

AGPL-3.0-or-later. Third-party files are listed in [CREDITS.md](CREDITS.md).
