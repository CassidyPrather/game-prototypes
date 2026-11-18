# Windfall

A tree's leaves, blown home by a few vanes.

Pick it from the [main menu](../README.md), or open the page with
`?game=windfall`; `Escape` goes back to the menu.

A garden is a small grid. Above it a tree holds a few coloured leaves, in
the order it will drop them. Below it stand baskets, one colour each. Place
vanes in the garden, press play, and watch the leaves fall. Every garden is
won when every leaf lands in a basket of its own colour.

Eleven gardens. All of them can be won with pieces to spare; the marks are
for winning well.

## The rules

- A leaf falls one cell a tick.
- A **vane** in a leaf's cell blows it one cell sideways instead, the way
  the vane points.
- A **flip vane** does the same, then turns round to face the other way. It
  arrives in the sixth garden.
- A leaf that falls past the last row lands in whatever is under it: home
  if that is a basket of its colour, lost if it is another basket or bare
  ground. A leaf blown off either side is lost, and so is one that goes
  round and round (vanes pointing at each other).
- Leaves pass through each other. Leaves move in the order the tree drops
  them, so two through one flip vane in the same tick turn it twice.
- **Stones** cannot be built on. Leaves fall straight over them.
- Pieces set in a stone base came with the garden. They cannot be moved and
  do not count against you.

With plain vanes two leaves' paths can never cross. If one leaf must end up
left of where another one ends up, but starts to its right, its row of
vanes is in the other's way. The solver found this before I did, and it
is why the flip vane earns its place: the crossing garden is the first one
that needs it.

## The marks

Three per garden, drawn as badges at the bottom; each garden keeps the
best of each mark, even if they came from different solutions.

| Badge | Mark | Earned by |
| --- | --- | --- |
| Basket | Home | Every leaf in its basket. |
| Vane | Few | Home, with no more pieces than the garden's par. |
| Curl | Gentle | Home, with no more wind than the garden's par. Wind is every sideways cell any leaf is blown, summed. |

The two rulers on the left are the same two numbers: pieces placed, and
wind the current plan will blow, each with a leaf on its par. They light
up orange when the plan would earn that mark.

Usually the fewest pieces are also the least wind. Not always: in the
pairs garden the five-piece win blows 32 cells of wind and the gentlest
win needs six pieces for 24. The marks are kept separately so that a
garden like that has two answers worth finding.

## Why it is like this

Written after the reflections on the earlier prototypes, and aimed at what
they say went wrong.

- **Easy to win, hard to win well.** The Opus Magnum review on wirenook.net
  praises puzzles that are simple to solve but scored, so you are never
  stuck but always have something to improve. Small Hours' reflection
  complains of a game that was mind-numbingly easy and then one wall. Here
  "win" is always in reach (a staircase of plain vanes solves most gardens
  by brute force), and the depth is in the marks.
- **Onboarding without words.** The first garden needs only the play
  button, which glows. The second and third show a pulsing ring where the
  first lost leaf's path begins, until you place a piece. Hovering a cell shows a ghost of the piece a
  click would put there, and each click cycles through the kit (right click
  or shift-click goes backwards). Each garden adds one idea.
- **You can see the future.** While building, every leaf's path is drawn as
  a faint dotted trail, ending in a cross where it would be lost. There is
  no guessing what a vane will do; playing is for watching it happen.
- **Small numbers.** Par is 0 to 6 pieces. The biggest garden is 6 by 5.
- **Real sounds.** Every sound is a CC0 recording (leaves, a bamboo swish,
  a cardboard thump, a wooden latch, a kalimba, a bell, a park with birds
  and wind) rather than a synthesiser. Sources and processing are in
  `assets/windfall/SOURCES.md`.
- **A look on purpose.** A cut-paper diorama of an autumn garden: flat
  shapes with soft paper shadows on a cream card, wirenook's creamsicle
  orange as the hero colour, leaves and never flowers. The three leaf
  colours are also three leaf shapes (maple, birch, oak), so they can be
  told apart without colour.

## Controls

| Key | Action |
| --- | --- |
| Click a cell | Place the next piece in the cycle: right vane, left vane, then (later) the two flip vanes, then empty |
| Right click, Shift + click | The cycle backwards |
| Space, or the play cap | Play; press again to stop and go back to building |
| Enter | Play; after a win, the next garden |
| F, or the fast cap | Toggle fast playback |
| Z, Backspace, or the undo cap | Undo |
| C, Delete, or the cross cap | Clear the garden |
| Left / Right, `[` / `]`, 1–9, 0 | Change garden |
| M, or the speaker | Mute |

Clicking the garden during a run stops it, so you can change something.
Plans are kept per garden while the game is open, and marks last as long as
the tab does. Like Small Hours, it touches no storage.

## Code

- `src/windfall/garden.rs` — the rules. `Level`, `Plan` (what the player
  placed), `Run` (one tick per `step`, `Cue`s out), `Marks`.
  `Run::deciding_cells` is what makes the solver exhaustive and still fast.
- `src/windfall/levels.rs` — the eleven gardens. Its tests are the balance
  check: every pieces par is the fewest that wins, the wind pars are the
  least the search finds, and no garden past the lessons is won by
  throwing pieces at random.
- `src/windfall/solve.rs` — `fewest`, `gentlest`, `luck`.
- `src/windfall/sound.rs` — which sample answers which cue, and the gains.
- `src/games/windfall.rs` — input, playback timing, cue reactions;
  `layout` is the one source of positions.
- `src/games/windfall/draw.rs` — everything on screen; `audio.rs` — the
  embedded samples and the ambience loop.
- `examples/windfall_lab.rs` — the solver as a design tool.

```bash
cargo test --lib windfall                                  # rules, pars (bar the slowest), luck
cargo test --release --lib windfall -- --ignored           # the slow proofs: last garden, wind pars
cargo run --release --example windfall_lab                 # every garden's par and the plans
cargo run --release --example windfall_lab -- 8            # just one
cargo run --release --example windfall_lab -- 8 --quick    # just its pieces par
```

## Not known

- Whether the trails make it too easy. They turn the game from "predict
  the machine" into "improve the machine". That is the trade Opus Magnum
  makes, but it is a guess that it suits this game.
- Whether the gentle mark is interesting enough. It agrees with the few
  mark in most gardens. More gardens like the pairs one, where the two
  disagree, would make it count.
- Nobody has listened to the samples on real speakers yet. They were
  chosen by measurement (length, loudness, brightness, the loop seam), not
  by ear.
- The wind par is the least the search finds within one piece of the
  pieces par, not a proof over every plan.
