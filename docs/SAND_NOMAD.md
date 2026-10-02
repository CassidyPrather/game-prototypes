# Sand Nomad

A trading circuit of a dry ocean, where everything you grow attached to
weighs on you.

Pick it from the [main menu](../README.md); `Escape` goes back there.

## The basin

The Grand Basin is the bed of what was the ocean of the Azure Steppe. The
people who sail it on sand ships believe that anything someone grows
attached to gathers an ethereal *weight*, and that a thing with too much of
it wakes up — comes alive — which is a nuisance. So heavy things get
offloaded on anyone gullible enough to take them, most often people who do
not practise the belief, and the worst are burned to appease the ships.

Nomads differ in how they manage it. The Theseans rebuild their ships plank
by plank as the planks grow heavy, until nothing of the old ship is left and
it is still the same ship. The Pyre-Kin believe their ships are alive and
their friends, and feed them weight on great pyres. Some nomads are pirates.
The Crimson Harbor — an empire of united warlords chasing the glory of the
republic that collapsed before it, convinced that *order* is what made the
republic good — thinks they are all pirates, does not believe in weight at
all, and keeps only a periphery presence in the basin.

You are none of the grand history. You are a nomad walking the circuit.

## The game

Every place in the basin but home has a **waystone**, and every waystone
wants an offering bearing its **motive**. Give all ten, then sail home to
the Tether where you started. That is the whole goal; how well you did is
how little it all weighed on you.

**Burden** is the score, lower is better. Every watch (a quarter of a day)
everything in your hold weighs on you by its weight, and the jar on the deck
fills with those grains — you can watch them leave the heavy things and fall
in. A day without water weighs extra. Three notches on the jar are the marks:
come home under the top one for all three stones.

So the route matters: every day at sea is four more watches of everything
you carry. It is a travelling salesman's problem with a hold full of
feelings — and with pickups and deliveries, because the motives each stone
wants are sold in some places and not others, the Harbor's hate-marked
goods only come from the reef it is about to raid, and water runs out.

### Weight, motives and the rune

The six motives are the ones from the motif designs, on three opposing
axes: Bliss (the chalice) against Pain (the spike), Repose (the club)
against Zeal (the diamond), Love (the heart) against Hate (the spade).
Nomads paint them on what they own, so a thing's motive is always visible.

Its weight is not. A thing's weight shows as a **rune**: the motive
hexagon, tiny, in the corner of its cell. Weight lights the hexagon's nodes
one a point, starting at the thing's own motive and going round clockwise,
so a heavy thing has more of the six feelings in it. At six, every feeling
is in it and it **wakes**: the eye opens in the middle (the anima mark,
the ring with an eye). A woken thing weighs double, wanders about the hold
on its own every dawn, excites its neighbours, and no nomad will touch it.

Things grow heavier while you carry them. Every dawn each thing in the hold
gathers a point of fondness, plus one for every neighbour bearing the same
motive (like feeds like), minus one for every neighbour bearing the
opposite (they quarrel, and neither settles in), plus one per woken
neighbour; four fondness make a point of weight. Things without a motive —
water, the Harbor's orderly wares — never gather any. Where you pack things
matters: hover a thing in the hold to see its links (dotted for friends, a
spark for a quarrel) and how much fonder a day will make you.

A waystone only takes a thing that has been felt — weight one or more — and
never a woken one.

### Barter

There is no money, only the **scale**. Set things from your hold on your
pan and ask for things from the trader's rug onto theirs; the beam swings to
show how the trader reckons it, and when your side is down or level the
handshake lights. Each trader has a **culture**, and each culture prices
things its own way: materials it prizes, motives it reveres or shuns, a few
things it wants outright (shown in a thought bubble over the trader, and on
the map), and — the part that makes the basin the basin — how it feels
about weight.

The trader's face says why, with the motive emotes: love, pleasure,
indifference, zeal for a thing they want, hate for a motive they shun, pain
for a thing too heavy to bear, and horror (the anima emote) at a thing that
is alive.

The manners rule is what makes barter a game: **nobody speaks of the weight
of what they are handing over**. A trader prices their own goods as if they
weighed nothing, so a heavy locket comes to you looking like a bargain. The
only way to know is to **appraise** it — drag it to the loupe, or click the
loupe and then things — and sitting with a thing costs a watch, which your
whole hold weighs on you for. Appraise what matters; buy the rest sight
unseen and take your chances. (Your own things' weights are known once you
have read them.)

| Culture | Where | Reveres | Shuns | Weight | Wants |
| --- | --- | --- | --- | --- | --- |
| Drifters | Hearthring | Bliss, Love | Hate | Dread it | Rugs, salt |
| Keeper | Lighthouse | Repose, Love | Zeal | Dislike it | Lamps, lightning glass |
| Theseans | Drydock | Repose | Zeal | Dislike it | Planks, rope |
| Salt-scrapers | Salt flats | Bliss | Pain | Dread it | Water |
| Pirates | Spade Reef | Zeal, Hate | Love, Repose | Past caring; take woken things as trophies | Pistols, coin |
| Crimson Harbor | Fort; the reef after the raid | — | — | Do not believe in it; take anything, woken or not | Busts, maps |
| Hermit | Drowned Senate | Pain, Repose | Zeal | Dislike it | Maps, letters |
| Pyre-Kin | Ashfleet | Zeal, Pain | Repose | *Pay* for it; woken things most of all | Planks, oars |
| The leviathan's bones | Leviathan | — | — | Nothing | Nothing: take and leave freely |

Besides trading: the **wells** at home and the Well of Teeth fill every jar
for free; the **pyres** at home and the Ashfleet burn a thing (a watch
each); the Theseans' **workbench** rebuilds a thing until none of its
weight is left (two watches), though a weightless thing is no use to a
waystone. A thing with no weight can be left in the **sand pit** on the
deck; anything heavier will not be left behind.

### The places

| Place | Waystone | Trader | Also |
| --- | --- | --- | --- |
| The Tether (home) | — | — | Well, pyre |
| Lighthouse on the old shore | Bliss | Keeper | |
| Hearthring | Love | Drifters | |
| Drydock, in a seagoing wreck | Repose | Theseans | Workbench |
| Salt flats | Bliss | Salt-scrapers | |
| Spade Reef | Zeal | Pirates, then the Harbor | |
| The leviathan's bones | Repose | Bones | |
| Fort Ordinance | Hate | Crimson Harbor | |
| The Ashfleet | Zeal | Pyre-Kin | Pyre |
| The drowned senate | Pain | Hermit | |
| The Well of Teeth | Love | — | Well |

Everything is placed by hand in `src/sand_nomad/world.rs`: the map's
terrain rows, where each place stands, and every thing on every rug with its
motive and its hidden weight.

### History, going on without you

The era's grand events happen on a fixed calendar, and you see whatever your
route happens to show you: the Harbor's frigates sail from the fort to burn
out the pirates' reef (by day 14 the reef has changed hands, and a sergeant
sells off the pirates' things, heavier now); semaphore towers go up one by
one across the basin; a survey dirigible hangs over the drowned senate; the
fort lets off fireworks for the warlord-consul's name-day; a comet crosses
the sky. You hear the guns, the clacking towers and the engines from
wherever you are. None of it needs you.

### No text

There is no writing in the game. Weight is a rune; faces are emotes; days
are tally marks on the deck, with the sun climbing and setting over them
each watch; the circuit is a row of little standing stones, lit as they are
given; burden is the jar and its grains; the load you carry right now is the
column of pebbles beside it. The ending traces your whole route in gold and
raises a stone for each mark earned.

## Controls

| Input | Action |
| --- | --- |
| Drag a thing | Move it in the hold; onto a pan to trade; onto the waystone to give it; onto the pyre, workbench or loupe; into the sand pit to leave it |
| Right-click or `Space` while dragging | Turn a long thing |
| Click a thing in the hold / on the rug | Put it on your pan / ask for it |
| Click a thing on a pan | Take it back |
| Click the loupe, then things | Appraise them (a watch each) |
| Click the handshake | Strike the deal on the scale |
| Click the well | Fill every jar |
| Click the waystone | Light up what in the hold it would take |
| Click the sail (bottom right) | Back to the map |
| Click a place on the map | Sail there (hover first to see the course: a pip a day, red past your water) |
| Hold `Space`, or the mouse over the map, while sailing | Sail faster |
| `M` | Mute |
| `R` | Start over, once home |
| `Escape` | Back to the menu |

## How it is built

The pure half lives under `src/sand_nomad/` and the macroquad half under
`src/games/sand_nomad/`, the same split as Leitmotif.

- `src/sand_nomad/motive.rs` — the six motives and their opposites.
- `src/sand_nomad/item.rs` — kinds, materials, weight, fondness, waking.
- `src/sand_nomad/grid.rs` — packing a grid; the daily growth rule; waking
  things wandering.
- `src/sand_nomad/barter.rs` — cultures, their tastes, the scale, faces.
- `src/sand_nomad/world.rs` — the hand-made basin.
- `src/sand_nomad/history.rs` — the calendar of events, as pure functions of
  the day.
- `src/sand_nomad/journey.rs` — the rules as a deterministic state machine:
  commands in, `Cue`s out, time passing only when you sail, appraise, burn
  or rebuild. Its tests include a bot that walks the circuit, and checks
  that a good route beats a careless one and that the marks are earned.
- `src/sand_nomad/art.rs` — every sprite as rows of palette characters, on
  a 16-pixel grid (items a cell each, people and places 32). Pure data, so a
  test checks every sprite is rectangular and in the palette.
- `src/sand_nomad/sfx.rs` — the sound design, synthesised: materials
  knocking and clinking as you handle them, the brass pan, the beam's creak,
  a note per motive for faces, runes and waystones (one D major
  pentatonic), sand, water, fire, hammers, distant guns and fireworks, and a
  looping wind. No music. Tests check every sound decodes, starts and ends
  silent, and that a busy moment cannot clip.
- `src/games/sand_nomad.rs` — the pointer: pressing, carrying, dropping,
  clicking, and turning cues into sound and motion.
- `src/games/sand_nomad/paint.rs` — a 400x300 render target the pixel art
  is drawn into at one pixel per pixel, then laid over the shell's frame.
- `src/games/sand_nomad/draw.rs` — the map, camp, deck and overlays.
- `src/games/sand_nomad/audio.rs` — the sound bank and the wind.

### Knobs

The constants that set how it plays: `DAY_PX`, `SAIL_SPEED`,
`APPRAISE_WATCHES`, `BURN_WATCHES`, `RENEW_WATCHES`, `THIRST_BURDEN` and
`PAR` in `journey.rs`; `ANIMA_WEIGHT`, `FONDNESS_PER_WEIGHT`, `JAR_SIPS` and
`ANIMA_BURDEN` in `item.rs`; each culture's `Taste` in `barter.rs`; and every
rug in `world.rs`. `cargo test --lib sand_nomad::journey -- --nocapture`
prints the bot's days, burden and deals; `BOT_TRACE=1` prints its hold at
every stop.

## Ideas not yet tried

- Traders who remember you: dump a heavy thing on someone and they are
  cooler next time.
- Weather: a sandstorm season that makes some legs longer, so the best
  route changes with the calendar.
- The Pyre-Kin's belief made mechanical: a ship fed on weight sails faster.
- A second circuit with the waystones' motives shuffled by hand, for a
  replay that is not a repeat.
