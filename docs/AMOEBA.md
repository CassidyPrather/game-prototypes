# Amoeba Roguelike Remastered

Looking back on it: [the reflection](https://wirenook.net/blog/reflections/amoeba/) on wirenook.net.

Play as a giant, constantly evolving amoeba and fight off intensifying waves
of humans trying to protect their cities. Engulf and digest them, craft new
organelles and cores from their remains, and destroy the city gates to escape
to the surface.

Pick it from the [main menu](../README.md); `Escape` goes back there once
nothing in the game is open for it to close (the examine cursor, the
organelle list, the settings panel).

A Rust remaster of Cassidy's 2021 [7DRL](https://7drl.com/) entry: the whole
game is a pure, deterministic, unit-tested library (`src/amoeba/sim/`), and
the frontend (`src/games/amoeba/`) is a thin shell that draws it with the
original's CP437 tileset. The original is still on
[itch.io](https://vectis.itch.io/amoeba-roguelike).

The game explains itself. It opens with its own controls, relabels them for
whatever you are holding, describes anything you point the examine cursor at,
and draws every interaction in the cell it happens in — so none of that is
written down here. What is written down here is the part you cannot find out
by playing.

## Where it came from

Amoeba began as its own repository,
[CassidyPrather/amoeba_rl](https://github.com/CassidyPrather/amoeba_rl), and
was moved here whole from its `main` at `f66e95c` (the merge of its pull
request #7). That repository's history was not carried over, and it is
where the original C# version (RogueSharp + RLNET) lives on, in the commits
before the port. [amoeba/PORT_SPEC.md](amoeba/PORT_SPEC.md) is the complete
spec the remaster was built from, including which of the original's quirks
were kept on purpose. Where the remaster departs from it, the code says so at
the point of departure — search for `DELIBERATE CHANGE`.
[amoeba/WISHLIST.md](amoeba/WISHLIST.md) is the original's wishlist, carried
over unedited.

The 7DRL entry and the patches after it shipped as *Amoeba RL*. Enough has
changed since — another language, animation, a layout that fits a phone, a
rebalanced core, a cavern with streets in it and humans with somewhere to be
— that this version takes a name of its own.

### What changed in the move

- The standalone loop became a `games::Game`. It now draws what the turn it
  just resolved left behind, one frame sooner than it used to; nothing else
  about the loop moved.
- It draws across the **whole window**, not inside the shell's letterboxed
  800x600 frame: its layout already fits anything from a portrait phone to a
  desktop, with an on-screen pad when a thumb turns up, and a letterbox would
  undo that. It is the one prototype that does.
- Escape is shared. The game uses it to put things away, so the shell asks
  `Game::wants_escape` before treating it as "leave for the menu".
- The title and post-mortem screens lost their row of links (wirenook.net,
  the source). Inside the prototypes' menu they pointed back at where the
  player already was, and they were the crate's one `unsafe` block plus a
  hand-written js plugin; the crate here forbids `unsafe`.
- Its menu emblem is new: a breathing blob of slime cells against a gate.
- It no longer deploys itself to `wirenook.net/amoeba_rl/`. That path is
  left to the old repository, which now sends visitors here.

## Rules worth knowing before changing it

- `src/amoeba/sim/` is the game, macroquad-free and turn-based: one
  `Command` in per `advance`, a `RenderView` out that has already decided
  every glyph and colour. No clock reaches it; the blink counter and the
  telegraph pulse are arguments.
- The sim reports what happened twice: `Cue`s (a deduplicated set) for
  sound, `Effect`s (the same news in order, with cells attached) for the
  animations in `src/games/amoeba/anim.rs`. Those two and the frame clock are
  the only places allowed to read a wall clock.
- An animation never gets between a keypress and a turn: a command arriving
  mid-animation cuts it short and runs on the next frame.
- It is the one prototype with text on screen. It is a roguelike; the
  message log is the game.

## Commands

```bash
cargo test --lib amoeba                                        # the sim
cargo test --bin game-prototypes amoeba                        # the frontend's layout and input
cargo test --release --lib amoeba -- --ignored                 # whole-playthrough bots
cargo bench --bench amoeba_bench -- --quick                    # a whole playout
```

## Credits

Original game, design, and this port's blueprint: Cassidy Prather.
Extensive playtesting, design, and support on the original from JackNine;
further playtesting from Qu and Decinym. Font: [libtcod](https://github.com/libtcod/libtcod)'s
`terminal12x12_gs_ro.png` (see [CREDITS.md](../CREDITS.md)).
