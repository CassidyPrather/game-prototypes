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

The binary opens on a menu of prototypes. Each card is mostly the toy's own
emblem, drawn by the toy, because the prototypes are meant to read without
reading; the name is there so a shelf of them stays usable.

| Key | Action |
| --- | --- |
| Up / Down, W / S | Move between prototypes |
| Enter, or click a card | Start the one under the pointer |
| Escape | Leave a prototype for the menu |

Leaving a prototype does not unload it, so coming back is a pause rather
than another wait on its sound bank. Starting a fresh run is the
prototype's own business; Leitmotif does it with `R`.

## The prototypes

- **[Leitmotif](docs/LEITMOTIF.md)** — a journey home whose rules are set by
  its music. Which motif is playing decides how the world behaves, and every
  voice in the band drives something that can hurt you.

## How it is built

- `src/lib.rs` — the library: everything pure and testable.
  - `src/shell.rs` — the menu's own state, which prototype it points at.
  - `src/leitmotif/` — one prototype's simulation, score, synthesis and mix.
- `src/main.rs` — the shell's loop: the menu, or whichever prototype is
  running.
  - `src/ui.rs` — the letterboxed 800x600 frame and the drawing primitives
    the menu and every prototype share.
  - `src/menu.rs` — the menu screen, reading input and drawing cards.
  - `src/games.rs` — the `Game` trait, and the two matches that wire a
    prototype in: how to load it, and how to draw its emblem.
  - `src/games/leitmotif.rs` — that prototype's macroquad frontend.

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

Audio headroom report: `cargo test --lib mix:: -- --nocapture`

Web build: `./scripts/build-web.sh` (needs
`rustup target add wasm32-unknown-unknown`; uses `wasm-opt` from
[binaryen](https://github.com/WebAssembly/binaryen) if installed)

Serve the result: `python3 -m http.server --directory dist/web 8080`

### Advanced

Benchmark: `cargo bench --bench sim_bench -- --quick`

Security audit: `cargo audit` (requires `cargo install cargo-audit`)

Pre-commit hook: `git config core.hooksPath .githooks` (runs `cargo fmt`)

## License

AGPL-3.0-or-later. Third-party files are listed in [CREDITS.md](CREDITS.md).
