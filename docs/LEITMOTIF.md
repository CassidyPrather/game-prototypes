# Leitmotif

A prototype for a game whose rules are set by its music. Which *motif* is
playing decides how the world behaves; which *instruments* are sounding
decides what in it moves. Everything you hear is doing something, and
nothing you cannot hear is.

Pick it from the [main menu](../README.md); `Escape` goes back there.

## The game

Home is far to the east. Get there.

The song is a loop of six four-bar sections. Each section is one of three
motifs — a theme with its own key, tempo and character — played by whichever
of the four instruments the arrangement gives it. The sequencer that walks
the score lives *inside* the deterministic simulation, and the same stream
of note events drives both the world and the speakers.

**The motif sets the rules.**

| Motif | Feel | Stompers | Walls | You |
| --- | --- | --- | --- | --- |
| Wander | C major, 110 bpm | Drift a step on each kick, turn on each snare (half the time toward you), and on alternate bars swell for a beat and stomp a shockwave | Sink in a beat and a half | Normal speed |
| Pursuit | A minor, 132 bpm | Lunge at you on every kick, and on alternate bars wind up for a beat showing the line they will take, then dash down it | Linger for two and a half beats | A little faster |
| Lullaby | F major, 84 bpm | Asleep where they stand, still solid, still sore to touch | Stay up for most of a bar | Slower, and a lost heart comes back |

**Every voice drives something, only while it sounds.**

| Voice | What its notes do |
| --- | --- |
| Kick | Steps the stompers |
| Snare | Slams the gates shut across every standing wall's gaps for a beat. Being in a gap when they slam is a hit |
| Hat | Turns the spinners that stand between some of the walls. Touching a bar is a hit |
| Bass | Each note raises every wall in the lane of its pitch class. Walls stand across the way home with a gap every so often, and every other wall's gaps sit half a period off, so the road zigzags and the bass line decides when it is open. A wall rising into you is a hit |
| Lead | Each note drops a strike ahead of you, further ahead the later in the bar, further to the side the further from the middle of the tune, so the melody rains down in its own shape. A strike closes over a beat, then bursts |
| Pad | Charges the fermata, below |

**Hold the music.** Hold `Space` and the music holds — and with it
everything the music drives: stompers freeze mid-dash, strikes hang, gates
and spinners stop. You can still move. It spends a pool that refills only
while the pad is sounding, so the sections without a pad are the ones to
save it for. Run it dry and the music comes back on its own; the pool has
to refill part way before it will take again.

**Three hearts.** Stompers, strikes, shockwaves, spinners, slamming gates
and rising walls all cost one. The Lullaby gives one back. Lose them all and
the music stops; reach home and it resolves on the motif's home chord.

It is meant to be hard. Two tests keep it honest: a player who holds east
and weaves loses on every seed tried, and a bot that reads the road — next
gap, cross after the snare, off the aim lines, clear of the shockwaves,
around the spinners, holding the music when crowded — gets well past where
weaving ever does. The bot is no champion and does not get home; whether a
person can is the next thing to find out by playing, and the knobs are the
constants at the top of `src/leitmotif/sim.rs`.

There is no text in the game. The HUD is built from the same shapes the
world is made of — a stomper stands for the drums, a wall for the bass, a
strike for the lead, a fermata sign for the pad — plus key caps. The motif
shows as a colour and a glyph: a wave, an eye, a crescent. While the music
is held, everything it drives turns cool blue.

## Controls

| Key | Action |
| --- | --- |
| Arrows / WASD | Move |
| Space (hold) | Hold the music, spending the pool |
| P | Pause |
| Escape | Back to the menu |
| R | Start over with a new seed |
| M | Mute the audio itself (the game keeps playing; the music keeps ruling) |
| Tab | Cue the next section at the next bar (a developer shortcut, not shown in the HUD) |

## How it is built

All of it lives under `src/leitmotif/` (the pure half, a library module)
and `src/games/leitmotif.rs` (the macroquad half, a `games::Game`).

- `src/leitmotif/song.rs` — the score and the sequencer. Motifs, patterns, the song
  form, and a `Sequencer` that emits `Event`s as sim ticks go by. Pure and
  unit-tested; it holds no clock of its own.
- `src/leitmotif/sim.rs` — the world. Turns song events into behaviour and, unchanged,
  into `Cue`s for the frontend. Unbounded: walls repeat on a grid, stompers
  respawn around you as you travel. Fixed 60 Hz timestep, seeded,
  deterministic: same seed plus same inputs is the same game.
- `src/leitmotif/mix.rs` — what a cue sounds like: the policy mapping every cue to a
  voice and a gain, plus an offline mixer. See below.
- `src/leitmotif/synth.rs` — the instruments, rendered to WAV bytes at startup. A
  Karplus-Strong pluck for the lead, a detuned pad, a sine bass, three
  drums, and for the game's own sounds only a bell (always in the current
  key), a thud, a breath and a riser. No audio assets ship.
- `src/games/leitmotif/audio.rs` — bakes one buffer per voice (macroquad cannot pitch a
  sound) and plays cues through the policy in `mix`.
- `src/games/leitmotif.rs` — input, camera, drawing, the icon HUD, and the
  emblem the menu shows.

### The mix is measured

The audio backend sums every playing voice and hands the total to the
device with nothing in between to limit it, so a busy moment can clip. The
cue-to-voice policy therefore lives in the library, and `cargo test` renders
the whole song through it — every layer on, a hit, a shockwave and a windup
forced onto every downbeat — and fails if the summed peak nears full scale. A second
test piles every voice that can coincide onto one tick. CI runs these on
their own too, so the measured peaks land in the log. When the tests were
first written they caught the mix clipping at 1.10; the master gain is
tuned from them.

Notes fire on the frame the sim crosses a step, so playback carries up to a
frame of jitter. The headroom the tests demand covers that.

## Ideas not yet tried

- Let the *player* be an instrument: moving on the beat adds a layer.
- Motifs that modulate: a section in a new key remaps the wall lanes.
- Stompers keyed to specific drums (one to the kick, one to the snare).
- Terrain that changes with the section, so the road home has movements.
- Scheduling audio a beat ahead so timing is sample-accurate.
