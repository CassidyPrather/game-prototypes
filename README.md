# Leitmotif

A prototype for a game whose rules are set by its music. Which *motif* is
playing decides how the arena behaves; which *instruments* are sounding
decides what in it moves. Everything you hear is doing something, and
nothing you cannot hear is.

Rust + [macroquad](https://macroquad.rs/), compiled to wasm, deployed as a
static page. Built on Cassidy's [game-template](https://github.com/CassidyPrather/game-template).
Native desktop builds work too.

## The idea

The song is a loop of six four-bar sections. Each section is one of three
motifs — a theme with its own key, tempo and character — played by whichever
of the four instruments the arrangement gives it. The sequencer that walks
the score lives *inside* the deterministic simulation, and the same stream
of note events drives both the arena and the speakers.

**The motif sets the rules.**

| Motif | Feel | Stompers | Player | Sparks worth |
| --- | --- | --- | --- | --- |
| Wander | C major, 110 bpm | Drift a step on each kick, re-aim on each snare | Normal speed | ×1 |
| Pursuit | A minor, 132 bpm | Lunge at you on each kick, harder | A little faster | ×2 |
| Lullaby | F major, 84 bpm | Asleep and harmless, whatever the drums do | Slower | ×3 |

**Each instrument moves one thing, only while it sounds.**

| Instrument | What its notes do |
| --- | --- |
| Drums | Kicks step the stompers; snares turn them |
| Bass | Each note raises a wall in the lane of its pitch class, which sinks again over a beat and a half |
| Lead | Each note drops a spark, placed left-to-right by pitch and top-to-bottom by its position in the bar, so a bar of melody draws a shape |
| Pad | While it holds, nearby sparks are pulled toward you |

An instrument sounds when the arrangement includes it *and* you have not
muted it. The intro has no drums, so nothing moves until section two; the
second chase drops the pad, so nothing helps you gather; the lullaby keeps
the drums but gives them no kicks, so the stompers stay put.

**Collect sparks, avoid stompers.** A spark is worth the number of
instruments sounding, times the motif's multiplier. Muting layers with `1`
to `4` makes the arena safer and poorer: no bass means no walls to hide
behind but nothing to trip over; no drums means the stompers freeze but
every spark pays less. Mutes land on the next bar, so the music never
stumbles. Three hits and the music stops.

## Controls

| Key | Action |
| --- | --- |
| Arrows / WASD | Move |
| 1 2 3 4 | Mute or unmute drums, bass, lead, pad (takes effect at the next bar) |
| Tab | Cue the next section (at the next bar) |
| Space | Pause |
| R | Start over with a new seed |
| M | Mute the audio itself (the game keeps playing; the music keeps ruling) |

## How it is built

- `src/song.rs` — the score and the sequencer. Motifs, patterns, the song
  form, and a `Sequencer` that emits `Event`s as sim ticks go by. Pure and
  unit-tested; it holds no clock of its own.
- `src/sim.rs` — the arena. Turns song events into behaviour and, unchanged,
  into `Cue`s for the frontend. Fixed 60 Hz timestep, seeded, deterministic:
  same seed plus same inputs is the same game.
- `src/synth.rs` — the instruments, rendered to WAV bytes at startup. A
  Karplus-Strong pluck for the lead, a detuned pad, a sine bass, three drums,
  and the game's own bleeps. No audio assets ship.
- `src/audio.rs` — bakes one buffer per pitch each instrument uses (the
  song says which; macroquad cannot pitch a sound) and plays cues.
- `src/main.rs` — input, drawing, HUD.

Notes fire on the frame the sim crosses a step, so playback carries up to a
frame of jitter. That is the accepted cost of keeping the sequencer inside
the deterministic sim rather than on an audio thread; a real build would
schedule audio a little ahead of the sim.

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

Web build: `./scripts/build-web.sh` (needs
`rustup target add wasm32-unknown-unknown`; uses `wasm-opt` from
[binaryen](https://github.com/WebAssembly/binaryen) if installed)

Serve the result: `python3 -m http.server --directory dist/web 8080`

### Advanced

Benchmark: `cargo bench --bench sim_bench -- --quick`

Security audit: `cargo audit` (requires `cargo install cargo-audit`)

Pre-commit hook: `git config core.hooksPath .githooks` (runs `cargo fmt`)

## Ideas not yet tried

- Let the *player* be an instrument: moving on the beat adds a layer.
- Motifs that modulate: a section in a new key remaps the wall lanes.
- Sparks that carry their note and play it back when collected, so a good
  run harmonises with the song.
- Stompers keyed to specific drums (one to the kick, one to the snare).
- Scheduling audio a beat ahead so timing is sample-accurate.

## License

AGPL-3.0-or-later. Third-party files are listed in [CREDITS.md](CREDITS.md).
