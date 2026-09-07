---
name: leitmotif
description: Leitmotif, a Rust + macroquad prototype where the music's motif and instruments set the game's rules — wasm, static deploy
---

# leitmotif

A prototype game whose rules are set by its music: the current motif decides
how the arena behaves, and each sounding instrument animates one thing.
Rust + macroquad, compiled to wasm, shipped as a static page. Native desktop
builds also work. Crate `leitmotif`, lib `leitmotif`, bin `leitmotif`.
Built on Cassidy's game-template. Code is AGPL-3.0-or-later.

## Repo Map

- `src/song.rs` — the score and the sequencer. Motifs (key, tempo, drum
  patterns, bass and lead notes, pad chords), the song form (`SONG`), and
  `Sequencer`, which emits `Event`s as it is advanced by sim ticks. Pure,
  unit-tested, holds no clock of its own. Also lists every pitch and chord
  used, for the frontend to bake.
- `src/sim.rs` — the arena. Pure, deterministic, no macroquad. Turns song
  events into behaviour (kicks move stompers, bass raises walls, lead drops
  sparks, pad pulls sparks in; the motif sets the rules) and forwards them,
  plus the game's own events, as `Cue`s. Most work belongs here or in `song`.
- `src/synth.rs` — procedural instruments as WAV bytes. Pure, unit-tested;
  the toy ships no audio assets.
- `src/main.rs` — thin macroquad frontend. Window, draw calls, HUD, input.
- `src/audio.rs` — the other half of the frontend: bakes one buffer per
  pitch per instrument and turns `sim::Cue`s into playback. Binary-crate
  module, not part of the library.
- `build.rs` — embeds a `git describe` version string.
- `web/index.html`, `web/gl.js`, `web/audio.js` — the static shell, the
  vendored miniquad loader, and the vendored quad-snd audio plugin. Zero
  external requests.
- `scripts/build-web.sh` — wasm build → `dist/web/`.
- `.github/workflows/ci-cd.yml` — lint, test, audit, size-budgeted web bundle,
  Pages deploy, release artifacts.
- `benches/` — criterion bench over the sim and the sequencer. Unit tests
  live beside the code in `src/`.

## Commands

```bash
cargo build                                                   # build
cargo run                                                     # run natively
cargo clippy --all-targets --all-features -- -D warnings      # lint
cargo clippy --target wasm32-unknown-unknown -- -D warnings   # lint, wasm
cargo fmt                                                     # format
cargo test                                                    # test
./scripts/build-web.sh                                        # wasm -> dist/web/
python3 -m http.server --directory dist/web 8080              # serve it
cargo bench --bench sim_bench -- --quick                      # bench
cargo audit                                                   # audit
```

The web build needs `rustup target add wasm32-unknown-unknown`, and uses
`wasm-opt` from binaryen when it is on PATH.

## The Determinism Contract

The sim advances on a fixed 60 Hz timestep via an accumulator with a frame-dt
clamp, seeded through `fastrand`, and receives input only as an `InputFrame`
struct — so a given seed plus a given input sequence always produces the same
run, and rendering interpolates between the last two states using an alpha.

The sequencer is part of that contract. It is advanced by the sim, one tick
at a time, and never reads a clock; the music is as replayable as the arena.
Player changes to the music (mutes, skipping a section) are requests that
land on the next bar boundary, never immediately.

Do not read wall-clock time, macroquad state, or randomness from inside
`src/sim.rs` or `src/song.rs`. If the frontend needs to tell the sim
something, it goes in `InputFrame`.

The sim has two output channels, and sound uses the second one exactly the way
rendering uses the first: the getters (`player`, `stompers`, `walls`,
`sparks`, `music`) for what to draw, `Sim::cues()` for what to play. A `Cue`
says what happened and how hard, never what it should sound like. Cues live
for one `advance()` and are cleared by the next.

The rule that makes the prototype honest: the sim and the audio consume the
*same* note events. If a layer is muted or not arranged, it emits nothing,
so it neither sounds nor acts. Keep it that way — never let the arena react
to a note the player cannot hear, or play one the arena ignores (hats being
the one deliberate exception: audible, inert).

The wider rule: any module that imports macroquad is untestable — macroquad's
globals panic under `cargo test` (a thread assert), they do not fail politely.
Logic you want tested must live macroquad-free like `sim`, `song` and
`synth` do; frontend modules get verified by playouts or eyeballs.

## House Rules

Every asset gets a `CREDITS.md` line at intake — source, author, license, URL.
CC0 first.

CI enforces a hard wasm size budget (`MAX_WASM_BYTES`, ~1.5 MB by default);
if a change blows it, shrink the change or retune the budget deliberately.

Audio is on: macroquad's `audio` feature plus quad-snd's `audio.js` plugin in
`web/`, which must load after `gl.js` and before `load()` runs. Sounds are
synthesised in `src/synth.rs` rather than loaded, so there are no audio assets
and nothing to credit. macroquad gives you volume and looping and no pitch
control, so every pitch an instrument plays is its own baked buffer;
`song::pitches` and `song::chords` are the list. Add a note to a score and
it bakes automatically; the tests check that nothing plays unbaked.

Browsers keep the audio context suspended until a real gesture. `audio.js`
handles the resume, and the sim waits on the title screen for a first press
before the song starts, so nothing tries to sound before it can.

Two independent decoders read `synth`'s bytes — `audrey` natively and the
browser's `decodeAudioData` on the web — and the web one reports failure by
never calling back, which hangs macroquad's loader on a black screen. That is
why `synth.rs` has header tests.

See `docs/GETTING_STARTED.md` for framework and asset-source links.
