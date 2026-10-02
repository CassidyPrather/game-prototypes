---
name: game-prototypes
description: Cassidy's small game prototypes behind one menu — Rust + macroquad, wasm, static deploy
---

# game-prototypes

A collection of small game prototypes behind a main menu. Rust + macroquad,
compiled to wasm, shipped as a static page. Native desktop builds also work.
Crate `game-prototypes`, lib `game_prototypes`, bin `game-prototypes`.
Built on Cassidy's game-template. Code is AGPL-3.0-or-later.

There are two prototypes. **Leitmotif**: a journey home whose rules are
set by its music — the current motif decides how the world behaves, each
voice in the band drives a hazard, and holding the music holds everything it
drives. **Sand Nomad**: a hand-made trading circuit of a dry ocean where
things gather *weight* as you grow attached to them; visit every waystone
and come home having carried as little as you can (a travelling salesman's
problem with a hold full of feelings), bartering on a scale with traders
who never mention the weight of what they hand over. Neither has any text,
only icons and key caps.

## Repo Map

The library half is pure and tested; the binary half is macroquad and is
not. Both are split the same way, project first and prototype second.

- `src/lib.rs` — the library root.
- `src/shell.rs` — the menu's state: `GameId`, and where the pointer is.
  Unit-tested; adding a prototype starts with a variant here.
- `src/shell/sfx.rs` — the menu's sounds (move, hover, bump, launch,
  back), synthesised in E major pentatonic with their gains, and tests that
  a player mashing the menu cannot clip.
- `src/main.rs` — the shell's loop. Menu, or the running prototype.
  Escape leaves a prototype without unloading it.
- `src/ui.rs` — the letterboxed 800x600 `Frame` and every drawing
  primitive, shared by the menu and the prototypes. Positions are pixels,
  sizes are frame units.
- `src/menu.rs` — the menu screen in wirenook's dress (wirenook.net/style):
  a carousel of striped windows, sparkles, tick rulers, an iris into and
  out of each prototype. Its `Screen` holds only presentation — springs and
  timers eased toward what `shell::Menu` says.
- `src/sounds.rs` — bakes `shell::sfx` and plays it once the player has
  pressed something (see the autoplay rule below).
- `src/games.rs` — the `Game` trait (`update`, `draw`) and the two matches
  that wire a prototype in: `load` and `emblem`.
- `src/ui.rs` also has `Frame::image`, for a prototype that paints into its
  own low-resolution texture and hands it to the frame whole.

### Leitmotif

- `src/leitmotif/song.rs` — the score and the sequencer. Motifs (key, tempo, drum
  patterns, bass and lead notes, pad chords), the song form (`SONG`), and
  `Sequencer`, which emits `Event`s as it is advanced by sim ticks. Pure,
  unit-tested, holds no clock of its own. Also lists every pitch and chord
  used, for the frontend to bake.
- `src/leitmotif/sim.rs` — the world. Pure, deterministic, no macroquad. Turns song
  events into behaviour (kicks move stompers, snares slam gates, hats turn
  spinners, bass raises walls, lead drops strikes, the pad charges the
  fermata; the motif sets the rules and the stompers' telegraphed moves)
  and forwards them, plus the game's own events, as `Cue`s. Unbounded:
  walls repeat on a grid, stompers respawn near the player. Most work
  belongs here or in `song`.
- `src/leitmotif/mix.rs` — what a cue sounds like. The cue-to-voice-and-gain policy
  the frontend plays through, and an offline `Mixer` whose tests render the
  whole song and fail if the sum nears full scale. Tune `MASTER` here.
- `src/leitmotif/synth.rs` — procedural instruments as WAV bytes. Pure, unit-tested;
  the toy ships no audio assets.
- `src/games/leitmotif.rs` — thin macroquad frontend. Camera, draw calls,
  the icon HUD, input, and the emblem the menu shows.
- `src/games/leitmotif/audio.rs` — the other half of the frontend: bakes one
  buffer per `mix::Voice` and plays `sim::Cue`s through `mix::voice_for`.
- `build.rs` — embeds a `git describe` version string.
- `web/index.html`, `web/gl.js`, `web/audio.js` — the static shell, the
  vendored miniquad loader, and the vendored quad-snd audio plugin. Zero
  external requests.
- `scripts/build-web.sh` — wasm build → `dist/web/`.
- `.github/workflows/ci-cd.yml` — lint, test, audit, size-budgeted web bundle,
  Pages deploy, release artifacts.
- `benches/` — criterion bench over the sim and the sequencer. Unit tests
  live beside the code in `src/`.

### Sand Nomad

Design, rules and knobs: `docs/SAND_NOMAD.md`.

- `src/sand_nomad/journey.rs` — the rules as a deterministic state machine.
  Commands (`arrange`, `offer`, `ask`, `deal`, `appraise`, `give`, `burn`,
  `renew`, `fill`, `set_sail`) and `advance` while sailing; `Cue`s out, one
  command's worth at a time. Time passes only in watches: sailing, and the
  few actions that cost it. Its `tests.rs` has a bot that walks the circuit;
  it is the balance check — keep it walking the whole circuit, beating the
  careless route, and landing on two marks of three.
- `src/sand_nomad/grid.rs` — the hold: packing, neighbours, the daily
  fondness rule (like feeds like, opposites quarrel), waking things wandering.
- `src/sand_nomad/barter.rs` — each culture's `Taste`; the scale; faces.
  Traders never count the weight of what *they* give (`give`), only of what
  they receive (`receive`). That asymmetry is what makes appraisal worth its
  watch; keep it.
- `src/sand_nomad/world.rs` — the hand-made basin: terrain rows, places,
  waystones, every rug. Nothing is rolled at runtime.
- `src/sand_nomad/history.rs` — the era's events, pure functions of the day.
- `src/sand_nomad/art.rs` — all pixel art as palette-character rows, 16 px
  grid; tested for shape and palette. `src/sand_nomad/sfx.rs` — every sound,
  synthesised, headroom-tested. No music, by request.
- `src/games/sand_nomad.rs` — pointer handling (click, drag, drop, turn) and
  cue reactions; `layout` holds every rectangle drawing and hit-testing
  share. `paint.rs` is a 400x300 render target (sample count 0: WebGL 1 has
  no multisample resolve) laid over the frame with `Frame::image`; textures
  are keyed by a hash of the sprite's rows, not its address, because a
  `const` may be copied wherever it is used. `draw.rs` draws everything.

## Commands

```bash
cargo build                                                   # build
cargo run                                                     # run natively
cargo clippy --all-targets --all-features -- -D warnings      # lint
cargo clippy --target wasm32-unknown-unknown -- -D warnings   # lint, wasm
cargo fmt                                                     # format
cargo test                                                    # test
cargo test --lib mix:: -- --nocapture                         # audio headroom report
cargo test --lib sand_nomad::journey -- --nocapture           # the bot's circuit, burden, deals
BOT_TRACE=1 cargo test --lib a_bot_can -- --nocapture         # ...and its hold at every stop
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
at a time, and never reads a clock; the music is as replayable as the world.
The fermata (holding Space) stops the sequencer and everything it drives
while the player keeps moving; it is gated by the sim's pool, which only
refills under the pad. Skipping a section lands on the next bar.

Do not read wall-clock time, macroquad state, or randomness from inside
`src/sim.rs` or `src/song.rs`. If the frontend needs to tell the sim
something, it goes in `InputFrame`.

The sim has two output channels, and sound uses the second one exactly the way
rendering uses the first: the getters (`player`, `stompers`, `walls`,
`strikes`, `walls`, `spin`, `gates`, `music`) for what to draw, `Sim::cues()` for what to play. A `Cue`
says what happened and how hard, never what it should sound like. Cues live
for one `advance()` and are cleared by the next.

The rule that makes the prototype honest: the sim and the audio consume the
*same* note events. If a layer is not arranged, or the music is held, it
emits nothing, so it neither sounds nor acts. Keep it that way — never let
the world react to a note the player cannot hear, or play one the world
ignores. Every hazard is telegraphed by something audible: the note that
drops a strike, the snare that slams the gates, the riser before a dash.

The mix is measured, not hoped for. Gains live in `src/mix.rs`, never in
`audio.rs`, so the tests that render the song offline see the same numbers
the speakers do. Add a voice or raise a gain and run the mix tests; if the
song's peak goes over the headroom, lower `MASTER` or the voice, do not
raise the headroom. Keep the game's own sounds few and in key: bells on the
motif's home note, or unpitched noise. Nothing that clashes with the band.

Difficulty is tested too. `holding_right_and_weaving_does_not_get_you_home`
is the floor and `a_careful_player_can_still_get_home` the ceiling; the
ignored `trace_the_careful_bot` prints where the bot goes and what it dies
of, for tuning.

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
`mix::all_voices` is the list, built from the song. Add a note to a score
and it bakes automatically; the tests check that nothing plays unbaked.

No text on screen. If the HUD needs to say something, say it with a shape
the world already uses, a colour, or a key cap with one character on it.
Sand Nomad keeps to this with runes, emotes, tally marks and a burden jar. The
menu bends this once, for the names on its cards, because a shelf of unnamed
pictures stops being a menu; everything else there is drawn.

Drawing goes through `ui::Frame`, never straight to macroquad, so every
prototype letterboxes the same way. A prototype that needs a camera wraps a
`Frame` rather than replacing it, as `games::leitmotif`'s `View` does.

Browsers keep the audio context suspended until a real gesture. `audio.js`
handles the resume, and the sim waits on the title screen for a first press
before the song starts, so nothing tries to sound before it can.

Two independent decoders read `synth`'s bytes — `audrey` natively and the
browser's `decodeAudioData` on the web — and the web one reports failure by
never calling back, which hangs macroquad's loader on a black screen. That is
why `synth.rs` has header tests.

See `docs/GETTING_STARTED.md` for framework and asset-source links.
