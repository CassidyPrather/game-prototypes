# Space Trucking

An ambient game about hauling cargo across the solar system, built to be
played in the background: launch, let the ship fly while you do something
else, come back to barter.

Pick it from the [main menu](../README.md); `Escape` goes back there. The
ship keeps flying while you are at the menu, exactly as it does when the tab
is closed.

Design intent, lore, and the long-term (3D/VRChat) ambitions live in
[space-trucking/DESIGN.md](space-trucking/DESIGN.md); the recurring
stay-on-target checklist lives in
[space-trucking/DESIGN_REVIEW.md](space-trucking/DESIGN_REVIEW.md). This
file sticks to what the prototype does today and how to work on it.

Space Trucking began as its own repository,
[CassidyPrather/space-trucking](https://github.com/CassidyPrather/space-trucking),
and was moved here whole from its `main` at `e312fed` (the merge of its
pull request #2), with nothing dropped; that repository's history was not
carried over. Its `docs/` live in [space-trucking/](space-trucking/).

## Playing

The screen is the ship's console: a star map, a 6×4 cargo hold, and a barter
panel. The planets orbit the sun in real time — Venus through Neptune,
Saturn included, plus a Spacing Guild station running its orbit the wrong
way round — and a few stranger stops that only show themselves under the
right conditions. While docked, click a point of interest and pull the
launch lever: the course is charted to where the destination *will* be, and
the ship crawls there in real time. Journeys between the outer-ring worlds
run tens of minutes on purpose; this is a game meant to sit in a corner of
your day. The three inner-ring factions barely tolerate each
other: charting a direct course from one inner world to another takes
transit papers, which the Guild happens to broker. Approaching from the
outer ring is nobody's business but yours.

Then barter — no currency, cargo for cargo, read off an eagerness dial. The
dial only reads true for goods you have traded at that station before;
unfamiliar goods fog the needle, and finding out what a station really pays
means pulling the lever and living with the answer. Stations have patience,
and three wasted pulls ends the visit's trading — though no station in the
system refuses a gift. Cargo has opinions about stowage (heavy rides low,
volatiles refuse adjacency, cryo hugs the hull), and one matte-black kind of
crate hums, vanishes into a Guild hangar on delivery, and fills an unlabeled
lamp plate with whatever is being counted. The barter panel moonlights
when no trade is open: underway, its shelf row becomes the outboard rail —
drag cargo there to jettison it, recoverable until the next port call or
cast-off sweeps it away (the humming crate refuses to go) — and its dial
housing wears the badge of whatever pulls alongside mid-leg. Encounter
salvage drifts into the same rail, and at stranger berths the panel shows
stranger things.

| Input           | Effect                                                     |
| --------------- | ---------------------------------------------------------- |
| Mouse           | everything — select, pull levers, drag cargo               |
| `Shift`+click   | quick-move a piece to its obvious destination              |
| `Space`         | pause                                                      |
| `M`             | mute                                                       |
| `R`             | new run                                                    |
| `Escape`        | leave for the menu (the ship flies on)                     |

Accessibility: on the web the game honors your system's reduced-motion
preference (applied when the game loads) — decorative idle animation freezes
to a readable static pose, while everything caused by play still moves. No
signal relies on color alone; refusals, warnings, and states all carry a
shape, brightness, or position tell alongside their hue.

The game auto-saves — to localStorage on the web, to a `local.data` file
natively (via quad-storage) — and on load fast-forwards up to six hours of
elapsed real time, so the ship keeps flying while the tab is closed. A
backgrounded tab catches up the same way the moment it wakes, and so does a
visit to the menu: real time always passes. The save format is versioned
(`STV4`) with no compatibility promises before 1.0; an unreadable save
becomes a fresh run, quietly.

### Privacy

Telemetry is opt-in and off by default. The web page asks once, the first
time you enter Space Trucking and before you play it, whether the game may
keep anonymous play statistics — coarse counts and whole-second durations
only, no identity — stored in your own browser's localStorage and sent
nowhere. Decline, or never answer, and nothing is recorded; any previously
stored buffer is deleted the next time the game loads. Clearing site data
clears the choice and re-asks. Native builds never ask and never collect. The
other prototypes collect nothing. The full contract, including the exact
schema, lives in [space-trucking/TELEMETRY.md](space-trucking/TELEMETRY.md).

## Multiplayer

The deterministic core is multiplayer-ready: up to six players crew one ship
in input-only lockstep, and crews report hangar deliveries to a central
guild server whose counters cannot double-count. The architecture and its
required network-failure properties live in
[space-trucking/NETWORKING.md](space-trucking/NETWORKING.md);
`cargo run --example space_trucking_convoy` runs a six-client crew over a
deliberately hostile simulated network. The live multiplayer console is a
later slice; the protocol it will speak (`SNP2`) is already under test.

## Sound

Sound is synthesised at startup in `src/space_trucking/synth.rs` — no audio
assets — ambient only, no music. `M` mutes. The standing loops (engine,
warp engine, the humming crate, station air) fall silent when you leave for
the menu and fade back in when you return. On the web it needs macroquad's
`audio` feature and quad-snd's `audio.js` plugin in `web/`, and browsers
refuse to make noise before the first click.

## Developer mode (fast-forward)

The game runs at 1× for everyone; the 16× fast-forward is a development
tool, hidden until asked for nicely. Natively, run with `--dev`
(`cargo run -- --dev`). On the web, open the page with `#pretty-please` in
the URL and answer the shell's one question honestly (`#no-thank-you`
revokes). Developer mode reveals the warp button and the `F` key in Space
Trucking.

## Flight recorder

The game keeps a black box: a recent save plus every input frame since,
stored beside the autosave under the key `space-trucking/replay`
(localStorage on the web, quad-storage's `local.data` natively) and re-based
on a rolling cap so it always holds the recent past. The sim is
deterministic, so that small text file *is* the session: copy it out and
`cargo run -- --replay <file>` plays it back natively, bit-identically, with
the version string tinted amber as the only tell. `--replay` skips the menu
and opens the tape directly. A recording attached to a bug report is a
perfect reproduction.

## Working on it

Space Trucking follows the same split as every prototype here, and the
notes in [SKILL.md](../SKILL.md#space-trucking) say where things live:

- `src/space_trucking/` — the pure library: `sim` (the game), `net`
  (lockstep multiplayer), `replay`, `telemetry`, `synth`.
- `src/games/space_trucking.rs` and `src/games/space_trucking/` — the
  macroquad frontend: the loop, `render`, `audio`, `juice`, `tutor`,
  `palette`, `storage`, and the menu `emblem`.

Space Trucking's own commands:

- Performance budgets (CI-enforced ceilings, see
  [space-trucking/BUDGETS.md](space-trucking/BUDGETS.md)):
  `cargo test --release --test space_trucking_perf -- --ignored`
- Benchmark: `cargo bench --bench space_trucking_bench -- --quick`
- Six-client convoy over hostile links:
  `cargo run --example space_trucking_convoy`
