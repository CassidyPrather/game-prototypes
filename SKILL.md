---
name: game-prototypes
description: Cassidy's small game prototypes behind one menu — Rust + macroquad, wasm, static deploy
---

# game-prototypes

A collection of small game prototypes behind a main menu. Rust + macroquad,
compiled to wasm, shipped as a static page. Native desktop builds also work.
Crate `game-prototypes`, lib `game_prototypes`, bin `game-prototypes`.
Built on Cassidy's game-template. Code is AGPL-3.0-or-later.

There are four prototypes. **Leitmotif**: a journey home whose rules are
set by its music — the current motif decides how the world behaves, each
voice in the band drives a hazard, and holding the music holds everything it
drives. **Sand Nomad**: a hand-made trading circuit of a dry ocean where
things gather *weight* as you grow attached to them; visit every waystone
and come home having carried as little as you can (a travelling salesman's
problem with a hold full of feelings), bartering on a scale with traders
who never mention the weight of what they hand over. **Space Trucking**: an
ambient, background-playable game of hauling cargo across the solar system and
bartering it cargo-for-cargo; it keeps its own save and the ship flies on in
real time while you are away. **Kitchen Garden**: one day of cooking from a garden
for whoever comes to the hatch — the demo of **the cooking module**
(`src/cooking/`), a std-only, copy-pasteable set of foods, processes (physical,
heat, time), recipes and a station state machine, meant to be lifted whole
into other games. None has any text, only icons and key caps (the
menu names its cards, and Space Trucking prints a version string in a corner).

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
- `src/games.rs` — the `Game` trait (`update`, `draw`, and `leave`, called
  when the player escapes to the menu; Sand Nomad's wind and Space
  Trucking's ambient loops stop there, since nothing else would) and the two matches that wire a prototype in: `load` and `emblem`.
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

### Space Trucking

Rules, privacy, multiplayer and the flight recorder: `docs/SPACE_TRUCKING.md`.
Design intent, lore and the stay-on-target checklist (`DESIGN_REVIEW.md`,
which runs at the end of every work stage there): `docs/space-trucking/`.
It arrived from its own repository whole, and keeps the conventions it grew
there, so this section is the map of where each landed.

- `src/space_trucking/sim.rs` and `src/space_trucking/sim/` — the simulation.
  Pure, deterministic, no macroquad. Most work belongs here.
  - `sim.rs` — `Sim`, `InputFrame`, `Cue`, `advance`, `fast_forward`.
  - `layout.rs` — shared console geometry, used for both hit-tests and
    rendering so they cannot disagree.
  - `map.rs` — points of interest, their orbits, and intercept travel.
    Positions are pure functions of the tick; nothing is stored.
  - `cargo.rs` — cargo kinds, pieces, and placement rules.
  - `barter.rs` — valuation and trade resolution.
  - `event.rs` / `rats.rs` / `encounter.rs` — the events, as siblings
    with a uniform hook shape (on_depart/on_dock/travel_tick/on_press +
    own save lines and cues); a new event should copy the shape, not
    invent a framework. `encounter.rs` holds both the travel encounters
    (derelict/gas station/casino/meteors/whale) and the ad drone.
  - `save.rs` — `STV4` serialization.
- `src/space_trucking/net.rs` and `net/` — deterministic lockstep
  multiplayer per `docs/space-trucking/NETWORKING.md`: protocol messages,
  helm/client session state machines, the guild server (idempotent
  max-merge delivery counters), and the seeded flaky-network harness the
  tests run on. Pure and macroquad-free like `sim`; transports are a later
  adapter. `examples/space_trucking_convoy.rs` runs six clients in one
  command.
- `src/space_trucking/replay.rs` — the flight recorder's tape format.
  `telemetry.rs` — the opt-in play-statistics aggregator
  (`docs/space-trucking/TELEMETRY.md`). `synth.rs` — procedural sound
  effects as WAV bytes, unit-tested; it ships no audio assets.
- `src/games/space_trucking.rs` — the macroquad loop: gathers an
  `InputFrame`, advances the sim, owns the save slot, the black box, the
  telemetry buffer and the one piece of text (the version string). Also
  `replay_session`, which `main.rs` runs for `--replay <file>`.
  `View` is the sim's world on the shell's `Frame`; the world is the same
  800x600 box, and a test says so.
  - `render.rs` (the console, into a crunched low-res target), `audio.rs`
    (cues to playback, plus the four ambient loops), `juice.rs`
    (cosmetic motion), `tutor.rs` (the idle onboarding ghost),
    `palette.rs` (every colour, by role), `storage.rs` (quad-storage),
    `emblem.rs` (the menu's mark).
- Leaving for the menu stops updates, not time: `stall_catch_up` replays the
  wall-clock gap on return, and `Game::leave` silences the loops and saves.
- `web/sapp_jsutils.js`, `web/quad-storage.js` — vendored plugins for the
  save slot (load order is load-bearing; `index.html` documents it). Its
  page also owns the reduced-motion and deep-night mirrors, the
  `#pretty-please` developer ceremony, and the telemetry consent card, which
  waits for the game to raise `space-trucking/consent-wanted` (the menu is
  not play) before it asks.
- `tests/space_trucking_perf.rs` — CI-enforced release-mode ceilings
  (`docs/space-trucking/BUDGETS.md`); `benches/space_trucking_bench.rs`.

### The cooking module and Kitchen Garden

The module's guide (copying it out, the station's states, bills, how to
extend): `docs/COOKING.md`. The demo's rules and knobs:
`docs/KITCHEN_GARDEN.md`.

- `src/cooking.rs`, `src/cooking/` — **the reusable part.** `food` (one flat
  enum, stages and groups), `process` (three kinds: physical is worked,
  heat is ticked and burns, time is ticked and waits), `recipe` (the
  `RECIPES` table and multiset lookups), `station` (the state machine:
  Idle → Working → Ready, events drained), `stock`, `bill` (cost from the
  ground up). It must stay std-only and refer to itself only through
  `super::` — `the_module_can_be_copied_out_whole` enforces it. Its tests
  keep the recipe table well-formed; run them after any recipe change.
- `src/kitchen_garden/day.rs` — the demo's rules as a deterministic state
  machine around seven `Station`s: places, the garden, hen, cow, well,
  woodpile and hearth fuel, pantry, market, compost, customers. Food moves
  only through `move_food`, which checks both ends first. Its `tests.rs` has
  the balance bot (first mark, not the third) and a monkey test.
- `src/kitchen_garden/prices.rs` — prices and patience derived from
  `cooking::bill`; keep them derived, never hand-set per dish.
- `src/kitchen_garden/sfx.rs` — sounds in one G major pentatonic, two loops,
  headroom-tested like Sand Nomad's.
- `src/games/kitchen_garden.rs` — pointer (drag, hold-to-work, click) and
  cue reactions; `layout` is the one source of rectangles. `pen.rs` is soft
  vector shapes in frame units plus the palette, `icons.rs` draws every
  `Food`/`Process`/`Kind`, `draw.rs` the scene, `fx.rs` cosmetic particles,
  `audio.rs` the bank and loops.

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
cargo test --lib cooking                                      # the reusable cooking module
cargo test --lib kitchen_garden::day -- --nocapture           # Kitchen Garden's bot, coins per seed
BOT_TRACE=1 cargo test --lib a_bot_can -- --nocapture         # ...and its hold at every stop
./scripts/build-web.sh                                        # wasm -> dist/web/
python3 -m http.server --directory dist/web 8080              # serve it
cargo bench --bench sim_bench -- --quick                      # bench
cargo bench --bench space_trucking_bench -- --quick           # ...and Space Trucking's
cargo test --release --test space_trucking_perf -- --ignored  # Space Trucking's perf budgets
cargo run --example space_trucking_convoy                      # six-client lockstep convoy
cargo run -- --replay <tape>                                  # play a Space Trucking black box
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

Space Trucking keeps the same contract with its own `InputFrame`: pointer
edges plus the toggles, splitmix RNG streams derived from the seed, and
`Cue`s that say what happened and how hard in `0..=1`, never what it should
sound like. `fast_forward` (warp and offline catch-up) suppresses cues. Do
not read wall-clock time, macroquad state or randomness from inside
`src/space_trucking/`; the frontend's `fresh_seed` and the `night` bit are
how the outside world gets in.

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

Space Trucking brought house rules of its own, which stand:

- Cargo is conserved: a piece the player owns never vanishes or changes
  hands except through four ceremonies — the accept lever, the Guild's hangar
  steal on docking (`Cue::Delivered`), ???'s three-for-one exchange
  (`Cue::Exchange`), and the outboard net's sweep (`Cue::Jettison`). The
  casino only ever transmutes a wagered piece, never destroys it. No drag can
  destroy anything. The ownership rule lives in exactly one place
  (`cargo::player_owned`), the drop matrix consumes it in `Sim::resolve_drop`,
  and the renderer's affordances come from `Sim::drop_targets()` — never
  restate any of them. The drag-monkey tests in `src/space_trucking/sim.rs`
  feed thousands of arbitrary input frames (solo and six-player) and fail the
  moment any interaction loses a piece outside those doors.
- Aesthetics are directed: `docs/space-trucking/ART_DIRECTION.md` holds the
  conceit (a worn instrument panel; screens vs metal), and all of its colour
  lives in `src/games/space_trucking/palette.rs` — a purity test fails the
  build on any raw colour constructor in its other frontend files. Follow the
  file or amend it in the same change.
- Its save string is versioned (magic `STV4`), hand-rolled in
  `src/space_trucking/sim/save.rs`, with no compatibility guarantees before
  1.0. Bump the magic on any breaking change; an old or corrupt save fails
  safe into a fresh game, never a panic.
- Telemetry is opt-in and local, and the consent card's wording in
  `web/index.html` must stay aligned with `docs/space-trucking/TELEMETRY.md`.
  Nothing identifying, nothing transmitted. Native builds never collect.
- Its ambient loops wait for the first press and start at zero volume, so
  nothing arrives mid-note when the browser wakes the audio context.
- The shared `web/index.html` carries Space Trucking's shell duties (the
  mirrors, the ceremony, the consent card); the game is the only reader of
  the `space-trucking/*` storage keys, and the other prototypes touch no
  storage.

See `docs/GETTING_STARTED.md` for framework and asset-source links.
