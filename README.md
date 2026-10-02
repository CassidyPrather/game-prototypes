# Game Prototypes

Cassidy's small game prototypes, behind one menu. Rust +
[macroquad](https://macroquad.rs/), compiled to wasm and deployed as a
static page. Native desktop builds work too. Built on Cassidy's
[game-template](https://github.com/CassidyPrather/game-template).

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
| Escape | Leave a prototype for the menu |

Moving, hovering, hitting the end of the shelf, starting and leaving each
have a short synthesised sound and a bit of motion; starting closes an iris
on the chosen window and opens it again on the prototype.

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

## How it is built

- `src/lib.rs` — the library: everything pure and testable.
  - `src/shell.rs` — the menu's own state, which prototype it points at.
    - `src/shell/sfx.rs` — the menu's sounds, synthesised and tested.
  - `src/leitmotif/` — one prototype's simulation, score, synthesis and mix.
  - `src/sand_nomad/` — another's rules, world, pixel art and sound.
  - `src/space_trucking/` — another's sim, lockstep netcode, flight
    recorder, opt-in telemetry and synthesis.
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

Adding a prototype is a `GameId` variant, a module under each half, and an
arm in `games::load` and `games::emblem`. The library tests check the menu
can reach every entry.

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
`sfx::` for the menu's and Sand Nomad's sounds)

Space Trucking's native-only command-line modes: `cargo run -- --dev`
unlocks its fast-forward, and `cargo run -- --replay <file>` plays a
flight-recorder tape back (see [its doc](docs/SPACE_TRUCKING.md)).

Web build: `./scripts/build-web.sh` (needs
`rustup target add wasm32-unknown-unknown`; uses `wasm-opt` from
[binaryen](https://github.com/WebAssembly/binaryen) if installed)

Serve the result: `python3 -m http.server --directory dist/web 8080`

### Advanced

Benchmark: `cargo bench --bench sim_bench -- --quick` (and
`--bench space_trucking_bench`)

Space Trucking's performance budgets:
`cargo test --release --test space_trucking_perf -- --ignored`

Security audit: `cargo audit` (requires `cargo install cargo-audit`)

Pre-commit hook: `git config core.hooksPath .githooks` (runs `cargo fmt`)

## License

AGPL-3.0-or-later. Third-party files are listed in [CREDITS.md](CREDITS.md).
